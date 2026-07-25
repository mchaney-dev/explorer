use serde::{Deserialize, Serialize};
use chrono::Utc;
use ulid::Ulid;

use crate::models::knowledge_graph::connection::Connection;
use crate::models::knowledge_graph::node::Node;
use crate::models::shared::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub id: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub title: String,
    pub description: String,
}

impl Graph {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            id: Ulid::new().to_string(),
            created_at: now,
            updated_at: now,
            nodes: Vec::new(),
            connections: Vec::new(),
            title: title.into(),
            description: description.into(),
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

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.touch();
    }

    // -- Nodes -----------------------------------------------------------

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
        self.touch();
    }

    /// Remove a node and every connection that referenced it, keeping the
    /// graph free of dangling edges.
    pub fn remove_node(&mut self, node_id: &str) -> Option<Node> {
        let idx = self.nodes.iter().position(|n| n.id == node_id)?;
        let node = self.nodes.remove(idx);
        self.connections.retain(|c| !c.touches(node_id));
        self.touch();
        Some(node)
    }

    pub fn find_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn find_node_mut(&mut self, node_id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == node_id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    // -- Connections -----------------------------------------------------

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
        self.touch();
    }

    pub fn remove_connection(&mut self, connection_id: &str) -> Option<Connection> {
        let idx = self.connections.iter().position(|c| c.id == connection_id)?;
        let connection = self.connections.remove(idx);
        self.touch();
        Some(connection)
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Every connection with an endpoint at `node_id`.
    pub fn connections_for(&self, node_id: &str) -> Vec<&Connection> {
        self.connections.iter().filter(|c| c.touches(node_id)).collect()
    }

    /// The nodes directly connected to `node_id` (deduplicated, resolved to
    /// nodes that still exist in the graph).
    pub fn neighbors(&self, node_id: &str) -> Vec<&Node> {
        let mut seen: Vec<&str> = Vec::new();
        let mut out: Vec<&Node> = Vec::new();
        for connection in self.connections.iter() {
            if let Some(other) = connection.other_end(node_id) {
                if !seen.contains(&other) {
                    seen.push(other);
                    if let Some(node) = self.find_node(other) {
                        out.push(node);
                    }
                }
            }
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
