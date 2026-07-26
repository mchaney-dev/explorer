//! Creates a custom error type for SQLite errors.

use std::fmt;

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    Sqlite(rusqlite::Error),
    NotFound,
    Migration(String),
}

/// Format depending on error type
impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Sqlite(e) => write!(f, "database error: {e}"),
            DbError::NotFound => write!(f, "record not found"),
            DbError::Migration(msg) => write!(f, "migration failed: {msg}"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        // transform QueryReturnedNoRows error to more readable NotFound
        if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
            DbError::NotFound
        } else {
            // otherwise, use generic Sqlite error
            DbError::Sqlite(e)
        }
    }
}

/// Serialize the result to string so it can be passed to the frontend
impl serde::Serialize for DbError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
