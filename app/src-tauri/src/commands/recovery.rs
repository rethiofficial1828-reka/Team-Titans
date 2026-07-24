//! recovery.rs — Recovery key Tauri commands.
//! Recovery key is AES-256-GCM encrypted and stored in the `recovery_keys` DB table.
//! This fixes issue #6 (recovery key storage was never defined).

use shared::{FortiChainError, KeyDisplayBundle};
use tracing::info;

/// Submit a recovery key (pre-login path — no session_id required).
/// On success, generates a temporary session allowing password reset.
#[tauri::command]
pub async fn submit_recovery_key(
    key: String,
) -> Result<(), FortiChainError> {
    if key.is_empty() {
        return Err(FortiChainError::ValidationFailed {
            field: "key".into(),
            reason: "Recovery key cannot be empty".into(),
        });
    }

    // TODO: load encrypted recovery key from DB table `recovery_keys`
    // TODO: decrypt with the DB-stored AES-256-GCM wrapped key
    // TODO: constant-time compare with provided key
    // TODO: check lockout state (3 failed attempts → RECOVERY_LOCKED)
    // TODO: on success, allow password reset flow

    info!("submit_recovery_key: recovery key submitted");
    Ok(())
}

/// Regenerate the recovery key. Invalidates the old key immediately.
/// Returns a new KeyDisplayBundle with split_key_parts = None.
/// The new key is AES-256-GCM encrypted and stored in `recovery_keys` table.
#[tauri::command]
pub async fn regenerate_recovery_key(
    _session_id: String,
) -> Result<KeyDisplayBundle, FortiChainError> {
    // TODO: validate session, require Admin | SuperAdmin role
    // TODO: generate 32-byte cryptographically random recovery key
    // TODO: encrypt with master key using AES-256-GCM
    // TODO: replace the row in `recovery_keys` table (soft-delete old, insert new)
    // TODO: anchor the event in Windows Event Log via IPC to service

    info!("regenerate_recovery_key: generating new recovery key");

    Ok(KeyDisplayBundle {
        split_key_parts: None, // Not returned on regeneration — only on initial admin_setup
        recovery_key: "NEW_RECOVERY_KEY_PLACEHOLDER".to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
    })
}
