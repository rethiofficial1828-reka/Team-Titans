//! db/users.rs — SQLite operations for the users table.

use rusqlite::{params, Connection, OptionalExtension};
use shared::{FortiChainError, Role};

pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
}

/// Checks if any user with the `Admin` or `SuperAdmin` role exists in the database.
pub fn admin_exists(conn: &Connection) -> Result<bool, FortiChainError> {
    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM users WHERE role IN ('Admin', 'SuperAdmin') AND is_active = 1")
        .map_err(|_| FortiChainError::Internal)?;

    let count: i64 = stmt
        .query_row([], |row| row.get(0))
        .map_err(|_| FortiChainError::Internal)?;

    Ok(count > 0)
}

/// Inserts a new user into the database. Returns the newly created user ID.
pub fn create_user(
    conn: &Connection,
    username: &str,
    password_hash: &str,
    role: Role,
) -> Result<i64, FortiChainError> {
    // Role must be converted to string for SQLite
    let role_str = match role {
        Role::SuperAdmin => "SuperAdmin",
        Role::Admin => "Admin",
        Role::Investigator => "Investigator",
        Role::Auditor => "Auditor",
        Role::ReadOnly => "ReadOnly",
        Role::RecoveryAdmin => "RecoveryAdmin",
    };

    let result = conn.execute(
        "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
        params![username, password_hash, role_str],
    );

    match result {
        Ok(_) => Ok(conn.last_insert_rowid()),
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.code == rusqlite::ErrorCode::ConstraintViolation => {
            Err(FortiChainError::ValidationFailed {
                field: "username".into(),
                reason: "Username already exists".into(),
            })
        }
        Err(_) => Err(FortiChainError::Internal),
    }
}

/// Fetches a user by username. Returns None if not found.
pub fn get_user_by_username(
    conn: &Connection,
    username: &str,
) -> Result<Option<UserRecord>, FortiChainError> {
    let mut stmt = conn
        .prepare("SELECT id, username, password_hash, role FROM users WHERE username = ?1 AND is_active = 1")
        .map_err(|_| FortiChainError::Internal)?;

    let user = stmt
        .query_row(params![username], |row| {
            let role_str: String = row.get(3)?;
            let role = match role_str.as_str() {
                "SuperAdmin" => Role::SuperAdmin,
                "Admin" => Role::Admin,
                "Investigator" => Role::Investigator,
                "Auditor" => Role::Auditor,
                "ReadOnly" => Role::ReadOnly,
                "RecoveryAdmin" => Role::RecoveryAdmin,
                _ => Role::ReadOnly, // Fallback
            };

            Ok(UserRecord {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                role,
            })
        })
        .optional()
        .map_err(|_| FortiChainError::Internal)?;

    Ok(user)
}
