use std::collections::VecDeque;
use std::time::Instant;

use super::bidirectional_pruner::{BidirectionalPruner, PrunerStats};
use super::counterfactual_simulator::{
    CounterfactualSimulator, CounterfactualStats, CounterfactualType,
};
use super::dead_end_detector::{DeadEndDetector, DeadEndStats};
use super::mcts_reasoner::{MctsReasoner, MctsStats};
use super::process_reward_model::{PrmStats, ProcessRewardModel, ReasoningStep, StepType};
use super::strategy_selector::{
    ReasoningStrategy, SelectorStats, SelfHealingSelector,
};
use super::vsa_blackboard::{ExpertType, Hypothesis, VsaBlackboard};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub enable_mcts: bool,
    pub enable_prm: bool,
    pub enable_pruner: bool,
    pub enable_selector: bool,
    pub enable_dead_end: bool,
    pub enable_counterfactual: bool,
    pub enable_curiosity: bool,
    pub max_pipeline_duration_us: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_mcts: true,
            enable_prm: true,
            enable_pruner: true,
            enable_selector: true,
            enable_dead_end: true,
            enable_counterfactual: false,
            enable_curiosity: false,
            max_pipeline_duration_us: 500_000,
        }
    }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PipelineReport {
    pub mcts_stats: Option<MctsStats>,
    pub prm_stats: Option<PrmStats>,
    pub pruner_stats: Option<PrunerStats>,
    pub selector_stats: Option<SelectorStats>,
    pub dead_end_stats: Option<DeadEndStats>,
    pub counterfactual_stats: Option<CounterfactualStats>,
    pub total_duration_us: u64,
    pub stages_completed: Vec<String>,
    pub stages_skipped: Vec<String>,
    pub stages_failed: Vec<String>,
    pub had_dead_end: bool,
    pub pipeline_id: u64,
}

// ---------------------------------------------------------------------------
// Pipeline
// ---------------------------------------------------------------------------

pub struct ReasoningPipeline {
    pub config: PipelineConfig,
    pub mcts: Option<MctsReasoner>,
    pub prm: Option<ProcessRewardModel>,
    pub pruner: Option<BidirectionalPruner>,
    pub selector: Option<SelfHealingSelector>,
    pub dead_end: Option<DeadEndDetector>,
    pub counterfactual: Option<CounterfactualSimulator>,
    pub pipeline_count: u64,
    pub report_history: VecDeque<PipelineReport>,
}

impl ReasoningPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            mcts: None,
            prm: None,
            pruner: None,
            selector: None,
            dead_end: None,
            counterfactual: None,
            pipeline_count: 0,
            report_history: VecDeque::new(),
        }
    }

    // -- builder methods ---------------------------------------------------

    pub fn with_mcts(mut self, m: MctsReasoner) -> Self {
        self.mcts = Some(m);
        self
    }

    pub fn with_prm(mut self, p: ProcessRewardModel) -> Self {
        self.prm = Some(p);
        self
    }

    pub fn with_pruner(mut self, p: BidirectionalPruner) -> Self {
        self.pruner = Some(p);
        self
    }

    pub fn with_selector(mut self, s: SelfHealingSelector) -> Self {
        self.selector = Some(s);
        self
    }

    pub fn with_dead_end(mut self, d: DeadEndDetector) -> Self {
        self.dead_end = Some(d);
        self
    }

    pub fn with_counterfactual(mut self, c: CounterfactualSimulator) -> Self {
        self.counterfactual = Some(c);
        self
    }

    // -- run ---------------------------------------------------------------

    pub fn run_pipeline(&mut self, input_hypothesis: &Hypothesis) -> PipelineReport {
        let start = Instant::now();
        let pipeline_id = self.pipeline_count;
        self.pipeline_count += 1;

        let mut stages_completed: Vec<String> = Vec::new();
        let mut stages_skipped: Vec<String> = Vec::new();
        let mut stages_failed: Vec<String> = Vec::new();

        // Stage 1 — Selector
        let strategy = self.stage_selector();
        if self.config.enable_selector && self.selector.is_some() {
            stages_completed.push("selector".into());
        } else {
            stages_skipped.push("selector".into());
        }

        // Stage 2 — MCTS
        let mcts_hypotheses = self.stage_mcts(input_hypothesis, strategy);
        match (&self.config.enable_mcts, &self.mcts) {
            (true, Some(_)) => stages_completed.push("mcts".into()),
            _ => stages_skipped.push("mcts".into()),
        }

        // Stages 3-6 depend on MCTS producing hypotheses
        let mcts_stats = self.mcts.as_ref().map(|m| m.stats());

        let (prm_stats, pruner_stats, dead_end_stats, cf_stats, had_dead_end) =
            if let Some(ref hyps) = mcts_hypotheses {
                // Stage 3 — PRM
                let prm = self.stage_prm(hyps);
                if prm.is_some() {
                    stages_completed.push("prm".into());
                } else if !self.config.enable_prm {
                    stages_skipped.push("prm".into());
                } else {
                    stages_failed.push("prm".into());
                }

                // Stage 4 — Pruner
                let pruner = self.stage_pruner(hyps);
                if pruner.is_some() {
                    stages_completed.push("pruner".into());
                } else if !self.config.enable_pruner {
                    stages_skipped.push("pruner".into());
                } else {
                    stages_failed.push("pruner".into());
                }

                // Stage 5 — Dead-end
                let de = self.stage_dead_end(hyps);
                let had_de = de
                    .as_ref()
                    .map(|d| d.dead_ends_detected > 0)
                    .unwrap_or(false);
                if de.is_some() {
                    stages_completed.push("dead_end".into());
                } else if !self.config.enable_dead_end {
                    stages_skipped.push("dead_end".into());
                } else {
                    stages_failed.push("dead_end".into());
                }

                // Stage 6 — Counterfactual (only if we have at least one hypothesis)
                let cf = if !hyps.is_empty() {
                    let r = self.stage_counterfactual(&hyps[0]);
                    if r.is_some() {
                        stages_completed.push("counterfactual".into());
                    } else if !self.config.enable_counterfactual {
                        stages_skipped.push("counterfactual".into());
                    } else {
                        stages_failed.push("counterfactual".into());
                    }
                    r
                } else {
                    stages_skipped.push("counterfactual".into());
                    None
                };

                (prm, pruner, de, cf, had_de)
            } else {
                stages_skipped.push("prm".into());
                stages_skipped.push("pruner".into());
                stages_skipped.push("dead_end".into());
                stages_skipped.push("counterfactual".into());
                (None, None, None, None, false)
            };

        let selector_stats = if self.config.enable_selector {
            self.selector.as_ref().map(|s| s.stats())
        } else {
            None
        };

        let total_duration_us = start.elapsed().as_micros() as u64;

        let report = PipelineReport {
            mcts_stats,
            prm_stats,
            pruner_stats,
            selector_stats,
            dead_end_stats,
            counterfactual_stats: cf_stats,
            total_duration_us,
            stages_completed,
            stages_skipped,
            stages_failed,
            had_dead_end,
            pipeline_id,
        };

        if self.report_history.len() >= 100 {
            self.report_history.pop_front();
        }
        self.report_history.push_back(report.clone());
        report
    }

    // -- stage methods -----------------------------------------------------

    fn stage_selector(&mut self) -> ReasoningStrategy {
        if !self.config.enable_selector {
            return ReasoningStrategy::Decomposition;
        }
        self.selector
            .as_mut()
            .map(|s| s.select_strategy("reasoning"))
            .unwrap_or(ReasoningStrategy::Decomposition)
    }

    fn stage_mcts(
        &mut self,
        input: &Hypothesis,
        _strategy: ReasoningStrategy,
    ) -> Option<Vec<Hypothesis>> {
        if !self.config.enable_mcts {
            return None;
        }
        self.mcts.as_mut().map(|m| m.search(input.clone()))
    }

    fn stage_prm(&mut self, hypotheses: &[Hypothesis]) -> Option<PrmStats> {
        if !self.config.enable_prm {
            return None;
        }
        let prm = self.prm.as_mut()?;
        for h in hypotheses {
            let step_type = hypothesis_to_step_type(h);
            let step = ReasoningStep::new(prm.steps.len() as u64, step_type_name(h), step_type);
            let step_id = prm.add_step(step);
            if step_id > 0 {
                prm.evaluate_step(step_id);
            }
        }
        Some(prm.stats())
    }

    fn stage_pruner(&mut self, hypotheses: &[Hypothesis]) -> Option<PrunerStats> {
        if !self.config.enable_pruner || hypotheses.is_empty() {
            return None;
        }
        let pruner = self.pruner.as_mut()?;
        let path_id = pruner.start_path(&hypotheses[0]);
        for h in hypotheses.iter().skip(1) {
            pruner.extend_path(path_id, h, h.confidence);
        }
        Some(pruner.stats())
    }

    fn stage_dead_end(&mut self, hypotheses: &[Hypothesis]) -> Option<DeadEndStats> {
        if !self.config.enable_dead_end || hypotheses.is_empty() {
            return None;
        }
        let de = self.dead_end.as_mut()?;
        let mut bb = VsaBlackboard::new(hypotheses.len());
        for h in hypotheses {
            de.record_step(h);
            bb.post_hypothesis(
                h.content.clone(),
                h.confidence,
                h.expert,
                h.supporting_evidence.clone(),
            );
        }
        if let Some(last) = hypotheses.last() {
            de.check_all(last, &bb);
        }
        Some(de.stats())
    }

    fn stage_counterfactual(&mut self, hypothesis: &Hypothesis) -> Option<CounterfactualStats> {
        if !self.config.enable_counterfactual {
            return None;
        }
        let cf = self.counterfactual.as_mut()?;
        let ids = cf.generate_scenarios(
            &hypothesis.content,
            CounterfactualType::InputPerturbation,
            cf.config.max_simulations,
        );
        for id in ids {
            cf.simulate_scenario(id);
        }
        Some(cf.stats())
    }

    // -- statistics --------------------------------------------------------

    pub fn avg_pipeline_duration(&self) -> f64 {
        let n = self.report_history.len();
        if n == 0 {
            return 0.0;
        }
        self.report_history
            .iter()
            .map(|r| r.total_duration_us as f64)
            .sum::<f64>()
            / n as f64
    }

    pub fn total_pipelines_run(&self) -> u64 {
        self.pipeline_count
    }

    pub fn failure_rate(&self) -> f64 {
        let n = self.report_history.len();
        if n == 0 {
            return 0.0;
        }
        let failed = self
            .report_history
            .iter()
            .filter(|r| !r.stages_failed.is_empty())
            .count();
        failed as f64 / n as f64
    }

    pub fn recent_reports(&self, n: usize) -> Vec<&PipelineReport> {
        self.report_history.iter().rev().take(n).collect()
    }
}

// -- helpers ---------------------------------------------------------------

fn hypothesis_to_step_type(h: &Hypothesis) -> StepType {
    match h.expert {
        ExpertType::Analogical => StepType::Analogize,
        ExpertType::Causal => StepType::Infer,
        ExpertType::MultiHop => StepType::Synthesize,
        ExpertType::Contradiction => StepType::Verify,
        ExpertType::Synthesis => StepType::Synthesize,
    }
}

fn step_type_name(h: &Hypothesis) -> String {
    format!("{}_{}", h.id, hypothesis_to_step_type(h).name())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::unix_now_ms;
    use crate::core::nt_core_reasoning::dead_end_detector::DeadEndConfig;
    use crate::core::nt_core_reasoning::mcts_reasoner::MctsConfig;
    use crate::core::nt_core_reasoning::process_reward_model::PrmConfig;
    use crate::core::nt_core_reasoning::bidirectional_pruner::PrunerConfig;
    use crate::core::nt_core_reasoning::strategy_selector::StrategyConfig;

    fn make_hypothesis(id: u64, confidence: f64, expert: ExpertType) -> Hypothesis {
        Hypothesis {
            id,
            content: vec![id as u8; 32],
            confidence,
            expert,
            supporting_evidence: vec![],
            created_at: unix_now_ms(),
            is_contradicted: false,
        }
    }

    fn default_pipeline() -> ReasoningPipeline {
        ReasoningPipeline::new(PipelineConfig {
            enable_mcts: true,
            enable_prm: true,
            enable_pruner: true,
            enable_selector: true,
            enable_dead_end: true,
            enable_counterfactual: false,
            enable_curiosity: false,
            max_pipeline_duration_us: 500_000,
        })
        .with_mcts(MctsReasoner::new(MctsConfig::default()))
        .with_prm(ProcessRewardModel::new(PrmConfig::default()))
        .with_pruner(BidirectionalPruner::new(PrunerConfig::default()))
        .with_selector(SelfHealingSelector::new(StrategyConfig::default()))
        .with_dead_end(DeadEndDetector::new(DeadEndConfig::default()))
    }

    #[test]
    fn test_empty_pipeline_all_disabled() {
        let cfg = PipelineConfig {
            enable_mcts: false,
            enable_prm: false,
            enable_pruner: false,
            enable_selector: false,
            enable_dead_end: false,
            enable_counterfactual: false,
            enable_curiosity: false,
            max_pipeline_duration_us: 500_000,
        };
        let mut p = ReasoningPipeline::new(cfg);
        let h = make_hypothesis(1, 0.8, ExpertType::Synthesis);
        let report = p.run_pipeline(&h);

        assert_eq!(p.total_pipelines_run(), 1);
        assert!(report.mcts_stats.is_none());
        assert!(report.prm_stats.is_none());
        assert!(report.pruner_stats.is_none());
        assert!(report.selector_stats.is_none());
        assert!(report.dead_end_stats.is_none());
        assert!(report.counterfactual_stats.is_none());
        assert!(report.stages_completed.is_empty());
        assert_eq!(report.stages_skipped.len(), 6);
        assert!(report.stages_failed.is_empty());
        assert!(!report.had_dead_end);
        assert!(report.total_duration_us > 0);
    }

    #[test]
    fn test_full_pipeline() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.9, ExpertType::Causal);
        let report = p.run_pipeline(&h);

        assert!(report.mcts_stats.is_some());
        assert!(report.selector_stats.is_some());
        assert_eq!(report.pipeline_id, 0);
        assert!(report.total_duration_us > 0);
    }

    #[test]
    fn test_mcts_only() {
        let cfg = PipelineConfig {
            enable_mcts: true,
            enable_prm: false,
            enable_pruner: false,
            enable_selector: false,
            enable_dead_end: false,
            enable_counterfactual: false,
            enable_curiosity: false,
            max_pipeline_duration_us: 500_000,
        };
        let mut p = ReasoningPipeline::new(cfg).with_mcts(MctsReasoner::new(MctsConfig::default()));
        let h = make_hypothesis(1, 0.85, ExpertType::Analogical);
        let report = p.run_pipeline(&h);

        assert!(report.mcts_stats.is_some());
        assert!(report.prm_stats.is_none());
        assert!(report.pruner_stats.is_none());
        assert!(report.dead_end_stats.is_none());
        assert!(report.counterfactual_stats.is_none());

        let mcts = report.mcts_stats.unwrap();
        assert!(mcts.best_value > 0.0 || mcts.root_visits == 0);
    }

    #[test]
    fn test_mcts_prm() {
        let mut p = default_pipeline().with_prm(ProcessRewardModel::new(PrmConfig::default()));
        let h = make_hypothesis(1, 0.75, ExpertType::MultiHop);
        let report = p.run_pipeline(&h);

        assert!(report.mcts_stats.is_some());
        assert!(report.prm_stats.is_some());
        let prm = report.prm_stats.unwrap();
        assert!(prm.avg_step_reward >= -1.0);
    }

    #[test]
    fn test_dead_end_detection_no_trigger() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.95, ExpertType::Synthesis);
        let report = p.run_pipeline(&h);
        assert!(report.dead_end_stats.is_some());
        assert!(!report.had_dead_end);
    }

    #[test]
    fn test_duration_tracking() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::Causal);
        let report = p.run_pipeline(&h);
        assert!(report.total_duration_us > 0);
        assert!(report.total_duration_us < 10_000_000);
    }

    #[test]
    fn test_pipeline_id_increment() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::Analogical);
        let r1 = p.run_pipeline(&h);
        let r2 = p.run_pipeline(&h);
        let r3 = p.run_pipeline(&h);
        assert_eq!(r1.pipeline_id, 0);
        assert_eq!(r2.pipeline_id, 1);
        assert_eq!(r3.pipeline_id, 2);
    }

    #[test]
    fn test_report_history() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::Causal);
        for _ in 0..5 {
            p.run_pipeline(&h);
        }
        assert_eq!(p.report_history.len(), 5);
        assert_eq!(p.total_pipelines_run(), 5);
    }

    #[test]
    fn test_avg_duration() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::Synthesis);
        for _ in 0..3 {
            p.run_pipeline(&h);
        }
        let avg = p.avg_pipeline_duration();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_recent_reports() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::Analogical);
        for _ in 0..10 {
            p.run_pipeline(&h);
        }
        let recent = p.recent_reports(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].pipeline_id, 9);
        assert_eq!(recent[2].pipeline_id, 7);
    }

    #[test]
    fn test_builder_chain() {
        let p = ReasoningPipeline::new(PipelineConfig::default())
            .with_mcts(MctsReasoner::new(MctsConfig::default()))
            .with_prm(ProcessRewardModel::new(PrmConfig::default()))
            .with_pruner(BidirectionalPruner::new(PrunerConfig::default()))
            .with_selector(SelfHealingSelector::new(StrategyConfig::default()))
            .with_dead_end(DeadEndDetector::new(DeadEndConfig::default()));

        assert!(p.mcts.is_some());
        assert!(p.prm.is_some());
        assert!(p.pruner.is_some());
        assert!(p.selector.is_some());
        assert!(p.dead_end.is_some());
        assert!(p.counterfactual.is_none());
    }

    #[test]
    fn test_failure_rate_zero() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::Causal);
        for _ in 0..4 {
            p.run_pipeline(&h);
        }
        assert_eq!(p.failure_rate(), 0.0);
    }

    #[test]
    fn test_stages_completed_contains_selector_and_mcts() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.8, ExpertType::MultiHop);
        let report = p.run_pipeline(&h);
        assert!(
            report.stages_completed.contains(&"selector".to_string()),
            "stages_completed: {:?}",
            report.stages_completed
        );
        assert!(
            report.stages_completed.contains(&"mcts".to_string()),
            "stages_completed: {:?}",
            report.stages_completed
        );
    }

    #[test]
    fn test_selector_disabled_uses_default_strategy() {
        let cfg = PipelineConfig {
            enable_selector: false,
            ..PipelineConfig::default()
        };
        let mut p = ReasoningPipeline::new(cfg).with_mcts(MctsReasoner::new(MctsConfig::default()));
        let h = make_hypothesis(1, 0.8, ExpertType::Synthesis);
        let report = p.run_pipeline(&h);
        assert!(report.selector_stats.is_none());
        assert!(report.mcts_stats.is_some());
    }

    #[test]
    fn test_counterfactual_disabled_by_default() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.9, ExpertType::Analogical);
        let report = p.run_pipeline(&h);
        assert!(report.counterfactual_stats.is_none());
    }

    #[test]
    fn test_report_history_capped_at_100() {
        let mut p = default_pipeline();
        let h = make_hypothesis(1, 0.7, ExpertType::Causal);
        for _ in 0..150 {
            p.run_pipeline(&h);
        }
        assert_eq!(p.report_history.len(), 100);
        assert_eq!(p.total_pipelines_run(), 150);
    }
}
