//! `nt_evolution_eval` 单测。
//!
//! ⛔ **这些测试的作用不只是「验证代码对」**，而是**证明这道门非空**。
//! 依据 `LESSONS-20260928-verification-must-be-executable`：任何「X 是好的」
//! 的断言都要能被执行验证。本文件里每条测试都在钉死一个**具体的失效模式**。
//!
//! 特别地，`test_*_must_reject` 系列的命名是刻意的 ——
//! 它们验证的是**门会拒绝**，不是「门会通过」。

#![cfg(test)]

use crate::l6_meta::nt_meta::*;

fn spec(id: &str, required: &[&str], forbidden: &[&str]) -> CaseSpec {
    CaseSpec {
        id: id.to_string(),
        prompt: "p".into(),
        required: required.iter().map(|s| s.to_string()).collect(),
        forbidden: forbidden.iter().map(|s| s.to_string()).collect(),
        tags: Vec::new(),
    }
}

fn clean_env() -> EnvFingerprint {
    EnvFingerprint::new("abc1234", Vec::new(), "lock1")
}

fn dirty_env(head: &str) -> EnvFingerprint {
    EnvFingerprint::new(head, vec!["b.rs".into(), "a.rs".into()], "lock1")
}

fn good_floor() -> NoiseFloor {
    NoiseFloor {
        n: 10,
        mean_pass_rate: 0.5,
        std_dev: 0.01,
    }
}

fn complete_prereg() -> Preregistration {
    Preregistration::new(
        "改动 T 让遗忘路径更准",
        "若 T 在 case-3 上不提升，则假设被削弱",
        "deadbee",
        "test-model-v1",
        0.01,
    )
}

// ── 环境指纹 ──────────────────────────────────────────────────────────

#[test]
fn env_fingerprint_is_order_independent() {
    // dirty_paths 顺序不同但内容相同 ⇒ 必须是同一指纹。
    // 依据：`nt_manifest.py:58-79` 的 env_fingerprint 对 dirty 路径**排序**。
    let a = EnvFingerprint::new("h1", vec!["z.rs".into(), "a.rs".into()], "L");
    let b = EnvFingerprint::new("h1", vec!["a.rs".into(), "z.rs".into()], "L");
    assert_eq!(a.digest, b.digest, "排序后仍不同 ⇒ 指纹不可用");
    assert!(!a.is_clean());
}

#[test]
fn env_fingerprint_detects_dirty_and_head_change() {
    let clean = clean_env();
    assert!(clean.is_clean());
    let dirty = dirty_env("abc1234");
    assert!(!dirty.same_env(&clean), "脏树与净树必须判为不同环境");
    let other_head = EnvFingerprint::new("def5678", vec![], "lock1");
    assert!(!other_head.same_env(&clean), "换 HEAD 必须判为不同环境");
    let other_lock = EnvFingerprint::new("abc1234", vec![], "lock2");
    assert!(!other_lock.same_env(&clean), "换 Cargo.lock 必须判为不同环境");
}

// ── 确定性判定 ────────────────────────────────────────────────────────

#[test]
fn judge_case_passes_only_when_all_required_present() {
    let s = spec("c1", &["A", "B"], &["BAD"]);
    let (ok, missing, hit) = judge_case(&s, "xxAxxBxx");
    assert!(ok && missing.is_empty() && hit.is_empty());
    let (ok, missing, hit) = judge_case(&s, "xxAxx");
    assert!(!ok, "缺 B 必须失败（不做部分给分）");
    assert_eq!(missing, vec!["B".to_string()]);
    assert!(hit.is_empty());
}

#[test]
fn judge_case_forbidden_is_veto_not_deduction() {
    // 依据：`rrsi/selection.py` 的 non-compensatory guard ——
    // 安全维度用否决，不用权重，否则别处得分可以买下这里的回归。
    let s = spec("c1", &[], &["rm -rf /"]);
    let (ok, _, hit) = judge_case(&s, "echo ok; rm -rf /");
    assert!(!ok, "命中 forbidden 必须否决");
    assert_eq!(hit, vec!["rm -rf /".to_string()]);
}

// ── 噪声地板 ──────────────────────────────────────────────────────────

#[test]
fn noise_floor_requires_at_least_two_runs() {
    // ⛔ 单次运行无法估计噪声。依据：`cline` 的教训 ——
    // 框架完整但 CI 被 removed，数据从不存在。
    assert!(estimate_noise_floor(&[]).is_none());
    assert!(estimate_noise_floor(&[0.5]).is_none());
    assert!(estimate_noise_floor(&[0.5, 0.6]).is_some());
}

#[test]
fn noise_floor_detects_the_soup_style_spread() {
    // 复刻 `Soup/benchmarks/gate-836`：13 次逐字节相同配置，
    // 吞吐横跨 376.4–915.3 tok/s（2.43×，CV 35.3%）。
    // 这里用通过率复刻同构现象：均值相近但方差可观。
    let rates = vec![0.50, 0.52, 0.48, 0.55, 0.47, 0.51, 0.49, 0.53, 0.46, 0.54];
    let floor = estimate_noise_floor(&rates).expect("n=10");
    assert!((floor.mean_pass_rate - 0.505).abs() < 1e-9);
    assert!(floor.std_dev > 0.0, "同配置多次运行仍有离散度");
    // 地板必须为正，否则任何微小差值都会被当成「显著」。
    assert!(noise_threshold(&floor, 3.0).unwrap_or(0.0) > 0.0);
}

#[test]
fn noise_floor_blocks_small_delta() {
    // ⚠️ 本测试最初写错了：baseline 与 candidate 各 2 条**全过** ⇒ delta=0，
    // 却断言"应当接受"。实测跑出来是 REJECT(within_noise) —— **判决是对的，测试是错的**。
    // 这正是 `LESSONS-20260928-verification-must-be-executable` 的用法：
    // 断言写错时，门会顶回来告诉你。
    //
    // 现在测真正想测的：差值**小于**噪声地板 ⇒ 判为噪声。
    let floor = NoiseFloor {
        n: 10,
        mean_pass_rate: 0.5,
        std_dev: 0.5, // 3σ = 1.5 ⇒ 任何真实差值都在噪声内
    };
    let baseline: Vec<CaseOutcome> = (0..10)
        .map(|_| CaseOutcome::pass("a", Arm::Baseline))
        .collect();
    let mut candidate = baseline.clone();
    for o in candidate.iter_mut().take(6) {
        o.arm = Arm::Candidate;
    }
    let v = judge_ab(
        &complete_prereg(),
        &baseline,
        &candidate,
        &clean_env(),
        &clean_env(),
        Some(floor),
    );
    // delta = 1.0 - 1.0 = 0；即便按通过率算也超不过 1.5 的地板
    assert!(!v.accept, "未超噪声地板的差值必须被拒：{}", v.reason);
    assert!(v.vetoes.contains(&Veto::WithinNoise));
}

#[test]
fn large_real_improvement_is_accepted() {
    // 与上一条配对：同样的地板调到窄，真实大幅提升必须被接受。
    // ⛔ 一个只会拒绝的门和不会拒绝的门一样没用 —— 必须双向验。
    let floor = NoiseFloor {
        n: 10,
        mean_pass_rate: 0.5,
        std_dev: 0.001,
    };
    let baseline = vec![CaseOutcome::fail(
        "a".to_string(),
        Arm::Baseline,
        vec!["missing".to_string()],
        Vec::new(),
    )]; // 0/1 ⇒ 0.0
    let candidate = vec![CaseOutcome::pass("a", Arm::Candidate)]; // 1/1 ⇒ 1.0
    let v = judge_ab(
        &complete_prereg(),
        &baseline,
        &candidate,
        &clean_env(),
        &clean_env(),
        Some(floor),
    );
    assert!((v.raw_delta - 1.0).abs() < 1e-9, "delta 必须是 +1.0，实际 {}", v.raw_delta);
    assert!(v.accept, "真实大幅提升必须被接受：{}", v.reason);
    assert!(v.vetoes.is_empty());
}

#[test]
fn no_noise_floor_means_no_claim() {
    // ⛔ 这是本模块最重要的一条拒绝：没有地板就不许判「改进」。
    let baseline = vec![CaseOutcome::pass("a", Arm::Baseline)];
    let candidate = vec![CaseOutcome::pass("a", Arm::Candidate)];
    let v = judge_ab(
        &complete_prereg(),
        &baseline,
        &candidate,
        &clean_env(),
        &clean_env(),
        None, // ← 无地板
    );
    assert!(!v.accept, "无噪声地板时不得接受任何改进");
    assert!(v.vetoes.contains(&Veto::InsufficientEvidence));
}

// ── veto（非补偿）──────────────────────────────────────────────────────

#[test]
fn regression_is_veto_even_when_aggregate_improves() {
    // ⛔ 本测试钉死「A 变好不能掩盖 B 变差」。
    let mut b_out = CaseOutcome::pass("bad", Arm::Baseline);
    b_out.passed = true;
    let c_out = CaseOutcome::fail(
        "bad".to_string(),
        Arm::Candidate,
        vec!["missing".to_string()],
        Vec::new(),
    );
    let c_out2 = CaseOutcome::pass("good", Arm::Candidate);

    let baseline = vec![b_out, CaseOutcome::pass("good", Arm::Baseline)];
    let candidate = vec![c_out, c_out2];

    let regressed = case_level_regressions(&baseline, &candidate);
    assert_eq!(regressed, vec!["bad".to_string()], "必须逐 case 定位回退");

    let v = judge_ab(
        &complete_prereg(),
        &baseline,
        &candidate,
        &clean_env(),
        &clean_env(),
        Some(NoiseFloor { n: 10, mean_pass_rate: 0.5, std_dev: 0.001 }),
    );
    assert!(!v.accept, "有 case 回退 ⇒ 即使聚合上升也必须否决");
    assert!(v.vetoes.contains(&Veto::DeterministicRegression));
    assert!(v.reason.contains("regressed=bad"), "理由必须能指名：{}", v.reason);
}

#[test]
fn safety_regression_vetoes_independently() {
    let base = CaseOutcome::fail("s".to_string(), Arm::Baseline, vec!["m".to_string()], Vec::new());
    let cand = CaseOutcome::fail(
        "s".to_string(),
        Arm::Candidate,
        vec!["m".to_string()],
        vec!["danger".to_string()],
    );

    let v = judge_ab(
        &complete_prereg(),
        &[base],
        &[cand],
        &clean_env(),
        &clean_env(),
        Some(NoiseFloor { n: 10, mean_pass_rate: 0.0, std_dev: 0.001 }),
    );
    assert!(!v.accept);
    assert!(v.vetoes.contains(&Veto::SafetyRegressed));
}

#[test]
fn environment_mismatch_blocks_comparison() {
    let baseline = vec![CaseOutcome::pass("a", Arm::Baseline)];
    let candidate = vec![CaseOutcome::pass("a", Arm::Candidate)];
    let v = judge_ab(
        &complete_prereg(),
        &baseline,
        &candidate,
        &clean_env(),
        &dirty_env("abc1234"), // ← 两臂环境不同
        Some(good_floor()),
    );
    assert!(!v.accept, "不同环境的结论不可比");
    assert!(v.vetoes.contains(&Veto::EnvironmentMismatch));
}

#[test]
fn missing_falsifier_blocks_acceptance() {
    // 依据：预注册必须说出「什么结果会削弱它」。
    // 说不出的，不是假设，是感觉。
    let bad_prereg = Preregistration::new("T 会更好", "", "abc", "m", 0.01);
    assert!(!bad_prereg.is_complete());

    let baseline = vec![CaseOutcome::pass("a", Arm::Baseline)];
    let candidate = vec![CaseOutcome::pass("a", Arm::Candidate)];
    let v = judge_ab(
        &bad_prereg,
        &baseline,
        &candidate,
        &clean_env(),
        &clean_env(),
        Some(NoiseFloor { n: 10, mean_pass_rate: 0.0, std_dev: 0.001 }),
    );
    assert!(!v.accept, "无 falsifier 不得接受");
    assert!(v.vetoes.contains(&Veto::NoFalsifier));
}

// ── 自述 vs 真相 ──────────────────────────────────────────────────────

#[test]
fn claim_fidelity_catches_false_completion() {
    // 依据：typesafe-computer-use 的 osworld jsonl 实测 10 行里
    // 2 行 `outcome='done'` 但 `score=0.0`。
    // ⇒ 只看 agent 自述会得到完全错误的结论。
    let mut liar = CaseOutcome::fail("l1".to_string(), Arm::Candidate, vec!["m".to_string()], Vec::new());
    liar.agent_claimed = Some("done".into());

    let mut honest = CaseOutcome::pass("l2", Arm::Candidate);
    honest.agent_claimed = Some("done".into());

    let f = ClaimFidelity::of(&[liar, honest]);
    assert_eq!(f.false_completion, 1);
    assert_eq!(f.honest_success, 1);
    assert!((f.false_completion_rate() - 0.5).abs() < 1e-9);
}

#[test]
fn conservative_stop_is_not_penalized_like_lying() {
    // 提前退出但其实做对了 = 保守，不是说谎。两者必须分开计数。
    let mut conservative = CaseOutcome::pass("c", Arm::Candidate);
    conservative.agent_claimed = Some("low_confidence".into());
    conservative.passed = true;

    let f = ClaimFidelity::of(&[conservative]);
    assert_eq!(f.conservative_stop, 1);
    assert_eq!(f.false_completion, 0);
    assert_eq!(f.false_completion_rate(), 0.0);
}

// ── 账本 ──────────────────────────────────────────────────────────────

#[test]
fn ledger_records_rejections_too() {
    // 依据：autoresearch —— 诚实的负面结果是信号，不是尴尬。
    // ⛔ 只记通过的账本是日志，不是测量。
    //
    // ⚠️ 修正记录：本测试初版把「两臂都全过（delta=0）」当成「接受」，
    // 实测被 REJECT —— **判决是对的，测试写错了**。两次同类错误说明
    // 「delta=0 不是改进」这条规则极易被忽略，故在此显式留痕。
    let mut ledger = Ledger::new();
    let env = clean_env();

    // 真接受：baseline 挂、candidate 过 ⇒ delta = +1.0，远超窄地板
    let b_fail = CaseOutcome::fail(
        "a".to_string(),
        Arm::Baseline,
        vec!["missing".to_string()],
        Vec::new(),
    );
    let accept = LedgerEntry {
        subject: "T-1".into(),
        prereg: complete_prereg(),
        env: env.clone(),
        verdict: judge_ab(
            &complete_prereg(),
            &[b_fail],
            &[CaseOutcome::pass("a", Arm::Candidate)],
            &env,
            &env,
            Some(NoiseFloor { n: 10, mean_pass_rate: 0.0, std_dev: 0.001 }),
        ),
        claim_fidelity: ClaimFidelity::of(&[]),
        noise_floor: None,
    };
    assert!(accept.verdict.accept, "正向用例必须真被接受：{}", accept.verdict.reason);
    ledger.append(accept);

    // 负结果：无地板 ⇒ 必拒，且必须照样进账
    let reject = LedgerEntry {
        subject: "T-1".into(),
        prereg: complete_prereg(),
        env: env.clone(),
        verdict: judge_ab(
            &complete_prereg(),
            &[CaseOutcome::pass("a", Arm::Baseline)],
            &[CaseOutcome::pass("a", Arm::Candidate)],
            &env,
            &env,
            None, // 无地板 ⇒ 必拒
        ),
        claim_fidelity: ClaimFidelity::of(&[]),
        noise_floor: None,
    };
    assert!(!reject.verdict.accept);
    ledger.append(reject);

    assert_eq!(ledger.len(), 2, "负面结果必须也进账");
    let tally = ledger.tally();
    assert_eq!(tally.get("T-1"), Some(&(1, 1)), "通过与拒绝都要可查");
    assert!(
        (ledger.self_refutation_rate("T-1") - 0.5).abs() < 1e-9,
        "自证失败率本身就是信号"
    );
}

// ── 未验证项 ──────────────────────────────────────────────────────────

#[test]
fn unverified_blocks_overclaim() {
    // 依据：dsh-use-wallpaper 的三段式范本。
    // ⛔ 有未验证项却写「已证明有效」= 过度声称。
    let mut u = Unverified::new();
    assert!(!u.blocks_claim());
    u.add("macOS 未测");
    u.add(""); // 空项应被忽略
    u.add("macOS 未测"); // 重复应去重
    assert!(u.blocks_claim());
    assert_eq!(u.items.len(), 1, "去重后只有 1 项：{}", u);
}

// ── 门本身非空 ────────────────────────────────────────────────────────

#[test]
fn gate_is_not_vacuous_every_veto_kind_reachable() {
    // 本测试的意义：**逐个确认每种 veto 都真的能触发**。
    // 一个从不触发的 veto 分支 = 一个永远绿的假门。
    let env = clean_env();
    let one = |arm: Arm| vec![CaseOutcome::pass("a", arm)];

    // EnvironmentMismatch
    let v = judge_ab(&complete_prereg(), &one(Arm::Baseline), &one(Arm::Candidate), &env, &dirty_env("x"), Some(good_floor()));
    assert!(v.vetoes.contains(&Veto::EnvironmentMismatch));

    // InsufficientEvidence
    let v = judge_ab(&complete_prereg(), &one(Arm::Baseline), &one(Arm::Candidate), &env, &env, None);
    assert!(v.vetoes.contains(&Veto::InsufficientEvidence));

    // NoFalsifier
    let v = judge_ab(&Preregistration::new("h", "", "c", "m", 0.0), &one(Arm::Baseline), &one(Arm::Candidate), &env, &env, Some(good_floor()));
    assert!(v.vetoes.contains(&Veto::NoFalsifier));

    // DeterministicRegression
    let c = CaseOutcome::fail("a".to_string(), Arm::Candidate, vec!["m".to_string()], Vec::new());
    let v = judge_ab(&complete_prereg(), &one(Arm::Baseline), &[c], &env, &env, Some(good_floor()));
    assert!(v.vetoes.contains(&Veto::DeterministicRegression));

    // SafetyRegressed
    let base = CaseOutcome::fail("a".to_string(), Arm::Baseline, vec!["m".to_string()], Vec::new());
    let c = CaseOutcome::fail(
        "a".to_string(),
        Arm::Candidate,
        vec!["m".to_string()],
        vec!["x".to_string()],
    );
    let v = judge_ab(&complete_prereg(), &[base], &[c], &env, &env, Some(good_floor()));
    assert!(v.vetoes.contains(&Veto::SafetyRegressed));
}

#[test]
fn empty_results_do_not_produce_nan() {
    // NaN 会静默传播并让所有比较为 false ⇒ 最难查的一类 bug。
    assert_eq!(pass_rate(&[]), 0.0);
    let v = judge_ab(
        &complete_prereg(),
        &[],
        &[],
        &clean_env(),
        &clean_env(),
        Some(NoiseFloor { n: 10, mean_pass_rate: 0.0, std_dev: 0.0 }),
    );
    assert!(!v.raw_delta.is_nan(), "空结果不得产生 NaN");
    assert_eq!(v.raw_delta, 0.0);
    assert!(!v.accept, "两臂都没数据不得接受");
}
