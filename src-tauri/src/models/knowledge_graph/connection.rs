use chrono::Utc;
use serde::{Deserialize, Serialize};
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
    pub fn new(source_node_id: impl Into<String>, dest_node_id: impl Into<String>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    // TC-CONN-001
    #[test]
    fn new_defaults() {
        let c = Connection::new("a", "b");
        assert_eq!(c.source_node_id.as_deref(), Some("a"));
        assert_eq!(c.dest_node_id.as_deref(), Some("b"));
        assert!(!c.is_directed);
        assert!(c.label.is_none());
    }

    // TC-CONN-002
    #[test]
    fn with_label_sets_label() {
        let c = Connection::new("a", "b").with_label(ConnectionLabel::Uses);
        assert!(matches!(c.label, Some(ConnectionLabel::Uses)));
    }

    // TC-CONN-003
    #[test]
    fn directed_sets_flag() {
        let c = Connection::new("a", "b").directed();
        assert!(c.is_directed);
    }

    // TC-CONN-004
    #[test]
    fn set_label_changes_label() {
        let mut c = Connection::new("a", "b");
        c.set_label(Some(ConnectionLabel::IsA));
        assert!(matches!(c.label, Some(ConnectionLabel::IsA)));
        assert!(c.updated_at >= c.created_at);
    }

    // TC-CONN-005
    #[test]
    fn set_directed_changes_flag() {
        let mut c = Connection::new("a", "b");
        c.set_directed(true);
        assert!(c.is_directed);
        assert!(c.updated_at >= c.created_at);
    }

    // TC-CONN-006
    #[test]
    fn reverse_swaps_endpoints() {
        let mut c = Connection::new("a", "b");
        c.reverse();
        assert_eq!(c.source_node_id.as_deref(), Some("b"));
        assert_eq!(c.dest_node_id.as_deref(), Some("a"));
    }

    // TC-CONN-007
    #[test]
    fn touches_true_for_either_endpoint() {
        let c = Connection::new("a", "b");
        assert!(c.touches("a"));
        assert!(c.touches("b"));
    }

    // TC-CONN-008
    #[test]
    fn touches_false_for_unrelated() {
        assert!(!Connection::new("a", "b").touches("z"));
    }

    // TC-CONN-009
    #[test]
    fn other_end_returns_opposite() {
        let c = Connection::new("a", "b");
        assert_eq!(c.other_end("a"), Some("b"));
        assert_eq!(c.other_end("b"), Some("a"));
    }

    // TC-CONN-010
    #[test]
    fn other_end_none_for_non_endpoint() {
        assert_eq!(Connection::new("a", "b").other_end("z"), None);
    }
}
