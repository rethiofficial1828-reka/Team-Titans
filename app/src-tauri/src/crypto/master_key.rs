//! master_key.rs — Master key derivation and wrapping using AES-256-GCM.
//! The master key is derived from the user's password via Argon2id.
//! It is NEVER stored in plaintext — only the Argon2id hash is stored in DB.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier, PasswordHash};
use rand::RngCore;
use zeroize::Zeroize;
use shared::FortiChainError;

/// The Argon2id parameters used for key derivation.
/// These are the default Argon2id params (m=19456, t=2, p=1) which are
/// OWASP-recommended for interactive logins.
const ARGON2_SALT_LEN: usize = 16;
const MASTER_KEY_LEN: usize = 32; // 256-bit key for AES-256-GCM

/// A fixed, well-known salt used ONLY for master key derivation (not for password
/// storage). The actual password is stored with a random salt via hash_password().
/// This provides a deterministic key from the same password — required so we can
/// re-derive the key after login to decrypt the recovery key.
///
/// NOTE: This salt is application-specific and constant by design.
/// The security comes from the Argon2id cost parameters + the password's entropy,
/// NOT from the salt's secrecy in this KDF context.
const MASTER_KEY_SALT: &[u8] = b"FortiChain-MasterKey-v1\0\0\0\0\0\0\0\0\0";

/// Derives a 32-byte master key from a password using Argon2id.
/// The key is zeroed when dropped.
pub struct MasterKey([u8; 32]);

impl Drop for MasterKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl MasterKey {
    /// Derive a master key from a password using Argon2id with a fixed application salt.
    /// This is deterministic: same password → same key. Required for decryption after login.
    ///
    /// SECURITY NOTE: The fixed salt is intentional for key derivation (KDF). The
    /// security of the master key relies entirely on the password's entropy and the
    /// Argon2id cost parameters. The key is never stored.
    pub fn derive(password: &str) -> Result<Self, FortiChainError> {
        let mut key_bytes = [0u8; MASTER_KEY_LEN];
        let argon2 = Argon2::default();

        // Use a fixed-length salt (padded to 16 bytes) for deterministic key derivation
        let salt_bytes = &MASTER_KEY_SALT[..ARGON2_SALT_LEN];
        let salt_b64 = base64_encode_salt(salt_bytes);
        let salt = SaltString::from_b64(&salt_b64)
            .map_err(|_| FortiChainError::CryptoFailure)?;

        // Hash the password to get raw Argon2 output
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| FortiChainError::CryptoFailure)?;

        // Extract the raw hash bytes as the key material
        let hash_output = hash.hash.ok_or(FortiChainError::CryptoFailure)?;
        let hash_bytes = hash_output.as_bytes();

        if hash_bytes.len() < MASTER_KEY_LEN {
            return Err(FortiChainError::CryptoFailure);
        }
        key_bytes.copy_from_slice(&hash_bytes[..MASTER_KEY_LEN]);

        Ok(Self(key_bytes))
    }

    /// Hash a password for storage using Argon2id with a random salt. Returns the PHC string.
    /// This is separate from key derivation — uses a random salt for better security.
    pub fn hash_password(password: &str) -> Result<String, FortiChainError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| FortiChainError::CryptoFailure)
    }

    /// Verify a password against a stored Argon2id PHC string hash.
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, FortiChainError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| FortiChainError::CryptoFailure)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Wrap (encrypt) arbitrary bytes using AES-256-GCM.
    /// Output format: [12 bytes nonce | ciphertext + 16 bytes tag]
    pub fn wrap(&self, plaintext: &[u8]) -> Result<Vec<u8>, FortiChainError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.0);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| FortiChainError::CryptoFailure)?;

        // Prepend nonce to ciphertext: [12 bytes nonce | ciphertext]
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Unwrap (decrypt) bytes previously encrypted with `wrap`.
    pub fn unwrap(&self, wrapped: &[u8]) -> Result<Vec<u8>, FortiChainError> {
        if wrapped.len() < 12 {
            return Err(FortiChainError::CryptoFailure);
        }
        let (nonce_bytes, ciphertext) = wrapped.split_at(12);
        let key = Key::<Aes256Gcm>::from_slice(&self.0);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| FortiChainError::CryptoFailure)
    }
}

/// Encode salt bytes as base64 for Argon2's SaltString.
/// Uses the standard base64 alphabet that SaltString::from_b64 expects.
fn base64_encode_salt(bytes: &[u8]) -> String {
    // Argon2's SaltString::from_b64 expects standard (non-URL-safe) base64
    // We implement a simple base64 encoder to avoid adding another dependency
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i + 2 < bytes.len() {
        let b0 = bytes[i] as usize;
        let b1 = bytes[i + 1] as usize;
        let b2 = bytes[i + 2] as usize;
        out.push(CHARS[b0 >> 2] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(CHARS[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        out.push(CHARS[b2 & 0x3f] as char);
        i += 3;
    }
    if i < bytes.len() {
        let b0 = bytes[i] as usize;
        out.push(CHARS[b0 >> 2] as char);
        if i + 1 < bytes.len() {
            let b1 = bytes[i + 1] as usize;
            out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
            out.push(CHARS[(b1 & 0xf) << 2] as char);
        } else {
            out.push(CHARS[(b0 & 3) << 4] as char);
        }
    }
    out
}

/// Generate a new random master key (for testing only — production uses derive()).
#[cfg(test)]
pub fn generate_random() -> MasterKey {
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    MasterKey(key)
}
