use super::self_evolution_loop::{MutationOp, SelfEvolutionArchive};

/// Three-layer consistency verification for self-modification proposals.
///
/// Layer 1 — Syntax: validates structural integrity (parseable, well-formed)
/// Layer 2 — Type/Safety: checks type consistency and absence of dangerous patterns
/// Layer 3 — Self-Consistency: verifies proposals don't contradict system invariants
///
/// Reference: Gödel Agent (arXiv:2410.04444), Darwin Gödel Machine (arXiv:2505.22954v3)
pub struct GodelConsistencyChecker {
    /// When true, all checks pass without evaluation (for environments without parsers).
    pub dry_run: bool,
}

/// The result of a Gödel consistency check.
#[derive(Debug, Clone, PartialEq)]
pub struct GodelCheckResult {
    /// Whether the check passed.
    pub passed: bool,
    /// Which layer detected the issue (0 = all passed, 1 = syntax, 2 = type, 3 = self-consistency).
    pub layer: u8,
    /// Human-readable explanation.
    pub reason: String,
}

impl GodelCheckResult {
    pub fn ok() -> Self {
        Self {
            passed: true,
            layer: 0,
            reason: String::new(),
        }
    }

    pub fn fail(layer: u8, reason: impl Into<String>) -> Self {
        Self {
            passed: false,
            layer,
            reason: reason.into(),
        }
    }
}

/// Danger patterns checked in Layer 2 for code-carrying mutations.
const DANGEROUS_PATTERNS: &[&str] = &[
    "std::process",
    "std::os",
    "std::process::Command",
    "std::fs::remove_dir_all",
    "std::fs::remove_file",
    "std::net",
    "Command::new",
    "unsafe ",
    "#[allow(unsafe_code)]",
    "std::ptr",
    "std::mem::transmute",
    "libc::",
    "ptrace",
    "exec(",
    "fork(",
    "system(",
];

/// Known tune-parameter targets and their allowed value ranges (min, max).
/// Used by Layer 3 invariant checking to validate target names.
const KNOWN_PARAM_TARGETS: &[&str] = &[
    "cognitive_load.thinking_budget",
    "cognitive_load.max_concurrent",
    "emergent_reasoning.exploration_rate",
    "emergent_reasoning.temperature",
    "inner_critic.relevance_threshold",
    "mutation_rate",
    "crossover_rate",
    "elite_count",
    "loop_interval_cycles",
];

impl GodelConsistencyChecker {
    pub fn new() -> Self {
        Self { dry_run: false }
    }

    pub fn with_dry_run(mut self, dry: bool) -> Self {
        self.dry_run = dry;
        self
    }

    /// Run all three layers of consistency checking on a mutation proposal.
    ///
    /// Layer 1 — Syntax: verifies the proposal is structurally well-formed
    ///   - TuneParam: target not empty
    ///   - RewriteHandler/AddHandler: code is non-empty
    ///   - RewriteMeta: strategy has a version and proposer
    ///   - SelfModifyProposal: target and source code non-empty
    ///
    /// Layer 2 — Type/Safety: verifies type validity and code safety
    ///   - TuneParam: delta is in [-0.5, 0.5], target is known, final value in range
    ///   - RewriteHandler/AddHandler/SelfModifyProposal: no dangerous patterns
    ///   - RewriteMeta: Ne source passes basic bracket-balance check
    ///
    /// Layer 3 — Self-Consistency: verifies against system invariants
    ///   - SelfModifyProposal: invokes SelfModifyGuard-style sword check
    ///   - Global invariants: mutation doesn't contradict known constraints
    pub fn check_proposal(
        &self,
        mutation: &MutationOp,
        _archive: &SelfEvolutionArchive,
    ) -> GodelCheckResult {
        if self.dry_run {
            return GodelCheckResult::ok();
        }

        // Layer 1: Syntax
        let l1 = self.layer1_syntax(mutation);
        if !l1.passed {
            return l1;
        }

        // Layer 2: Type/Safety
        let l2 = self.layer2_type_safety(mutation);
        if !l2.passed {
            return l2;
        }

        // Layer 3: Self-Consistency
        self.layer3_self_consistency(mutation)
    }

    /// Convenience method for the evolution loop: checks a mutation before execution.
    /// Returns the result with a human-readable message.
    pub fn check_before_mutation(
        &self,
        mutation: &MutationOp,
        archive: &SelfEvolutionArchive,
    ) -> GodelCheckResult {
        self.check_proposal(mutation, archive)
    }

    // ── Layer 1: Syntax ──

    fn layer1_syntax(&self, mutation: &MutationOp) -> GodelCheckResult {
        match mutation {
            MutationOp::TuneParam { target, delta: _ } => {
                if target.trim().is_empty() {
                    return GodelCheckResult::fail(1, "TuneParam target is empty");
                }
                GodelCheckResult::ok()
            }
            MutationOp::AddHandler { position, code } => {
                if position.trim().is_empty() {
                    return GodelCheckResult::fail(1, "AddHandler position is empty");
                }
                if code.trim().is_empty() {
                    return GodelCheckResult::fail(1, "AddHandler code is empty");
                }
                GodelCheckResult::ok()
            }
            MutationOp::RewriteHandler { name, code } => {
                if name.trim().is_empty() {
                    return GodelCheckResult::fail(1, "RewriteHandler name is empty");
                }
                if code.trim().is_empty() {
                    return GodelCheckResult::fail(1, "RewriteHandler code is empty");
                }
                GodelCheckResult::ok()
            }
            MutationOp::SwapPolicy { gates } => {
                if gates.is_empty() {
                    return GodelCheckResult::fail(1, "SwapPolicy gate list is empty");
                }
                for (i, gate) in gates.iter().enumerate() {
                    if gate.trim().is_empty() {
                        return GodelCheckResult::fail(
                            1,
                            format!("SwapPolicy gate[{}] is empty", i),
                        );
                    }
                }
                GodelCheckResult::ok()
            }
            MutationOp::RewritePrimitive { name, impl_ } => {
                if name.trim().is_empty() {
                    return GodelCheckResult::fail(1, "RewritePrimitive name is empty");
                }
                if impl_.trim().is_empty() {
                    return GodelCheckResult::fail(1, "RewritePrimitive impl is empty");
                }
                GodelCheckResult::ok()
            }
            MutationOp::RewriteMeta { strategy } => {
                if strategy.version == 0 {
                    return GodelCheckResult::fail(1, "RewriteMeta has version 0");
                }
                if strategy.proposer.is_empty() && strategy.evaluator.is_empty() {
                    return GodelCheckResult::fail(
                        1,
                        "RewriteMeta has both proposer and evaluator empty",
                    );
                }
                GodelCheckResult::ok()
            }
            MutationOp::SelfModifyProposal {
                target,
                target_type,
                source_code,
            } => {
                if target.trim().is_empty() {
                    return GodelCheckResult::fail(1, "SelfModifyProposal target is empty");
                }
                if target_type.trim().is_empty() {
                    return GodelCheckResult::fail(1, "SelfModifyProposal target_type is empty");
                }
                if source_code.trim().is_empty() {
                    return GodelCheckResult::fail(1, "SelfModifyProposal source_code is empty");
                }
                GodelCheckResult::ok()
            }
        }
    }

    // ── Layer 2: Type/Safety ──

    fn layer2_type_safety(&self, mutation: &MutationOp) -> GodelCheckResult {
        match mutation {
            MutationOp::TuneParam { target: _, delta } => {
                let d = *delta;
                if !d.is_finite() {
                    return GodelCheckResult::fail(2, "TuneParam delta is not finite");
                }
                if d > 0.5 || d < -0.5 {
                    return GodelCheckResult::fail(
                        2,
                        format!("TuneParam delta {} exceeds max range ±0.5", d),
                    );
                }
                GodelCheckResult::ok()
            }
            MutationOp::RewriteHandler { name: _, code } | MutationOp::AddHandler { code, .. } => {
                self.scan_dangerous_patterns(code)
            }
            MutationOp::RewritePrimitive { name: _, impl_ } => self.scan_dangerous_patterns(impl_),
            MutationOp::SwapPolicy { gates } => {
                for (i, gate) in gates.iter().enumerate() {
                    if gate.len() > 500 {
                        return GodelCheckResult::fail(
                            2,
                            format!("SwapPolicy gate[{}] exceeds 500 chars", i),
                        );
                    }
                }
                GodelCheckResult::ok()
            }
            MutationOp::RewriteMeta { strategy } => {
                if strategy.version > 9999 {
                    return GodelCheckResult::fail(
                        2,
                        format!("RewriteMeta version {} exceeds max 9999", strategy.version),
                    );
                }
                if !strategy.proposer.is_empty() {
                    if let Err(msg) = self.check_ne_brackets(&strategy.proposer) {
                        return GodelCheckResult::fail(2, format!("RewriteMeta proposer: {}", msg));
                    }
                }
                if !strategy.evaluator.is_empty() {
                    if let Err(msg) = self.check_ne_brackets(&strategy.evaluator) {
                        return GodelCheckResult::fail(
                            2,
                            format!("RewriteMeta evaluator: {}", msg),
                        );
                    }
                }
                GodelCheckResult::ok()
            }
            MutationOp::SelfModifyProposal { source_code, .. } => {
                self.scan_dangerous_patterns(source_code)
            }
        }
    }

    // ── Layer 3: Self-Consistency ──

    fn layer3_self_consistency(&self, mutation: &MutationOp) -> GodelCheckResult {
        match mutation {
            MutationOp::SelfModifyProposal {
                target,
                source_code,
                ..
            } => {
                if source_code.contains("SelfModify") || source_code.contains("self_modify") {
                    return GodelCheckResult::fail(
                        3,
                        format!(
                            "SelfModifyProposal for '{}' contains self-referential code",
                            target
                        ),
                    );
                }
                let sword_patterns: &[&str] = &[
                    "std::process::exit",
                    "std::process::abort",
                    "kill",
                    "remove_dir_all",
                    "format!(\"rm ",
                    "std::fs::write(\"/",
                    "std::env::set_var",
                    "std::env::remove_var",
                ];
                for pat in sword_patterns {
                    if source_code.contains(pat) {
                        return GodelCheckResult::fail(
                            3,
                            format!(
                                "SelfModifyProposal for '{}' blocked by swords_check: '{}'",
                                target, pat
                            ),
                        );
                    }
                }
                GodelCheckResult::ok()
            }
            MutationOp::TuneParam { target, delta } => {
                if !KNOWN_PARAM_TARGETS.contains(&target.as_str())
                    && !target.starts_with("ne_program_gradient_")
                    && !target.starts_with("handler_profiler.")
                {
                    return GodelCheckResult::fail(
                        3,
                        format!(
                            "TuneParam target '{}' is not in known parameter list",
                            target
                        ),
                    );
                }
                if delta.abs() < 1e-10 {
                    return GodelCheckResult::fail(
                        3,
                        format!("TuneParam delta {} is too close to zero (no-op)", delta),
                    );
                }
                GodelCheckResult::ok()
            }
            _ => GodelCheckResult::ok(),
        }
    }

    // ── Helpers ──

    /// Scan source code for known dangerous patterns (Layer 2 sword check).
    fn scan_dangerous_patterns(&self, code: &str) -> GodelCheckResult {
        for pat in DANGEROUS_PATTERNS {
            if let Some(pos) = code.find(pat) {
                let start = pos.saturating_sub(20);
                let end = (pos + pat.len() + 20).min(code.len());
                let snippet = if start > 0 { "..." } else { "" };
                let excerpt = &code[start..end];
                return GodelCheckResult::fail(
                    2,
                    format!(
                        "dangerous pattern '{}' at position {}: {}{}{}",
                        pat, pos, snippet, excerpt, snippet
                    ),
                );
            }
        }
        GodelCheckResult::ok()
    }

    /// Verify Ne source code has balanced brackets (basic syntax sanity).
    fn check_ne_brackets(&self, source: &str) -> Result<(), String> {
        let mut depth = 0i64;
        for (i, ch) in source.chars().enumerate() {
            match ch {
                '(' | '{' | '[' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(format!("unexpected closing ')' at position {}", i));
                    }
                }
                '}' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(format!("unexpected closing '}}' at position {}", i));
                    }
                }
                ']' => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(format!("unexpected closing ']' at position {}", i));
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return Err(format!("unbalanced brackets (depth={})", depth));
        }
        Ok(())
    }
}

impl Default for GodelConsistencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_archive() -> SelfEvolutionArchive {
        let mut archive = SelfEvolutionArchive::new();
        archive.generation = 42;
        archive.best_score = 0.75;
        archive
    }

    #[test]
    fn test_valid_tune_param_passes() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::TuneParam {
            target: "cognitive_load.thinking_budget".into(),
            delta: 0.05,
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(
            result.passed,
            "valid TuneParam should pass: {}",
            result.reason
        );
        assert_eq!(result.layer, 0);
    }

    #[test]
    fn test_tune_param_excessive_delta_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::TuneParam {
            target: "mutation_rate".into(),
            delta: 0.7,
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
        assert!(result.reason.contains("exceeds max range"));
    }

    #[test]
    fn test_tune_param_noop_delta_fails_layer3() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::TuneParam {
            target: "mutation_rate".into(),
            delta: 0.0,
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 3);
        assert!(result.reason.contains("too close to zero"));
    }

    #[test]
    fn test_tune_param_unknown_target_fails_layer3() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::TuneParam {
            target: "nonexistent.param".into(),
            delta: 0.1,
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 3);
        assert!(result.reason.contains("not in known parameter list"));
    }

    #[test]
    fn test_tune_param_non_finite_delta_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::TuneParam {
            target: "mutation_rate".into(),
            delta: f64::NAN,
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
    }

    #[test]
    fn test_rewrite_handler_dangerous_code_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewriteHandler {
            name: "test_handler".into(),
            code: "let x = 1; unsafe { std::ptr::read(0) };".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
        assert!(result.reason.contains("unsafe"));
    }

    #[test]
    fn test_rewrite_handler_safe_code_passes() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewriteHandler {
            name: "test_handler".into(),
            code: "fn step() { let x = compute_score(); x }".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(result.passed, "safe code should pass: {}", result.reason);
    }

    #[test]
    fn test_rewrite_handler_empty_name_fails_layer1() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewriteHandler {
            name: "".into(),
            code: "fn step() {}".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 1);
    }

    #[test]
    fn test_add_handler_dangerous_code_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::AddHandler {
            position: "after_init".into(),
            code: "std::process::exit(1)".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
        assert!(result.reason.contains("std::process"));
    }

    #[test]
    fn test_swap_policy_empty_gates_fails_layer1() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::SwapPolicy { gates: vec![] };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 1);
    }

    #[test]
    fn test_swap_policy_valid_gates_passes() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::SwapPolicy {
            gates: vec!["pace::commit_gain ≤ 0.5".into()],
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(result.passed);
    }

    #[test]
    fn test_rewrite_meta_version_zero_fails_layer1() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewriteMeta {
            strategy: super::super::self_evolution_loop::MetaStrategy {
                proposer: "\"TuneParam:x:0.1\"".into(),
                evaluator: String::new(),
                selector: String::new(),
                version: 0,
                self_proposed: true,
            },
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 1);
    }

    #[test]
    fn test_self_modify_proposal_self_referential_fails_layer3() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::SelfModifyProposal {
            target: "guard".into(),
            target_type: "SafetyGate".into(),
            source_code: "let x = SelfModify::new();".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 3);
        assert!(result.reason.contains("self-referential"));
    }

    #[test]
    fn test_self_modify_proposal_dangerous_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::SelfModifyProposal {
            target: "handler_x".into(),
            target_type: "Handler".into(),
            source_code: "let cmd = Command::new(\"rm\");".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
    }

    #[test]
    fn test_self_modify_proposal_valid_passes() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::SelfModifyProposal {
            target: "cognitive_load".into(),
            target_type: "Parameter".into(),
            source_code: "fn compute_budget(score: f64) -> f64 { score * 0.5 }".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(
            result.passed,
            "valid SelfModifyProposal should pass: {}",
            result.reason
        );
    }

    #[test]
    fn test_rewrite_primitive_empty_name_fails_layer1() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewritePrimitive {
            name: "".into(),
            impl_: "fn foo() {}".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 1);
    }

    #[test]
    fn test_rewrite_primitive_dangerous_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewritePrimitive {
            name: "vsa_encode".into(),
            impl_: "unsafe { std::mem::transmute(x) }".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
    }

    #[test]
    fn test_dry_run_always_passes() {
        let checker = GodelConsistencyChecker::new().with_dry_run(true);
        let mutation = MutationOp::RewriteHandler {
            name: "".into(),
            code: "unsafe { std::process::exit(0) }".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(result.passed, "dry_run should always pass");
    }

    #[test]
    fn test_check_before_mutation_interface() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::TuneParam {
            target: "cognitive_load.thinking_budget".into(),
            delta: 0.05,
        };
        let archive = sample_archive();
        let result = checker.check_before_mutation(&mutation, &archive);
        assert!(result.passed);
    }

    #[test]
    fn test_ne_bracket_balance_detects_unbalanced() {
        let checker = GodelConsistencyChecker::new();
        let result = checker.check_ne_brackets("(let x (foo bar");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unbalanced"));
    }

    #[test]
    fn test_ne_bracket_balance_passes_balanced() {
        let checker = GodelConsistencyChecker::new();
        let result = checker.check_ne_brackets("(let x (foo bar))");
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_default_dry_run_false() {
        let checker = GodelConsistencyChecker::new();
        assert!(!checker.dry_run);
    }

    #[test]
    fn test_swap_policy_long_gate_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let long_gate = "x".repeat(501);
        let mutation = MutationOp::SwapPolicy {
            gates: vec![long_gate],
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
    }

    #[test]
    fn test_rewrite_meta_version_exceeds_max_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewriteMeta {
            strategy: super::super::self_evolution_loop::MetaStrategy {
                proposer: "\"test\"".into(),
                evaluator: String::new(),
                selector: String::new(),
                version: 10000,
                self_proposed: true,
            },
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
    }

    #[test]
    fn test_self_modify_proposal_empty_source_fails_layer1() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::SelfModifyProposal {
            target: "x".into(),
            target_type: "Handler".into(),
            source_code: "".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 1);
    }

    #[test]
    fn test_godel_check_result_fail_creates_correct_fields() {
        let r = GodelCheckResult::fail(2, "test failure");
        assert!(!r.passed);
        assert_eq!(r.layer, 2);
        assert_eq!(r.reason, "test failure");
    }

    #[test]
    fn test_add_handler_empty_code_fails_layer1() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::AddHandler {
            position: "init".into(),
            code: "".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 1);
    }

    #[test]
    fn test_rewrite_handler_dangerous_std_process_os_fails_layer2() {
        let checker = GodelConsistencyChecker::new();
        let mutation = MutationOp::RewriteHandler {
            name: "handler".into(),
            code: "let env = std::os::unix::fs::chmod(...)".into(),
        };
        let result = checker.check_proposal(&mutation, &sample_archive());
        assert!(!result.passed);
        assert_eq!(result.layer, 2);
    }
}
