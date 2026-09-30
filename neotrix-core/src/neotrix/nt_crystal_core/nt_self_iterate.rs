//! NT-SELF-ITERATE — 晶体自我迭代闭环（D2 结晶复用 + D5 任务验收落地）
//!
//! 把散件串成“越跑越像人”的闭环，每轮：
//! ```text
//! orchestrator.tick（观测→自醒→进化）→ 结晶率度量 → NtEvalLoop::run
//!   → triage 动作 → 自适应（awaken_rounds ±1 爬坡，钳 [min,max]）
//!   → Adaptation 记进化层 → 收敛检查
//! ```
//! 收敛判据：eval accuracy 连续 `patience` 轮变化 < `eps` 即停；上限 `max_rounds`。
//! 度量（PBT helper 范式）：yield=chosen/proposed，crystallize=new_patterns/chosen。
//! eval 数据由调用方按轮提供（`eval_provider`），内核只管调度与度量。
//! 无 unwrap / expect / panic；无 `[]` 索引。

use super::consciousness::{CrystalConsciousness, MemoryType};
use super::nt_eval_loop::{NtEvalLoop, THRESHOLD_MAX, THRESHOLD_MIN};
use super::{CrystalCore, NtOrchestrator};
use crate::l5_cognition::nt_jev::eval::{EvalCase, EvalPrediction};

/// 自迭代配置
#[derive(Debug, Clone)]
pub struct NtSelfIterateConfig {
    /// 上限轮数
    pub max_rounds: usize,
    /// 收敛阈值（accuracy 轮间变化小于此值算稳定）
    pub eps: f64,
    /// 连续稳定多少轮算收敛
    pub patience: usize,
    /// 评估能力域（记入重训请求）
    pub capability: String,
    /// 初始门限
    pub init_threshold: f64,
    /// 自醒轮数下限
    pub min_awaken_rounds: usize,
    /// 自醒轮数上限
    pub max_awaken_rounds: usize,
}

impl Default for NtSelfIterateConfig {
    fn default() -> Self {
        Self {
            max_rounds: 6,
            eps: 0.02,
            patience: 2,
            capability: "reasoning".to_string(),
            init_threshold: 0.7,
            min_awaken_rounds: 1,
            max_awaken_rounds: 8,
        }
    }
}

/// 单轮报告
#[derive(Debug, Clone, Default)]
pub struct RoundReport {
    pub round: u64,
    pub proposed: usize,
    pub chosen: usize,
    pub rejected: usize,
    /// 学习速度：chosen/proposed
    pub yield_rate: f64,
    /// 本轮结束时 Pattern 记忆总数
    pub patterns_total: usize,
    /// 本轮新增 Pattern 数
    pub new_patterns: usize,
    /// 结晶率：new_patterns/chosen（PBT helper 范式）
    pub crystallize_rate: f64,
    pub accuracy: f64,
    pub ece: f64,
    pub threshold: f64,
    pub awaken_rounds: usize,
    pub stable_streak: usize,
}

/// 自迭代总报告
#[derive(Debug, Default)]
pub struct NtSelfIterateReport {
    pub rounds_run: usize,
    pub converged: bool,
    pub final_accuracy: f64,
    pub final_ece: f64,
    pub final_threshold: f64,
    pub final_awaken_rounds: usize,
    pub adaptations: usize,
    pub history: Vec<RoundReport>,
}

fn count_patterns(consciousness: &CrystalConsciousness) -> usize {
    consciousness
        .memories
        .values()
        .filter(|m| m.memory_type == MemoryType::Pattern)
        .count()
}

/// 晶体自我迭代循环（无状态 associated fn；状态全在入参里流转）。
pub struct NtSelfIterate;

impl NtSelfIterate {
    /// 跑完闭环：按轮 tick → 度量 → 评估 → 自适应 → 收敛即停。
    ///
    /// observations: 每轮复用的外部观测（空则纯自醒）；
    /// eval_provider: 按（意识只读快照，轮号）产出 (cases, preds)，决定本轮的 accuracy/ece。
    ///   只读意识 + 与三件套 &mut 无借用冲突，eval 看到的是 evolving 的实时状态（真在线评估）。
    pub fn run(
        orchestrator: &mut NtOrchestrator,
        consciousness: &mut CrystalConsciousness,
        core: &mut CrystalCore,
        observations: &[(String, String, f64)],
        eval_provider: &mut dyn FnMut(
            &CrystalConsciousness,
            u64,
        ) -> (Vec<EvalCase>, Vec<EvalPrediction>),
        config: &NtSelfIterateConfig,
    ) -> NtSelfIterateReport {
        let mut rep = NtSelfIterateReport::default();
        let mut threshold = config.init_threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX);
        let mut awaken_rounds = orchestrator.awaken_rounds().max(1);
        orchestrator.set_awaken_rounds(awaken_rounds);
        let mut prev_acc: Option<f64> = None;
        let mut streak = 0usize;
        let patience = config.patience.max(1);
        let lo = config.min_awaken_rounds.max(1);
        let hi = config.max_awaken_rounds.max(lo);

        for round in 0..config.max_rounds.max(1) {
            let round_idx = round as u64;
            let patterns_before = count_patterns(consciousness);
            let tick = orchestrator.tick(consciousness, core, observations);
            let patterns_after = count_patterns(consciousness);
            let new_patterns = patterns_after.saturating_sub(patterns_before);

            let (cases, preds) = eval_provider(consciousness, round_idx);
            let eval = NtEvalLoop::run(&cases, &preds, &config.capability, threshold, core);
            threshold = eval.new_threshold;

            let yield_rate = tick.chosen as f64 / tick.proposed.max(1) as f64;
            let crystallize_rate = new_patterns as f64 / tick.chosen.max(1) as f64;
            let delta = eval.accuracy - prev_acc.unwrap_or(eval.accuracy);

            // 自适应爬坡：涨且太挑 → 多醒一轮；跌 → 收一轮稳住
            if round > 0 {
                if delta > 0.0 && yield_rate < 0.5 && awaken_rounds < hi {
                    awaken_rounds += 1;
                    orchestrator.set_awaken_rounds(awaken_rounds);
                    core.evolution.record_adaptation(
                        format!("self-iterate round {round_idx} acc +{delta:.3}"),
                        format!("awaken_rounds → {awaken_rounds} (yield {yield_rate:.2} < 0.5)"),
                        "pending next round".to_string(),
                        delta,
                    );
                    rep.adaptations += 1;
                } else if delta < 0.0 && awaken_rounds > lo {
                    awaken_rounds -= 1;
                    orchestrator.set_awaken_rounds(awaken_rounds);
                    core.evolution.record_adaptation(
                        format!("self-iterate round {round_idx} acc {delta:.3}"),
                        format!("awaken_rounds → {awaken_rounds} (stabilize)"),
                        "pending next round".to_string(),
                        delta,
                    );
                    rep.adaptations += 1;
                }
            }

            // 收敛检查
            if round > 0 && delta.abs() < config.eps {
                streak += 1;
            } else if round > 0 {
                streak = 0;
            }
            prev_acc = Some(eval.accuracy);

            rep.history.push(RoundReport {
                round: round_idx,
                proposed: tick.proposed,
                chosen: tick.chosen,
                rejected: tick.rejected,
                yield_rate,
                patterns_total: patterns_after,
                new_patterns,
                crystallize_rate,
                accuracy: eval.accuracy,
                ece: eval.ece,
                threshold,
                awaken_rounds,
                stable_streak: streak,
            });
            rep.rounds_run += 1;
            if streak >= patience {
                rep.converged = true;
                break;
            }
        }

        if let Some(last) = rep.history.last() {
            rep.final_accuracy = last.accuracy;
            rep.final_ece = last.ece;
            rep.final_threshold = last.threshold;
            rep.final_awaken_rounds = last.awaken_rounds;
        }
        rep
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_crystal_core::NtOrchestratorConfig;
    use crate::l5_cognition::nt_jev::eval::GoldAnswer;
    use crate::l5_cognition::nt_jev::primitives::{DecisionStatus, JevDecision, NoulAnswer};

    fn noul_pair(id: &str, gold: bool, p: f64) -> (EvalCase, EvalPrediction) {
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

    fn quiet_orchestrator() -> NtOrchestrator {
        NtOrchestrator::new(NtOrchestratorConfig {
            save_on_evolve: false,
            evolve_every: 100,
            ..Default::default()
        })
    }

    #[test]
    fn test_converges_on_stable_perfect_eval() {
        // 每轮全对：acc 恒 1.0 → patience=2 时第 3 轮收敛
        let mut orch = quiet_orchestrator();
        let mut c = CrystalConsciousness::new("t");
        let mut core = CrystalCore::new("t");
        let cfg = NtSelfIterateConfig {
            max_rounds: 6,
            patience: 2,
            ..Default::default()
        };
        let mut provider = |_: &CrystalConsciousness, round: u64| {
            split(
                (0..10)
                    .map(|i| noul_pair(&format!("r{round}c{i}"), true, 0.95))
                    .collect(),
            )
        };
        let rep = NtSelfIterate::run(&mut orch, &mut c, &mut core, &[], &mut provider, &cfg);
        assert!(rep.converged, "stable perfect eval must converge");
        assert_eq!(rep.rounds_run, 3, "patience=2 → stop after 3 rounds");
        assert!((rep.final_accuracy - 1.0).abs() < 1e-9);
        assert!(
            (rep.final_threshold - 0.7).abs() < 1e-9,
            "perfect run keeps threshold"
        );
    }

    #[test]
    fn test_adapts_rounds_down_on_accuracy_drop() {
        // round0 全对 → round1 起全错：delta<0 → awaken_rounds 4→3，只动一次
        let mut orch = quiet_orchestrator();
        let mut c = CrystalConsciousness::new("t");
        let mut core = CrystalCore::new("t");
        let cfg = NtSelfIterateConfig {
            max_rounds: 4,
            patience: 10,
            ..Default::default()
        };
        let mut provider = |_: &CrystalConsciousness, round: u64| {
            let gold = round == 0;
            split(
                (0..10)
                    .map(|i| noul_pair(&format!("r{round}c{i}"), gold, 0.95))
                    .collect(),
            )
        };
        let rep = NtSelfIterate::run(&mut orch, &mut c, &mut core, &[], &mut provider, &cfg);
        assert!(!rep.converged);
        assert_eq!(rep.rounds_run, 4);
        assert_eq!(rep.final_awaken_rounds, 3, "drop must shrink rounds once");
        assert_eq!(rep.adaptations, 1);
        assert_eq!(orch.awaken_rounds(), 3, "orchestrator mirrors adaptation");
    }

    #[test]
    fn test_patterns_total_and_rates_bounded() {
        // 预埋 2 个 Pattern：history 能数到；各率 ∈ [0,1]
        let mut orch = quiet_orchestrator();
        let mut c = CrystalConsciousness::new("t");
        let _ = c.remember("火焰 燃烧 释放 热量", MemoryType::Pattern, "physics", 0.9);
        let _ = c.remember("光合 叶绿素 阳光", MemoryType::Pattern, "bio", 0.9);
        let mut core = CrystalCore::new("t");
        let cfg = NtSelfIterateConfig {
            max_rounds: 1,
            ..Default::default()
        };
        let mut provider = |_: &CrystalConsciousness, round: u64| {
            split(
                (0..10)
                    .map(|i| noul_pair(&format!("r{round}c{i}"), true, 0.95))
                    .collect(),
            )
        };
        let rep = NtSelfIterate::run(&mut orch, &mut c, &mut core, &[], &mut provider, &cfg);
        assert_eq!(rep.rounds_run, 1);
        assert!(!rep.converged);
        let first = rep.history.first();
        assert!(first.is_some(), "must record one round");
        if let Some(r) = first {
            assert!(r.patterns_total >= 2, "seeded patterns counted");
            assert!((0.0..=1.0).contains(&r.yield_rate));
            assert!((0.0..=1.0).contains(&r.crystallize_rate));
        }
    }

    #[test]
    fn test_empty_eval_terminates() {
        // 空评估不卡死：上限轮数内必出
        let mut orch = quiet_orchestrator();
        let mut c = CrystalConsciousness::new("t");
        let mut core = CrystalCore::new("t");
        let cfg = NtSelfIterateConfig {
            max_rounds: 3,
            ..Default::default()
        };
        let mut provider = |_: &CrystalConsciousness, _round: u64| (Vec::new(), Vec::new());
        let rep = NtSelfIterate::run(&mut orch, &mut c, &mut core, &[], &mut provider, &cfg);
        assert!(rep.rounds_run <= 3);
        assert_eq!(rep.history.len(), rep.rounds_run);
    }
}
