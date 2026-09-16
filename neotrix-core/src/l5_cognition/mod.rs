pub mod traits;
pub mod nt_core;
pub mod nt_mind;
/// Goal management — cognitive/RL modules (migrated from L1 nt_act_goal)
pub mod nt_goal;
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
/// L2 Facade — L5 对 L2 感知层类型的集中 re-export 门面
///
/// L2 类型通过此模块访问, 避免散布 `use crate::l2_perception::*`。
/// 单一事实源仍在 L2, 此处仅 re-export 保持跨层引用集中可审计。
/// ⚠️ 向下依赖: L5 → L2, 已通过 facade 集中化并记录。
pub mod l2_facade;
