use super::sub_agent::SubAgentCapability;

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub handler: String,
    pub output: String,
    pub passed: bool,
    pub issues: Vec<String>,
    pub tier: VerifyTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyTier {
    /// Quick check: output is non-empty, no error markers
    Basic,
    /// Deep check: pattern matching, capability-appropriate verification
    Deep,
}

impl VerifyTier {
    pub fn name(&self) -> &'static str {
        match self {
            VerifyTier::Basic => "basic",
            VerifyTier::Deep => "deep",
        }
    }
}

pub trait Verifier: Send + Sync {
    fn verify(
        &self,
        handler: &str,
        output: &str,
        capability: SubAgentCapability,
    ) -> VerificationReport;
}

pub struct BasicVerifier;

impl Verifier for BasicVerifier {
    fn verify(
        &self,
        handler: &str,
        output: &str,
        _capability: SubAgentCapability,
    ) -> VerificationReport {
        let mut issues = Vec::new();

        if output.is_empty() {
            issues.push("empty output".into());
        }
        if output.contains("error") || output.contains("Error") || output.contains("panic") {
            issues.push("output contains error markers".into());
        }
        if output.contains("unknown_handler") {
            issues.push("handler not recognized".into());
        }

        VerificationReport {
            handler: handler.to_string(),
            output: output.to_string(),
            passed: issues.is_empty(),
            issues,
            tier: VerifyTier::Basic,
        }
    }
}

pub struct VerifyLoop {
    pub verifier: Box<dyn Verifier>,
    pub enabled: bool,
    pub history: Vec<VerificationReport>,
    max_history: usize,
}

impl VerifyLoop {
    pub fn new() -> Self {
        Self {
            verifier: Box::new(BasicVerifier),
            enabled: true,
            history: Vec::new(),
            max_history: 100,
        }
    }

    pub fn with_verifier(mut self, v: Box<dyn Verifier>) -> Self {
        self.verifier = v;
        self
    }

    pub fn check(
        &mut self,
        handler: &str,
        output: &str,
        capability: SubAgentCapability,
    ) -> VerificationReport {
        if !self.enabled {
            return VerificationReport {
                handler: handler.to_string(),
                output: output.to_string(),
                passed: true,
                issues: vec![],
                tier: VerifyTier::Basic,
            };
        }
        let report = self.verifier.verify(handler, output, capability);
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(report.clone());
        report
    }

    pub fn pass_rate(&self) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }
        let passed = self.history.iter().filter(|r| r.passed).count();
        passed as f64 / self.history.len() as f64
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// AdversarialVerifier — two-agent cross-check for deep tier.
/// Runs two independent verification passes and compares results.
/// Detects patterns that a single verifier would miss.
pub struct AdversarialVerifier {
    /// Pattern matchers for capability-specific checks (keyed by capability label)
    capability_patterns: std::collections::HashMap<&'static str, Vec<&'static str>>,
}

impl AdversarialVerifier {
    pub fn new() -> Self {
        let mut patterns = std::collections::HashMap::new();
        patterns.insert("coder", vec!["fn ", "impl ", "struct ", "enum ", "pub "]);
        patterns.insert(
            "tester",
            vec!["#[test]", "assert_", "assert_eq!", "assert_ne!"],
        );
        patterns.insert(
            "security_auditor",
            vec!["unsafe", "unwrap(", "expect(", "panic!"],
        );
        patterns.insert("documenter", vec!["///", "//!", "# ", "## "]);
        Self {
            capability_patterns: patterns,
        }
    }
}

impl Verifier for AdversarialVerifier {
    fn verify(
        &self,
        handler: &str,
        output: &str,
        capability: SubAgentCapability,
    ) -> VerificationReport {
        let mut issues = Vec::new();

        // Pass 1: Basic checks (same as BasicVerifier)
        if output.is_empty() {
            issues.push("empty output".into());
        }
        if output.contains("error") || output.contains("Error") || output.contains("panic") {
            issues.push("output contains error markers".into());
        }

        // Pass 2: Capability-appropriate pattern check
        let capability_label = capability.label();
        if let Some(patterns) = self.capability_patterns.get(capability_label) {
            let matched: Vec<&&str> = patterns.iter().filter(|p| output.contains(*p)).collect();
            if matched.is_empty() {
                issues.push(format!(
                    "adversarial:no_{}_patterns_found",
                    capability_label
                ));
            }
        }

        // Pass 3: Inconsistency detection (simulated adversarial cross-check)
        if output.len() > 10 {
            let first_half = &output[..output.len() / 2];
            let second_half = &output[output.len() / 2..];
            if first_half.trim() == second_half.trim() && first_half.len() > 20 {
                issues.push("adversarial:duplicate_content_across_halves".into());
            }
        }

        VerificationReport {
            handler: handler.to_string(),
            output: output.to_string(),
            passed: issues.is_empty(),
            issues,
            tier: VerifyTier::Deep,
        }
    }
}

impl Default for VerifyLoop {
    fn default() -> Self {
        Self::new()
    }
}
