//! forensics/event_manager.rs — Event Gateway and async background worker.

use crate::db::DbConn;
use super::{
    db, hash, incident_manager,
    models::{AttackLogRecord, RawAttackEvent},
    recommendation_engine, risk_engine, statistics_engine, threat_analyzer, timeline_engine,
};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc::UnboundedSender, OnceCell};

pub struct EventManagerHandle {
    pub tx: UnboundedSender<RawAttackEvent>,
}

pub static EVENT_MANAGER: OnceCell<EventManagerHandle> = OnceCell::const_new();

pub fn submit_event(event: RawAttackEvent) {
    if let Some(handle) = EVENT_MANAGER.get() {
        let _ = handle.tx.send(event);
    } else {
        eprintln!("[forensics] event manager not initialized yet, dropping event");
    }
}

pub fn spawn_worker(app: AppHandle, db_conn: DbConn) -> EventManagerHandle {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<RawAttackEvent>();

    let db_conn_clone = db_conn.clone();
    tauri::async_runtime::spawn(async move {
        let mut last_hash = db::load_last_hash(&db_conn_clone)
            .unwrap_or_else(|| hash::GENESIS_HASH.to_string());

        while let Some(raw) = rx.recv().await {
            if raw.source_module.is_empty() {
                continue;
            }

            let attack_type = threat_analyzer::classify(&raw);
            let (severity, risk_score) = risk_engine::score(&attack_type, &raw);

            if severity == crate::forensics::models::Severity::Critical {
                if let Some(pid) = raw.process_id {
                    match crate::deception::containment::FrozenAttacker::contain(pid as u32) {
                        Ok(frozen) => {
                            eprintln!("[forensics] Successfully froze attacker process {}", pid);
                            // Terminate immediately to stop ransomware completely
                            frozen.terminate();
                        }
                        Err(e) => {
                            eprintln!("[forensics] Failed to freeze attacker {}: {}", pid, e);
                        }
                    }
                }
            }

            let incident_id = incident_manager::resolve_incident(
                &db_conn_clone,
                &attack_type,
                &raw.username,
                &raw.target_folder,
                severity,
            );

            let timestamp = chrono::Utc::now().timestamp_millis();
            let status = raw.action_taken.clone().unwrap_or_else(|| "OPEN".into());
            let payload = format!(
                "{}{}{}{}{}",
                incident_id,
                timestamp,
                attack_type,
                severity.as_str(),
                risk_score
            );
            let sha3_hash = hash::hash_record(&payload, &last_hash);

            let record = AttackLogRecord {
                incident_id: incident_id.clone(),
                timestamp,
                attack_type: attack_type.clone(),
                severity: severity.as_str().to_string(),
                risk_score,
                username: raw.username.clone(),
                computer_name: raw.computer_name.clone(),
                process_name: raw.process_name.clone(),
                process_id: raw.process_id,
                executable_path: raw.executable_path.clone(),
                target_folder: raw.target_folder.clone(),
                target_file: raw.target_file.clone(),
                action_taken: raw.action_taken.clone(),
                status: status.clone(),
                sha3_hash: sha3_hash.clone(),
                prev_hash: last_hash.clone(),
                remarks: raw.remarks.clone(),
            };
            last_hash = sha3_hash;

            if let Err(e) = db::insert_attack_log(&db_conn_clone, &record) {
                eprintln!("[forensics] insert failed: {:?}", e);
                continue;
            }

            let _ = timeline_engine::append_step(&db_conn_clone, &incident_id, "Detected", timestamp);
            let _ = recommendation_engine::generate(&db_conn_clone, &incident_id, &attack_type, severity);
            let _ = statistics_engine::update_today(&db_conn_clone, severity, &status);

            let _ = app.emit("forensics://new-event", &record);
        }
    });

    EventManagerHandle { tx }
}
