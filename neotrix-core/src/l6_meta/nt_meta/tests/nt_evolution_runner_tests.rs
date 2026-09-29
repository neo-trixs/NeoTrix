//! `nt_evolution_runner` 单测。
//!
//! ⛔ **本文件是 runner 的非空门证明。**
//! 上一轮 `nt_evolution_eval` 754 行 + 19 测试**零消费者** ——
//! 「存在但不起作用」比不存在更危险。本文件确保 runner **真的会拒绝**。
#![cfg(test)]

use crate::l6_meta::nt_meta::nt_evolution_eval::nt_evolution_eval::{
    Arm, CaseOutcome, EnvFingerprint, Preregistration,
};
use crate::l6_meta::nt_meta::nt_evolution_runner::ExperimentRunner;
use std::cell::Cell;

fn env() -> EnvFingerprint {
    EnvFingerprint::new("head1", Vec::new(), "lock1")
}

fn prereg() -> Preregistration {
    Preregistration::new(
        "把记忆去重路径补回来会减少重复写入",
        "若 case-2 上通过率不提升，则假设被削弱",
        "abc1234",
        "test-model",
        0.01,
    )
}

/// 构造一个确定性的「跑一次」：baseline 全过、candidate 全过
/// （用于测「无差异 ⇒ 不得判改进」）。
fn same_run(_arm: Arm, _round: u32) -> Vec<CaseOutcome> {
    vec![CaseOutcome::pass("c1", Arm::Candidate)]
}

/// baseline 挂、candidate 过 ⇒ 真实大幅提升。
fn improving_run(arm: Arm, _round: u32) -> Vec<CaseOutcome> {
    match arm {
        Arm::Baseline => vec![CaseOutcome::fail(
            "c1".to_string(),
            Arm::Baseline,
            vec!["missing".to_string()],
            Vec::new(),
        )],
        Arm::Candidate => vec![CaseOutcome::pass("c1", Arm::Candidate)],
    }
}

// ── 核心行为 ─────────────────────────────────────────────────────────

#[test]
fn rejects_when_no_falsifier_and_does_not_burn_runs() {
    // ⛔ 预注册缺 falsifier ⇒ 必须在**跑任何东西之前**就拒。
    //    用计数 closure 证明「一次都没跑」。
    // ⛔ 用 `Cell` 而非 `let mut runs`：`RunOnce` 的约束是 `Fn`
    //    （不是 `FnMut`），可变捕获会编译失败。`Cell` 是内部可变的零成本解法。
    let runs = Cell::new(0u32);
    let counted = |_arm: Arm, _r: u32| -> Vec<CaseOutcome> {
        runs.set(runs.get() + 1);
        vec![CaseOutcome::pass("c1", Arm::Candidate)]
    };
    let runner = ExperimentRunner::new();
    let bad = Preregistration::new("h", "", "c", "m", 0.01);
    let out = runner.run_experiment("s", &bad, &env(), &env(), &counted);

    assert!(!out.verdict.accept, "无 falsifier 不得接受");
    assert_eq!(runs.get(), 0, "预注册不合格时**一次都不该跑**：{}", runs.get());
    assert_eq!(out.verdict.vetoes.iter().any(|v| v.as_str() == "no_falsifier"), true);
}

#[test]
fn no_improvement_is_not_an_improvement() {
    // ⛔ 核心纪律：delta=0 **不是**改进。
    //    这条在 nt_evolution_eval 的测试里已钉过一次，
    //    在 runner 层再钉一次 —— 因为 runner 是对外的实际入口。
    let runner = ExperimentRunner::new();
    let out = runner.run_experiment("s", &prereg(), &env(), &env(), &same_run);
    assert!(
        !out.verdict.accept,
        "两臂无差异不得接受：{}",
        out.verdict.reason
    );
}

#[test]
fn real_improvement_with_narrow_floor_is_accepted() {
    // baseline 全挂 → candidate 全过 ⇒ delta = +1.0。
    // ⛔ 但 repeats 的地板是「baseline 全挂」⇒ std_dev = 0 ⇒ 3σ = 0。
    //    delta=1.0 > 0 且 ≥ min_effect(0.01) ⇒ 接受。
    let runner = ExperimentRunner::new();
    let out = runner.run_experiment("s", &prereg(), &env(), &env(), &improving_run);
    assert!(
        out.verdict.accept,
        "真实大幅提升必须被接受：{}",
        out.verdict.reason
    );
    assert!(out.floor.is_some(), "必须有噪声地板（跑过 repeats）");
}

#[test]
fn environment_mismatch_blocks_comparison() {
    // 两臂不同环境 ⇒ 结论不可比。
    let dirty = EnvFingerprint::new("head2", vec!["a.rs".into()], "lock1");
    let runner = ExperimentRunner::new();
    let out = runner.run_experiment("s", &prereg(), &env(), &dirty, &improving_run);
    assert!(!out.verdict.accept, "不同环境的结论不可比");
    assert_eq!(out.verdict.vetoes.iter().any(|v| v.as_str() == "environment_mismatch"), true);
}

// ── 账本累积 ─────────────────────────────────────────────────────────

#[test]
fn ledger_accumulates_and_exposes_self_refutation_rate() {
    // ⛔ 「被跑了」不等于「被记了」。账本必须累积，
    //    否则 self_refutation_rate 永远返回 0（看起来很健康）。
    let runner = ExperimentRunner::new();
    // 第一次：无 falsifier ⇒ 必拒
    let bad = Preregistration::new("h", "", "c", "m", 0.01);
    runner.run_experiment("T1", &bad, &env(), &env(), &same_run);
    // 第二次：真实提升 ⇒ 接受
    runner.run_experiment("T1", &prereg(), &env(), &env(), &improving_run);

    let ledger = runner.ledger_snapshot();
    assert_eq!(ledger.len(), 2, "正负都必须入账");
    let rate = runner.self_refutation_rate("T1");
    assert!((rate - 0.5).abs() < 1e-9, "自证失败率应 0.5，实际 {}", rate);
}

#[test]
fn unknown_subject_refutation_rate_is_zero() {
    let runner = ExperimentRunner::new();
    assert_eq!(runner.self_refutation_rate("never-ran"), 0.0);
}

// ── 配置护栏 ─────────────────────────────────────────────────────────

#[test]
fn repeats_below_two_is_clamped() {
    // ⛔ repeats=1 ⇒ 地板为 None ⇒ 判决必被 insufficient_evidence 否决。
    //    那是对的，但让调用方以为「配了 1 轮」更糟 ⇒ 夹到 2。
    let runner = ExperimentRunner::new().with_repeats(1);
    let calls = Cell::new(0u32);
    let counted = |arm: Arm, _r: u32| -> Vec<CaseOutcome> {
        calls.set(calls.get() + 1);
        improving_run(arm, 0)
    };
    let out = runner.run_experiment("s", &prereg(), &env(), &env(), &counted);
    assert!(out.repeats >= 2, "repeats 必须被夹到 ≥2，实际 {}", out.repeats);
    // 2 轮地板 + 1 轮 baseline + 1 轮 candidate = 至少 4 次
    assert!(calls.get() >= 4, "至少跑 4 次（2 地板 + 2 对照），实际 {}", calls.get());
}

#[test]
fn repeat_arm_reports_pass_rates() {
    // 单臂重复的原料应可被外部取用（要报方差分解的调用方不必重写循环）。
    let runner = ExperimentRunner::new().with_repeats(3);
    let rates = runner.repeat_arm(Arm::Baseline, &same_run);
    assert_eq!(rates.len(), 3);
    assert!(rates.iter().all(|r| *r == 1.0), "same_run 全过 ⇒ rates 应全 1.0");
}
