use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::shared::hex_color::HexColor;
use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
