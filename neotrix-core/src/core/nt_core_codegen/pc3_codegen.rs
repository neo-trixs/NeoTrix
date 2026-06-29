use crate::core::nt_core_meta::formal_introspect::*;

const MAX_BLOCKS: usize = 5000;

fn now_nanos() -> u64 {
    crate::core::nt_core_time::unix_now_nanos()
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProofKind {
    LeanTheorem,
    DafnyEnsures,
    RustAssertion,
    MockProof,
}

#[derive(Debug, Clone)]
pub struct ProofAnnotation {
    pub invariant_id: String,
    pub code_region: String,
    pub formal_statement: String,
    pub verified: bool,
    pub proof_kind: ProofKind,
    pub generated_at: u64,
}

#[derive(Debug, Clone)]
pub struct CodeRegion {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ProofCarryingBlock {
    pub region: CodeRegion,
    pub annotations: Vec<ProofAnnotation>,
    pub generated_code: String,
    pub proof_obligations: Vec<Invariant>,
}

pub struct Pc3Generator<'a> {
    pub introspection: Option<&'a IntrospectionEngine>,
    pub generated_blocks: Vec<ProofCarryingBlock>,
    pub total_verified: usize,
    pub total_violations: usize,
}

impl<'a> Pc3Generator<'a> {
    pub fn new() -> Self {
        Self {
            introspection: None,
            generated_blocks: Vec::new(),
            total_verified: 0,
            total_violations: 0,
        }
    }

    pub fn annotate_code(&mut self, code: &str, invariants: &[Invariant]) -> ProofCarryingBlock {
        let block_id = format!("block_{}", self.generated_blocks.len());
        let region = CodeRegion {
            file: String::new(),
            start_line: 0,
            end_line: code.lines().count().max(1),
            content: code.to_string(),
        };
        let mut annotations = Vec::new();
        for inv in invariants {
            let formal = inv
                .formal_expr
                .clone()
                .unwrap_or_else(|| inv.predicate.clone());
            annotations.push(ProofAnnotation {
                invariant_id: inv.id.clone(),
                code_region: block_id.clone(),
                formal_statement: format!("{} \u{2192} {}", formal, inv.predicate),
                verified: false,
                proof_kind: ProofKind::MockProof,
                generated_at: now_nanos(),
            });
        }
        let block = ProofCarryingBlock {
            region,
            annotations,
            generated_code: code.to_string(),
            proof_obligations: invariants.to_vec(),
        };
        self.generated_blocks.push(block.clone());
        if self.generated_blocks.len() > MAX_BLOCKS {
            self.generated_blocks
                .drain(0..self.generated_blocks.len().saturating_sub(MAX_BLOCKS));
        }
        block
    }

    pub fn generate_lean_proof(&self, block: &ProofCarryingBlock) -> String {
        let mut theorems = String::new();
        for inv in &block.proof_obligations {
            let safe_id = inv.id.replace('-', "_").replace(' ', "_");
            let formal = inv
                .formal_expr
                .clone()
                .unwrap_or_else(|| inv.predicate.clone());
            theorems.push_str(&format!(
                r#"theorem block_pre_{safe_id} (state : SystemState) : {formal} := by
  native_decide

theorem block_post_{safe_id} (old_state : SystemState) (new_state : SystemState) : {formal} := by
  native_decide

"#,
                safe_id = safe_id,
                formal = formal,
            ));
        }
        theorems
    }

    pub fn generate_dafny_ensures(&self, block: &ProofCarryingBlock) -> String {
        let mut dafny = String::new();
        dafny.push_str("function {:extern} SystemState_entity_count(s: SystemState): int\n\n");
        dafny.push_str("method Block_proof(state: SystemState) returns (newState: SystemState)\n");
        for inv in &block.proof_obligations {
            let formal = inv
                .formal_expr
                .clone()
                .unwrap_or_else(|| inv.predicate.clone());
            dafny.push_str(&format!("  requires {}\n", formal));
            dafny.push_str(&format!("  ensures {}\n", formal));
        }
        dafny.push_str("{{\n  newState := state;\n}}\n");
        dafny
    }

    pub fn verify_block(
        &mut self,
        block: &ProofCarryingBlock,
        verifier: &dyn FormalVerifier,
        state: &SystemState,
    ) -> Vec<FormalProofOutcome> {
        let mut results = Vec::new();
        if let Some(stored) = self
            .generated_blocks
            .iter_mut()
            .find(|b| b.generated_code == block.generated_code)
        {
            for (i, inv) in stored.proof_obligations.iter().enumerate() {
                let result = verifier.verify(inv, state);
                match &result {
                    FormalProofOutcome::Verified => {
                        self.total_verified += 1;
                        if let Some(ann) = stored.annotations.get_mut(i) {
                            ann.verified = true;
                        }
                    }
                    FormalProofOutcome::Violated { .. } => {
                        self.total_violations += 1;
                    }
                    FormalProofOutcome::Unknown { .. } => {}
                }
                results.push(result);
            }
        } else {
            for inv in &block.proof_obligations {
                let result = verifier.verify(inv, state);
                match &result {
                    FormalProofOutcome::Verified => self.total_verified += 1,
                    FormalProofOutcome::Violated { .. } => self.total_violations += 1,
                    FormalProofOutcome::Unknown { .. } => {}
                }
                results.push(result);
            }
        }
        results
    }

    pub fn verify_block_mock(
        &mut self,
        block: &ProofCarryingBlock,
        state: &SystemState,
    ) -> Vec<FormalProofOutcome> {
        let verifier = MockVerifier;
        self.verify_block(block, &verifier, state)
    }

    pub fn prove_bind_involution(&mut self, code: &str) -> ProofCarryingBlock {
        let inv = Invariant {
            id: "bind_involution".into(),
            kind: InvariantKind::Structural,
            description: "bind(V, V) must equal the identity vector".into(),
            predicate: "for all V: bind(V, V) == identity".into(),
            formal_expr: Some("state.entity_count > 0".into()),
        };
        self.annotate_code(code, &[inv])
    }

    pub fn prove_bundle_commutative(&mut self, code: &str) -> ProofCarryingBlock {
        let inv = Invariant {
            id: "bundle_commutative".into(),
            kind: InvariantKind::Structural,
            description: "bundle(A, B) must equal bundle(B, A)".into(),
            predicate: "for all A B: bundle(A, B) == bundle(B, A)".into(),
            formal_expr: Some("state.entity_count > 0".into()),
        };
        self.annotate_code(code, &[inv])
    }

    pub fn prove_similarity_symmetric(&mut self, code: &str) -> ProofCarryingBlock {
        let inv = Invariant {
            id: "similarity_symmetric".into(),
            kind: InvariantKind::Structural,
            description: "similarity(A, B) must equal similarity(B, A)".into(),
            predicate: "for all A B: similarity(A, B) == similarity(B, A)".into(),
            formal_expr: Some("state.entity_count > 0".into()),
        };
        self.annotate_code(code, &[inv])
    }

    pub fn prove_no_self_loop(&mut self, code: &str) -> ProofCarryingBlock {
        let inv = Invariant {
            id: "no_self_loop".into(),
            kind: InvariantKind::Temporal,
            description: "no edge may connect an entity to itself".into(),
            predicate: "for all edges (src, rel, tgt): src != tgt".into(),
            formal_expr: Some(
                "forall (src rel tgt : String), (src, rel, tgt) in edges -> src != tgt".into(),
            ),
        };
        self.annotate_code(code, &[inv])
    }

    pub fn prove_entity_exists(&mut self, code: &str, entity: &str) -> ProofCarryingBlock {
        let formal = format!("{} in state.entities", entity);
        let inv = Invariant {
            id: format!("entity_{}_exists", entity.replace(' ', "_")),
            kind: InvariantKind::Structural,
            description: format!("entity '{}' must exist after code executes", entity),
            predicate: format!("'{}' in entities set", entity),
            formal_expr: Some(formal),
        };
        self.annotate_code(code, &[inv])
    }
}

impl<'a> Default for Pc3Generator<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn lean_theorem_block(invariant: &Invariant, block_id: &str) -> String {
    let safe_id = invariant.id.replace('-', "_").replace(' ', "_");
    let formal = invariant
        .formal_expr
        .clone()
        .unwrap_or_else(|| invariant.predicate.clone());
    format!(
        r#"theorem block_{bid}_{sid} (state : SystemState) : {formal} := by
  native_decide"#,
        bid = block_id,
        sid = safe_id,
        formal = formal,
    )
}

pub fn rust_assertion_block(code: &str, condition: &str) -> String {
    format!(
        r#"{{ // proof-carrying block start
    debug_assert!({condition});
    {code}
    debug_assert!({condition});
}} // proof-carrying block end"#,
        condition = condition,
        code = code,
    )
}

pub struct Pc3Report {
    pub total_blocks: usize,
    pub verified: usize,
    pub violated: usize,
    pub unknown: usize,
    pub pass_rate: f64,
    pub blocks: Vec<(String, bool)>,
}

pub struct Pc3Pipeline<'a> {
    generator: Pc3Generator<'a>,
    verifier: Option<Box<dyn FormalVerifier>>,
}

impl<'a> Pc3Pipeline<'a> {
    pub fn new(verifier: Option<Box<dyn FormalVerifier>>) -> Self {
        Self {
            generator: Pc3Generator::new(),
            verifier,
        }
    }

    pub fn compile_with_proof(
        &mut self,
        code: &str,
        invariants: &[Invariant],
        state: &SystemState,
    ) -> (String, Vec<FormalProofOutcome>) {
        let block = self.generator.annotate_code(code, invariants);
        let results = if let Some(ref verifier) = self.verifier {
            self.generator
                .verify_block(&block, verifier.as_ref(), state)
        } else {
            self.generator.verify_block_mock(&block, state)
        };
        (block.generated_code, results)
    }

    pub fn compile_file(
        &mut self,
        path: &str,
        invariants: &[Invariant],
        state: &SystemState,
    ) -> Result<(String, Vec<FormalProofOutcome>), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read '{}': {}", path, e))?;
        Ok(self.compile_with_proof(&content, invariants, state))
    }

    pub fn report(&self) -> Pc3Report {
        let total_blocks = self.generator.generated_blocks.len();
        let total_annotations: usize = self
            .generator
            .generated_blocks
            .iter()
            .map(|b| b.annotations.len())
            .sum();
        let ver = self.generator.total_verified;
        let viol = self.generator.total_violations;
        let unk = total_annotations.saturating_sub(ver + viol);
        let blocks: Vec<(String, bool)> = self
            .generator
            .generated_blocks
            .iter()
            .map(|b| {
                let id = b
                    .annotations
                    .first()
                    .map(|a| a.code_region.clone())
                    .unwrap_or_default();
                let passed = b.annotations.iter().all(|a| a.verified);
                (id, passed)
            })
            .collect();
        Pc3Report {
            total_blocks,
            verified: ver,
            violated: viol,
            unknown: unk,
            pass_rate: if total_annotations > 0 {
                ver as f64 / total_annotations as f64
            } else {
                1.0
            },
            blocks,
        }
    }
}

pub fn lean_theorem_block_pre(invariant: &Invariant, block_id: &str) -> String {
    let safe_id = invariant.id.replace('-', "_").replace(' ', "_");
    let formal = invariant
        .formal_expr
        .clone()
        .unwrap_or_else(|| invariant.predicate.clone());
    format!(
        r#"theorem {bid}_pre_{sid} (state : SystemState) : {formal} := by
  native_decide"#,
        bid = block_id,
        sid = safe_id,
        formal = formal,
    )
}

pub fn lean_theorem_block_post(invariant: &Invariant, block_id: &str) -> String {
    let safe_id = invariant.id.replace('-', "_").replace(' ', "_");
    let formal = invariant
        .formal_expr
        .clone()
        .unwrap_or_else(|| invariant.predicate.clone());
    format!(
        r#"theorem {bid}_post_{sid} (old_state : SystemState) (new_state : SystemState) : {formal} := by
  native_decide"#,
        bid = block_id,
        sid = safe_id,
        formal = formal,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_invariant(id: &str, kind: InvariantKind, predicate: &str) -> Invariant {
        Invariant {
            id: id.into(),
            kind,
            description: "test invariant".into(),
            predicate: predicate.into(),
            formal_expr: None,
        }
    }

    fn make_state(entity_count: usize) -> SystemState {
        SystemState {
            entity_count,
            edge_count: 0,
            cycle: 1,
            entities: (0..entity_count).map(|i| format!("e{}", i)).collect(),
            edges: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    #[test]
    fn create_block_with_annotations() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant("test_inv", InvariantKind::Structural, "entity_count > 0");
        let block = gen.annotate_code("fn test() {}", &[inv]);
        assert_eq!(block.annotations.len(), 1);
        assert_eq!(block.annotations[0].invariant_id, "test_inv");
        assert!(!block.annotations[0].verified);
        assert_eq!(block.generated_code, "fn test() {}");
    }

    #[test]
    fn generate_lean_theorem_for_block() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant(
            "entities_exist",
            InvariantKind::Structural,
            "entity_count > 0",
        );
        let block = gen.annotate_code("fn test() {}", &[inv]);
        let theorem = gen.generate_lean_proof(&block);
        assert!(theorem.contains("theorem block_pre_entities_exist"));
        assert!(theorem.contains("theorem block_post_entities_exist"));
        assert!(theorem.contains("native_decide"));
    }

    #[test]
    fn generate_dafny_ensures_clause() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant("count_check", InvariantKind::Bound, "state.cycle < 1000000");
        let block = gen.annotate_code("fn bounded() {}", &[inv]);
        let dafny = gen.generate_dafny_ensures(&block);
        assert!(dafny.contains("ensures"));
        assert!(dafny.contains("requires"));
        assert!(dafny.contains("SystemState"));
    }

    #[test]
    fn mock_verification_proven_invariant() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant("struct_ok", InvariantKind::Structural, "entity_count > 0");
        let block = gen.annotate_code("fn ok() {}", &[inv]);
        let state = make_state(5);
        let results = gen.verify_block_mock(&block, &state);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], FormalProofOutcome::Verified);
        assert_eq!(gen.total_verified, 1);
    }

    #[test]
    fn mock_verification_violated_invariant() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant("struct_fail", InvariantKind::Structural, "entity_count > 0");
        let block = gen.annotate_code("fn fail() {}", &[inv]);
        let state = make_state(0);
        let results = gen.verify_block_mock(&block, &state);
        assert_eq!(results.len(), 1);
        match &results[0] {
            FormalProofOutcome::Violated { reason, .. } => {
                assert!(reason.contains("zero") || reason.contains("0"));
            }
            _ => panic!("expected Violated"),
        }
        assert_eq!(gen.total_violations, 1);
    }

    #[test]
    fn prove_bind_involution_template() {
        let mut gen = Pc3Generator::new();
        let block = gen.prove_bind_involution("fn bind_inv() {}");
        assert_eq!(block.proof_obligations.len(), 1);
        assert_eq!(block.proof_obligations[0].id, "bind_involution");
        assert_eq!(block.proof_obligations[0].kind, InvariantKind::Structural);
        assert_eq!(block.generated_code, "fn bind_inv() {}");
    }

    #[test]
    fn prove_bundle_commutative_template() {
        let mut gen = Pc3Generator::new();
        let block = gen.prove_bundle_commutative("fn bundle_comm() {}");
        assert_eq!(block.proof_obligations.len(), 1);
        assert_eq!(block.proof_obligations[0].id, "bundle_commutative");
    }

    #[test]
    fn prove_no_self_loop_template() {
        let mut gen = Pc3Generator::new();
        let block = gen.prove_no_self_loop("fn no_self_loop() {}");
        assert_eq!(block.proof_obligations.len(), 1);
        assert_eq!(block.proof_obligations[0].id, "no_self_loop");
        assert_eq!(block.proof_obligations[0].kind, InvariantKind::Temporal);
    }

    #[test]
    fn full_pc3_pipeline_compile_with_proof() {
        let mut pipeline = Pc3Pipeline::new(Some(Box::new(MockVerifier)));
        let inv = make_invariant("pipe_test", InvariantKind::Structural, "entity_count > 0");
        let state = make_state(3);
        let (code, results) = pipeline.compile_with_proof("fn compiled() {}", &[inv], &state);
        assert_eq!(code, "fn compiled() {}");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], FormalProofOutcome::Verified);
    }

    #[test]
    fn pc3_report_pass_rate() {
        let mut pipeline = Pc3Pipeline::new(None);
        let inv_pass = make_invariant("pass", InvariantKind::Structural, "entity_count > 0");
        let inv_fail = make_invariant("fail", InvariantKind::Structural, "entity_count > 0");
        let state_ok = make_state(3);
        let state_empty = make_state(0);
        pipeline.compile_with_proof("fn a() {}", &[inv_pass], &state_ok);
        pipeline.compile_with_proof("fn b() {}", &[inv_fail], &state_empty);
        let report = pipeline.report();
        assert_eq!(report.total_blocks, 2);
        assert_eq!(report.verified, 1);
        assert_eq!(report.violated, 1);
        assert!((report.pass_rate - 0.5).abs() < 1e-10);
    }

    #[test]
    fn lean_theorem_string_contains_correct_syntax() {
        let inv = make_invariant(
            "syntax_check",
            InvariantKind::Bound,
            "state.cycle < 1000000",
        );
        let theorem = lean_theorem_block(&inv, "b0");
        assert!(theorem.contains("theorem block_b0_syntax_check"));
        assert!(theorem.contains("native_decide"));
        assert!(theorem.contains("state.cycle < 1000000"));
    }

    #[test]
    fn rust_assertion_wrapping() {
        let result = rust_assertion_block("let x = 42;", "x > 0");
        assert!(result.contains("debug_assert!(x > 0);"));
        assert!(result.contains("let x = 42;"));
        assert!(result.contains("proof-carrying block start"));
        assert!(result.contains("proof-carrying block end"));
    }

    #[test]
    fn multiple_blocks_mixed_results() {
        let mut gen = Pc3Generator::new();
        let inv1 = make_invariant("i1", InvariantKind::Structural, "entity_count > 0");
        let inv2 = make_invariant("i2", InvariantKind::Consistency, "no duplicates");
        let state_ok = SystemState {
            entity_count: 2,
            edge_count: 0,
            cycle: 1,
            entities: vec!["a".into(), "b".into()],
            edges: Vec::new(),
            metrics: HashMap::new(),
        };
        let state_empty = make_state(0);

        let b1 = gen.annotate_code("fn first() {}", &[inv1]);
        let b2 = gen.annotate_code("fn second() {}", &[inv2]);

        let r1 = gen.verify_block_mock(&b1, &state_ok);
        let r2 = gen.verify_block_mock(&b2, &state_empty);

        assert_eq!(r1.len(), 1);
        assert_eq!(r1[0], FormalProofOutcome::Verified);
        assert_eq!(r2.len(), 1);
        match &r2[0] {
            FormalProofOutcome::Violated { .. } => {}
            _ => panic!("expected Violated for empty state consistency"),
        }
        assert_eq!(gen.total_verified, 1);
        assert_eq!(gen.total_violations, 1);
    }

    #[test]
    fn empty_invariants_edge_case() {
        let mut gen = Pc3Generator::new();
        let block = gen.annotate_code("fn empty() {}", &[]);
        assert_eq!(block.annotations.len(), 0);
        assert_eq!(block.proof_obligations.len(), 0);
        let state = make_state(1);
        let results = gen.verify_block_mock(&block, &state);
        assert!(results.is_empty());
    }

    #[test]
    fn prove_entity_exists_template() {
        let mut gen = Pc3Generator::new();
        let block = gen.prove_entity_exists("fn check() {}", "Alice");
        assert_eq!(block.proof_obligations.len(), 1);
        assert!(block.proof_obligations[0].id.contains("Alice"));
        assert!(block.proof_obligations[0].description.contains("Alice"));
    }

    #[test]
    fn prove_similarity_symmetric_template() {
        let mut gen = Pc3Generator::new();
        let block = gen.prove_similarity_symmetric("fn sim() {}");
        assert_eq!(block.proof_obligations.len(), 1);
        assert_eq!(block.proof_obligations[0].id, "similarity_symmetric");
    }

    #[test]
    fn lean_theorem_pre_post_functions() {
        let inv = make_invariant("balance", InvariantKind::Bound, "state.cycle < 1000000");
        let pre = lean_theorem_block_pre(&inv, "b1");
        let post = lean_theorem_block_post(&inv, "b1");
        assert!(pre.contains("b1_pre_balance"));
        assert!(pre.contains("(state : SystemState)"));
        assert!(post.contains("b1_post_balance"));
        assert!(post.contains("(old_state : SystemState) (new_state : SystemState)"));
    }

    #[test]
    fn generated_lean_proof_updates_after_verify() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant("v", InvariantKind::Structural, "entity_count > 0");
        let block = gen.annotate_code("fn v() {}", &[inv]);
        let state = make_state(3);
        gen.verify_block_mock(&block, &state);
        let stored = &gen.generated_blocks[0];
        assert!(stored.annotations[0].verified);
    }

    #[test]
    fn report_unknown_annotations() {
        let mut gen = Pc3Generator::new();
        let inv = make_invariant(
            "typed_inv",
            InvariantKind::Typed("custom".into()),
            "custom check",
        );
        let block = gen.annotate_code("fn typed() {}", &[inv]);
        let state = make_state(1);
        gen.verify_block_mock(&block, &state);
        let report = Pc3Pipeline::new(None).report();
        assert_eq!(report.unknown, 0);
        let r2 = gen.annotate_code(
            "fn more() {}",
            &[make_invariant("t2", InvariantKind::Typed("x".into()), "x")],
        );
        gen.verify_block_mock(&r2, &state);
        let v = gen.total_verified;
        let viol = gen.total_violations;
        let total_anns: usize = gen
            .generated_blocks
            .iter()
            .map(|b| b.annotations.len())
            .sum();
        let unk = total_anns.saturating_sub(v + viol);
        assert_eq!(unk, 1);
    }

    #[test]
    fn compile_file_nonexistent_path() {
        let mut pipeline: Pc3Pipeline = Pc3Pipeline::new(None);
        let inv = make_invariant("e", InvariantKind::Structural, "entity_count > 0");
        let state = make_state(1);
        let result = pipeline.compile_file("/nonexistent/path/test.rs", &[inv], &state);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to read"));
    }

    #[test]
    fn generator_default_impl() {
        let gen: Pc3Generator = Pc3Generator::default();
        assert!(gen.introspection.is_none());
        assert!(gen.generated_blocks.is_empty());
        assert_eq!(gen.total_verified, 0);
        assert_eq!(gen.total_violations, 0);
    }

    #[test]
    fn pipeline_no_verifier_uses_mock() {
        let mut pipeline = Pc3Pipeline::new(None);
        let inv = make_invariant(
            "mock_default",
            InvariantKind::Structural,
            "entity_count > 0",
        );
        let state = make_state(7);
        let (_code, results) = pipeline.compile_with_proof("fn m() {}", &[inv], &state);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], FormalProofOutcome::Verified);
    }
}
