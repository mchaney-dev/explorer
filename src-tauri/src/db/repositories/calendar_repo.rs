use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::calendar::calendar::Calendar;
use crate::models::calendar::calendar_event::CalendarEvent;

fn row_to_calendar(row: &Row) -> rusqlite::Result<Calendar> {
    Ok(Calendar {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        calendar_events: Vec::new(),
        name: row.get("name")?,
        color: row.get("color")?,
        is_visible: row.get("is_visible")?,
        is_default: row.get("is_default")?,
    })
}

fn row_to_event(row: &Row) -> rusqlite::Result<CalendarEvent> {
    Ok(CalendarEvent {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        recurrence_id: row.get("recurrence_id")?,
        calendar_id: row.get("calendar_id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        start_datetime: row.get("start_datetime")?,
        end_datetime: row.get("end_datetime")?,
        is_all_day: row.get("is_all_day")?,
        location: row.get("location")?,
        color: row.get("color")?,
    })
}

fn load_events(conn: &Connection, calendar_id: &str) -> DbResult<Vec<CalendarEvent>> {
    let mut stmt = conn.prepare("SELECT * FROM calendar_events WHERE calendar_id = ?1")?;
    let rows = stmt.query_map([calendar_id], row_to_event)?;
    Ok(rows.collect::<rusqlite::Result<Vec<CalendarEvent>>>()?)
}

fn write_events(conn: &Connection, calendar: &Calendar) -> DbResult<()> {
    for event in &calendar.calendar_events {
        conn.execute(
            "INSERT INTO calendar_events
                (id, created_at, updated_at, recurrence_id, calendar_id, title, description,
                 start_datetime, end_datetime, is_all_day, location, color)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                event.id,
                event.created_at,
                event.updated_at,
                event.recurrence_id,
                calendar.id, // ensure the event points at this calendar
                event.title,
                event.description,
                event.start_datetime,
                event.end_datetime,
                event.is_all_day,
                event.location,
                event.color,
            ],
        )?;
    }
    Ok(())
}

pub fn insert(conn: &Connection, calendar: &Calendar) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO calendars
            (id, created_at, updated_at, name, color, is_visible, is_default)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            calendar.id,
            calendar.created_at,
            calendar.updated_at,
            calendar.name,
            calendar.color,
            calendar.is_visible,
            calendar.is_default,
        ],
    )?;
    write_events(&tx, calendar)?;
    tx.commit()?;
    Ok(())
}

pub fn update(conn: &Connection, calendar: &Calendar) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let affected = tx.execute(
        "UPDATE calendars SET
            updated_at = ?2, name = ?3, color = ?4, is_visible = ?5, is_default = ?6
          WHERE id = ?1",
        rusqlite::params![
            calendar.id,
            calendar.updated_at,
            calendar.name,
            calendar.color,
            calendar.is_visible,
            calendar.is_default,
        ],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    tx.execute(
        "DELETE FROM calendar_events WHERE calendar_id = ?1",
        [&calendar.id],
    )?;
    write_events(&tx, calendar)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<Calendar> {
    let mut calendar = conn.query_row(
        "SELECT * FROM calendars WHERE id = ?1",
        [id],
        row_to_calendar,
    )?;
    calendar.calendar_events = load_events(conn, &calendar.id)?;
    Ok(calendar)
}

pub fn list(conn: &Connection) -> DbResult<Vec<Calendar>> {
    let mut stmt = conn.prepare("SELECT * FROM calendars ORDER BY id DESC")?;
    let calendars = stmt
        .query_map([], row_to_calendar)?
        .collect::<rusqlite::Result<Vec<Calendar>>>()?;
    let mut out = Vec::with_capacity(calendars.len());
    for mut calendar in calendars {
        calendar.calendar_events = load_events(conn, &calendar.id)?;
        out.push(calendar);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM calendars WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use chrono::{Duration, Utc};

    // TC-DBCAL-001
    #[test]
    fn insert_roundtrips_events() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut cal = Calendar::new("Personal");
        cal.add_event(CalendarEvent::new("Meeting"));
        insert(&conn, &cal).unwrap();
        let got = get(&conn, &cal.id).unwrap();
        assert_eq!(got.calendar_events.len(), 1);
        assert_eq!(got.calendar_events[0].title, "Meeting");
    }

    // TC-DBCAL-002
    #[test]
    fn event_times_and_all_day_persist() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let start = Utc::now();
        let end = start + Duration::minutes(60);
        let mut cal = Calendar::new("P");
        cal.add_event(CalendarEvent::new("E").with_times(start, end).all_day());
        insert(&conn, &cal).unwrap();
        let got = get(&conn, &cal.id).unwrap();
        let e = &got.calendar_events[0];
        assert_eq!(e.duration_minutes(), Some(60));
        assert!(e.is_all_day);
    }

    // TC-DBCAL-003
    #[test]
    fn delete_cascades_events() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut cal = Calendar::new("P");
        cal.add_event(CalendarEvent::new("E"));
        insert(&conn, &cal).unwrap();
        delete(&conn, &cal.id).unwrap();
        let events: i64 = conn
            .query_row("SELECT COUNT(*) FROM calendar_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(events, 0);
    }
}
