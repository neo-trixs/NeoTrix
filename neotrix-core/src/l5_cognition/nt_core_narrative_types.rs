//! 叙事/节奏共享类型
//!
//! L4 认知层定义的基础数据类型，供 L5 (SEAL pipeline) 和 L6 (跨模块审计) 共用。
//! 消除 L6→L5 直接依赖，所有层通过 `crate::core::nt_core_narrative_types` 引用。

use serde::{Serialize, Deserialize};

/// 节段类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SegmentType {
    Setup,
    Conflict,
    Climax,
    Transition,
}

/// 节段数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentData {
    pub r#type: SegmentType,
    pub base_length: f32,
    pub content_priority: f32,
    pub is_core_scuang: bool,
}
