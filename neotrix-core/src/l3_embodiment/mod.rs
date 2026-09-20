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
