use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::tag::Tag;
use crate::models::tasks::task::{Priority, Status, Task};

fn status_to_db(status: &Status) -> &'static str {
    match status {
        Status::Todo => "todo",
        Status::InProgress => "in_progress",
        Status::Done => "done",
        Status::Cancelled => "cancelled",
    }
}

fn status_from_db(value: Option<String>) -> Option<Status> {
    match value.as_deref() {
        Some("todo") => Some(Status::Todo),
        Some("in_progress") => Some(Status::InProgress),
        Some("done") => Some(Status::Done),
        Some("cancelled") => Some(Status::Cancelled),
        _ => None,
    }
}

fn priority_to_db(priority: &Priority) -> &'static str {
    match priority {
        Priority::Low => "low",
        Priority::Medium => "medium",
        Priority::High => "high",
        Priority::Urgent => "urgent",
    }
}

fn priority_from_db(value: Option<String>) -> Option<Priority> {
    match value.as_deref() {
        Some("low") => Some(Priority::Low),
        Some("medium") => Some(Priority::Medium),
        Some("high") => Some(Priority::High),
        Some("urgent") => Some(Priority::Urgent),
        _ => None,
    }
}

/// Decode the `tasks` row itself
fn row_to_task(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        recurrence_id: row.get("recurrence_id")?,
        parent_id: row.get("parent_id")?,
        project_id: row.get("project_id")?,
        completed_at: row.get("completed_at")?,
        title: row.get("title")?,
        description: row.get("description")?,
        status: status_from_db(row.get("status")?),
        priority: priority_from_db(row.get("priority")?),
        due_date: row.get("due_date")?,
        tags: Vec::new(),
    })
}

/// Load the tags linked to one task by joining `task_tags` to `tags`.
fn load_tags(conn: &Connection, task_id: &str) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.created_at, t.updated_at, t.label, t.color
           FROM tags t
           JOIN task_tags tt ON tt.tag_id = t.id
          WHERE tt.task_id = ?1
          ORDER BY t.id DESC",
    )?;
    let rows = stmt.query_map([task_id], |row| {
        Ok(Tag {
            id: row.get("id")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            label: row.get("label")?,
            color: row.get("color")?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<Tag>>>()?)
}

/// Insert a task and all of its tag links
pub fn insert(conn: &Connection, task: &Task) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "INSERT INTO tasks (
            id, created_at, updated_at, recurrence_id, parent_id, project_id,
            completed_at, title, description, status, priority, due_date
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            task.id,
            task.created_at,
            task.updated_at,
            task.recurrence_id,
            task.parent_id,
            task.project_id,
            task.completed_at,
            task.title,
            task.description,
            task.status.as_ref().map(status_to_db),
            task.priority.as_ref().map(priority_to_db),
            task.due_date,
        ],
    )?;

    write_tag_links(&tx, task)?;

    tx.commit()?;
    Ok(())
}

/// Update a task's columns and fully resync its tag links
pub fn update(conn: &Connection, task: &Task) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;

    let affected = tx.execute(
        "UPDATE tasks SET
            updated_at    = ?2,
            recurrence_id = ?3,
            parent_id     = ?4,
            project_id    = ?5,
            completed_at  = ?6,
            title         = ?7,
            description   = ?8,
            status        = ?9,
            priority      = ?10,
            due_date      = ?11
          WHERE id = ?1",
        rusqlite::params![
            task.id,
            task.updated_at,
            task.recurrence_id,
            task.parent_id,
            task.project_id,
            task.completed_at,
            task.title,
            task.description,
            task.status.as_ref().map(status_to_db),
            task.priority.as_ref().map(priority_to_db),
            task.due_date,
        ],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }

    tx.execute("DELETE FROM task_tags WHERE task_id = ?1", [&task.id])?;
    write_tag_links(&tx, task)?;

    tx.commit()?;
    Ok(())
}

/// Ensure each of the task's tags exists in `tags`, then link it in `task_tags`.
fn write_tag_links(conn: &Connection, task: &Task) -> DbResult<()> {
    for tag in &task.tags {
        // Make sure the referenced tag row exists
        conn.execute(
            "INSERT INTO tags (id, created_at, updated_at, label, color)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO NOTHING",
            rusqlite::params![tag.id, tag.created_at, tag.updated_at, tag.label, tag.color],
        )?;
        // Link tag <-> task.
        conn.execute(
            "INSERT INTO task_tags (task_id, tag_id) VALUES (?1, ?2)
             ON CONFLICT(task_id, tag_id) DO NOTHING",
            rusqlite::params![task.id, tag.id],
        )?;
    }
    Ok(())
}

/// Fetch one task with its tags attached.
pub fn get(conn: &Connection, id: &str) -> DbResult<Task> {
    let mut task = conn.query_row("SELECT * FROM tasks WHERE id = ?1", [id], row_to_task)?;
    task.tags = load_tags(conn, &task.id)?;
    Ok(task)
}

/// Fetch every task (newest first), each with its tags
pub fn list(conn: &Connection) -> DbResult<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT * FROM tasks ORDER BY id DESC")?;
    let tasks = stmt
        .query_map([], row_to_task)?
        .collect::<rusqlite::Result<Vec<Task>>>()?;

    // attach each task's tags
    let mut with_tags = Vec::with_capacity(tasks.len());
    for mut task in tasks {
        task.tags = load_tags(conn, &task.id)?;
        with_tags.push(task);
    }
    Ok(with_tags)
}

/// Delete a task.
pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    let affected = conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repositories::tag_repo;
    use crate::db::Db;

    // TC-DBTASK-001
    #[test]
    fn insert_with_tags_roundtrips() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut task = Task::new("T", "desc");
        task.add_tag(Tag::new("work", None));
        insert(&conn, &task).unwrap();
        let got = get(&conn, &task.id).unwrap();
        assert_eq!(got.title, "T");
        assert_eq!(got.description, "desc");
        assert_eq!(got.tags.len(), 1);
        assert_eq!(got.tags[0].label, "work");
    }

    // TC-DBTASK-002
    #[test]
    fn status_and_priority_persist() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut task = Task::new("T", "").with_priority(Priority::High);
        task.mark_as_in_progress();
        insert(&conn, &task).unwrap();
        let got = get(&conn, &task.id).unwrap();
        assert!(matches!(got.status, Some(Status::InProgress)));
        assert!(matches!(got.priority, Some(Priority::High)));
    }

    // TC-DBTASK-003
    #[test]
    fn update_resyncs_tags() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut task = Task::new("T", "");
        let tag = Tag::new("work", None);
        let tag_id = tag.id.clone();
        task.add_tag(tag);
        insert(&conn, &task).unwrap();
        task.remove_tag(&tag_id);
        update(&conn, &task).unwrap();
        assert_eq!(get(&conn, &task.id).unwrap().tags.len(), 0);
    }

    // TC-DBTASK-004
    #[test]
    fn delete_task_preserves_shared_tag() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut task = Task::new("T", "");
        let tag = Tag::new("work", None);
        let tag_id = tag.id.clone();
        task.add_tag(tag);
        insert(&conn, &task).unwrap();
        delete(&conn, &task.id).unwrap();
        assert!(get(&conn, &task.id).is_err());
        assert!(
            tag_repo::get(&conn, &tag_id).is_ok(),
            "shared tag row remains"
        );
    }

    // TC-DBTASK-005
    #[test]
    fn delete_parent_cascades_to_subtask() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let parent = Task::new("parent", "");
        insert(&conn, &parent).unwrap();
        let child = Task::new("child", "").with_parent(parent.id.clone());
        insert(&conn, &child).unwrap();
        delete(&conn, &parent.id).unwrap();
        assert!(
            get(&conn, &child.id).is_err(),
            "subtask should cascade-delete"
        );
    }

    // TC-DBTASK-006
    #[test]
    fn shared_tag_upserted_once() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let tag = Tag::new("work", None);
        let mut a = Task::new("A", "");
        a.add_tag(tag.clone());
        let mut b = Task::new("B", "");
        b.add_tag(tag);
        insert(&conn, &a).unwrap();
        insert(&conn, &b).unwrap();
        assert_eq!(tag_repo::list(&conn).unwrap().len(), 1);
        assert_eq!(get(&conn, &a.id).unwrap().tags.len(), 1);
        assert_eq!(get(&conn, &b.id).unwrap().tags.len(), 1);
    }
}
