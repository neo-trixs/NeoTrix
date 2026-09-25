//! Verify/Oracle 子系统 — 自验证奖励 + 统一验证 + Oracle 阶梯 (纯搬移自门面)。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ────────────────────────────────────────────────────────────────
// P9: SelfVerifiableReward (吸收 arXiv 2607.23802 RLSVR)
// 可自验证奖励信号: 无需 ground truth 也能给模型反馈。
// 三个自验证源: 可判定性(确定性校验) / 可提取性(答案可从响应提取) /
//              约束满足(拒绝策略)。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VerificationChannel {
    #[default]
    Deterministic,
    Extractable,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfVerifiableReward {
    pub channel: VerificationChannel,
    pub score: f64,
    pub verifiable: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RewardSignal {
    pub rewards: Vec<SelfVerifiableReward>,
}

impl RewardSignal {
    pub fn total(&self) -> f64 {
        self.rewards.iter().map(|r| r.score).sum()
    }

    /// 只在所有通道都可验证时才给出最终奖励 (RLSVR: 弱信号仅用于对比, 不用于训练)。
    pub(crate) fn _gated_total(&self) -> Option<f64> {
        if self.rewards.iter().all(|r| r.verifiable) {
            Some(self.total())
        } else {
            None
        }
    }
}

pub fn verify_deterministic(expected: &str, actual: &str) -> SelfVerifiableReward {
    let ok = expected.trim() == actual.trim();
    SelfVerifiableReward {
        channel: VerificationChannel::Deterministic,
        score: if ok { 1.0 } else { 0.0 },
        verifiable: true,
        detail: format!("deterministic match: {}", ok),
    }
}

pub fn verify_extractable(needle: &str, actual: &str) -> SelfVerifiableReward {
    let ok = !actual.is_empty() && actual.contains(needle);
    SelfVerifiableReward {
        channel: VerificationChannel::Extractable,
        score: if ok { 1.0 } else { 0.0 },
        verifiable: true,
        detail: format!("answer extractable: {}", ok),
    }
}

pub fn verify_constraint(policy: &str, actual: &str, forbidden: &[&str]) -> SelfVerifiableReward {
    let violated = forbidden.iter().any(|f| actual.contains(f));
    SelfVerifiableReward {
        channel: VerificationChannel::Constraint,
        score: if violated { 0.0 } else { 1.0 },
        verifiable: !actual.is_empty(),
        detail: format!("policy {} violated={}", policy, violated),
    }
}

// ────────────────────────────────────────────────────────────────
// llm-as-a-verifier 吸收 (R-P79 代码级接线): 统一验证框架
// "Any modality, Many Applications, One Unified Verification Framework"。
// 提供模态无关的分派入口: 任意产出 (代码/JSON/策略约束/确定性答案) 经
// 对应通道验证, 聚合为单一判定。与既有 verify_* 通道一致, 不建平行系统。
// ────────────────────────────────────────────────────────────────

/// 统一验证请求 — 模态无关的任意产出描述。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedVerifyRequest {
    /// 产出类型: "code" | "json" | "deterministic" | "policy"。
    pub modality: String,
    /// 模型产出 (待验证内容)。
    pub actual: String,
    /// 预期 (deterministic/code 通道用)。
    pub expected: Option<String>,
    /// 需从产出中可提取的锚点 (json/code 通道用)。
    pub extractable: Option<String>,
    /// 约束策略 + 禁止串 (policy 通道用)。
    pub policy: Option<String>,
    pub forbidden: Vec<String>,
}

/// 统一验证结果 — 单入口多通道聚合判定。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnifiedVerifyResult {
    pub channel: VerificationChannel,
    pub score: f64,
    pub verifiable: bool,
    pub detail: String,
}

impl UnifiedVerifyResult {
    /// 判定为通过: 可验证且满分。
    pub fn passed(&self) -> bool {
        self.verifiable && self.score >= 1.0
    }
}

/// 统一验证入口: 按模态路由到对应验证通道, 聚合为单一判定。
pub fn verify_unified(req: &UnifiedVerifyRequest) -> UnifiedVerifyResult {
    match req.modality.as_str() {
        "deterministic" => {
            let r = verify_deterministic(
                req.expected.as_deref().unwrap_or(""),
                &req.actual,
            );
            UnifiedVerifyResult {
                channel: r.channel,
                score: r.score,
                verifiable: r.verifiable,
                detail: r.detail,
            }
        }
        "json" | "code" => {
            let r = verify_extractable(
                req.extractable.as_deref().unwrap_or(""),
                &req.actual,
            );
            UnifiedVerifyResult {
                channel: r.channel,
                score: r.score,
                verifiable: r.verifiable && !req.actual.trim().is_empty(),
                detail: r.detail,
            }
        }
        "policy" => {
            let r = verify_constraint(
                req.policy.as_deref().unwrap_or("default"),
                &req.actual,
                &req.forbidden.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            );
            UnifiedVerifyResult {
                channel: r.channel,
                score: r.score,
                verifiable: r.verifiable,
                detail: r.detail,
            }
        }
        other => UnifiedVerifyResult {
            channel: VerificationChannel::Constraint,
            score: 0.0,
            verifiable: false,
            detail: format!("unknown modality: {other}"),
        },
    }
}

/// 批量统一验证: 逐条分派并汇总 (全通过才算整体通过)。
pub fn verify_unified_batch(requests: &[UnifiedVerifyRequest]) -> (Vec<UnifiedVerifyResult>, bool) {
    let results: Vec<UnifiedVerifyResult> =
        requests.iter().map(verify_unified).collect();
    let all_passed = !results.is_empty() && results.iter().all(|r| r.passed());
    (results, all_passed)
}

/// SelfTest 包装件 (T2 注册): 可自验证奖励纯函数健康检测。
/// 复用 EvalHarness SelfTest 的 verify_* 逻辑, 但无 provider 构造依赖。
#[derive(Default)]
pub struct SelfVerifiableRewardSelfTest;

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for SelfVerifiableRewardSelfTest {
    fn name(&self) -> &str {
        "nt_mind_eval_harness_self_verifiable_reward"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = verify_deterministic("42", "42");
        if !r.verifiable || r.score != 1.0 {
            return Err(vec!["deterministic channel failed".into()]);
        }
        let e = verify_extractable("json", r#"{"answer": 42}"#);
        if !e.verifiable {
            return Err(vec!["extractable channel failed".into()]);
        }
        let c = verify_constraint("no-pii", "user@example.com", &["@example.com"]);
        if c.score != 0.0 {
            return Err(vec!["constraint channel should reject PII".into()]);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────
// P11: OracleLadder — T0-T3 执行式 oracle 验证阶梯 (defending-harness F5)
// T0 构建检查 → T1 复现 → T2 回归 → T3 重攻击。每级都是可执行 oracle;
// 变更仅在每级通过后提升。无模型判断门 — 只有可执行验证
// ("No patch fails based on model judgment")。证据回传, 非 prose。
// ────────────────────────────────────────────────────────────────

/// 阶梯的 4 个 rung: T0 build-check → T1 repro → T2 regression → T3 re-attack。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OracleRung {
    /// T0: apply + rebuild 构建退出码 (cheapest, 第一层)
    T0BuildCheck,
    /// T1: 原始 PoC 不再触发 (exit 0 且无崩溃信号)
    T1Repro,
    /// T2: 项目测试套件通过 (回归)
    T2Regression,
    /// T3: 重新攻击 (fresh find-agent 二次攻击; ASAN 判定)
    T3Reattack,
}

impl OracleRung {
    pub const LADDER: [OracleRung; 4] = [
        OracleRung::T0BuildCheck,
        OracleRung::T1Repro,
        OracleRung::T2Regression,
        OracleRung::T3Reattack,
    ];

    pub fn label(&self) -> &'static str {
        use OracleRung::*;
        match self {
            T0BuildCheck => "build-check",
            T1Repro => "repro",
            T2Regression => "regression",
            T3Reattack => "re-attack",
        }
    }

    pub fn next(self) -> Option<OracleRung> {
        use OracleRung::*;
        match self {
            T0BuildCheck => Some(T1Repro),
            T1Repro => Some(T2Regression),
            T2Regression => Some(T3Reattack),
            T3Reattack => None,
        }
    }
}

/// 单级可执行 oracle 的结果: passed + 证据 (evidence), 非 prose。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RungResult {
    pub rung: OracleRung,
    pub passed: bool,
    pub evidence: String,
}

impl RungResult {
    pub fn pass(rung: OracleRung, evidence: impl Into<String>) -> Self {
        Self { rung, passed: true, evidence: evidence.into() }
    }

    pub fn fail(rung: OracleRung, evidence: impl Into<String>) -> Self {
        Self { rung, passed: false, evidence: evidence.into() }
    }
}

/// 阶梯运行报告: 达到的最高级 + 每级结果 + 是否整体提升。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderReport {
    pub results: Vec<RungResult>,
    pub highest_passed: Option<OracleRung>,
    pub promoted: bool,
}

impl LadderReport {
    pub(crate) fn _rung_result(&self, rung: OracleRung) -> Option<&RungResult> {
        self.results.iter().find(|r| r.rung == rung)
    }
}

/// 可执行 oracle: 确定性执行的闭包, 返回 (passed, evidence)。
pub type RungOracle = Box<dyn Fn() -> RungResult + Send + Sync>;

/// 执行式 oracle 验证阶梯 (defending-harness F5): 按 T0→T3 单调顺序执行,
/// 停在第一个失败级; 只有全部通过才提升到 T3。无模型判断门。
#[derive(Default)]
pub struct OracleLadder {
    oracles: HashMap<OracleRung, RungOracle>,
}

impl OracleLadder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 为某个 rung 注册可执行 oracle。
    pub fn with_oracle(
        mut self,
        rung: OracleRung,
        oracle: impl Fn() -> RungResult + Send + Sync + 'static,
    ) -> Self {
        self.oracles.insert(rung, Box::new(oracle));
        self
    }

    pub fn has(&self, rung: OracleRung) -> bool {
        self.oracles.contains_key(&rung)
    }

    /// 按单调阶梯执行: T0→T1→T2→T3, 停在第一个失败级。
    /// 未注册的级视为未达到 (不提升), 但不产生失败证据。
    pub fn run(&self) -> LadderReport {
        let mut results = Vec::new();
        let mut highest_passed: Option<OracleRung> = None;
        for rung in OracleRung::LADDER {
            match self.oracles.get(&rung) {
                Some(oracle) => {
                    let res = oracle();
                    let passed = res.passed;
                    results.push(res);
                    if passed {
                        highest_passed = Some(rung);
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
        let promoted = highest_passed == Some(OracleRung::T3Reattack);
        LadderReport { results, highest_passed, promoted }
    }

    /// 阶梯有效性 (C5 自愈): 已注册 rung 集合必须是 T0 起的连续前缀 (无空洞),
    /// 且不超出阶梯总级数 (T3)。空洞阶梯 (如 T2 注册而 T1 缺失) 无法单调执行。
    pub fn is_valid(&self) -> bool {
        let mut expect_next = true;
        for rung in OracleRung::LADDER {
            if self.oracles.contains_key(&rung) {
                if !expect_next {
                    return false;
                }
            } else {
                expect_next = false;
            }
        }
        true
    }

    /// 重置阶梯到 T0 (C5 自愈): 清空全部 oracle 回到基准状态 (构建检查基态,
    /// 无空洞), 返回重置动作描述。
    pub fn reset_to_t0(&mut self) -> Vec<String> {
        let mut actions = Vec::new();
        for rung in OracleRung::LADDER {
            if self.oracles.remove(&rung).is_some() {
                actions.push(format!("dropped oracle at rung {}", rung.label()));
            }
        }
        actions.push("ladder reset to T0 (build-check base)".to_string());
        actions
    }
}

/// C5 自愈检测件 (MIND-eval, oracle_ladder): 构造含空洞 rung 的阶梯,
/// reset_to_t0 重置后断言 is_valid。
pub struct OracleLadderHealer;

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for OracleLadderHealer {
    fn name(&self) -> &str {
        "nt_mind_eval_harness::oracle_ladder_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        let healthy = OracleLadder::new()
            .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
            .with_oracle(OracleRung::T1Repro, || RungResult::pass(OracleRung::T1Repro, "repro clean"));
        if !healthy.is_valid() {
            failures.push("healthy contiguous ladder reported invalid".into());
        }

        let mut holed = OracleLadder::new()
            .with_oracle(OracleRung::T0BuildCheck, || RungResult::pass(OracleRung::T0BuildCheck, "build ok"))
            .with_oracle(OracleRung::T2Regression, || RungResult::pass(OracleRung::T2Regression, "regression ok"));
        if holed.is_valid() {
            failures.push("ladder with hole (T2 without T1) reported valid".into());
        }
        let actions = holed.reset_to_t0();
        if actions.is_empty() {
            failures.push("reset_to_t0 returned no actions".into());
        }
        if !holed.is_valid() {
            failures.push("ladder still invalid after reset".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}
