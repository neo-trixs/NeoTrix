//! Core verdict types for judgment outcomes.

/// Binary verification result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Verified {
    Verified,
    Contradicted,
    Unsupported,
}

/// Relation between two claims
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Relation {
    SameFact,
    Contradicts,
    DifferentFacts,
}

/// Action recommendation from judgment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Action {
    Auto,
    Review,
    Escalate,
}

/// Pass/Review/Block gate result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GateResult {
    Pass,
    Review,
    Block,
    Skip,
}

/// A calibrated judgment output
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JudgmentOutput {
    /// The verdict or classification result
    pub result: String,
    /// Calibrated confidence [0.0, 1.0]
    pub confidence: f64,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Latency in milliseconds
    pub latency_ms: u64,
}
