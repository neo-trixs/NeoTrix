use std::collections::{HashMap, HashSet, VecDeque};

use rusqlite::{params, Connection};

use super::nt_memory_types::*;

pub fn shortest_path(
    conn: &Connection,
    from_id: &str,
    to_id: &str,
    max_depth: usize,
) -> rusqlite::Result<Option<GraphPath>> {
    if from_id == to_id {
        let node = super::nt_memory_store::get_node(conn, from_id)?.unwrap();
        return Ok(Some(GraphPath {
            nodes: vec![node],
            edges: vec![],
            total_distance: 0.0,
        }));
    }

    let mut visited: HashMap<String, (String, String, f64)> = HashMap::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    visited.insert(from_id.to_string(), (String::new(), String::new(), 0.0));
    queue.push_back(from_id.to_string());

    let mut found = false;
    while let Some(current) = queue.pop_front() {
        let dist = visited[&current].2;
        if dist as usize >= max_depth {
            continue;
        }

        let edges = super::nt_memory_store::get_edges_for_node(conn, &current)?;
        for edge in &edges {
            let neighbor = if edge.source_id == current {
                &edge.target_id
            } else {
                &edge.source_id
            };

            if !visited.contains_key(neighbor) {
                let new_dist = dist + edge.weight;
                visited.insert(
                    neighbor.to_string(),
                    (current.clone(), edge.id.clone(), new_dist),
                );
                if neighbor == to_id {
                    found = true;
                    break;
                }
                queue.push_back(neighbor.to_string());
            }
        }
        if found {
            break;
        }
    }

    if !found {
        return Ok(None);
    }

    let mut path_node_ids: Vec<String> = Vec::new();
    let mut path_edge_ids: Vec<String> = Vec::new();
    let mut cur = to_id.to_string();

    while cur != from_id {
        path_node_ids.push(cur.clone());
        if let Some((prev, edge_id, _)) = visited.get(&cur) {
            path_edge_ids.push(edge_id.clone());
            cur = prev.clone();
        } else {
            break;
        }
    }
    path_node_ids.push(from_id.to_string());
    path_node_ids.reverse();
    path_edge_ids.reverse();

    let mut nodes = Vec::new();
    for nid in &path_node_ids {
        if let Some(node) = super::nt_memory_store::get_node(conn, nid)? {
            nodes.push(node);
        }
    }

    let mut edges = Vec::new();
    for eid in &path_edge_ids {
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

    let total_distance = edges.iter().map(|e| e.weight).sum();

    Ok(Some(GraphPath {
        nodes,
        edges,
        total_distance,
    }))
}

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

pub fn community_detection(
    conn: &Connection,
    min_community_size: usize,
) -> rusqlite::Result<Vec<Vec<KnowledgeNode>>> {
    let mut stmt = conn.prepare("SELECT id FROM nodes LIMIT 1000")?;
    let all_ids: Vec<String> = stmt.query_map([], |r| r.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    let mut visited = HashSet::new();
    let mut communities = Vec::new();

    for id in &all_ids {
        if visited.contains(id) {
            continue;
        }

        let mut community = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(id.clone());
        visited.insert(id.clone());

        while let Some(current) = queue.pop_front() {
            if let Some(node) = super::nt_memory_store::get_node(conn, &current)? {
                community.push(node);
            }
            let edges = super::nt_memory_store::get_edges_for_node(conn, &current)?;
            for edge in &edges {
                let neighbor = if edge.source_id == current {
                    &edge.target_id
                } else {
                    &edge.source_id
                };
                if visited.insert(neighbor.to_string()) {
                    queue.push_back(neighbor.to_string());
                }
            }
        }

        if community.len() >= min_community_size {
            communities.push(community);
        }
    }

    Ok(communities)
}
