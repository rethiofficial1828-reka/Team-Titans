//! forensics/db.rs — SQLite database operations for forensics.

use crate::db::DbConn;
use super::models::AttackLogRecord;
use rusqlite::params;
use shared::FortiChainError;

pub fn load_last_hash(db_conn: &DbConn) -> Option<String> {
    let conn = db_conn.lock().ok()?;
    let mut stmt = conn
        .prepare("SELECT sha3_hash FROM attack_logs ORDER BY log_id DESC LIMIT 1")
        .ok()?;

    stmt.query_row([], |row| row.get(0)).ok()
}

pub fn insert_attack_log(db_conn: &DbConn, r: &AttackLogRecord) -> Result<(), FortiChainError> {
    let conn = db_conn.lock().map_err(|_| FortiChainError::Internal)?;
    conn.execute(
        r#"INSERT INTO attack_logs
           (incident_id, timestamp, attack_type, severity, risk_score, username, computer_name,
            process_name, process_id, executable_path, target_folder, target_file,
            action_taken, status, sha3_hash, prev_hash, remarks)
           VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#,
        params![
            r.incident_id,
            r.timestamp,
            r.attack_type,
            r.severity,
            r.risk_score,
            r.username,
            r.computer_name,
            r.process_name,
            r.process_id,
            r.executable_path,
            r.target_folder,
            r.target_file,
            r.action_taken,
            r.status,
            r.sha3_hash,
            r.prev_hash,
            r.remarks
        ],
    ).map_err(|_| FortiChainError::Internal)?;
    Ok(())
}
