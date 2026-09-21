//! Loop Runner — 进化闭环主驱动
//!
//! 五阶段循环: observe → orient → decide → act → verify
//! 每阶段产出 PhaseResult, 最终聚合为 CycleResult。

use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::absorber::{AbsorptionResult, KnowledgeAbsorber};
use super::self_evolver::{EvolutionPlan, SelfEvolver};
use super::verifier::{EvolutionVerifier, VerificationResult};

use crate::l6_meta::evolving_evaluator::EvolvingEvaluator;
use crate::neotrix::nt_jev::eval::EvalReport;

/// 进化闭环配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionLoopConfig {
    /// 最大吸收轮次
    pub max_absorption_rounds: usize,
    /// 最小改进阈值 (net_score 提升)
    pub min_improvement_threshold: f64,
    /// 是否启用自动验证
    pub enable_verification: bool,
    /// 阶段超时 (ms)
    pub phase_timeout_ms: u64,
}

impl Default for EvolutionLoopConfig {
    fn default() -> Self {
        Self {
            max_absorption_rounds: 3,
            min_improvement_threshold: 0.01,
            enable_verification: true,
            phase_timeout_ms: 5000,
        }
    }
}

/// 系统快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub capability_count: usize,
    pub overall_score: f64,
    pub growth_phase: String,
    pub active_modules: Vec<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 循环阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CyclePhase {
    Observe,
    Orient,
    Decide,
    Act,
    Verify,
}

impl CyclePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Orient => "orient",
            Self::Decide => "decide",
            Self::Act => "act",
            Self::Verify => "verify",
        }
    }
}

/// 单阶段结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    pub phase: CyclePhase,
    pub duration_ms: u64,
    pub success: bool,
    pub detail: String,
}

/// 改进记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    pub description: String,
    pub phase: CyclePhase,
    pub impact: f64,
}

/// 循环结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleResult {
    pub cycle_id: u64,
    pub phase_results: Vec<PhaseResult>,
    pub improvements: Vec<Improvement>,
    pub duration_ms: u64,
    pub net_score_delta: f64,
}

/// Outcome of one JEV nightly step (serializable for trace archives).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevNightlyOutcome {
    pub target: String,
    pub weighted_score: f64,
    pub criteria_scores: std::collections::HashMap<String, f64>,
    pub absorption: AbsorptionResult,
    /// Rule ids newly applied this night (not previously accumulated).
    pub new_rule_ids: Vec<String>,
    pub improvements: Vec<Improvement>,
}

/// 进化闭环驱动器
pub struct EvolutionLoop {
    config: EvolutionLoopConfig,
    absorber: KnowledgeAbsorber,
    evolver: SelfEvolver,
    verifier: EvolutionVerifier,
    cycle_counter: u64,
}

impl EvolutionLoop {
    pub fn new(config: EvolutionLoopConfig) -> Self {
        Self {
            absorber: KnowledgeAbsorber::new(config.max_absorption_rounds),
            evolver: SelfEvolver::new(),
            verifier: EvolutionVerifier::new(config.min_improvement_threshold),
            config,
            cycle_counter: 0,
        }
    }

    /// 执行一个完整进化循环
    pub fn run_cycle(&mut self, state: &SystemState) -> CycleResult {
        let start = Instant::now();
        self.cycle_counter += 1;
        let cycle_id = self.cycle_counter;

        let mut phase_results = Vec::new();
        let mut improvements = Vec::new();

        // Phase 1: Observe — 吸收外部知识
        let observe_start = Instant::now();
        let (absorption, obs_improvements) = self.observe(state);
        phase_results.push(PhaseResult {
            phase: CyclePhase::Observe,
            duration_ms: observe_start.elapsed().as_millis() as u64,
            success: absorption.rules_extracted > 0,
            detail: format!(
                "extracted={}, applied={}, conflicts={}",
                absorption.rules_extracted,
                absorption.rules_applied,
                absorption.conflicts_resolved
            ),
        });
        improvements.extend(obs_improvements);

        // Phase 2: Orient — 分析能力差距
        let orient_start = Instant::now();
        let (plan, ori_improvements) = self.orient(state);
        phase_results.push(PhaseResult {
            phase: CyclePhase::Orient,
            duration_ms: orient_start.elapsed().as_millis() as u64,
            success: !plan.promotions.is_empty() || !plan.new_modules.is_empty(),
            detail: format!(
                "promotions={}, demotions={}, new_modules={}",
                plan.promotions.len(),
                plan.demotions.len(),
                plan.new_modules.len()
            ),
        });
        improvements.extend(ori_improvements);

        // Phase 3: Decide — 制定执行计划
        let decide_start = Instant::now();
        let decision_detail = self.decide(&plan);
        phase_results.push(PhaseResult {
            phase: CyclePhase::Decide,
            duration_ms: decide_start.elapsed().as_millis() as u64,
            success: true,
            detail: decision_detail,
        });

        // Phase 4: Act — 执行进化动作
        let act_start = Instant::now();
        let new_state = self.act(state, &plan);
        phase_results.push(PhaseResult {
            phase: CyclePhase::Act,
            duration_ms: act_start.elapsed().as_millis() as u64,
            success: true,
            detail: format!("applied {} promotions", plan.promotions.len()),
        });

        // Phase 5: Verify — 验证进化结果
        if self.config.enable_verification {
            let verify_start = Instant::now();
            let verification = self.verify(state, &new_state);
            phase_results.push(PhaseResult {
                phase: CyclePhase::Verify,
                duration_ms: verify_start.elapsed().as_millis() as u64,
                success: verification.regressions.is_empty(),
                detail: format!(
                    "improvements={}, regressions={}, net={:.4}",
                    verification.improvements.len(),
                    verification.regressions.len(),
                    verification.net_score
                ),
            });
        }

        let total_duration = start.elapsed().as_millis() as u64;
        let net_score_delta = new_state.overall_score - state.overall_score;

        CycleResult {
            cycle_id,
            phase_results,
            improvements,
            duration_ms: total_duration,
            net_score_delta,
        }
    }

    /// Observe: 吸收外部知识
    fn observe(&mut self, state: &SystemState) -> (AbsorptionResult, Vec<Improvement>) {
        let source = format!(
            "system_state_{}",
            state.growth_phase
        );
        let result = self.absorber.absorb(&source);
        let mut improvements = Vec::new();
        if result.rules_applied > 0 {
            improvements.push(Improvement {
                description: format!("Absorbed {} rules from knowledge base", result.rules_applied),
                phase: CyclePhase::Observe,
                impact: result.rules_applied as f64 * 0.01,
            });
        }
        (result, improvements)
    }

    /// Orient: 分析能力差距, 制定进化计划
    fn orient(&self, state: &SystemState) -> (EvolutionPlan, Vec<Improvement>) {
        let plan = self.evolver.evolve(state);
        let mut improvements = Vec::new();
        for promo in &plan.promotions {
            improvements.push(Improvement {
                description: format!("Promote {} → C{}", promo.module_id, promo.target_constellation),
                phase: CyclePhase::Orient,
                impact: 0.05,
            });
        }
        (plan, improvements)
    }

    /// Decide: 制定执行计划 (当前为空操作, 逻辑已内嵌)
    fn decide(&self, plan: &EvolutionPlan) -> String {
        format!(
            "Decided: {} promotions, {} demotions, {} new modules",
            plan.promotions.len(),
            plan.demotions.len(),
            plan.new_modules.len()
        )
    }

    /// Act: 执行进化动作, 返回新状态
    fn act(&self, state: &SystemState, plan: &EvolutionPlan) -> SystemState {
        let mut new_state = state.clone();
        // 模拟执行: 每个 promotion 增加 capability_count
        new_state.capability_count += plan.promotions.len();
        // 新模块也增加 capability_count
        new_state.capability_count += plan.new_modules.len();
        // 评分提升
        let boost = plan.promotions.len() as f64 * 0.02
            + plan.new_modules.len() as f64 * 0.01;
        new_state.overall_score = (new_state.overall_score + boost).min(1.0);
        new_state
    }

    /// Verify: 验证进化结果
    fn verify(&self, before: &SystemState, after: &SystemState) -> VerificationResult {
        self.verifier.verify(before, after)
    }

    /// Nightly JEV step: score the report, evolve criteria, absorb tuning rules.
    ///
    /// Landing point of the autonomous loop (`evaluator` is caller-owned so
    /// weights persist across nights; this method only borrows it):
    /// evaluate_jev_report → evolve_criteria → absorb_jev_report.
    pub fn run_jev_nightly(
        &mut self,
        evaluator: &EvolvingEvaluator,
        target: &str,
        report: &EvalReport,
    ) -> JevNightlyOutcome {
        // Score + evolve (Orient/Decide).
        let breakdown = evaluator.evaluate_jev_report(target, report);
        evaluator.evolve_criteria();
        // Absorb prescriptive rules (Observe).
        let absorption = self.absorber.absorb_jev_report(report);
        let new_rule_ids: Vec<String> = {
            let ids = self.absorber.rule_ids();
            let n = absorption.rules_applied.min(ids.len());
            ids[ids.len() - n..].to_vec()
        };
        let mut improvements = Vec::new();
        for id in &new_rule_ids {
            improvements.push(Improvement {
                description: format!("JEV nightly rule: {}", id),
                phase: CyclePhase::Observe,
                impact: 0.02,
            });
        }
        improvements.push(Improvement {
            description: format!("JEV nightly {} scored {:.3}", target, breakdown.weighted_score),
            phase: CyclePhase::Decide,
            impact: breakdown.weighted_score,
        });
        JevNightlyOutcome {
            target: target.to_string(),
            weighted_score: breakdown.weighted_score,
            criteria_scores: breakdown.criteria_scores,
            absorption,
            new_rule_ids,
            improvements,
        }
    }

    /// 获取当前循环计数
    pub fn cycle_count(&self) -> u64 {
        self.cycle_counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_state() -> SystemState {
        SystemState {
            capability_count: 10,
            overall_score: 0.5,
            growth_phase: "C1-UnitTest".into(),
            active_modules: vec!["core".into()],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn run_cycle_produces_five_phases() {
        let mut loop_runner = EvolutionLoop::new(EvolutionLoopConfig::default());
        let state = mock_state();
        let result = loop_runner.run_cycle(&state);
        assert_eq!(result.phase_results.len(), 5);
        assert_eq!(result.phase_results[0].phase, CyclePhase::Observe);
        assert_eq!(result.phase_results[1].phase, CyclePhase::Orient);
        assert_eq!(result.phase_results[2].phase, CyclePhase::Decide);
        assert_eq!(result.phase_results[3].phase, CyclePhase::Act);
        assert_eq!(result.phase_results[4].phase, CyclePhase::Verify);
    }

    #[test]
    fn run_cycle_increments_counter() {
        let mut loop_runner = EvolutionLoop::new(EvolutionLoopConfig::default());
        let state = mock_state();
        assert_eq!(loop_runner.cycle_count(), 0);
        loop_runner.run_cycle(&state);
        assert_eq!(loop_runner.cycle_count(), 1);
        loop_runner.run_cycle(&state);
        assert_eq!(loop_runner.cycle_count(), 2);
    }

    #[test]
    fn run_cycle_produces_positive_score_delta() {
        let mut loop_runner = EvolutionLoop::new(EvolutionLoopConfig::default());
        let state = mock_state();
        let result = loop_runner.run_cycle(&state);
        assert!(result.net_score_delta >= 0.0, "score should not regress");
    }

    #[test]
    fn cycle_result_serializes() {
        let mut loop_runner = EvolutionLoop::new(EvolutionLoopConfig::default());
        let state = mock_state();
        let result = loop_runner.run_cycle(&state);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("cycle_id"));
        assert!(json.contains("phase_results"));
    }

    fn weak_report() -> EvalReport {
        EvalReport {
            n: 40,
            accuracy: 0.5,
            brier: 0.5,
            ece: 0.5,
            coverage_at_p90: 0.2,
            mean_latency_ms: 500.0,
        }
    }

    fn strong_report() -> EvalReport {
        EvalReport {
            n: 40,
            accuracy: 0.95,
            brier: 0.05,
            ece: 0.03,
            coverage_at_p90: 0.9,
            mean_latency_ms: 20.0,
        }
    }

    #[test]
    fn jev_nightly_weak_report_yields_rules() {
        use crate::l6_meta::evolving_evaluator::EvolvingEvaluator;
        let mut loop_runner = EvolutionLoop::new(EvolutionLoopConfig::default());
        let evaluator = EvolvingEvaluator::new(EvolvingEvaluator::jev_criteria(), 0.05);
        let out = loop_runner.run_jev_nightly(&evaluator, "nightly-1", &weak_report());
        assert_eq!(out.new_rule_ids.len(), 5);
        assert!(!out.improvements.is_empty());
        assert_eq!(out.target, "nightly-1");
        assert_eq!(evaluator.evaluation_count(), 1);
        let json = serde_json::to_string(&out).unwrap();
        assert!(json.contains("weighted_score"));
    }

    #[test]
    fn jev_nightly_strong_report_scores_higher() {
        use crate::l6_meta::evolving_evaluator::EvolvingEvaluator;
        let mut loop_runner = EvolutionLoop::new(EvolutionLoopConfig::default());
        let evaluator = EvolvingEvaluator::new(EvolvingEvaluator::jev_criteria(), 0.05);
        let weak = loop_runner.run_jev_nightly(&evaluator, "n1", &weak_report());
        let strong = loop_runner.run_jev_nightly(&evaluator, "n2", &strong_report());
        assert!(strong.weighted_score > weak.weighted_score);
        assert!(strong.new_rule_ids.is_empty());
        // Second weak run: rules already accumulated → nothing new.
        let weak2 = loop_runner.run_jev_nightly(&evaluator, "n3", &weak_report());
        assert!(weak2.new_rule_ids.is_empty());
        assert_eq!(evaluator.evaluation_count(), 3);
    }
}
