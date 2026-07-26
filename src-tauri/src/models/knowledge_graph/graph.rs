use chrono::Utc;
use serde::{Deserialize, Serialize};
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
        let idx = self
            .connections
            .iter()
            .position(|c| c.id == connection_id)?;
        let connection = self.connections.remove(idx);
        self.touch();
        Some(connection)
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }

    /// Every connection with an endpoint at `node_id`.
    pub fn connections_for(&self, node_id: &str) -> Vec<&Connection> {
        self.connections
            .iter()
            .filter(|c| c.touches(node_id))
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::knowledge_graph::connection::Connection;
    use crate::models::knowledge_graph::node::{Node, NodeType};

    fn node() -> Node {
        Node::new("N", "", NodeType::Text)
    }

    // TC-GRAPH-001
    #[test]
    fn new_is_empty() {
        let g = Graph::new("G", "d");
        assert!(g.nodes.is_empty());
        assert!(g.connections.is_empty());
    }

    // TC-GRAPH-002
    #[test]
    fn set_title_changes_title() {
        let mut g = Graph::new("G", "d");
        g.set_title("New");
        assert_eq!(g.title, "New");
        assert!(g.updated_at >= g.created_at);
    }

    // TC-GRAPH-003
    #[test]
    fn set_description_changes_description() {
        let mut g = Graph::new("G", "d");
        g.set_description("New");
        assert_eq!(g.description, "New");
        assert!(g.updated_at >= g.created_at);
    }

    // TC-GRAPH-004
    #[test]
    fn add_node_appends() {
        let mut g = Graph::new("G", "d");
        g.add_node(node());
        assert_eq!(g.node_count(), 1);
    }

    // TC-GRAPH-005
    #[test]
    fn remove_node_removes_it() {
        let mut g = Graph::new("G", "d");
        let n = node();
        let id = n.id.clone();
        g.add_node(n);
        assert!(g.remove_node(&id).is_some());
        assert_eq!(g.node_count(), 0);
    }

    // TC-GRAPH-006
    #[test]
    fn remove_node_also_removes_touching_connections() {
        let mut g = Graph::new("G", "d");
        let a = node();
        let b = node();
        let (aid, bid) = (a.id.clone(), b.id.clone());
        g.add_node(a);
        g.add_node(b);
        g.add_connection(Connection::new(&aid, &bid));
        g.remove_node(&aid);
        assert_eq!(
            g.connection_count(),
            0,
            "edges touching a removed node must go"
        );
    }

    // TC-GRAPH-007
    #[test]
    fn remove_node_absent_returns_none() {
        let mut g = Graph::new("G", "d");
        assert!(g.remove_node("nope").is_none());
    }

    // TC-GRAPH-008
    #[test]
    fn find_node_and_mut() {
        let mut g = Graph::new("G", "d");
        let n = node();
        let id = n.id.clone();
        g.add_node(n);
        assert!(g.find_node(&id).is_some());
        g.find_node_mut(&id).unwrap().set_title("Edited");
        assert_eq!(g.find_node(&id).unwrap().title, "Edited");
    }

    // TC-GRAPH-009
    #[test]
    fn add_connection_appends() {
        let mut g = Graph::new("G", "d");
        g.add_connection(Connection::new("a", "b"));
        assert_eq!(g.connection_count(), 1);
    }

    // TC-GRAPH-010
    #[test]
    fn remove_connection_present_returns_it() {
        let mut g = Graph::new("G", "d");
        let c = Connection::new("a", "b");
        let id = c.id.clone();
        g.add_connection(c);
        assert!(g.remove_connection(&id).is_some());
    }

    // TC-GRAPH-011
    #[test]
    fn remove_connection_absent_returns_none() {
        let mut g = Graph::new("G", "d");
        assert!(g.remove_connection("nope").is_none());
    }

    // TC-GRAPH-012
    #[test]
    fn connections_for_returns_edges_at_node() {
        let mut g = Graph::new("G", "d");
        g.add_connection(Connection::new("a", "b"));
        g.add_connection(Connection::new("a", "c"));
        g.add_connection(Connection::new("b", "c"));
        assert_eq!(g.connections_for("a").len(), 2);
    }

    // TC-GRAPH-013
    #[test]
    fn neighbors_returns_connected_nodes_deduped() {
        let mut g = Graph::new("G", "d");
        let a = node();
        let b = node();
        let c = node();
        let (aid, bid, cid) = (a.id.clone(), b.id.clone(), c.id.clone());
        g.add_node(a);
        g.add_node(b);
        g.add_node(c);
        g.add_connection(Connection::new(&aid, &bid));
        g.add_connection(Connection::new(&aid, &cid));
        g.add_connection(Connection::new(&aid, &bid)); // duplicate edge
        let neighbors = g.neighbors(&aid);
        assert_eq!(neighbors.len(), 2);
    }

    // TC-GRAPH-014
    #[test]
    fn neighbors_skips_missing_nodes() {
        let mut g = Graph::new("G", "d");
        let a = node();
        let aid = a.id.clone();
        g.add_node(a);
        g.add_connection(Connection::new(&aid, "ghost"));
        assert!(g.neighbors(&aid).is_empty());
    }

    // TC-GRAPH-015
    #[test]
    fn is_empty_reflects_nodes() {
        let mut g = Graph::new("G", "d");
        assert!(g.is_empty());
        g.add_node(node());
        assert!(!g.is_empty());
    }
}
