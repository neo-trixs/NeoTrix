//! 评测单测 (纯搬移自门面 tests，其余导入在文件内显式补齐)。

use super::nt_budget::DEFAULT_BUDGET_GRID;
use super::nt_compliance::{
    ap_acc_score, ComplianceGate, InstructionPlane, PlaneConflictCase, WithholdingResult,
};
use super::nt_harness::EvalHarness;
use super::nt_pareto::{hda_attribution, HdaComponent};
use super::nt_regression::SmallScaleMethod;
use super::nt_types::EvalPoint;
use super::nt_verify_oracle::{
    verify_constraint, verify_deterministic, verify_extractable, verify_unified,
    verify_unified_batch, OracleLadder, OracleRung, RewardSignal, RungResult, UnifiedVerifyRequest,
    VerificationChannel,
};
use crate::l1_action::nt_io::nt_io_provider::{LlmError, LlmProvider, LlmRequest, LlmResponse};
use crate::l6_meta::healing::nt_core_self_test::SelfTest;
use std::sync::Arc;

#[test]
fn test_interpolate_quality() {
    let points = vec![
        EvalPoint {
            model_name: "m".into(),
            budget: 1024,
            query_id: "q1".into(),
            response: "".into(),
            actual_tokens: 100,
            quality_score: 0.5,
            judge_justification: "".into(),
            latency_ms: 100,
            cost_usd: 0.0,
            consciousness_phi: None,
            consciousness_level: None,
        },
        EvalPoint {
            model_name: "m".into(),
            budget: 4096,
            query_id: "q1".into(),
            response: "".into(),
            actual_tokens: 400,
            quality_score: 0.8,
            judge_justification: "".into(),
            latency_ms: 200,
            cost_usd: 0.0,
            consciousness_phi: None,
            consciousness_level: None,
        },
    ];
    let grid = vec![1024, 2048, 4096];
    let interp = EvalHarness::interpolate_quality(&points, &grid);
    assert_eq!(interp[0], 0.5);
    assert!((interp[1] - 0.6).abs() < 0.01); // 线性插值: 2048 位于 1024→4096 的 1/3 处
    assert_eq!(interp[2], 0.8);
}

#[test]
fn test_budget_grid_constants() {
    assert_eq!(DEFAULT_BUDGET_GRID.len(), 8);
    assert_eq!(DEFAULT_BUDGET_GRID[0], 0); // Low: 无思考
    assert_eq!(DEFAULT_BUDGET_GRID[3], 2048); // Medium
    assert_eq!(DEFAULT_BUDGET_GRID[6], 16384); // Max
}

struct DummyProvider;

#[async_trait::async_trait]
impl LlmProvider for DummyProvider {
fn set_proxy(&mut self, _proxy_url: &str) {}
fn data_trust(&self) -> crate::l1_action::nt_core_llm::DataTrust {
    crate::l1_action::nt_core_llm::DataTrust::Trusted
}

    async fn complete_raw(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        unreachable!("DummyProvider::complete should not be called")
    }
    async fn stream_complete_raw(
        &self,
        _request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        unreachable!("DummyProvider::stream_complete should not be called")
    }
}

fn harness() -> EvalHarness {
    let provider: Arc<dyn LlmProvider> = Arc::new(DummyProvider);
    EvalHarness::new_default(vec![], vec![], provider, "judge".into())
}

#[test]
fn test_ap_acc_score() {
    assert!((ap_acc_score(0.3, 0.8) - 0.5).abs() < 1e-9); // aided > plain → delta
    assert_eq!(ap_acc_score(0.8, 0.3), 0.0); // plain > aided → 0
    assert_eq!(ap_acc_score(0.5, 0.5), 0.0); // 相等 → 0
}

#[test]
fn test_ap_acc_is_meaningful_threshold() {
    assert!(WithholdingResult { plain_pass: 0.3, aided_pass: 0.8, samples: 10 }._is_meaningful());
    assert!(!WithholdingResult { plain_pass: 0.3, aided_pass: 0.34, samples: 10 }._is_meaningful()); // < ε
    assert!(!WithholdingResult { plain_pass: 0.8, aided_pass: 0.5, samples: 10 }._is_meaningful()); // 负增量
}

#[test]
fn test_plane_conflict_conforms() {
    let conforming = PlaneConflictCase {
        higher: InstructionPlane { rank: 1, name: "system", content: "h".into() },
        lower: InstructionPlane { rank: 4, name: "tool", content: "l".into() },
        higher_instruction: "follow system".into(),
        lower_instruction: "follow tool".into(),
        model_followed_higher: true,
    };
    assert!(conforming._conforms());

    let nonconforming = PlaneConflictCase {
        model_followed_higher: false,
        ..conforming
    };
    assert!(!nonconforming._conforms());
}

#[test]
fn test_withholding_result_ap_acc() {
    let r = WithholdingResult { plain_pass: 0.2, aided_pass: 0.7, samples: 20 };
    assert!((r.ap_acc() - 0.5).abs() < 1e-9);
}

#[test]
fn test_compliance_gate_passes() {
    let mut gate = ComplianceGate::new();
    gate.record_withholding(0.3, 0.8, 10); // ap_acc = 0.5
    gate.record_withholding(0.4, 0.9, 10); // ap_acc = 0.5
    assert!((gate.mean_ap_acc() - 0.5).abs() < 1e-9);
    for i in 0..5 {
        // 5 条冲突用例, 4 条遵循更高平面 → conformance 0.8
        gate._record_conflict(
            InstructionPlane { rank: 1, name: "system", content: "h".into() },
            InstructionPlane { rank: 3, name: "user", content: "l".into() },
            "follow system".into(),
            "follow user".into(),
            i != 4,
        );
    }
    assert!((gate._plane_conformance() - 0.8).abs() < 1e-9);
    assert!(gate.passes());
}

#[test]
fn test_compliance_gate_fails_below_threshold() {
    // 低 AP-Acc: mean_ap_acc = 0 < gate
    let mut gate = ComplianceGate::new();
    gate.record_withholding(0.8, 0.8, 10);
    gate._record_conflict(
        InstructionPlane { rank: 2, name: "project", content: "h".into() },
        InstructionPlane { rank: 4, name: "skill", content: "l".into() },
        "follow project".into(),
        "follow skill".into(),
        true,
    );
    assert!(!gate.passes());

    // 低 conformance: ap_acc 达标但 conformance < 0.8
    let mut gate2 = ComplianceGate::new();
    gate2.record_withholding(0.2, 0.9, 10); // ap_acc = 0.7 >= 0.5
    gate2._record_conflict(
        InstructionPlane { rank: 1, name: "system", content: "h".into() },
        InstructionPlane { rank: 4, name: "tool", content: "l".into() },
        "a".into(),
        "b".into(),
        true,
    );
    gate2._record_conflict(
        InstructionPlane { rank: 1, name: "system", content: "h".into() },
        InstructionPlane { rank: 4, name: "tool", content: "l".into() },
        "a".into(),
        "b".into(),
        false,
    );
    gate2._record_conflict(
        InstructionPlane { rank: 1, name: "system", content: "h".into() },
        InstructionPlane { rank: 4, name: "tool", content: "l".into() },
        "a".into(),
        "b".into(),
        false,
    );
    assert!((gate2._plane_conformance() - (1.0 / 3.0)).abs() < 1e-9);
    assert!(!gate2.passes());
}

#[test]
fn test_default_planes_priority_order() {
    let planes = InstructionPlane::_default_planes();
    assert_eq!(planes.len(), 5);
    assert_eq!(planes[0].name, "system");
    assert_eq!(planes[0].rank, 1);
    assert_eq!(planes[1].name, "project");
    assert_eq!(planes[1].rank, 2);
    assert_eq!(planes[2].name, "user");
    assert_eq!(planes[2].rank, 3);
    // Tool/Skill 共享最低 rank
    assert!(planes[3].rank == 4 && planes[4].rank == 4);
}

#[test]
fn test_ap_acc_matrix_length() {
    let h = harness();
    assert_eq!(h._ap_acc_matrix(&[0.3, 0.5], &[0.8, 0.6]).len(), h.budget_grid.len());
    assert_eq!(h._ap_acc_matrix(&[], &[]).len(), h.budget_grid.len()); // 越界按 0.0 补齐
}

#[test]
fn test_ap_acc_matrix_values() {
    let h = harness();
    let n = h.budget_grid.len();
    let plain: Vec<f64> = (0..n).map(|i| 0.3 + 0.1 * i as f64).collect();
    let aided: Vec<f64> = (0..n).map(|i| plain[i] + 0.2).collect();
    let matrix = h._ap_acc_matrix(&plain, &aided);
    for (i, v) in matrix.iter().enumerate() {
        assert!((v - 0.2).abs() < 1e-9, "idx {i}");
    }
    // aided < plain → 0
    let regress = h._ap_acc_matrix(&[0.9, 0.9], &[0.4, 0.2]);
    assert_eq!(regress[0], 0.0);
    assert_eq!(regress[1], 0.0);
}

#[test]
fn test_with_gate_overrides_default() {
    assert!((harness().compliance.gate - 0.5).abs() < 1e-9); // 默认 0.5
    let h = harness()._with_gate(0.8);
    assert!((h.compliance.gate - 0.8).abs() < 1e-9);
}

#[test]
fn test_harness_record_withholding_and_report() {
    let mut h = harness();
    h.record_withholding(0.3, 0.8, 10);
    h.record_withholding(0.4, 0.6, 10);
    assert_eq!(h.compliance.ap_results.len(), 2);
    let (conformance, mean_ap_acc, passes) = h.compliance_report();
    assert!((mean_ap_acc - 0.35).abs() < 1e-9);
    assert_eq!(conformance, 0.0); // 无冲突用例 → 0
    assert!(!passes); // 无冲突用例使 conformance < 0.8
}

// ── P2 HdaAttribution ──
#[test]
fn test_hda_attribution_complete() {
    let report = hda_attribution(0.3, 0.1, 0.05, 0.45);
    assert!(report.is_complete());
    assert_eq!(report.attributions.len(), 3);
}

#[test]
fn test_hda_top_orders_by_delta() {
    let report = hda_attribution(0.5, 0.2, 0.1, 0.8);
    let top = report.top(2);
    assert_eq!(top[0].component, HdaComponent::Tuned);
    assert_eq!(top[1].component, HdaComponent::Open);
}

#[test]
fn test_hda_incomplete_when_sum_mismatch() {
    let report = hda_attribution(0.3, 0.1, 0.05, 0.99);
    assert!(!report.is_complete());
}

#[test]
fn test_hda_confidence_sanity() {
    let report = hda_attribution(0.0, 0.0, 0.0, 0.0);
    for a in &report.attributions {
        assert!(a.confidence > 0.0 && a.confidence <= 1.0);
    }
}

// ── P9 SelfVerifiableReward ──
#[test]
fn test_reward_deterministic() {
    let r = verify_deterministic("42", "42");
    assert!(r.verifiable);
    assert_eq!(r.score, 1.0);
    let r2 = verify_deterministic("42", "43");
    assert_eq!(r2.score, 0.0);
}

#[test]
fn test_reward_extractable() {
    let r = verify_extractable("cherry", "the answer is cherry on top");
    assert_eq!(r.score, 1.0);
    let r2 = verify_extractable("cherry", "the answer is orange");
    assert_eq!(r2.score, 0.0);
}

#[test]
fn test_reward_constraint_rejects_forbidden() {
    let r = verify_constraint("no-pii", "contact: alice@example.com", &["@example.com", "alice"]);
    assert_eq!(r.score, 0.0);
    let r2 = verify_constraint("no-pii", "all clear", &["@example.com"]);
    assert_eq!(r2.score, 1.0);
}

#[test]
fn test_reward_gated_total() {
    let signal = RewardSignal {
        rewards: vec![
            verify_deterministic("a", "a"),
            verify_constraint("c", "ok", &[]),
        ],
    };
    assert_eq!(signal._gated_total(), Some(2.0));
    let empty = RewardSignal {
        rewards: vec![verify_constraint("c", "", &[])],
    };
    assert_eq!(empty._gated_total(), None);
}

// ── llm-as-a-verifier 统一验证框架 ──
#[test]
fn test_unified_verify_dispatches_by_modality() {
    let d = verify_unified(&UnifiedVerifyRequest {
        modality: "deterministic".into(),
        actual: "42".into(),
        expected: Some("42".into()),
        extractable: None,
        policy: None,
        forbidden: vec![],
    });
    assert_eq!(d.channel, VerificationChannel::Deterministic);
    assert!(d.passed());

    let j = verify_unified(&UnifiedVerifyRequest {
        modality: "json".into(),
        actual: r#"{"answer": 42}"#.into(),
        expected: None,
        extractable: Some("answer".into()),
        policy: None,
        forbidden: vec![],
    });
    assert_eq!(j.channel, VerificationChannel::Extractable);
    assert!(j.passed());

    let p = verify_unified(&UnifiedVerifyRequest {
        modality: "policy".into(),
        actual: "user@example.com".into(),
        expected: None,
        extractable: None,
        policy: Some("no-pii".into()),
        forbidden: vec!["@example.com".into()],
    });
    assert_eq!(p.channel, VerificationChannel::Constraint);
    assert_eq!(p.score, 0.0);
    assert!(!p.passed());
}

#[test]
fn test_unified_verify_unknown_modality_and_batch() {
    let bad = verify_unified(&UnifiedVerifyRequest {
        modality: "audio".into(),
        actual: "x".into(),
        expected: None,
        extractable: None,
        policy: None,
        forbidden: vec![],
    });
    assert!(!bad.verifiable);
    assert_eq!(bad.score, 0.0);

    let reqs = vec![
        UnifiedVerifyRequest {
            modality: "deterministic".into(),
            actual: "ok".into(),
            expected: Some("ok".into()),
            extractable: None,
            policy: None,
            forbidden: vec![],
        },
        UnifiedVerifyRequest {
            modality: "policy".into(),
            actual: "clean".into(),
            expected: None,
            extractable: None,
            policy: Some("p".into()),
            forbidden: vec!["bad".into()],
        },
    ];
    let (results, all) = verify_unified_batch(&reqs);
    assert!(all, "所有通道通过 → 整体通过");
    assert_eq!(results.len(), 2);

    let failing = vec![
        reqs[0].clone(),
        UnifiedVerifyRequest {
            modality: "policy".into(),
            actual: "contains bad".into(),
            expected: None,
            extractable: None,
            policy: Some("p".into()),
            forbidden: vec!["bad".into()],
        },
    ];
    let (_, all2) = verify_unified_batch(&failing);
    assert!(!all2, "任一通道失败 → 整体失败");
}

#[test]
fn test_reward_selftest_passes() {
    assert!(harness().self_test().is_ok());
}

// ── P10 SmallScaleMethod ──
#[test]
fn test_small_scale_sensitivity_decreases_with_scale() {
    let method = SmallScaleMethod::new(1.0, 0.5, 32);
    assert!(
        method._sensitivity_at_scale(1e9) < method._sensitivity_at_scale(1e3),
        "sensitivity must fade with scale"
    );
    assert!(method._sensitivity_at_scale(1e3) < method._sensitivity_at_scale(10.0));
    assert!(method._sensitivity_at_scale(1e9) > 0.0);
}

#[test]
fn test_small_scale_effective_dimensions_monotonic() {
    let method = SmallScaleMethod::new(1.0, 0.5, 32);
    let dims: Vec<usize> = vec![1e2, 1e3, 1e5, 1e7, 1e9]
        .into_iter()
        .map(|s| method._effective_dimensions(s))
        .collect();
    for w in dims.windows(2) {
        assert!(w[0] >= w[1], "effective dims must monotonically decrease: {dims:?}");
    }
    // 下限 1 维
    assert_eq!(method._effective_dimensions(1e12), 1);
}

#[test]
fn test_small_scale_warn_small_scale_flags() {
    let method = SmallScaleMethod::new(1.0, 0.5, 32);
    let small = method._warn_small_scale(1e5);
    assert!(small.needs_full_tuning);
    assert!(small.reason.contains("fully-tuned frontier"));
    let big = method._warn_small_scale(1e9);
    assert!(!big.needs_full_tuning);
}

#[test]
fn test_small_scale_tuned_frontier_threshold() {
    let method = SmallScaleMethod::new(1.0, 0.5, 32);
    assert!(method._tuned_frontier_reached(0.001, 0.01));
    assert!(!method._tuned_frontier_reached(0.05, 0.01));
    assert!(method._tuned_frontier_reached(0.01, 0.01)); // 边界: <= 视为达到
}

#[test]
fn test_small_scale_hyperparam_rank_ordering() {
    let method = SmallScaleMethod::new(1.0, 0.5, 32);
    let low = method._hyperparam_rank(0.2, 10);
    let mid = method._hyperparam_rank(0.5, 20);
    let high = method._hyperparam_rank(1.0, 50);
    assert!(low < mid);
    assert!(mid < high);
    // 高方差 > 低方差 (同维度)
    assert!(method._hyperparam_rank(0.9, 20) > method._hyperparam_rank(0.1, 20));
    // 高维度 > 低维度 (同方差)
    assert!(method._hyperparam_rank(0.5, 40) > method._hyperparam_rank(0.5, 10));
}

#[test]
fn test_small_scale_default_and_selftest() {
    let d = SmallScaleMethod::default();
    assert_eq!(d.sensitivity_decay_rate, 1.0);
    assert_eq!(d.dimension_decay_rate, 0.5);
    assert_eq!(d.tuning_budget, 32);
    assert!(!d.full_tuning_frontier);
    let method = SmallScaleMethod::new(1.0, 0.5, 32);
    assert!(method.self_test().is_ok());
}

// ── P11 OracleLadder (defending-harness F5) ──

#[test]
fn test_ladder_stops_at_first_failing_rung() {
    let ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
        .with_oracle(OracleRung::T1Repro, || RungResult::fail(OracleRung::T1Repro, "poc still crashes"))
        .with_oracle(OracleRung::T2Regression, || RungResult::pass(OracleRung::T2Regression, "regression ok"));
    let report = ladder.run();
    assert!(!report.promoted);
    assert_eq!(report.highest_passed, Some(OracleRung::T0BuildCheck));
    assert_eq!(report.results.len(), 2, "停在第一个失败级");
}

#[test]
fn test_all_oracles_pass_promotes_to_t3() {
    let ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
        .with_oracle(OracleRung::T1Repro, || RungResult::pass(OracleRung::T1Repro, "poc no longer crashes"))
        .with_oracle(OracleRung::T2Regression, || RungResult::pass(OracleRung::T2Regression, "suite passes"))
        .with_oracle(OracleRung::T3Reattack, || RungResult::pass(OracleRung::T3Reattack, "survives re-attack"));
    let report = ladder.run();
    assert!(report.promoted, "全部 oracle 通过 → 提升到 T3");
    assert_eq!(report.highest_passed, Some(OracleRung::T3Reattack));
    assert_eq!(report.results.len(), 4);
}

#[test]
fn test_per_rung_result_recorded() {
    let ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "exit 0"))
        .with_oracle(OracleRung::T1Repro, || RungResult::fail(OracleRung::T1Repro, "AddressSanitizer: heap-buffer-overflow"));
    let report = ladder.run();
    let t0 = report._rung_result(OracleRung::T0BuildCheck).unwrap();
    assert!(t0.passed && t0.evidence == "exit 0");
    let t1 = report._rung_result(OracleRung::T1Repro).unwrap();
    assert!(!t1.passed && t1.evidence.contains("AddressSanitizer"));
    assert_eq!(report.results.len(), 2);
}

#[test]
fn test_t1_failure_does_not_run_t2() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let t2_ran = Arc::new(AtomicBool::new(false));
    let t2_flag = t2_ran.clone();
    let ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
        .with_oracle(OracleRung::T1Repro, || RungResult::fail(OracleRung::T1Repro, "poc crashes"))
        .with_oracle(OracleRung::T2Regression, move || {
            t2_flag.store(true, Ordering::SeqCst);
            RungResult::pass(OracleRung::T2Regression, "should not run")
        });
    let report = ladder.run();
    assert!(!t2_ran.load(Ordering::SeqCst), "T1 失败时 T2 oracle 不得执行");
    assert_eq!(report.results.len(), 2);
    assert_eq!(report.highest_passed, Some(OracleRung::T0BuildCheck));
}

#[test]
fn test_ladder_missing_rung_does_not_promote() {
    let ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
        .with_oracle(OracleRung::T1Repro, || RungResult::pass(OracleRung::T1Repro, "poc clean"));
    // T2/T3 未注册 → 不提升
    let report = ladder.run();
    assert!(!report.promoted);
    assert_eq!(report.highest_passed, Some(OracleRung::T1Repro));
}

// ── C5 OracleLadder 自愈 ──
#[test]
fn test_ladder_valid_when_contiguous() {
    let ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
        .with_oracle(OracleRung::T1Repro, || RungResult::pass(OracleRung::T1Repro, "repro clean"))
        .with_oracle(OracleRung::T2Regression, || RungResult::pass(OracleRung::T2Regression, "regression ok"));
    assert!(ladder.is_valid(), "T0→T2 连续前缀必须有效");
}

#[test]
fn test_ladder_holereset_to_t0() {
    let mut ladder = OracleLadder::new()
        .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
        .with_oracle(OracleRung::T2Regression, || RungResult::pass(OracleRung::T2Regression, "regression ok"));
    assert!(!ladder.is_valid(), "T2 注册而 T1 缺失 → 空洞阶梯");
    let actions = ladder.reset_to_t0();
    assert!(!actions.is_empty());
    assert!(ladder.is_valid(), "重置后必须回到有效基态");
    assert!(!ladder.has(OracleRung::T0BuildCheck), "重置后不应残留任何 oracle");
}
