use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::notes::note_block::NoteBlock;
use crate::models::shared::Timestamp;
use crate::models::tag::Tag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub note_blocks: Vec<NoteBlock>,
    pub title: String,
    pub is_pinned: bool,
    pub tags: Vec<Tag>,
}

impl Note {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            note_blocks: Vec::new(),
            title: title.into(),
            is_pinned: false,
            tags: Vec::new(),
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.touch();
    }

    pub fn pin(&mut self) {
        self.is_pinned = true;
        self.touch();
    }

    pub fn unpin(&mut self) {
        self.is_pinned = false;
        self.touch();
    }

    pub fn toggle_pin(&mut self) {
        self.is_pinned = !self.is_pinned;
        self.touch();
    }

    // -- Blocks ----------------------------------------------------------

    pub fn add_block(&mut self, mut block: NoteBlock) {
        block.note_id = Some(self.id.clone());
        self.note_blocks.push(block);
        self.touch();
    }

    pub fn remove_block(&mut self, block_id: &str) -> Option<NoteBlock> {
        let idx = self.note_blocks.iter().position(|b| b.id == block_id)?;
        let block = self.note_blocks.remove(idx);
        self.touch();
        Some(block)
    }

    pub fn find_block(&self, block_id: &str) -> Option<&NoteBlock> {
        self.note_blocks.iter().find(|b| b.id == block_id)
    }

    pub fn find_block_mut(&mut self, block_id: &str) -> Option<&mut NoteBlock> {
        self.note_blocks.iter_mut().find(|b| b.id == block_id)
    }

    pub fn block_count(&self) -> usize {
        self.note_blocks.len()
    }

    /// Blocks ordered by their `position`; blocks without a position sort last.
    pub fn blocks_ordered(&self) -> Vec<&NoteBlock> {
        let mut blocks: Vec<&NoteBlock> = self.note_blocks.iter().collect();
        blocks.sort_by(|a, b| {
            let a = a.position.unwrap_or(f64::MAX);
            let b = b.position.unwrap_or(f64::MAX);
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        });
        blocks
    }

    // -- Tags ------------------------------------------------------------

    pub fn add_tag(&mut self, tag: Tag) {
        if !self.has_tag(&tag.id) {
            self.tags.push(tag);
            self.touch();
        }
    }

    pub fn remove_tag(&mut self, tag_id: &str) -> bool {
        let before = self.tags.len();
        self.tags.retain(|t| t.id != tag_id);
        let removed = self.tags.len() != before;
        if removed {
            self.touch();
        }
        removed
    }

    pub fn has_tag(&self, tag_id: &str) -> bool {
        self.tags.iter().any(|t| t.id == tag_id)
    }
}
