//! Knowledge Management Enhanced — 知识管理增强
//!
//! 吸收 Superbrain (知识管理/语义搜索):
//! - 知识图谱增强
//! - 语义搜索
//! - 知识推理
//! - 知识融合
//! - 知识演化

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 知识管理引擎
pub(crate) struct _KnowledgeManagementEngine {
    knowledge_graph: _EnhancedKnowledgeGraph,
    semantic_index: _SemanticIndex,
    reasoning_engine: ReasoningEngine,
    config: _KnowledgeConfig,
    stats: KnowledgeStats,
}

/// 知识配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _KnowledgeConfig {
    pub max_entities: usize,
    pub embedding_dim: usize,
    pub similarity_threshold: f64,
    pub enable_reasoning: bool,
    pub enable_fusion: bool,
}

impl Default for _KnowledgeConfig {
    fn default() -> Self {
        Self {
            max_entities: 100000,
            embedding_dim: 384,
            similarity_threshold: 0.7,
            enable_reasoning: true,
            enable_fusion: true,
        }
    }
}

/// 增强知识图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _EnhancedKnowledgeGraph {
    pub entities: HashMap<String, Entity>,
    pub relations: Vec<Relation>,
    pub embeddings: HashMap<String, Vec<f32>>,
    pub communities: Vec<Community>,
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
    pub importance: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation_type: String,
    pub weight: f64,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 社区
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: String,
    pub entities: Vec<String>,
    pub centroid: Vec<f32>,
    pub coherence: f64,
}

/// 语义索引
pub(crate) struct _SemanticIndex {
    index: HashMap<String, Vec<f32>>,
    metadata: HashMap<String, HashMap<String, String>>,
}

/// 推理引擎
pub struct ReasoningEngine {
    rules: Vec<ReasoningRule>,
    inference_cache: HashMap<String, Vec<String>>,
}

/// 推理规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningRule {
    pub id: String,
    pub rule_type: String,
    pub antecedent: String,
    pub consequent: String,
    pub confidence: f64,
}

/// 知识查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _KnowledgeQueryResult {
    pub query: String,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub inferred_facts: Vec<_InferredFact>,
    pub relevance_score: f64,
}

/// 推断事实
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _InferredFact {
    pub fact: String,
    pub confidence: f64,
    pub reasoning_chain: Vec<String>,
}

/// 知识统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_entities: u64,
    pub total_relations: u64,
    pub total_queries: u64,
    pub avg_query_time: f64,
    pub knowledge_quality: f64,
}

impl _KnowledgeManagementEngine {
    /// 创建新的知识管理引擎
    pub fn new(config: _KnowledgeConfig) -> Self {
        Self {
            knowledge_graph: _EnhancedKnowledgeGraph {
                entities: HashMap::new(),
                relations: Vec::new(),
                embeddings: HashMap::new(),
                communities: Vec::new(),
            },
            semantic_index: _SemanticIndex {
                index: HashMap::new(),
                metadata: HashMap::new(),
            },
            reasoning_engine: ReasoningEngine {
                rules: Vec::new(),
                inference_cache: HashMap::new(),
            },
            config,
            stats: KnowledgeStats {
                total_entities: 0,
                total_relations: 0,
                total_queries: 0,
                avg_query_time: 0.0,
                knowledge_quality: 0.0,
            },
        }
    }

    /// 添加实体
    pub fn add_entity(&mut self, entity: Entity) {
        if let Some(ref embedding) = entity.embedding {
            self.knowledge_graph.embeddings.insert(entity.id.clone(), embedding.clone());
            self.semantic_index.index.insert(entity.id.clone(), embedding.clone());
        }
        self.knowledge_graph.entities.insert(entity.id.clone(), entity);
        self.stats.total_entities += 1;
    }

    /// 添加关系
    pub fn add_relation(&mut self, relation: Relation) {
        self.knowledge_graph.relations.push(relation);
        self.stats.total_relations += 1;
    }

    /// 语义搜索
    pub fn semantic_search(&self, query_embedding: &[f32], top_k: usize) -> Vec<(Entity, f64)> {
        let mut results: Vec<(Entity, f64)> = self.knowledge_graph.embeddings.iter()
            .filter_map(|(id, emb)| {
                let similarity = cosine_similarity(query_embedding, emb);
                if similarity >= self.config.similarity_threshold {
                    self.knowledge_graph.entities.get(id).map(|e| (e.clone(), similarity))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.into_iter().take(top_k).collect()
    }

    /// 推理查询
    pub fn reason(&self, query: &str) -> Vec<_InferredFact> {
        let mut inferred = Vec::new();

        // 简化版: 基于规则的推理
        for rule in &self.reasoning_engine.rules {
            if query.contains(&rule.antecedent) {
                inferred.push(_InferredFact {
                    fact: rule.consequent.clone(),
                    confidence: rule.confidence,
                    reasoning_chain: vec![
                        format!("Observed: {}", rule.antecedent),
                        format!("Rule applied: {}", rule.id),
                        format!("Inferred: {}", rule.consequent),
                    ],
                });
            }
        }

        inferred
    }

    /// 知识融合
    pub(crate) fn _fuse_knowledge(&mut self, new_entities: Vec<Entity>, new_relations: Vec<Relation>) {
        if !self.config.enable_fusion {
            return;
        }

        // 合并实体
        for entity in new_entities {
            if let Some(existing) = self.knowledge_graph.entities.get_mut(&entity.id) {
                // 更新属性
                for (key, value) in entity.properties {
                    existing.properties.insert(key, value);
                }
                existing.last_updated = chrono::Utc::now();
            } else {
                self.add_entity(entity);
            }
        }

        // 合并关系
        for relation in new_relations {
            let exists = self.knowledge_graph.relations.iter().any(|r| {
                r.source == relation.source && r.target == relation.target && r.relation_type == relation.relation_type
            });

            if !exists {
                self.add_relation(relation);
            }
        }
    }

    /// 添加推理规则
    pub(crate) fn _add_reasoning_rule(&mut self, rule: ReasoningRule) {
        self.reasoning_engine.rules.push(rule);
    }

    /// 获取统计信息
    pub fn stats(&self) -> &KnowledgeStats {
        &self.stats
    }
}

use crate::core::nt_core_math::cosine_similarity_f32 as cosine_similarity;
