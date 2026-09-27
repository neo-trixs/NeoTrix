//! NT-EVAL-LOOP — 评估闭环（Phase 2 缺陷修复：#3 eval→evolve 断路）
//!
//! `evaluate()` 测完不管 → 本模块把 `EvalReport` 变成动作：
//! - ECE 高（>0.15）→ 保守化（门限 +step）+ 建议 Platt 重校准；
//! - accuracy 低（<0.7）→ RequestRetrain（缺数据/缺能力信号）；
//! - coverage 低但 accuracy 高 → 适度放开（门限 −step，少弃权多干活）；
//! - Brier 高（>0.25）→ 建议重校准（概率质量差）。
//! 门限钳 [0.5, 0.95]，步长 0.05；每次动作记一条 Adaptation 进进化层。
//! 决策纯函数（triage），应用函数（run）只做记录+返回新门限，不直接改 gate。
//! 无 unwrap / expect / panic；无 `[]` 索引。

use super::CrystalCore;
use crate::neotrix::nt_jev::eval::{evaluate, EvalCase, EvalPrediction, EvalReport};

/// ECE 失准线（高于此值认为校准坏了）
pub const ECE_BAD_FLOOR: f64 = 0.15;
/// 准确率请求重训线
pub const ACC_RETRAIN_FLOOR: f64 = 0.7;
/// 高准确但过度弃权线
pub const COVERAGE_TIMID_FLOOR: f64 = 0.5;
pub const ACC_BOLD_FLOOR: f64 = 0.8;
/// Brier 概率质量差线
pub const BRIER_BAD_FLOOR: f64 = 0.25;
/// 门限步长与钳位
pub const THRESHOLD_STEP: f64 = 0.05;
pub const THRESHOLD_MIN: f64 = 0.5;
pub const THRESHOLD_MAX: f64 = 0.95;

/// 评估动作
#[derive(Debug, Clone, PartialEq)]
pub enum EvalAction {
    RaiseThreshold { from: f64, to: f64, reason: String },
    LowerThreshold { from: f64, to: f64, reason: String },
    RequestRetrain { capability: String, reason: String },
    SuggestRecalibration { ece: f64, brier: f64 },
}

/// 单轮评估报告（含动作与新门限）
#[derive(Debug, Default)]
pub struct EvalLoopReport {
    pub evaluated: usize,
    pub accuracy: f64,
    pub ece: f64,
    pub brier: f64,
    pub actions: Vec<EvalAction>,
    pub new_threshold: f64,
}

pub struct NtEvalLoop;

impl NtEvalLoop {
    /// 纯决策：报告 + 当前门限 → 动作（门限只算不落，由 run 应用）
    pub fn triage(
        report: &EvalReport,
        capability: &str,
        current_threshold: f64,
    ) -> (Vec<EvalAction>, f64) {
        let mut actions = Vec::new();
        if report.n == 0 {
            return (actions, current_threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX));
        }
        let mut threshold = current_threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX);

        if report.ece > ECE_BAD_FLOOR {
            let to = (threshold + THRESHOLD_STEP).min(THRESHOLD_MAX);
            actions.push(EvalAction::RaiseThreshold {
                from: threshold,
                to,
                reason: format!("ECE {:.3} > {:.2}, be conservative", report.ece, ECE_BAD_FLOOR),
            });
            threshold = to;
            actions.push(EvalAction::SuggestRecalibration {
                ece: report.ece,
                brier: report.brier,
            });
        }
        if report.accuracy < ACC_RETRAIN_FLOOR {
            actions.push(EvalAction::RequestRetrain {
                capability: capability.to_string(),
                reason: format!(
                    "accuracy {:.3} < {:.2}, needs data/capability",
                    report.accuracy, ACC_RETRAIN_FLOOR
                ),
            });
        }
        if report.coverage_at_p90 < COVERAGE_TIMID_FLOOR && report.accuracy >= ACC_BOLD_FLOOR {
            let to = (threshold - THRESHOLD_STEP).max(THRESHOLD_MIN);
            actions.push(EvalAction::LowerThreshold {
                from: threshold,
                to,
                reason: format!(
                    "accurate ({:.3}) but abstinent (coverage {:.3}), be bolder",
                    report.accuracy, report.coverage_at_p90
                ),
            });
            threshold = to;
        }
        if report.brier > BRIER_BAD_FLOOR
            && !actions.iter().any(|a| matches!(a, EvalAction::SuggestRecalibration { .. }))
        {
            actions.push(EvalAction::SuggestRecalibration {
                ece: report.ece,
                brier: report.brier,
            });
        }
        (actions, threshold)
    }

    /// 跑一轮：评估 → 决策 → 记 Adaptation → 返回报告。
    ///
    /// capability：本轮评估的能力域（如 "reasoning"），记入重训请求。
    pub fn run(
        cases: &[EvalCase],
        preds: &[EvalPrediction],
        capability: &str,
        current_threshold: f64,
        core: &mut CrystalCore,
    ) -> EvalLoopReport {
        let report = evaluate(cases, preds);
        let mut out = EvalLoopReport {
            evaluated: report.n,
            accuracy: report.accuracy,
            ece: report.ece,
            brier: report.brier,
            new_threshold: current_threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX),
            ..Default::default()
        };
        if report.n == 0 {
            return out;
        }
        let (actions, new_threshold) = Self::triage(&report, capability, current_threshold);
        out.new_threshold = new_threshold;
        for a in &actions {
            let (trigger, change, delta) = match a {
                EvalAction::RaiseThreshold { from, to, reason } => (
                    format!("eval ECE {:.3}", report.ece),
                    format!("threshold {from:.2} → {to:.2}: {reason}"),
                    to - from,
                ),
                EvalAction::LowerThreshold { from, to, reason } => (
                    format!("eval coverage {:.3}", report.coverage_at_p90),
                    format!("threshold {from:.2} → {to:.2}: {reason}"),
                    from - to,
                ),
                EvalAction::RequestRetrain { capability, reason } => (
                    format!("eval accuracy {:.3}", report.accuracy),
                    format!("retrain {capability}: {reason}"),
                    0.0,
                ),
                EvalAction::SuggestRecalibration { ece, brier } => (
                    format!("eval ECE {ece:.3} / Brier {brier:.3}"),
                    "suggest Platt recalibration on 200-question cal set".to_string(),
                    0.0,
                ),
            };
            core.evolution.record_adaptation(trigger, change, "pending next eval".to_string(), delta);
        }
        out.actions = actions;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_jev::eval::GoldAnswer;
    use crate::neotrix::nt_jev::primitives::{DecisionStatus, JevDecision, NoulAnswer};

    fn noul_case(id: &str, gold: bool, p: f64) -> (EvalCase, EvalPrediction) {
        (
            EvalCase {
                id: id.to_string(),
                gold: GoldAnswer::Noul(gold),
            },
            EvalPrediction {
                case_id: id.to_string(),
                decision: JevDecision::Noul(NoulAnswer {
                    noul: p.clamp(0.0, 1.0),
                    needs_review: false,
                    reason: None,
                    status: DecisionStatus::Selected,
                }),
                latency_ms: 1,
            },
        )
    }

    fn split(pairs: Vec<(EvalCase, EvalPrediction)>) -> (Vec<EvalCase>, Vec<EvalPrediction>) {
        let mut cs = Vec::new();
        let mut ps = Vec::new();
        for (c, p) in pairs {
            cs.push(c);
            ps.push(p);
        }
        (cs, ps)
    }

    #[test]
    fn test_empty_eval_no_actions() {
        let mut core = CrystalCore::new("t");
        let rep = NtEvalLoop::run(&[], &[], "reasoning", 0.7, &mut core);
        assert_eq!(rep.evaluated, 0);
        assert!(rep.actions.is_empty());
        assert!((rep.new_threshold - 0.7).abs() < 1e-9);
        assert!(core.evolution.adaptations.is_empty());
    }

    #[test]
    fn test_perfect_score_no_actions() {
        // 全对且高置信：accuracy=1, ECE≈0, coverage=1 → 无动作
        let pairs: Vec<(EvalCase, EvalPrediction)> = (0..10)
            .map(|i| noul_case(&format!("c{i}"), true, 0.95))
            .collect();
        let (cs, ps) = split(pairs);
        let mut core = CrystalCore::new("t");
        let rep = NtEvalLoop::run(&cs, &ps, "reasoning", 0.7, &mut core);
        assert_eq!(rep.evaluated, 10);
        assert!((rep.accuracy - 1.0).abs() < 1e-9);
        assert!(rep.actions.is_empty(), "perfect run needs no action: {:?}", rep.actions);
    }

    #[test]
    fn test_low_accuracy_requests_retrain() {
        // 全错：accuracy=0 → 重训；ECE 也会高（0.95自信全错）→ 保守化+重校准
        let pairs: Vec<(EvalCase, EvalPrediction)> = (0..10)
            .map(|i| noul_case(&format!("c{i}"), false, 0.95))
            .collect();
        let (cs, ps) = split(pairs);
        let mut core = CrystalCore::new("t");
        let rep = NtEvalLoop::run(&cs, &ps, "reasoning", 0.7, &mut core);
        assert!(rep.actions.iter().any(|a| matches!(a, EvalAction::RequestRetrain { .. })));
        assert!(rep.actions.iter().any(|a| matches!(a, EvalAction::RaiseThreshold { .. })));
        assert!(!core.evolution.adaptations.is_empty());
    }

    #[test]
    fn test_miscalibrated_raises_threshold() {
        // 系统性过自信：报0.9，实际对60% → ECE 高 → 保守化
        let pairs: Vec<(EvalCase, EvalPrediction)> = (0..100)
            .map(|i| noul_case(&format!("c{i}"), i % 10 < 6, 0.9))
            .collect();
        let (cs, ps) = split(pairs);
        let mut core = CrystalCore::new("t");
        let rep = NtEvalLoop::run(&cs, &ps, "reasoning", 0.7, &mut core);
        assert!(rep.ece > 0.1, "this setup must miscalibrate, ece={}", rep.ece);
        let raise = rep.actions.iter().find_map(|a| match a {
            EvalAction::RaiseThreshold { from, to, .. } => Some((*from, *to)),
            _ => None,
        });
        assert!(raise.is_some(), "miscalibration must raise threshold");
        let (from, to) = raise.unwrap();
        assert!((from - 0.7).abs() < 1e-9);
        assert!((to - 0.75).abs() < 1e-9);
        assert!((rep.new_threshold - 0.75).abs() < 1e-9);
    }

    #[test]
    fn test_threshold_clamps_at_bounds() {
        let pairs: Vec<(EvalCase, EvalPrediction)> = (0..10)
            .map(|i| noul_case(&format!("c{i}"), false, 0.95))
            .collect();
        let (cs, ps) = split(pairs);
        let mut core = CrystalCore::new("t");
        // 已在上界：不再涨
        let rep = NtEvalLoop::run(&cs, &ps, "reasoning", 0.95, &mut core);
        assert!((rep.new_threshold - 0.95).abs() < 1e-9);
        // 越界输入被钳
        let rep2 = NtEvalLoop::run(&cs, &ps, "reasoning", 99.0, &mut core);
        assert!(rep2.new_threshold <= THRESHOLD_MAX + 1e-9);
    }

    #[test]
    fn test_adaptations_recorded_with_ids() {
        let pairs: Vec<(EvalCase, EvalPrediction)> = (0..10)
            .map(|i| noul_case(&format!("c{i}"), false, 0.95))
            .collect();
        let (cs, ps) = split(pairs);
        let mut core = CrystalCore::new("t");
        let rep = NtEvalLoop::run(&cs, &ps, "reasoning", 0.7, &mut core);
        assert_eq!(core.evolution.adaptations.len(), rep.actions.len());
        assert!(core.evolution.adaptations.iter().all(|a| a.id.starts_with("ADP-")));
    }
}
