//! OsintModule trait and shared types for the automation engine.
//!
//! R-P48: Zero external binary dependencies — all resolution is pure Rust.
//! R-P122: Localized maintenance — each module manages its own scope.

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// A single finding produced by an OSINT module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub key: String,
    pub value: String,
    pub source: String,
    pub confidence: f64,
}

impl Finding {
    pub fn new(
        category: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
        source: impl Into<String>,
        confidence: f64,
    ) -> Self {
        Self {
            category: category.into(),
            key: key.into(),
            value: value.into(),
            source: source.into(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Input passed to every module's `execute`.
#[derive(Debug, Clone)]
pub struct ModuleInput {
    pub target: String,
    pub options: HashMap<String, String>,
}

impl ModuleInput {
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            options: HashMap::new(),
        }
    }

    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Output produced by a module's `execute`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleOutput {
    pub findings: Vec<Finding>,
    pub confidence: f64,
    pub duration_ms: u64,
}

impl ModuleOutput {
    pub fn empty() -> Self {
        Self {
            findings: Vec::new(),
            confidence: 0.0,
            duration_ms: 0,
        }
    }

    pub fn merge(&mut self, other: ModuleOutput) {
        self.findings.extend(other.findings);
        self.recalculate_confidence();
    }

    fn recalculate_confidence(&mut self) {
        if self.findings.is_empty() {
            self.confidence = 0.0;
            return;
        }
        self.confidence =
            self.findings.iter().map(|f| f.confidence).sum::<f64>() / self.findings.len() as f64;
    }
}

/// Category of OSINT module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleCategory {
    Dns,
    Whois,
    Network,
    Web,
    Social,
    Custom,
}

impl std::fmt::Display for ModuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dns => write!(f, "dns"),
            Self::Whois => write!(f, "whois"),
            Self::Network => write!(f, "network"),
            Self::Web => write!(f, "web"),
            Self::Social => write!(f, "social"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Core trait every OSINT automation module must implement.
///
/// Modules are invoked by `ReconEngine` and must be `Send + Sync`.
pub trait OsintModule: Send + Sync {
    /// Human-readable module name (e.g. "dns_lookup").
    fn name(&self) -> &'static str;

    /// Module category for grouping and filtering.
    fn category(&self) -> ModuleCategory;

    /// Execute the module against a target, returning findings.
    fn execute(&self, input: &ModuleInput) -> ModuleOutput;
}

/// Wraps any `OsintModule` with timing instrumentation.
pub fn execute_timed<M: OsintModule + ?Sized>(module: &M, input: &ModuleInput) -> ModuleOutput {
    let start = Instant::now();
    let mut output = module.execute(input);
    output.duration_ms = start.elapsed().as_millis() as u64;
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_clamp_confidence() {
        let f = Finding::new("cat", "k", "v", "src", 2.5);
        assert!((f.confidence - 1.0).abs() < f64::EPSILON);
        let f = Finding::new("cat", "k", "v", "src", -0.3);
        assert!((f.confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn module_output_merge() {
        let mut a = ModuleOutput {
            findings: vec![Finding::new("a", "k1", "v1", "m1", 0.8)],
            confidence: 0.8,
            duration_ms: 10,
        };
        let b = ModuleOutput {
            findings: vec![Finding::new("b", "k2", "v2", "m2", 0.6)],
            confidence: 0.6,
            duration_ms: 20,
        };
        a.merge(b);
        assert_eq!(a.findings.len(), 2);
        assert!((a.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn module_input_builder() {
        let input = ModuleInput::new("example.com").with_option("depth", "2");
        assert_eq!(input.target, "example.com");
        assert_eq!(input.options.get("depth").unwrap(), "2");
    }
}
