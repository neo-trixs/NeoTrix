//! NT-MEMORY Graphify 后端适配 (条目13, 源 github.com/Graphify-Labs/graphify)
//!
//! 参照: github.com/Graphify-Labs/graphify — 知识图谱构建 / 检索能力, 从文本中
//! 抽取实体与关系并构建可检索的知识图谱。吸收为 NT-MEMORY 知识层的图谱构建
//! 后端 trait, 将抽取结果映射为 KB 的 `KnowledgeNode` + `KnowledgeEdge`
//! (C1: T1 存在级 + 3 单元测)。
//!
//! 机制: `GraphifyBackend` 实现 `KnowledgeGraphBuilder` trait, 预留实体/关系
//! 抽取 → 图存储钩子, 复用 NeoTrix 现有 KB 图结构 (nt_memory_graph.rs 的
//! shortest_path / subgraph / community_detection)。

use crate::core::nt_core_kb_types::{KnowledgeEdge, KnowledgeNode, NodeType, RelationType};
use crate::core::nt_core_self_test::SelfTest;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use uuid::Uuid;

use super::shared_utils::now_ts;

/// 抽取出的实体。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExtractedEntity {
    pub label: String,
    pub entity_type: String,
}

/// 抽取出的关系 (head - rel -> tail)。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ExtractedRelation {
    pub head: String,
    pub relation: String,
    pub tail: String,
}

/// 知识图谱构建后端统一接口。
pub(crate) trait KnowledgeGraphBuilder: Send + Sync {
    /// 后端标识 (如 "graphify")。
    fn backend_id(&self) -> &str;
    /// 由文本抽取实体/关系 (stub: 行级 token 兜底抽取)。
    fn extract(&self, text: &str) -> Result<(Vec<ExtractedEntity>, Vec<ExtractedRelation>), String>;
    /// C2 接线: 将抽取结果作为 KB 节点 + 边写入图存储, 返回 (节点数, 边数)。
    fn build_graph(
        &self,
        kb: &KnowledgeBase,
        text: &str,
    ) -> Result<(usize, usize), String>;
    /// 后端是否可用 (可达性探测 stub)。
    fn is_available(&self) -> bool;
}

/// Graphify 后端实现 (stub)。
pub(crate) struct GraphifyBackend {
    pub endpoint: String,
}

impl Default for GraphifyBackend {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8789".to_string(),
        }
    }
}

impl KnowledgeGraphBuilder for GraphifyBackend {
    fn backend_id(&self) -> &str {
        "graphify"
    }

    fn extract(&self, text: &str) -> Result<(Vec<ExtractedEntity>, Vec<ExtractedRelation>), String> {
        if text.trim().is_empty() {
            return Err("text must not be empty".to_string());
        }
        let tokens: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|t| t.len() >= 3)
            .collect();
        let entities: Vec<ExtractedEntity> = tokens
            .iter()
            .map(|t| ExtractedEntity {
                label: t.to_string(),
                entity_type: "term".to_string(),
            })
            .collect();
        let mut relations = Vec::new();
        for w in tokens.windows(2) {
            relations.push(ExtractedRelation {
                head: w[0].to_string(),
                relation: "RelatedTo".to_string(),
                tail: w[1].to_string(),
            });
        }
        Ok((entities, relations))
    }

    fn build_graph(&self, kb: &KnowledgeBase, text: &str) -> Result<(usize, usize), String> {
        let (entities, relations) = self.extract(text)?;
        let now = now_ts();
        let mut node_ids: Vec<String> = Vec::with_capacity(entities.len());
        let mut node_count = 0;
        let mut edge_count = 0;
        for e in &entities {
            let node_id = Uuid::new_v4().to_string();
            let node = KnowledgeNode {
                id: node_id.clone(),
                node_type: NodeType::Concept,
                title: e.label.clone(),
                summary: Some(format!("Graphify entity ({})", e.entity_type)),
                content: None,
                url: None,
                domain: Some("graphify".to_string()),
                language: "en".to_string(),
                confidence: 0.85,
                importance: 0.5,
                recall_weight: 1.0,
                created_at: now,
                updated_at: now,
                access_count: 0,
                metadata: None,
                temporal: None,
                supersedes: None,
                source_episode: None,
                parent_id: None,
                depth: 0,
                cluster_id: None,
            };
            kb.insert_node(&node)?;
            node_ids.push(node_id);
            node_count += 1;
        }
        for (i, r) in relations.iter().enumerate() {
            if i + 1 >= node_ids.len() {
                break;
            }
            let edge = KnowledgeEdge {
                id: Uuid::new_v4().to_string(),
                source_id: node_ids[i].clone(),
                target_id: node_ids[i + 1].clone(),
                relation_type: RelationType::from_str(&r.relation),
                weight: 1.0,
                description: Some(format!("{} -> {} -> {}", r.head, r.relation, r.tail)),
                created_at: now,
                metadata: None,
            };
            kb.insert_edge(&edge)?;
            edge_count += 1;
        }
        Ok((node_count, edge_count))
    }

    fn is_available(&self) -> bool {
        !self.endpoint.is_empty()
    }
}

/// SelfTest (T1 存在级) — 验证后端 trait 契约与实体/关系抽取。
impl SelfTest for GraphifyBackend {
    fn name(&self) -> &str {
        "nt_memory_graphify"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.backend_id() != "graphify" {
            failures.push(format!("unexpected backend_id: {}", self.backend_id()));
        }
        if !self.is_available() {
            failures.push("backend reported unavailable".to_string());
        }
        match self.extract("alpha beta gamma") {
            Ok((ents, rels)) => {
                if ents.len() != 3 {
                    failures.push(format!("expected 3 entities, got {}", ents.len()));
                }
                if rels.len() != 2 {
                    failures.push(format!("expected 2 relations, got {}", rels.len()));
                }
            }
            Err(e) => failures.push(format!("extract failed: {e}")),
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_id_is_graphify() {
        let b = GraphifyBackend::default();
        assert_eq!(b.backend_id(), "graphify");
    }

    #[test]
    fn extract_yields_entities_and_relations() {
        let b = GraphifyBackend::default();
        let (ents, rels) = b.extract("rust cargo tokio").unwrap();
        assert_eq!(ents.len(), 3);
        assert_eq!(rels.len(), 2);
    }

    #[test]
    fn empty_text_is_rejected() {
        let b = GraphifyBackend::default();
        assert!(b.extract("   ").is_err());
    }
}
