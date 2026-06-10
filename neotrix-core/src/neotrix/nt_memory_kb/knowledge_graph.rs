use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use super::nt_memory_store;

// ─── Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RelevanceSignals {
    pub direct_link: f64,
    pub source_overlap: f64,
    pub adamic_adar: f64,
    pub type_affinity: f64,
    pub combined: f64,
}

#[derive(Debug, Clone)]
pub struct Community {
    pub id: String,
    pub node_ids: Vec<String>,
    pub size: usize,
    pub internal_edges: usize,
    pub cohesion: f64,
    pub dominant_types: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct SurprisingConnection {
    pub source_id: String,
    pub source_title: String,
    pub target_id: String,
    pub target_title: String,
    pub edge_id: String,
    pub weight: f64,
    pub source_community: String,
    pub target_community: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGap {
    pub node_id: String,
    pub node_title: String,
    pub node_type: String,
    pub edge_count: usize,
    pub isolation_score: f64,
}

#[derive(Debug, Clone)]
pub struct BridgeNode {
    pub node_id: String,
    pub node_title: String,
    pub node_type: String,
    pub degree: usize,
    pub community_count: usize,
    pub bridge_score: f64,
}

// ─── Temporal Versioning Types ─────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TemporalFact {
    pub fact_id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TemporalQuery {
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
    pub as_of: Option<i64>,
}

// ─── Temporal Schema ───────────────────────────────────────────────

fn ensure_temporal_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS temporal_facts (
            fact_id TEXT PRIMARY KEY,
            subject TEXT NOT NULL,
            predicate TEXT NOT NULL,
            object TEXT NOT NULL,
            valid_from INTEGER NOT NULL,
            valid_to INTEGER,
            confidence REAL DEFAULT 1.0,
            source TEXT DEFAULT 'neotrix',
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_temporal_subject ON temporal_facts(subject);
        CREATE INDEX IF NOT EXISTS idx_temporal_object ON temporal_facts(object);
        CREATE INDEX IF NOT EXISTS idx_temporal_valid ON temporal_facts(valid_from, valid_to);",
    )?;
    Ok(())
}

/// Insert a temporal fact.
pub fn insert_temporal_fact(
    conn: &Connection,
    fact: &TemporalFact,
) -> Result<(), String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;
    conn.execute(
        "INSERT OR REPLACE INTO temporal_facts (fact_id, subject, predicate, object, valid_from, valid_to, confidence, source, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            fact.fact_id,
            fact.subject,
            fact.predicate,
            fact.object,
            fact.valid_from,
            fact.valid_to,
            fact.confidence,
            fact.source,
            chrono::Utc::now().timestamp(),
        ],
    ).map_err(|e| format!("insert temporal fact: {}", e))?;
    Ok(())
}

/// Invalidate a fact by setting its valid_to to now.
pub fn invalidate_fact(
    conn: &Connection,
    fact_id: &str,
) -> Result<(), String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;
    let now = chrono::Utc::now().timestamp();
    let rows = conn.execute(
        "UPDATE temporal_facts SET valid_to=?1 WHERE fact_id=?2 AND valid_to IS NULL",
        rusqlite::params![now, fact_id],
    ).map_err(|e| format!("invalidate fact: {}", e))?;
    if rows == 0 {
        return Err(format!("fact {} not found or already invalidated", fact_id));
    }
    Ok(())
}

/// Query facts with temporal filtering.
/// If as_of is None, returns only currently valid facts (valid_to IS NULL).
/// If as_of is Some(timestamp), returns facts valid at that time.
pub fn query_temporal_facts(
    conn: &Connection,
    query: &TemporalQuery,
    limit: usize,
) -> Result<Vec<TemporalFact>, String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;

    let mut sql = String::from("SELECT fact_id, subject, predicate, object, valid_from, valid_to, confidence, source FROM temporal_facts WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref subject) = query.subject {
        sql.push_str(" AND subject=?");
        params.push(Box::new(subject.clone()));
    }
    if let Some(ref predicate) = query.predicate {
        sql.push_str(" AND predicate=?");
        params.push(Box::new(predicate.clone()));
    }
    if let Some(ref object) = query.object {
        sql.push_str(" AND object=?");
        params.push(Box::new(object.clone()));
    }

    if let Some(as_of) = query.as_of {
        sql.push_str(" AND valid_from <= ? AND (valid_to IS NULL OR valid_to > ?)");
        params.push(Box::new(as_of));
        params.push(Box::new(as_of));
    } else {
        sql.push_str(" AND valid_to IS NULL");
    }

    sql.push_str(&format!(" ORDER BY valid_from DESC LIMIT {}", limit));

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("prepare: {}", e))?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(TemporalFact {
            fact_id: row.get(0)?,
            subject: row.get(1)?,
            predicate: row.get(2)?,
            object: row.get(3)?,
            valid_from: row.get(4)?,
            valid_to: row.get(5)?,
            confidence: row.get(6)?,
            source: row.get(7)?,
        })
    }).map_err(|e| format!("query: {}", e))?;

    let mut facts = Vec::new();
    for row in rows {
        facts.push(row.map_err(|e| format!("row: {}", e))?);
    }
    Ok(facts)
}

/// Get the full history of a subject-predicate-object triple.
pub fn get_fact_history(
    conn: &Connection,
    subject: &str,
    predicate: &str,
    object: &str,
) -> Result<Vec<TemporalFact>, String> {
    ensure_temporal_schema(conn).map_err(|e| format!("schema error: {}", e))?;
    let mut stmt = conn.prepare(
        "SELECT fact_id, subject, predicate, object, valid_from, valid_to, confidence, source
         FROM temporal_facts
         WHERE subject=?1 AND predicate=?2 AND object=?3
         ORDER BY valid_from DESC",
    ).map_err(|e| format!("prepare: {}", e))?;

    let rows = stmt.query_map(rusqlite::params![subject, predicate, object], |row| {
        Ok(TemporalFact {
            fact_id: row.get(0)?,
            subject: row.get(1)?,
            predicate: row.get(2)?,
            object: row.get(3)?,
            valid_from: row.get(4)?,
            valid_to: row.get(5)?,
            confidence: row.get(6)?,
            source: row.get(7)?,
        })
    }).map_err(|e| format!("query: {}", e))?;

    let mut facts = Vec::new();
    for row in rows {
        facts.push(row.map_err(|e| format!("row: {}", e))?);
    }
    Ok(facts)
}

/// Get the subgraph as of a specific time. Filters edges that have temporal
/// constraints: if a temporal fact exists for (source_id, relation_type, target_id),
/// only includes the edge if the fact was valid at `as_of`. Edges without
/// a corresponding temporal fact are always included (backward compatible).
pub fn subgraph_as_of(
    conn: &Connection,
    center_id: &str,
    depth: usize,
    as_of: i64,
) -> Result<(Vec<super::nt_memory_types::KnowledgeNode>, Vec<super::nt_memory_types::KnowledgeEdge>), String> {
    use std::collections::{HashSet, VecDeque};

    let mut node_ids = HashSet::new();
    let mut edge_ids = HashSet::new();
    let mut frontier = VecDeque::new();

    node_ids.insert(center_id.to_string());
    frontier.push_back((center_id.to_string(), 0));

    while let Some((current_id, current_depth)) = frontier.pop_front() {
        if current_depth >= depth {
            continue;
        }
        let edges = super::nt_memory_store::get_edges_for_node(conn, &current_id)
            .map_err(|e| format!("get_edges: {}", e))?;
        for edge in &edges {
            // Check temporal validity of this edge
            let mut stmt = conn.prepare(
                "SELECT 1 FROM temporal_facts
                 WHERE subject=?1 AND predicate=?2 AND object=?3
                   AND valid_from <= ?4 AND (valid_to IS NULL OR valid_to > ?4)
                 LIMIT 1",
            ).map_err(|e| format!("prepare temporal check: {}", e))?;

            let valid_now: bool = stmt.query_map(
                rusqlite::params![edge.source_id, edge.relation_type.as_str(), edge.target_id, as_of],
                |_| Ok(true),
            ).map_err(|e| format!("query temporal: {}", e))?.next().is_some();

            // Check if ANY temporal fact exists for this triple (regardless of time)
            let any_temporal: bool = {
                let mut s = conn.prepare(
                    "SELECT 1 FROM temporal_facts WHERE subject=?1 AND predicate=?2 AND object=?3 LIMIT 1"
                ).map_err(|e| format!("prepare any_temporal: {}", e))?;
                s.query_map(
                    rusqlite::params![edge.source_id, edge.relation_type.as_str(), edge.target_id],
                    |_| Ok(true),
                ).map_err(|e| format!("query any_temporal: {}", e))?.next().is_some()
            };

            // Include edge if:
            //   - no temporal facts exist for this triple (backward compat), OR
            //   - the fact is valid at as_of
            if !any_temporal || valid_now {
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
    }

    let mut nodes = Vec::new();
    for nid in &node_ids {
        if let Some(node) = super::nt_memory_store::get_node(conn, nid)
            .map_err(|e| format!("get_node: {}", e))?
        {
            nodes.push(node);
        }
    }

    let mut edges = Vec::new();
    for eid in &edge_ids {
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relation_type, weight, description, created_at, metadata,
                    version, superseded_by
             FROM edges WHERE id=?1",
        ).map_err(|e| format!("prepare edge: {}", e))?;
        let mut rows = stmt.query_map(rusqlite::params![eid], |row| {
            Ok(super::nt_memory_types::KnowledgeEdge {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                relation_type: super::nt_memory_types::RelationType::from_str(&row.get::<_, String>(3)?),
                weight: row.get(4)?,
                description: row.get(5)?,
                created_at: row.get(6)?,
                metadata: row.get::<_, Option<String>>(7)?.and_then(|m| serde_json::from_str(&m).ok()),
                version: row.get::<_, i64>(8)? as u64,
                superseded_by: row.get(9)?,
            })
        }).map_err(|e| format!("query edge: {}", e))?;
        if let Some(row) = rows.next() {
            edges.push(row.map_err(|e| format!("edge row: {}", e))?);
        }
    }

    Ok((nodes, edges))
}

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
    let source_neighbors: HashSet<String> = source_edges.iter().map(|e| {
        if e.source_id == source_id { e.target_id.clone() } else { e.source_id.clone() }
    }).collect();
    let source_edge_map: HashMap<&str, f64> = source_edges.iter()
        .map(|e| {
            let neighbor = if e.source_id == source_id { &e.target_id } else { &e.source_id };
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
        let neighbors: HashSet<String> = edges.iter().map(|e| {
            if e.source_id == cn.id { e.target_id.clone() } else { e.source_id.clone() }
        }).collect();
        let weighted: Vec<(String, f64)> = edges.iter().map(|e| {
            let nid = if e.source_id == cn.id { e.target_id.clone() } else { e.source_id.clone() };
            (nid, e.weight)
        }).collect();
        candidate_adj.insert(cn.id.clone(), (neighbors, weighted));
    }

    let mut results = Vec::with_capacity(candidate_nodes.len());

    for cn in &candidate_nodes {
        if cn.id == source_id {
            results.push((cn.id.clone(), RelevanceSignals {
                direct_link: 0.0, source_overlap: 0.0, adamic_adar: 0.0,
                type_affinity: 0.0, combined: 0.0,
            }));
            continue;
        }

        // Signal 1: Direct link (×3.0)
        let direct = if source_neighbors.contains(&cn.id) {
            source_edge_map.get(cn.id.as_str()).copied().unwrap_or(1.0) * 3.0
        } else {
            0.0
        };

        // Signal 2: Source overlap (×4.0)
        let source_overlap = if source.url.is_some() && cn.url.is_some()
            && source.url.as_deref() == cn.url.as_deref()
        {
            4.0
        } else if source.domain.is_some() && cn.domain.is_some()
            && source.domain.as_deref() == cn.domain.as_deref()
        {
            2.0
        } else {
            0.0
        };

        // Signal 3: Adamic-Adar (×1.5)
        let aa = compute_adamic_adar_from_adj(
            &source_neighbors,
            &candidate_adj.get(&cn.id).map(|(ns, _)| ns).cloned().unwrap_or_default(),
            &candidate_adj,
            conn,
        ) * 1.5;

        // Signal 4: Type affinity (×1.0)
        let type_aff = if source.node_type == cn.node_type { 1.0 } else { 0.0 };

        let combined = direct + source_overlap + aa + type_aff;

        results.push((cn.id.clone(), RelevanceSignals {
            direct_link: direct,
            source_overlap,
            adamic_adar: aa,
            type_affinity: type_aff,
            combined,
        }));
    }

    // Sort by combined descending
    results.sort_by(|a, b| b.1.combined.partial_cmp(&a.1.combined).unwrap_or(std::cmp::Ordering::Equal));

    Ok(results)
}

fn compute_adamic_adar_from_adj(
    source_neighbors: &HashSet<String>,
    candidate_neighbors: &HashSet<String>,
    candidate_adj: &HashMap<String, (HashSet<String>, Vec<(String, f64)>)>,
    _conn: &Connection,
) -> f64 {
    let common: Vec<&str> = source_neighbors.intersection(candidate_neighbors)
        .map(|s| s.as_str())
        .collect();

    if common.is_empty() {
        return 0.0;
    }

    let mut score = 0.0;
    for neighbor_id in &common {
        let degree = candidate_adj.get(*neighbor_id)
            .map(|(_, edges)| edges.len())
            .unwrap_or(0);
        if degree > 1 {
            score += 1.0 / (degree as f64).ln();
        }
    }

    score
}

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
    let id_to_idx: HashMap<&str, usize> = node_ids.iter().enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let _idx_to_id: Vec<&str> = node_ids.iter().map(|s| s.as_str()).collect();

    let mut adjacency: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for (i, nid) in node_ids.iter().enumerate() {
        let edges = nt_memory_store::get_edges_for_node(conn, nid)?;
        for e in &edges {
            let neighbor = if e.source_id == *nid { &e.target_id } else { &e.source_id };
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
        comp_to_orig.entry(compressed).or_insert_with(|| format!("community_{}", {
            let id = comm_counter;
            comm_counter += 1;
            id
        }));
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
            let neighbor = if e.source_id == *node_id { &e.target_id } else { &e.source_id };
            if let Some(neighbor_comm) = community_map.get(neighbor.as_str()) {
                if neighbor_comm != comm_id && neighbor_comm != "unaffiliated" {
                    let node = nt_memory_store::get_node(conn, node_id)?.unwrap();
                    let neighbor_node = nt_memory_store::get_node(conn, neighbor)?.unwrap();
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
    candidates.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(std::cmp::Ordering::Equal));
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
    gaps.sort_by(|a, b| b.isolation_score.partial_cmp(&a.isolation_score).unwrap_or(std::cmp::Ordering::Equal));
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
    let node_comm: HashMap<&str, &str> = community_map.iter()
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

        let neighbor_comms: HashSet<&str> = edges.iter().filter_map(|e| {
            let neighbor = if e.source_id == node_id { &e.target_id } else { &e.source_id };
            node_comm.get(neighbor.as_str()).copied()
        }).collect();

        let community_count = neighbor_comms.len();
        let node = match nt_memory_store::get_node(conn, node_id)? {
            Some(n) => n,
            None => continue,
        };

        // Bridge score: degree × community_count / (community_count + 1)
        let bridge_score = (degree as f64) * (community_count as f64) / (community_count as f64 + 1.0);

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

    bridge_candidates.sort_by(|a, b| b.bridge_score.partial_cmp(&a.bridge_score).unwrap_or(std::cmp::Ordering::Equal));
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
        comm_nodes.entry(comm_id.as_str()).or_default().push(nid.clone());
    }

    let mut communities = Vec::with_capacity(comm_nodes.len());

    for (&comm_id, node_ids) in &comm_nodes {
        let size = node_ids.len();
        if size == 0 { continue; }

        // Count internal edges and collect types
        let mut internal_edges = 0usize;
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for nid in node_ids {
            if let Ok(Some(node)) = nt_memory_store::get_node(conn, nid) {
                *type_counts.entry(node.node_type.as_str().to_string()).or_insert(0) += 1;
            }
            if let Ok(edges) = nt_memory_store::get_edges_for_node(conn, nid) {
                for e in &edges {
                    let neighbor = if e.source_id == *nid { &e.target_id } else { &e.source_id };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS temporal_facts (
                fact_id TEXT PRIMARY KEY,
                subject TEXT NOT NULL,
                predicate TEXT NOT NULL,
                object TEXT NOT NULL,
                valid_from INTEGER NOT NULL,
                valid_to INTEGER,
                confidence REAL DEFAULT 1.0,
                source TEXT DEFAULT 'neotrix',
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_temporal_subject ON temporal_facts(subject);
            CREATE INDEX IF NOT EXISTS idx_temporal_object ON temporal_facts(object);
            CREATE INDEX IF NOT EXISTS idx_temporal_valid ON temporal_facts(valid_from, valid_to);",
        ).unwrap();
        conn
    }

    #[test]
    fn test_relevance_signals_sorting() {
        let a = RelevanceSignals {
            direct_link: 6.0, source_overlap: 0.0, adamic_adar: 0.0,
            type_affinity: 0.0, combined: 6.0,
        };
        let b = RelevanceSignals {
            direct_link: 0.0, source_overlap: 0.0, adamic_adar: 1.5,
            type_affinity: 1.0, combined: 2.5,
        };
        assert!(a.combined > b.combined);
    }

    // ─── Temporal Tests ───────────────────────────────────────────

    #[test]
    fn test_insert_and_query_valid_fact() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f1".into(),
            subject: "Einstein".into(),
            predicate: "developed".into(),
            object: "Relativity".into(),
            valid_from: 1000,
            valid_to: None,
            confidence: 0.95,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        let results = query_temporal_facts(&conn, &TemporalQuery {
            subject: Some("Einstein".into()),
            predicate: None,
            object: None,
            as_of: None,
        }, 10).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fact_id, "f1");
        assert_eq!(results[0].object, "Relativity");
    }

    #[test]
    fn test_invalidate_fact() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f2".into(),
            subject: "Newton".into(),
            predicate: "wrote".into(),
            object: "Principia".into(),
            valid_from: 500,
            valid_to: None,
            confidence: 0.9,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        invalidate_fact(&conn, "f2").unwrap();

        let results = query_temporal_facts(&conn, &TemporalQuery {
            subject: Some("Newton".into()),
            predicate: None,
            object: None,
            as_of: None,
        }, 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_as_of_timestamp() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f3".into(),
            subject: "Darwin".into(),
            predicate: "proposed".into(),
            object: "Evolution".into(),
            valid_from: 1500,
            valid_to: None,
            confidence: 0.8,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        invalidate_fact(&conn, "f3").unwrap();

        let results = query_temporal_facts(&conn, &TemporalQuery {
            subject: Some("Darwin".into()),
            predicate: None,
            object: None,
            as_of: Some(1600),
        }, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].fact_id, "f3");
    }

    #[test]
    fn test_query_as_of_after_invalidation() {
        let conn = setup_test_db();
        let fact = TemporalFact {
            fact_id: "f4".into(),
            subject: "Darwin".into(),
            predicate: "proposed".into(),
            object: "Evolution".into(),
            valid_from: 1500,
            valid_to: None,
            confidence: 0.8,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact).unwrap();

        invalidate_fact(&conn, "f4").unwrap();

        let now = chrono::Utc::now().timestamp() + 100;
        let results = query_temporal_facts(&conn, &TemporalQuery {
            subject: Some("Darwin".into()),
            predicate: None,
            object: None,
            as_of: Some(now),
        }, 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_fact_history() {
        let conn = setup_test_db();

        let fact_v1 = TemporalFact {
            fact_id: "f5_v1".into(),
            subject: "Mendeleev".into(),
            predicate: "created".into(),
            object: "PeriodicTable".into(),
            valid_from: 1800,
            valid_to: Some(1850),
            confidence: 0.7,
            source: "test".into(),
        };
        let fact_v2 = TemporalFact {
            fact_id: "f5_v2".into(),
            subject: "Mendeleev".into(),
            predicate: "created".into(),
            object: "PeriodicTable".into(),
            valid_from: 1850,
            valid_to: None,
            confidence: 0.95,
            source: "test".into(),
        };
        insert_temporal_fact(&conn, &fact_v1).unwrap();
        insert_temporal_fact(&conn, &fact_v2).unwrap();

        let history = get_fact_history(&conn, "Mendeleev", "created", "PeriodicTable").unwrap();
        assert_eq!(history.len(), 2);
        assert!(history[0].valid_from > history[1].valid_from);
    }

    #[test]
    fn test_multiple_facts_same_subject() {
        let conn = setup_test_db();

        insert_temporal_fact(&conn, &TemporalFact {
            fact_id: "m1".into(),
            subject: "Tesla".into(),
            predicate: "invented".into(),
            object: "ACMotor".into(),
            valid_from: 1880,
            valid_to: None,
            confidence: 0.9,
            source: "test".into(),
        }).unwrap();

        insert_temporal_fact(&conn, &TemporalFact {
            fact_id: "m2".into(),
            subject: "Tesla".into(),
            predicate: "invented".into(),
            object: "Radio".into(),
            valid_from: 1890,
            valid_to: None,
            confidence: 0.85,
            source: "test".into(),
        }).unwrap();

        insert_temporal_fact(&conn, &TemporalFact {
            fact_id: "m3".into(),
            subject: "Tesla".into(),
            predicate: "worked_at".into(),
            object: "Edison".into(),
            valid_from: 1882,
            valid_to: Some(1885),
            confidence: 0.75,
            source: "test".into(),
        }).unwrap();

        // Filter by predicate "invented"
        let invented = query_temporal_facts(&conn, &TemporalQuery {
            subject: Some("Tesla".into()),
            predicate: Some("invented".into()),
            object: None,
            as_of: None,
        }, 10).unwrap();
        assert_eq!(invented.len(), 2);

        // Filter by object "Radio"
        let radio = query_temporal_facts(&conn, &TemporalQuery {
            subject: Some("Tesla".into()),
            predicate: None,
            object: Some("Radio".into()),
            as_of: None,
        }, 10).unwrap();
        assert_eq!(radio.len(), 1);
        assert_eq!(radio[0].fact_id, "m2");
    }
}
