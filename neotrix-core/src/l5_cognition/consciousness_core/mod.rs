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

// T39-A4：旧 `nt_core_consciousness_core.rs`（4725 行）已删除；E2 即正典，
// `CoreSnapshot` / status / tick / 自测注册皆以此处为准，不再保留旧路径。
pub use self::core::*;
pub use self::kb_persistence::*;
pub use self::dispatch::*;
pub use self::external_closure::*;
