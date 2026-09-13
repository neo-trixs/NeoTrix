//! SIE/Superbrain Semantic Extraction Pipeline
//!
//! 吸收 SIE (Semantic Information Extraction) + Superbrain:
//! - 实体提取 (命名实体识别)
//! - 关系提取 (实体关系图谱)
//! - 嵌入向量生成
//! - 知识图谱构建
//! - 语义搜索

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 语义提取管线 — SIE/Superbrain 核心
pub struct _SemanticExtractionPipeline {
    entity_extractor: _EntityExtractor,
    relation_extractor: _RelationExtractor,
    embedding_generator: _EmbeddingGenerator,
    knowledge_graph: KnowledgeGraph,
}

/// 实体提取器
pub struct _EntityExtractor {
    patterns: Vec<_EntityPattern>,
    #[allow(dead_code)]
    confidence_threshold: f64,
}

/// 实体模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _EntityPattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub regex: Option<String>,
    pub keywords: Vec<String>,
}

/// 模式类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    Regex,
    Keyword,
    NER,
    Dependency,
}

/// 提取的实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub properties: HashMap<String, serde_json::Value>,
    pub confidence: f64,
    pub source_text: String,
}

/// 实体类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Event,
    Concept,
    Technology,
    Code,
    File,
    Module,
    Function,
}

/// 关系提取器
pub struct _RelationExtractor {
    patterns: Vec<_RelationPattern>,
    #[allow(dead_code)]
    confidence_threshold: f64,
}

/// 关系模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RelationPattern {
    pub name: String,
    pub source_type: EntityType,
    pub target_type: EntityType,
    pub keywords: Vec<String>,
}

/// 提取的关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: String,
    pub source: Entity,
    pub target: Entity,
    pub relation_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub confidence: f64,
}

/// 嵌入生成器
pub struct _EmbeddingGenerator {
    #[allow(dead_code)]
    model: String,
    dimension: usize,
}

/// 知识图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<Entity>,
    pub edges: Vec<Relation>,
    pub embeddings: HashMap<String, Vec<f32>>,
}

/// 语义搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SemanticSearchResult {
    pub entity: Entity,
    pub score: f64,
    pub highlights: Vec<String>,
}

/// 提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub embedding: Option<Vec<f32>>,
    pub source: String,
    pub confidence: f64,
}

impl _SemanticExtractionPipeline {
    /// 创建新的提取管线
    pub fn new() -> Self {
        Self {
            entity_extractor: _EntityExtractor {
                patterns: vec![
                    _EntityPattern {
                        name: "person".into(),
                        pattern_type: PatternType::NER,
                        regex: None,
                        keywords: vec!["Mr.".into(), "Ms.".into(), "Dr.".into()],
                    },
                    _EntityPattern {
                        name: "organization".into(),
                        pattern_type: PatternType::NER,
                        regex: None,
                        keywords: vec!["Inc.".into(), "LLC".into(), "Corp.".into()],
                    },
                    _EntityPattern {
                        name: "technology".into(),
                        pattern_type: PatternType::Keyword,
                        regex: None,
                        keywords: vec![
                            "Rust".into(), "Python".into(), "TypeScript".into(),
                            "Docker".into(), "Kubernetes".into(), "MCP".into(),
                        ],
                    },
                    _EntityPattern {
                        name: "code".into(),
                        pattern_type: PatternType::Regex,
                        regex: Some(r"`[^`]+`".into()),
                        keywords: vec![],
                    },
                ],
                confidence_threshold: 0.6,
            },
            relation_extractor: _RelationExtractor {
                patterns: vec![
                    _RelationPattern {
                        name: "depends_on".into(),
                        source_type: EntityType::Module,
                        target_type: EntityType::Module,
                        keywords: vec!["depends on".into(), "requires".into(), "uses".into()],
                    },
                    _RelationPattern {
                        name: "implements".into(),
                        source_type: EntityType::Technology,
                        target_type: EntityType::Concept,
                        keywords: vec!["implements".into(), "provides".into(), "enables".into()],
                    },
                ],
                confidence_threshold: 0.6,
            },
            embedding_generator: _EmbeddingGenerator {
                model: "default".into(),
                dimension: 384,
            },
            knowledge_graph: KnowledgeGraph {
                nodes: vec![],
                edges: vec![],
                embeddings: HashMap::new(),
            },
        }
    }

    /// 从文本提取实体和关系
    pub fn extract(&mut self, text: &str) -> ExtractionResult {
        let entities = self.extract_entities(text);
        let relations = self.extract_relations(text, &entities);
        let embedding = self.generate_embedding(text);

        let confidence = if entities.is_empty() {
            0.0
        } else {
            entities.iter().map(|e| e.confidence).sum::<f64>() / entities.len() as f64
        };

        ExtractionResult {
            entities,
            relations,
            embedding,
            source: text.to_string(),
            confidence,
        }
    }

    /// 提取实体
    fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();

        for pattern in &self.entity_extractor.patterns {
            match pattern.pattern_type {
                PatternType::Keyword => {
                    for keyword in &pattern.keywords {
                        if text.to_lowercase().contains(&keyword.to_lowercase()) {
                            entities.push(Entity {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: keyword.clone(),
                                entity_type: EntityType::Technology,
                                properties: HashMap::new(),
                                confidence: 0.8,
                                source_text: keyword.clone(),
                            });
                        }
                    }
                }
                PatternType::Regex => {
                    if let Some(ref _regex_str) = pattern.regex {
                        // 简化版: 查找反引号中的代码
                        let mut start = 0;
                        while let Some(begin) = text[start..].find('`') {
                            let abs_begin = start + begin + 1;
                            if let Some(end) = text[abs_begin..].find('`') {
                                let code = &text[abs_begin..abs_begin + end];
                                entities.push(Entity {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name: code.to_string(),
                                    entity_type: EntityType::Code,
                                    properties: HashMap::new(),
                                    confidence: 0.9,
                                    source_text: format!("`{}`", code),
                                });
                                start = abs_begin + end + 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        entities
    }

    /// 提取关系
    fn extract_relations(&self, text: &str, entities: &[Entity]) -> Vec<Relation> {
        let mut relations = Vec::new();

        for pattern in &self.relation_extractor.patterns {
            for keyword in &pattern.keywords {
                if text.to_lowercase().contains(&keyword.to_lowercase()) {
                    // 简化版: 创建关系
                    if entities.len() >= 2 {
                        relations.push(Relation {
                            id: uuid::Uuid::new_v4().to_string(),
                            source: entities[0].clone(),
                            target: entities[1].clone(),
                            relation_type: pattern.name.clone(),
                            properties: HashMap::new(),
                            confidence: 0.7,
                        });
                    }
                }
            }
        }

        relations
    }

    /// 生成嵌入向量 (简化版)
    fn generate_embedding(&self, text: &str) -> Option<Vec<f32>> {
        // 简化版: 基于字符的哈希生成伪嵌入
        let mut embedding = vec![0.0f32; self.embedding_generator.dimension];
        for (i, c) in text.chars().enumerate() {
            if i < embedding.len() {
                embedding[i] = (c as u32 as f32) / 1000.0;
            }
        }
        Some(embedding)
    }

    /// 构建知识图谱
    pub fn build_graph(&mut self, extractions: &[ExtractionResult]) {
        for extraction in extractions {
            // 添加节点
            for entity in &extraction.entities {
                if !self.knowledge_graph.nodes.iter().any(|n| n.id == entity.id) {
                    self.knowledge_graph.nodes.push(entity.clone());
                }
            }

            // 添加边
            for relation in &extraction.relations {
                if !self.knowledge_graph.edges.iter().any(|e| e.id == relation.id) {
                    self.knowledge_graph.edges.push(relation.clone());
                }
            }

            // 添加嵌入
            if let Some(ref embedding) = extraction.embedding {
                if let Some(first_entity) = extraction.entities.first() {
                    self.knowledge_graph.embeddings
                        .insert(first_entity.id.clone(), embedding.clone());
                }
            }
        }
    }

    /// 语义搜索
    pub fn search(&self, query: &str) -> Vec<_SemanticSearchResult> {
        let query_embedding = self.generate_embedding(query);
        let mut results = Vec::new();

        for entity in &self.knowledge_graph.nodes {
            let score = if let Some(ref q_emb) = query_embedding {
                if let Some(e_emb) = self.knowledge_graph.embeddings.get(&entity.id) {
                    cosine_similarity_f32(q_emb, e_emb)
                } else {
                    0.0
                }
            } else {
                // 关键词匹配
                if entity.name.to_lowercase().contains(&query.to_lowercase()) {
                    0.8
                } else {
                    0.0
                }
            };

            if score > 0.1 {
                results.push(_SemanticSearchResult {
                    entity: entity.clone(),
                    score,
                    highlights: vec![entity.name.clone()],
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// 获取知识图谱
    pub fn get_graph(&self) -> &KnowledgeGraph {
        &self.knowledge_graph
    }
}

use crate::core::nt_core_math::cosine_similarity_f32;
