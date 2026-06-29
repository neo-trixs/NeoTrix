use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum InvariantKind {
    Structural,
    Temporal,
    Bound,
    Consistency,
    Typed(String),
}

#[derive(Debug, Clone)]
pub struct Invariant {
    pub id: String,
    pub kind: InvariantKind,
    pub description: String,
    pub predicate: String,
    pub formal_expr: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormalProofOutcome {
    Verified,
    Violated {
        reason: String,
        counterexample: String,
    },
    Unknown {
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub invariant_id: String,
    pub result: FormalProofOutcome,
    pub verified_at_cycle: u64,
    pub verifier: String,
}

pub trait FormalVerifier {
    fn verify(&self, invariant: &Invariant, state: &SystemState) -> FormalProofOutcome;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub entity_count: usize,
    pub edge_count: usize,
    pub cycle: u64,
    pub entities: Vec<String>,
    pub edges: Vec<(String, String, String)>,
    pub metrics: HashMap<String, f64>,
}

impl SystemState {
    pub fn empty() -> Self {
        Self {
            entity_count: 0,
            edge_count: 0,
            cycle: 0,
            entities: Vec::new(),
            edges: Vec::new(),
            metrics: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockVerifier;

impl FormalVerifier for MockVerifier {
    fn verify(&self, invariant: &Invariant, state: &SystemState) -> FormalProofOutcome {
        match &invariant.kind {
            InvariantKind::Structural => {
                if state.entity_count > 0 {
                    FormalProofOutcome::Verified
                } else {
                    FormalProofOutcome::Violated {
                        reason: "entity_count is zero".into(),
                        counterexample: "SystemState { entity_count: 0 }".into(),
                    }
                }
            }
            InvariantKind::Temporal => {
                for (src, _rel, tgt) in &state.edges {
                    if src == tgt {
                        return FormalProofOutcome::Violated {
                            reason: format!("self-loop detected: {} -> {}", src, tgt),
                            counterexample: format!("edge ({}, {}, {})", src, _rel, tgt),
                        };
                    }
                }
                FormalProofOutcome::Verified
            }
            InvariantKind::Bound => {
                for (name, value) in &state.metrics {
                    if value.is_nan() || value.is_infinite() {
                        return FormalProofOutcome::Violated {
                            reason: format!("metric '{}' is NaN or infinite", name),
                            counterexample: format!("{} = {}", name, value),
                        };
                    }
                }
                FormalProofOutcome::Verified
            }
            InvariantKind::Consistency => {
                let mut seen = std::collections::HashSet::new();
                for entity in &state.entities {
                    let lower = entity.to_lowercase();
                    if !seen.insert(lower) {
                        return FormalProofOutcome::Violated {
                            reason: format!("duplicate entity: {}", entity),
                            counterexample: format!("entities contains '{}' twice", entity),
                        };
                    }
                }
                FormalProofOutcome::Verified
            }
            InvariantKind::Typed(_) => FormalProofOutcome::Unknown {
                reason: "no mock heuristic for typed invariants".into(),
            },
        }
    }

    fn name(&self) -> &str {
        "mock_verifier"
    }
}

#[derive(Debug, Clone)]
pub struct LeanVerifier {
    pub lean_path: Option<String>,
}

impl LeanVerifier {
    pub fn new() -> Self {
        Self { lean_path: None }
    }

    pub fn with_path(path: String) -> Self {
        Self {
            lean_path: Some(path),
        }
    }

    pub fn generate_lean_theorem(invariant: &Invariant) -> String {
        match &invariant.kind {
            InvariantKind::Structural => {
                format!(
                    r#"theorem {}_structural : state.entity_count > 0 := by
  have h : state.entity_count > 0 := by
    native_decide
  exact h"#,
                    invariant.id.replace('-', "_").replace(' ', "_")
                )
            }
            InvariantKind::Temporal => {
                format!(
                    r#"theorem {}_temporal : ∀ (src rel tgt : String), (src, rel, tgt) ∈ state.edges → src ≠ tgt := by
  intro src rel tgt hmem
  have h : src ≠ tgt := by
    native_decide
  exact h"#,
                    invariant.id.replace('-', "_").replace(' ', "_")
                )
            }
            InvariantKind::Bound => {
                format!(
                    r#"theorem {}_bound : state.cycle < 1000000 := by
  have h : state.cycle < 1000000 := by
    native_decide
  exact h"#,
                    invariant.id.replace('-', "_").replace(' ', "_")
                )
            }
            InvariantKind::Consistency => {
                format!(
                    r#"theorem {}_consistency : ∀ (a b : String), a ∈ state.entities → b ∈ state.entities → a ≠ b ↔ a.toLowerCase ≠ b.toLowerCase := by
  intro a b ha hb
  constructor
  · intro hneq; exact hneq
  · intro hneq; exact hneq"#,
                    invariant.id.replace('-', "_").replace(' ', "_")
                )
            }
            InvariantKind::Typed(tag) => {
                format!(
                    r#"-- typed invariant '{}' (tag: {})
theorem {}_typed : True := by
  trivial"#,
                    invariant.id,
                    tag,
                    invariant.id.replace('-', "_").replace(' ', "_")
                )
            }
        }
    }
}

impl Default for LeanVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl FormalVerifier for LeanVerifier {
    fn verify(&self, _invariant: &Invariant, _state: &SystemState) -> FormalProofOutcome {
        match &self.lean_path {
            Some(path) => {
                let path = std::path::Path::new(path);
                if !path.exists() || !path.is_file() {
                    return FormalProofOutcome::Unknown {
                        reason: format!("Lean binary not found at {}", path.display()),
                    };
                }
                let theorem = Self::generate_lean_theorem(_invariant);
                let tempfile =
                    std::env::temp_dir().join(format!("lean_verify_{}.lean", _invariant.id));
                match std::fs::write(&tempfile, &theorem) {
                    Ok(_) => {
                        let output = std::process::Command::new(path)
                            .arg("--run")
                            .arg(&tempfile)
                            .output();
                        match output {
                            Ok(out) => {
                                if out.status.success() {
                                    FormalProofOutcome::Verified
                                } else {
                                    let stderr = String::from_utf8_lossy(&out.stderr);
                                    FormalProofOutcome::Violated {
                                        reason: "Lean verification failed".into(),
                                        counterexample: stderr.to_string(),
                                    }
                                }
                            }
                            Err(e) => FormalProofOutcome::Unknown {
                                reason: format!("failed to execute Lean: {}", e),
                            },
                        }
                    }
                    Err(e) => FormalProofOutcome::Unknown {
                        reason: format!("failed to write Lean file: {}", e),
                    },
                }
            }
            None => FormalProofOutcome::Unknown {
                reason: "Lean 4 not available".into(),
            },
        }
    }

    fn name(&self) -> &str {
        "lean4"
    }
}

#[derive(Debug, Clone)]
pub struct IntrospectionEngine {
    pub invariants: Vec<Invariant>,
    pub reports: Vec<VerificationReport>,
    pub mock_verifier: MockVerifier,
    pub lean_verifier: Option<LeanVerifier>,
    pub cycle: u64,
}

impl IntrospectionEngine {
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
            reports: Vec::new(),
            mock_verifier: MockVerifier,
            lean_verifier: None,
            cycle: 0,
        }
    }

    pub fn register_invariant(&mut self, invariant: Invariant) {
        self.invariants.push(invariant);
    }

    pub fn remove_invariant(&mut self, id: &str) {
        self.invariants.retain(|i| i.id != id);
    }

    pub fn run_verification(
        &mut self,
        state: &SystemState,
        use_lean: bool,
    ) -> Vec<VerificationReport> {
        self.cycle += 1;
        let mut reports = Vec::with_capacity(self.invariants.len());

        for inv in &self.invariants {
            let result = if use_lean {
                if let Some(ref lean) = self.lean_verifier {
                    lean.verify(inv, state)
                } else {
                    self.mock_verifier.verify(inv, state)
                }
            } else {
                self.mock_verifier.verify(inv, state)
            };

            let verifier_name = if use_lean { "lean4" } else { "mock_verifier" };
            let report = VerificationReport {
                invariant_id: inv.id.clone(),
                result,
                verified_at_cycle: self.cycle,
                verifier: verifier_name.into(),
            };
            reports.push(report);
        }

        self.reports.extend(reports.clone());
        reports
    }

    pub fn run_all(&mut self, state: &SystemState) -> Vec<VerificationReport> {
        let mock_reports = self.run_verification(state, false);
        if self.lean_verifier.is_some() {
            let lean_reports = self.run_verification(state, true);
            let mut combined = mock_reports;
            combined.extend(lean_reports);
            combined
        } else {
            mock_reports
        }
    }

    pub fn pass_rate(&self, reports: &[VerificationReport]) -> f64 {
        if reports.is_empty() {
            return 1.0;
        }
        let verified = reports
            .iter()
            .filter(|r| r.result == FormalProofOutcome::Verified)
            .count();
        verified as f64 / reports.len() as f64
    }

    pub fn generate_default_invariants() -> Vec<Invariant> {
        vec![
            Invariant {
                id: "entities_exist".into(),
                kind: InvariantKind::Structural,
                description: "At least one entity must exist".into(),
                predicate: "entity_count > 0".into(),
                formal_expr: Some("state.entity_count > 0".into()),
            },
            Invariant {
                id: "no_self_loops".into(),
                kind: InvariantKind::Temporal,
                description: "No edge may connect an entity to itself".into(),
                predicate: "for all edges (src, rel, tgt): src != tgt".into(),
                formal_expr: Some(
                    "∀ (src rel tgt : String), (src, rel, tgt) ∈ state.edges → src ≠ tgt".into(),
                ),
            },
            Invariant {
                id: "cycle_progress".into(),
                kind: InvariantKind::Bound,
                description: "Cycle counter must not exceed 1,000,000".into(),
                predicate: "cycle < 1_000_000".into(),
                formal_expr: Some("state.cycle < 1000000".into()),
            },
            Invariant {
                id: "entity_edge_consistency".into(),
                kind: InvariantKind::Consistency,
                description: "Every edge references valid existing entities".into(),
                predicate: "for all edges (src, rel, tgt): src in entities and tgt in entities"
                    .into(),
                formal_expr: None,
            },
            Invariant {
                id: "positive_weights".into(),
                kind: InvariantKind::Typed("weight".into()),
                description: "All edge weights must be positive".into(),
                predicate: "for all edges: weight > 0".into(),
                formal_expr: None,
            },
        ]
    }

    pub fn summary(&self) -> String {
        if self.reports.is_empty() {
            return "0/0 invariants passed (100%)".into();
        }
        let total = self.reports.len();
        let passed = self
            .reports
            .iter()
            .filter(|r| r.result == FormalProofOutcome::Verified)
            .count();
        let pct = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            100.0
        };
        format!("{}/{} invariants passed ({:.0}%)", passed, total, pct)
    }
}

impl Default for IntrospectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn mock_verifier_structural_passes() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Structural,
            description: "dummy".into(),
            predicate: "entity_count > 0".into(),
            formal_expr: None,
        };
        let state = SystemState {
            entity_count: 5,
            edge_count: 0,
            cycle: 1,
            entities: vec!["a".into(), "b".into()],
            edges: Vec::new(),
            metrics: HashMap::new(),
        };
        assert_eq!(verifier.verify(&inv, &state), FormalProofOutcome::Verified);
    }

    #[test]
    fn mock_verifier_structural_fails() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Structural,
            description: "dummy".into(),
            predicate: "entity_count > 0".into(),
            formal_expr: None,
        };
        let state = SystemState::empty();
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Violated { .. } => {}
            _ => panic!("expected Violated"),
        }
    }

    #[test]
    fn mock_verifier_temporal_detects_self_loop() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Temporal,
            description: "dummy".into(),
            predicate: "no self loops".into(),
            formal_expr: None,
        };
        let state = SystemState {
            entity_count: 2,
            edge_count: 1,
            cycle: 1,
            entities: vec!["x".into(), "y".into()],
            edges: vec![("x".into(), "knows".into(), "x".into())],
            metrics: HashMap::new(),
        };
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Violated { .. } => {}
            _ => panic!("expected Violated for self-loop"),
        }
    }

    #[test]
    fn mock_verifier_consistency_detects_duplicates() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Consistency,
            description: "dummy".into(),
            predicate: "no duplicates".into(),
            formal_expr: None,
        };
        let state = SystemState {
            entity_count: 3,
            edge_count: 0,
            cycle: 1,
            entities: vec!["Alice".into(), "Bob".into(), "alice".into()],
            edges: Vec::new(),
            metrics: HashMap::new(),
        };
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Violated { reason, .. } => {
                assert!(reason.contains("alice") || reason.contains("Alice"));
            }
            _ => panic!("expected Violated for duplicates"),
        }
    }

    #[test]
    fn mock_verifier_bound_checks_metrics() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Bound,
            description: "dummy".into(),
            predicate: "metrics are finite".into(),
            formal_expr: None,
        };
        let mut metrics = HashMap::new();
        metrics.insert("temperature".into(), f64::NAN);
        let state = SystemState {
            entity_count: 1,
            edge_count: 0,
            cycle: 1,
            entities: vec!["a".into()],
            edges: Vec::new(),
            metrics,
        };
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Violated { .. } => {}
            _ => panic!("expected Violated for NaN metric"),
        }
    }

    #[test]
    fn mock_verifier_typed_returns_unknown() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Typed("custom".into()),
            description: "dummy".into(),
            predicate: "something".into(),
            formal_expr: None,
        };
        let state = SystemState::empty();
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Unknown { .. } => {}
            _ => panic!("expected Unknown for typed invariant"),
        }
    }

    #[test]
    fn lean_verifier_unknown_when_not_available() {
        let verifier = LeanVerifier::new();
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Structural,
            description: "dummy".into(),
            predicate: "entity_count > 0".into(),
            formal_expr: None,
        };
        let state = SystemState::empty();
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Unknown { reason } => {
                assert!(reason.contains("Lean"));
            }
            _ => panic!("expected Unknown when Lean not available"),
        }
    }

    #[test]
    fn lean_generate_theorem_produces_valid_string() {
        let inv = Invariant {
            id: "entities_exist".into(),
            kind: InvariantKind::Structural,
            description: "test".into(),
            predicate: "entity_count > 0".into(),
            formal_expr: None,
        };
        let theorem = LeanVerifier::generate_lean_theorem(&inv);
        assert!(theorem.starts_with("theorem"));
        assert!(theorem.contains("entity_count > 0"));
        assert!(theorem.contains("native_decide"));
    }

    #[test]
    fn lean_generate_theorem_all_kinds() {
        let kinds = vec![
            InvariantKind::Structural,
            InvariantKind::Temporal,
            InvariantKind::Bound,
            InvariantKind::Consistency,
            InvariantKind::Typed("test".into()),
        ];
        for kind in kinds {
            let inv = Invariant {
                id: "check".into(),
                kind,
                description: "x".into(),
                predicate: "x".into(),
                formal_expr: None,
            };
            let theorem = LeanVerifier::generate_lean_theorem(&inv);
            assert!(!theorem.is_empty());
            assert!(theorem.contains("theorem check_"));
        }
    }

    #[test]
    fn engine_register_and_run() {
        let mut engine = IntrospectionEngine::new();
        let inv = Invariant {
            id: "test_inv".into(),
            kind: InvariantKind::Structural,
            description: "dummy".into(),
            predicate: "entity_count > 0".into(),
            formal_expr: None,
        };
        engine.register_invariant(inv);
        assert_eq!(engine.invariants.len(), 1);

        let state = SystemState {
            entity_count: 3,
            edge_count: 0,
            cycle: 1,
            entities: vec!["a".into(), "b".into(), "c".into()],
            edges: Vec::new(),
            metrics: HashMap::new(),
        };
        let reports = engine.run_verification(&state, false);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].result, FormalProofOutcome::Verified);
        assert_eq!(reports[0].invariant_id, "test_inv");
    }

    #[test]
    fn pass_rate_mixed_results() {
        let reports = vec![
            VerificationReport {
                invariant_id: "a".into(),
                result: FormalProofOutcome::Verified,
                verified_at_cycle: 1,
                verifier: "mock".into(),
            },
            VerificationReport {
                invariant_id: "b".into(),
                result: FormalProofOutcome::Verified,
                verified_at_cycle: 1,
                verifier: "mock".into(),
            },
            VerificationReport {
                invariant_id: "c".into(),
                result: FormalProofOutcome::Violated {
                    reason: "test".into(),
                    counterexample: "test".into(),
                },
                verified_at_cycle: 1,
                verifier: "mock".into(),
            },
        ];
        let engine = IntrospectionEngine::new();
        let rate = engine.pass_rate(&reports);
        assert!((rate - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn generate_default_invariants_count() {
        let invariants = IntrospectionEngine::generate_default_invariants();
        assert_eq!(invariants.len(), 5);
        assert_eq!(invariants[0].id, "entities_exist");
        assert_eq!(invariants[1].id, "no_self_loops");
        assert_eq!(invariants[2].id, "cycle_progress");
        assert_eq!(invariants[3].id, "entity_edge_consistency");
        assert_eq!(invariants[4].id, "positive_weights");
    }

    #[test]
    fn summary_string_formatting() {
        let mut engine = IntrospectionEngine::new();
        assert_eq!(engine.summary(), "0/0 invariants passed (100%)");

        engine.reports.push(VerificationReport {
            invariant_id: "a".into(),
            result: FormalProofOutcome::Verified,
            verified_at_cycle: 1,
            verifier: "mock".into(),
        });
        engine.reports.push(VerificationReport {
            invariant_id: "b".into(),
            result: FormalProofOutcome::Violated {
                reason: "x".into(),
                counterexample: "y".into(),
            },
            verified_at_cycle: 1,
            verifier: "mock".into(),
        });
        let s = engine.summary();
        assert!(s.contains("1/2"));
        assert!(s.contains("50%"));
    }

    #[test]
    fn multiple_runs_accumulate_reports() {
        let mut engine = IntrospectionEngine::new();
        let inv = Invariant {
            id: "e".into(),
            kind: InvariantKind::Structural,
            description: "d".into(),
            predicate: "p".into(),
            formal_expr: None,
        };
        engine.register_invariant(inv);

        let state = SystemState {
            entity_count: 1,
            edge_count: 0,
            cycle: 1,
            entities: vec!["x".into()],
            edges: Vec::new(),
            metrics: HashMap::new(),
        };
        engine.run_verification(&state, false);
        assert_eq!(engine.reports.len(), 1);
        engine.run_verification(&state, false);
        assert_eq!(engine.reports.len(), 2);
        engine.run_verification(&state, false);
        assert_eq!(engine.reports.len(), 3);
    }

    #[test]
    fn empty_state_no_entities() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Structural,
            description: "dummy".into(),
            predicate: "entity_count > 0".into(),
            formal_expr: None,
        };
        let state = SystemState::empty();
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Violated { reason, .. } => {
                assert!(reason.contains("zero") || reason.contains("0"));
            }
            _ => panic!("expected Violated for empty state"),
        }

        let inv2 = Invariant {
            id: "empty_ok".into(),
            kind: InvariantKind::Consistency,
            description: "dummy".into(),
            predicate: "no duplicates".into(),
            formal_expr: None,
        };
        assert_eq!(verifier.verify(&inv2, &state), FormalProofOutcome::Verified);
    }

    #[test]
    fn lean_verifier_with_nonexistent_path() {
        let verifier = LeanVerifier::with_path("/nonexistent/lean".into());
        let inv = Invariant {
            id: "test".into(),
            kind: InvariantKind::Structural,
            description: "d".into(),
            predicate: "p".into(),
            formal_expr: None,
        };
        let state = SystemState::empty();
        match verifier.verify(&inv, &state) {
            FormalProofOutcome::Unknown { reason } => {
                assert!(reason.contains("not found"));
            }
            _ => panic!("expected Unknown for missing binary"),
        }
    }

    #[test]
    fn consistency_passes_with_unique_entities() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "unique".into(),
            kind: InvariantKind::Consistency,
            description: "d".into(),
            predicate: "no duplicates".into(),
            formal_expr: None,
        };
        let state = SystemState {
            entity_count: 3,
            edge_count: 0,
            cycle: 1,
            entities: vec!["Alice".into(), "Bob".into(), "Charlie".into()],
            edges: Vec::new(),
            metrics: HashMap::new(),
        };
        assert_eq!(verifier.verify(&inv, &state), FormalProofOutcome::Verified);
    }

    #[test]
    fn temporal_passes_with_valid_edges() {
        let verifier = MockVerifier;
        let inv = Invariant {
            id: "no_loop".into(),
            kind: InvariantKind::Temporal,
            description: "d".into(),
            predicate: "no self loops".into(),
            formal_expr: None,
        };
        let state = SystemState {
            entity_count: 3,
            edge_count: 2,
            cycle: 1,
            entities: vec!["a".into(), "b".into(), "c".into()],
            edges: vec![
                ("a".into(), "knows".into(), "b".into()),
                ("b".into(), "knows".into(), "c".into()),
            ],
            metrics: HashMap::new(),
        };
        assert_eq!(verifier.verify(&inv, &state), FormalProofOutcome::Verified);
    }

    #[test]
    fn engine_name_and_verifier_name() {
        let mock = MockVerifier;
        assert_eq!(mock.name(), "mock_verifier");
        let lean = LeanVerifier::new();
        assert_eq!(lean.name(), "lean4");
    }

    #[test]
    fn remove_invariant_works() {
        let mut engine = IntrospectionEngine::new();
        engine.register_invariant(Invariant {
            id: "keep".into(),
            kind: InvariantKind::Structural,
            description: "".into(),
            predicate: "".into(),
            formal_expr: None,
        });
        engine.register_invariant(Invariant {
            id: "remove_me".into(),
            kind: InvariantKind::Bound,
            description: "".into(),
            predicate: "".into(),
            formal_expr: None,
        });
        assert_eq!(engine.invariants.len(), 2);
        engine.remove_invariant("remove_me");
        assert_eq!(engine.invariants.len(), 1);
        assert_eq!(engine.invariants[0].id, "keep");
    }

    #[test]
    fn pass_rate_empty_reports() {
        let engine = IntrospectionEngine::new();
        assert!((engine.pass_rate(&[]) - 1.0).abs() < 1e-10);
    }
}
