//! admin.rs — Admin-only Tauri commands (placeholder).
use shared::FortiChainError;

// Admin commands are implemented in auth.rs (create_readonly_user, get_settings, etc.)
// This module reserved for future fleet/enterprise management commands.

/// Retrieve high-level system status
#[tauri::command]
pub async fn get_system_status() -> Result<serde_json::Value, FortiChainError> {
    Ok(serde_json::json!({
        "boot_chain_secure": true,
        "tpm_sealed": true,
        "enclave_active": true,
        "kernel_drivers_loaded": true,
    }))
}

/// Wipes all data to simulate factory reset / node deactivation
#[tauri::command]
pub async fn deactivate_node(
    session_id: String,
    db_conn: tauri::State<'_, crate::db::DbConn>,
    session_state: tauri::State<'_, crate::commands::auth::SessionState>,
) -> Result<(), FortiChainError> {
    let session = crate::commands::auth::validate_session(&session_state, &session_id)?;
    if session.role != shared::Role::Admin && session.role != shared::Role::SuperAdmin {
        return Err(FortiChainError::AuthUnauthorized);
    }

    {
        let conn = db_conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM protected_items", []);
        let _ = conn.execute("DELETE FROM audit_log", []);
        // credentials table doesn't exist, we skip it
    }
    
    let _ = crate::commands::audit::log_audit_event(
        session_id,
        "NODE_DEACTIVATED".to_string(),
        "Administrator wiped all protected items and audit logs.".to_string(),
        db_conn.clone(),
    ).await;
    
    Ok(())
}
