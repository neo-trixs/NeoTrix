//! L2 Perception Layer Traits
//!
//! 感知层合约: 世界感知 (nt_world) + 感官处理 (nt_sense)
//! 吸收来源: SIE (语义提取), Superbrain (知识图谱), OSINT Arsenal

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 感知事件 — 从世界获取的原始信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionEvent {
    pub source: String,
    pub event_type: PerceptionEventType,
    pub payload: serde_json::Value,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 感知事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PerceptionEventType {
    Text,
    Image,
    Audio,
    Network,
    FileChange,
    Security,
    Semantic,
}

/// 语义提取结果 — SIE/Superbrain 吸收
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticExtraction {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub embedding: Option<Vec<f32>>,
    pub source_url: Option<String>,
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub source: String,
    pub target: String,
    pub relation_type: String,
    pub weight: f64,
}

/// 感知层核心合约
pub trait PerceptionLayer: Send + Sync {
    /// 初始化感知层
    fn initialize(&mut self) -> Result<(), String>;

    /// 处理感知事件
    fn process_event(&mut self, event: PerceptionEvent) -> Result<PerceptionEvent, String>;

    /// 语义提取 — 从文本提取实体和关系 (SIE 吸收)
    fn extract_semantics(&self, text: &str) -> Result<SemanticExtraction, String>;

    /// 知识图谱构建 — 从感知数据构建图谱 (Superbrain 吸收)
    fn build_knowledge_graph(
        &self,
        extractions: &[SemanticExtraction],
    ) -> Result<HashMap<String, Vec<(String, String)>>, String>;

    /// OSINT 情报收集 — OSINT Arsenal 吸收
    fn gather_intelligence(&self, target: &str) -> Result<Vec<PerceptionEvent>, String>;

    /// 获取感知状态快照
    fn snapshot(&self) -> PerceptionSnapshot;
}

/// 感知状态快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionSnapshot {
    pub active_sources: Vec<String>,
    pub event_count: u64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// 知识写入抽象 — L2 感知层对 L1 知识层的写入接口
///
/// L2 模块通过此 trait 写入知识，而非直接依赖 L1 KnowledgeBase 具体类型。
/// 实现者: L1 KnowledgeBase (via blanket or manual impl).
/// Trait 定义在 core/nt_core_traits，此处 re-export 供 L2 引用。
pub use crate::core::nt_core_traits::KnowledgeSink;
