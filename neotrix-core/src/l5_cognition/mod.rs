pub mod traits;
pub mod nt_core;
pub mod nt_mind;
/// KB Facade — L5 对 L1 NT-MEMORY KB 类型的 re-export 门面
pub mod kb_facade;
/// IO Facade — L5 对 L1 NT-IO 共享类型的 re-export 门面
pub mod io_facade;
/// IO Skills Facade — L5 对 L1 NT-IO 技能模块的 re-export 门面
pub mod io_skills_facade;
/// ACT Facade — L5 对 L1 NT-ACT 共享类型的 re-export 门面
pub mod act_facade;
/// L3 Facade — L5 对 L3 共享类型的 re-export 门面
pub mod l3_facade;
/// L6 Facade — L5 对 L6 元认知层类型的集中 re-export 门面
///
/// L6 类型通过此模块访问, 避免散布 `use crate::l6_meta::*`。
/// 单一事实源仍在 L6, 此处仅 re-export 保持跨层引用集中可审计。
/// ⚠️ 向上依赖: L5 → L6, 已通过 facade 集中化并记录。
pub mod l6_facade;
