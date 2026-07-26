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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notes::note_block::NoteBlock;
    use crate::models::tag::Tag;

    // TC-NOTE-001
    #[test]
    fn new_defaults() {
        let n = Note::new("My note");
        assert!(n.note_blocks.is_empty());
        assert!(!n.is_pinned);
        assert!(n.tags.is_empty());
    }

    // TC-NOTE-002
    #[test]
    fn set_title_changes_title() {
        let mut n = Note::new("A");
        n.set_title("B");
        assert_eq!(n.title, "B");
        assert!(n.updated_at >= n.created_at);
    }

    // TC-NOTE-003
    #[test]
    fn pin_sets_flag() {
        let mut n = Note::new("A");
        n.pin();
        assert!(n.is_pinned);
    }

    // TC-NOTE-004
    #[test]
    fn unpin_clears_flag() {
        let mut n = Note::new("A");
        n.pin();
        n.unpin();
        assert!(!n.is_pinned);
    }

    // TC-NOTE-005
    #[test]
    fn toggle_pin_flips() {
        let mut n = Note::new("A");
        n.toggle_pin();
        assert!(n.is_pinned);
        n.toggle_pin();
        assert!(!n.is_pinned);
    }

    // TC-NOTE-006
    #[test]
    fn add_block_appends_and_stamps_note_id() {
        let mut n = Note::new("A");
        n.add_block(NoteBlock::new("other-note"));
        assert_eq!(n.note_blocks.len(), 1);
        assert_eq!(n.note_blocks[0].note_id.as_deref(), Some(n.id.as_str()));
    }

    // TC-NOTE-007
    #[test]
    fn remove_block_present_returns_it() {
        let mut n = Note::new("A");
        let block = NoteBlock::new(&n.id);
        let id = block.id.clone();
        n.add_block(block);
        let removed = n.remove_block(&id);
        assert!(removed.is_some());
        assert!(n.note_blocks.is_empty());
    }

    // TC-NOTE-008
    #[test]
    fn remove_block_absent_returns_none() {
        let mut n = Note::new("A");
        assert!(n.remove_block("nope").is_none());
    }

    // TC-NOTE-009
    #[test]
    fn find_block_returns_block() {
        let mut n = Note::new("A");
        let block = NoteBlock::new(&n.id);
        let id = block.id.clone();
        n.add_block(block);
        assert!(n.find_block(&id).is_some());
    }

    // TC-NOTE-010
    #[test]
    fn find_block_mut_allows_edit() {
        let mut n = Note::new("A");
        let block = NoteBlock::new(&n.id);
        let id = block.id.clone();
        n.add_block(block);
        n.find_block_mut(&id).unwrap().set_content("edited");
        assert_eq!(n.find_block(&id).unwrap().content, "edited");
    }

    // TC-NOTE-011
    #[test]
    fn block_count_is_correct() {
        let mut n = Note::new("A");
        n.add_block(NoteBlock::new(&n.id));
        n.add_block(NoteBlock::new(&n.id));
        assert_eq!(n.block_count(), 2);
    }

    // TC-NOTE-012
    #[test]
    fn blocks_ordered_sorts_by_position_unpositioned_last() {
        let mut n = Note::new("A");
        n.add_block(NoteBlock::new(&n.id).with_position(2.0));
        n.add_block(NoteBlock::new(&n.id)); // no position
        n.add_block(NoteBlock::new(&n.id).with_position(1.0));
        let ordered = n.blocks_ordered();
        assert_eq!(ordered[0].position, Some(1.0));
        assert_eq!(ordered[1].position, Some(2.0));
        assert_eq!(ordered[2].position, None);
    }

    // TC-NOTE-013
    #[test]
    fn add_tag_adds() {
        let mut n = Note::new("A");
        n.add_tag(Tag::new("t", None));
        assert_eq!(n.tags.len(), 1);
    }

    // TC-NOTE-014
    #[test]
    fn add_tag_ignores_duplicate() {
        let mut n = Note::new("A");
        let tag = Tag::new("t", None);
        n.add_tag(tag.clone());
        n.add_tag(tag);
        assert_eq!(n.tags.len(), 1);
    }

    // TC-NOTE-015
    #[test]
    fn remove_tag_present_returns_true() {
        let mut n = Note::new("A");
        let tag = Tag::new("t", None);
        let id = tag.id.clone();
        n.add_tag(tag);
        assert!(n.remove_tag(&id));
    }

    // TC-NOTE-016
    #[test]
    fn remove_tag_absent_returns_false() {
        let mut n = Note::new("A");
        assert!(!n.remove_tag("nope"));
    }

    // TC-NOTE-017
    #[test]
    fn has_tag_reflects_membership() {
        let mut n = Note::new("A");
        let tag = Tag::new("t", None);
        let id = tag.id.clone();
        n.add_tag(tag);
        assert!(n.has_tag(&id));
        assert!(!n.has_tag("nope"));
    }
}
