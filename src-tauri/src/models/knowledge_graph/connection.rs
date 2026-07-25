use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionLabel {
    RelatesTo,
    CausedBy,
    IsA,
    Uses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub source_node_id: Option<String>,
    pub dest_node_id: Option<String>,
    pub label: Option<ConnectionLabel>,
    pub is_directed: bool,
}

impl Connection {
    pub fn new(
        source_node_id: impl Into<String>,
        dest_node_id: impl Into<String>,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            source_node_id: Some(source_node_id.into()),
            dest_node_id: Some(dest_node_id.into()),
            label: None,
            is_directed: false,
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    // -- Builders --------------------------------------------------------

    pub fn with_label(mut self, label: ConnectionLabel) -> Self {
        self.label = Some(label);
        self
    }

    pub fn directed(mut self) -> Self {
        self.is_directed = true;
        self
    }

    // -- Setters ---------------------------------------------------------

    pub fn set_label(&mut self, label: Option<ConnectionLabel>) {
        self.label = label;
        self.touch();
    }

    pub fn set_directed(&mut self, is_directed: bool) {
        self.is_directed = is_directed;
        self.touch();
    }

    /// Swap source and destination endpoints (only meaningful when directed).
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.source_node_id, &mut self.dest_node_id);
        self.touch();
    }

    // -- Queries ---------------------------------------------------------

    pub fn touches(&self, node_id: &str) -> bool {
        self.source_node_id.as_deref() == Some(node_id)
            || self.dest_node_id.as_deref() == Some(node_id)
    }

    /// Given one endpoint, return the node on the other end (if set).
    pub fn other_end(&self, node_id: &str) -> Option<&str> {
        if self.source_node_id.as_deref() == Some(node_id) {
            self.dest_node_id.as_deref()
        } else if self.dest_node_id.as_deref() == Some(node_id) {
            self.source_node_id.as_deref()
        } else {
            None
        }
    }
}
