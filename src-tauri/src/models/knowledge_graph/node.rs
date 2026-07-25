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
