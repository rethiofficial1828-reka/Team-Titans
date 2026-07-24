//! forensics/timeline_engine.rs — Appends step history to incident timeline.

use crate::db::DbConn;
use rusqlite::params;
use shared::FortiChainError;

pub fn append_step(db_conn: &DbConn, incident_id: &str, label: &str, timestamp: i64) -> Result<(), FortiChainError> {
    let conn = db_conn.lock().map_err(|_| FortiChainError::Internal)?;

    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(step_order), 0) + 1 FROM timeline WHERE incident_id = ?1",
            params![incident_id],
            |row| row.get(0),
        )
        .unwrap_or(1);

    conn.execute(
        "INSERT INTO timeline (incident_id, step_order, label, timestamp, detail) VALUES (?1, ?2, ?3, ?4, NULL)",
        params![incident_id, next_order, label, timestamp],
    )
    .map_err(|_| FortiChainError::Internal)?;

    Ok(())
}
