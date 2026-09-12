
// ============================================
// Defense modules (subdirectories)
// ============================================
pub mod defense;
pub mod guard;
pub mod evasion;

// Re-exports for backward compatibility
pub use defense::unified_defense::UnifiedDefenseLayer;
pub use defense::guardrail_traversal::GuardrailTraversalEngine;
pub use defense::reasoning_protection::ReasoningProtectionEngine;
pub use defense::refusal_tamper::RefusalTamperEngine;
pub use defense::anti_distillation::AntiDistillation;
pub use guard::input_gatekeeper::InputGatekeeper;
pub use guard::output_sentinel::OutputSentinel;
pub use guard::prompt_guardian::PromptGuardian;
pub use evasion::grapple_hooks::GrappleHookChain;
pub use evasion::fullbreak::FullBreak;
pub use evasion::cloud_evade::CloudEvade;
