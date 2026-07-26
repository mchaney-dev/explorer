use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::tasks::recurrence_rule::{DayOfWeek, Frequency, RecurrenceRule};

fn frequency_to_db(f: &Frequency) -> &'static str {
    match f {
        Frequency::Daily => "daily",
        Frequency::Weekly => "weekly",
        Frequency::Monthly => "monthly",
        Frequency::Yearly => "yearly",
    }
}

fn frequency_from_db(value: Option<String>) -> Option<Frequency> {
    match value.as_deref() {
        Some("daily") => Some(Frequency::Daily),
        Some("weekly") => Some(Frequency::Weekly),
        Some("monthly") => Some(Frequency::Monthly),
        Some("yearly") => Some(Frequency::Yearly),
        _ => None,
    }
}

fn day_to_db(d: &DayOfWeek) -> &'static str {
    match d {
        DayOfWeek::Monday => "monday",
        DayOfWeek::Tuesday => "tuesday",
        DayOfWeek::Wednesday => "wednesday",
        DayOfWeek::Thursday => "thursday",
        DayOfWeek::Friday => "friday",
        DayOfWeek::Saturday => "saturday",
        DayOfWeek::Sunday => "sunday",
    }
}

fn day_from_db(value: &str) -> Option<DayOfWeek> {
    match value {
        "monday" => Some(DayOfWeek::Monday),
        "tuesday" => Some(DayOfWeek::Tuesday),
        "wednesday" => Some(DayOfWeek::Wednesday),
        "thursday" => Some(DayOfWeek::Thursday),
        "friday" => Some(DayOfWeek::Friday),
        "saturday" => Some(DayOfWeek::Saturday),
        "sunday" => Some(DayOfWeek::Sunday),
        _ => None,
    }
}

fn row_to_rule(row: &Row) -> rusqlite::Result<RecurrenceRule> {
    Ok(RecurrenceRule {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        frequency: frequency_from_db(row.get("frequency")?),
        interval: row.get("interval")?,
        days_of_week: Vec::new(),
        day_of_month: row.get("day_of_month")?,
        end_date: row.get("end_date")?,
    })
}

fn load_days(conn: &Connection, recurrence_id: &str) -> DbResult<Vec<DayOfWeek>> {
    let mut stmt =
        conn.prepare("SELECT day_of_week FROM recurrence_days WHERE recurrence_id = ?1")?;
    let rows = stmt.query_map([recurrence_id], |row| row.get::<_, String>("day_of_week"))?;
    let mut days = Vec::new();
    for value in rows {
        if let Some(day) = day_from_db(&value?) {
            days.push(day);
        }
    }
    Ok(days)
}

fn write_rule_row(conn: &Connection, rule: &RecurrenceRule, insert: bool) -> DbResult<usize> {
    let sql = if insert {
        "INSERT INTO recurrence_rules
            (id, created_at, updated_at, frequency, interval, day_of_month, end_date)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    } else {
        "UPDATE recurrence_rules SET
            updated_at = ?3, frequency = ?4, interval = ?5, day_of_month = ?6, end_date = ?7
          WHERE id = ?1"
    };
    let affected = conn.execute(
        sql,
        rusqlite::params![
            rule.id,
            rule.created_at,
            rule.updated_at,
            rule.frequency.as_ref().map(frequency_to_db),
            rule.interval,
            rule.day_of_month,
            rule.end_date,
        ],
    )?;
    Ok(affected)
}

fn write_days(conn: &Connection, rule: &RecurrenceRule) -> DbResult<()> {
    for day in &rule.days_of_week {
        conn.execute(
            "INSERT INTO recurrence_days (recurrence_id, day_of_week) VALUES (?1, ?2)
             ON CONFLICT(recurrence_id, day_of_week) DO NOTHING",
            rusqlite::params![rule.id, day_to_db(day)],
        )?;
    }
    Ok(())
}

pub fn insert(conn: &Connection, rule: &RecurrenceRule) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    write_rule_row(&tx, rule, true)?;
    write_days(&tx, rule)?;
    tx.commit()?;
    Ok(())
}

pub fn update(conn: &Connection, rule: &RecurrenceRule) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    if write_rule_row(&tx, rule, false)? == 0 {
        return Err(DbError::NotFound);
    }
    tx.execute(
        "DELETE FROM recurrence_days WHERE recurrence_id = ?1",
        [&rule.id],
    )?;
    write_days(&tx, rule)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<RecurrenceRule> {
    let mut rule = conn.query_row(
        "SELECT * FROM recurrence_rules WHERE id = ?1",
        [id],
        row_to_rule,
    )?;
    rule.days_of_week = load_days(conn, &rule.id)?;
    Ok(rule)
}

pub fn list(conn: &Connection) -> DbResult<Vec<RecurrenceRule>> {
    let mut stmt = conn.prepare("SELECT * FROM recurrence_rules ORDER BY id DESC")?;
    let rules = stmt
        .query_map([], row_to_rule)?
        .collect::<rusqlite::Result<Vec<RecurrenceRule>>>()?;
    let mut out = Vec::with_capacity(rules.len());
    for mut rule in rules {
        rule.days_of_week = load_days(conn, &rule.id)?;
        out.push(rule);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM recurrence_rules WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    // TC-DBRECUR-001
    #[test]
    fn insert_roundtrips_days() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let rule = RecurrenceRule::new()
            .with_frequency(Frequency::Weekly)
            .with_days_of_week(vec![DayOfWeek::Monday, DayOfWeek::Friday]);
        insert(&conn, &rule).unwrap();
        let got = get(&conn, &rule.id).unwrap();
        assert_eq!(got.days_of_week.len(), 2);
    }

    // TC-DBRECUR-002
    #[test]
    fn frequency_and_interval_persist() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let rule = RecurrenceRule::new()
            .with_frequency(Frequency::Daily)
            .with_interval(3);
        insert(&conn, &rule).unwrap();
        let got = get(&conn, &rule.id).unwrap();
        assert!(matches!(got.frequency, Some(Frequency::Daily)));
        assert_eq!(got.interval, Some(3));
    }

    // TC-DBRECUR-003
    #[test]
    fn update_resyncs_days() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut rule = RecurrenceRule::new().with_days_of_week(vec![DayOfWeek::Monday]);
        insert(&conn, &rule).unwrap();
        rule.add_day(DayOfWeek::Tuesday);
        update(&conn, &rule).unwrap();
        assert_eq!(get(&conn, &rule.id).unwrap().days_of_week.len(), 2);
    }

    // TC-DBRECUR-004
    #[test]
    fn delete_removes_rule() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let rule = RecurrenceRule::new();
        insert(&conn, &rule).unwrap();
        delete(&conn, &rule.id).unwrap();
        assert!(get(&conn, &rule.id).is_err());
    }
}
