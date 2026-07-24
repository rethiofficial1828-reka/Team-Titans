use rusqlite::Connection;
use shared::FortiChainError;

pub fn run(conn: &Connection) -> Result<(), FortiChainError> {
    conn.execute_batch(include_str!("migrations/001_initial.sql"))
        .map_err(|_| FortiChainError::Internal)?;

    let version: i32 = conn.query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0)).unwrap_or(1);
    if version < 2 {
        conn.execute_batch(include_str!("migrations/002_update_check.sql"))
            .map_err(|_| FortiChainError::Internal)?;
        conn.execute("INSERT INTO schema_version (version) VALUES (2)", [])
            .map_err(|_| FortiChainError::Internal)?;
    }
    if version < 3 {
        conn.execute_batch(include_str!("migrations/003_forensics_schema.sql"))
            .map_err(|_| FortiChainError::Internal)?;
        conn.execute("INSERT INTO schema_version (version) VALUES (3)", [])
            .map_err(|_| FortiChainError::Internal)?;
    }

    Ok(())
}
