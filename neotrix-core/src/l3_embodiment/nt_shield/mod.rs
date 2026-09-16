//! NT-SHIELD Security Module

// Core modules (renamed from 'core' to avoid shadowing std::core)
pub mod shield_core;
// pub mod content_moderation; // DEAD: zero external references
pub mod circuit_breaker;
pub mod dual_evidence;
pub mod proxy_detection;
pub mod slang_norm;
pub mod shield_capability;

// Shield implementation modules
// pub mod nt_shield_adversarial; // DEAD: zero external references
pub mod nt_shield_agentic_scan;
// pub mod nt_shield_approval; // DEAD: zero external references
pub mod nt_shield_audit;
// pub mod nt_shield_audit_phases; // DEAD: zero external references
pub mod nt_shield_comm;
pub mod nt_shield_internal_scan;
// pub mod nt_shield_cleanup; // DEAD: zero external references
pub mod nt_shield_impl;
pub mod http_intercept;
pub mod osint;
pub mod nt_shield_oversight;
pub mod nt_shield_propagation_guard;
// pub mod nt_shield_recon; // DEAD: zero external references
pub mod nt_shield_sandbox;
pub mod nt_shield_sandbox_entry;
pub mod nt_shield_sentry;
// pub mod nt_shield_threat_detection; // DEAD: zero external references
pub mod nt_shield_traffic;
pub mod nt_shield_ztnet;
pub mod binary_analyzer;

#[cfg(feature = "stealth-net")]
pub mod nt_shield_stealth_net;

// Security analysis modules
pub mod mitigation_auditor;
pub mod sink_analyzer;
pub mod vulnerability_pipeline;

// Defense subdirectories
pub mod defense;
pub mod guard;
pub mod evasion;
pub mod safety;

// Re-exports from shield_core
pub use shield_core::context_boundary::{ContextBoundary, ContextRequest, TrustLevel, ValidationResult};
pub use shield_core::guard_chain;

// Defense re-exports
pub use defense::unified_defense::UnifiedDefenseLayer;
pub use defense::guardrail_traversal::GuardrailTraversalEngine;
pub use defense::reasoning_protection::ReasoningProtectionEngine;
pub use defense::refusal_tamper::RefusalTamperEngine;
pub use defense::anti_distillation::AntiDistillationEngine;
pub use guard::input_gatekeeper::InputGatekeeper;
pub use guard::output_sentinel::OutputSentinel;
pub use guard::prompt_guardian::PromptGuardian;
pub use evasion::grapple_hooks::GrappleHookChain;
pub use evasion::fullbreak::FullbreakEngine;
pub use evasion::cloud_evade::CloudEvadeEngine;
pub use binary_analyzer::BinaryAnalyzer;
