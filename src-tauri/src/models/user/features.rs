use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    KnowledgeGraph,
    NotesWhiteboard,
    Tasks,
    Calendar,
    Feed,
    Notes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
