//! 实验运行器 —— 把 `nt_evolution_eval` 的判决能力变成**可调用的能力**
//! （2026-09-29，B6）
//!
//! ## 为什么需要这一层
//!
//! `nt_evolution_eval`（同目录）提供臂中立 A/B + 噪声地板 + veto + 账本，
//! 但它**零消费者** —— 754 行代码 + 19 个测试，从未被真正调用过。
//!
//! ⛔ 这就是本轮刚根治的 `check-doc-drift` 恒红与 `seal_loop` 硬传 `None`
//! **同一个病第三次出现**：「存在但不起作用的东西，比不存在更危险 ——
//! 它让人以为防护在位」。
//!
//! ## 为什么不能直接在 `seal_loop` 的迭代钩子里调用 `judge_ab`
//!
//! **噪声地板需要同一臂重复运行 ≥2 次**（`estimate_noise_floor` 在 `n < 2`
//! 时返回 `None`）。而 `seal_loop` 的闭环钩子每次迭代只有**一个**候选
//! —— 物理上凑不出一对可比的两臂。
//!
//! ⇒ 正确的接法**不是**「每次迭代跑一次 A/B」，而是：
//! **把「跑一次完整实验」封装成可调用能力**，让能提供多臂多轮的
//! 外部流程（实验脚本 / eval 批次 / 人工对照实验）去调用它。
//!
//! ⛔ 本模块**不 spawn cargo、不调 LLM、不碰文件系统**。
//! 「跑一次」由调用方以 closure 注入 —— 本模块只负责
//! **重复、统计、判决、记账**这四件纯逻辑。
//!
//! ## 与既有能力的边界
//!
//! | | 覆盖 | 不覆盖 |
//! |---|---|---|
//! | `nt_mind_eval_harness`（`run_regression_test`） | 单点确定性回归 | 统计显著性 |
//! | 本模块 `ExperimentRunner` | 臂中立 A/B + 噪声地板 + veto + 账本 | 具体产出的判定逻辑（由 closure 注入）|

use crate::l5_cognition::traits::RegressionResult;
use crate::l6_meta::nt_meta::nt_evolution_eval::nt_evolution_eval::{
    estimate_noise_floor, judge_ab, Arm, CaseOutcome, ClaimFidelity, EnvFingerprint, Ledger,
    LedgerEntry, Preregistration, Verdict,
};
use std::sync::Mutex;

/// 「跑一次」的执行器。
///
/// 返回 `Vec<CaseOutcome>`：**一条 case 一个结果**。
/// 调用方负责把实际产出转成 `CaseOutcome`（含 `required`/`forbidden` 判定）。
type RunOnce<'a> = &'a dyn Fn(Arm, u32) -> Vec<CaseOutcome>;

/// 一次实验的完整结果。
#[derive(Debug, Clone)]
pub struct ExperimentOutcome {
    pub verdict: Verdict,
    /// 两臂各跑了几轮。
    pub repeats: u32,
    /// 实际使用的噪声地板。
    pub floor: Option<crate::l6_meta::nt_meta::nt_evolution_eval::nt_evolution_eval::NoiseFloor>,
    /// agent 自述 vs 真相的偏差（**两个臂都记**，因为撒谎不分臂）。
    pub claim_fidelity: ClaimFidelity,
}

/// 实验运行器。
///
/// ## 为什么状态是 `Mutex<Ledger>` 而不是 `&mut Ledger`
///
/// 账本要**跨实验累积**（这正是它的价值：`self_refutation_rate` 需要历史）。
/// 若让调用方自己传 `&mut Ledger`，每个实验都得自己管存储，
/// 于是「忘了记账」就成了默认行为 —— 而账本的价值恰恰在于**从不遗忘**。
///
/// ⇒ 内部持有，调用方只管跑实验。
pub struct ExperimentRunner {
    ledger: Mutex<Ledger>,
    /// 默认重复轮数。⛔ **必须 ≥2**，否则估不出噪声地板。
    default_repeats: u32,
}

impl Default for ExperimentRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentRunner {
    /// 构造。默认 3 轮 —— 够估地板，又不至于太慢。
    pub fn new() -> Self {
        Self {
            ledger: Mutex::new(Ledger::new()),
            default_repeats: 3,
        }
    }

    /// 自定义重复轮数。⛔ <2 会被夹到 2（否则地板为 `None`，
    /// 判决必然被 `insufficient_evidence` 否决 —— 那是对的，
    /// 但让调用方以为自己配置生效了更糟）。
    pub fn with_repeats(mut self, repeats: u32) -> Self {
        self.default_repeats = repeats.max(2);
        self
    }

    /// 账本快照。
    pub fn ledger_snapshot(&self) -> Ledger {
        self.ledger
            .lock()
            .map(|l| l.clone())
            .unwrap_or_else(|_| Ledger::new())
    }

    /// 某 subject 的自证失败率。
    ///
    /// ⚠️ **高值是危险信号**：说明这套预注册老在自证失败，
    /// 其「通过」记录不可信。依据 `EVOLUTION-ROADMAP-2026-09-28 §0.2`。
    pub fn self_refutation_rate(&self, subject: &str) -> f64 {
        self.ledger_snapshot().self_refutation_rate(subject)
    }

    /// **单臂重复**运行，返回各轮通过率 —— 这是噪声地板的原料。
    ///
    /// 公开它的理由：想自己算地板的调用方（例如要报告方差分解的）
    /// 不必重复实现循环。
    pub fn repeat_arm(&self, arm: Arm, run_once: RunOnce<'_>) -> Vec<f64> {
        (0..self.default_repeats)
            .map(|i| super::nt_evolution_eval::nt_evolution_eval::pass_rate(&run_once(arm, i)))
            .collect()
    }

    /// 跑一次完整实验并入账。
    ///
    /// ## 流程（顺序不可换）
    /// 1. 预注册校验 —— 缺 `falsifier` 直接拒（不烧任何运行成本）
    /// 2. 单臂重复 ≥2 次 ⇒ 估噪声地板
    /// 3. 两臂各跑一轮正式对照
    /// 4. `judge_ab` 判决
    /// 5. **无论接受与否都入账**
    ///
    /// ## 参数
    /// - `prereg` 预注册。⛔ `falsifier` 为空 ⇒ 第 1 步就拒。
    /// - `subject` 账本聚合键（如 `"seal-loop/iteration-42"`）。
    /// - `baseline_env` / `candidate_env` 两臂的环境指纹。
    ///   ⛔ 不同 ⇒ `environment_mismatch`，**结论不可比**。
    ///   ⚠️ 本函数**信任**调用方传入的指纹，不自己算 ——
    ///   因为「算指纹」需要读 git 与 Cargo.lock，那是 I/O，
    ///   属于本纯逻辑模块之外的事。
    pub fn run_experiment(
        &self,
        subject: &str,
        prereg: &Preregistration,
        baseline_env: &EnvFingerprint,
        candidate_env: &EnvFingerprint,
        run_once: RunOnce<'_>,
    ) -> ExperimentOutcome {
        // ① 预注册：缺 falsifier 立即拒，不烧运行成本
        if !prereg.is_complete() {
            let empty: Vec<CaseOutcome> = Vec::new();
            let verdict = judge_ab(
                prereg,
                &empty,
                &empty,
                baseline_env,
                candidate_env,
                None,
            );
            return self.record(
                subject,
                prereg,
                baseline_env,
                verdict,
                None,
                ClaimFidelity::of(&empty),
            );
        }

        // ② 噪声地板：单臂重复
        let rates = self.repeat_arm(Arm::Baseline, run_once);
        let floor = estimate_noise_floor(&rates);

        // ③ 两臂正式对照（第 0 轮已在地板测量中跑过 baseline，
        //    这里重跑一次以保证两臂走的是同一条代码路径）
        let baseline = run_once(Arm::Baseline, 0);
        let candidate = run_once(Arm::Candidate, 0);

        // ④ 判决
        let verdict = judge_ab(prereg, &baseline, &candidate, baseline_env, candidate_env, floor);

        // ⑤ 记两个臂的自述偏差
        let mut all = baseline.clone();
        all.extend(candidate.iter().cloned());
        let fidelity = ClaimFidelity::of(&all);

        self.record(subject, prereg, baseline_env, verdict, floor, fidelity)
    }

    fn record(
        &self,
        subject: &str,
        prereg: &Preregistration,
        env: &EnvFingerprint,
        verdict: Verdict,
        floor: Option<crate::l6_meta::nt_meta::nt_evolution_eval::nt_evolution_eval::NoiseFloor>,
        claim_fidelity: ClaimFidelity,
    ) -> ExperimentOutcome {
        let outcome = ExperimentOutcome {
            verdict: verdict.clone(),
            repeats: self.default_repeats,
            floor,
            claim_fidelity,
        };
        if let Ok(mut l) = self.ledger.lock() {
            l.append(LedgerEntry {
                subject: subject.to_string(),
                prereg: prereg.clone(),
                env: env.clone(),
                verdict,
                claim_fidelity,
                noise_floor: floor,
            });
        }
        outcome
    }
}

/// 把 `RegressionResult` 转成 `CaseOutcome` —— 供 `seal_loop` 的
/// 单点回归结果汇入账本。
///
/// ⛔ **失败时必须带 reason**：本模块在 `claim_fidelity` 之外
/// 不检查 reason，但 `seal_loop.rs` 的告警依赖它才有信息量。
/// 转换器**不制造** reason（没有就空着），由调用方负责填。
pub fn outcome_from_regression(
    case_id: impl Into<String>,
    arm: Arm,
    result: &RegressionResult,
) -> CaseOutcome {
    CaseOutcome {
        case_id: case_id.into(),
        arm,
        agent_claimed: None,
        passed: result.passed,
        missing_required: result.reasons.clone(),
        hit_forbidden: Vec::new(),
        elapsed_ms: 0,
    }
}
