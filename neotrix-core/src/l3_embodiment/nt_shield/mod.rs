//! NT-SHIELD Security Module
//!
//! NeoTrix安全防护模块，包含:
//! - 统一防御层 (Unified Defense Layer)
//! - 输入验证 (Input Gatekeeper)
//! - 输出验证 (Output Sentinel)
//! - 提示守护 (Prompt Guardian)
//! - 拒答篡改 (Refusal Tamper)
//! - 护栏穿越 (Guardrail Traversal)
//! - 黑话规范化 (Slang Norm)
//! - 双证据扫描 (Dual Evidence)
//! - 钩链锁存 (Grapple Hooks)
//! - 代理检测 (Proxy Detection)
//! - 推理保护 (Reasoning Protection)
//! - 反分馏 (Anti-Distillation)

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
