//! L1 Facade — observer/health/self-test 子门面（由 `l1_facade.rs` 拆分）
//!
//! 领域：观察 / 健康 / 自测类 L6 具体类型再导出。
//! 单一事实源仍在 L6，此处仅 re-export；`l1_facade.rs` 以
//! `pub use self::l1_facade_observer::*;` 回导，保持外部 `l1_facade::Xxx` 路径零断裂。

// ─── L6 observer/gold-standard types ────────────────────────────────────────
pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::ConsciousnessGoldStandard; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::E8HexagramState; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::healing::nt_mind_consciousness_gold_standard::{
    // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
    DEFAULT_COHERENCE_THRESHOLD,
    DEFAULT_PHI_THRESHOLD,
};
pub use crate::l6_meta::healing::nt_mind_consciousness_monitor::ConsciousnessMonitor; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_observer::OneObserver; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_observer_error::ObserverErrorRecovery; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 healing/self-test types ─────────────────────────────────────────────
pub use crate::l6_meta::healing::nt_core_self_test::ExternalVerifier; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::nt_core_self_review::SelfReviewGate; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 coordination quality/watchdog/verifier (健康/自测侧) ─────────────────
// 注：同 block 内 `self_improvement` / `sentrux` 属进化侧，已归 `l1_facade_meta`。
pub use crate::l6_meta::coordination::layered_qa::_LayeredQA; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::coordination::nt_meta_build_watchdog::{BuildWatchdog, WatchdogConfig}; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::coordination::quality_control::_QualityControlPipeline; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::coordination::quality_gate::QualityGate; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::coordination::template_tag_registry::_TemplateTagRegistry; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::coordination::verifier_agent::_VerifierAgent; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）

// ─── L6 memory meta-observer (观察侧) ───────────────────────────────────────
// 注：同 block 内 `evolution_harness` / `transcendent_loop::LoopConfig` 属进化侧，已归 `l1_facade_meta`。
pub use crate::l6_meta::memory::meta_observer::MetaObserverSelfTest; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
pub use crate::l6_meta::memory::meta_observer::{MetaObserver, MetaObserverConfig}; // ALLOW: 观察/健康/自测 字段直访（trait-object 不可行，见 l1_facade.rs 125-128 注释）
