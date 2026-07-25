use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::shared::{HexColor, Timestamp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub label: String,
    pub color: Option<HexColor>
}

impl Tag {
    pub fn new(
        label: impl Into<String>,
        color: impl Into<Option<HexColor>>
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            label: label.into(),
            color: color.into()
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
