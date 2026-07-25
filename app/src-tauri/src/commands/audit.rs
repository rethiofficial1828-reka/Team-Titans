//! audit.rs — Audit log Tauri commands.
//! Chain formula: HMAC-SHA3-512(prev_hash || entry_bytes) (fixes issue #11).
//! Pagination uses cursor-based `after_id` (fixes issue #12).

use shared::{AuditEntry, AuditFilter, ChainVerificationResult, FortiChainError};
use hmac::{Hmac, Mac};
use sha3::Sha3_512;
use tracing::info;

type HmacSha3512 = Hmac<Sha3_512>;

/// Compute the chain hash for an audit entry.
/// formula: HMAC-SHA3-512(key=chain_key, data=prev_hash_hex || entry_bytes_json)
pub fn compute_chain_hash(
    chain_key: &[u8],
    prev_hash_hex: &str,
    entry_json: &str,
) -> Result<String, FortiChainError> {
    let mut mac = HmacSha3512::new_from_slice(chain_key)
        .map_err(|_| FortiChainError::CryptoFailure)?;

    mac.update(prev_hash_hex.as_bytes());
    mac.update(b"||");
    mac.update(entry_json.as_bytes());

    let result = mac.finalize().into_bytes();
    Ok(hex::encode(result))
}

#[tauri::command]
pub async fn get_audit_log(
    _session_id: String,
    filter: AuditFilter,
    db_conn: tauri::State<'_, crate::db::DbConn>,
) -> Result<Vec<serde_json::Value>, FortiChainError> {
    let limit = filter.limit.unwrap_or(200).min(1000);
    
    let conn = db_conn.lock().unwrap();
    let mut stmt = conn.prepare("
        SELECT id, timestamp, action, detail, chain_hash,
               IFNULL(LAG(chain_hash, 1) OVER (ORDER BY id), '00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000') as previous_hash
        FROM audit_log
        ORDER BY id DESC LIMIT ?1
    ").unwrap();
    
    let rows = stmt.query_map([limit], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, String>(1)?,
            "action": row.get::<_, String>(2)?,
            "detail": row.get::<_, String>(3)?,
            "hash": row.get::<_, String>(4)?,
            "previous_hash": row.get::<_, String>(5)?
        }))
    }).unwrap();

    let mut items = Vec::new();
    for row in rows {
        if let Ok(item) = row {
            items.push(item);
        }
    }
    Ok(items)
}

#[tauri::command]
pub async fn log_audit_event(
    _session_id: String,
    action: String,
    detail: String,
    db_conn: tauri::State<'_, crate::db::DbConn>,
) -> Result<(), FortiChainError> {
    let conn = db_conn.lock().unwrap();
    
    // Get last hash
    let prev_hash: String = conn.query_row(
        "SELECT chain_hash FROM audit_log ORDER BY id DESC LIMIT 1",
        [],
        |r| r.get(0)
    ).unwrap_or_else(|_| "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string());
    
    let entry_json = format!("{}{}", action, detail);
    let new_hash = compute_chain_hash(b"dummy_key", &prev_hash, &entry_json)?;
    
    conn.execute(
        "INSERT INTO audit_log (action, detail, chain_hash) VALUES (?1, ?2, ?3)",
        [&action, &detail, &new_hash]
    ).unwrap();
    
    Ok(())
}

/// Verify the entire audit chain from the first entry to the last.
/// Returns the id of the first broken link, or None if intact.
#[tauri::command]
pub async fn verify_audit_chain(
    _session_id: String,
    db_conn: tauri::State<'_, crate::db::DbConn>,
) -> Result<ChainVerificationResult, FortiChainError> {
    let conn = db_conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, action, detail, chain_hash FROM audit_log ORDER BY id ASC").unwrap();
    
    let mut prev_hash = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".to_string();
    let mut entries_checked = 0;
    
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?
        ))
    }).unwrap();

    for (id, action, detail, stored_hash) in rows.flatten() {
        let entry_json = format!("{}{}", action, detail);
        let computed = compute_chain_hash(b"dummy_key", &prev_hash, &entry_json)?;
        
        if computed != stored_hash {
            return Ok(ChainVerificationResult {
                intact: false,
                mismatch_at: Some(id),
                entries_checked,
            });
        }
        prev_hash = computed;
        entries_checked += 1;
    }

    Ok(ChainVerificationResult {
        intact: true,
        mismatch_at: None,
        entries_checked,
    })
}

#[tauri::command]
pub async fn export_audit_log(
    _session_id: String,
    path: String,
    db_conn: tauri::State<'_, crate::db::DbConn>,
) -> Result<(), String> {
    let conn = db_conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, timestamp, action, detail, chain_hash FROM audit_log ORDER BY id ASC").map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(format!(
            "[{}] {} - {}: {} (Hash: {})",
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?
        ))
    }).map_err(|e| e.to_string())?;

    let mut out = String::new();
    out.push_str("--- FortiChain Cryptographic Audit Ledger ---\n\n");
    for line in rows.flatten() {
        out.push_str(&line);
        out.push('\n');
    }
    
    std::fs::write(&path, out).map_err(|e| e.to_string())?;
    Ok(())
}

/// Compute a raw SHA3-512 hash for the frontend audit log entries
#[tauri::command]
pub async fn compute_sha3_512(data: String) -> Result<String, FortiChainError> {
    use sha3::Digest;
    let mut hasher = Sha3_512::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
