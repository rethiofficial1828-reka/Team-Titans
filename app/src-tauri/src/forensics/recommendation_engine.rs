//! forensics/recommendation_engine.rs — Action suggestions for incidents.

use crate::db::DbConn;
use super::models::Severity;
use rusqlite::params;
use shared::FortiChainError;

pub fn generate(db_conn: &DbConn, incident_id: &str, attack_type: &str, severity: Severity) -> Result<(), FortiChainError> {
    let conn = db_conn.lock().map_err(|_| FortiChainError::Internal)?;

    let recs: Vec<(&str, &str)> = match (attack_type, severity) {
        (_, Severity::Critical) => vec![("Disconnect Network", "HIGH"), ("Terminate Process", "HIGH"), ("Notify Administrator", "HIGH")],
        ("Encryption Attempt", _) => vec![("Lock Folder", "HIGH"), ("Run Full Scan", "MEDIUM")],
        ("Mass Rename", _) => vec![("Lock Folder", "HIGH"), ("Backup Data", "MEDIUM")],
        _ => vec![("Run Full Scan", "LOW"), ("Notify Administrator", "LOW")],
    };

    for (text, priority) in recs {
        let _ = conn.execute(
            "INSERT INTO recommendations (incident_id, recommendation, priority, applied) VALUES (?1, ?2, ?3, 0)",
            params![incident_id, text, priority],
        );
    }

    Ok(())
}
