//! NT-ACT Cleanup - 清理执行层
//!
//! 安全删除、缓存清理、开发工具清理
//! 域: NT-ACT (行动执行者)
//! 层: L1 Action
//!
//! 吸收 PureMac (MIT) 能力:
//! - `locations` — allow-list / 云 provider 拒绝根
//! - `exclusions` — 用户排除表 (持久化)
//! - `safety` — 符号链接 + 凭据 + TOCTOU 复测
//! - `catalog` — dev/junk/ai/trash 分类目标
//! - `scan_engine` — 分类扫描 → CleanableItem
//! - `cleaning_engine` — allow-list 守卫下的安全删除

pub mod cache_cleaner;
pub mod catalog;
pub mod cleaning_engine;
pub mod dev_tool_cleaner;
pub mod exclusions;
pub mod locations;
pub mod safe_deleter;
pub mod safety;
pub mod scan_engine;
pub mod shared;

pub use cleaning_engine::{CleaningConfig, CleaningEngine, CleaningResult};
pub use exclusions::CleanupExclusions;
pub use scan_engine::{scan_all, scan_category, CategoryResult, CleanableItem};
pub use shared::*;
