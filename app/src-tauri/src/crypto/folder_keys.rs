//! folder_keys.rs — Per-folder encryption key management.
//! Each protected folder gets a unique Data Encryption Key (DEK).
//! DEKs are wrapped with the master key and stored in the DB.

use rand::RngCore;
use zeroize::Zeroize;

/// A 32-byte per-folder encryption key. Zeroed on drop.
pub struct FolderKey([u8; 32]);

impl Drop for FolderKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl FolderKey {
    /// Generate a new cryptographically random folder key.
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        Self(key)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
