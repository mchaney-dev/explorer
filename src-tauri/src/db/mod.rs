//! Handles the database layer.

pub mod connection;
pub mod error;
pub mod migrations;
pub mod repositories;

pub use error::{DbError, DbResult};

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

pub struct Db {
    pub conn: Mutex<Connection>,
}

impl Db {
    /// Open the database at `path` (creating the file if needed) and run any
    /// pending migrations so the schema is current before the app uses it
    pub fn new<P: AsRef<Path>>(path: P) -> DbResult<Self> {
        let conn = connection::open(path)?;
        migrations::run(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Convenience function for unit tests
    pub fn in_memory() -> DbResult<Self> {
        let conn = connection::open_in_memory()?;
        migrations::run(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}
