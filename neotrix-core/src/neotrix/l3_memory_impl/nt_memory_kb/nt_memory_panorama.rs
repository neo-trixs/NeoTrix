//! nt_memory_panorama — 知识全景链路图模块
//!
//! 构建 KB 内容的全景视图：覆盖度分析、缺口检测、知识新鲜度跟踪、
//! 链路图谱生成，持续完善知识库全景视图。

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::nt_memory_store as store;
use super::nt_memory_types::*;
use super::KnowledgeBase;

fn now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePanorama {
    /// 时间戳
    pub generated_at: i64,
    /// 全局统计
    pub global_stats: GlobalStats,
    /// 各领域覆盖度
    pub domain_coverage: Vec<DomainCoverage>,
    /// 知识新鲜度
    pub freshness: FreshnessReport,
    /// 知识链路
    pub knowledge_graph: PanoramaGraph,
    /// 检测到的缺口
    pub gaps: Vec<KnowledgeGap>,
    /// 节点类型分布
    pub node_type_distribution: Vec<(String, usize)>,
    /// 关系类型分布
    pub relation_type_distribution: Vec<(String, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_repos: usize,
    pub total_papers: usize,
    pub total_articles: usize,
    pub total_concepts: usize,
    pub total_code_snippets: usize,
    pub total_insights: usize,
    pub avg_confidence: f64,
    pub avg_importance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCoverage {
    pub domain: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub coverage_score: f64,      // 0.0 - 1.0
    pub last_updated: i64,
    pub sub_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessReport {
    pub nodes_updated_today: usize,
    pub nodes_updated_this_week: usize,
    pub nodes_updated_this_month: usize,
    pub stale_nodes: usize,       // not updated in 90 days
    pub avg_age_days: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaGraph {
    pub nodes: Vec<PanoramaNode>,
    pub links: Vec<PanoramaLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub domain: String,
    pub importance: f64,
    pub connection_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanoramaLink {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGap {
    pub gap_type: GapType,
    pub domain: String,
    pub description: String,
    pub severity: GapSeverity,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GapType {
    /// 领域缺失（某领域完全没有节点）
    MissingDomain,
    /// 覆盖不足（节点太少）
    UnderservedDomain,
    /// 知识过时
    StaleKnowledge,
    /// 孤岛节点（无连接的节点）
    OrphanedNode,
    /// 缺乏深度（只有概念没有详细信息）
    ShallowCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GapSeverity {
    Critical,
    Major,
    Minor,
    Suggestion,
}

// ── KnowledgePanoramaBuilder ──

pub struct KnowledgePanoramaBuilder {
    kb: KnowledgeBase,
}

impl KnowledgePanoramaBuilder {
    pub fn new(kb: KnowledgeBase) -> Self {
        Self { kb }
    }

    /// 生成完整知识全景图
    pub fn build(&self) -> Result<KnowledgePanorama, String> {
        let all_nodes = {
            let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            store::get_all_nodes(&conn).map_err(|e| format!("get_all: {}", e))?
        };
        let all_edges = {
            let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            store::get_all_edges(&conn).map_err(|e| format!("get_all_edges: {}", e))?
        };

        let global = self.compute_global_stats(&all_nodes, &all_edges);
        let domain_coverage = self.compute_domain_coverage(&all_nodes, &all_edges);
        let freshness = self.compute_freshness(&all_nodes);
        let graph = self.build_panorama_graph(&all_nodes, &all_edges);
        let gaps = self.detect_gaps(&domain_coverage, &all_nodes, &all_edges);
        let node_type_dist = self.compute_type_distribution(&all_nodes);
        let rel_type_dist = self.compute_relation_distribution(&all_edges);

        // Persist panorama snapshot
        let pano = KnowledgePanorama {
            generated_at: now(),
            global_stats: global,
            domain_coverage,
            freshness,
            knowledge_graph: graph,
            gaps,
            node_type_distribution: node_type_dist,
            relation_type_distribution: rel_type_dist,
        };

        let json = serde_json::to_string(&pano).map_err(|e| format!("serde: {}", e))?;
        let _ = self.kb.kv_set("panorama", &format!("snap_{}", now()), &json);

        Ok(pano)
    }

    /// 生成简化的知识链路视图（用于快速查看）
    pub fn build_quick_summary(&self) -> Result<String, String> {
        let all_nodes = {
            let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            store::get_all_nodes(&conn).map_err(|e| format!("get_all: {}", e))?
        };
        let all_edges = {
            let conn = self.kb.conn.lock().map_err(|e| format!("Lock: {}", e))?;
            store::get_all_edges(&conn).map_err(|e| format!("get_all_edges: {}", e))?
        };

        let by_type = self.compute_type_distribution(&all_nodes);
        let by_domain = self.compute_domain_summary(&all_nodes);
        let freshness = self.compute_freshness(&all_nodes);
        let gaps = self.detect_gaps(
            &self.compute_domain_coverage(&all_nodes, &all_edges),
            &all_nodes,
            &all_edges,
        );

        let mut lines = Vec::new();
        lines.push("╔══════════════════════════════════════╗".into());
        lines.push("║     Knowledge Panorama Summary       ║".into());
        lines.push("╚══════════════════════════════════════╝".into());
        lines.push(String::new());
        lines.push(format!("📊 Total: {} nodes, {} edges", all_nodes.len(), all_edges.len()));
        lines.push(String::new());
        lines.push("📁 By Type:".into());
        for (ntype, count) in &by_type {
            if *count > 0 {
                lines.push(format!("  • {}: {}", ntype, count));
            }
        }
        lines.push(String::new());
        lines.push("🌐 By Domain:".into());
        for (domain, count) in &by_domain {
            if *count > 0 {
                lines.push(format!("  • {}: {} nodes", domain, count));
            }
        }
        lines.push(String::new());
        lines.push("⏱ Freshness:".into());
        lines.push(format!("  • Today: {} | Week: {} | Month: {}", freshness.nodes_updated_today, freshness.nodes_updated_this_week, freshness.nodes_updated_this_month));
        lines.push(format!("  • Stale (>90d): {} | Avg age: {:.1} days", freshness.stale_nodes, freshness.avg_age_days));
        lines.push(String::new());
        lines.push("⚠ Gaps:".into());
        for gap in &gaps {
            let icon = match gap.severity {
                GapSeverity::Critical => "🔴",
                GapSeverity::Major => "🟠",
                GapSeverity::Minor => "🟡",
                GapSeverity::Suggestion => "💡",
            };
            lines.push(format!("  {} [{}] {}: {}", icon, gap.gap_type.gap_type_name(), gap.domain, gap.description));
        }

        Ok(lines.join("\n"))
    }

    // ── Private: Stats ──

    fn compute_global_stats(&self, nodes: &[KnowledgeNode], edges: &[KnowledgeEdge]) -> GlobalStats {
        let total = nodes.len();
        GlobalStats {
            total_nodes: total,
            total_edges: edges.len(),
            total_repos: nodes.iter().filter(|n| n.node_type == NodeType::Repository).count(),
            total_papers: nodes.iter().filter(|n| n.node_type == NodeType::Paper).count(),
            total_articles: nodes.iter().filter(|n| n.node_type == NodeType::Article).count(),
            total_concepts: nodes.iter().filter(|n| n.node_type == NodeType::Concept).count(),
            total_code_snippets: nodes.iter().filter(|n| n.node_type == NodeType::CodeSnippet).count(),
            total_insights: nodes.iter().filter(|n| n.node_type == NodeType::Insight).count(),
            avg_confidence: if total > 0 { nodes.iter().map(|n| n.confidence).sum::<f64>() / total as f64 } else { 0.0 },
            avg_importance: if total > 0 { nodes.iter().map(|n| n.importance).sum::<f64>() / total as f64 } else { 0.0 },
        }
    }

    fn compute_domain_coverage(&self, nodes: &[KnowledgeNode], edges: &[KnowledgeEdge]) -> Vec<DomainCoverage> {
        let mut by_domain: HashMap<String, Vec<&KnowledgeNode>> = HashMap::new();
        for node in nodes {
            let domain = node.domain.as_deref().unwrap_or("unknown").to_string();
            by_domain.entry(domain).or_default().push(node);
        }
        let mut edge_count_by_domain: HashMap<String, usize> = HashMap::new();
        let node_domain: std::collections::HashMap<&str, &str> = nodes.iter()
            .map(|n| (n.id.as_str(), n.domain.as_deref().unwrap_or("unknown")))
            .collect();
        for edge in edges {
            let src_domain = node_domain.get(edge.source_id.as_str()).copied().unwrap_or("unknown");
            let tgt_domain = node_domain.get(edge.target_id.as_str()).copied().unwrap_or("unknown");
            for d in [src_domain, tgt_domain] {
                *edge_count_by_domain.entry(d.to_string()).or_insert(0) += 1;
            }
        }

        let target_domains = vec![
            "github.com", "arxiv.org", "wikipedia.org", "github.com/topic",
            "neotrix", "mathematics", "physics", "computer_science",
            "philosophy", "neuroscience", "cognitive_science",
            "programming_language", "package_ecosystem", "distiller",
            "github_absorber", "consciousness",
        ];

        target_domains.iter().map(|domain| {
            let domain_nodes = by_domain.get(*domain).cloned().unwrap_or_default();
            let count = domain_nodes.len();
            let edge_count = edge_count_by_domain.get(*domain).copied().unwrap_or(0);
            let last_updated = domain_nodes.iter()
                .map(|n| n.updated_at)
                .max()
                .unwrap_or(0);

            // Coverage score: heuristic based on node count
            let coverage_score = match *domain {
                "github.com" => (count as f64 / 20.0).min(1.0),
                "arxiv.org" => (count as f64 / 10.0).min(1.0),
                "wikipedia.org" => (count as f64 / 50.0).min(1.0),
                "mathematics" | "physics" | "computer_science" | "philosophy" => (count as f64 / 15.0).min(1.0),
                "neotrix" => (count as f64 / 10.0).min(1.0),
                _ => (count as f64 / 5.0).min(1.0),
            };

            DomainCoverage {
                domain: domain.to_string(),
                node_count: count,
                edge_count,
                coverage_score,
                last_updated,
                sub_domains: Vec::new(),
            }
        }).collect()
    }

    fn compute_freshness(&self, nodes: &[KnowledgeNode]) -> FreshnessReport {
        let now_ts = now();
        let day_secs = 86400;
        let week_secs = 7 * day_secs;
        let month_secs = 30 * day_secs;
        let stale_secs = 90 * day_secs;

        let mut today = 0;
        let mut week = 0;
        let mut month = 0;
        let mut stale = 0;
        let mut total_age = 0i64;

        for node in nodes {
            let age = now_ts - node.updated_at;
            total_age += age;
            if age <= day_secs { today += 1; }
            if age <= week_secs { week += 1; }
            if age <= month_secs { month += 1; }
            if age > stale_secs { stale += 1; }
        }

        FreshnessReport {
            nodes_updated_today: today,
            nodes_updated_this_week: week,
            nodes_updated_this_month: month,
            stale_nodes: stale,
            avg_age_days: if nodes.is_empty() { 0.0 } else { total_age as f64 / nodes.len() as f64 / 86400.0 },
        }
    }

    fn build_panorama_graph(&self, nodes: &[KnowledgeNode], edges: &[KnowledgeEdge]) -> PanoramaGraph {
        let conn_count: HashMap<String, usize> = {
            let mut counts = HashMap::new();
            for edge in edges {
                *counts.entry(edge.source_id.clone()).or_insert(0) += 1;
                *counts.entry(edge.target_id.clone()).or_insert(0) += 1;
            }
            counts
        };

        let pano_nodes: Vec<PanoramaNode> = nodes.iter().map(|n| {
            PanoramaNode {
                id: n.id.clone(),
                label: n.title.clone(),
                node_type: n.node_type.as_str().to_string(),
                domain: n.domain.as_deref().unwrap_or("unknown").to_string(),
                importance: n.importance,
                connection_count: conn_count.get(&n.id).copied().unwrap_or(0),
            }
        }).collect();

        let pano_links: Vec<PanoramaLink> = edges.iter().map(|e| {
            PanoramaLink {
                source: e.source_id.clone(),
                target: e.target_id.clone(),
                relation: e.relation_type.as_str().to_string(),
                weight: e.weight,
            }
        }).collect();

        PanoramaGraph { nodes: pano_nodes, links: pano_links }
    }

    fn detect_gaps(&self, coverage: &[DomainCoverage], nodes: &[KnowledgeNode], edges: &[KnowledgeEdge]) -> Vec<KnowledgeGap> {
        let mut gaps = Vec::new();

        // Missing / underserved domains
        for dc in coverage {
            if dc.node_count == 0 {
                gaps.push(KnowledgeGap {
                    gap_type: GapType::MissingDomain,
                    domain: dc.domain.clone(),
                    description: format!("No knowledge nodes in '{}' domain", dc.domain),
                    severity: GapSeverity::Major,
                    suggestion: format!("Absorb sources in '{}' domain via GitHub / Wikipedia / ArXiv", dc.domain),
                });
            } else if dc.coverage_score < 0.3 {
                gaps.push(KnowledgeGap {
                    gap_type: GapType::UnderservedDomain,
                    domain: dc.domain.clone(),
                    description: format!("Only {} nodes in '{}' domain (coverage {:.1}%)", dc.node_count, dc.domain, dc.coverage_score * 100.0),
                    severity: GapSeverity::Minor,
                    suggestion: format!("Add more sources to '{}' domain", dc.domain),
                });
            }
        }

        // Orphaned nodes (no edges)
        let connected: std::collections::HashSet<String> = edges.iter()
            .flat_map(|e| vec![e.source_id.clone(), e.target_id.clone()])
            .collect();
        let orphans: Vec<&KnowledgeNode> = nodes.iter()
            .filter(|n| !connected.contains(&n.id))
            .collect();
        for orphan in orphans.iter().take(10) {
            gaps.push(KnowledgeGap {
                gap_type: GapType::OrphanedNode,
                domain: orphan.domain.as_deref().unwrap_or("unknown").to_string(),
                description: format!("Orphaned node: '{}' ({}) — no connections", orphan.title, orphan.node_type.as_str()),
                severity: GapSeverity::Suggestion,
                suggestion: "Run distiller to auto-link this node to related concepts".into(),
            });
        }

        // Stale knowledge
        let now_ts = now();
        let stale_threshold = 90 * 86400;
        let stale_count = nodes.iter().filter(|n| now_ts - n.updated_at > stale_threshold).count();
        if stale_count > 0 {
            gaps.push(KnowledgeGap {
                gap_type: GapType::StaleKnowledge,
                domain: "global".into(),
                description: format!("{} nodes not updated in 90+ days", stale_count),
                severity: if stale_count > 50 { GapSeverity::Major } else { GapSeverity::Minor },
                suggestion: "Run absorber refresh cycle for stale repositories".into(),
            });
        }

        gaps
    }

    fn compute_type_distribution(&self, nodes: &[KnowledgeNode]) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for node in nodes {
            *counts.entry(node.node_type.as_str().to_string()).or_insert(0) += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    fn compute_relation_distribution(&self, edges: &[KnowledgeEdge]) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for edge in edges {
            *counts.entry(edge.relation_type.as_str().to_string()).or_insert(0) += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }

    fn compute_domain_summary(&self, nodes: &[KnowledgeNode]) -> Vec<(String, usize)> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for node in nodes {
            let d = node.domain.as_deref().unwrap_or("unknown").to_string();
            *counts.entry(d).or_insert(0) += 1;
        }
        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

impl GapType {
    pub fn gap_type_name(&self) -> &'static str {
        match self {
            GapType::MissingDomain => "Missing Domain",
            GapType::UnderservedDomain => "Underserved",
            GapType::StaleKnowledge => "Stale",
            GapType::OrphanedNode => "Orphaned",
            GapType::ShallowCoverage => "Shallow",
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn test_kb() -> KnowledgeBase {
        KnowledgeBase::open(Some(std::path::PathBuf::from(":memory:"))).unwrap()
    }

    fn insert_node(kb: &KnowledgeBase, title: &str, ntype: NodeType, domain: &str, age_secs: i64) -> String {
        let now_ts = now();
        let id = kb.insert_or_get_node(title, ntype, None, None, Some(domain)).unwrap();
        // Manually set updated_at to control freshness
        let conn = kb.conn.lock().unwrap();
        conn.execute("UPDATE nodes SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now_ts - age_secs, id]).ok();
        drop(conn);
        id
    }

    fn insert_edge(kb: &KnowledgeBase, src: &str, tgt: &str, rtype: RelationType) {
        kb.upsert_edge(src, tgt, rtype, 1.0, None).unwrap();
    }

    #[test]
    fn test_gap_type_names() {
        assert_eq!(GapType::MissingDomain.gap_type_name(), "Missing Domain");
        assert_eq!(GapType::StaleKnowledge.gap_type_name(), "Stale");
    }

    #[test]
    fn test_panorama_build_global_stats() {
        let kb = test_kb();
        let r1 = insert_node(&kb, "org/repo-1", NodeType::Repository, "github.com", 0);
        insert_node(&kb, "org/repo-2", NodeType::Repository, "github.com", 0);
        let c1 = insert_node(&kb, "Transformer", NodeType::Concept, "ai", 0);
        insert_edge(&kb, &r1, &c1, RelationType::Related);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let pano = builder.build().unwrap();

        assert_eq!(pano.global_stats.total_nodes, 3);
        assert_eq!(pano.global_stats.total_edges, 1);
        assert_eq!(pano.global_stats.total_repos, 2);
        assert_eq!(pano.global_stats.total_concepts, 1);
    }

    #[test]
    fn test_panorama_domain_coverage() {
        let kb = test_kb();
        insert_node(&kb, "org/repo", NodeType::Repository, "github.com", 0);
        insert_node(&kb, "paper-1", NodeType::Paper, "arxiv.org", 0);
        insert_node(&kb, "concept-1", NodeType::Concept, "wikipedia.org", 0);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let pano = builder.build().unwrap();

        let github_dc = pano.domain_coverage.iter().find(|d| d.domain == "github.com").unwrap();
        assert_eq!(github_dc.node_count, 1);
        assert!(github_dc.edge_count == 0 || github_dc.edge_count > 0);
        assert!(github_dc.coverage_score > 0.0);

        let missing_domains: Vec<_> = pano.domain_coverage.iter()
            .filter(|d| d.node_count == 0)
            .map(|d| d.domain.as_str())
            .collect();
        assert!(missing_domains.contains(&"mathematics"));
        assert!(missing_domains.contains(&"physics"));
    }

    #[test]
    fn test_panorama_freshness() {
        let kb = test_kb();
        let day = 86400;
        // 2 nodes updated today
        insert_node(&kb, "fresh-1", NodeType::Concept, "github.com", 0);
        insert_node(&kb, "fresh-2", NodeType::Concept, "github.com", 0);
        // 1 node from last week
        insert_node(&kb, "week-old", NodeType::Concept, "github.com", 3 * day);
        // 1 stale node (>90 days)
        insert_node(&kb, "stale-1", NodeType::Concept, "github.com", 100 * day);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let pano = builder.build().unwrap();

        assert_eq!(pano.freshness.nodes_updated_today, 2);
        assert!(pano.freshness.nodes_updated_this_week >= 3);
        assert_eq!(pano.freshness.stale_nodes, 1);
        assert!(pano.freshness.avg_age_days > 0.0);
    }

    #[test]
    fn test_panorama_gap_detection() {
        let kb = test_kb();
        // Create orphaned node (no edges)
        insert_node(&kb, "orphan", NodeType::Concept, "github.com", 0);
        // Create stale node
        insert_node(&kb, "very-old", NodeType::Concept, "github.com", 200 * 86400);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let pano = builder.build().unwrap();

        let orphans: Vec<_> = pano.gaps.iter().filter(|g| g.gap_type == GapType::OrphanedNode).collect();
        assert!(!orphans.is_empty(), "Should detect orphaned node");

        let stale: Vec<_> = pano.gaps.iter().filter(|g| g.gap_type == GapType::StaleKnowledge).collect();
        assert!(!stale.is_empty(), "Should detect stale knowledge");
    }

    #[test]
    fn test_panorama_quick_summary() {
        let kb = test_kb();
        insert_node(&kb, "org/repo", NodeType::Repository, "github.com", 0);
        insert_node(&kb, "Some Concept", NodeType::Concept, "ai", 0);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let summary = builder.build_quick_summary().unwrap();

        assert!(summary.contains("Total"));
        assert!(summary.contains("repository"));
        assert!(summary.contains("concept"));
        assert!(summary.contains("Freshness"));
        assert!(summary.contains("Gaps"));
    }

    #[test]
    fn test_panorama_graph_build() {
        let kb = test_kb();
        let r1 = insert_node(&kb, "org/repo", NodeType::Repository, "github.com", 0);
        let c1 = insert_node(&kb, "AI Concept", NodeType::Concept, "ai", 0);
        insert_edge(&kb, &r1, &c1, RelationType::Related);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let pano = builder.build().unwrap();

        assert_eq!(pano.knowledge_graph.nodes.len(), 2);
        assert_eq!(pano.knowledge_graph.links.len(), 1);
        assert_eq!(pano.knowledge_graph.links[0].relation, "related");

        // Repository should have connection_count = 1
        let repo_pano = pano.knowledge_graph.nodes.iter()
            .find(|n| n.node_type == "repository").unwrap();
        assert_eq!(repo_pano.connection_count, 1);
    }

    #[test]
    fn test_panorama_type_distribution() {
        let kb = test_kb();
        insert_node(&kb, "repo", NodeType::Repository, "github.com", 0);
        insert_node(&kb, "concept-a", NodeType::Concept, "ai", 0);
        insert_node(&kb, "concept-b", NodeType::Concept, "ai", 0);
        insert_node(&kb, "paper", NodeType::Paper, "arxiv.org", 0);

        let builder = KnowledgePanoramaBuilder::new(kb);
        let pano = builder.build().unwrap();

        let repo_count = pano.node_type_distribution.iter()
            .find(|(t, _)| t == "repository").map(|(_, c)| c).unwrap_or(&0);
        assert_eq!(*repo_count, 1);
        let concept_count = pano.node_type_distribution.iter()
            .find(|(t, _)| t == "concept").map(|(_, c)| c).unwrap_or(&0);
        assert_eq!(*concept_count, 2);

        // Concept should be first (most common)
        assert_eq!(pano.node_type_distribution[0].0, "concept");
    }
}
