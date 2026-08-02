use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    KnowledgeGraph,
    NotesWhiteboard,
    Tasks,
    Calendar,
    Feed,
    Notes,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Features {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub knowledge_graph_enabled: bool,
    pub notes_whiteboard_enabled: bool,
    pub tasks_enabled: bool,
    pub calendar_enabled: bool,
    pub feed_enabled: bool,
    pub notes_enabled: bool,
}

impl Features {
    pub fn new() -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            knowledge_graph_enabled: true,
            notes_whiteboard_enabled: true,
            tasks_enabled: true,
            calendar_enabled: true,
            feed_enabled: true,
            notes_enabled: true,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Whether a given feature is currently enabled.
    pub fn is_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::KnowledgeGraph => self.knowledge_graph_enabled,
            Feature::NotesWhiteboard => self.notes_whiteboard_enabled,
            Feature::Tasks => self.tasks_enabled,
            Feature::Calendar => self.calendar_enabled,
            Feature::Feed => self.feed_enabled,
            Feature::Notes => self.notes_enabled,
        }
    }

    pub fn set(&mut self, feature: Feature, enabled: bool) {
        match feature {
            Feature::KnowledgeGraph => self.knowledge_graph_enabled = enabled,
            Feature::NotesWhiteboard => self.notes_whiteboard_enabled = enabled,
            Feature::Tasks => self.tasks_enabled = enabled,
            Feature::Calendar => self.calendar_enabled = enabled,
            Feature::Feed => self.feed_enabled = enabled,
            Feature::Notes => self.notes_enabled = enabled,
        }
        self.touch();
    }

    pub fn toggle(&mut self, feature: Feature) {
        self.set(feature, !self.is_enabled(feature));
    }

    pub fn enable_all(&mut self) {
        self.knowledge_graph_enabled = true;
        self.notes_whiteboard_enabled = true;
        self.tasks_enabled = true;
        self.calendar_enabled = true;
        self.feed_enabled = true;
        self.notes_enabled = true;
        self.touch();
    }

    pub fn disable_all(&mut self) {
        self.knowledge_graph_enabled = false;
        self.notes_whiteboard_enabled = false;
        self.tasks_enabled = false;
        self.calendar_enabled = false;
        self.feed_enabled = false;
        self.notes_enabled = false;
        self.touch();
    }
}

impl Default for Features {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-FEAT-001
    #[test]
    fn new_enables_all() {
        let f = Features::new();
        assert!(f.knowledge_graph_enabled);
        assert!(f.notes_whiteboard_enabled);
        assert!(f.tasks_enabled);
        assert!(f.calendar_enabled);
        assert!(f.feed_enabled);
        assert!(f.notes_enabled);
    }

    // TC-FEAT-002
    #[test]
    fn is_enabled_reflects_flags() {
        let mut f = Features::new();
        assert!(f.is_enabled(Feature::Tasks));
        f.tasks_enabled = false;
        assert!(!f.is_enabled(Feature::Tasks));
    }

    // TC-FEAT-003
    #[test]
    fn set_changes_only_target_feature() {
        let mut f = Features::new();
        f.set(Feature::Feed, false);
        assert!(!f.feed_enabled);
        assert!(f.tasks_enabled, "other features must be untouched");
        assert!(f.calendar_enabled);
    }

    // TC-FEAT-004
    #[test]
    fn toggle_flips_feature() {
        let mut f = Features::new();
        f.toggle(Feature::Notes);
        assert!(!f.is_enabled(Feature::Notes));
        f.toggle(Feature::Notes);
        assert!(f.is_enabled(Feature::Notes));
    }

    // TC-FEAT-005
    #[test]
    fn enable_all_sets_all_true() {
        let mut f = Features::new();
        f.disable_all();
        f.enable_all();
        assert!(f.tasks_enabled && f.feed_enabled && f.notes_enabled);
    }

    // TC-FEAT-006
    #[test]
    fn disable_all_sets_all_false() {
        let mut f = Features::new();
        f.disable_all();
        assert!(!f.knowledge_graph_enabled);
        assert!(!f.notes_whiteboard_enabled);
        assert!(!f.tasks_enabled);
        assert!(!f.calendar_enabled);
        assert!(!f.feed_enabled);
        assert!(!f.notes_enabled);
    }
}
