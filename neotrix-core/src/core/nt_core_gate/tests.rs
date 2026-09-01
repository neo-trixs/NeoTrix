    use super::*;
    use crate::core::nt_core_hex::ReasoningHexagram;
    use crate::core::nt_core_prm::TrajectoryStep;
    use crate::core::nt_core_traits::SpecialistType;
    use crate::l1_action::nt_io::nt_io_provider::{
        FinishReason, LlmError, LlmRequest, LlmResponse, Usage,
    };

    /// 测试用 mock LLM 法官 — 返回固定结构化 JSON 评分。
    struct MockJudgeProvider {
        score: f64,
        confidence: f64,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockJudgeProvider {
    fn data_trust(&self) -> crate::core::nt_core_llm::DataTrust {
        crate::core::nt_core_llm::DataTrust::Trusted
    }

        async fn complete_raw(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            let content = format!(
                r#"{{"score":{}, "confidence":{}, "rationale":"mock judge"}}"#,
                self.score, self.confidence
            );
            Ok(LlmResponse {
                content,
                model: "mock-judge".into(),
                usage: Usage::default(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
             reasoning: None,})
        }

        async fn stream_complete_raw(
            &self,
            _request: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (_, rx) = tokio::sync::mpsc::channel(1);
            Ok(rx)
        }
    }

    fn simple_traj(id: u64, task: &str, successes: usize, total: usize) -> AgentTrajectory {
        let mut t = AgentTrajectory::new(id, task.to_string());
        for i in 0..total {
            t.push(TrajectoryStep {
                step_idx: i,
                specialist: SpecialistType::Planner,
                e8_mode: ReasoningHexagram::new(0b001010),
                action: format!("step {}", i),
                input: String::new(),
                output: format!("out {}", i),
                duration_ms: None,
                success: i < successes,
                external_reward: None,
            });
        }
        t
    }

    // ── ActionTier / blast radius ──
    #[test]
    fn tier_all_read_only_is_autonomous() {
        let tools = vec![ToolSpec::read_only("select"), ToolSpec::read_only("get")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier1Autonomous);
        assert_eq!(
            ActionTier::classify(&tools).required_gate(),
            GateLevel::Light
        );
    }

    #[test]
    fn tier_reversible_is_logged_light() {
        let tools = vec![ToolSpec::reversible("edit_file", "undo_edit")];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier2Logged);
        assert_eq!(
            ActionTier::classify(&tools).required_gate(),
            GateLevel::Light
        );
    }

    #[test]
    fn tier_irreversible_forces_human() {
        let tools = vec![
            ToolSpec::read_only("get"),
            ToolSpec::irreversible("send_email"),
        ];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier4Human);
        assert_eq!(
            ActionTier::classify(&tools).required_gate(),
            GateLevel::Human
        );
    }

    #[test]
    fn tier_authority_modifying_forces_human() {
        let mut t = ToolSpec::read_only("grant_role");
        t.authority_modifying = true;
        assert_eq!(ActionTier::classify(&[t]), ActionTier::Tier4Human);
    }

    #[test]
    fn tier_compensable_is_review() {
        let tools = vec![
            ToolSpec::read_only("get"),
            ToolSpec {
                name: "refund".to_string(),
                reversibility: ToolReversibility::Compensable,
                undo: None,
                authority_modifying: false,
            },
        ];
        assert_eq!(ActionTier::classify(&tools), ActionTier::Tier3Review);
    }

    // ── Faithfulness ──
    #[test]
    fn faithfulness_set_difference_catches_fabrication() {
        let claims = vec![
            Claim::new("claim with real ref", &["E-001"]),
            Claim::new("claim with missing ref", &["E-999"]),
            Claim::new("claim with no ref", &[]),
        ];
        let report =
            FaithfulnessReport::audit(&claims, &["E-001".to_string(), "E-002".to_string()]);
        assert_eq!(report.grounded, 1);
        assert_eq!(report.fabricated.len(), 2);
        assert!(!report.is_grounded(0.6));
        assert!(!report.quarantine().is_empty());
    }

    #[test]
    fn faithfulness_empty_claims_is_zero() {
        let report = FaithfulnessReport::audit(&[], &["E-001".to_string()]);
        assert_eq!(report.grounding_ratio, 0.0);
        assert!(!report.is_grounded(0.6));
    }

    // ── Schema ──
    #[test]
    fn schema_blocks_missing_field() {
        let checks = check_schema_fields(&["id", "result"], r#"{"id": 1}"#);
        assert_eq!(checks.len(), 2);
        assert!(checks.iter().any(|c| c.field == "result" && !c.present));
    }

    #[test]
    fn schema_passes_complete_json() {
        let checks = check_schema_fields(&["id", "result"], r#"{"id": 1, "result": "ok"}"#);
        assert!(checks.iter().all(|c| c.present));
    }

    #[test]
    fn schema_fails_on_non_object() {
        let checks = check_schema_fields(&["id"], "[1,2,3]");
        assert!(!checks[0].present);
    }

    // ── JudgePanel ──
    #[test]
    fn panel_passes_grounded_input() {
        let input = JudgeInput {
            candidate: "a concise grounded conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Pass, "{}", v.reasoning);
        assert!(v.is_pass());
        assert!(v.median_score >= 0.6);
    }

    #[test]
    fn panel_blocks_low_grounding() {
        let input = JudgeInput {
            candidate: "conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-999"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Block, "{}", v.reasoning);
    }

    #[test]
    fn panel_blocks_broken_trajectory() {
        let input = JudgeInput {
            candidate: "task".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: Some(simple_traj(1, "t", 0, 5)),
            grounding_failures: 3,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Block, "{}", v.reasoning);
    }

    #[test]
    fn family_separation_excludes_same_family_judge() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-001"])];
        input.evidence_ids = vec!["E-001".to_string()];
        input.producer_family = JudgeFamily::Heuristic;
        let panel = JudgePanel::default_panel();
        let v = panel.run(&input);
        assert_eq!(v.opinions.len(), 2, "同族证据法官应被排除");
        assert!(v
            .opinions
            .iter()
            .all(|o| o.family != JudgeFamily::Heuristic));
    }

    #[test]
    fn high_disagreement_routes_to_human() {
        let mut panel = JudgePanel::default_panel();
        panel.debias.agreement_review_threshold = 0.99;
        let input = JudgeInput {
            candidate: "a moderately long but grounded conclusion about caching layers".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let v = panel.run(&input);
        assert_eq!(v.verdict, Verdict::Review, "{}", v.reasoning);
        assert!(v.routed_to_human);
    }

    #[test]
    fn ensemble_pass_k_requires_all_runs() {
        let input = JudgeInput {
            candidate: "concise grounded conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 5, 5);
        assert!(ens.passed, "pass^5 应全部通过: {}", ens.last.reasoning);
        assert_eq!(ens.passes, 5);
    }

    // ── Guardrail ──
    #[test]
    fn guardrail_rejects_schema_failure() {
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck {
            field: "result".to_string(),
            present: false,
            detail: "missing".to_string(),
        }];
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Reject);
    }

    #[test]
    fn guardrail_rejects_low_grounding() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-999"])];
        input.evidence_ids = vec!["E-001".to_string()];
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Reject);
    }

    #[test]
    fn guardrail_quarantines_fabrications() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![
            Claim::new("c1", &["E-001"]),
            Claim::new("c2", &["E-001"]),
            Claim::new("c3", &["E-999"]),
        ];
        input.evidence_ids = vec!["E-001".to_string(), "E-002".to_string()];
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Quarantine, "{}", g.reason);
        assert_eq!(g.faithfulness.quarantine().len(), 1);
    }

    #[test]
    fn guardrail_rejects_grounding_failures() {
        let mut input = JudgeInput::new("grounded");
        input.claims = vec![Claim::new("c1", &["E-001"])];
        input.evidence_ids = vec!["E-001".to_string()];
        input.grounding_failures = 2;
        let g = GuardrailReport::evaluate(&input, &DebiasConfig::default());
        assert_eq!(g.action, GuardAction::Reject);
    }

    // ── GateDecision ──
    #[test]
    fn gate_allows_autonomous_low_risk() {
        let input = JudgeInput {
            candidate: "low risk grounded change".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let tools = vec![
            ToolSpec::read_only("get"),
            ToolSpec::reversible("edit", "undo"),
        ];
        let d = GateDecision::decide(&tools, &input, &JudgePanel::default_panel());
        assert!(d.allows_autonomous(), "{}", d.reason);
    }

    #[test]
    fn gate_human_blocks_irreversible_even_if_high_score() {
        let input = JudgeInput {
            candidate: "high confidence grounded".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let tools = vec![ToolSpec::irreversible("send_email")];
        let d = GateDecision::decide(&tools, &input, &JudgePanel::default_panel());
        assert!(!d.allows_autonomous(), "{}", d.reason);
        assert_eq!(d.action, GuardAction::Escalate);
        assert_eq!(d.level, GateLevel::Human);
    }

    #[test]
    fn gate_reject_wins_over_confidence() {
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck {
            field: "id".to_string(),
            present: false,
            detail: "missing".to_string(),
        }];
        let tools = vec![ToolSpec::read_only("get")];
        let d = GateDecision::decide(&tools, &input, &JudgePanel::default_panel());
        assert_eq!(d.action, GuardAction::Reject);
        assert_eq!(d.verdict, Verdict::Block);
        assert!(!d.allows_autonomous());
    }

    // ── Calibration (真实 clean/broken 日志黄金集) ──
    #[test]
    fn calibration_clean_pass_broken_block() {
        let clean = GoldTrajectory {
            id: "clean-1".to_string(),
            label: TrajectoryLabel::Clean,
            trajectory: simple_traj(1, "t1", 5, 5),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            grounding_failures: 0,
            schema_failures: Vec::new(),
        };
        let broken = GoldTrajectory {
            id: "broken-1".to_string(),
            label: TrajectoryLabel::Broken,
            trajectory: simple_traj(2, "t2", 0, 5),
            claims: vec![Claim::new("c1", &["E-999"])],
            evidence_ids: vec!["E-001".to_string()],
            grounding_failures: 4,
            schema_failures: vec![SchemaCheck {
                field: "result".to_string(),
                present: false,
                detail: "missing".to_string(),
            }],
        };
        let set = CalibrationSet::new(vec![clean, broken]);
        let report = set.pass_k(&JudgePanel::default_panel(), 3, 3);
        assert_eq!(report.clean_recall, 1.0, "clean 应放行");
        assert_eq!(report.broken_precision, 1.0, "broken 应拦截");
        assert_eq!(report.balanced, 1.0);
    }

    // ── Debias: verbosity ──
    #[test]
    fn verbosity_penalty_reduces_score() {
        let cfg = DebiasConfig::default();
        assert_eq!(cfg.verbosity_penalty_for("short"), 0.0);
        let long = "x".repeat(cfg.verbosity_norm_len * 3);
        let penalty = cfg.verbosity_penalty_for(&long);
        assert!(penalty > 0.0);
        assert!(penalty <= cfg.verbosity_penalty_cap);
    }

    // ── Ensemble: 机械护栏前置压过聚合 ──
    #[test]
    fn ensemble_rejects_on_schema_failure() {
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck {
            field: "id".to_string(),
            present: false,
            detail: "missing".to_string(),
        }];
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 5, 3);
        assert_eq!(ens.verdict, Verdict::Block, "{}", ens.last.reasoning);
        assert!(!ens.passed);
    }

    #[test]
    fn ensemble_rejects_on_low_grounding() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![Claim::new("c1", &["E-999"])];
        input.evidence_ids = vec!["E-001".to_string()];
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 3, 3);
        assert_eq!(ens.verdict, Verdict::Block, "{}", ens.last.reasoning);
    }

    #[test]
    fn ensemble_quarantines_on_fabrication() {
        let mut input = JudgeInput::new("x");
        input.claims = vec![
            Claim::new("c1", &["E-001"]),
            Claim::new("c2", &["E-001"]),
            Claim::new("c3", &["E-999"]),
        ];
        input.evidence_ids = vec!["E-001".to_string(), "E-002".to_string()];
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 3, 3);
        assert_eq!(ens.verdict, Verdict::Review, "{}", ens.last.reasoning);
        assert!(!ens.passed);
    }

    #[test]
    fn ensemble_passes_clean_input() {
        let input = JudgeInput {
            candidate: "concise grounded conclusion".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let panel = JudgePanel::default_panel();
        let ens = panel.run_ensemble(&input, 5, 5);
        assert!(ens.passed, "pass^5 应全部通过: {}", ens.last.reasoning);
        assert_eq!(ens.verdict, Verdict::Pass);
    }

    // ── ToolRegistry + check_tool_call + check_path ──
    #[test]
    fn registry_unknown_tool_denied() {
        let reg = ToolRegistry::from_read_only(&["get", "query"]);
        assert!(reg.get("get").is_some());
        let (allowed, reason) = GateDecision::check_tool_call(
            "rm -rf",
            &reg,
            &JudgeInput::new("x"),
            &JudgePanel::default_panel(),
        );
        assert!(!allowed, "{}", reason);
    }

    #[test]
    fn registry_read_only_allowed_irreversible_denied() {
        let reg = ToolRegistry::new()
            .register(ToolSpec::read_only("get"))
            .register(ToolSpec::irreversible("send_email"));
        let panel = JudgePanel::default_panel();
        let input = JudgeInput::new("x");
        let (read_ok, _) = GateDecision::check_tool_call("get", &reg, &input, &panel);
        assert!(read_ok);
        let (email_ok, reason) = GateDecision::check_tool_call("send_email", &reg, &input, &panel);
        assert!(!email_ok, "{}", reason);
    }

    #[test]
    fn check_path_read_only_autonomous_irreversible_human() {
        let panel = JudgePanel::default_panel();
        let input = JudgeInput {
            candidate: "concise grounded".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let ro_reg = ToolRegistry::from_read_only(&["get"]);
        let ro = GateDecision::check_path(&ro_reg.cloned_specs(), &input, &panel);
        assert!(ro.allows_autonomous(), "{}", ro.reason);
        let irr = GateDecision::decide(&[ToolSpec::irreversible("send_email")], &input, &panel);
        assert!(!irr.allows_autonomous());
        assert_eq!(irr.level, GateLevel::Human);
    }

    // ── LLM 法官 (异步评审路径) ──
    #[tokio::test]
    async fn llm_judge_scores_good_provider() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider {
            score: 4.0,
            confidence: 0.9,
        });
        let judge =
            LLMJudgeAdapter::new("llm-good", JudgeFamily::Heuristic, provider, "mock-model");
        let input = JudgeInput::new("test");
        let op = judge.score(&input).await;
        assert!(
            (op.raw_score - 1.0).abs() < 0.01,
            "4/4 → 1.0, got {}",
            op.raw_score
        );
        assert!((op.confidence - 0.9).abs() < 0.01);
        assert!(op.attribution_tags.iter().any(|t| t == "llm_provider"));
    }

    #[tokio::test]
    async fn llm_judge_low_score_drags_panel_to_review() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider {
            score: 1.0,
            confidence: 0.9,
        });
        let judge = LLMJudgeAdapter::new("llm-bad", JudgeFamily::Heuristic, provider, "mock-model");
        let input = JudgeInput {
            candidate: "concise grounded".to_string(),
            claims: vec![Claim::new("c1", &["E-001"])],
            evidence_ids: vec!["E-001".to_string()],
            trajectory: None,
            grounding_failures: 0,
            schema_failures: Vec::new(),
            producer_family: JudgeFamily::None,
            attestation: None,
            rubric: None,
            samples: 1,
        };
        let panel = JudgePanel::default_panel();
        let verdict = panel.run_async(&input, &[&judge]).await;
        assert!(
            !verdict.is_pass(),
            "LLM 低分应阻止放行: {}",
            verdict.reasoning
        );
    }

    #[tokio::test]
    async fn llm_judge_guardrail_still_blocks_schema_failure() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider {
            score: 4.0,
            confidence: 0.9,
        });
        let judge =
            LLMJudgeAdapter::new("llm-good", JudgeFamily::Heuristic, provider, "mock-model");
        let mut input = JudgeInput::new("x");
        input.schema_failures = vec![SchemaCheck {
            field: "id".to_string(),
            present: false,
            detail: "missing".to_string(),
        }];
        let panel = JudgePanel::default_panel();
        let verdict = panel.run_async(&input, &[&judge]).await;
        assert_eq!(
            verdict.verdict,
            Verdict::Block,
            "机械检查压过 LLM 分数: {}",
            verdict.reasoning
        );
    }

    #[tokio::test]
    async fn judge_registry_builds_async_judges() {
        let p1: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider {
            score: 3.0,
            confidence: 0.8,
        });
        let p2: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider {
            score: 4.0,
            confidence: 0.85,
        });
        let registry = JudgeRegistry::new()
            .register(JudgeFamily::Analytic, p1, "model-a")
            .register(JudgeFamily::Symbolic, p2, "model-b");
        assert_eq!(registry.entries().len(), 2);
        let judges = registry.build_async_judges();
        assert_eq!(judges.len(), 2);
        assert!(judges.iter().all(|j| j.family() != JudgeFamily::None));
    }

    // ── Replica 式 rubric judge (arXiv 2608.13331 吸收) ──

    /// 测试用 rubric mock LLM — 返回五维 criteria JSON (0..1)。
    struct MockRubricProvider {
        score: f64,
        dim: f64,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockRubricProvider {
    fn data_trust(&self) -> crate::core::nt_core_llm::DataTrust {
        crate::core::nt_core_llm::DataTrust::Trusted
    }

        async fn complete_raw(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
            let content = format!(
                r#"{{"score":{}, "confidence":0.8, "rationale":"rubric judge",
                    "criteria":[
                      {{"name":"claim_support","score":{}, "rationale":"supports"}},
                      {{"name":"evidence_ground","score":{}, "rationale":"grounded"}},
                      {{"name":"mechanism","score":{}, "rationale":"real"}},
                      {{"name":"resource","score":{}, "rationale":"ok"}},
                      {{"name":"fidelity","score":{}, "rationale":"faithful"}}
                    ]}}"#,
                self.score, self.dim, self.dim, self.dim, self.dim, self.dim
            );
            Ok(LlmResponse {
                content,
                model: "mock-rubric".into(),
                usage: Usage::default(),
                finish_reason: FinishReason::Stop,
                tool_calls: None,
             reasoning: None,})
        }

        async fn stream_complete_raw(
            &self,
            _request: &LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
            let (_, rx) = tokio::sync::mpsc::channel(1);
            Ok(rx)
        }
    }

    #[tokio::test]
    async fn rubric_judge_parses_five_dimensions() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockRubricProvider {
            score: 0.8,
            dim: 0.7,
        });
        let judge =
            LLMJudgeAdapter::new("llm-rubric", JudgeFamily::Analytic, provider, "mock-rubric");
        let mut input = JudgeInput::new("candidate");
        input.rubric = Some(RubricSpec::default_five());
        let op = judge.score(&input).await;
        assert!(
            (op.raw_score - 0.8).abs() < 0.01,
            "rubric 综合分 0..1 直用: {}",
            op.raw_score
        );
        let names: Vec<&str> = op.criteria.iter().map(|c| c.name.as_str()).collect();
        for dim in [
            "llm_judge",
            "claim_support",
            "evidence_ground",
            "mechanism",
            "resource",
            "fidelity",
        ] {
            assert!(names.contains(&dim), "缺少维度 {}: {:?}", dim, names);
        }
        assert!(op.attribution_tags.iter().any(|t| t == "llm_provider"));
    }

    #[tokio::test]
    async fn rubric_judge_falls_back_without_spec() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockJudgeProvider {
            score: 4.0,
            confidence: 0.9,
        });
        let judge =
            LLMJudgeAdapter::new("llm-plain", JudgeFamily::Analytic, provider, "mock-model");
        let input = JudgeInput::new("no rubric");
        let op = judge.score(&input).await;
        assert!(
            (op.raw_score - 1.0).abs() < 0.01,
            "fallback 1-4 量表: {}",
            op.raw_score
        );
        assert!(
            op.criteria.iter().all(|c| c.name == "llm_judge"),
            "无 rubric 不产维度标准"
        );
    }

    #[tokio::test]
    async fn rubric_judge_multi_sample_aggregates_mean() {
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(MockRubricProvider {
            score: 0.8,
            dim: 0.7,
        });
        let judge =
            LLMJudgeAdapter::new("llm-sample", JudgeFamily::Analytic, provider, "mock-rubric")
                .with_samples(3);
        let mut input = JudgeInput::new("candidate");
        input.rubric = Some(RubricSpec::default_five());
        let op = judge.score(&input).await;
        assert!(
            (op.raw_score - 0.8).abs() < 0.01,
            "均值聚合: {}",
            op.raw_score
        );
        assert!(
            op.attribution_tags.iter().any(|t| t == "multi_sample_3"),
            "应标记多采样"
        );
        let cs = op.criteria.iter().filter(|c| c.name == "mechanism").count();
        assert_eq!(cs, 1, "维度标准应合并为均值单条, got {}", cs);
    }

    // ── 真实 LLM 端到端链路 (env-gated) ──
    // LLM7 codestral-latest 匿名可用 (2026-08 实测); 网络隔离需逃生门。
    // 运行: NT_E2E_LLM7=1 NEOTRIX_NETWORK_UNBLOCK=1 cargo test -p neotrix --lib nt_core_gate::tests::rubric_judge_llm7_live
    #[tokio::test]
    async fn rubric_judge_llm7_live() {
        if std::env::var("NT_E2E_LLM7")
            .map(|v| v == "1")
            .unwrap_or(false)
            != true
        {
            eprintln!("skipped: set NT_E2E_LLM7=1 to run live LLM7 e2e");
            return;
        }
        use crate::l1_action::nt_io::nt_io_provider::openai::OpenAiProvider;
        let mut provider = OpenAiProvider::new(String::new());
        provider = provider.with_base_url("https://api.llm7.io/v1");
        let provider: std::sync::Arc<dyn LlmProvider> = std::sync::Arc::new(provider);
        let judge = LLMJudgeAdapter::new(
            "llm7-live",
            JudgeFamily::Analytic,
            provider,
            "codestral-latest",
        );
        let mut input = JudgeInput::new("缓存分层方案将冷热数据分离");
        input.claims = vec![Claim::new("c1", &["E-001"])];
        input.evidence_ids = vec!["E-001".to_string()];
        input.rubric = Some(RubricSpec::default_five());
        let op = judge.score(&input).await;
        assert!(
            op.raw_score > 0.0 && op.raw_score <= 1.0,
            "真实 LLM rubric 综合分应在 0..1: {}",
            op.raw_score
        );
        assert!(
            op.confidence > 0.0,
            "真实 LLM 应有 confidence: {}",
            op.confidence
        );
        let names: Vec<&str> = op.criteria.iter().map(|c| c.name.as_str()).collect();
        for dim in [
            "llm_judge",
            "claim_support",
            "evidence_ground",
            "mechanism",
            "resource",
            "fidelity",
        ] {
            assert!(
                names.contains(&dim),
                "真实 LLM 应产出五维标准, 缺 {}: {:?}",
                dim,
                names
            );
        }
        assert!(
            !op.attribution_tags.iter().any(|t| t == "llm_parse_failed"),
            "真实 LLM 响应应可解析 (markdown 围栏已剥离): {:?}",
            op.attribution_tags
        );
    }

    // ── 真实经验 → 校准集 (experience-tree KB) ──
    #[test]
    fn from_kb_experience_labels_clean_and_broken() {
        let entries: Vec<(String, String)> = vec![
            ("branch_195_0_ab12cd".to_string(), r#"{"type":"defect","domain":"NT-CORE","content":"defect fixed","evidence":"E-001"}"#.to_string()),
            ("branch_195_1_ef34".to_string(), r#"{"type":"insight","domain":"NT-IO","content":"cleaned up UI","evidence":"E-002"}"#.to_string()),
            ("branch_196_0_9a01".to_string(), r#"{"type":"fail","domain":"NT-SHIELD","content":"rollback done","evidence":"E-003"}"#.to_string()),
            ("not_a_branch".to_string(), r#"{"type":"insight","content":"ignored"}"#.to_string()),
        ];
        let set = CalibrationSet::from_kb_experience(&entries);
        assert_eq!(set.gold.len(), 3, "非 branch_ 前缀应被跳过");
        assert_eq!(
            set.gold[0].label,
            TrajectoryLabel::Broken,
            "defect → Broken"
        );
        assert_eq!(set.gold[1].label, TrajectoryLabel::Clean, "insight → Clean");
        assert_eq!(set.gold[2].label, TrajectoryLabel::Broken, "fail → Broken");
        assert!(set.gold[0].trajectory.steps.len() >= 1);
    }

    #[test]
    fn calibration_from_kb_runs_to_completion() {
        let entries: Vec<(String, String)> = vec![
            (
                "branch_1_0_a".into(),
                r#"{"type":"insight","content":"clean grounded conclusion","evidence":"E-001"}"#
                    .to_string(),
            ),
            (
                "branch_1_1_b".into(),
                r#"{"type":"regression","content":"broken step zero","evidence":"E-900"}"#
                    .to_string(),
            ),
        ];
        let set = CalibrationSet::from_kb_experience(&entries);
        let panel = JudgePanel::default_panel();
        let report = set.pass_k(&panel, 3, 3);
        assert!(report.broken_total >= 1);
        assert!(report.clean_total >= 1);
        assert!(report.balanced > 0.0 || report.broken_precision == 1.0);
    }

    #[test]
    fn deliberate_converges_when_consensus() {
        // D6: 意见一致 (高分) → 辩论收敛, 无分歧
        let mut ops = Vec::new();
        for i in 0..3 {
            let mut op = JudgeOpinion::new(&format!("j{}", i), JudgeFamily::Analytic);
            op.debiased_score = 0.9;
            op.confidence = 0.8;
            ops.push(op);
        }
        let report = deliberate(&ops);
        assert!(report.converged, "consensus → converged");
        assert!(report.divergence < 0.1, "no divergence on consensus");
        assert!((report.converged_score - 0.9).abs() < 0.1);
        assert_eq!(report.rounds.len(), 3);
    }

    #[test]
    fn deliberate_pulls_toward_contrarian_on_split() {
        // D6: 意见分裂 (两高一分低) → 辩论向反方收敛, 分数被拉低
        let mut ops = Vec::new();
        for (i, s) in [0.9, 0.9, 0.1].iter().enumerate() {
            let mut op = JudgeOpinion::new(&format!("j{}", i), JudgeFamily::Symbolic);
            op.debiased_score = *s;
            op.confidence = 0.7;
            ops.push(op);
        }
        let report = deliberate(&ops);
        assert!(!report.converged, "split → not converged");
        assert!(report.divergence > 0.3, "split creates divergence");
        // 反方 0.1 显著拉低收敛分 (低于纯均值 0.633)
        assert!(
            report.converged_score < 0.6,
            "contrarian drags below mean, got {}",
            report.converged_score
        );
        // 反方角色被分配
        assert!(report.rounds.iter().any(|r| r.role == DebateRole::Con));
        assert!(report.rounds.iter().any(|r| r.role == DebateRole::Pro));
    }

    #[test]
    fn deliberate_empty_opinions_is_trivial() {
        let report = deliberate(&[]);
        assert!(report.converged);
        assert_eq!(report.converged_score, 0.0);
    }

    // ── Replica #2: 洞察事件检测 ──
    #[test]
    fn insight_detector_fires_on_breakthrough() {
        let mut d = InsightDetector::new(0.1);
        assert!(d.record(0.3, 1).is_none(), "首个样本仅建立基线, 不触发");
        let ev = d.record(0.6, 2).expect("越过阈值应触发");
        assert_eq!(ev.previous_best, 0.3);
        assert!((ev.strength - (0.6 - 0.3) / (1.0 - 0.3)).abs() < 1e-9);
        assert_eq!(ev.step, 2);
        assert_eq!(d.best(), 0.6);
    }

    #[test]
    fn insight_detector_saturates_and_resets() {
        let mut d = InsightDetector::new(0.1);
        d.record(0.95, 1); // 建立基线
                           // 接近饱和后小幅增长 → 强度低于阈值 → 不再触发
        assert!(d.record(0.952, 2).is_none(), "饱和后小步幅应静默");
        d.reset();
        assert_eq!(d.best(), 0.0);
        assert!(d.record(0.9, 3).is_none(), "reset 后首个样本重建基线");
        assert!(d.record(0.95, 4).is_some(), "重建基线后新突破触发");
    }

    #[test]
    fn insight_detector_serializes_across_sessions() {
        let mut d = InsightDetector::new(0.05);
        d.record(0.5, 1);
        let json = serde_json::to_string(&d).unwrap();
        let mut restored: InsightDetector = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.best(), 0.5);
        let ev = restored.record(0.7, 2).expect("恢复后继续追踪");
        assert_eq!(ev.previous_best, 0.5);
    }

    // ── Replica #9: 自证纪律 ──
    #[test]
    fn attestation_contradiction_halves_score() {
        let att = Attestation {
            real_slice: Some("slice-1".into()),
            budget_used: Some(100.0),
            budget_total: 100.0,
            claimed_seeds: Some(5),
        };
        // 声明预算 100 但观测只有 50 → 矛盾
        let p = att.contradiction_score(Some(50.0), Some(5));
        assert!(p >= 0.5, "预算矛盾应重罚, got {}", p);
        // 声明种子 5 但观测 3 → 矛盾
        let p2 = att.contradiction_score(Some(100.0), Some(3));
        assert!(p2 >= 0.5, "种子矛盾应重罚, got {}", p2);
        // 完全一致 → 无扣分
        let p3 = att.contradiction_score(Some(100.0), Some(5));
        assert!(p3 < 0.1, "一致应接近无扣分, got {}", p3);
    }

    #[test]
    fn attestation_missing_self_report_is_light_penalty() {
        let att = Attestation::new(100.0);
        let p = att.contradiction_score(None, None);
        assert!(p >= 0.1 && p < 0.5, "未自证应轻微扣分, got {}", p);
    }
