use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use super::super::nt_memory_store;

use super::types::RelevanceSignals;

// ─── 4-Signal Relevance Model ──────────────────────────────────────

/// 4-signal relevance: direct_link ×3.0, source_overlap ×4.0, Adamic-Adar ×1.5, type_affinity ×1.0
pub fn compute_relevance(
    conn: &Connection,
    source_id: &str,
    candidate_ids: &[String],
) -> rusqlite::Result<Vec<(String, RelevanceSignals)>> {
    if candidate_ids.is_empty() {
        return Ok(Vec::new());
    }

    let source = match nt_memory_store::get_node(conn, source_id)? {
        Some(n) => n,
        None => return Ok(Vec::new()),
    };

    // Preload edges for source once
    let source_edges = nt_memory_store::get_edges_for_node(conn, source_id)?;
    let source_neighbors: HashSet<String> = source_edges
        .iter()
        .map(|e| {
            if e.source_id == source_id {
                e.target_id.clone()
            } else {
                e.source_id.clone()
            }
        })
        .collect();
    let source_edge_map: HashMap<&str, f64> = source_edges
        .iter()
        .map(|e| {
            let neighbor = if e.source_id == source_id {
                &e.target_id
            } else {
                &e.source_id
            };
            (neighbor.as_str(), e.weight)
        })
        .collect();

    // Preload candidates in batch
    let mut candidate_nodes = Vec::with_capacity(candidate_ids.len());
    for cid in candidate_ids {
        if let Some(n) = nt_memory_store::get_node(conn, cid)? {
            candidate_nodes.push(n);
        }
    }

    // Preload neighbor sets for Adamic-Adar: candidate -> neighbor set + degree
    let mut candidate_adj: HashMap<String, (HashSet<String>, Vec<(String, f64)>)> = HashMap::new();
    for cn in &candidate_nodes {
        let edges = nt_memory_store::get_edges_for_node(conn, &cn.id)?;
        let neighbors: HashSet<String> = edges
            .iter()
            .map(|e| {
                if e.source_id == cn.id {
                    e.target_id.clone()
                } else {
                    e.source_id.clone()
                }
            })
            .collect();
        let weighted: Vec<(String, f64)> = edges
            .iter()
            .map(|e| {
                let nid = if e.source_id == cn.id {
                    e.target_id.clone()
                } else {
                    e.source_id.clone()
                };
                (nid, e.weight)
            })
            .collect();
        candidate_adj.insert(cn.id.clone(), (neighbors, weighted));
    }

    let mut results = Vec::with_capacity(candidate_nodes.len());

    for cn in &candidate_nodes {
        if cn.id == source_id {
            results.push((
                cn.id.clone(),
                RelevanceSignals {
                    direct_link: 0.0,
                    source_overlap: 0.0,
                    adamic_adar: 0.0,
                    type_affinity: 0.0,
                    combined: 0.0,
                },
            ));
            continue;
        }

        // Signal 1: Direct link (×3.0)
        let direct = if source_neighbors.contains(&cn.id) {
            source_edge_map.get(cn.id.as_str()).copied().unwrap_or(1.0) * 3.0
        } else {
            0.0
        };

        // Signal 2: Source overlap (×4.0)
        let source_overlap = if source.url.is_some()
            && cn.url.is_some()
            && source.url.as_deref() == cn.url.as_deref()
        {
            4.0
        } else if source.domain.is_some()
            && cn.domain.is_some()
            && source.domain.as_deref() == cn.domain.as_deref()
        {
            2.0
        } else {
            0.0
        };

        // Signal 3: Adamic-Adar (×1.5)
        let aa = compute_adamic_adar_from_adj(
            &source_neighbors,
            &candidate_adj
                .get(&cn.id)
                .map(|(ns, _)| ns)
                .cloned()
                .unwrap_or_default(),
            &candidate_adj,
            conn,
        ) * 1.5;

        // Signal 4: Type affinity (×1.0)
        let type_aff = if source.node_type == cn.node_type {
            1.0
        } else {
            0.0
        };

        let combined = direct + source_overlap + aa + type_aff;

        results.push((
            cn.id.clone(),
            RelevanceSignals {
                direct_link: direct,
                source_overlap,
                adamic_adar: aa,
                type_affinity: type_aff,
                combined,
            },
        ));
    }

    // Sort by combined descending
    results.sort_by(|a, b| {
        b.1.combined
            .partial_cmp(&a.1.combined)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}

fn compute_adamic_adar_from_adj(
    source_neighbors: &HashSet<String>,
    candidate_neighbors: &HashSet<String>,
    candidate_adj: &HashMap<String, (HashSet<String>, Vec<(String, f64)>)>,
    _conn: &Connection,
) -> f64 {
    let common: Vec<&str> = source_neighbors
        .intersection(candidate_neighbors)
        .map(|s| s.as_str())
        .collect();

    if common.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;
    for neighbor_id in &common {
        let degree = candidate_adj
            .get(*neighbor_id)
            .map(|(_, edges)| edges.len())
            .unwrap_or(0);
        if degree > 1 {
            score += 1.0 / (degree as f64).ln();
        }
    }

    score
}
