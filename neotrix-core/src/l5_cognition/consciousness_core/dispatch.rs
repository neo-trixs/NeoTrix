//! 意识核心任务环 — 拆解/分配/内置调度/反思补齐（facade）
//! 
//! 实际实现已按职责拆入 `dispatch/` 子模块（纯搬移，行为零变更）；
//! 本文件仅保留 `pub mod` + `pub use`，外部 `dispatch::X` 路径保持不变。

pub mod nt_dispatch_entity;
pub mod nt_dispatch_loop;
pub mod nt_dispatch_registry;
pub mod nt_dispatch_routes;
pub mod nt_dispatch_types;

pub use self::nt_dispatch_entity::*;
pub use self::nt_dispatch_loop::*;
pub use self::nt_dispatch_registry::*;
pub use self::nt_dispatch_routes::*;
pub use self::nt_dispatch_types::*;
