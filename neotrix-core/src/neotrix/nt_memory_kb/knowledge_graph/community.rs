use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use super::super::nt_memory_store;

use super::types::{BridgeNode, Community, KnowledgeGap, SurprisingConnection};

// ─── Louvain Community Detection ──────────────────────────────────

/// Simplified Louvain algorithm (binary weights only; no refinement phase).
/// Returns node_id → community_id mapping.
pub fn louvain_communities(
    conn: &Connection,
    node_ids: &[String],
    min_community_size: usize,
    max_iterations: usize,
) -> rusqlite::Result<HashMap<String, String>> {
    if node_ids.len() < 2 {
        let mut result = HashMap::new();
        for nid in node_ids {
            result.insert(nid.clone(), "community_0".to_string());
        }
        return Ok(result);
    }

    // Build adjacency list
    let n = node_ids.len();
    let id_to_idx: HashMap<&str, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let _idx_to_id: Vec<&str> = node_ids.iter().map(|s| s.as_str()).collect();

    let mut adjacency: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for (i, nid) in node_ids.iter().enumerate() {
        let edges = nt_memory_store::get_edges_for_node(conn, nid)?;
        for e in &edges {
            let neighbor = if e.source_id == *nid {
                &e.target_id
            } else {
                &e.source_id
            };
            if let Some(&j) = id_to_idx.get(neighbor.as_str()) {
                adjacency[i].push((j, e.weight));
            }
        }
    }

    // Calculate total weight m and node degrees
    let mut total_weight = 0.0_f64;
    let mut node_degree = vec![0.0_f64; n];
    for (i, edges) in adjacency.iter().enumerate() {
        for (_, w) in edges {
            node_degree[i] += w;
            total_weight += w;
        }
    }
    total_weight /= 2.0; // each edge counted twice

    if total_weight == 0.0 {
        // No edges — every node is its own community
        let mut result = HashMap::new();
        for (i, nid) in node_ids.iter().enumerate() {
            result.insert(nid.clone(), format!("community_{}", i));
        }
        return Ok(result);
    }

    // Initialize: each node in its own community
    let mut community_of: Vec<usize> = (0..n).collect();
    let mut community_total: Vec<f64> = node_degree.clone(); // Σ_tot per community
    let mut community_internal: Vec<f64> = vec![0.0_f64; n]; // Σ_in per community

    // Louvain Phase 1 iterations
    let mut improved = true;
    let mut iteration = 0;
    while improved && iteration < max_iterations {
        improved = false;
        iteration += 1;

        for i in 0..n {
            let current_comm = community_of[i];
            let k_i = node_degree[i];

            // Remove i from its community
            community_total[current_comm] -= k_i;
            for &(j, w) in &adjacency[i] {
                if community_of[j] == current_comm {
                    community_internal[current_comm] -= w;
                }
            }

            // Find best community among neighbors
            let mut best_comm = current_comm;
            let mut best_gain = 0.0_f64;

            // Collect neighbor communities
            let mut neighbor_comms: HashMap<usize, f64> = HashMap::new();
            for &(j, w) in &adjacency[i] {
                let comm = community_of[j];
                *neighbor_comms.entry(comm).or_insert(0.0) += w;
            }

            for (&candidate_comm, &k_i_in) in &neighbor_comms {
                let sigma_tot = community_total[candidate_comm];
                let gain = (k_i_in - sigma_tot * k_i / (2.0 * total_weight)) / total_weight;

                if gain > best_gain {
                    best_gain = gain;
                    best_comm = candidate_comm;
                }
            }

            // Move i to best community
            community_of[i] = best_comm;
            community_total[best_comm] += k_i;
            for &(j, w) in &adjacency[i] {
                if community_of[j] == best_comm {
                    community_internal[best_comm] += w;
                }
            }

            if best_comm != current_comm {
                improved = true;
            }
        }
    }

    // Build result: compress community IDs
    let mut comm_counter = 0usize;
    let mut comm_mapping: HashMap<usize, usize> = HashMap::new();
    let mut comp_to_orig: HashMap<usize, String> = HashMap::new();
    for &c in &community_of {
        let entry = comm_mapping.len();
        let compressed = *comm_mapping.entry(c).or_insert(entry);
        comp_to_orig.entry(compressed).or_insert_with(|| {
            format!("community_{}", {
                let id = comm_counter;
                comm_counter += 1;
                id
            })
        });
    }

    let mut result: HashMap<String, String> = HashMap::new();
    for (i, nid) in node_ids.iter().enumerate() {
        let compressed = comm_mapping[&community_of[i]];
        let comm_id = comp_to_orig[&compressed].clone();
        result.insert(nid.clone(), comm_id);
    }

    // Filter out communities smaller than min_community_size: assign them to "unaffiliated"
    let mut comm_sizes: HashMap<String, usize> = HashMap::new();
    for comm_id in result.values() {
        *comm_sizes.entry(comm_id.clone()).or_insert(0) += 1;
    }

    for (_nid, comm_id) in result.iter_mut() {
        if let Some(&size) = comm_sizes.get(comm_id.as_str()) {
            if size < min_community_size {
                *comm_id = "unaffiliated".to_string();
            }
        }
    }

    Ok(result)
}

// ─── Graph Insights ───────────────────────────────────────────────

/// Find surprising connections: edges between nodes in different communities with high weight.
pub fn find_surprising_connections(
    conn: &Connection,
    community_map: &HashMap<String, String>,
    top_k: usize,
) -> rusqlite::Result<Vec<SurprisingConnection>> {
    let mut candidates = Vec::new();

    for (node_id, comm_id) in community_map {
        if comm_id == "unaffiliated" {
            continue;
        }
        let edges = nt_memory_store::get_edges_for_node(conn, node_id)?;
        for e in &edges {
            let neighbor = if e.source_id == *node_id {
                &e.target_id
            } else {
                &e.source_id
            };
            if let Some(neighbor_comm) = community_map.get(neighbor.as_str()) {
                if neighbor_comm != comm_id && neighbor_comm != "unaffiliated" {
                    let node = match nt_memory_store::get_node(conn, node_id)? {
                        Some(n) => n,
                        None => continue,
                    };
                    let neighbor_node = match nt_memory_store::get_node(conn, neighbor)? {
                        Some(n) => n,
                        None => continue,
                    };
                    candidates.push(SurprisingConnection {
                        source_id: node_id.clone(),
                        source_title: node.title.clone(),
                        target_id: neighbor.clone(),
                        target_title: neighbor_node.title.clone(),
                        edge_id: e.id.clone(),
                        weight: e.weight,
                        source_community: comm_id.clone(),
                        target_community: neighbor_comm.clone(),
                    });
                }
            }
        }
    }

    // Sort by weight descending, take top_k
    candidates.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.truncate(top_k);

    // Deduplicate: keep only the highest-weight for each source→target pair
    let mut seen = HashSet::new();
    let mut deduped = Vec::with_capacity(candidates.len());
    for c in candidates {
        let key = if c.source_id < c.target_id {
            format!("{}→{}", c.source_id, c.target_id)
        } else {
            format!("{}→{}", c.target_id, c.source_id)
        };
        if seen.insert(key) {
            deduped.push(c);
        }
    }

    Ok(deduped)
}

/// Find knowledge gaps: nodes with few or no connections, isolated from the main graph.
pub fn find_knowledge_gaps(
    conn: &Connection,
    community_map: &HashMap<String, String>,
    top_k: usize,
) -> rusqlite::Result<Vec<KnowledgeGap>> {
    let mut gaps = Vec::new();

    for (node_id, comm_id) in community_map {
        let node = match nt_memory_store::get_node(conn, node_id)? {
            Some(n) => n,
            None => continue,
        };

        let edges = nt_memory_store::get_edges_for_node(conn, node_id)?;
        let edge_count = edges.len();

        let unaffiliated = comm_id == "unaffiliated";
        let isolation_score = if edge_count == 0 {
            1.0
        } else if unaffiliated {
            0.7
        } else {
            // In a large community — compute relative to community size
            let community_size = community_map.values().filter(|c| *c == comm_id).count();
            if community_size > 1 {
                1.0 - (edge_count as f64 / community_size as f64).min(1.0)
            } else {
                0.8
            }
        };

        if edge_count < 3 || unaffiliated {
            gaps.push(KnowledgeGap {
                node_id: node_id.clone(),
                node_title: node.title.clone(),
                node_type: node.node_type.as_str().to_string(),
                edge_count,
                isolation_score,
            });
        }
    }

    // Sort by isolation_score descending
    gaps.sort_by(|a, b| {
        b.isolation_score
            .partial_cmp(&a.isolation_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    gaps.truncate(top_k);

    Ok(gaps)
}

/// Find bridge nodes: nodes with high degree that connect multiple communities.
pub fn find_bridge_nodes(
    conn: &Connection,
    community_map: &HashMap<String, String>,
    top_k: usize,
) -> rusqlite::Result<Vec<BridgeNode>> {
    // Count communities per node and compute bridge score
    let node_comm: HashMap<&str, &str> = community_map
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let mut bridge_candidates = Vec::new();

    for (node_id_str, _comm_id) in community_map {
        let node_id = node_id_str.as_str();
        let edges = match nt_memory_store::get_edges_for_node(conn, node_id) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let degree = edges.len();
        if degree == 0 {
            continue;
        }

        let neighbor_comms: HashSet<&str> = edges
            .iter()
            .filter_map(|e| {
                let neighbor = if e.source_id == node_id {
                    &e.target_id
                } else {
                    &e.source_id
                };
                node_comm.get(neighbor.as_str()).copied()
            })
            .collect();

        let community_count = neighbor_comms.len();
        let node = match nt_memory_store::get_node(conn, node_id)? {
            Some(n) => n,
            None => continue,
        };

        // Bridge score: degree × community_count / (community_count + 1)
        let bridge_score =
            (degree as f64) * (community_count as f64) / (community_count as f64 + 1.0);

        if community_count > 1 || degree > 5 {
            bridge_candidates.push(BridgeNode {
                node_id: node.id.clone(),
                node_title: node.title.clone(),
                node_type: node.node_type.as_str().to_string(),
                degree,
                community_count,
                bridge_score,
            });
        }
    }

    bridge_candidates.sort_by(|a, b| {
        b.bridge_score
            .partial_cmp(&a.bridge_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    bridge_candidates.truncate(top_k);

    Ok(bridge_candidates)
}

/// Build community summary from node_id→community_id mapping.
pub fn build_community_summary(
    conn: &Connection,
    community_map: &HashMap<String, String>,
) -> rusqlite::Result<Vec<Community>> {
    // Group node IDs by community
    let mut comm_nodes: HashMap<&str, Vec<String>> = HashMap::new();
    for (nid, comm_id) in community_map {
        comm_nodes
            .entry(comm_id.as_str())
            .or_default()
            .push(nid.clone());
    }

    let mut communities = Vec::with_capacity(comm_nodes.len());

    for (&comm_id, node_ids) in &comm_nodes {
        let size = node_ids.len();
        if size == 0 {
            continue;
        }

        // Count internal edges and collect types
        let mut internal_edges = 0usize;
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for nid in node_ids {
            if let Ok(Some(node)) = nt_memory_store::get_node(conn, nid) {
                *type_counts
                    .entry(node.node_type.as_str().to_string())
                    .or_insert(0) += 1;
            }
            if let Ok(edges) = nt_memory_store::get_edges_for_node(conn, nid) {
                for e in &edges {
                    let neighbor = if e.source_id == *nid {
                        &e.target_id
                    } else {
                        &e.source_id
                    };
                    if node_ids.contains(neighbor) {
                        internal_edges += 1;
                    }
                }
            }
        }

        // Each internal edge counted twice (once from each end), so divide by 2
        internal_edges /= 2;

        let max_possible = if size > 1 { size * (size - 1) / 2 } else { 1 };
        let cohesion = if max_possible > 0 {
            internal_edges as f64 / max_possible as f64
        } else {
            0.0
        };

        let mut dominant_types: Vec<(String, usize)> = type_counts.into_iter().collect();
        dominant_types.sort_by(|a, b| b.1.cmp(&a.1));

        communities.push(Community {
            id: comm_id.to_string(),
            node_ids: node_ids.clone(),
            size,
            internal_edges,
            cohesion,
            dominant_types,
        });
    }

    // Sort by size descending
    communities.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(communities)
}
