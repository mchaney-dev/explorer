//! Schema creation and versioning.

use rusqlite::Connection;

use crate::db::error::DbResult;

const MIGRATIONS: &[&str] = &[SCHEMA_V1];

/// Reads the database's current `user_version`, then runs every migration whose
/// target version is higher, wrapping each in a transaction
pub fn run(conn: &Connection) -> DbResult<()> {
    // get the current version
    let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for (index, migration_sql) in MIGRATIONS.iter().enumerate() {
        let target_version = (index + 1) as i64;
        // if target_version is newer, migrate
        if current < target_version {
            let tx = conn.unchecked_transaction()?;
            tx.execute_batch(migration_sql)?;
            tx.pragma_update(None, "user_version", target_version)?;
            tx.commit()?;
        }
    }

    Ok(())
}

const SCHEMA_V1: &str = "
-- ============================================================================
-- Tags: a shared entity referenced by tasks, projects, notes and graph nodes
-- through the *_tags join tables below.
-- ============================================================================
CREATE TABLE tags (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    label      TEXT NOT NULL,
    color      TEXT
);

-- ============================================================================
-- Recurrence rules (shared by tasks and calendar events)
-- ============================================================================
CREATE TABLE recurrence_rules (
    id           TEXT PRIMARY KEY,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    frequency    TEXT,               -- 'daily' | 'weekly' | 'monthly' | 'yearly'
    interval     INTEGER,
    day_of_month INTEGER,
    end_date     TEXT
);

-- The Vec<DayOfWeek> on a rule becomes its own small table: one row per day.
CREATE TABLE recurrence_days (
    recurrence_id TEXT NOT NULL REFERENCES recurrence_rules(id) ON DELETE CASCADE,
    day_of_week   TEXT NOT NULL,     -- 'monday' .. 'sunday'
    PRIMARY KEY (recurrence_id, day_of_week)
);

-- ============================================================================
-- Projects
-- ============================================================================
CREATE TABLE projects (
    id          TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    is_archived INTEGER NOT NULL DEFAULT 0,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    color       TEXT
);

CREATE TABLE project_tags (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tag_id     TEXT NOT NULL REFERENCES tags(id)     ON DELETE CASCADE,
    PRIMARY KEY (project_id, tag_id)
);

-- ============================================================================
-- Tasks
-- ============================================================================
CREATE TABLE tasks (
    id            TEXT PRIMARY KEY,
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL,
    recurrence_id TEXT REFERENCES recurrence_rules(id) ON DELETE SET NULL,
    parent_id     TEXT REFERENCES tasks(id)           ON DELETE CASCADE,
    project_id    TEXT REFERENCES projects(id)        ON DELETE SET NULL,
    completed_at  TEXT,
    title         TEXT NOT NULL,
    description   TEXT NOT NULL,
    status        TEXT,   -- 'todo' | 'in_progress' | 'done' | 'cancelled'
    priority      TEXT,   -- 'low' | 'medium' | 'high' | 'urgent'
    due_date      TEXT
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

CREATE INDEX idx_tasks_project  ON tasks(project_id);
CREATE INDEX idx_tasks_parent   ON tasks(parent_id);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);

-- ============================================================================
-- Notes
-- ============================================================================
CREATE TABLE notes (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    title      TEXT NOT NULL,
    is_pinned  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE note_blocks (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    note_id    TEXT REFERENCES notes(id) ON DELETE CASCADE,
    block_type TEXT,   -- 'text' | 'drawing' | 'image' | 'embed'
    content    TEXT NOT NULL,
    position   REAL
);

CREATE TABLE note_tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);

CREATE INDEX idx_note_blocks_note ON note_blocks(note_id);

-- ============================================================================
-- Knowledge graph
-- ============================================================================
CREATE TABLE graphs (
    id          TEXT PRIMARY KEY,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    title       TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE nodes (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    graph_id   TEXT REFERENCES graphs(id) ON DELETE CASCADE,
    node_type  TEXT NOT NULL,   -- 'file' | 'video' | 'audio' | 'text' | 'image'
    x          INTEGER NOT NULL,
    y          INTEGER NOT NULL,
    title      TEXT NOT NULL,
    content    TEXT NOT NULL
);

CREATE TABLE node_tags (
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (node_id, tag_id)
);

CREATE TABLE connections (
    id             TEXT PRIMARY KEY,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    graph_id       TEXT REFERENCES graphs(id) ON DELETE CASCADE,
    source_node_id TEXT REFERENCES nodes(id)  ON DELETE CASCADE,
    dest_node_id   TEXT REFERENCES nodes(id)  ON DELETE CASCADE,
    label          TEXT,   -- 'relates_to' | 'caused_by' | 'is_a' | 'uses'
    is_directed    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_nodes_graph       ON nodes(graph_id);
CREATE INDEX idx_connections_graph ON connections(graph_id);

-- ============================================================================
-- Calendar
-- ============================================================================
CREATE TABLE calendars (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    name       TEXT NOT NULL,
    color      TEXT,
    is_visible INTEGER NOT NULL DEFAULT 1,
    is_default INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE calendar_events (
    id             TEXT PRIMARY KEY,
    created_at     TEXT NOT NULL,
    updated_at     TEXT NOT NULL,
    recurrence_id  TEXT REFERENCES recurrence_rules(id) ON DELETE SET NULL,
    calendar_id    TEXT REFERENCES calendars(id)        ON DELETE CASCADE,
    title          TEXT NOT NULL,
    description    TEXT NOT NULL,
    start_datetime TEXT,
    end_datetime   TEXT,
    is_all_day     INTEGER NOT NULL DEFAULT 0,
    location       TEXT,
    color          TEXT
);

CREATE INDEX idx_calendar_events_calendar ON calendar_events(calendar_id);

-- ============================================================================
-- User, preferences, features, feed interests/sources
-- ============================================================================
CREATE TABLE users (
    id         TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE preferences (
    id                TEXT PRIMARY KEY,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    user_id           TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    display_name      TEXT NOT NULL,
    theme             TEXT NOT NULL,   -- 'light' | 'dark'
    accent_color      TEXT,
    sidebar_collapsed INTEGER NOT NULL DEFAULT 0,
    default_view      TEXT NOT NULL    -- 'graph' | 'feed' | 'tasks' | 'notes' | 'calendar'
);

CREATE TABLE features (
    id                       TEXT PRIMARY KEY,
    created_at               TEXT NOT NULL,
    updated_at               TEXT NOT NULL,
    preferences_id           TEXT NOT NULL REFERENCES preferences(id) ON DELETE CASCADE,
    knowledge_graph_enabled  INTEGER NOT NULL DEFAULT 1,
    notes_whiteboard_enabled INTEGER NOT NULL DEFAULT 1,
    tasks_enabled            INTEGER NOT NULL DEFAULT 1,
    calendar_enabled         INTEGER NOT NULL DEFAULT 1,
    feed_enabled             INTEGER NOT NULL DEFAULT 1,
    notes_enabled            INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE feed_sources (
    id           TEXT PRIMARY KEY,
    created_at   TEXT NOT NULL,
    updated_at   TEXT NOT NULL,
    source_type  TEXT,   -- 'text' | 'image' | 'video' | 'audio'
    name         TEXT NOT NULL,
    base_url     TEXT NOT NULL,
    is_enabled   INTEGER NOT NULL DEFAULT 1,
    requires_key INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE feed_interests (
    id                 TEXT PRIMARY KEY,
    created_at         TEXT NOT NULL,
    updated_at         TEXT NOT NULL,
    preferences_id     TEXT NOT NULL REFERENCES preferences(id) ON DELETE CASCADE,
    last_interacted_at TEXT,
    topic              TEXT NOT NULL,
    explicit_weight    REAL,
    open_count         INTEGER NOT NULL DEFAULT 0,
    save_count         INTEGER NOT NULL DEFAULT 0,
    dismiss_count      INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE feed_interest_sources (
    interest_id TEXT NOT NULL REFERENCES feed_interests(id) ON DELETE CASCADE,
    source_id   TEXT NOT NULL REFERENCES feed_sources(id)   ON DELETE CASCADE,
    PRIMARY KEY (interest_id, source_id)
);
";
