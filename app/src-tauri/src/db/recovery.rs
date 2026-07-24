//! db/recovery.rs — SQLite operations for recovery keys.

use rusqlite::{params, Connection};
use shared::FortiChainError;

/// Stores an encrypted recovery key for a user.
/// The `encrypted_key_hex` should be the AES-256-GCM wrapped key (nonce || ciphertext) in hex format.
pub fn store_recovery_key(
    conn: &Connection,
    user_id: i64,
    encrypted_key_hex: &str,
) -> Result<(), FortiChainError> {
    // Invalidate any existing keys for this user
    conn.execute(
        "UPDATE recovery_keys SET is_active = 0 WHERE user_id = ?1",
        params![user_id],
    )
    .map_err(|_| FortiChainError::Internal)?;

    // Insert the new key
    conn.execute(
        "INSERT INTO recovery_keys (user_id, encrypted_key) VALUES (?1, ?2)",
        params![user_id, encrypted_key_hex],
    )
    .map_err(|_| FortiChainError::Internal)?;

    Ok(())
}
