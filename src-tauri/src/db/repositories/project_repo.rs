use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::db::repositories::task_repo;
use crate::models::tag::Tag;
use crate::models::tasks::project::Project;

fn row_to_project(row: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        tasks: Vec::new(),
        is_archived: row.get("is_archived")?,
        title: row.get("title")?,
        description: row.get("description")?,
        color: row.get("color")?,
        tags: Vec::new(),
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

fn load_tags(conn: &Connection, project_id: &str) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.* FROM tags t
           JOIN project_tags pt ON pt.tag_id = t.id
          WHERE pt.project_id = ?1 ORDER BY t.id DESC",
    )?;
    let rows = stmt.query_map([project_id], row_to_tag)?;
    Ok(rows.collect::<rusqlite::Result<Vec<Tag>>>()?)
}

fn load_tasks(
    conn: &Connection,
    project_id: &str,
) -> DbResult<Vec<crate::models::tasks::task::Task>> {
    let ids: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT id FROM tasks WHERE project_id = ?1 ORDER BY id DESC")?;
        let rows = stmt.query_map([project_id], |row| row.get::<_, String>("id"))?;
        rows.collect::<rusqlite::Result<Vec<String>>>()?
    };

    let mut tasks = Vec::with_capacity(ids.len());
    for id in ids {
        tasks.push(task_repo::get(conn, &id)?);
    }
    Ok(tasks)
}

fn write_tags(conn: &Connection, project: &Project) -> DbResult<()> {
    for tag in &project.tags {
        conn.execute(
            "INSERT INTO tags (id, created_at, updated_at, label, color)
             VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO NOTHING",
            rusqlite::params![tag.id, tag.created_at, tag.updated_at, tag.label, tag.color],
        )?;
        conn.execute(
            "INSERT INTO project_tags (project_id, tag_id) VALUES (?1, ?2)
             ON CONFLICT(project_id, tag_id) DO NOTHING",
            rusqlite::params![project.id, tag.id],
        )?;
    }
    Ok(())
}

pub fn insert(conn: &Connection, project: &Project) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO projects
            (id, created_at, updated_at, is_archived, title, description, color)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            project.id,
            project.created_at,
            project.updated_at,
            project.is_archived,
            project.title,
            project.description,
            project.color,
        ],
    )?;
    write_tags(&tx, project)?;
    tx.commit()?;
    Ok(())
}

pub fn update(conn: &Connection, project: &Project) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let affected = tx.execute(
        "UPDATE projects SET
            updated_at = ?2, is_archived = ?3, title = ?4, description = ?5, color = ?6
          WHERE id = ?1",
        rusqlite::params![
            project.id,
            project.updated_at,
            project.is_archived,
            project.title,
            project.description,
            project.color,
        ],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    tx.execute(
        "DELETE FROM project_tags WHERE project_id = ?1",
        [&project.id],
    )?;
    write_tags(&tx, project)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<Project> {
    let mut project =
        conn.query_row("SELECT * FROM projects WHERE id = ?1", [id], row_to_project)?;
    project.tags = load_tags(conn, &project.id)?;
    project.tasks = load_tasks(conn, &project.id)?;
    Ok(project)
}

pub fn list(conn: &Connection) -> DbResult<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT * FROM projects ORDER BY id DESC")?;
    let projects = stmt
        .query_map([], row_to_project)?
        .collect::<rusqlite::Result<Vec<Project>>>()?;
    let mut out = Vec::with_capacity(projects.len());
    for mut project in projects {
        project.tags = load_tags(conn, &project.id)?;
        project.tasks = load_tasks(conn, &project.id)?;
        out.push(project);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM projects WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repositories::task_repo;
    use crate::db::Db;
    use crate::models::tasks::task::Task;

    // TC-DBPROJ-001
    #[test]
    fn insert_roundtrips_tags() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut project = Project::new("P", "d");
        project.add_tag(Tag::new("t", None));
        insert(&conn, &project).unwrap();
        let got = get(&conn, &project.id).unwrap();
        assert_eq!(got.title, "P");
        assert_eq!(got.tags.len(), 1);
    }

    // TC-DBPROJ-002
    #[test]
    fn get_loads_project_tasks() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let project = Project::new("P", "d");
        insert(&conn, &project).unwrap();
        let task = Task::new("t", "").with_project(project.id.clone());
        task_repo::insert(&conn, &task).unwrap();
        let got = get(&conn, &project.id).unwrap();
        assert_eq!(got.tasks.len(), 1);
        assert_eq!(got.tasks[0].id, task.id);
    }

    // TC-DBPROJ-003
    #[test]
    fn delete_removes_project() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let project = Project::new("P", "d");
        insert(&conn, &project).unwrap();
        delete(&conn, &project.id).unwrap();
        assert!(get(&conn, &project.id).is_err());
    }
}
