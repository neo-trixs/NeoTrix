//! # L1 — 身体层 (Body)
//!
//! 与世界交互的接口、安全边界、行动工具。
//! 科幻映射: GitS Shell / Matrix 程序的形态 / Lain 终端
//!
//! ## 规则
//! - L1 是唯一可以并行执行多个能力的层
//! - L1 的执行必须通过模式链验证 (plan→acceptEdits→bypassPermissions→execute)
//! - L1 不产生推理 (那是 L4 的工作) — 只执行
//!
//! ## 实现位置
//! L1 的纯数据模型定义在 core/ 层，完整实现在 neotrix/l1_body_impl/
//! - `neotrix::nt_shield` — 安全系统实现
//! - `neotrix::nt_act_*` — 行动工具实现
//! - `neotrix::nt_io_*` — IO 接口实现
//! - `agent::executor` — 智能体执行器

// 公开 re-exports of telemetry types from core layer
#[cfg(feature = "telemetry")]
pub use crate::core::nt_io_telemetry::{
    AttributeValue, ConsoleTracer, CostTracker, NoopTracer, SpanKind, Tracer,
};

// L1 的纯数据模型当前在 neotrix/ 中定义
// 随着迁移推进，trait 和数据类型会逐步移入此处
