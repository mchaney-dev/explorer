use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Text,
    Image,
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct FeedSource {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub source_type: Option<SourceType>,
    pub name: String,
    pub base_url: String,
    pub is_enabled: bool,
    pub requires_key: bool,
}

impl FeedSource {
    pub fn new(name: impl Into<String>, base_url: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            source_type: None,
            name: name.into(),
            base_url: base_url.into(),
            is_enabled: true,
            requires_key: false,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_type(mut self, source_type: SourceType) -> Self {
        self.source_type = Some(source_type);
        self
    }

    pub fn requiring_key(mut self) -> Self {
        self.requires_key = true;
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
        self.touch();
    }

    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        self.base_url = base_url.into();
        self.touch();
    }

    pub fn set_type(&mut self, source_type: Option<SourceType>) {
        self.source_type = source_type;
        self.touch();
    }

    pub fn enable(&mut self) {
        self.is_enabled = true;
        self.touch();
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
        self.touch();
    }

    pub fn toggle_enabled(&mut self) {
        self.is_enabled = !self.is_enabled;
        self.touch();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-FEEDSRC-001
    #[test]
    fn new_defaults() {
        let s = FeedSource::new("Blog", "https://blog.example");
        assert!(s.is_enabled);
        assert!(!s.requires_key);
        assert!(s.source_type.is_none());
    }

    // TC-FEEDSRC-002
    #[test]
    fn with_type_sets_type() {
        let s = FeedSource::new("B", "u").with_type(SourceType::Video);
        assert!(matches!(s.source_type, Some(SourceType::Video)));
    }

    // TC-FEEDSRC-003
    #[test]
    fn requiring_key_sets_flag() {
        assert!(FeedSource::new("B", "u").requiring_key().requires_key);
    }

    // TC-FEEDSRC-004
    #[test]
    fn setters_change_fields() {
        let mut s = FeedSource::new("B", "u");
        s.set_name("New");
        s.set_base_url("https://new.example");
        s.set_type(Some(SourceType::Audio));
        assert_eq!(s.name, "New");
        assert_eq!(s.base_url, "https://new.example");
        assert!(matches!(s.source_type, Some(SourceType::Audio)));
        assert!(s.updated_at >= s.created_at);
    }

    // TC-FEEDSRC-005
    #[test]
    fn enable_disable_toggle() {
        let mut s = FeedSource::new("B", "u");
        s.disable();
        assert!(!s.is_enabled);
        s.enable();
        assert!(s.is_enabled);
        s.toggle_enabled();
        assert!(!s.is_enabled);
    }
}
