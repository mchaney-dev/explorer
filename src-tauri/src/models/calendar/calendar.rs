use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::calendar::calendar_event::CalendarEvent;
use crate::models::shared::hex_color::HexColor;
use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub calendar_events: Vec<CalendarEvent>,
    pub name: String,
    pub color: Option<HexColor>,
    pub is_visible: bool,
    pub is_default: bool,
}

impl Calendar {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            calendar_events: Vec::new(),
            name: name.into(),
            color: None,
            is_visible: true,
            is_default: false,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_color(mut self, color: HexColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.touch();
    }

    pub fn set_color(&mut self, color: Option<HexColor>) {
        self.color = color;
        self.touch();
    }

    pub fn show(&mut self) {
        self.is_visible = true;
        self.touch();
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.touch();
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
        self.touch();
    }

    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
        self.touch();
    }

    // -- Events ----------------------------------------------------------

    pub fn add_event(&mut self, mut event: CalendarEvent) {
        event.calendar_id = Some(self.id.clone());
        self.calendar_events.push(event);
        self.touch();
    }

    pub fn remove_event(&mut self, event_id: &str) -> Option<CalendarEvent> {
        let idx = self.calendar_events.iter().position(|e| e.id == event_id)?;
        let event = self.calendar_events.remove(idx);
        self.touch();
        Some(event)
    }

    pub fn find_event(&self, event_id: &str) -> Option<&CalendarEvent> {
        self.calendar_events.iter().find(|e| e.id == event_id)
    }

    pub fn find_event_mut(&mut self, event_id: &str) -> Option<&mut CalendarEvent> {
        self.calendar_events.iter_mut().find(|e| e.id == event_id)
    }

    pub fn event_count(&self) -> usize {
        self.calendar_events.len()
    }
}
