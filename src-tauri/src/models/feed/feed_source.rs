use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Text,
    Image,
    Video,
    Audio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
