use std::collections::{HashSet, VecDeque};

use rusqlite::{params, Connection};

use super::nt_memory_types::*;

pub fn subgraph(
    conn: &Connection,
    center_id: &str,
    depth: usize,
) -> rusqlite::Result<(Vec<KnowledgeNode>, Vec<KnowledgeEdge>)> {
    let mut node_ids = HashSet::new();
    let mut edge_ids = HashSet::new();
    let mut frontier = VecDeque::new();

    node_ids.insert(center_id.to_string());
    frontier.push_back((center_id.to_string(), 0));

    while let Some((current_id, current_depth)) = frontier.pop_front() {
        if current_depth >= depth {
            continue;
        }
        let edges = super::nt_memory_store::get_edges_for_node(conn, &current_id)?;
        for edge in &edges {
            let neighbor = if edge.source_id == current_id {
                &edge.target_id
            } else {
                &edge.source_id
            };
            edge_ids.insert(edge.id.clone());
            if node_ids.insert(neighbor.to_string()) {
                frontier.push_back((neighbor.to_string(), current_depth + 1));
            }
        }
    }

    let mut nodes = Vec::new();
    for nid in &node_ids {
        if let Some(node) = super::nt_memory_store::get_node(conn, nid)? {
            nodes.push(node);
        }
    }

    let mut edges = Vec::new();
    for eid in &edge_ids {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relation_type, weight, description, created_at, metadata
             FROM edges WHERE id=?1",
        )?;
        let mut rows = stmt.query(params![eid])?;
        if let Some(row) = rows.next()? {
            edges.push(KnowledgeEdge {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                relation_type: RelationType::from_str(&row.get::<_, String>(3)?),
                weight: row.get(4)?,
                description: row.get(5)?,
                created_at: row.get(6)?,
                metadata: row.get::<_, Option<String>>(7)?.and_then(|m| serde_json::from_str(&m).ok()),
            });
        }
    }

    Ok((nodes, edges))
}


