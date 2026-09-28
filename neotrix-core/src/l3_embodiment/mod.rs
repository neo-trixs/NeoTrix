pub mod nt_shield;
/// L1 Facade — L3 对 L1 共享类型的 re-export 门面
pub mod l1_facade;
pub mod nt_security;
// 从 core/ 迁移
pub mod nt_core_guard_chain;
/// P3: Computer first-class abstraction (absorbed from cumora)
pub mod nt_computer;
/// Cross-OS computer fleet management (integrated from trycua/cua patterns)
pub mod nt_computer_fleet;

// ============================================================================
// Absorbed — 从 neotrix-sim 吸收
// ============================================================================
/// A* Pathfinding — absorbed from neotrix-sim
pub mod nt_astar;
pub mod nt_sandboxed_shell;
/// Sandbox execution guard — migrated from cli::sandbox (decoupled from CLI types)
pub mod nt_sandbox;
/// Shield enforcer — migrated from cli::shield_enforcer
pub mod nt_shield_enforcer;
/// 跨层错误转换（E0.5/T05：L3 From 实现下沉于此，L0 只留枚举）
pub mod error_conversions;
/// EVO-09 近场协作平面（发现注册/通道策略/帧长门纯逻辑）
pub mod nt_near_field;
/// 多源 Dijkstra 流场 + A*（2026-09-28 自 crates/neotrix-abilities 萃取；该 crate 已归档至
/// ~/Downloads/Neo/neotrix-archive/crates/neotrix-abilities/）。
/// 度量与本层 `nt_astar` **已统一为欧氏**（对角 ×√2），裁决理由见 `nt_flow.rs`
/// 文件头：原 Chebyshev 边代价配 octile 启发会高估并返回次优路径。
pub mod nt_flow;
