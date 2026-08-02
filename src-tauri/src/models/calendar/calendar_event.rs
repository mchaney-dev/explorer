use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::hex_color::HexColor;
use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CalendarEvent {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub recurrence_id: Option<String>,
    pub calendar_id: Option<String>,
    pub title: String,
    pub description: String,
    pub start_datetime: Option<Timestamp>,
    pub end_datetime: Option<Timestamp>,
    pub is_all_day: bool,
    pub location: Option<String>,
    pub color: Option<HexColor>,
}

impl CalendarEvent {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            recurrence_id: None,
            calendar_id: None,
            title: title.into(),
            description: String::new(),
            start_datetime: None,
            end_datetime: None,
            is_all_day: false,
            location: None,
            color: None,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_times(mut self, start: Timestamp, end: Timestamp) -> Self {
        self.start_datetime = Some(start);
        self.end_datetime = Some(end);
        self
    }

    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    pub fn with_color(mut self, color: HexColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn all_day(mut self) -> Self {
        self.is_all_day = true;
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.touch();
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.touch();
    }

    pub fn set_times(&mut self, start: Option<Timestamp>, end: Option<Timestamp>) {
        self.start_datetime = start;
        self.end_datetime = end;
        self.touch();
    }

    pub fn set_location(&mut self, location: Option<String>) {
        self.location = location;
        self.touch();
    }

    // -- Queries ---------------------------------------------------------

    pub fn is_recurring(&self) -> bool {
        self.recurrence_id.is_some()
    }

    /// Duration in whole minutes when both endpoints are set.
    pub fn duration_minutes(&self) -> Option<i64> {
        let start = self.start_datetime?;
        let end = self.end_datetime?;
        Some((end - start).num_minutes())
    }

    /// True when this event's time range overlaps `other`'s. Events missing
    /// either endpoint never overlap.
    pub fn overlaps(&self, other: &CalendarEvent) -> bool {
        match (
            self.start_datetime,
            self.end_datetime,
            other.start_datetime,
            other.end_datetime,
        ) {
            (Some(a_start), Some(a_end), Some(b_start), Some(b_end)) => {
                a_start < b_end && b_start < a_end
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn at(offset_minutes: i64) -> Timestamp {
        Utc::now() + Duration::minutes(offset_minutes)
    }

    // TC-CALEVT-001
    #[test]
    fn new_defaults() {
        let e = CalendarEvent::new("Meeting");
        assert!(e.start_datetime.is_none());
        assert!(e.end_datetime.is_none());
        assert!(!e.is_all_day);
        assert!(e.location.is_none());
        assert!(e.color.is_none());
    }

    // TC-CALEVT-002
    #[test]
    fn with_description_sets_it() {
        let e = CalendarEvent::new("M").with_description("notes");
        assert_eq!(e.description, "notes");
    }

    // TC-CALEVT-003
    #[test]
    fn with_times_sets_start_and_end() {
        let (s, en) = (at(0), at(60));
        let e = CalendarEvent::new("M").with_times(s, en);
        assert_eq!(e.start_datetime, Some(s));
        assert_eq!(e.end_datetime, Some(en));
    }

    // TC-CALEVT-004
    #[test]
    fn with_location_sets_it() {
        let e = CalendarEvent::new("M").with_location("Room 1");
        assert_eq!(e.location.as_deref(), Some("Room 1"));
    }

    // TC-CALEVT-005
    #[test]
    fn with_color_sets_it() {
        let e = CalendarEvent::new("M").with_color("#fff".to_string());
        assert_eq!(e.color.as_deref(), Some("#fff"));
    }

    // TC-CALEVT-006
    #[test]
    fn all_day_sets_flag() {
        assert!(CalendarEvent::new("M").all_day().is_all_day);
    }

    // TC-CALEVT-007
    #[test]
    fn setters_change_fields() {
        let mut e = CalendarEvent::new("M");
        e.set_title("New");
        e.set_description("d");
        e.set_location(Some("here".to_string()));
        e.set_times(Some(at(0)), Some(at(30)));
        assert_eq!(e.title, "New");
        assert_eq!(e.description, "d");
        assert_eq!(e.location.as_deref(), Some("here"));
        assert!(e.start_datetime.is_some());
        assert!(e.updated_at >= e.created_at);
    }

    // TC-CALEVT-008
    #[test]
    fn is_recurring_when_recurrence_set() {
        let mut e = CalendarEvent::new("M");
        assert!(!e.is_recurring());
        e.recurrence_id = Some("rec".to_string());
        assert!(e.is_recurring());
    }

    // TC-CALEVT-009
    #[test]
    fn duration_minutes_when_both_set() {
        let e = CalendarEvent::new("M").with_times(at(0), at(90));
        assert_eq!(e.duration_minutes(), Some(90));
    }

    // TC-CALEVT-010
    #[test]
    fn duration_minutes_none_when_endpoint_missing() {
        let mut e = CalendarEvent::new("M");
        e.start_datetime = Some(at(0));
        assert_eq!(e.duration_minutes(), None);
    }

    // TC-CALEVT-011
    #[test]
    fn overlaps_true_when_ranges_overlap() {
        let a = CalendarEvent::new("A").with_times(at(0), at(60));
        let b = CalendarEvent::new("B").with_times(at(30), at(90));
        assert!(a.overlaps(&b));
    }

    // TC-CALEVT-012
    #[test]
    fn overlaps_false_when_disjoint() {
        let a = CalendarEvent::new("A").with_times(at(0), at(60));
        let b = CalendarEvent::new("B").with_times(at(120), at(180));
        assert!(!a.overlaps(&b));
    }

    // TC-CALEVT-013
    #[test]
    fn overlaps_false_when_endpoint_missing() {
        let a = CalendarEvent::new("A").with_times(at(0), at(60));
        let b = CalendarEvent::new("B"); // no times
        assert!(!a.overlaps(&b));
    }
}
