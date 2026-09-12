//! NT-SHIELD Security Module
//!
//! NeoTrix安全防护模块，包含:
//! - 原有模块 (nt_shield, sandbox, audit, etc.)
//! - 新增防御模块 (Unified Defense Layer)

// ============================================
// 原有模块 (保持兼容)
// ============================================

pub mod core;
pub mod nt_shield_agentic_scan;
pub mod nt_shield_approval;
pub mod nt_shield_audit;
pub mod nt_shield_audit_phases;
pub mod nt_shield_comm;
pub mod nt_shield_oversight;
pub mod nt_shield_propagation_guard;
pub mod nt_shield_recon;
pub mod nt_shield_sandbox;
pub mod content_moderation;

#[cfg(feature = "sandbox")]
pub mod nt_shield_sandbox_entry;

pub mod nt_shield_sentry;

#[cfg(feature = "stealth-net")]
pub mod nt_shield_stealth_net;

pub mod nt_shield_traffic;

// ============================================
// 新增防御模块 (Phase 1)
// ============================================

pub mod unified_defense;
pub mod input_gatekeeper;
pub mod output_sentinel;
pub mod prompt_guardian;
pub mod refusal_tamper;
pub mod guardrail_traversal;
pub mod slang_norm;
pub mod dual_evidence;
pub mod grapple_hooks;
pub mod proxy_detection;
pub mod reasoning_protection;
pub mod anti_distillation;
pub mod fullbreak;
pub mod cloud_evade;
pub mod nt_shield_ztnet;

// ============================================
// Re-exports (保持向后兼容)
// ============================================

// 原有re-exports
pub use nt_shield::context_boundary::{ContextBoundary, ContextRequest, TrustLevel, ValidationResult};

// 新增re-exports
pub use unified_defense::UnifiedDefenseLayer;
pub use input_gatekeeper::InputGatekeeper;
pub use output_sentinel::OutputSentinel;
pub use prompt_guardian::PromptGuardian;
pub use refusal_tamper::RefusalTamperEngine;
pub use guardrail_traversal::GuardrailTraversalEngine;
pub use slang_norm::SlangNormEngine;
pub use dual_evidence::DualEvidenceScanner;
pub use grapple_hooks::GrappleHookChain;
pub use proxy_detection::ProxyDetectionEngine;
pub use reasoning_protection::ReasoningProtectionEngine;
pub use anti_distillation::AntiDistillationEngine;
pub use fullbreak::{AttackResult, AttackSurface, FullbreakEngine};
pub use cloud_evade::{CloudEvadeEngine, EvasionResult, EvasionTechnique, ObfuscationType};
