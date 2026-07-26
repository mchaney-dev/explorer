use std::path::Path;

use rusqlite::Connection;

use crate::db::error::DbResult;

/// Opens (or creates) the database file at `path` and configures it.
pub fn open<P: AsRef<Path>>(path: P) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    Ok(conn)
}

/// Convenience function for unit tests. Opens data in-memory only
pub fn open_in_memory() -> DbResult<Connection> {
    let conn = Connection::open_in_memory()?;
    configure(&conn)?;
    Ok(conn)
}

/// Configures PRAGMAs for the database.
fn configure(conn: &Connection) -> DbResult<()> {
    conn.execute_batch(
        "
        PRAGMA busy_timeout = 5000;
        PRAGMA foreign_keys = ON;
        ",
    )?;
    Ok(())
}
