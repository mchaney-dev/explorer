use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::feed::feed_source::{FeedSource, SourceType};

pub(crate) fn source_type_to_db(t: &SourceType) -> &'static str {
    match t {
        SourceType::Text => "text",
        SourceType::Image => "image",
        SourceType::Video => "video",
        SourceType::Audio => "audio",
    }
}

pub(crate) fn source_type_from_db(value: Option<String>) -> Option<SourceType> {
    match value.as_deref() {
        Some("text") => Some(SourceType::Text),
        Some("image") => Some(SourceType::Image),
        Some("video") => Some(SourceType::Video),
        Some("audio") => Some(SourceType::Audio),
        _ => None,
    }
}

pub(crate) fn row_to_source(row: &Row) -> rusqlite::Result<FeedSource> {
    Ok(FeedSource {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        source_type: source_type_from_db(row.get("source_type")?),
        name: row.get("name")?,
        base_url: row.get("base_url")?,
        is_enabled: row.get("is_enabled")?,
        requires_key: row.get("requires_key")?,
    })
}

/// Insert a source, or leave the existing row untouched if the id already exists.
pub(crate) fn upsert(conn: &Connection, source: &FeedSource) -> DbResult<()> {
    conn.execute(
        "INSERT INTO feed_sources
            (id, created_at, updated_at, source_type, name, base_url, is_enabled, requires_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO NOTHING",
        rusqlite::params![
            source.id,
            source.created_at,
            source.updated_at,
            source.source_type.as_ref().map(source_type_to_db),
            source.name,
            source.base_url,
            source.is_enabled,
            source.requires_key,
        ],
    )?;
    Ok(())
}

pub fn insert(conn: &Connection, source: &FeedSource) -> DbResult<()> {
    conn.execute(
        "INSERT INTO feed_sources
            (id, created_at, updated_at, source_type, name, base_url, is_enabled, requires_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            source.id,
            source.created_at,
            source.updated_at,
            source.source_type.as_ref().map(source_type_to_db),
            source.name,
            source.base_url,
            source.is_enabled,
            source.requires_key,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, source: &FeedSource) -> DbResult<()> {
    let affected = conn.execute(
        "UPDATE feed_sources SET
            updated_at = ?2, source_type = ?3, name = ?4, base_url = ?5,
            is_enabled = ?6, requires_key = ?7
          WHERE id = ?1",
        rusqlite::params![
            source.id,
            source.updated_at,
            source.source_type.as_ref().map(source_type_to_db),
            source.name,
            source.base_url,
            source.is_enabled,
            source.requires_key,
        ],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<FeedSource> {
    Ok(conn.query_row(
        "SELECT * FROM feed_sources WHERE id = ?1",
        [id],
        row_to_source,
    )?)
}

pub fn list(conn: &Connection) -> DbResult<Vec<FeedSource>> {
    let mut stmt = conn.prepare("SELECT * FROM feed_sources ORDER BY id DESC")?;
    let rows = stmt.query_map([], row_to_source)?;
    Ok(rows.collect::<rusqlite::Result<Vec<FeedSource>>>()?)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM feed_sources WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    // TC-DBFEED-001
    #[test]
    fn insert_then_get_roundtrips() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let source = FeedSource::new("Blog", "https://blog.example");
        insert(&conn, &source).unwrap();
        let got = get(&conn, &source.id).unwrap();
        assert_eq!(got.name, "Blog");
        assert_eq!(got.base_url, "https://blog.example");
    }

    // TC-DBFEED-002
    #[test]
    fn source_type_persists() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let source = FeedSource::new("B", "u").with_type(SourceType::Video);
        insert(&conn, &source).unwrap();
        assert!(matches!(
            get(&conn, &source.id).unwrap().source_type,
            Some(SourceType::Video)
        ));
    }

    // TC-DBFEED-003
    #[test]
    fn upsert_is_idempotent() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let source = FeedSource::new("B", "u");
        insert(&conn, &source).unwrap();
        upsert(&conn, &source).unwrap(); // second write of the same id must not error
        assert_eq!(list(&conn).unwrap().len(), 1);
    }

    // TC-DBFEED-004
    #[test]
    fn delete_removes_source() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let source = FeedSource::new("B", "u");
        insert(&conn, &source).unwrap();
        delete(&conn, &source.id).unwrap();
        assert!(get(&conn, &source.id).is_err());
    }
}
