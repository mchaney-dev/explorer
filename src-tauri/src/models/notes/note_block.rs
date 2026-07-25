use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    Text,
    Drawing,
    Image,
    Embed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteBlock {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub note_id: Option<String>,
    pub block_type: Option<BlockType>,
    pub content: String,
    pub position: Option<f64>,
}

impl NoteBlock {
    pub fn new(note_id: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            note_id: Some(note_id.into()),
            block_type: None,
            content: String::new(),
            position: None,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_type(mut self, block_type: BlockType) -> Self {
        self.block_type = Some(block_type);
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn with_position(mut self, position: f64) -> Self {
        self.position = Some(position);
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.touch();
    }

    pub fn set_type(&mut self, block_type: Option<BlockType>) {
        self.block_type = block_type;
        self.touch();
    }

    pub fn set_position(&mut self, position: Option<f64>) {
        self.position = position;
        self.touch();
    }
}
