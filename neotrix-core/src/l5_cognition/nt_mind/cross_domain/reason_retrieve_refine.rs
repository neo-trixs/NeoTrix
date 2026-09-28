#![forbid(unsafe_code)]

//! Reason-Retrieve-Refine 管线
//!
//! 基于 Agent KB 论文的核心模式：
//! 1. Reason: 分析任务需求，生成检索查询
//! 2. Retrieve: 从知识库中检索相关知识
//! 3. Refine: 对检索结果进行精炼和适配
//!
//! 跨域迁移的关键是保持语义一致性，同时适配目标域的上下文

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 域标识符
pub type DomainId = String;

/// 知识片段标识符
pub type KnowledgeId = String;

/// 迁移任务上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferContext {
    /// 源域标识符
    pub source_domain: DomainId,
    /// 目标域标识符
    pub target_domain: DomainId,
    /// 任务描述
    pub task_description: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 约束条件
    pub constraints: Vec<Constraint>,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    /// 类比迁移
    AnalogyTransfer,
    /// 模式迁移
    PatternTransfer,
    /// 概念迁移
    ConceptTransfer,
    /// 方法迁移
    MethodTransfer,
    /// 结构迁移
    StructureTransfer,
}

/// 约束条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// 约束类型
    pub constraint_type: ConstraintType,
    /// 约束值
    pub value: String,
    /// 约束强度 (0.0-1.0)
    pub strength: f64,
}

/// 约束类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConstraintType {
    /// 语义相似度阈值
    SemanticSimilarity,
    /// 结构一致性
    StructuralConsistency,
    /// 领域兼容性
    DomainCompatibility,
    /// 复杂度限制
    ComplexityLimit,
}

/// Reason 阶段结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonResult {
    /// 生成的检索查询
    pub queries: Vec<RetrievalQuery>,
    /// 推理过程
    pub reasoning_trace: Vec<RefineStep>,
    /// 预期的知识类型
    pub expected_knowledge_types: Vec<KnowledgeType>,
    /// 置信度
    pub confidence: f64,
}

/// 检索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalQuery {
    /// 查询文本
    pub query: String,
    /// 查询向量 (可选)
    pub query_vector: Option<Vec<f32>>,
    /// 查询意图
    pub intent: QueryIntent,
    /// 权重
    pub weight: f64,
}

/// 查询意图
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum QueryIntent {
    /// 查找相似概念
    FindSimilarConcept,
    /// 查找对应模式
    FindCorrespondingPattern,
    /// 查找类比关系
    FindAnalogyRelation,
    /// 查找迁移路径
    FindTransferPath,
    /// 查找验证案例
    FindValidationCase,
}

/// 推理步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineStep {
    /// 步骤类型
    pub step_type: ReasoningStepType,
    /// 步骤描述
    pub description: String,
    /// 输入
    pub input: HashMap<String, String>,
    /// 输出
    pub output: HashMap<String, String>,
    /// 置信度
    pub confidence: f64,
}

/// 推理步骤类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasoningStepType {
    /// 任务分析
    TaskAnalysis,
    /// 域映射
    DomainMapping,
    /// 查询生成
    QueryGeneration,
    /// 约束推导
    ConstraintDerivation,
    /// 验证假设
    ValidationHypothesis,
}

/// 知识类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeType {
    /// 概念知识
    Concept,
    /// 程序知识
    Procedural,
    /// 结构知识
    Structural,
    /// 启发式知识
    Heuristic,
    /// 验证知识
    Validation,
}

/// Retrieve 阶段结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResult {
    /// 检索到的知识片段
    pub knowledge_items: Vec<KnowledgeItem>,
    /// 检索统计
    pub retrieval_stats: RetrievalStats,
    /// 相关性评分
    pub relevance_scores: HashMap<KnowledgeId, f64>,
}

/// 知识片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeItem {
    /// 知识ID
    pub id: KnowledgeId,
    /// 知识内容
    pub content: String,
    /// 知识类型
    pub knowledge_type: KnowledgeType,
    /// 来源域
    pub source_domain: DomainId,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 嵌入向量 (可选)
    pub embedding: Option<Vec<f32>>,
}

/// 检索统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalStats {
    /// 总检索数量
    pub total_retrieved: usize,
    /// 高相关性数量
    pub high_relevance_count: usize,
    /// 平均相关性
    pub average_relevance: f64,
    /// 检索耗时
    pub retrieval_time_ms: u64,
}

/// Refine 阶段结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineResult {
    /// 精炼后的知识
    pub refined_knowledge: Vec<RefinedKnowledge>,
    /// 精炼统计
    pub refine_stats: RefineStats,
    /// 适配度评分
    pub adaptation_scores: HashMap<KnowledgeId, f64>,
}

/// 精炼后的知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedKnowledge {
    /// 原始知识ID
    pub original_id: KnowledgeId,
    /// 精炼后的内容
    pub refined_content: String,
    /// 适配后的类型
    pub adapted_type: KnowledgeType,
    /// 适配度
    pub adaptation_score: f64,
    /// 变换描述
    pub transformation_description: String,
}

/// 精炼统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineStats {
    /// 输入知识数量
    pub input_count: usize,
    /// 输出知识数量
    pub output_count: usize,
    /// 平均适配度
    pub average_adaptation: f64,
    /// 精炼耗时
    pub refine_time_ms: u64,
}

/// Reason-Retrieve-Refine 管线配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RrrPipelineConfig {
    /// 最大检索数量
    pub max_retrieval_count: usize,
    /// 最小相关性阈值
    pub min_relevance_threshold: f64,
    /// 最小适配度阈值
    pub min_adaptation_threshold: f64,
    /// 是否启用向量检索
    pub enable_vector_retrieval: bool,
    /// 是否启用语义精炼
    pub enable_semantic_refinement: bool,
    /// 并行处理数量
    pub parallel_count: usize,
}

impl Default for RrrPipelineConfig {
    fn default() -> Self {
        Self {
            max_retrieval_count: 10,
            min_relevance_threshold: 0.3,
            min_adaptation_threshold: 0.5,
            enable_vector_retrieval: true,
            enable_semantic_refinement: true,
            parallel_count: 4,
        }
    }
}

/// Reason-Retrieve-Refine 管线
pub struct ReasonRetrieveRefinePipeline {
    /// 配置
    config: RrrPipelineConfig,
    /// 域映射表
    domain_mappings: HashMap<DomainId, DomainMapping>,
}

/// 域映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainMapping {
    /// 源域
    pub source_domain: DomainId,
    /// 目标域
    pub target_domain: DomainId,
    /// 概念映射
    pub concept_mappings: HashMap<String, String>,
    /// 模式映射
    pub pattern_mappings: HashMap<String, String>,
    /// 映射置信度
    pub confidence: f64,
}

impl ReasonRetrieveRefinePipeline {
    /// 创建新的管线
    pub fn new(config: RrrPipelineConfig) -> Self {
        Self {
            config,
            domain_mappings: HashMap::new(),
        }
    }

    /// 注册域映射
    pub fn register_domain_mapping(&mut self, mapping: DomainMapping) {
        self.domain_mappings
            .insert(mapping.source_domain.clone(), mapping);
    }

    /// 执行完整的 Reason-Retrieve-Refine 管线
    pub async fn execute_pipeline(
        &self,
        context: &TransferContext,
    ) -> Result<RefineResult, TransferError> {
        // 1. Reason 阶段
        let reason_result = self.reason_phase(context).await?;

        // 2. Retrieve 阶段
        let retrieve_result = self.retrieve_phase(context, &reason_result).await?;

        // 3. Refine 阶段
        let refine_result = self
            .refine_phase(context, &reason_result, &retrieve_result)
            .await?;

        Ok(refine_result)
    }

    /// Reason 阶段：分析任务需求，生成检索查询
    async fn reason_phase(&self, context: &TransferContext) -> Result<ReasonResult, TransferError> {
        let mut queries = Vec::new();
        let mut reasoning_trace = Vec::new();
        let mut expected_knowledge_types = Vec::new();

        // 任务分析步骤
        let task_analysis = RefineStep {
            step_type: ReasoningStepType::TaskAnalysis,
            description: format!("分析迁移任务: {}", context.task_description),
            input: HashMap::new(),
            output: [("task_type".to_string(), format!("{:?}", context.task_type))]
                .into_iter()
                .collect(),
            confidence: 0.8,
        };
        reasoning_trace.push(task_analysis);

        // 域映射步骤
        if let Some(mapping) = self.domain_mappings.get(&context.source_domain) {
            let domain_mapping = RefineStep {
                step_type: ReasoningStepType::DomainMapping,
                description: format!(
                    "映射域 {} -> {}",
                    context.source_domain, context.target_domain
                ),
                input: [("source".to_string(), context.source_domain.clone())]
                    .into_iter()
                    .collect(),
                output: [("target".to_string(), context.target_domain.clone())]
                    .into_iter()
                    .collect(),
                confidence: mapping.confidence,
            };
            reasoning_trace.push(domain_mapping);
        }

        // 生成检索查询
        match context.task_type {
            TaskType::AnalogyTransfer => {
                queries.push(RetrievalQuery {
                    query: format!(
                        "类似 {} 在 {} 中的对应概念",
                        context.task_description, context.target_domain
                    ),
                    query_vector: None,
                    intent: QueryIntent::FindSimilarConcept,
                    weight: 1.0,
                });
                expected_knowledge_types.push(KnowledgeType::Concept);
            }
            TaskType::PatternTransfer => {
                queries.push(RetrievalQuery {
                    query: format!("{} 的通用模式", context.task_description),
                    query_vector: None,
                    intent: QueryIntent::FindCorrespondingPattern,
                    weight: 1.0,
                });
                expected_knowledge_types.push(KnowledgeType::Procedural);
            }
            TaskType::ConceptTransfer => {
                queries.push(RetrievalQuery {
                    query: format!("{} 的核心概念", context.task_description),
                    query_vector: None,
                    intent: QueryIntent::FindSimilarConcept,
                    weight: 1.0,
                });
                expected_knowledge_types.push(KnowledgeType::Concept);
            }
            TaskType::MethodTransfer => {
                queries.push(RetrievalQuery {
                    query: format!("{} 的实现方法", context.task_description),
                    query_vector: None,
                    intent: QueryIntent::FindTransferPath,
                    weight: 1.0,
                });
                expected_knowledge_types.push(KnowledgeType::Procedural);
            }
            TaskType::StructureTransfer => {
                queries.push(RetrievalQuery {
                    query: format!("{} 的结构设计", context.task_description),
                    query_vector: None,
                    intent: QueryIntent::FindCorrespondingPattern,
                    weight: 1.0,
                });
                expected_knowledge_types.push(KnowledgeType::Structural);
            }
        }

        let confidence = reasoning_trace.iter().map(|s| s.confidence).sum::<f64>()
            / reasoning_trace.len() as f64;

        Ok(ReasonResult {
            queries,
            reasoning_trace,
            expected_knowledge_types,
            confidence,
        })
    }

    /// Retrieve 阶段：从知识库中检索相关知识
    async fn retrieve_phase(
        &self,
        context: &TransferContext,
        reason_result: &ReasonResult,
    ) -> Result<RetrieveResult, TransferError> {
        let start_time = std::time::Instant::now();
        let mut knowledge_items = Vec::new();
        let mut relevance_scores = HashMap::new();

        // 这里应该实际查询知识库，现在返回模拟结果
        // 在实际实现中，需要调用 nt_memory_kb 的检索接口
        for query in &reason_result.queries {
            // 模拟检索结果
            let item = KnowledgeItem {
                id: format!("kb_{}", uuid::Uuid::new_v4()),
                content: format!("基于查询 '{}' 的知识片段", query.query),
                knowledge_type: reason_result
                    .expected_knowledge_types
                    .first()
                    .cloned()
                    .unwrap_or(KnowledgeType::Concept),
                source_domain: context.source_domain.clone(),
                metadata: HashMap::new(),
                embedding: None,
            };
            let relevance = 0.7; // 模拟相关性
            relevance_scores.insert(item.id.clone(), relevance);
            knowledge_items.push(item);
        }

        let retrieval_time = start_time.elapsed().as_millis() as u64;
        let high_relevance_count = relevance_scores
            .values()
            .filter(|&&r| r >= self.config.min_relevance_threshold)
            .count();

        Ok(RetrieveResult {
            knowledge_items: knowledge_items.clone(),
            retrieval_stats: RetrievalStats {
                total_retrieved: knowledge_items.len(),
                high_relevance_count,
                average_relevance: if relevance_scores.is_empty() {
                    0.0
                } else {
                    relevance_scores.values().sum::<f64>() / relevance_scores.len() as f64
                },
                retrieval_time_ms: retrieval_time,
            },
            relevance_scores,
        })
    }

    /// Refine 阶段：对检索结果进行精炼和适配
    async fn refine_phase(
        &self,
        context: &TransferContext,
        _reason_result: &ReasonResult,
        retrieve_result: &RetrieveResult,
    ) -> Result<RefineResult, TransferError> {
        let start_time = std::time::Instant::now();
        let mut refined_knowledge = Vec::new();
        let mut adaptation_scores = HashMap::new();

        for item in &retrieve_result.knowledge_items {
            // 计算适配度
            let adaptation_score = self.calculate_adaptation_score(item, context);

            if adaptation_score >= self.config.min_adaptation_threshold {
                let refined_content = self.refine_content(&item.content, context);
                let adapted_type = self.adapt_knowledge_type(&item.knowledge_type, context);

                let refined = RefinedKnowledge {
                    original_id: item.id.clone(),
                    refined_content,
                    adapted_type,
                    adaptation_score,
                    transformation_description: format!(
                        "从 {} 域适配到 {} 域",
                        item.source_domain, context.target_domain
                    ),
                };
                adaptation_scores.insert(item.id.clone(), adaptation_score);
                refined_knowledge.push(refined);
            }
        }

        let refine_time = start_time.elapsed().as_millis() as u64;
        let average_adaptation = if adaptation_scores.is_empty() {
            0.0
        } else {
            adaptation_scores.values().sum::<f64>() / adaptation_scores.len() as f64
        };

        Ok(RefineResult {
            refined_knowledge: refined_knowledge.clone(),
            refine_stats: RefineStats {
                input_count: retrieve_result.knowledge_items.len(),
                output_count: refined_knowledge.len(),
                average_adaptation,
                refine_time_ms: refine_time,
            },
            adaptation_scores,
        })
    }

    /// 计算适配度
    fn calculate_adaptation_score(&self, item: &KnowledgeItem, context: &TransferContext) -> f64 {
        // 基础适配度
        let mut score = 0.5;

        // 如果有域映射，增加分数
        if self.domain_mappings.contains_key(&item.source_domain) {
            score += 0.2;
        }

        // 根据约束条件调整分数
        for constraint in &context.constraints {
            match constraint.constraint_type {
                ConstraintType::SemanticSimilarity => {
                    score += constraint.strength * 0.1;
                }
                ConstraintType::StructuralConsistency => {
                    score += constraint.strength * 0.1;
                }
                _ => {}
            }
        }

        score.min(1.0)
    }

    /// 精炼内容
    fn refine_content(&self, content: &str, context: &TransferContext) -> String {
        // 基础精炼：添加域适配标记
        format!(
            "[适配自 {} 域] {} [目标域: {}]",
            context.source_domain, content, context.target_domain
        )
    }

    /// 适配知识类型
    fn adapt_knowledge_type(
        &self,
        original_type: &KnowledgeType,
        context: &TransferContext,
    ) -> KnowledgeType {
        // 根据目标域的特点适配知识类型
        match context.task_type {
            TaskType::PatternTransfer => KnowledgeType::Procedural,
            TaskType::StructureTransfer => KnowledgeType::Structural,
            _ => original_type.clone(),
        }
    }
}

/// 迁移错误类型
#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("Reason 阶段失败: {0}")]
    ReasonPhaseError(String),

    #[error("Retrieve 阶段失败: {0}")]
    RetrievePhaseError(String),

    #[error("Refine 阶段失败: {0}")]
    RefinePhaseError(String),

    #[error("域映射未找到: {0}")]
    DomainMappingNotFound(String),

    #[error("知识库查询失败: {0}")]
    KnowledgeBaseQueryError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = RrrPipelineConfig::default();
        assert_eq!(config.max_retrieval_count, 10);
        assert_eq!(config.min_relevance_threshold, 0.3);
        assert_eq!(config.min_adaptation_threshold, 0.5);
        assert!(config.enable_vector_retrieval);
        assert!(config.enable_semantic_refinement);
    }

    #[test]
    fn test_task_type_variants() {
        assert_eq!(TaskType::AnalogyTransfer, TaskType::AnalogyTransfer);
        assert_eq!(TaskType::PatternTransfer, TaskType::PatternTransfer);
        assert_eq!(TaskType::ConceptTransfer, TaskType::ConceptTransfer);
        assert_eq!(TaskType::MethodTransfer, TaskType::MethodTransfer);
        assert_eq!(TaskType::StructureTransfer, TaskType::StructureTransfer);
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let config = RrrPipelineConfig::default();
        let mut pipeline = ReasonRetrieveRefinePipeline::new(config);

        // 注册域映射
        let mapping = DomainMapping {
            source_domain: "rust".to_string(),
            target_domain: "python".to_string(),
            concept_mappings: HashMap::new(),
            pattern_mappings: HashMap::new(),
            confidence: 0.8,
        };
        pipeline.register_domain_mapping(mapping);

        // 创建迁移上下文
        let context = TransferContext {
            source_domain: "rust".to_string(),
            target_domain: "python".to_string(),
            task_description: "所有权系统".to_string(),
            task_type: TaskType::ConceptTransfer,
            constraints: vec![Constraint {
                constraint_type: ConstraintType::SemanticSimilarity,
                value: "memory_management".to_string(),
                strength: 0.7,
            }],
        };

        // 执行管线
        let result = pipeline.execute_pipeline(&context).await.unwrap();
        assert!(result.refine_stats.output_count > 0);
    }
}
