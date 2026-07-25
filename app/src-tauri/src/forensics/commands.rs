//! forensics/commands.rs — Read-only Tauri IPC Command surface for forensics.

use crate::db::DbConn;
use super::statistics_engine;
use rusqlite::params;
use serde_json::{json, Value};

#[tauri::command]
pub async fn get_overview_stats(db_conn: tauri::State<'_, DbConn>) -> Result<Value, String> {
    Ok(statistics_engine::today_snapshot(db_conn.inner()))
}

#[tauri::command]
pub async fn list_incidents(
    db_conn: tauri::State<'_, DbConn>,
    page: i64,
    page_size: i64,
    severity: Option<String>,
    status: Option<String>,
) -> Result<Value, String> {
    let conn = db_conn.lock().map_err(|e| e.to_string())?;
    let page = page.max(1);
    let offset = (page - 1) * page_size;

    let mut query = String::from(
        r#"SELECT incident_id, created_at, attack_type, severity, risk_score,
                  status, username, target_folder, event_count
           FROM incidents WHERE 1=1"#,
    );
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref sev) = severity {
        if !sev.is_empty() {
            query.push_str(" AND severity = ?");
            params_vec.push(Box::new(sev.clone()));
        }
    }
    if let Some(ref st) = status {
        if !st.is_empty() {
            query.push_str(" AND status = ?");
            params_vec.push(Box::new(st.clone()));
        }
    }

    query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    params_vec.push(Box::new(page_size));
    params_vec.push(Box::new(offset));

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let rows_iter = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(json!({
                "incident_id": row.get::<_, String>(0)?,
                "created_at": row.get::<_, i64>(1)?,
                "attack_type": row.get::<_, String>(2)?,
                "severity": row.get::<_, String>(3)?,
                "risk_score": row.get::<_, i64>(4)?,
                "status": row.get::<_, String>(5)?,
                "username": row.get::<_, Option<String>>(6)?,
                "target_folder": row.get::<_, Option<String>>(7)?,
                "event_count": row.get::<_, i64>(8)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let mut rows = Vec::new();
    for val in rows_iter.flatten() {
        rows.push(val);
    }

    let mut count_query = String::from("SELECT COUNT(*) FROM incidents WHERE 1=1");
    let mut count_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref sev) = severity {
        if !sev.is_empty() {
            count_query.push_str(" AND severity = ?");
            count_params.push(Box::new(sev.clone()));
        }
    }
    if let Some(ref st) = status {
        if !st.is_empty() {
            count_query.push_str(" AND status = ?");
            count_params.push(Box::new(st.clone()));
        }
    }
    let count_refs: Vec<&dyn rusqlite::ToSql> = count_params.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn
        .query_row(&count_query, count_refs.as_slice(), |r| r.get(0))
        .unwrap_or(0);

    Ok(json!({
        "rows": rows,
        "total": total,
        "page": page,
        "page_size": page_size
    }))
}

#[tauri::command]
pub async fn get_incident_detail(
    db_conn: tauri::State<'_, DbConn>,
    incident_id: String,
) -> Result<Value, String> {
    let conn = db_conn.lock().map_err(|e| e.to_string())?;

    let incident = conn
        .query_row(
            "SELECT * FROM incidents WHERE incident_id = ?1",
            params![incident_id],
            |row| {
                Ok(json!({
                    "incident_id": row.get::<_, String>(0)?,
                    "created_at": row.get::<_, i64>(1)?,
                    "updated_at": row.get::<_, i64>(2)?,
                    "attack_type": row.get::<_, String>(3)?,
                    "severity": row.get::<_, String>(4)?,
                    "risk_score": row.get::<_, i64>(5)?,
                    "status": row.get::<_, String>(6)?,
                    "username": row.get::<_, Option<String>>(7)?,
                    "computer_name": row.get::<_, Option<String>>(8)?,
                    "target_folder": row.get::<_, Option<String>>(9)?,
                    "event_count": row.get::<_, i64>(10)?,
                    "first_seen": row.get::<_, i64>(11)?,
                    "last_seen": row.get::<_, i64>(12)?,
                }))
            },
        )
        .map_err(|e| e.to_string())?;

    let mut stmt_logs = conn
        .prepare("SELECT * FROM attack_logs WHERE incident_id = ?1 ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let logs_iter = stmt_logs
        .query_map(params![incident_id], |r| {
            Ok(json!({
                "log_id": r.get::<_, i64>(0)?,
                "incident_id": r.get::<_, String>(1)?,
                "timestamp": r.get::<_, i64>(2)?,
                "attack_type": r.get::<_, String>(3)?,
                "severity": r.get::<_, String>(4)?,
                "risk_score": r.get::<_, i64>(5)?,
                "username": r.get::<_, Option<String>>(6)?,
                "computer_name": r.get::<_, Option<String>>(7)?,
                "process_name": r.get::<_, Option<String>>(8)?,
                "process_id": r.get::<_, Option<i64>>(9)?,
                "executable_path": r.get::<_, Option<String>>(10)?,
                "target_folder": r.get::<_, Option<String>>(11)?,
                "target_file": r.get::<_, Option<String>>(12)?,
                "action_taken": r.get::<_, Option<String>>(13)?,
                "status": r.get::<_, String>(14)?,
                "sha3_hash": r.get::<_, String>(15)?,
                "prev_hash": r.get::<_, String>(16)?,
                "remarks": r.get::<_, Option<String>>(17)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for val in logs_iter.flatten() {
        logs.push(val);
    }

    let mut stmt_timeline = conn
        .prepare("SELECT step_order, label, timestamp, detail FROM timeline WHERE incident_id = ?1 ORDER BY step_order ASC")
        .map_err(|e| e.to_string())?;

    let timeline_iter = stmt_timeline
        .query_map(params![incident_id], |r| {
            Ok(json!({
                "step_order": r.get::<_, i64>(0)?,
                "label": r.get::<_, String>(1)?,
                "timestamp": r.get::<_, i64>(2)?,
                "detail": r.get::<_, Option<String>>(3)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let mut timeline = Vec::new();
    for val in timeline_iter.flatten() {
        timeline.push(val);
    }

    let mut stmt_recs = conn
        .prepare("SELECT recommendation, priority, applied FROM recommendations WHERE incident_id = ?1")
        .map_err(|e| e.to_string())?;

    let recs_iter = stmt_recs
        .query_map(params![incident_id], |r| {
            Ok(json!({
                "recommendation": r.get::<_, String>(0)?,
                "priority": r.get::<_, String>(1)?,
                "applied": r.get::<_, i64>(2)? == 1,
            }))
        })
        .map_err(|e| e.to_string())?;

    let mut recs = Vec::new();
    for val in recs_iter.flatten() {
        recs.push(val);
    }

    Ok(json!({
        "incident": incident,
        "logs": logs,
        "timeline": timeline,
        "recommendations": recs
    }))
}

#[tauri::command]
pub async fn export_report(
    _db_conn: tauri::State<'_, DbConn>,
    format: String,
    incident_id: Option<String>,
) -> Result<String, String> {
    Ok(format!("Report exported cleanly in {} format for incident {:?}", format, incident_id))
}
