//! Judgment primitive types — the building blocks for typed decisions.

use super::verdict::JudgmentOutput;

/// Input for any judgment primitive
#[derive(Debug, Clone)]
pub struct JudgmentInput {
    /// The claim or text to judge
    pub content: String,
    /// Optional context for judgment
    pub context: Option<String>,
    /// Optional reference material
    pub references: Vec<String>,
}

/// Trait for all judgment primitives
pub trait JudgmentPrimitive: Send + Sync {
    /// Name of this primitive
    fn name(&self) -> &str;
    /// Execute the judgment
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String>;
}

/// Verify: binary fact-check (Verified | Contradicted | Unsupported)
pub struct Verify;

impl JudgmentPrimitive for Verify {
    fn name(&self) -> &str {
        "jev_verify"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        // Placeholder — will be wired to TypeSafe SDK or local model
        Ok(JudgmentOutput {
            result: "unsupported".into(),
            confidence: 0.5,
            reasoning: "Stub verify judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Screen: risk screening (pass | review | block)
pub struct Screen;

impl JudgmentPrimitive for Screen {
    fn name(&self) -> &str {
        "jev_screen"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "pass".into(),
            confidence: 0.8,
            reasoning: "Stub screen judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Classify: multi-class classification
pub struct Classify;

impl JudgmentPrimitive for Classify {
    fn name(&self) -> &str {
        "jev_classify"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "unknown".into(),
            confidence: 0.5,
            reasoning: "Stub classify judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Decide: action recommendation (auto | review | escalate)
pub struct Decide;

impl JudgmentPrimitive for Decide {
    fn name(&self) -> &str {
        "jev_decide"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "review".into(),
            confidence: 0.6,
            reasoning: "Stub decide judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Compare: relation detection (same_fact | contradicts | different_facts)
pub struct Compare;

impl JudgmentPrimitive for Compare {
    fn name(&self) -> &str {
        "jev_compare"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "different_facts".into(),
            confidence: 0.5,
            reasoning: "Stub compare judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Rank: context relevance ranking
pub struct Rank;

impl JudgmentPrimitive for Rank {
    fn name(&self) -> &str {
        "jev_rank"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "0.5".into(),
            confidence: 0.5,
            reasoning: "Stub rank judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Extract: structured extraction
pub struct Extract;

impl JudgmentPrimitive for Extract {
    fn name(&self) -> &str {
        "jev_extract"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "{}".into(),
            confidence: 0.5,
            reasoning: "Stub extract judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Review: quality review
pub struct Review;

impl JudgmentPrimitive for Review {
    fn name(&self) -> &str {
        "jev_review"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "pass".into(),
            confidence: 0.7,
            reasoning: "Stub review judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Find: evidence finding
pub struct Find;

impl JudgmentPrimitive for Find {
    fn name(&self) -> &str {
        "jev_find"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "[]".into(),
            confidence: 0.5,
            reasoning: "Stub find judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Gate: claim verification gate
pub struct Gate;

impl JudgmentPrimitive for Gate {
    fn name(&self) -> &str {
        "jev_gate"
    }
    fn judge(&self, _input: &JudgmentInput) -> Result<JudgmentOutput, String> {
        Ok(JudgmentOutput {
            result: "pass".into(),
            confidence: 0.6,
            reasoning: "Stub gate judgment".into(),
            latency_ms: 0,
        })
    }
}

/// Registry of all available judgment primitives
pub struct JudgmentRegistry {
    primitives: Vec<Box<dyn JudgmentPrimitive>>,
}

impl JudgmentRegistry {
    pub fn new() -> Self {
        let mut primitives: Vec<Box<dyn JudgmentPrimitive>> = Vec::new();
        primitives.push(Box::new(Verify));
        primitives.push(Box::new(Screen));
        primitives.push(Box::new(Classify));
        primitives.push(Box::new(Decide));
        primitives.push(Box::new(Compare));
        primitives.push(Box::new(Rank));
        primitives.push(Box::new(Extract));
        primitives.push(Box::new(Review));
        primitives.push(Box::new(Find));
        primitives.push(Box::new(Gate));
        Self { primitives }
    }

    pub fn get(&self, name: &str) -> Option<&dyn JudgmentPrimitive> {
        self.primitives
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.primitives.iter().map(|p| p.name()).collect()
    }
}
