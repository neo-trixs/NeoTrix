//! Hierarchical KB Retrieval (LeanRAG-inspired, AAAI 2026).
//!
//! Extends flat FTS5+embedding search with semantic aggregation and
//! hierarchical bottom-up retrieval. Clusters fine-grained entity nodes
//! into summary super-nodes, then retrieves by navigating the hierarchy.
//!
//! Key insight (LeanRAG): semantic aggregation → navigable KG → 46% lower
//! redundancy vs flat retrieval. Query anchors at fine-grained level,
//! then traverses upward through aggregate layers.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use super::nt_memory_types::*;

/// A semantic cluster: group of related entity nodes aggregated into a
/// summary super-node.
#[derive(Debug, Clone)]
pub struct SemanticCluster {
    pub cluster_id: String,
    pub label: String,
    pub summary: String,
    pub member_ids: Vec<String>,
    pub avg_importance: f64,
    pub topic_tags: Vec<String>,
}

/// A hierarchical search result with provenance tracking.
#[derive(Debug, Clone)]
pub struct HierarchicalSearchResult {
    pub node: KnowledgeNode,
    pub score: f64,
    pub matched_on: Vec<SearchMatchType>,
    /// Path from cluster root to this node
    pub hierarchy_path: Vec<String>,
    /// Whether this result came from an aggregate cluster
    pub from_aggregate: bool,
    pub redundancy_score: f64,
}

/// Build semantic clusters by grouping nodes with shared topic tags or
/// high graph connectivity.
pub fn build_semantic_clusters(
    conn: &Connection,
    min_cluster_size: usize,
    max_cluster_size: usize,
) -> rusqlite::Result<Vec<SemanticCluster>> {
    // Step 1: collect all nodes with their topic tags
    let mut stmt = conn.prepare(
        "SELECT id, title, summary, metadata, importance FROM nodes
         WHERE metadata IS NOT NULL AND node_type != 'Chunk'
         ORDER BY importance DESC"
    )?;
    let nodes: Vec<(String, String, Option<String>, Option<String>, f64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Step 2: extract topics and group by them
    let mut topic_groups: HashMap<String, Vec<(String, String, f64)>> = HashMap::new();
    for (id, title, _summary, metadata, importance) in &nodes {
        if let Some(meta_str) = metadata {
            if let Ok(meta) = serde_json::from_str::<serde_json::Value>(meta_str) {
                if let Some(tags) = meta.get("tags").and_then(|t| t.as_array()) {
                    for tag in tags {
                        if let Some(tag_str) = tag.as_str() {
                            if tag_str.starts_with("absorbed-") {
                                continue;
                            }
                            topic_groups
                                .entry(tag_str.to_string())
                                .or_default()
                                .push((id.clone(), title.clone(), *importance));
                        }
                    }
                }
            }
        }
    }

    // Step 3: filter and build clusters
    let mut clusters = Vec::new();
    for (topic, members) in topic_groups {
        if members.len() < min_cluster_size {
            continue;
        }
        let members: Vec<(String, String, f64)> = members.into_iter().take(max_cluster_size).collect();
        let avg_imp = members.iter().map(|(_, _, imp)| imp).sum::<f64>() / members.len() as f64;
        let member_ids: Vec<String> = members.iter().map(|(id, _, _)| id.clone()).collect();
        let titles: Vec<&str> = members.iter().map(|(_, t, _)| t.as_str()).collect();

        let cluster_id = format!("cluster:{}", topic);
        let label = topic.clone();
        let summary = format!(
            "Cluster of {} nodes related to '{}': {}",
            members.len(),
            topic,
            titles.join(", ")
        );

        clusters.push(SemanticCluster {
            cluster_id,
            label,
            summary,
            member_ids,
            avg_importance: avg_imp,
            topic_tags: vec![topic],
        });
    }

    Ok(clusters)
}

/// Hierarchical search: query at fine-grained level first, then aggregate.
///
/// 1. Run FTS5 search to find matching leaf nodes
/// 2. For each result, find its containing cluster(s)
/// 3. Rerank by cluster-level relevance
/// 4. Deduplicate with redundancy scoring
pub fn hierarchical_search(
    conn: &Connection,
    query: &str,
    limit: usize,
    clusters: &[SemanticCluster],
) -> rusqlite::Result<Vec<HierarchicalSearchResult>> {
    // Phase 1: fine-grained FTS5 search
    let fts_results = super::nt_memory_search::search_fts(conn, query, limit * 2)?;

    if fts_results.is_empty() {
        return Ok(Vec::new());
    }

    // Phase 2: build cluster membership lookup
    let mut node_to_clusters: HashMap<String, Vec<&SemanticCluster>> = HashMap::new();
    for cluster in clusters {
        for member_id in &cluster.member_ids {
            node_to_clusters.entry(member_id.clone()).or_default().push(cluster);
        }
    }

    // Phase 3: compute aggregate scores
    let mut cluster_scores: HashMap<String, (f64, usize)> = HashMap::new();
    for result in &fts_results {
        if let Some(clusters_for_node) = node_to_clusters.get(&result.node.id) {
            for cluster in clusters_for_node {
                let entry = cluster_scores.entry(cluster.cluster_id.clone()).or_default();
                entry.0 += result.score;
                entry.1 += 1;
            }
        }
    }

    // Phase 4: build hierarchical results with redundancy scoring
    let mut seen_ids = HashSet::new();
    let mut results: Vec<HierarchicalSearchResult> = Vec::new();

    for result in &fts_results {
        if seen_ids.contains(&result.node.id) {
            continue;
        }
        seen_ids.insert(result.node.id.clone());

        let hierarchy_path: Vec<String> = node_to_clusters
            .get(&result.node.id)
            .map(|cs| cs.iter().map(|c| c.cluster_id.clone()).collect())
            .unwrap_or_default();

        let from_aggregate = !hierarchy_path.is_empty();

        // Redundancy score: how many clusters this node belongs to (more = more redundant)
        let redundancy_score = if from_aggregate {
            (hierarchy_path.len() as f64 - 1.0).max(0.0) * 0.1
        } else {
            0.0
        };

        results.push(HierarchicalSearchResult {
            node: result.node.clone(),
            score: result.score,
            matched_on: result.matched_on.clone(),
            hierarchy_path,
            from_aggregate,
            redundancy_score,
        });
    }

    // Phase 5: rerank by (score - redundancy_penalty)
    results.sort_by(|a, b| {
        let a_adj = a.score - a.redundancy_score;
        let b_adj = b.score - b.redundancy_score;
        b_adj.partial_cmp(&a_adj).unwrap_or(std::cmp::Ordering::Equal)
    });

    results.truncate(limit);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cluster(id: &str, members: Vec<&str>) -> SemanticCluster {
        SemanticCluster {
            cluster_id: id.to_string(),
            label: id.to_string(),
            summary: format!("Cluster: {}", id),
            member_ids: members.iter().map(|m| m.to_string()).collect(),
            avg_importance: 0.5,
            topic_tags: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_semantic_cluster_creation() {
        let cluster = make_cluster("cluster:test", vec!["node-1", "node-2"]);
        assert_eq!(cluster.member_ids.len(), 2);
        assert!((cluster.avg_importance - 0.5).abs() < 0.01);
    }

    #[allow(dead_code)]
    fn make_node(id: &str, title: &str) -> KnowledgeNode {
        KnowledgeNode {
            id: id.to_string(),
            node_type: NodeType::Concept,
            title: title.to_string(),
            summary: None,
            content: None,
            url: None,
            domain: None,
            language: "en".to_string(),
            confidence: 0.5,
            importance: 0.5,
            created_at: 0,
            updated_at: 0,
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        }
    }

    #[test]
    fn test_hierarchical_result_ordering() {
        let node_a = make_node("a", "Alpha");
        let node_b = make_node("b", "Beta");

        let r1 = HierarchicalSearchResult {
            node: node_a,
            score: 0.9,
            matched_on: vec![],
            hierarchy_path: vec!["cluster:x".into()],
            from_aggregate: true,
            redundancy_score: 0.0,
        };
        let r2 = HierarchicalSearchResult {
            node: node_b,
            score: 0.8,
            matched_on: vec![],
            hierarchy_path: vec![],
            from_aggregate: false,
            redundancy_score: 0.0,
        };

        let mut results = vec![r2, r1];
        results.sort_by(|a, b| {
            let a_adj = a.score - a.redundancy_score;
            let b_adj = b.score - b.redundancy_score;
            b_adj.partial_cmp(&a_adj).unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(results[0].score, 0.9);
        assert_eq!(results[0].node.id, "a");
    }

    #[test]
    fn test_cluster_redundancy_penalty() {
        let result = HierarchicalSearchResult {
            node: make_node("redundant", "Redundant"),
            score: 0.8,
            matched_on: vec![],
            hierarchy_path: vec!["c1".into(), "c2".into(), "c3".into()],
            from_aggregate: true,
            redundancy_score: 0.2,
        };
        assert!(result.redundancy_score > 0.0);
        let adjusted = result.score - result.redundancy_score;
        assert!(adjusted < result.score);
    }
}
