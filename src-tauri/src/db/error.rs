//! Creates a custom error type for SQLite errors.

use std::fmt;

pub type DbResult<T> = Result<T, DbError>;

#[derive(Debug, specta::Type)]
pub enum DbError {
    Sqlite(String),
    NotFound,
    Migration(String),
}

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
        if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
            DbError::NotFound
        } else {
            DbError::Sqlite(e.to_string())
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
