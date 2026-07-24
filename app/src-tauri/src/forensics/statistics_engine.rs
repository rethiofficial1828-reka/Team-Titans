//! forensics/statistics_engine.rs — Aggregated daily attack statistics & snapshots.

use crate::db::DbConn;
use super::models::Severity;
use chrono::Utc;
use rusqlite::params;
use serde_json::{json, Value};
use shared::FortiChainError;

pub fn update_today(db_conn: &DbConn, severity: Severity, status: &str) -> Result<(), FortiChainError> {
    let conn = db_conn.lock().map_err(|_| FortiChainError::Internal)?;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let is_critical = (severity == Severity::Critical) as i64;
    let is_blocked = (status == "BLOCKED") as i64;
    let is_allowed = (status == "ALLOWED") as i64;

    conn.execute(
        r#"INSERT INTO statistics (stat_date, total_attacks, critical_attacks, blocked_attacks, allowed_attacks, avg_response_ms)
           VALUES (?1, 1, ?2, ?3, ?4, 0)
           ON CONFLICT(stat_date) DO UPDATE SET
             total_attacks = total_attacks + 1,
             critical_attacks = critical_attacks + ?2,
             blocked_attacks = blocked_attacks + ?3,
             allowed_attacks = allowed_attacks + ?4"#,
        params![today, is_critical, is_blocked, is_allowed],
    )
    .map_err(|_| FortiChainError::Internal)?;

    Ok(())
}

pub fn today_snapshot(db_conn: &DbConn) -> Value {
    let conn = match db_conn.lock() {
        Ok(c) => c,
        Err(_) => return json!({ "totalIncidents": 0, "critical": 0, "blocked": 0, "avgRisk": 0, "protectedFolders": 0 }),
    };

    let today = Utc::now().format("%Y-%m-%d").to_string();

    let (total_attacks, critical_attacks, blocked_attacks): (i64, i64, i64) = conn
        .query_row(
            "SELECT total_attacks, critical_attacks, blocked_attacks FROM statistics WHERE stat_date = ?1",
            params![today],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap_or((0, 0, 0));

    let protected_folders: i64 = conn
        .query_row("SELECT COUNT(DISTINCT path) FROM protected_items", [], |row| row.get(0))
        .unwrap_or(0);

    let cutoff = Utc::now().timestamp_millis() - 86_400_000;
    let avg_risk: f64 = conn
        .query_row(
            "SELECT AVG(risk_score) FROM incidents WHERE created_at >= ?1",
            params![cutoff],
            |row| row.get(0),
        )
        .unwrap_or(Some(0.0))
        .unwrap_or(0.0);

    json!({
        "totalIncidents": total_attacks,
        "critical": critical_attacks,
        "blocked": blocked_attacks,
        "avgRisk": avg_risk.round(),
        "protectedFolders": protected_folders
    })
}
