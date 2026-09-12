use std::collections::{HashMap, HashSet, VecDeque};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

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

// ═══════════════════════════════════════════════════════════════════
// P1: Trust-from-Topology — PageRank-like trust computation
// ═══════════════════════════════════════════════════════════════════
// Computes trust scores from graph structure alone (no metadata).
// High in-degree from well-connected nodes → high trust.
// Dangling nodes (no outgoing edges) distribute uniform trust.

/// Trust score result for a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TrustScore {
    pub node_id: String,
    pub trust: f64,
    pub in_degree: usize,
    pub out_degree: usize,
}

/// PageRank-like trust computation over the KB graph.
///
/// Algorithm:
/// 1. Load all nodes and edges into adjacency lists
/// 2. Initialize uniform trust (1/N)
/// 3. Iterate: trust(node) = (1-d)/N + d × Σ(trust(parent) / out_degree(parent))
/// 4. Dangling nodes distribute uniform trust to all nodes
/// 5. Normalize to [0, 1]
///
/// `damping` is the PageRank damping factor (typically 0.85).
/// `max_iter` controls convergence iterations (15-20 usually sufficient).
pub(crate) fn compute_trust_scores(
    conn: &Connection,
    damping: f64,
    max_iter: usize,
) -> rusqlite::Result<Vec<TrustScore>> {
    // Load all node IDs
    let mut node_ids: Vec<String> = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id FROM nodes")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for r in rows.filter_map(|r| r.ok()) {
            node_ids.push(r);
        }
    }
    if node_ids.is_empty() {
        return Ok(Vec::new());
    }
    let n = node_ids.len();
    let id_index: HashMap<&str, usize> = node_ids.iter().enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    // Load edges as adjacency lists
    let mut out_edges: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut in_edges: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    {
        let mut stmt = conn.prepare(
            "SELECT source_id, target_id, weight FROM edges"
        )?;
        let rows = stmt.query_map([], |row| {
            let src: String = row.get(0)?;
            let tgt: String = row.get(1)?;
            let w: f64 = row.get(2)?;
            Ok((src, tgt, w))
        })?;
        for r in rows.filter_map(|r| r.ok()) {
            if let (Some(&si), Some(&ti)) = (id_index.get(r.0.as_str()), id_index.get(r.1.as_str())) {
                out_edges[si].push((ti, r.2));
                in_edges[ti].push((si, r.2));
            }
        }
    }

    // PageRank iteration
    let mut trust = vec![1.0 / n as f64; n];
    let mut dangling_sum: f64;

    for _iter in 0..max_iter {
        // Compute dangling contribution
        dangling_sum = 0.0;
        for i in 0..n {
            if out_edges[i].is_empty() {
                dangling_sum += trust[i];
            }
        }
        let uniform_dangling = dangling_sum / n as f64;

        let mut new_trust = vec![0.0f64; n];
        for i in 0..n {
            let mut incoming = 0.0;
            for &(parent, w) in &in_edges[i] {
                let parent_out_sum: f64 = out_edges[parent].iter().map(|(_, ew)| ew).sum();
                if parent_out_sum > 0.0 {
                    incoming += trust[parent] * w / parent_out_sum;
                }
            }
            new_trust[i] = (1.0 - damping) / n as f64 + damping * (incoming + uniform_dangling);
        }

        // Convergence check (L1 norm)
        let diff: f64 = trust.iter().zip(new_trust.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        trust = new_trust;
        if diff < 1e-6 {
            break;
        }
    }

    // Normalize to [0, 1]
    let max_trust = trust.iter().cloned().fold(0.0f64, f64::max);
    if max_trust > 0.0 {
        for t in trust.iter_mut() {
            *t /= max_trust;
        }
    }

    Ok(node_ids.iter().enumerate().map(|(i, id)| {
        TrustScore {
            node_id: id.clone(),
            trust: trust[i],
            in_degree: in_edges[i].len(),
            out_degree: out_edges[i].len(),
        }
    }).collect())
}

/// Quick trust lookup for a set of node IDs (for search result augmentation).
/// Returns HashMap<node_id, trust_score>.
pub fn trust_for_nodes(
    conn: &Connection,
    target_ids: &[&str],
    damping: f64,
    max_iter: usize,
) -> rusqlite::Result<HashMap<String, f64>> {
    let all_trust = compute_trust_scores(conn, damping, max_iter)?;
    let target_set: HashSet<&str> = target_ids.iter().copied().collect();
    Ok(all_trust.into_iter()
        .filter(|ts| target_set.contains(ts.node_id.as_str()))
        .map(|ts| (ts.node_id, ts.trust))
        .collect())
}


