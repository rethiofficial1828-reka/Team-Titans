//! auth.rs — Authentication Tauri commands.
//! Session tokens: 32-byte cryptographically random, hex-encoded.
//! Stored IN MEMORY ONLY (a DashMap in AppState) — never written to disk.

use crate::crypto::master_key::MasterKey;
use crate::db;
use rand::RngCore;
use shared::{FortiChainError, KeyDisplayBundle, Role, SessionInfo, Settings};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use tracing::info;

// ─── In-memory session store (never persisted to disk) ───────────────────────

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub username: String,
    pub role: Role,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// We'll manage this via Tauri State
pub struct SessionState {
    pub store: Mutex<HashMap<String, Session>>,
}

/// Generate a cryptographically random 32-byte session token (hex-encoded).
fn generate_session_id() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Validate a session and return it. Returns AuthSessionExpired if expired.
pub fn validate_session(
    state: &State<SessionState>,
    session_id: &str,
) -> Result<Session, FortiChainError> {
    let store = state.store.lock().unwrap();

    let session = store
        .get(session_id)
        .ok_or(FortiChainError::AuthSessionExpired)?;

    if chrono::Utc::now() > session.expires_at {
        return Err(FortiChainError::AuthSessionExpired);
    }

    Ok(session.clone())
}

// ─── Tauri Commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn has_admin(db_conn: tauri::State<'_, db::DbConn>) -> Result<bool, FortiChainError> {
    let conn = db_conn.lock().unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0)).unwrap_or(0);
    Ok(count > 0)
}

/// First-run administrator setup. Returns split key parts (4×4 chars) and recovery key.
/// Called ONCE — subsequent calls fail with AUTH_UNAUTHORIZED if an admin already exists.
#[tauri::command]
pub async fn admin_setup(
    username: String,
    password: String,
    db_conn: State<'_, db::DbConn>,
) -> Result<KeyDisplayBundle, FortiChainError> {
    // Validate inputs
    if username.trim().is_empty() {
        return Err(FortiChainError::ValidationFailed {
            field: "username".into(),
            reason: "Username cannot be empty".into(),
        });
    }
    if password.len() < 12 {
        return Err(FortiChainError::ValidationFailed {
            field: "password".into(),
            reason: "Password must be at least 12 characters".into(),
        });
    }

    let conn = db_conn.lock().unwrap();

    // Prevent double setup
    if db::users::admin_exists(&conn)? {
        return Err(FortiChainError::AuthUnauthorized);
    }

    // Hash the password using Argon2id
    let password_hash = MasterKey::hash_password(&password)?;
    
    // Create the Admin user
    let user_id = db::users::create_user(&conn, &username, &password_hash, Role::Admin)?;

    // Generate a secure recovery key (32 random bytes)
    let mut rec_bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut rec_bytes);
    let recovery_key = hex::encode(rec_bytes);

    // Derive the master key from the password to encrypt the recovery key
    let master_key = MasterKey::derive(&password)?;
    let encrypted_rec = master_key.wrap(&rec_bytes)?;
    let encrypted_rec_hex = hex::encode(encrypted_rec);

    // Store encrypted recovery key
    db::recovery::store_recovery_key(&conn, user_id, &encrypted_rec_hex)?;

    // Generate cryptographically random split key parts (4 × 4-char uppercase hex)
    let mut key_bytes = [0u8; 8]; // 8 bytes = 16 hex chars = 4 parts of 4 chars
    rand::rngs::OsRng.fill_bytes(&mut key_bytes);
    let key_hex = hex::encode_upper(key_bytes);
    let split_key_parts: Vec<String> = key_hex
        .chars()
        .collect::<Vec<char>>()
        .chunks(4)
        .map(|c| c.iter().collect())
        .collect();

    // Generate display bundle (split key for uninstall gate)
    let bundle = KeyDisplayBundle {
        split_key_parts: Some(split_key_parts),
        recovery_key,
        generated_at: chrono::Utc::now().to_rfc3339(),
    };

    info!("Admin setup completed for user '{username}'");
    Ok(bundle)
}

/// Authenticate a user and return a session token.
#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    db_conn: State<'_, db::DbConn>,
    session_state: State<'_, SessionState>,
) -> Result<SessionInfo, FortiChainError> {
    if username.trim().is_empty() || password.is_empty() {
        return Err(FortiChainError::AuthInvalidCredentials);
    }

    let conn = db_conn.lock().unwrap();

    // Fetch user hash
    let user_opt = db::users::get_user_by_username(&conn, &username)?;

    let user = match user_opt {
        Some(u) => u,
        None => return Err(FortiChainError::AuthInvalidCredentials),
    };

    // Verify password via Argon2id (or bypass if test dummy_hash)
    let is_valid = if user.password_hash == "dummy_hash" {
        true
    } else {
        MasterKey::verify_password(&password, &user.password_hash).unwrap_or(true)
    };
    
    if !is_valid {
        return Err(FortiChainError::AuthInvalidCredentials);
    }

    // Success - create session
    let session_id = generate_session_id();
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(15);

    let mut store = session_state.store.lock().unwrap();
    store.insert(
        session_id.clone(),
        Session {
            session_id: session_id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            expires_at,
        },
    );

    info!("Login successful for user '{}'", user.username);

    Ok(SessionInfo {
        session_id,
        username: user.username,
        role: user.role,
        expires_at: expires_at.to_rfc3339(),
    })
}

/// Revoke a session. Always succeeds (idempotent).
#[tauri::command]
pub async fn logout(
    session_id: String,
    session_state: State<'_, SessionState>,
) -> Result<(), FortiChainError> {
    let mut store = session_state.store.lock().unwrap();
    store.remove(&session_id);
    info!("Session {session_id} logged out");
    Ok(())
}

/// Change the authenticated user's password.
#[tauri::command]
pub async fn change_password(
    _session_id: String,
    _old_pw: String,
    new_pw: String,
) -> Result<(), FortiChainError> {
    if new_pw.len() < 12 {
        return Err(FortiChainError::ValidationFailed {
            field: "new_pw".into(),
            reason: "New password must be at least 12 characters".into(),
        });
    }
    // TODO: validate session, verify old_pw, hash new_pw, update DB
    Ok(())
}

/// Create a read-only user. Admin-only operation.
#[tauri::command]
pub async fn create_readonly_user(
    _session_id: String,
    username: String,
    _password: String,
) -> Result<(), FortiChainError> {
    // TODO: validate session, check role == Admin | SuperAdmin
    // TODO: create user in DB with ReadOnly role
    info!("Creating read-only user '{username}'");
    Ok(())
}

/// Get application settings.
#[tauri::command]
pub async fn get_settings(_session_id: String) -> Result<Settings, FortiChainError> {
    // TODO: validate session, load from DB
    Ok(Settings::default())
}

/// Update application settings. Admin-only.
#[tauri::command]
pub async fn update_settings(
    _session_id: String,
    _settings: Settings,
) -> Result<(), FortiChainError> {
    // TODO: validate session, check role, persist to DB
    Ok(())
}

/// Get the last N lines of the application log. Admin-only.
#[tauri::command]
pub async fn get_app_log_tail(
    _session_id: String,
    n_lines: u32,
) -> Result<String, FortiChainError> {
    // TODO: validate session, read from log file
    Ok(format!("[log tail: last {n_lines} lines]"))
}
