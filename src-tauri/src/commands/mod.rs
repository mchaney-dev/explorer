use tauri::State;

use crate::db::repositories::{
    calendar_repo, feed_repo, graph_repo, note_repo, project_repo, recurrence_repo, tag_repo,
    task_repo, user_repo,
};
use crate::db::{Db, DbResult};
use crate::models::calendar::calendar::Calendar;
use crate::models::feed::feed_source::FeedSource;
use crate::models::knowledge_graph::graph::Graph;
use crate::models::notes::note::Note;
use crate::models::tag::Tag;
use crate::models::tasks::project::Project;
use crate::models::tasks::recurrence_rule::RecurrenceRule;
use crate::models::tasks::task::Task;
use crate::models::user::user::User;

/// Lock the database mutex
macro_rules! conn {
    ($db:expr) => {
        $db.conn.lock().expect("database mutex was poisoned")
    };
}

#[tauri::command]
pub fn create_tag(db: State<Db>, label: String, color: Option<String>) -> DbResult<Tag> {
    let tag = Tag::new(label, color);
    tag_repo::insert(&conn!(db), &tag)?;
    Ok(tag)
}

#[tauri::command]
pub fn save_tag(db: State<Db>, tag: Tag) -> DbResult<()> {
    tag_repo::update(&conn!(db), &tag)
}

#[tauri::command]
pub fn get_tag(db: State<Db>, id: String) -> DbResult<Tag> {
    tag_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_tags(db: State<Db>) -> DbResult<Vec<Tag>> {
    tag_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_tag(db: State<Db>, id: String) -> DbResult<()> {
    tag_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_task(db: State<Db>, title: String, description: String) -> DbResult<Task> {
    let task = Task::new(title, description);
    task_repo::insert(&conn!(db), &task)?;
    Ok(task)
}

#[tauri::command]
pub fn save_task(db: State<Db>, task: Task) -> DbResult<()> {
    task_repo::update(&conn!(db), &task)
}

#[tauri::command]
pub fn get_task(db: State<Db>, id: String) -> DbResult<Task> {
    task_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_tasks(db: State<Db>) -> DbResult<Vec<Task>> {
    task_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_task(db: State<Db>, id: String) -> DbResult<()> {
    task_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_project(db: State<Db>, title: String, description: String) -> DbResult<Project> {
    let project = Project::new(title, description);
    project_repo::insert(&conn!(db), &project)?;
    Ok(project)
}

#[tauri::command]
pub fn save_project(db: State<Db>, project: Project) -> DbResult<()> {
    project_repo::update(&conn!(db), &project)
}

#[tauri::command]
pub fn get_project(db: State<Db>, id: String) -> DbResult<Project> {
    project_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_projects(db: State<Db>) -> DbResult<Vec<Project>> {
    project_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_project(db: State<Db>, id: String) -> DbResult<()> {
    project_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_note(db: State<Db>, title: String) -> DbResult<Note> {
    let note = Note::new(title);
    note_repo::insert(&conn!(db), &note)?;
    Ok(note)
}

#[tauri::command]
pub fn save_note(db: State<Db>, note: Note) -> DbResult<()> {
    note_repo::update(&conn!(db), &note)
}

#[tauri::command]
pub fn get_note(db: State<Db>, id: String) -> DbResult<Note> {
    note_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_notes(db: State<Db>) -> DbResult<Vec<Note>> {
    note_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_note(db: State<Db>, id: String) -> DbResult<()> {
    note_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_graph(db: State<Db>, title: String, description: String) -> DbResult<Graph> {
    let graph = Graph::new(title, description);
    graph_repo::insert(&conn!(db), &graph)?;
    Ok(graph)
}

#[tauri::command]
pub fn save_graph(db: State<Db>, graph: Graph) -> DbResult<()> {
    graph_repo::update(&conn!(db), &graph)
}

#[tauri::command]
pub fn get_graph(db: State<Db>, id: String) -> DbResult<Graph> {
    graph_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_graphs(db: State<Db>) -> DbResult<Vec<Graph>> {
    graph_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_graph(db: State<Db>, id: String) -> DbResult<()> {
    graph_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_calendar(db: State<Db>, name: String) -> DbResult<Calendar> {
    let calendar = Calendar::new(name);
    calendar_repo::insert(&conn!(db), &calendar)?;
    Ok(calendar)
}

#[tauri::command]
pub fn save_calendar(db: State<Db>, calendar: Calendar) -> DbResult<()> {
    calendar_repo::update(&conn!(db), &calendar)
}

#[tauri::command]
pub fn get_calendar(db: State<Db>, id: String) -> DbResult<Calendar> {
    calendar_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_calendars(db: State<Db>) -> DbResult<Vec<Calendar>> {
    calendar_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_calendar(db: State<Db>, id: String) -> DbResult<()> {
    calendar_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_recurrence(db: State<Db>) -> DbResult<RecurrenceRule> {
    let rule = RecurrenceRule::new();
    recurrence_repo::insert(&conn!(db), &rule)?;
    Ok(rule)
}

#[tauri::command]
pub fn save_recurrence(db: State<Db>, rule: RecurrenceRule) -> DbResult<()> {
    recurrence_repo::update(&conn!(db), &rule)
}

#[tauri::command]
pub fn get_recurrence(db: State<Db>, id: String) -> DbResult<RecurrenceRule> {
    recurrence_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_recurrences(db: State<Db>) -> DbResult<Vec<RecurrenceRule>> {
    recurrence_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_recurrence(db: State<Db>, id: String) -> DbResult<()> {
    recurrence_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_feed_source(db: State<Db>, name: String, base_url: String) -> DbResult<FeedSource> {
    let source = FeedSource::new(name, base_url);
    feed_repo::insert(&conn!(db), &source)?;
    Ok(source)
}

#[tauri::command]
pub fn save_feed_source(db: State<Db>, source: FeedSource) -> DbResult<()> {
    feed_repo::update(&conn!(db), &source)
}

#[tauri::command]
pub fn get_feed_source(db: State<Db>, id: String) -> DbResult<FeedSource> {
    feed_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_feed_sources(db: State<Db>) -> DbResult<Vec<FeedSource>> {
    feed_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_feed_source(db: State<Db>, id: String) -> DbResult<()> {
    feed_repo::delete(&conn!(db), &id)
}

#[tauri::command]
pub fn create_user(db: State<Db>) -> DbResult<User> {
    let user = User::new();
    user_repo::insert(&conn!(db), &user)?;
    Ok(user)
}

#[tauri::command]
pub fn save_user(db: State<Db>, user: User) -> DbResult<()> {
    user_repo::update(&conn!(db), &user)
}

#[tauri::command]
pub fn get_user(db: State<Db>, id: String) -> DbResult<User> {
    user_repo::get(&conn!(db), &id)
}

#[tauri::command]
pub fn list_users(db: State<Db>) -> DbResult<Vec<User>> {
    user_repo::list(&conn!(db))
}

#[tauri::command]
pub fn delete_user(db: State<Db>, id: String) -> DbResult<()> {
    user_repo::delete(&conn!(db), &id)
}
