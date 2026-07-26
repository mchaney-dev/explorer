use chrono::Utc;
use serde::{Deserialize, Serialize};
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

#[cfg(test)]
mod tests {
    use super::*;

    // TC-NOTEBLK-001
    #[test]
    fn new_defaults() {
        let b = NoteBlock::new("note-1");
        assert_eq!(b.note_id.as_deref(), Some("note-1"));
        assert_eq!(b.content, "");
        assert!(b.block_type.is_none());
        assert_eq!(b.created_at, b.updated_at);
    }

    // TC-NOTEBLK-002
    #[test]
    fn with_type_sets_type() {
        let b = NoteBlock::new("n").with_type(BlockType::Text);
        assert!(matches!(b.block_type, Some(BlockType::Text)));
    }

    // TC-NOTEBLK-003
    #[test]
    fn with_content_sets_content() {
        let b = NoteBlock::new("n").with_content("hi");
        assert_eq!(b.content, "hi");
    }

    // TC-NOTEBLK-004
    #[test]
    fn with_position_sets_position() {
        let b = NoteBlock::new("n").with_position(1.5);
        assert_eq!(b.position, Some(1.5));
    }

    // TC-NOTEBLK-005
    #[test]
    fn set_content_changes_content() {
        let mut b = NoteBlock::new("n");
        b.set_content("hi");
        assert_eq!(b.content, "hi");
        assert!(b.updated_at >= b.created_at);
    }

    // TC-NOTEBLK-006
    #[test]
    fn set_type_changes_type() {
        let mut b = NoteBlock::new("n");
        b.set_type(Some(BlockType::Image));
        assert!(matches!(b.block_type, Some(BlockType::Image)));
        assert!(b.updated_at >= b.created_at);
    }

    // TC-NOTEBLK-007
    #[test]
    fn set_position_changes_position() {
        let mut b = NoteBlock::new("n");
        b.set_position(Some(2.0));
        assert_eq!(b.position, Some(2.0));
        assert!(b.updated_at >= b.created_at);
    }
}
