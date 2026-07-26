use rusqlite::{Connection, Row};

use crate::db::error::{DbError, DbResult};
use crate::models::knowledge_graph::connection::{Connection as GraphConnection, ConnectionLabel};
use crate::models::knowledge_graph::graph::Graph;
use crate::models::knowledge_graph::node::{Node, NodeType};
use crate::models::tag::Tag;

fn node_type_to_db(t: &NodeType) -> &'static str {
    match t {
        NodeType::File => "file",
        NodeType::Video => "video",
        NodeType::Audio => "audio",
        NodeType::Text => "text",
        NodeType::Image => "image",
    }
}

fn node_type_from_db(value: &str) -> NodeType {
    match value {
        "video" => NodeType::Video,
        "audio" => NodeType::Audio,
        "text" => NodeType::Text,
        "image" => NodeType::Image,
        _ => NodeType::File,
    }
}

fn label_to_db(l: &ConnectionLabel) -> &'static str {
    match l {
        ConnectionLabel::RelatesTo => "relates_to",
        ConnectionLabel::CausedBy => "caused_by",
        ConnectionLabel::IsA => "is_a",
        ConnectionLabel::Uses => "uses",
    }
}

fn label_from_db(value: Option<String>) -> Option<ConnectionLabel> {
    match value.as_deref() {
        Some("relates_to") => Some(ConnectionLabel::RelatesTo),
        Some("caused_by") => Some(ConnectionLabel::CausedBy),
        Some("is_a") => Some(ConnectionLabel::IsA),
        Some("uses") => Some(ConnectionLabel::Uses),
        _ => None,
    }
}

fn row_to_graph(row: &Row) -> rusqlite::Result<Graph> {
    Ok(Graph {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        nodes: Vec::new(),
        connections: Vec::new(),
        title: row.get("title")?,
        description: row.get("description")?,
    })
}

fn row_to_node(row: &Row) -> rusqlite::Result<Node> {
    Ok(Node {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        node_type: node_type_from_db(&row.get::<_, String>("node_type")?),
        x: row.get("x")?,
        y: row.get("y")?,
        title: row.get("title")?,
        content: row.get("content")?,
        tags: Vec::new(),
    })
}

fn row_to_connection(row: &Row) -> rusqlite::Result<GraphConnection> {
    Ok(GraphConnection {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        source_node_id: row.get("source_node_id")?,
        dest_node_id: row.get("dest_node_id")?,
        label: label_from_db(row.get("label")?),
        is_directed: row.get("is_directed")?,
    })
}

fn row_to_tag(row: &Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        label: row.get("label")?,
        color: row.get("color")?,
    })
}

fn load_node_tags(conn: &Connection, node_id: &str) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.* FROM tags t
           JOIN node_tags nt ON nt.tag_id = t.id
          WHERE nt.node_id = ?1 ORDER BY t.id DESC",
    )?;
    let rows = stmt.query_map([node_id], row_to_tag)?;
    Ok(rows.collect::<rusqlite::Result<Vec<Tag>>>()?)
}

fn load_nodes(conn: &Connection, graph_id: &str) -> DbResult<Vec<Node>> {
    let mut stmt = conn.prepare("SELECT * FROM nodes WHERE graph_id = ?1")?;
    let nodes = stmt
        .query_map([graph_id], row_to_node)?
        .collect::<rusqlite::Result<Vec<Node>>>()?;
    let mut out = Vec::with_capacity(nodes.len());
    for mut node in nodes {
        node.tags = load_node_tags(conn, &node.id)?;
        out.push(node);
    }
    Ok(out)
}

fn load_connections(conn: &Connection, graph_id: &str) -> DbResult<Vec<GraphConnection>> {
    let mut stmt = conn.prepare("SELECT * FROM connections WHERE graph_id = ?1")?;
    let rows = stmt.query_map([graph_id], row_to_connection)?;
    Ok(rows.collect::<rusqlite::Result<Vec<GraphConnection>>>()?)
}

fn write_children(conn: &Connection, graph: &Graph) -> DbResult<()> {
    for node in &graph.nodes {
        conn.execute(
            "INSERT INTO nodes
                (id, created_at, updated_at, graph_id, node_type, x, y, title, content)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                node.id,
                node.created_at,
                node.updated_at,
                graph.id,
                node_type_to_db(&node.node_type),
                node.x,
                node.y,
                node.title,
                node.content,
            ],
        )?;
        for tag in &node.tags {
            conn.execute(
                "INSERT INTO tags (id, created_at, updated_at, label, color)
                 VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO NOTHING",
                rusqlite::params![tag.id, tag.created_at, tag.updated_at, tag.label, tag.color],
            )?;
            conn.execute(
                "INSERT INTO node_tags (node_id, tag_id) VALUES (?1, ?2)
                 ON CONFLICT(node_id, tag_id) DO NOTHING",
                rusqlite::params![node.id, tag.id],
            )?;
        }
    }
    for c in &graph.connections {
        conn.execute(
            "INSERT INTO connections
                (id, created_at, updated_at, graph_id, source_node_id, dest_node_id,
                 label, is_directed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                c.id,
                c.created_at,
                c.updated_at,
                graph.id,
                c.source_node_id,
                c.dest_node_id,
                c.label.as_ref().map(label_to_db),
                c.is_directed,
            ],
        )?;
    }
    Ok(())
}

pub fn insert(conn: &Connection, graph: &Graph) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO graphs (id, created_at, updated_at, title, description)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            graph.id,
            graph.created_at,
            graph.updated_at,
            graph.title,
            graph.description
        ],
    )?;
    write_children(&tx, graph)?;
    tx.commit()?;
    Ok(())
}

pub fn update(conn: &Connection, graph: &Graph) -> DbResult<()> {
    let tx = conn.unchecked_transaction()?;
    let affected = tx.execute(
        "UPDATE graphs SET updated_at = ?2, title = ?3, description = ?4 WHERE id = ?1",
        rusqlite::params![graph.id, graph.updated_at, graph.title, graph.description],
    )?;
    if affected == 0 {
        return Err(DbError::NotFound);
    }
    tx.execute("DELETE FROM connections WHERE graph_id = ?1", [&graph.id])?;
    tx.execute("DELETE FROM nodes WHERE graph_id = ?1", [&graph.id])?;
    write_children(&tx, graph)?;
    tx.commit()?;
    Ok(())
}

pub fn get(conn: &Connection, id: &str) -> DbResult<Graph> {
    let mut graph = conn.query_row("SELECT * FROM graphs WHERE id = ?1", [id], row_to_graph)?;
    graph.nodes = load_nodes(conn, &graph.id)?;
    graph.connections = load_connections(conn, &graph.id)?;
    Ok(graph)
}

pub fn list(conn: &Connection) -> DbResult<Vec<Graph>> {
    let mut stmt = conn.prepare("SELECT * FROM graphs ORDER BY id DESC")?;
    let graphs = stmt
        .query_map([], row_to_graph)?
        .collect::<rusqlite::Result<Vec<Graph>>>()?;
    let mut out = Vec::with_capacity(graphs.len());
    for mut graph in graphs {
        graph.nodes = load_nodes(conn, &graph.id)?;
        graph.connections = load_connections(conn, &graph.id)?;
        out.push(graph);
    }
    Ok(out)
}

pub fn delete(conn: &Connection, id: &str) -> DbResult<()> {
    if conn.execute("DELETE FROM graphs WHERE id = ?1", [id])? == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;
    use crate::models::tag::Tag;

    // TC-DBGRAPH-001
    #[test]
    fn insert_roundtrips_nodes_connections_tags() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut graph = Graph::new("G", "d");
        let mut a = Node::new("A", "", NodeType::Text);
        a.add_tag(Tag::new("x", None));
        let b = Node::new("B", "", NodeType::Image);
        let (aid, bid) = (a.id.clone(), b.id.clone());
        graph.add_node(a);
        graph.add_node(b);
        graph.add_connection(GraphConnection::new(&aid, &bid));
        insert(&conn, &graph).unwrap();
        let got = get(&conn, &graph.id).unwrap();
        assert_eq!(got.nodes.len(), 2);
        assert_eq!(got.connections.len(), 1);
        assert!(got.nodes.iter().any(|n| n.tags.len() == 1));
    }

    // TC-DBGRAPH-002
    #[test]
    fn node_type_and_label_persist() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut graph = Graph::new("G", "d");
        let a = Node::new("A", "", NodeType::Audio);
        let b = Node::new("B", "", NodeType::Video);
        let (aid, bid) = (a.id.clone(), b.id.clone());
        graph.add_node(a);
        graph.add_node(b);
        graph.add_connection(GraphConnection::new(&aid, &bid).with_label(ConnectionLabel::Uses));
        insert(&conn, &graph).unwrap();
        let got = get(&conn, &graph.id).unwrap();
        assert!(got
            .nodes
            .iter()
            .any(|n| matches!(n.node_type, NodeType::Audio)));
        assert!(matches!(
            got.connections[0].label,
            Some(ConnectionLabel::Uses)
        ));
    }

    // TC-DBGRAPH-003
    #[test]
    fn delete_cascades_nodes_and_connections() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut graph = Graph::new("G", "d");
        let a = Node::new("A", "", NodeType::Text);
        let b = Node::new("B", "", NodeType::Text);
        let (aid, bid) = (a.id.clone(), b.id.clone());
        graph.add_node(a);
        graph.add_node(b);
        graph.add_connection(GraphConnection::new(&aid, &bid));
        insert(&conn, &graph).unwrap();
        delete(&conn, &graph.id).unwrap();
        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .unwrap();
        let conns: i64 = conn
            .query_row("SELECT COUNT(*) FROM connections", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nodes, 0);
        assert_eq!(conns, 0);
    }

    // TC-DBGRAPH-004
    #[test]
    fn update_resyncs_children() {
        let db = Db::in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let mut graph = Graph::new("G", "d");
        graph.add_node(Node::new("A", "", NodeType::Text));
        insert(&conn, &graph).unwrap();
        graph.add_node(Node::new("B", "", NodeType::Text));
        update(&conn, &graph).unwrap();
        assert_eq!(get(&conn, &graph.id).unwrap().nodes.len(), 2);
    }
}
