//! Agent Guardrails — OpenAI-Agents inspired input/output validation for NT-SHIELD
//!
//! R-P132: Guardrails are non-optional in production.
//! R-SEC07: Guardrail decouple-or-judge — per-category false-positive overrides.
//! R-P129: HITL gate for high-stakes decisions.
//!
//! Architecture:
//! - `GuardrailPolicy`: pluggable JSON-configurable compliance policy
//! - `InputValidator` / `OutputValidator`: trait-based validation pipeline
//! - `PolicyEngine`: orchestrates validators, manages severity, HITL routing
//! - ViolationSeverity: Block | Warn | Log (determines enforcement action)

pub mod input_validator;
pub mod output_validator;
pub mod policy_engine;

pub use input_validator::{InputValidator, InputValidationResult, InputViolation};
pub use output_validator::{OutputValidator, OutputValidationResult, OutputViolation};
pub use policy_engine::{PolicyEngine, PolicyConfig, ViolationSeverity, GuardrailVerdict, HitlRequest};

use serde::{Deserialize, Serialize};

/// Guardrail violation — unified type returned by all validators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailViolation {
    pub rule_id: String,
    pub category: GuardrailCategory,
    pub severity: ViolationSeverity,
    pub message: String,
    pub details: Option<String>,
    pub confidence: f64,
}

/// Guardrail categories — maps to compliance domains.
// ⛔ **不能 derive Copy**：本枚举含 `Custom(String)` 变体（`mod.rs:44`），
//   `String: Copy` 不成立 ⇒ E0277。这是本目录「从未编译」的第一处硬错误。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardrailCategory {
    PromptInjection,
    ToolAbuse,
    CredentialLeak,
    Hallucination,
    UnsafeCode,
    DataExfiltration,
    PolicyViolation,
    Custom(String),
}

/// Validation context — passed to validators for decision-making.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GuardrailContext {
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub risk_level: RiskLevel,
    pub metadata: std::collections::HashMap<String, String>,
}

/// Risk level for HITL gating (R-P129).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Guardrail pipeline result — wraps validation output with verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailResult<T> {
    pub passed: bool,
    pub verdict: GuardrailVerdict,
    pub output: Option<T>,
    pub violations: Vec<GuardrailViolation>,
    pub hitl_request: Option<HitlRequest>,
}

/// Default policy config — production-safe defaults (R-P132).
impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            max_input_length: 100_000,
            max_output_length: 500_000,
            blocked_input_patterns: vec![],
            blocked_output_patterns: vec![],
            credential_patterns: vec![
                r"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token)\s*[:=]\s*\S+".to_string(),
                r"(?i)(password|passwd|pwd)\s*[:=]\s*\S+".to_string(),
                r"(?i)-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----".to_string(),
            ],
            injection_patterns: vec![
                r"(?i)ignore\s+(all\s+)?previous\s+instructions".to_string(),
                r"(?i)you\s+are\s+now\s+a\s+".to_string(),
                r"(?i)disregard\s+(all\s+)?prior".to_string(),
                r"(?i)system\s*:\s*".to_string(),
                r"(?i)<\|im_start\|>".to_string(),
                r"(?i)forget\s+everything".to_string(),
            ],
            exfil_patterns: vec![
                r"(?i)(curl|wget)\s+.*https?://".to_string(),
                r"(?i)base64\s+(encode|decode)".to_string(),
                r"(?i)nc\s+-[elp]\s+".to_string(),
            ],
            false_positive_overrides: std::collections::HashMap::new(),
            hitl_threshold: RiskLevel::High,
            enabled: true,
        }
    }
}
