#![forbid(unsafe_code)]

//! 跨域迁移主模块
//!
//! 整合 Reason-Retrieve-Refine 管线、分歧门控和实体映射，
//! 提供完整的跨域知识迁移能力。
//!
//! 基于 Agent KB 论文的跨域迁移框架设计

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    DisagreementGate, DisagreementGateConfig, GateDecision, GateEvaluation,
};
use super::{
    EntityDefinition, EntityMapper, EntityMappingConfig, MappingResult,
};
use super::{
    KnowledgeItem, ReasonRetrieveRefinePipeline,
    RrrPipelineConfig, TransferContext, TransferError,
};

/// 迁移任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferStatus {
    /// 待处理
    Pending,
    /// 推理中
    Reasoning,
    /// 检索中
    Retrieving,
    /// 精炼中
    Refining,
    /// 门控评估中
    GateEvaluation,
    /// 实体映射中
    EntityMapping,
    /// 完成
    Completed,
    /// 失败
    Failed,
}

/// 迁移任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTask {
    /// 任务ID
    pub task_id: String,
    /// 任务上下文
    pub context: TransferContext,
    /// 任务状态
    pub status: TransferStatus,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: Option<i64>,
    /// 迁移结果
    pub result: Option<TransferResult>,
    /// 错误信息
    pub error: Option<String>,
    /// 进度 (0.0-1.0)
    pub progress: f64,
}

/// 迁移结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    /// 精炼后的知识
    pub refined_knowledge: Vec<RefinedKnowledgeWithMapping>,
    /// 门控评估结果
    pub gate_evaluation: GateEvaluation,
    /// 实体映射结果
    pub entity_mapping_result: MappingResult,
    /// 迁移统计
    pub transfer_stats: TransferStats,
}

/// 带映射的精炼知识
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinedKnowledgeWithMapping {
    /// 精炼后的知识
    pub knowledge: super::reason_retrieve_refine::RefinedKnowledge,
    /// 相关的实体映射
    pub entity_mappings: Vec<super::EntityMapping>,
    /// 迁移路径
    pub transfer_path: Vec<String>,
}

/// 迁移统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStats {
    /// 总耗时 (ms)
    pub total_time_ms: u64,
    /// Reason 阶段耗时 (ms)
    pub reason_time_ms: u64,
    /// Retrieve 阶段耗时 (ms)
    pub retrieve_time_ms: u64,
    /// Refine 阶段耗时 (ms)
    pub refine_time_ms: u64,
    /// 门控评估耗时 (ms)
    pub gate_time_ms: u64,
    /// 实体映射耗时 (ms)
    pub mapping_time_ms: u64,
    /// 成功迁移的知识数量
    pub transferred_knowledge_count: usize,
    /// 总知识数量
    pub total_knowledge_count: usize,
    /// 平均置信度
    pub average_confidence: f64,
}

/// 跨域迁移配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossDomainTransferConfig {
    /// RRR 管线配置
    pub rrr_config: RrrPipelineConfig,
    /// 分歧门控配置
    pub gate_config: DisagreementGateConfig,
    /// 实体映射配置
    pub entity_mapping_config: EntityMappingConfig,
    /// 最大并行任务数
    pub max_concurrent_tasks: usize,
    /// 任务超时时间 (ms)
    pub task_timeout_ms: u64,
    /// 是否启用缓存
    pub enable_caching: bool,
}

impl Default for CrossDomainTransferConfig {
    fn default() -> Self {
        Self {
            rrr_config: RrrPipelineConfig::default(),
            gate_config: DisagreementGateConfig::default(),
            entity_mapping_config: EntityMappingConfig::default(),
            max_concurrent_tasks: 5,
            task_timeout_ms: 30_000,
            enable_caching: true,
        }
    }
}

/// 跨域迁移框架
pub struct CrossDomainTransferFramework {
    /// 配置
    config: CrossDomainTransferConfig,
    /// RRR 管线
    rrr_pipeline: ReasonRetrieveRefinePipeline,
    /// 分歧门控
    disagreement_gate: DisagreementGate,
    /// 实体映射器
    entity_mapper: EntityMapper,
    /// 任务队列
    task_queue: Vec<TransferTask>,
    /// 迁移历史
    history: Vec<TransferTask>,
    /// 缓存
    cache: HashMap<String, TransferResult>,
}

impl CrossDomainTransferFramework {
    /// 创建新的跨域迁移框架
    pub fn new(config: CrossDomainTransferConfig) -> Self {
        let rrr_pipeline = ReasonRetrieveRefinePipeline::new(config.rrr_config.clone());
        let disagreement_gate = DisagreementGate::new(config.gate_config.clone());
        let entity_mapper = EntityMapper::new(config.entity_mapping_config.clone());

        Self {
            config,
            rrr_pipeline,
            disagreement_gate,
            entity_mapper,
            task_queue: Vec::new(),
            history: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// 注册域映射
    pub fn register_domain_mapping(
        &mut self,
        mapping: super::DomainMapping,
    ) {
        self.rrr_pipeline.register_domain_mapping(mapping);
    }

    /// 注册实体定义
    pub fn register_entity_definition(&mut self, definition: EntityDefinition) {
        self.entity_mapper.register_entity_definition(definition);
    }

    /// 注册分歧模式
    pub fn register_disagreement_pattern(
        &mut self,
        pattern: super::DisagreementPattern,
    ) {
        self.disagreement_gate.register_pattern(pattern);
    }

    /// 提交迁移任务
    pub fn submit_task(&mut self, context: TransferContext) -> String {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task = TransferTask {
            task_id: task_id.clone(),
            context,
            status: TransferStatus::Pending,
            start_time: chrono::Utc::now().timestamp(),
            end_time: None,
            result: None,
            error: None,
            progress: 0.0,
        };

        self.task_queue.push(task);
        task_id
    }

    /// 执行迁移任务
    pub async fn execute_task(&mut self, task_id: &str) -> Result<TransferResult, TransferError> {
        // 查找任务
        let task_index = self
            .task_queue
            .iter()
            .position(|t| t.task_id == task_id)
            .ok_or_else(|| TransferError::ReasonPhaseError(format!("任务 {} 未找到", task_id)))?;

        // 检查缓存
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&self.task_queue[task_index].context);
            if let Some(cached_result) = self.cache.get(&cache_key) {
                return Ok(cached_result.clone());
            }
        }

        // 更新任务状态
        self.task_queue[task_index].status = TransferStatus::Reasoning;
        self.task_queue[task_index].progress = 0.1;

        let start_time = std::time::Instant::now();
        let context = self.task_queue[task_index].context.clone();

        // 1. 执行 Reason-Retrieve-Refine 管线
        let rrr_result = self.rrr_pipeline.execute_pipeline(&context).await?;

        // 更新进度
        self.task_queue[task_index].status = TransferStatus::GateEvaluation;
        self.task_queue[task_index].progress = 0.6;

        // 2. 执行分歧门控评估
        let gate_result = self.disagreement_gate.evaluate(
            &rrr_result
                .refined_knowledge
                .iter()
                .map(|k| KnowledgeItem {
                    id: k.original_id.clone(),
                    content: k.refined_content.clone(),
                    knowledge_type: k.adapted_type.clone(),
                    source_domain: context.source_domain.clone(),
                    metadata: HashMap::new(),
                    embedding: None,
                })
                .collect::<Vec<_>>(),
            &context,
        );

        // 检查门控决策
        match gate_result.decision {
            GateDecision::Block => {
                self.task_queue[task_index].status = TransferStatus::Failed;
                self.task_queue[task_index].error =
                    Some(format!("迁移被门控阻止: {}", gate_result.summary));
                return Err(TransferError::RefinePhaseError(format!(
                    "迁移被门控阻止: {}",
                    gate_result.summary
                )));
            }
            GateDecision::NeedsAdjustment => {
                // 需要调整，但继续执行
                log::warn!("迁移需要调整: {}", gate_result.summary);
            }
            _ => {}
        }

        // 更新进度
        self.task_queue[task_index].status = TransferStatus::EntityMapping;
        self.task_queue[task_index].progress = 0.8;

        // 3. 执行实体映射
        let mapping_result = self
            .entity_mapper
            .map_entities(&context.source_domain, &context.target_domain);

        // 合并结果
        let refined_knowledge_with_mapping =
            self.merge_rrr_and_mapping(&rrr_result.refined_knowledge, &mapping_result);

        let total_time = start_time.elapsed().as_millis() as u64;

        let result = TransferResult {
            refined_knowledge: refined_knowledge_with_mapping,
            gate_evaluation: gate_result,
            entity_mapping_result: mapping_result,
            transfer_stats: TransferStats {
                total_time_ms: total_time,
                reason_time_ms: rrr_result.refine_stats.refine_time_ms,
                retrieve_time_ms: 0, // 从 RRR 结果中提取
                refine_time_ms: rrr_result.refine_stats.refine_time_ms,
                gate_time_ms: 0,    // 门控评估时间
                mapping_time_ms: 0, // 映射时间
                transferred_knowledge_count: rrr_result.refined_knowledge.len(),
                total_knowledge_count: rrr_result.refine_stats.input_count,
                average_confidence: rrr_result.refine_stats.average_adaptation,
            },
        };

        // 更新任务状态
        self.task_queue[task_index].status = TransferStatus::Completed;
        self.task_queue[task_index].progress = 1.0;
        self.task_queue[task_index].end_time = Some(chrono::Utc::now().timestamp());
        self.task_queue[task_index].result = Some(result.clone());

        // 缓存结果
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&context);
            self.cache.insert(cache_key, result.clone());
        }

        // 移动到历史记录
        let task = self.task_queue.remove(task_index);
        self.history.push(task);

        Ok(result)
    }

    /// 合并 RRR 结果和实体映射结果
    fn merge_rrr_and_mapping(
        &self,
        refined_knowledge: &[super::reason_retrieve_refine::RefinedKnowledge],
        mapping_result: &MappingResult,
    ) -> Vec<RefinedKnowledgeWithMapping> {
        refined_knowledge
            .iter()
            .map(|k| {
                // 查找相关的实体映射
                let relevant_mappings = mapping_result
                    .mappings
                    .iter()
                    .filter(|m| {
                        m.source_entity_id == k.original_id || m.target_entity_id == k.original_id
                    })
                    .cloned()
                    .collect();

                RefinedKnowledgeWithMapping {
                    knowledge: k.clone(),
                    entity_mappings: relevant_mappings,
                    transfer_path: vec![
                        format!("从源域检索"),
                        format!("精炼适配"),
                        format!("映射到目标域"),
                    ],
                }
            })
            .collect()
    }

    /// 生成缓存键
    fn generate_cache_key(&self, context: &TransferContext) -> String {
        format!(
            "{}_{}_{:?}",
            context.source_domain, context.target_domain, context.task_type
        )
    }

    /// 获取任务状态
    pub fn get_task_status(&self, task_id: &str) -> Option<&TransferTask> {
        self.task_queue
            .iter()
            .find(|t| t.task_id == task_id)
            .or_else(|| self.history.iter().find(|t| t.task_id == task_id))
    }

    /// 获取历史统计
    pub fn get_history_stats(&self) -> TransferHistoryStats {
        let total_tasks = self.history.len();
        let completed_tasks = self
            .history
            .iter()
            .filter(|t| t.status == TransferStatus::Completed)
            .count();
        let failed_tasks = self
            .history
            .iter()
            .filter(|t| t.status == TransferStatus::Failed)
            .count();

        let average_time = if completed_tasks == 0 {
            0.0
        } else {
            self.history
                .iter()
                .filter(|t| t.status == TransferStatus::Completed)
                .map(|t| {
                    t.result
                        .as_ref()
                        .map(|r| r.transfer_stats.total_time_ms as f64)
                        .unwrap_or(0.0)
                })
                .sum::<f64>()
                / completed_tasks as f64
        };

        let average_confidence = if completed_tasks == 0 {
            0.0
        } else {
            self.history
                .iter()
                .filter(|t| t.status == TransferStatus::Completed)
                .map(|t| {
                    t.result
                        .as_ref()
                        .map(|r| r.transfer_stats.average_confidence)
                        .unwrap_or(0.0)
                })
                .sum::<f64>()
                / completed_tasks as f64
        };

        TransferHistoryStats {
            total_tasks,
            completed_tasks,
            failed_tasks,
            average_time_ms: average_time,
            average_confidence,
        }
    }

    /// 清理缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// 迁移历史统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistoryStats {
    /// 总任务数
    pub total_tasks: usize,
    /// 完成任务数
    pub completed_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 平均耗时 (ms)
    pub average_time_ms: f64,
    /// 平均置信度
    pub average_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_mind::cross_domain::{Constraint, ConstraintType, TaskType};

    fn create_test_context() -> TransferContext {
        TransferContext {
            source_domain: "rust".to_string(),
            target_domain: "python".to_string(),
            task_description: "所有权系统".to_string(),
            task_type: TaskType::ConceptTransfer,
            constraints: vec![Constraint {
                constraint_type: ConstraintType::SemanticSimilarity,
                value: "memory_management".to_string(),
                strength: 0.7,
            }],
        }
    }

    #[test]
    fn test_framework_config_default() {
        let config = CrossDomainTransferConfig::default();
        assert_eq!(config.max_concurrent_tasks, 5);
        assert_eq!(config.task_timeout_ms, 30_000);
        assert!(config.enable_caching);
    }

    #[tokio::test]
    async fn test_task_lifecycle() {
        let config = CrossDomainTransferConfig::default();
        let mut framework = CrossDomainTransferFramework::new(config);

        // 提交任务
        let context = create_test_context();
        let task_id = framework.submit_task(context);

        // 检查任务状态
        let task = framework.get_task_status(&task_id).unwrap();
        assert_eq!(task.status, TransferStatus::Pending);

        // 执行任务
        let result = framework.execute_task(&task_id).await;
        assert!(result.is_ok());

        // 检查完成状态
        let task = framework.get_task_status(&task_id).unwrap();
        assert_eq!(task.status, TransferStatus::Completed);
    }

    #[test]
    fn test_history_stats() {
        let config = CrossDomainTransferConfig::default();
        let framework = CrossDomainTransferFramework::new(config);

        let stats = framework.get_history_stats();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.completed_tasks, 0);
        assert_eq!(stats.failed_tasks, 0);
    }
}
