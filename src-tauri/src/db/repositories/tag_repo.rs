use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::tag::Tag;

/// Turn one database row into a [`Tag`].
fn row_to_tag(row: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        label: row.get("label")?,
        color: row.get("color")?,
    })
}

/// Insert a brand-new tag.
pub fn insert(conn: &Connection, tag: &Tag) -> DbResult<()> {
    conn.execute(
        "INSERT INTO tags (id, created_at, updated_at, label, color)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![tag.id, tag.created_at, tag.updated_at, tag.label, tag.color],
    )?;
    Ok(())
}

/// Overwrite an existing tag's mutable columns, matched by id.
pub fn update(conn: &Connection, tag: &Tag) -> DbResult<()> {
    let affected = conn.execute(
        "UPDATE tags
            SET updated_at = ?2, label = ?3, color = ?4
          WHERE id = ?1",
        rusqlite::params![tag.id, tag.updated_at, tag.label, tag.color],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Fetch a single tag by id. Returns [`DbError::NotFound`] if there's no match
pub fn get(conn: &Connection, id: &str) -> DbResult<Tag> {
    let tag = conn.query_row(
        "SELECT id, created_at, updated_at, label, color FROM tags WHERE id = ?1",
        [id],
        row_to_tag,
    )?;
    Ok(tag)
}

/// Fetch every tag, newest first.
pub fn list(conn: &Connection) -> DbResult<Vec<Tag>> {
    let mut stmt =
        conn.prepare("SELECT id, created_at, updated_at, label, color FROM tags ORDER BY id DESC")?;
    let rows = stmt.query_map([], row_to_tag)?;

    let tags = rows.collect::<rusqlite::Result<Vec<Tag>>>()?;
    Ok(tags)
}

/// Delete a tag by id.
pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    let affected = conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    // TC-DBTAG-001
    #[test]
    fn insert_then_get_roundtrips() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let tag = Tag::new("Work", Some("#ffffff".to_string()));
        insert(&conn, &tag).unwrap();
        let got = get(&conn, &tag.id).unwrap();
        assert_eq!(got.label, "Work");
        assert_eq!(got.color.as_deref(), Some("#ffffff"));
    }

    // TC-DBTAG-002
    #[test]
    fn update_persists_changes() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut tag = Tag::new("Work", None);
        insert(&conn, &tag).unwrap();
        tag.set_label("Home");
        update(&conn, &tag).unwrap();
        assert_eq!(get(&conn, &tag.id).unwrap().label, "Home");
    }

    // TC-DBTAG-003
    #[test]
    fn get_missing_returns_not_found() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        assert!(matches!(get(&conn, "nope"), Err(DbError::NotFound)));
    }

    // TC-DBTAG-004
    #[test]
    fn delete_removes_row() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let tag = Tag::new("Work", None);
        insert(&conn, &tag).unwrap();
        delete(&conn, &tag.id).unwrap();
        assert!(get(&conn, &tag.id).is_err());
    }

    // TC-DBTAG-005
    #[test]
    fn list_returns_inserted() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        insert(&conn, &Tag::new("a", None)).unwrap();
        insert(&conn, &Tag::new("b", None)).unwrap();
        assert_eq!(list(&conn).unwrap().len(), 2);
    }
}
