//! shared/src/lib.rs
//! Canonical types shared between the Tauri app and the Windows service.
//! No Tauri, no Win32 — compiles on all platforms.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

// ─────────────────────────────────────────────
// ROLES  (fixes: only 2 of 6 roles defined)
// ─────────────────────────────────────────────

/// All user roles in the system. Every role gate check must use this enum —
/// never compare raw strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Role {
    SuperAdmin,
    Admin,
    Investigator,
    Auditor,
    ReadOnly,
    RecoveryAdmin,
}

impl Role {
    /// Returns true if this role may perform write / protection operations.
    pub fn can_write(&self) -> bool {
        matches!(self, Role::SuperAdmin | Role::Admin | Role::Investigator)
    }

    /// Returns true if this role may view full audit detail (not redacted).
    pub fn can_view_full_audit(&self) -> bool {
        matches!(self, Role::SuperAdmin | Role::Admin | Role::Auditor)
    }

    /// Returns true if this role may trigger recovery operations.
    pub fn can_recover(&self) -> bool {
        matches!(self, Role::SuperAdmin | Role::RecoveryAdmin)
    }
}

// ─────────────────────────────────────────────
// SESSION
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionInfo {
    /// 32-byte cryptographically random token, hex-encoded. Stored in memory only — never on disk.
    pub session_id: String,
    pub username: String,
    pub role: Role,
    /// ISO 8601 UTC expiry timestamp.
    pub expires_at: String,
}

// ─────────────────────────────────────────────
// PROTECTED ITEMS
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ProtectedItemState {
    /// No protection active. Default state.
    Idle,
    /// Encryption in progress. Files partially encrypted — crash recovery needed if interrupted.
    Protecting,
    /// Fully encrypted and monitored. Normal operating state.
    Protected,
    /// Decryption in progress to remove protection.
    Unprotecting,
    /// Crash recovery pending — previous `Protecting` run was interrupted.
    CrashRecoveryPending,
    /// File explicitly marked as Read-Only via OS attributes.
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProtectedItem {
    pub id: i64,
    /// Absolute path to the folder/drive being protected.
    pub path: String,
    pub state: ProtectedItemState,
    /// ISO 8601, None while Idle or CrashRecoveryPending.
    pub protected_at: Option<String>,
    /// Display username — never the raw user_id.
    pub protected_by: Option<String>,
    /// Number of files processed in last protect/unprotect job. Used for crash recovery.
    pub files_processed: u32,
    /// Total files counted when the job started. None if not yet computed.
    pub files_total: Option<u32>,
}

// ─────────────────────────────────────────────
// PROTECT JOB PROGRESS
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct JobHandle {
    pub job_id: String,
}

/// Pushed as a Tauri event "protect-progress" — not polled via command.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "status")]
pub enum ProtectJobStatus {
    Running { files_done: u32, files_total: u32 },
    Completed,
    Failed { reason: String },
    LockedFilesSkipped { paths: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProtectProgressEvent {
    pub job_id: String,
    pub item_id: i64,
    pub status: ProtectJobStatus,
}

// ─────────────────────────────────────────────
// AUDIT LOG
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditEntry {
    pub id: i64,
    /// ISO 8601 UTC.
    pub timestamp: String,
    /// Display username — None for system-generated entries.
    pub user: Option<String>,
    pub action: String,
    /// Redacted to None for ReadOnly role (server-side, never client-side).
    pub detail: Option<String>,
    /// SHA3-512 HMAC chain link: HMAC-SHA3-512(prev_hash || entry_bytes).
    /// This field is stored in the DB and verified by `verify_audit_chain`.
    pub chain_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuditFilter {
    /// ISO 8601 inclusive lower bound.
    pub since: Option<String>,
    /// ISO 8601 inclusive upper bound.
    pub until: Option<String>,
    pub action_contains: Option<String>,
    /// Page size. None → default 200. Max 1000 enforced server-side.
    pub limit: Option<u32>,
    /// Cursor-based pagination: the `id` of the last entry from the previous page.
    /// None → start from the beginning (or `since`).
    pub after_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChainVerificationResult {
    pub intact: bool,
    /// `audit_log.id` of the first broken link. None if intact.
    pub mismatch_at: Option<i64>,
    /// Total entries verified.
    pub entries_checked: u64,
}

// ─────────────────────────────────────────────
// KEY MATERIAL
// ─────────────────────────────────────────────

/// Displayed once to the admin after `admin_setup` or `regenerate_recovery_key`.
/// The recovery_key is AES-256-GCM encrypted and stored in the `recovery_keys` DB table.
/// The split_key_parts are the four 4-character segments required for uninstall/unprotect.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct KeyDisplayBundle {
    /// Some only from `admin_setup`. None from `regenerate_recovery_key`.
    /// Length validated at runtime: must be exactly 4 parts, each 4 chars.
    pub split_key_parts: Option<Vec<String>>,
    /// The 32-byte recovery key, base64-encoded for display.
    pub recovery_key: String,
    /// ISO 8601 UTC generation timestamp.
    pub generated_at: String,
}

impl KeyDisplayBundle {
    /// Validates that split_key_parts, if present, contains exactly 4 parts of 4 chars each.
    pub fn validate_split_key(&self) -> Result<(), String> {
        if let Some(ref parts) = self.split_key_parts {
            if parts.len() != 4 {
                return Err(format!(
                    "Expected 4 key parts, got {}",
                    parts.len()
                ));
            }
            for (i, part) in parts.iter().enumerate() {
                if part.chars().count() != 4 {
                    return Err(format!(
                        "Key part {} must be 4 characters, got {}",
                        i + 1,
                        part.chars().count()
                    ));
                }
            }
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────
// SETTINGS
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Settings {
    /// Minutes of inactivity before session expires. Default 15. Matches lockout scale.
    pub session_timeout_minutes: u32,
    /// Maximum failed login attempts before account lockout. Default 3.
    pub max_login_attempts: u32,
    /// Lockout duration in seconds after max_login_attempts exceeded. Default 900 (15 min).
    pub lockout_duration_secs: u64,
    /// Whether to show integrity alerts in the UI immediately. Default true.
    pub realtime_integrity_alerts: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            session_timeout_minutes: 15,
            max_login_attempts: 3,
            lockout_duration_secs: 900,
            realtime_integrity_alerts: true,
        }
    }
}

// ─────────────────────────────────────────────
// IPC ENVELOPE  (app ↔ service)
// ─────────────────────────────────────────────

/// All messages in both directions over the named pipe use this envelope.
/// The nonce is strictly increasing per connection. On reconnect the counter
/// resets to 0, and the service tracks the last-seen nonce per connection
/// handle to detect replays within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEnvelope {
    /// Strictly increasing per connection. Starts at 1. Replay guard.
    pub nonce: u64,
    /// Command name: "watch_folder", "anchor_audit_entry", "integrity_alert", etc.
    pub command: String,
    /// Command-specific payload. Always validated against expected shape server-side.
    pub payload: serde_json::Value,
}

impl IpcEnvelope {
    pub fn new(nonce: u64, command: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            nonce,
            command: command.into(),
            payload,
        }
    }
}

// ─────────────────────────────────────────────
// ERROR TAXONOMY  (fixes: serde wire code mismatch)
// ─────────────────────────────────────────────

/// The single error type returned by every Tauri command and IPC response.
/// Wire codes are pinned via #[serde(rename)] — the Rust variant name is
/// never serialized. A unit test in tests/ verifies the wire encoding.
#[derive(Debug, Clone, Serialize, Deserialize, Error, TS)]
#[ts(export)]
#[serde(tag = "code", content = "detail")]
pub enum FortiChainError {
    #[error("Invalid credentials")]
    #[serde(rename = "AUTH_INVALID_CREDENTIALS")]
    AuthInvalidCredentials,

    #[error("Account locked for {retry_after_secs}s")]
    #[serde(rename = "AUTH_ACCOUNT_LOCKED")]
    AuthAccountLocked { retry_after_secs: u64 },

    #[error("Session expired")]
    #[serde(rename = "AUTH_SESSION_EXPIRED")]
    AuthSessionExpired,

    #[error("Unauthorized — insufficient role")]
    #[serde(rename = "AUTH_UNAUTHORIZED")]
    AuthUnauthorized,

    #[error("Validation failed on field '{field}': {reason}")]
    #[serde(rename = "VALIDATION_FAILED")]
    ValidationFailed { field: String, reason: String },

    #[error("Cryptographic operation failed")]
    #[serde(rename = "CRYPTO_FAILURE")]
    CryptoFailure,
    // NEVER include internal crypto detail in this payload —
    // full error goes to tracing log keyed by correlation_id only.

    #[error("Folder is locked by another operation")]
    #[serde(rename = "FOLDER_LOCKED")]
    FolderLocked,

    #[error("Folder state conflict: current state is '{current_state}'")]
    #[serde(rename = "FOLDER_STATE_CONFLICT")]
    FolderStateConflict { current_state: String },

    #[error("Crash recovery required before this folder can be modified")]
    #[serde(rename = "FOLDER_CRASH_RECOVERY_REQUIRED")]
    FolderCrashRecoveryRequired,

    #[error("Invalid recovery key")]
    #[serde(rename = "RECOVERY_INVALID_KEY")]
    RecoveryInvalidKey,

    #[error("Recovery locked for {retry_after_secs}s")]
    #[serde(rename = "RECOVERY_LOCKED")]
    RecoveryLocked { retry_after_secs: u64 },

    #[error("Invalid key part for removal")]
    #[serde(rename = "REMOVAL_INVALID_KEY_PART")]
    RemovalInvalidKeyPart,

    #[error("Removal locked for {retry_after_secs}s")]
    #[serde(rename = "REMOVAL_LOCKED")]
    RemovalLocked { retry_after_secs: u64 },

    #[error("Windows service is unavailable — named pipe unreachable")]
    #[serde(rename = "SERVICE_UNAVAILABLE")]
    ServiceUnavailable,

    #[error("Resource not found: {resource}")]
    #[serde(rename = "NOT_FOUND")]
    NotFound { resource: String },

    #[error("Internal error (see server log for correlation_id)")]
    #[serde(rename = "INTERNAL_ERROR")]
    Internal,
    // NEVER include the underlying error string in the payload sent to the frontend.
}


// ─────────────────────────────────────────────
// IPC COMMANDS (service-side known commands)
// ─────────────────────────────────────────────

pub mod ipc_commands {
    pub const WATCH_FOLDER: &str = "watch_folder";
    pub const UNWATCH_FOLDER: &str = "unwatch_folder";
    pub const ANCHOR_AUDIT_ENTRY: &str = "anchor_audit_entry";
    pub const INTEGRITY_ALERT: &str = "integrity_alert";
    pub const SERVICE_STATUS: &str = "service_status";
}

/// Resolves or provisions a machine-wide install_id in CommonAppData.
pub fn get_install_id() -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    use rand::RngCore;

    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    let dir = Path::new(&program_data).join("FortiChain");
    let file_path = dir.join("config.json");

    if file_path.exists() {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(id) = json["install_id"].as_str() {
                    return Ok(id.to_string());
                }
            }
        }
    }

    // Provision a new one
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let install_id = hex::encode(bytes);

    let config = serde_json::json!({
        "install_id": install_id
    });

    let config_str = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&file_path, config_str).map_err(|e| e.to_string())?;

    Ok(install_id)
}

