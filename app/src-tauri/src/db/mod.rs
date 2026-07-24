pub mod migrations;
pub mod users;
pub mod recovery;

use rusqlite::Connection;
use shared::FortiChainError;
use std::sync::{Arc, Mutex};

pub type DbConn = Arc<Mutex<Connection>>;

/// Initialize the database: open, run migrations, return the connection.
pub fn init(db_path: &str) -> Result<DbConn, FortiChainError> {
    let conn = Connection::open(db_path)
        .map_err(|_| FortiChainError::Internal)?;

    // Enable WAL mode for concurrent reads during writes
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|_| FortiChainError::Internal)?;

    migrations::run(&conn)?;

    Ok(Arc::new(Mutex::new(conn)))
}
