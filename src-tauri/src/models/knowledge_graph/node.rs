use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::models::shared::Timestamp;
use crate::models::tag::Tag;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    File,
    Video,
    Audio,
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub node_type: NodeType,
    pub x: i32,
    pub y: i32,
    pub title: String,
    pub content: String,
    pub tags: Vec<Tag>,
}

impl Node {
    pub fn new(title: impl Into<String>, content: impl Into<String>, node_type: NodeType) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            node_type,
            x: 0,
            y: 0,
            title: title.into(),
            content: content.into(),
            tags: Vec::new(),
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
        self.touch();
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.touch();
    }

    pub fn set_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
        self.touch();
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        self.touch();
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
        self.touch();
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
    use crate::models::tag::Tag;

    // TC-NODE-001
    #[test]
    fn new_defaults() {
        let n = Node::new("Title", "Content", NodeType::Text);
        assert_eq!(n.x, 0);
        assert_eq!(n.y, 0);
        assert!(n.tags.is_empty());
        assert_eq!(n.created_at, n.updated_at);
    }

    // TC-NODE-002
    #[test]
    fn with_position_sets_coords() {
        let n = Node::new("T", "", NodeType::Text).with_position(3, 4);
        assert_eq!((n.x, n.y), (3, 4));
    }

    // TC-NODE-003
    #[test]
    fn set_title_changes_title() {
        let mut n = Node::new("T", "", NodeType::Text);
        n.set_title("New");
        assert_eq!(n.title, "New");
        assert!(n.updated_at >= n.created_at);
    }

    // TC-NODE-004
    #[test]
    fn set_content_changes_content() {
        let mut n = Node::new("T", "", NodeType::Text);
        n.set_content("New");
        assert_eq!(n.content, "New");
        assert!(n.updated_at >= n.created_at);
    }

    // TC-NODE-005
    #[test]
    fn set_type_changes_type() {
        let mut n = Node::new("T", "", NodeType::Text);
        n.set_type(NodeType::Image);
        assert!(matches!(n.node_type, NodeType::Image));
        assert!(n.updated_at >= n.created_at);
    }

    // TC-NODE-006
    #[test]
    fn set_position_sets_coords() {
        let mut n = Node::new("T", "", NodeType::Text);
        n.set_position(5, 6);
        assert_eq!((n.x, n.y), (5, 6));
        assert!(n.updated_at >= n.created_at);
    }

    // TC-NODE-007
    #[test]
    fn move_by_adds_delta() {
        let mut n = Node::new("T", "", NodeType::Text).with_position(10, 10);
        n.move_by(-3, 5);
        assert_eq!((n.x, n.y), (7, 15));
    }

    // TC-NODE-008
    #[test]
    fn add_tag_adds() {
        let mut n = Node::new("T", "", NodeType::Text);
        n.add_tag(Tag::new("t", None));
        assert_eq!(n.tags.len(), 1);
    }

    // TC-NODE-009
    #[test]
    fn add_tag_ignores_duplicate() {
        let mut n = Node::new("T", "", NodeType::Text);
        let tag = Tag::new("t", None);
        n.add_tag(tag.clone());
        n.add_tag(tag);
        assert_eq!(n.tags.len(), 1);
    }

    // TC-NODE-010
    #[test]
    fn remove_tag_present_returns_true() {
        let mut n = Node::new("T", "", NodeType::Text);
        let tag = Tag::new("t", None);
        let id = tag.id.clone();
        n.add_tag(tag);
        assert!(n.remove_tag(&id));
    }

    // TC-NODE-011
    #[test]
    fn remove_tag_absent_returns_false() {
        let mut n = Node::new("T", "", NodeType::Text);
        assert!(!n.remove_tag("nope"));
    }

    // TC-NODE-012
    #[test]
    fn has_tag_reflects_membership() {
        let mut n = Node::new("T", "", NodeType::Text);
        let tag = Tag::new("t", None);
        let id = tag.id.clone();
        n.add_tag(tag);
        assert!(n.has_tag(&id));
        assert!(!n.has_tag("nope"));
    }
}
