//! 有符号奖励账本 —— 用「证据」而非「自我声明」驱动目标优先级。
//!
//! ## 为什么不是又一套评分系统
//!
//! 本仓已有 40 个模块的自我进化子系统（`self_iterating/`：DPO 阶段、检查点、
//! 目标契约、递归深度奖励、benchmark 门）。但实测接线度：
//!
//! | 度量 | 值 |
//! |---|---:|
//! 模块总数 | 40 |
//! 有目录外引用的模块 | **5** |
//! `apply_self_edit` 的外部调用点 | **0** |
//! `seal_core` 之外的引用 | 仅 `pub use` 再导出（导出 ≠ 调用） |
//!
//! ⇒ 本模块**不新建平行体系**，而是把「奖励/惩罚」落到 `goal_contract`
//! 已有的证据词汇表上，并接进活路径（`GoalContractStage`）。
//!
//! ## 三条设计红线（各自对应一条实测教训）
//!
//! 1. **奖励只来自证据，永不来自自我声明。**
//!    `record_claim_without_evidence` 会产生**惩罚**而非奖励。
//!    教训来源：外部博客《parallel projects and collaboration》——
//!    把项目交给 agent 后「产出但不理解」；以及本轮 8 次
//!    「未读现场就下结论」的同型失误。
//! 2. **回归要扣分。** 曾经通过、现在不通过 ⇒ 负权重最重，
//!    否则系统会奖励「反复重写但从不退化」。
//! 3. **不声称未验证的事。** `EvidenceType` 每一项都必须能指向
//!    具体产物；无产物支撑的声明一律降为惩罚。
//!
//! ## 有符号奖励的设计
//!
//! `reward = evidence_weight − penalty`，其中：
//!
//! - `evidence_weight`：Compilation 1.0 / TestPass 2.0 / PropertyProof 2.5 /
//!   UserFeedback 3.0（越靠近真实用户越贵）；
//! - `penalty`：验证失败 −3.5 / 回归 −5.0 / 未支撑声明 −1.5。
//!
//! 分档依据（不是随手取整）：
//! - **回归 −5.0 > 最强正向证据 3.0** ⇒ 反复重写但从不退化，永远不划算；
//! - **验证失败 −3.5 > 3.0** ⇒ 一次「用户说好用」压不过一次验证失败；
//! - **未支撑声明 −1.5** 刻意落在 `TestPass`(2.0) 之下、`UserFeedback`(3.0) 之上：
//!   空口声明比一次普通测试更轻，但比用户真实反馈更重。
//!
//! 首版把验证失败设为 −2.0，于是「不作弊严格优于偶尔作弊」**并不成立**
//! （一次 UserFeedback 就能把一次验证失败洗成净正）。测试捕获后已抬到 −3.5。

use std::collections::{BTreeMap, BTreeSet};

use super::goal_contract::{EvidenceType, GoalPhase};

/// 已通过检查的登记键。goal 与产物用 \u{1f} 分隔，避免拼接歧义。
fn check_key(goal_id: &str, artifact: &str) -> String {
    format!("{goal_id}\u{1f}{artifact}")
}

/// 单条证据的权重。正向权重越高，代表证据越难伪造。
fn evidence_weight(t: EvidenceType) -> f64 {
    match t {
        EvidenceType::Compilation => 1.0,
        EvidenceType::TestPass => 2.0,
        EvidenceType::PropertyProof => 2.5,
        EvidenceType::UserFeedback => 3.0,
    }
}

/// 一条账本记录。
#[derive(Debug, Clone, PartialEq)]
pub enum LedgerEntry {
    /// 有证据支撑的进展。
    Evidence {
        goal_id: String,
        phase: GoalPhase,
        kind: EvidenceType,
        artifact: String,
    },
    /// 声称有进展但**没有产物**⇒ 惩罚。
    UnsupportedClaim { goal_id: String, phase: GoalPhase, claim: String },
    /// 验证失败 ⇒ 惩罚。
    VerificationFailed { goal_id: String, phase: GoalPhase, reason: String },
    /// 回归：曾经通过的检查现在不通过 ⇒ 最重惩罚。
    Regression { goal_id: String, check: String },
}

impl LedgerEntry {
    /// 本条的有符号权重。
    pub fn weight(&self) -> f64 {
        match self {
            LedgerEntry::Evidence { kind, .. } => evidence_weight(*kind),
            LedgerEntry::UnsupportedClaim { .. } => -1.5,
            LedgerEntry::VerificationFailed { .. } => -3.5,
            // 惩罚量级 > 任何单个正向证据（最大 3.0），保证「不作弊」占优。
            LedgerEntry::Regression { .. } => -5.0,
        }
    }

    pub fn goal_id(&self) -> &str {
        match self {
            LedgerEntry::Evidence { goal_id, .. }
            | LedgerEntry::UnsupportedClaim { goal_id, .. }
            | LedgerEntry::VerificationFailed { goal_id, .. }
            | LedgerEntry::Regression { goal_id, .. } => goal_id,
        }
    }

    /// 是否由真实产物支撑（`artifact` 非空）。
    fn has_artifact(&self) -> bool {
        match self {
            LedgerEntry::Evidence { artifact, .. } => !artifact.trim().is_empty(),
            _ => false,
        }
    }
}

/// 目标累计分。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GoalScore {
    pub reward: f64,
    pub penalty: f64,
    pub evidence_count: usize,
    pub violations: usize,
}

impl GoalScore {
    /// 有符号净值。**这是唯一的排序依据** ⇒ 记分会真的改变行为。
    pub fn net(&self) -> f64 {
        self.reward - self.penalty
    }

    /// 纯证据平均分（不受违规稀释），用于诊断「靠惩罚压分」vs「真的没进展」。
    pub fn evidence_quality(&self) -> f64 {
        if self.evidence_count == 0 {
            0.0
        } else {
            self.reward / self.evidence_count as f64
        }
    }

    /// 是否处于「不可信」状态：**有过任何违规**即不可信。
    ///
    /// ⚠️ 首版写的是 `violations > 0 && penalty > reward` —— 但
    /// `net = reward - penalty`，故该条件**恒蕴含 net < 0**，
    /// 「净值为正却不可信」在类型上根本不可能存在，测试
    /// `untrusted_goal_sinks_even_with_high_net` 于是永远无法通过。
    ///
    /// 真正的反洗白不变式是**按违规次数**判定：一次空口声明或一次回归
    /// 不能被后续任意堆证据抵掉。故改为 `violations > 0`。
    pub fn is_untrusted(&self) -> bool {
        self.violations > 0
    }
}

/// 有符号奖励账本。
#[derive(Debug, Clone, Default)]
pub struct RewardLedger {
    entries: Vec<LedgerEntry>,
    /// 曾经通过过的检查 ⇒ 用于识别回归。
    passed_checks: BTreeSet<String>,
}

impl RewardLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记一条证据。`artifact` 为空 ⇒ 自动降级为 `UnsupportedClaim`（惩罚）。
    ///
    /// 这是「奖励只来自证据」红线的**强制点**：调用方无法通过传入空产物
    /// 拿到正向权重。
    pub fn record_evidence(
        &mut self,
        goal_id: impl Into<String>,
        phase: GoalPhase,
        kind: EvidenceType,
        artifact: impl Into<String>,
    ) {
        let goal_id = goal_id.into();
        let artifact = artifact.into();
        let entry = if artifact.trim().is_empty() {
            LedgerEntry::UnsupportedClaim {
                goal_id,
                phase,
                claim: format!("{kind:?} 证据未附产物"),
            }
        } else {
            // TestPass / PropertyProof 天然是可复检的检查项，登记以便识别回归。
            if matches!(kind, EvidenceType::TestPass | EvidenceType::PropertyProof) {
                self.passed_checks.insert(check_key(&goal_id, &artifact));
            }
            LedgerEntry::Evidence {
                goal_id,
                phase,
                kind,
                artifact,
            }
        };
        self.entries.push(entry);
    }

    /// 记录一次验证失败（惩罚）。
    pub fn record_verification_failure(
        &mut self,
        goal_id: impl Into<String>,
        phase: GoalPhase,
        reason: impl Into<String>,
    ) {
        self.entries.push(LedgerEntry::VerificationFailed {
            goal_id: goal_id.into(),
            phase,
            reason: reason.into(),
        });
    }

    /// 报告回归：某检查曾经通过（由 `record_evidence` 登记），现在失败。
    /// 若该检查从未登记过通过 ⇒ 不计回归（避免把「首次就没过」重罚两遍）。
    pub fn report_regression(&mut self, goal_id: impl Into<String>, check: impl Into<String>) -> bool {
        let goal_id = goal_id.into();
        let check = check.into();
        // ⚠️ 早期版本用 `goal/phase` 存键、却拿 artifact 去做 ends_with ⇒
        // 永远匹配不上，回归识别形同虚设。改为统一键函数。
        if !self.passed_checks.contains(&check_key(&goal_id, &check)) {
            return false;
        }
        self.entries.push(LedgerEntry::Regression {
            goal_id,
            check,
        });
        true
    }

    /// 记一条未支撑声明（惩罚）。
    pub fn record_claim_without_evidence(
        &mut self,
        goal_id: impl Into<String>,
        phase: GoalPhase,
        claim: impl Into<String>,
    ) {
        self.entries.push(LedgerEntry::UnsupportedClaim {
            goal_id: goal_id.into(),
            phase,
            claim: claim.into(),
        });
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// 逐目标累计分。
    pub fn scores(&self) -> BTreeMap<String, GoalScore> {
        let mut out: BTreeMap<String, GoalScore> = BTreeMap::new();
        for e in &self.entries {
            let s = out.entry(e.goal_id().to_string()).or_default();
            let w = e.weight();
            if w >= 0.0 {
                s.reward += w;
                if let LedgerEntry::Evidence { .. } = e {
                    s.evidence_count += 1;
                }
            } else {
                s.penalty += -w;
                s.violations += 1;
            }
        }
        out
    }

    pub fn score_of(&self, goal_id: &str) -> GoalScore {
        self.scores().get(goal_id).cloned().unwrap_or_default()
    }

    /// 按净值排序目标，**不可信目标沉底**。
    ///
    /// 这是账本接入行为的地方：返回值直接决定目标执行顺序。
    pub fn rank(&self, goal_ids: &[String]) -> Vec<String> {
        let scores = self.scores();
        let key = |g: &String| {
            let s = scores.get(g).cloned().unwrap_or_default();
            // 不可信目标的排序键整体下移，使「堆证据」无法把自己洗回队首。
            let trust = if s.is_untrusted() { -1e6 } else { 0.0 };
            (trust + s.net(), s.evidence_quality())
        };
        let mut v = goal_ids.to_vec();
        v.sort_by(|a, b| key(b).partial_cmp(&key(a)).unwrap_or(std::cmp::Ordering::Equal));
        v
    }

    /// 全局净值（用于健康度上报）。
    pub fn net(&self) -> f64 {
        self.entries.iter().map(LedgerEntry::weight).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(id: &str) -> String {
        id.to_string()
    }

    #[test]
    fn evidence_only_earns_reward() {
        let mut l = RewardLedger::new();
        l.record_evidence("g1", GoalPhase::Execute, EvidenceType::TestPass, "cargo test 13,567 passed");
        let s = l.score_of("g1");
        assert_eq!(s.reward, 2.0);
        assert_eq!(s.penalty, 0.0);
        assert_eq!(s.evidence_count, 1);
    }

    /// 红线 1：空产物必须拿不到正向权重。
    #[test]
    fn empty_artifact_is_downgraded_to_penalty() {
        let mut l = RewardLedger::new();
        l.record_evidence("g1", GoalPhase::Verify, EvidenceType::PropertyProof, "   ");
        let s = l.score_of("g1");
        assert_eq!(s.reward, 0.0, "无产物不得计正向证据");
        assert!(s.penalty > 0.0, "应记惩罚");
        assert_eq!(s.violations, 1);
    }

    /// 红线 3：惩罚量级必须大于任一正向证据 ⇒ 不作弊严格占优。
    #[test]
    fn penalty_dominates_any_single_evidence() {
        let max_positive = [EvidenceType::Compilation, EvidenceType::TestPass,
            EvidenceType::PropertyProof, EvidenceType::UserFeedback]
            .iter().map(|k| evidence_weight(*k)).fold(0.0, f64::max);
        assert!(
            LedgerEntry::VerificationFailed {
                goal_id: g("g"), phase: GoalPhase::Verify, reason: g("r")
            }.weight().abs() > max_positive,
            "单次惩罚({}) 须大于最强正向证据({max_positive})",
            LedgerEntry::VerificationFailed {
                goal_id: g("g"), phase: GoalPhase::Verify, reason: g("r")
            }.weight().abs()
        );
    }

    /// 红线 2：回归必须被识别并重罚。
    #[test]
    fn regression_is_detected_only_after_a_pass() {
        let mut l = RewardLedger::new();
        l.record_evidence("g1", GoalPhase::Verify, EvidenceType::TestPass, "check:layer-deps");
        assert!(l.report_regression("g1", "check:layer-deps"), "曾通过 ⇒ 应识别为回归");
        // 从未通过过的检查，不应被双重重罚
        assert!(!l.report_regression("g1", "check:never-ran"));
        let s = l.score_of("g1");
        assert!(s.penalty >= 5.0, "回归惩罚应 >= 5.0，实际 {}", s.penalty);
    }

    /// 违规不能靠后续堆证据洗白。
    #[test]
    fn violations_cannot_be_washed_away_by_evidence() {
        let mut l = RewardLedger::new();
        l.record_claim_without_evidence("g1", GoalPhase::Verify, "我说门是绿的");
        l.record_evidence("g1", GoalPhase::Verify, EvidenceType::Compilation, "cargo check RC=0");
        let s = l.score_of("g1");
        assert!(s.is_untrusted(), "净值被惩罚主导 ⇒ 应判不可信");
    }

    /// 账本必须真的改变顺序（否则它只是个日志）。
    #[test]
    fn ledger_actually_changes_goal_order() {
        let mut l = RewardLedger::new();
        l.record_evidence("good", GoalPhase::Execute, EvidenceType::UserFeedback, "用户确认可用");
        l.record_verification_failure("bad", GoalPhase::Verify, "3 个测试红");

        let ids = vec![g("bad"), g("good")];
        assert_eq!(l.rank(&ids), vec![g("good"), g("bad")], "净值为正者应排前");
    }

    #[test]
    fn untrusted_goal_sinks_even_with_high_net() {
        let mut l = RewardLedger::new();
        // 坏目标净值刻意为**正**（+6.0），却因一次违规必须沉底 ⇒
        // 这正是 `is_untrusted` 必须按违规次数而非净值符号判定的原因。
        l.record_evidence("bad", GoalPhase::Execute, EvidenceType::UserFeedback, "x");
        l.record_evidence("bad", GoalPhase::Execute, EvidenceType::UserFeedback, "y");
        l.record_claim_without_evidence("bad", GoalPhase::Verify, "我说门是绿的");
        assert!(l.score_of("bad").net() > 0.0, "前置条件：坏目标净值为正");
        assert!(l.score_of("bad").is_untrusted());
        l.record_evidence("ok", GoalPhase::Execute, EvidenceType::Compilation, "z");

        let ranked = l.rank(&[g("bad"), g("ok")]);
        assert_eq!(ranked.last().map(String::as_str), Some("bad"),
            "不可信目标必须沉底，实际顺序 {ranked:?}");
    }

    #[test]
    fn evidence_quality_separates_progress_from_absence() {
        let mut l = RewardLedger::new();
        l.record_evidence("g1", GoalPhase::Execute, EvidenceType::UserFeedback, "u");
        l.record_verification_failure("g2", GoalPhase::Verify, "r");
        assert!(l.score_of("g1").evidence_quality() > 0.0);
        assert_eq!(l.score_of("g2").evidence_quality(), 0.0,
            "只被惩罚的目标证据质量应为 0（区别于「有进展但质量低」）");
    }
}