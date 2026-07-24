//! forensics/incident_manager.rs — Incident grouping and FC-YYYY-NNNNNN minting.

use crate::db::DbConn;
use super::models::Severity;
use chrono::Datelike;
use rusqlite::{params, OptionalExtension};

fn severity_rank(s: &str) -> u8 {
    match s {
        "LOW" => 1,
        "MEDIUM" => 2,
        "HIGH" => 3,
        "CRITICAL" => 4,
        _ => 0,
    }
}

pub fn resolve_incident(
    db_conn: &DbConn,
    attack_type: &str,
    username: &Option<String>,
    target_folder: &Option<String>,
    severity: Severity,
) -> String {
    const CORRELATION_WINDOW_MS: i64 = 5 * 60 * 1000; // 5 minutes

    if let Some(existing) = find_open_incident(db_conn, attack_type, username, target_folder, CORRELATION_WINDOW_MS) {
        bump_incident(db_conn, &existing, severity);
        return existing;
    }
    mint_incident_id(db_conn, attack_type, username, target_folder, severity)
}

fn find_open_incident(
    db_conn: &DbConn,
    attack_type: &str,
    username: &Option<String>,
    target_folder: &Option<String>,
    window_ms: i64,
) -> Option<String> {
    let conn = db_conn.lock().ok()?;
    let now = chrono::Utc::now().timestamp_millis();
    let cutoff = now - window_ms;

    let mut stmt = conn
        .prepare(
            r#"SELECT incident_id FROM incidents
               WHERE attack_type = ?1 AND status = 'OPEN'
                 AND (username IS ?2) AND (target_folder IS ?3)
                 AND last_seen >= ?4
               ORDER BY last_seen DESC LIMIT 1"#,
        )
        .ok()?;

    stmt.query_row(params![attack_type, username, target_folder, cutoff], |row| row.get(0)).optional().ok()?
}

fn mint_incident_id(
    db_conn: &DbConn,
    attack_type: &str,
    username: &Option<String>,
    target_folder: &Option<String>,
    severity: Severity,
) -> String {
    let conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return "FC-2026-000001".into(),
    };

    let year = chrono::Utc::now().year();
    let prefix = format!("FC-{}-%", year);

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM incidents WHERE incident_id LIKE ?1", params![prefix], |row| row.get(0))
        .unwrap_or(0);

    let incident_id = format!("FC-{}-{:06}", year, count + 1);
    let now = chrono::Utc::now().timestamp_millis();

    let _ = conn.execute(
        r#"INSERT INTO incidents
           (incident_id, created_at, updated_at, attack_type, severity, risk_score,
            status, username, computer_name, target_folder, event_count, first_seen, last_seen)
           VALUES (?1, ?2, ?2, ?3, ?4, 0, 'OPEN', ?5, NULL, ?6, 1, ?2, ?2)"#,
        params![incident_id, now, attack_type, severity.as_str(), username, target_folder],
    );

    incident_id
}

fn bump_incident(db_conn: &DbConn, incident_id: &str, severity: Severity) {
    let conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return,
    };
    let now = chrono::Utc::now().timestamp_millis();

    let current: String = conn
        .query_row("SELECT severity FROM incidents WHERE incident_id = ?1", params![incident_id], |row| row.get(0))
        .unwrap_or_else(|_| "LOW".into());

    let new_severity = if severity.rank() > severity_rank(&current) {
        severity.as_str()
    } else {
        current.as_str()
    };

    let _ = conn.execute(
        r#"UPDATE incidents SET updated_at = ?1, last_seen = ?1, event_count = event_count + 1,
           severity = ?2 WHERE incident_id = ?3"#,
        params![now, new_severity, incident_id],
    );
}
