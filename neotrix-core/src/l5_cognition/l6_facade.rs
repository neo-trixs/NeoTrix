//! L6 Facade — L5 对 L6 元认知层类型的集中 re-export 门面
//!
//! L5 认知层通过此模块访问 L6 元认知层提供的具体类型,
//! 避免散布 `use crate::l6_meta::*` 造成分散的向上依赖。
//! 单一事实源仍在 L6, 此处仅 re-export 保持跨层引用集中可审计。
//!
//! ⚠️ 向上依赖: L5 → L6, 已通过 facade 集中化。
//! 对于仅需接口而非实现的场景, 优先使用 `crate::l5_cognition::traits` 中的 trait 抽象。

// ── nt_repair (healing/) ──
pub use crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard;
pub use crate::l6_meta::nt_repair::nt_mind_consciousness_monitor::ConsciousnessMonitor;
pub use crate::l6_meta::nt_repair::nt_mind_eval_harness::EvalHarness;

// ── memory/ ──
pub use crate::l6_meta::memory::evolution_harness::EvolutionHarness;
pub use crate::l6_meta::memory::transcendent_loop::LoopConfig;

// ── coordination/ ──
pub use crate::l6_meta::coordination::self_improvement::SystemMetrics;
