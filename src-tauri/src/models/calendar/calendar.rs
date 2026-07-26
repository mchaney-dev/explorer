use chrono::Utc;
use serde::{Deserialize, Serialize};
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

#[cfg(test)]
mod tests {
    use super::*;

    // TC-CAL-001
    #[test]
    fn new_defaults() {
        let c = Calendar::new("Personal");
        assert!(c.is_visible);
        assert!(!c.is_default);
        assert!(c.calendar_events.is_empty());
    }

    // TC-CAL-002
    #[test]
    fn with_color_sets_color() {
        let c = Calendar::new("P").with_color("#123456".to_string());
        assert_eq!(c.color.as_deref(), Some("#123456"));
    }

    // TC-CAL-003
    #[test]
    fn as_default_sets_flag() {
        assert!(Calendar::new("P").as_default().is_default);
    }

    // TC-CAL-004
    #[test]
    fn set_name_changes_name() {
        let mut c = Calendar::new("P");
        c.set_name("Work");
        assert_eq!(c.name, "Work");
        assert!(c.updated_at >= c.created_at);
    }

    // TC-CAL-005
    #[test]
    fn set_color_changes_color() {
        let mut c = Calendar::new("P");
        c.set_color(Some("#abcdef".to_string()));
        assert_eq!(c.color.as_deref(), Some("#abcdef"));
        assert!(c.updated_at >= c.created_at);
    }

    // TC-CAL-006
    #[test]
    fn show_hide_toggle_visibility() {
        let mut c = Calendar::new("P");
        c.hide();
        assert!(!c.is_visible);
        c.show();
        assert!(c.is_visible);
        c.toggle_visibility();
        assert!(!c.is_visible);
    }

    // TC-CAL-007
    #[test]
    fn set_default_sets_flag() {
        let mut c = Calendar::new("P");
        c.set_default(true);
        assert!(c.is_default);
    }

    // TC-CAL-008
    #[test]
    fn add_event_appends_and_stamps_calendar_id() {
        let mut c = Calendar::new("P");
        c.add_event(CalendarEvent::new("Meeting"));
        assert_eq!(c.event_count(), 1);
        assert_eq!(
            c.calendar_events[0].calendar_id.as_deref(),
            Some(c.id.as_str())
        );
    }

    // TC-CAL-009
    #[test]
    fn remove_event_present_returns_it() {
        let mut c = Calendar::new("P");
        let event = CalendarEvent::new("Meeting");
        let id = event.id.clone();
        c.add_event(event);
        assert!(c.remove_event(&id).is_some());
        assert_eq!(c.event_count(), 0);
    }

    // TC-CAL-010
    #[test]
    fn remove_event_absent_returns_none() {
        let mut c = Calendar::new("P");
        assert!(c.remove_event("nope").is_none());
    }

    // TC-CAL-011
    #[test]
    fn find_event_and_mut() {
        let mut c = Calendar::new("P");
        let event = CalendarEvent::new("Meeting");
        let id = event.id.clone();
        c.add_event(event);
        assert!(c.find_event(&id).is_some());
        c.find_event_mut(&id).unwrap().set_title("Standup");
        assert_eq!(c.find_event(&id).unwrap().title, "Standup");
    }

    // TC-CAL-012
    #[test]
    fn event_count_is_correct() {
        let mut c = Calendar::new("P");
        c.add_event(CalendarEvent::new("A"));
        c.add_event(CalendarEvent::new("B"));
        assert_eq!(c.event_count(), 2);
    }
}
