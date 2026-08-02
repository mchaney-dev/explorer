use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::db::repositories::feed_repo;
use crate::models::feed::feed_interest::FeedInterest;
use crate::models::feed::feed_source::FeedSource;
use crate::models::user::features::Features;
use crate::models::user::preferences::{DefaultView, Preferences, Theme};
use crate::models::user::user::User;

fn theme_to_db(t: &Theme) -> &'static str {
    match t {
        Theme::Light => "light",
        Theme::Dark => "dark",
    }
}

fn theme_from_db(value: &str) -> Theme {
    match value {
        "dark" => Theme::Dark,
        _ => Theme::Light,
    }
}

fn view_to_db(v: &DefaultView) -> &'static str {
    match v {
        DefaultView::Graph => "graph",
        DefaultView::Feed => "feed",
        DefaultView::Tasks => "tasks",
        DefaultView::Notes => "notes",
        DefaultView::Calendar => "calendar",
    }
}

fn view_from_db(value: &str) -> DefaultView {
    match value {
        "feed" => DefaultView::Feed,
        "tasks" => DefaultView::Tasks,
        "notes" => DefaultView::Notes,
        "calendar" => DefaultView::Calendar,
        _ => DefaultView::Graph,
    }
}

fn row_to_user(row: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        preferences: Preferences::new(),
    })
}

fn row_to_features(row: &Row) -> rusqlite::Result<Features> {
    Ok(Features {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        knowledge_graph_enabled: row.get("knowledge_graph_enabled")?,
        notes_whiteboard_enabled: row.get("notes_whiteboard_enabled")?,
        tasks_enabled: row.get("tasks_enabled")?,
        calendar_enabled: row.get("calendar_enabled")?,
        feed_enabled: row.get("feed_enabled")?,
        notes_enabled: row.get("notes_enabled")?,
    })
}

fn row_to_preferences(row: &Row) -> rusqlite::Result<Preferences> {
    Ok(Preferences {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        feed_interests: Vec::new(),
        display_name: row.get("display_name")?,
        theme: theme_from_db(&row.get::<_, String>("theme")?),
        accent_color: row.get("accent_color")?,
        features: Features::new(),
        sidebar_collapsed: row.get("sidebar_collapsed")?,
        default_view: view_from_db(&row.get::<_, String>("default_view")?),
    })
}

fn row_to_interest(row: &Row) -> rusqlite::Result<FeedInterest> {
    Ok(FeedInterest {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        last_interacted_at: row.get("last_interacted_at")?,
        topic: row.get("topic")?,
        sources: Vec::new(),
        explicit_weight: row.get("explicit_weight")?,
        open_count: row.get("open_count")?,
        save_count: row.get("save_count")?,
        dismiss_count: row.get("dismiss_count")?,
    })
}

fn load_interest_sources(conn: &Connection, interest_id: &str) -> DbResult<Vec<FeedSource>> {
    let mut stmt = conn.prepare(
        "SELECT s.* FROM feed_sources s
           JOIN feed_interest_sources fis ON fis.source_id = s.id
          WHERE fis.interest_id = ?1 ORDER BY s.id DESC",
    )?;
    let rows = stmt.query_map([interest_id], feed_repo::row_to_source)?;
    Ok(rows.collect::<rusqlite::Result<Vec<FeedSource>>>()?)
}

fn load_interests(conn: &Connection, preferences_id: &str) -> DbResult<Vec<FeedInterest>> {
    let mut stmt = conn.prepare("SELECT * FROM feed_interests WHERE preferences_id = ?1")?;
    let interests = stmt
        .query_map([preferences_id], row_to_interest)?
        .collect::<rusqlite::Result<Vec<FeedInterest>>>()?;
    let mut out = Vec::with_capacity(interests.len());
    for mut interest in interests {
        interest.sources = load_interest_sources(conn, &interest.id)?;
        out.push(interest);
    }
    Ok(out)
}

fn load_preferences(conn: &Connection, user_id: &str) -> DbResult<Preferences> {
    let mut prefs = conn.query_row(
        "SELECT * FROM preferences WHERE user_id = ?1",
        [user_id],
        row_to_preferences,
    )?;
    prefs.features = conn.query_row(
        "SELECT * FROM features WHERE preferences_id = ?1",
        [&prefs.id],
        row_to_features,
    )?;
    prefs.feed_interests = load_interests(conn, &prefs.id)?;
    Ok(prefs)
}

fn write_preferences(conn: &Connection, user_id: &str, prefs: &Preferences) -> DbResult<()> {
    conn.execute(
        "INSERT INTO preferences
            (id, created_at, updated_at, user_id, display_name, theme, accent_color,
             sidebar_collapsed, default_view)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            prefs.id,
            prefs.created_at,
            prefs.updated_at,
            user_id,
            prefs.display_name,
            theme_to_db(&prefs.theme),
            prefs.accent_color,
            prefs.sidebar_collapsed,
            view_to_db(&prefs.default_view),
        ],
    )?;

    let f = &prefs.features;
    conn.execute(
        "INSERT INTO features
            (id, created_at, updated_at, preferences_id, knowledge_graph_enabled,
             notes_whiteboard_enabled, tasks_enabled, calendar_enabled, feed_enabled, notes_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            f.id,
            f.created_at,
            f.updated_at,
            prefs.id,
            f.knowledge_graph_enabled,
            f.notes_whiteboard_enabled,
            f.tasks_enabled,
            f.calendar_enabled,
            f.feed_enabled,
            f.notes_enabled,
        ],
    )?;

    for interest in &prefs.feed_interests {
        conn.execute(
            "INSERT INTO feed_interests
                (id, created_at, updated_at, preferences_id, last_interacted_at, topic,
                 explicit_weight, open_count, save_count, dismiss_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                interest.id,
                interest.created_at,
                interest.updated_at,
                prefs.id,
                interest.last_interacted_at,
                interest.topic,
                interest.explicit_weight,
                interest.open_count,
                interest.save_count,
                interest.dismiss_count,
            ],
        )?;
        for source in &interest.sources {
            feed_repo::upsert(conn, source)?;
            conn.execute(
                "INSERT INTO feed_interest_sources (interest_id, source_id) VALUES (?1, ?2)
                 ON CONFLICT(interest_id, source_id) DO NOTHING",
                rusqlite::params![interest.id, source.id],
            )?;
        }
    }
    Ok(())
}

pub fn insert(conn: &Connection, user: &User) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO users (id, created_at, updated_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![user.id, user.created_at, user.updated_at],
    )?;
    write_preferences(&tx, &user.id, &user.preferences)?;
    tx.commit()?;
    Ok(())
}

pub fn update(conn: &Connection, user: &User) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let affected = tx.execute(
        "UPDATE users SET updated_at = ?2 WHERE id = ?1",
        rusqlite::params![user.id, user.updated_at],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    tx.execute("DELETE FROM preferences WHERE user_id = ?1", [&user.id])?;
    write_preferences(&tx, &user.id, &user.preferences)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<User> {
    let mut user = conn.query_row("SELECT * FROM users WHERE id = ?1", [id], row_to_user)?;
    user.preferences = load_preferences(conn, &user.id)?;
    Ok(user)
}

pub fn list(conn: &Connection) -> DbResult<Vec<User>> {
    let mut stmt = conn.prepare("SELECT * FROM users ORDER BY id DESC")?;
    let users = stmt
        .query_map([], row_to_user)?
        .collect::<rusqlite::Result<Vec<User>>>()?;
    let mut out = Vec::with_capacity(users.len());
    for mut user in users {
        user.preferences = load_preferences(conn, &user.id)?;
        out.push(user);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM users WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    // TC-DBUSER-001
    #[test]
    fn deep_roundtrip() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut user = User::new();
        let mut interest = FeedInterest::new("rust");
        interest.add_source(FeedSource::new("crates", "https://crates.io"));
        user.preferences.add_feed_interest(interest);
        insert(&conn, &user).unwrap();
        let got = get(&conn, &user.id).unwrap();
        assert_eq!(got.preferences.feed_interests.len(), 1);
        assert_eq!(got.preferences.feed_interests[0].sources.len(), 1);
    }

    // TC-DBUSER-002
    #[test]
    fn theme_and_default_view_persist() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut user = User::new();
        user.preferences.set_theme(Theme::Dark);
        user.preferences.set_default_view(DefaultView::Tasks);
        insert(&conn, &user).unwrap();
        let got = get(&conn, &user.id).unwrap();
        assert_eq!(got.preferences.theme, Theme::Dark);
        assert_eq!(got.preferences.default_view, DefaultView::Tasks);
    }

    // TC-DBUSER-003
    #[test]
    fn delete_cascades_preferences_features_interests() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut user = User::new();
        user.preferences
            .add_feed_interest(FeedInterest::new("rust"));
        insert(&conn, &user).unwrap();
        delete(&conn, &user.id).unwrap();
        let prefs: i64 = conn
            .query_row("SELECT COUNT(*) FROM preferences", [], |r| r.get(0))
            .unwrap();
        let feats: i64 = conn
            .query_row("SELECT COUNT(*) FROM features", [], |r| r.get(0))
            .unwrap();
        let ints: i64 = conn
            .query_row("SELECT COUNT(*) FROM feed_interests", [], |r| r.get(0))
            .unwrap();
        assert_eq!((prefs, feats, ints), (0, 0, 0));
    }

    // TC-DBUSER-004
    #[test]
    fn update_resyncs_interests() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut user = User::new();
        insert(&conn, &user).unwrap();
        user.preferences
            .add_feed_interest(FeedInterest::new("rust"));
        update(&conn, &user).unwrap();
        assert_eq!(
            get(&conn, &user.id)
                .unwrap()
                .preferences
                .feed_interests
                .len(),
            1
        );
    }
}
