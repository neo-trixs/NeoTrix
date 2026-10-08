//! nt_optimize_stages — 从 `pipeline.rs` 拆分 (optimize_stages — 优化族: scaffold-RL/hypercube/蒸馏/元改进/harness/架构/趋势/元目标 (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::nt_types::*;
use std::collections::VecDeque;
use super::super::SelfIteratingBrain;
use crate::l0_substrate::nt_core_error::NeoTrixError;
use crate::l2_perception::nt_world::nt_world_model::TaskType;

/// Ornith-1-style Scaffold-Aware Reinforcement Learning stage.
///
/// Implements self-scaffolding: the model learns to generate both solution
/// rollouts and the task-specific scaffolds that guide those rollouts.
/// Two-stage process:
///   1. Scaffold stage — propose refined scaffold from past experience
///   2. Solution stage — generate solution rollout conditioned on scaffold
///
/// Jointly optimizes scaffold and solution using staleness-weighted GRPO,
/// where older off-policy tokens are down-weighted by age.
///
/// Inspired by Ornith-1.0 (DeepReinforce, arXiv 2026):
/// "Jointly optimizing the scaffold and the resulting solution, the model
/// discovers better search trajectories and generates higher-quality solutions."
#[derive(Debug, Clone)]
pub struct _ScaffoldAwareRLStage {
    pub scaffold_history: VecDeque<_ScaffoldRecord>,
    pub max_history: usize,
    pub staleness_threshold: u64,
    pub grpo_clip: f64,
}

#[derive(Debug, Clone)]
pub struct _ScaffoldRecord {
    pub iteration: u64,
    pub scaffold: String,
    pub solution_score: f64,
    pub reward: f64,
    pub staleness: u64,
}

impl _ScaffoldAwareRLStage {
    pub fn new() -> Self {
        Self {
            scaffold_history: VecDeque::new(),
            max_history: 50,
            staleness_threshold: 10,
            grpo_clip: 0.2,
        }
    }

    /// Stage 1: Propose a refined scaffold based on past high-reward scaffolds.
    fn propose_scaffold(&self, brain: &SelfIteratingBrain) -> String {
        if self.scaffold_history.is_empty() {
            return format!("iter_{}_baseline", brain.iteration);
        }
        let mut best = &self.scaffold_history[0];
        for record in &self.scaffold_history {
            if record.reward > best.reward {
                best = record;
            }
        }
        let staleness_weight = self.compute_staleness_weight(best.staleness);
        if staleness_weight < 0.3 {
            format!("iter_{}_fresh", brain.iteration)
        } else {
            format!("{}_refined", best.scaffold)
        }
    }

    /// Stage 2: Generate solution score from brain state and scaffold.
    fn compute_solution_score(&self, brain: &SelfIteratingBrain, _scaffold: &str) -> f64 {
        let champion_score = brain.champion.as_ref().map(|c| c.score).unwrap_or(0.0);
        let prm_reward = brain._prm_cumulative_reward.clamp(0.0, 1.0);
        let cap_mean = if brain.brain.capability.arr.is_empty() {
            0.0
        } else {
            brain.brain.capability.arr.iter().sum::<f64>() / brain.brain.capability.arr.len() as f64
        };
        let entropy_penalty = (1.0 - brain.entropy_crisis_level.clamp(0.0, 1.0)) * 0.1;

        0.4 * champion_score + 0.3 * prm_reward + 0.2 * cap_mean + entropy_penalty
    }

    /// Staleness weight: down-weights older off-policy tokens.
    /// w(d) = exp(-d / threshold) — tokens older than threshold get near-zero weight.
    fn compute_staleness_weight(&self, age: u64) -> f64 {
        if age == 0 {
            return 1.0;
        }
        (-(age as f64) / (self.staleness_threshold as f64)).exp()
    }

    /// GRPO loss with staleness weighting:
    /// L = -E[ w(d) * min(ratio * A, clip(ratio, 1-ε, 1+ε) * A) ]
    fn compute_grpo_loss(&self, reward: f64, old_reward: f64, staleness: u64) -> f64 {
        let ratio = if old_reward.abs() > 1e-10 {
            (reward / old_reward).clamp(0.1, 10.0)
        } else {
            1.0
        };
        let advantage = reward - 0.5;
        let clipped = ratio.clamp(1.0 - self.grpo_clip, 1.0 + self.grpo_clip);
        let w = self.compute_staleness_weight(staleness);
        -w * ratio.min(clipped) * advantage
    }
}

impl Default for _ScaffoldAwareRLStage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrainStage for _ScaffoldAwareRLStage {
    fn name(&self) -> &str {
        "scaffold_aware_rl"
    }
    fn frequency(&self) -> usize {
        3
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut stage = self.clone();
        let scaffold = stage.propose_scaffold(brain);

        let old_score = brain.champion.as_ref().map(|c| c.score).unwrap_or(0.0);
        let solution_score = stage.compute_solution_score(brain, &scaffold);

        let age = brain.iteration.saturating_sub(
            stage
                .scaffold_history
                .back()
                .map(|r| r.iteration)
                .unwrap_or(0),
        );
        let reward = solution_score * stage.compute_staleness_weight(age);

        // GRPO loss → E8 policy update (修复 no-op):
        // 原先 loss 被 `let _loss` 丢弃, GRPO 信号从未进入策略学习,
        // 与 DPO/GRPO 训练闭环断裂 (同 H3 no-op 模式)。现在把
        // -loss (负 loss = 正 advantage) 作为策略奖励喂给 E8Policy,
        // 与 _SSMUpdateStage 的更新风格一致, 形成真实 GRPO 训练回路。
        let loss = stage.compute_grpo_loss(reward, old_score, age);
        let grpo_reward = (-loss).clamp(0.0, 1.0);
        let policy_mode = brain._e8_policy.best_mode();
        brain._e8_policy.mode_values[policy_mode.0 as usize] =
            (brain._e8_policy.mode_values[policy_mode.0 as usize] * 0.9 + grpo_reward * 0.1)
                .min(1.0);
        brain._e8_policy.mode_counts[policy_mode.0 as usize] += 1;
        brain._e8_policy.decay_epsilon();
        log::trace!(
            "[scaffold_grpo] loss={:.4} grpo_reward={:.4} mode={} age={}",
            loss,
            grpo_reward,
            policy_mode.0,
            age
        );

        // Record scaffold
        stage.scaffold_history.push_back(_ScaffoldRecord {
            iteration: brain.iteration,
            scaffold,
            solution_score,
            reward,
            staleness: age,
        });
        if stage.scaffold_history.len() > stage.max_history {
            stage.scaffold_history.pop_front();
        }

        // ECHO terminal-prediction signal quality — available with full echo bridge
        #[cfg(feature = "echo_bridge")]
        {
            let report = brain.echo_bridge.echo.batch_signal_quality();
            let echo_loss = 1.0 - report.signal_coverage();
            if echo_loss > 0.001 {
                log::debug!(
                    "[e8-echo] loss={:.6} (coverage={:.3}, error_rate={:.3})",
                    echo_loss,
                    report.signal_coverage(),
                    report.error_rate()
                );
            }
        }

        // Update champion if improved
        let improved = solution_score > old_score + 0.01;
        if improved {
            let champ = BrainSnapshot::new(&brain.brain, &TaskType::CodeGeneration);
            return Ok(StageDecision::Promote(champ));
        }

        Ok(StageDecision::Continue)
    }
}


/// Hypercube optimize stage — prunes low-access entries via HyperCubeBridge.
pub struct _HyperCubeOptimizeStage;
impl Default for _HyperCubeOptimizeStage {
    fn default() -> Self {
        Self
    }
}
impl _HyperCubeOptimizeStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _HyperCubeOptimizeStage {
    fn name(&self) -> &str {
        "hypercube_optimize"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let mut pruned = 0usize;
        if let Some(ref mut router) = brain.attention_router {
            let before = router.bridge.hypercube.cell_count();
            pruned = router.bridge.hypercube.prune_low_access(2);
            if pruned > 0 {
                let after = router.bridge.hypercube.cell_count();
                log::info!(
                    "[hypercube_optimize] iter={}: pruned {} entries ({} → {})",
                    brain.iteration,
                    pruned,
                    before,
                    after
                );
            }
            let sparse_dims: Vec<String> = (0..16)
                .filter_map(|dim| {
                    let d = router.bridge.hypercube.coord_density(dim);
                    if d < 0.01 {
                        Some(format!("dim{}:{:.3}", dim, d))
                    } else {
                        None
                    }
                })
                .collect();
            if !sparse_dims.is_empty() {
                log::debug!(
                    "[hypercube_optimize] sparse dims: [{}]",
                    sparse_dims.join(",")
                );
            }
        }
        if pruned == 0 {
            log::trace!(
                "[hypercube_optimize] iter={}: no pruning needed",
                brain.iteration
            );
        }
        Ok(StageDecision::Continue)
    }
}


/// Distillation stage — extracts principles from pipeline trajectory into knowledge distiller
pub struct _DistillationStage;
impl Default for _DistillationStage {
    fn default() -> Self {
        Self
    }
}
impl _DistillationStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _DistillationStage {
    fn name(&self) -> &str {
        "distillation"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let traj_len = brain
            .reasoning_engine
            .as_ref()
            .map(|e| e.state_trajectory.len())
            .unwrap_or(0);
        if traj_len > 0 {
            let session = crate::l5_cognition::nt_mind::knowledge_distiller::SessionRecord {
                id: format!("pipeline-iter-{}", brain.iteration),
                user_messages: vec![brain._current_task.clone()],
                actions_taken: vec![format!("pipeline_iter_{}", brain.iteration)],
                outcomes: vec![format!("reward_{:.4}", brain._reward)],
                reward_signal: brain._reward,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                task_type: None,
                e8_mode: None,
                edit_types: vec![],
            };
            let principles = brain._knowledge_distiller.distill(&session);
            let absorbed = brain
                ._knowledge_distiller
                .absorb(&mut brain.brain.capability);
            if !principles.is_empty() || absorbed > 0 {
                log::info!(
                    "[distillation] {} principles from iter {}, {} absorbed into capability",
                    principles.len(),
                    brain.iteration,
                    absorbed
                );
            }
        }
        Ok(StageDecision::Continue)
    }
}


/// Meta improvement stage (self-evolution planning)
pub struct _MetaImprovementStage;
impl Default for _MetaImprovementStage {
    fn default() -> Self {
        Self
    }
}
impl _MetaImprovementStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _MetaImprovementStage {
    fn name(&self) -> &str {
        "meta_improvement"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let hist = &brain.brain.evaluation_history;
        let recent_count = hist.len().min(10);
        if recent_count >= 3 {
            let recent: Vec<f64> = hist
                .iter()
                .rev()
                .take(recent_count)
                .map(|r| r.score_after)
                .collect();
            let avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;
            let improving = recent.windows(2).filter(|w| w[1] > w[0]).count();
            let plateau = improving < recent_count / 3 && avg < 0.6;
            if plateau {
                brain.curiosity_bonus = (brain.curiosity_bonus + 0.05).min(0.3);
                log::info!(
                    "[meta_improvement] plateau detected, curiosity={:.4}",
                    brain.curiosity_bonus
                );
            } else {
                brain.curiosity_bonus = (brain.curiosity_bonus * 0.95).max(0.0);
            }
            if let Some(ref kb) = brain._nt_memory_kb {
                let _ = kb.kv_set("meta_improvement", &format!("iter_{}", brain.iteration),
                    &serde_json::json!({"avg_score": avg, "improving": improving, "plateau": plateau,
                        "curiosity": brain.curiosity_bonus}).to_string());
            }
        }
        log::trace!(
            "[meta_improvement] iter={} eval_history={}",
            brain.iteration,
            hist.len()
        );
        Ok(StageDecision::Continue)
    }
}


/// Harness adaptation stage — adjusts capability vector based on low reward
pub struct HarnessAdaptStage;
impl Default for HarnessAdaptStage {
    fn default() -> Self {
        Self
    }
}
impl HarnessAdaptStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for HarnessAdaptStage {
    fn name(&self) -> &str {
        "harness_adapt"
    }
    fn frequency(&self) -> usize {
        2
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let reward = brain._reward;
        let caps = brain.brain.capability.arr().to_vec();
        if reward < 0.3 && !caps.is_empty() {
            let weak_count = caps.iter().filter(|&&v| v < 0.2).count();
            let boost = (0.3 - reward) * 0.1;
            for v in brain.brain.capability.arr_mut().iter_mut() {
                if *v < 0.2 {
                    *v = (*v + boost).min(0.3);
                }
            }
            // H2 修复: 抬升后同步快照, 使保底抬升不产生虚假 delta。
            // 否则下一 tick compute_capability_deltas 会把 Harness 抬升当真实
            // 能力提升 → SFT 自产正样本 → "自抬能力→自产 delta→自训"奖励黑客。
            // 保底抬升是防能力归零的自我修正, 不是真实进化, 不应进入学习信号。
            let snap = brain._snapshot_capability();
            let mut new_snap = snap.clone();
            for v in new_snap.arr_mut().iter_mut() {
                if *v < 0.2 {
                    *v = (*v + boost).min(0.3);
                }
            }
            brain._set_snapshot(crate::l5_cognition::nt_mind::nt_mind::self_iterating::BrainSnapshot {
                capability: new_snap,
                learning_rate: brain._snapshot_lr(),
                score: brain._snapshot_score(),
            });
            log::info!(
                "[harness_adapt] low reward={:.4}, boosted {} weak caps by {:.4} (snapshot synced)",
                reward,
                weak_count,
                boost
            );
        }
        Ok(StageDecision::Continue)
    }
}


/// ArchitectureOptimizer stage: runs self-architecture analysis every 15 iterations.
/// Uses SelfArchitectureOptimizer to identify structural improvements.
pub struct _ArchitectureOptimizerStage;
impl Default for _ArchitectureOptimizerStage {
    fn default() -> Self {
        Self
    }
}
impl _ArchitectureOptimizerStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _ArchitectureOptimizerStage {
    fn name(&self) -> &str {
        "arch_optimizer"
    }
    fn frequency(&self) -> usize {
        15
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::l5_cognition::l1_facade::arch_optimizer::SelfArchitectureOptimizer;
        let optimizer = SelfArchitectureOptimizer::new();
        // Pass module sizes as proxy file list
        let caps = brain.brain.capability.arr();
        let files: Vec<(String, usize)> = caps
            .iter()
            .enumerate()
            .map(|(i, v)| (format!("cap_{}", i), (*v * 1000.0) as usize))
            .collect();
        let report = optimizer.analyze(&files, None);
        if report.total_suggestions > 0 {
            log::info!(
                "[arch_optimizer] suggestions={} auto_fixable={}",
                report.total_suggestions,
                report.auto_fixable_count
            );
        }
        if let Some(ref kb) = brain._nt_memory_kb {
            let _ = kb.kv_set(
                "arch_optimizer",
                &format!("iter_{}", brain.iteration),
                &serde_json::json!({
                    "suggestions": report.total_suggestions,
                    "auto_fixable": report.auto_fixable_count,
                    "large_modules": report.large_modules.len(),
                })
                .to_string(),
            );
        }
        Ok(StageDecision::Continue)
    }
}


/// _TrendAnalysisStage: runs evolution trend analysis every 15 iterations.
/// Uses EvolutionTrendAnalyzer to detect capability trends over time.
pub struct _TrendAnalysisStage;
impl Default for _TrendAnalysisStage {
    fn default() -> Self {
        Self
    }
}
impl _TrendAnalysisStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _TrendAnalysisStage {
    fn name(&self) -> &str {
        "trend_analysis"
    }
    fn frequency(&self) -> usize {
        15
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::l5_cognition::nt_mind::trend_analyzer::EvolutionTrendAnalyzer;
        let mut analyzer = EvolutionTrendAnalyzer::new();
        let caps = brain.brain.capability.arr();
        for (i, val) in caps.iter().enumerate() {
            analyzer.record(&format!("cap_{}", i), *val, Some("capability"));
        }
        let report = analyzer.analyze();
        log::trace!(
            "[trend_analysis] trends={} dir={:?} improving={} declining={}",
            report.trends.len(),
            report.overall_direction,
            report.improving_count,
            report.declining_count
        );
        if let Some(ref kb) = brain._nt_memory_kb {
            let _ = kb.kv_set(
                "trend_analysis",
                &format!("iter_{}", brain.iteration),
                &serde_json::json!({"trends": report.trends.len(),
                    "direction": format!("{:?}", report.overall_direction),
                    "improving": report.improving_count,
                    "declining": report.declining_count,
                })
                .to_string(),
            );
        }
        Ok(StageDecision::Continue)
    }
}


/// _MetaGoalStage: generates meta-goals every 12 iterations from trend report.
pub struct _MetaGoalStage;
impl Default for _MetaGoalStage {
    fn default() -> Self {
        Self
    }
}
impl _MetaGoalStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _MetaGoalStage {
    fn name(&self) -> &str {
        "meta_goal"
    }
    fn frequency(&self) -> usize {
        12
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::l5_cognition::nt_mind::trend_analyzer::EvolutionTrendAnalyzer;
        use crate::l5_cognition::nt_mind::meta_goal_generator::MetaGoalGenerator;
        let mut analyzer = EvolutionTrendAnalyzer::new();
        let caps = brain.brain.capability.arr();
        for (i, val) in caps.iter().enumerate() {
            analyzer.record(&format!("cap_{}", i), *val, Some("capability"));
        }
        let report = analyzer.analyze();
        let generator = MetaGoalGenerator::new();
        let goals = generator.generate_from_trends(&report);
        log::debug!("[meta_goal] generated {} goals from trends", goals.len());
        if let Some(ref kb) = brain._nt_memory_kb {
            let goal_str: Vec<String> = goals
                .iter()
                .map(|g| format!("{}:{:.2}", g.description, g.priority as u8))
                .collect();
            let _ = kb.kv_set(
                "meta_goals",
                &format!("iter_{}", brain.iteration),
                &serde_json::json!({"count": goals.len(), "goals": goal_str}).to_string(),
            );
        }
        Ok(StageDecision::Continue)
    }
}

