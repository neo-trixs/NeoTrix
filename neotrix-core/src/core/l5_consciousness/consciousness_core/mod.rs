//! 持久化意识核心 — 拆分后的模块结构
//!
//! - `core`: CoreSnapshot / ConsciousnessCoreHandle / 核心 tick/status
//! - `kb_persistence`: KB 快照读写 + 趋势追加 + 金标刷新
//! - `dispatch`: 意识核心任务环 (拆解/分配/内置调度/反思补齐)
//! - `external_closure`: 外部缺口闭环 (知识获取 + 试错求解)

pub mod core;
pub mod kb_persistence;
pub mod dispatch;
pub mod external_closure;

// Backward-compatible re-exports: 所有 pub 项从原路径 `nt_core_consciousness_core::*` 仍可达
pub use self::core::*;
pub use self::kb_persistence::*;
pub use self::dispatch::*;
pub use self::external_closure::*;
