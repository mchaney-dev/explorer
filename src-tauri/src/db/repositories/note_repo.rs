use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::notes::note::Note;
use crate::models::notes::note_block::{BlockType, NoteBlock};
use crate::models::tag::Tag;

fn block_type_to_db(b: &BlockType) -> &'static str {
    match b {
        BlockType::Text => "text",
        BlockType::Drawing => "drawing",
        BlockType::Image => "image",
        BlockType::Embed => "embed",
    }
}

fn block_type_from_db(value: Option<String>) -> Option<BlockType> {
    match value.as_deref() {
        Some("text") => Some(BlockType::Text),
        Some("drawing") => Some(BlockType::Drawing),
        Some("image") => Some(BlockType::Image),
        Some("embed") => Some(BlockType::Embed),
        _ => None,
    }
}

fn row_to_note(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        note_blocks: Vec::new(),
        title: row.get("title")?,
        is_pinned: row.get("is_pinned")?,
        tags: Vec::new(),
    })
}

fn row_to_block(row: &Row) -> rusqlite::Result<NoteBlock> {
    Ok(NoteBlock {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        note_id: row.get("note_id")?,
        block_type: block_type_from_db(row.get("block_type")?),
        content: row.get("content")?,
        position: row.get("position")?,
    })
}

fn row_to_tag(row: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        label: row.get("label")?,
        color: row.get("color")?,
    })
}

fn load_blocks(conn: &Connection, note_id: &str) -> DbResult<Vec<NoteBlock>> {
    let mut stmt = conn.prepare("SELECT * FROM note_blocks WHERE note_id = ?1")?;
    let rows = stmt.query_map([note_id], row_to_block)?;
    Ok(rows.collect::<rusqlite::Result<Vec<NoteBlock>>>()?)
}

fn load_tags(conn: &Connection, note_id: &str) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.* FROM tags t
           JOIN note_tags nt ON nt.tag_id = t.id
          WHERE nt.note_id = ?1 ORDER BY t.id DESC",
    )?;
    let rows = stmt.query_map([note_id], row_to_tag)?;
    Ok(rows.collect::<rusqlite::Result<Vec<Tag>>>()?)
}

fn write_blocks(conn: &Connection, note: &Note) -> DbResult<()> {
    for block in &note.note_blocks {
        conn.execute(
            "INSERT INTO note_blocks
                (id, created_at, updated_at, note_id, block_type, content, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                block.id,
                block.created_at,
                block.updated_at,
                note.id, // ensure the block points at this note
                block.block_type.as_ref().map(block_type_to_db),
                block.content,
                block.position,
            ],
        )?;
    }
    Ok(())
}

fn write_tags(conn: &Connection, note: &Note) -> DbResult<()> {
    for tag in &note.tags {
        conn.execute(
            "INSERT INTO tags (id, created_at, updated_at, label, color)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO NOTHING",
            rusqlite::params![tag.id, tag.created_at, tag.updated_at, tag.label, tag.color],
        )?;
        conn.execute(
            "INSERT INTO note_tags (note_id, tag_id) VALUES (?1, ?2)
             ON CONFLICT(note_id, tag_id) DO NOTHING",
            rusqlite::params![note.id, tag.id],
        )?;
    }
    Ok(())
}

pub fn insert(conn: &Connection, note: &Note) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO notes (id, created_at, updated_at, title, is_pinned)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            note.id,
            note.created_at,
            note.updated_at,
            note.title,
            note.is_pinned
        ],
    )?;
    write_blocks(&tx, note)?;
    write_tags(&tx, note)?;
    tx.commit()?;
    Ok(())
}

pub fn update(conn: &Connection, note: &Note) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let affected = tx.execute(
        "UPDATE notes SET updated_at = ?2, title = ?3, is_pinned = ?4 WHERE id = ?1",
        rusqlite::params![note.id, note.updated_at, note.title, note.is_pinned],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    tx.execute("DELETE FROM note_blocks WHERE note_id = ?1", [&note.id])?;
    tx.execute("DELETE FROM note_tags WHERE note_id = ?1", [&note.id])?;
    write_blocks(&tx, note)?;
    write_tags(&tx, note)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<Note> {
    let mut note = conn.query_row("SELECT * FROM notes WHERE id = ?1", [id], row_to_note)?;
    note.note_blocks = load_blocks(conn, &note.id)?;
    note.tags = load_tags(conn, &note.id)?;
    Ok(note)
}

pub fn list(conn: &Connection) -> DbResult<Vec<Note>> {
    let mut stmt = conn.prepare("SELECT * FROM notes ORDER BY id DESC")?;
    let notes = stmt
        .query_map([], row_to_note)?
        .collect::<rusqlite::Result<Vec<Note>>>()?;
    let mut out = Vec::with_capacity(notes.len());
    for mut note in notes {
        note.note_blocks = load_blocks(conn, &note.id)?;
        note.tags = load_tags(conn, &note.id)?;
        out.push(note);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM notes WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    // TC-DBNOTE-001
    #[test]
    fn insert_roundtrips_blocks_and_tags() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut note = Note::new("N");
        note.add_block(NoteBlock::new(&note.id).with_content("hello"));
        note.add_tag(Tag::new("t", None));
        insert(&conn, &note).unwrap();
        let got = get(&conn, &note.id).unwrap();
        assert_eq!(got.note_blocks.len(), 1);
        assert_eq!(got.note_blocks[0].content, "hello");
        assert_eq!(got.tags.len(), 1);
    }

    // TC-DBNOTE-002
    #[test]
    fn block_type_and_position_persist() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut note = Note::new("N");
        note.add_block(
            NoteBlock::new(&note.id)
                .with_type(BlockType::Image)
                .with_position(2.5),
        );
        insert(&conn, &note).unwrap();
        let got = get(&conn, &note.id).unwrap();
        let block = &got.note_blocks[0];
        assert!(matches!(block.block_type, Some(BlockType::Image)));
        assert_eq!(block.position, Some(2.5));
    }

    // TC-DBNOTE-003
    #[test]
    fn update_resyncs_blocks() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut note = Note::new("N");
        note.add_block(NoteBlock::new(&note.id).with_content("a"));
        insert(&conn, &note).unwrap();
        note.add_block(NoteBlock::new(&note.id).with_content("b"));
        update(&conn, &note).unwrap();
        assert_eq!(get(&conn, &note.id).unwrap().note_blocks.len(), 2);
    }

    // TC-DBNOTE-004
    #[test]
    fn delete_cascades_blocks() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut note = Note::new("N");
        note.add_block(NoteBlock::new(&note.id).with_content("a"));
        insert(&conn, &note).unwrap();
        delete(&conn, &note.id).unwrap();
        let blocks: i64 = conn
            .query_row("SELECT COUNT(*) FROM note_blocks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(blocks, 0, "blocks should cascade-delete with the note");
    }
}
