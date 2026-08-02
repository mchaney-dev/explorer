use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::{HexColor, Timestamp};

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct Tag {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub label: String,
    pub color: Option<HexColor>,
}

impl Tag {
    pub fn new(label: impl Into<String>, color: impl Into<Option<HexColor>>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            label: label.into(),
            color: color.into(),
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = label.into();
        self.touch();
    }

    pub fn set_color(&mut self, color: Option<HexColor>) {
        self.color = color;
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-TAG-001
    #[test]
    fn new_sets_fields_and_equal_timestamps() {
        let tag = Tag::new("Work", Some("#ff0000".to_string()));
        assert_eq!(tag.label, "Work");
        assert_eq!(tag.color.as_deref(), Some("#ff0000"));
        assert!(!tag.id.is_empty());
        assert_eq!(tag.created_at, tag.updated_at);
    }

    // TC-TAG-002
    #[test]
    fn set_label_changes_label_and_bumps_updated_at() {
        let mut tag = Tag::new("Work", None);
        tag.set_label("Home");
        assert_eq!(tag.label, "Home");
        assert!(tag.updated_at >= tag.created_at);
    }

    // TC-TAG-003
    #[test]
    fn set_color_changes_color_and_bumps_updated_at() {
        let mut tag = Tag::new("Work", None);
        tag.set_color(Some("#00ff00".to_string()));
        assert_eq!(tag.color.as_deref(), Some("#00ff00"));
        assert!(tag.updated_at >= tag.created_at);
    }
}
