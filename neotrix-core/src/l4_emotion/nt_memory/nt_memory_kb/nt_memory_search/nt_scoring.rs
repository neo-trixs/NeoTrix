//! KB 检索打分：4 维信号融合（semantic / temporal / confidence / relational）。
//!
//! ── 能力边界（2026-09-28 消歧时评估两次后确立）──
//! 本模块面向 **KB 节点**（有 `updated_at` / `node_confidence` / 图边），故 4 维权重
//! 全部有数据源。**不要**把它套到 `l1_action/nt_core_bank/bank/rag_engine.rs`
//! （记忆层级管理）的 `score_memory` 上 —— 后者对象是 `ReasoningMemory`
//! （`task_description` 等），**没有 temporal/confidence/relational 的数据源**，
//! 硬接只能传 0 或造假值，比它现有的 bm25+embedding+关键词重叠三信号线性加权更差。
//! 详见该文件顶部的「能力边界」段（含若要升级的正确顺序：先补数据源，再换算法）。

use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::super::nt_memory_types::*;

// ── SmartVector scoring (moved from nt_memory_search.rs, pure move) ──

/// Entity graph scores: find seed nodes matching query keywords, then propagate
/// probability via Personalized PageRank (1 iteration). Seeds get base score,
/// 1-hop neighbors get edge-weight boost, 2-hop neighbors get attenuated boost.
pub fn entity_graph_scores(
    conn: &Connection,
    query: &str,
) -> rusqlite::Result<HashMap<String, f64>> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .collect();

    if query_words.is_empty() {
        return Ok(HashMap::new());
    }

    // FTS5 seed discovery: use MATCH instead of LIKE for tokenized search
    let fts_query: String = query_words.join(" OR ");
    let seed_ids: Vec<String> = if let Ok(mut stmt) = conn.prepare(
        "SELECT n.id FROM nodes n JOIN nodes_fts f ON n.rowid = f.rowid WHERE nodes_fts MATCH ?1 LIMIT 50",
    ) {
        stmt.query_map(params![fts_query], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect()
    } else {
        // Fallback: LIKE only when FTS5 is unavailable
        let mut stmt = conn.prepare("SELECT id FROM nodes WHERE LOWER(title) LIKE ?1")?;
        let pattern = format!("%{}%", query_lower);
        let results: Vec<String> = stmt.query_map(params![pattern], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        results
    };

    if seed_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut scores: HashMap<String, f64> = HashMap::new();
    let seed_set: std::collections::HashSet<String> = seed_ids.iter().cloned().collect();

    for id in &seed_ids {
        *scores.entry(id.clone()).or_insert(0.0) = 0.5;
    }

    // 1-hop: +0.1 per edge weight
    let mut one_hop: HashMap<String, f64> = HashMap::new();
    for seed_id in &seed_ids {
        let edges = super::super::nt_memory_store::get_edges_for_node(conn, seed_id)?;
        for edge in &edges {
            let neighbor = if edge.source_id == *seed_id {
                &edge.target_id
            } else {
                &edge.source_id
            };
            if !seed_set.contains(neighbor.as_str()) {
                *one_hop.entry(neighbor.clone()).or_insert(0.0) += 0.1 * edge.weight;
            }
        }
    }
    let one_hop_set: std::collections::HashSet<String> = one_hop.keys().cloned().collect();

    // 2-hop: +0.05 per edge weight
    let mut two_hop: HashMap<String, f64> = HashMap::new();
    for neighbor_id in one_hop.keys() {
        let edges = super::super::nt_memory_store::get_edges_for_node(conn, neighbor_id)?;
        for edge in &edges {
            let neighbor2 = if edge.source_id == *neighbor_id {
                &edge.target_id
            } else {
                &edge.source_id
            };
            if !seed_set.contains(neighbor2.as_str()) && !one_hop_set.contains(neighbor2.as_str()) {
                *two_hop.entry(neighbor2.clone()).or_insert(0.0) += 0.05 * edge.weight;
            }
        }
    }

    for (id, score) in one_hop {
        *scores.entry(id).or_insert(0.0) += score.min(0.5);
    }
    for (id, score) in two_hop {
        *scores.entry(id).or_insert(0.0) += score.min(0.3);
    }

    // Normalize to [0, 1]
    let max_score = scores.values().cloned().fold(0.0, f64::max);
    if max_score > 0.0 {
        for score in scores.values_mut() {
            *score /= max_score;
        }
    }

    Ok(scores)
}

/// Fuse 4 signals into a single ranked list via weighted linear combination.
/// Returns Vec<(node_id, fused_score, [fts5, bm25, embed, graph])>.
pub fn fuse_signals(
    fts_results: &[SearchResult],
    bm25_results: &[(f64, String)],
    embed_results: &[(f64, String)],
    graph_scores: &HashMap<String, f64>,
    limit: usize,
    weights: [f64; 4],
) -> Vec<(String, f64, [f64; 4])> {
    let mut node_scores: HashMap<&str, (f64, [f64; 4])> = HashMap::new();

    for r in fts_results {
        node_scores
            .entry(r.node.id.as_str())
            .and_modify(|(s, sig)| {
                *s += weights[0] * r.score;
                sig[0] = r.score;
            })
            .or_insert((weights[0] * r.score, [r.score, 0.0, 0.0, 0.0]));
    }

    for (score, id) in bm25_results {
        node_scores
            .entry(id.as_str())
            .and_modify(|(s, sig)| {
                *s += weights[1] * score;
                sig[1] = *score;
            })
            .or_insert((weights[1] * *score, [0.0, *score, 0.0, 0.0]));
    }

    for (score, id) in embed_results {
        node_scores
            .entry(id.as_str())
            .and_modify(|(s, sig)| {
                *s += weights[2] * score;
                sig[2] = *score;
            })
            .or_insert((weights[2] * *score, [0.0, 0.0, *score, 0.0]));
    }

    for (id, score) in graph_scores {
        node_scores
            .entry(id.as_str())
            .and_modify(|(s, sig)| {
                *s += weights[3] * score;
                sig[3] = *score;
            })
            .or_insert((weights[3] * *score, [0.0, 0.0, 0.0, *score]));
    }

    let mut results: Vec<(String, f64, [f64; 4])> = node_scores
        .into_iter()
        .map(|(id, (score, signals))| (id.to_string(), score, signals))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(limit);
    results
}

// ═══════════════════════════════════════════════════════════════════
// P0: SmartVector 4-Signal Scoring
// ═══════════════════════════════════════════════════════════════════
// Semantic (FTS/BM25/embedding) + Temporal (freshness) + Confidence (epistemic)
// + Relational (graph topology) fused via configurable linear weights.

/// Configurable weights for the 4 SmartVector scoring signals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartVectorWeights {
    /// Semantic relevance (FTS + BM25 + embedding cosine)
    pub semantic: f64,
    /// Temporal freshness (decay from last update)
    pub temporal: f64,
    /// Epistemic confidence (from ConfidenceStore aggregate)
    pub confidence: f64,
    /// Relational strength (graph edge density + PageRank trust)
    pub relational: f64,
}

impl Default for SmartVectorWeights {
    fn default() -> Self {
        Self {
            semantic: 0.50,
            temporal: 0.15,
            confidence: 0.20,
            relational: 0.15,
        }
    }
}

impl SmartVectorWeights {
    /// Normalize weights to sum to 1.0 for convex combination.
    pub fn normalized(&self) -> Self {
        let sum = self.semantic + self.temporal + self.confidence + self.relational;
        if sum <= 0.0 {
            return Self::default();
        }
        Self {
            semantic: self.semantic / sum,
            temporal: self.temporal / sum,
            confidence: self.confidence / sum,
            relational: self.relational / sum,
        }
    }
}

/// Compute temporal freshness score for a node.
/// Uses exponential decay from `updated_at` with configurable half-life.
/// Returns [0.0, 1.0] where 1.0 = just updated, approaching 0.0 for old nodes.
pub fn temporal_score(updated_at: i64, now: i64, half_life_secs: i64) -> f64 {
    if half_life_secs <= 0 {
        return 1.0;
    }
    let age = now.saturating_sub(updated_at).max(0);
    0.5_f64.powf(age as f64 / half_life_secs as f64)
}

/// Compute confidence score from ConfidenceStore aggregate.
/// Maps the epistemic confidence [0.0, 1.0] directly.
/// Falls back to node.confidence if ConfidenceStore lookup fails.
pub fn confidence_score(node_confidence: f64, store_confidence: Option<f64>) -> f64 {
    store_confidence
        .unwrap_or(node_confidence)
        .max(0.0)
        .min(1.0)
}

/// Compute relational score from graph topology: edge density + PageRank trust.
/// More edges with higher weights → higher relational score.
/// Trust-from-topology (PageRank) adds structural importance signal.
/// Uses logarithmic scaling to prevent degree-1000 hub domination.
pub fn relational_score(edge_count: usize, total_weight: f64, trust: f64) -> f64 {
    if edge_count == 0 {
        return trust.min(1.0);
    }
    // Logarithmic degree + normalized weight + trust (PageRank) component
    let degree_component = (1.0 + edge_count as f64).ln() / 5.0; // ln(33) ≈ 3.5 for 32 edges
    let weight_component = (total_weight / edge_count as f64).min(1.0);
    (degree_component * 0.4 + weight_component * 0.2 + trust * 0.4).min(1.0)
}

/// Result of SmartVector 4-signal scoring for a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartVectorScore {
    pub node_id: String,
    pub fused_score: f64,
    pub semantic: f64,
    pub temporal: f64,
    pub confidence: f64,
    pub relational: f64,
}

/// SmartVector scorer — computes 4-signal scores for search results.
pub struct SmartVectorScorer {
    pub weights: SmartVectorWeights,
    pub half_life_secs: i64,
}

impl Default for SmartVectorScorer {
    fn default() -> Self {
        Self {
            weights: SmartVectorWeights::default(),
            half_life_secs: 7 * 24 * 3600, // 7 days
        }
    }
}

impl SmartVectorScorer {
    pub fn new(weights: SmartVectorWeights, half_life_secs: i64) -> Self {
        Self {
            weights: weights.normalized(),
            half_life_secs,
        }
    }

    /// Score a batch of search results using 4-signal fusion.
    /// Requires a connection to query graph edges per node and ConfidenceStore lookup.
    pub fn score_results(
        &self,
        results: &[SearchResult],
        conn: &Connection,
    ) -> Vec<SmartVectorScore> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Batch-fetch edge counts and weights for all node IDs
        let node_ids: Vec<&str> = results.iter().map(|r| r.node.id.as_str()).collect();
        let edge_map = batch_edge_stats(conn, &node_ids);

        // PageRank trust scores from topology (best-effort, non-fatal)
        let trust_map: HashMap<String, f64> =
            super::super::nt_memory_graph::trust_for_nodes(conn, &node_ids, 0.85, 50).unwrap_or_default();

        // Load ConfidenceStore if available (best-effort, non-fatal)
        let confidence_store: Option<super::super::nt_memory_confidence::ConfidenceStore> =
            load_confidence_store_from_conn(conn);

        results
            .iter()
            .map(|r| {
                let sem = r.score.max(0.0).min(1.0);
                let temp = temporal_score(r.node.updated_at, now, self.half_life_secs);

                let store_conf = confidence_store
                    .as_ref()
                    .and_then(|cs| cs.get_confidence_by_str(&r.node.id).ok().flatten())
                    .map(|ec| ec.aggregate());
                let conf = confidence_score(r.node.confidence, store_conf);

                let (edge_count, total_weight) =
                    edge_map.get(&r.node.id).copied().unwrap_or((0, 0.0));
                let trust = trust_map
                    .get(&r.node.id)
                    .copied()
                    .unwrap_or(1.0 / (results.len() as f64).max(1.0));
                let rel = relational_score(edge_count, total_weight, trust);

                let fused = self.weights.semantic * sem
                    + self.weights.temporal * temp
                    + self.weights.confidence * conf
                    + self.weights.relational * rel;

                SmartVectorScore {
                    node_id: r.node.id.clone(),
                    fused_score: fused,
                    semantic: sem,
                    temporal: temp,
                    confidence: conf,
                    relational: rel,
                }
            })
            .collect()
    }
}

/// Batch-fetch edge statistics for multiple nodes in one query.
fn batch_edge_stats(conn: &Connection, node_ids: &[&str]) -> HashMap<String, (usize, f64)> {
    if node_ids.is_empty() {
        return HashMap::new();
    }
    let placeholders: Vec<String> = node_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    let sql = format!(
        "SELECT node_id, COUNT(*) as cnt, COALESCE(SUM(weight), 0.0) as total_w FROM (
            SELECT source_id as node_id, weight FROM edges WHERE target_id IN ({})
            UNION ALL
            SELECT target_id as node_id, weight FROM edges WHERE source_id IN ({})
        ) GROUP BY node_id",
        placeholders.join(","),
        placeholders.join(","),
    );
    let mut result = HashMap::new();
    if let Ok(mut stmt) = conn.prepare(&sql) {
        let mut all_ids: Vec<&dyn rusqlite::types::ToSql> = Vec::with_capacity(node_ids.len() * 2);
        for id in node_ids {
            all_ids.push(id);
        }
        for id in node_ids {
            all_ids.push(id);
        }
        if let Ok(rows) = stmt.query_map(all_ids.as_slice(), |row| {
            let id: String = row.get(0)?;
            let cnt: i64 = row.get(1)?;
            let total_w: f64 = row.get(2)?;
            Ok((id, cnt as usize, total_w))
        }) {
            for r in rows.filter_map(|r| r.ok()) {
                result.insert(r.0, (r.1, r.2));
            }
        }
    }
    result
}

/// Best-effort load ConfidenceStore from KB conn (kv_store persistence).
fn load_confidence_store_from_conn(
    conn: &Connection,
) -> Option<super::super::nt_memory_confidence::ConfidenceStore> {
    let data: Option<String> = conn
        .query_row(
            "SELECT value FROM kv_store WHERE namespace = 'confidence' AND key = 'store'",
            [],
            |row| row.get(0),
        )
        .ok();
    data.and_then(|d| serde_json::from_str(&d).ok())
}

/// f32 余弦相似度 (单遍计算 dot/|a|/|b|, 无额外分配)。
#[inline]
pub fn cosine_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f32;
    let mut na = 0.0_f32;
    let mut nb = 0.0_f32;
    for i in 0..a.len() {
        let x = a[i];
        let y = b[i];
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}
