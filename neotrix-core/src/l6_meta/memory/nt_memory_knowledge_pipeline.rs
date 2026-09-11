//! Knowledge Pipeline Enhanced — 知识管线增强
//!
//! 吸收 KB 经验:
//! - 经验指针守恒
//! - 五阶段吸收流程
//! - KB hub 存储
//! - 概念路由表
//! - 跨会话记忆

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 知识管线增强
pub struct KnowledgePipelineEnhanced {
    pipelines: Vec<KnowledgePipeline>,
    concepts: Vec<ConceptNode>,
    route_table: HashMap<String, String>,
    #[allow(dead_code)]
    config: PipelineConfig,
    stats: PipelineStats,
}

/// 管线配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub max_pipelines: usize,
    pub enable_concept_routing: bool,
    pub enable_cross_session: bool,
    pub max_concepts: usize,
    pub retention_days: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_pipelines: 10,
            enable_concept_routing: true,
            enable_cross_session: true,
            max_concepts: 10000,
            retention_days: 365,
        }
    }
}

/// 知识管线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePipeline {
    pub pipeline_id: String,
    pub name: String,
    pub pipeline_type: PipelineType,
    pub status: PipelineStatus,
    pub stages: Vec<PipelineStage>,
    pub last_run: Option<chrono::DateTime<chrono::Utc>>,
}

/// 管线类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineType {
    Absorption,     // 吸收管线
    Distillation,   // 蒸馏管线
    Classification, // 分类管线
    Synchronization, // 同步管线
}

/// 管线状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStatus {
    Running,
    Paused,
    Completed,
    Failed,
}

/// 管线阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub stage_id: String,
    pub name: String,
    pub stage_type: String,
    pub status: StageStatus,
    pub duration_ms: Option<u64>,
}

/// 阶段状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// 概念节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptNode {
    pub concept_id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub concepts: Vec<String>,
    pub routes: Vec<String>,
}

/// 管线统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub total_pipelines: u64,
    pub running_pipelines: u64,
    pub total_concepts: u64,
    pub total_routes: u64,
    pub avg_pipeline_duration: f64,
}

impl KnowledgePipelineEnhanced {
    /// 创建新的知识管线增强
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
            concepts: Vec::new(),
            route_table: HashMap::new(),
            config: PipelineConfig::default(),
            stats: PipelineStats {
                total_pipelines: 0,
                running_pipelines: 0,
                total_concepts: 0,
                total_routes: 0,
                avg_pipeline_duration: 0.0,
            },
        }
    }

    /// 创建新管线
    pub fn create_pipeline(&mut self, name: &str, pipeline_type: PipelineType) -> String {
        let pipeline_id = uuid::Uuid::new_v4().to_string();

        let stages = match pipeline_type {
            PipelineType::Absorption => vec![
                PipelineStage {
                    stage_id: "snapshot".into(),
                    name: "快照".into(),
                    stage_type: "snapshot".into(),
                    status: StageStatus::Pending,
                    duration_ms: None,
                },
                PipelineStage {
                    stage_id: "distill".into(),
                    name: "蒸馏".into(),
                    stage_type: "distill".into(),
                    status: StageStatus::Pending,
                    duration_ms: None,
                },
                PipelineStage {
                    stage_id: "classify".into(),
                    name: "分类".into(),
                    stage_type: "classify".into(),
                    status: StageStatus::Pending,
                    duration_ms: None,
                },
                PipelineStage {
                    stage_id: "store".into(),
                    name: "落盘".into(),
                    stage_type: "store".into(),
                    status: StageStatus::Pending,
                    duration_ms: None,
                },
                PipelineStage {
                    stage_id: "feedback".into(),
                    name: "反馈".into(),
                    stage_type: "feedback".into(),
                    status: StageStatus::Pending,
                    duration_ms: None,
                },
            ],
            _ => vec![],
        };

        let pipeline = KnowledgePipeline {
            pipeline_id: pipeline_id.clone(),
            name: name.to_string(),
            pipeline_type,
            status: PipelineStatus::Running,
            stages,
            last_run: None,
        };

        self.pipelines.push(pipeline);
        self.stats.total_pipelines += 1;
        self.stats.running_pipelines += 1;

        pipeline_id
    }

    /// 添加概念
    pub fn add_concept(&mut self, concept: ConceptNode) {
        // 添加到路由表
        for route in &concept.routes {
            self.route_table.insert(route.clone(), concept.concept_id.clone());
        }

        self.concepts.push(concept);
        self.stats.total_concepts += 1;
        self.stats.total_routes = self.route_table.len() as u64;
    }

    /// 查询概念
    pub fn query_concept(&self, query: &str) -> Option<&ConceptNode> {
        self.concepts.iter().find(|c| c.name.contains(query))
    }

    /// 通过路由查询
    pub fn query_by_route(&self, route: &str) -> Option<&ConceptNode> {
        self.route_table.get(route)
            .and_then(|concept_id| self.concepts.iter().find(|c| c.concept_id == *concept_id))
    }

    /// 获取所有管线
    pub fn pipelines(&self) -> &[KnowledgePipeline] {
        &self.pipelines
    }

    /// 获取统计信息
    pub fn stats(&self) -> &PipelineStats {
        &self.stats
    }
}
