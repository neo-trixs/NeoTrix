use super::SelfIteratingBrain;
use super::super::self_edit::MicroEdit;
use super::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_core_error::NeoTrixError;
use std::collections::VecDeque;

/// Concrete edit budget: bounds the L1 edit count per step.
#[derive(Debug, Clone)]
pub struct EditBudget {
    pub max_edits: usize,
    pub current: usize,
}

impl EditBudget {
    pub fn new(max_edits: usize) -> Self {
        Self { max_edits, current: 0 }
    }
    pub fn reset(&mut self) { self.current = 0; }
    pub fn is_exhausted(&self) -> bool { self.current >= self.max_edits }
    pub fn consume(&mut self) -> bool {
        if self.is_exhausted() { return false; }
        self.current += 1;
        true
    }
}

impl Default for EditBudget {
    fn default() -> Self { Self::new(8) }
}

/// Textual learning-rate scheduler — controls how many edits are allowed per step.
///
/// Inspired by SkillOpt's bounded text-space optimization: the edit budget $L_t$
/// limits how far the skill artifact can change in one step, preventing uncontrolled
/// rewrites that erase useful rules or overfit to local failures.
#[derive(Debug, Clone)]
pub enum LrScheduler {
    Constant(usize),
    Cosine { max_lr: usize, min_lr: usize, total_steps: usize, current: usize },
    Linear { start: usize, end: usize, total_steps: usize, current: usize },
}

impl LrScheduler {
    pub fn current_budget(&self) -> usize {
        match self {
            LrScheduler::Constant(lr) => *lr,
            LrScheduler::Cosine { max_lr, min_lr, total_steps, current } => {
                let progress = (*current as f64 / *total_steps as f64).min(1.0);
                let cosine = (progress * std::f64::consts::PI * 0.5).cos();
                let lr = *min_lr as f64 + (*max_lr - *min_lr) as f64 * cosine;
                (lr.round() as usize).max(*min_lr)
            }
            LrScheduler::Linear { start, end, total_steps, current } => {
                let progress = (*current as f64 / *total_steps as f64).min(1.0);
                let lr = *start as f64 + (*end as f64 - *start as f64) * progress;
                (lr.round() as usize).max(1)
            }
        }
    }

    pub fn step(&mut self) {
        match self {
            LrScheduler::Constant(_) => {}
            LrScheduler::Cosine { current, .. } => *current += 1,
            LrScheduler::Linear { current, .. } => *current += 1,
        }
    }

    pub fn reset(&mut self) {
        match self {
            LrScheduler::Constant(_) => {}
            LrScheduler::Cosine { current, .. } => *current = 0,
            LrScheduler::Linear { current, .. } => *current = 0,
        }
    }
}

impl Default for LrScheduler {
    fn default() -> Self {
        LrScheduler::Cosine { max_lr: 8, min_lr: 2, total_steps: 100, current: 0 }
    }
}

/// Records a rejected edit and the score drop it caused.
///
/// Analogous to SkillOpt's rejected-edit buffer: rejected updates are still useful
/// as negative feedback, preventing the optimizer from repeating failed edits.
#[derive(Debug, Clone)]
pub struct RejectedEdit {
    pub edits: Vec<MicroEdit>,
    pub score_drop: f64,
    pub reason: String,
    pub iteration: u64,
}

/// Rolling buffer of rejected edits, used as negative feedback.
#[derive(Debug, Clone)]
pub struct RejectedEditBuffer {
    pub max_size: usize,
    pub entries: VecDeque<RejectedEdit>,
}

impl RejectedEditBuffer {
    pub fn new(max_size: usize) -> Self {
        Self { max_size, entries: VecDeque::with_capacity(max_size) }
    }

    pub fn push(&mut self, edit: RejectedEdit) {
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        self.entries.push_back(edit);
    }

    pub fn all(&self) -> impl Iterator<Item = &RejectedEdit> {
        self.entries.iter()
    }

    pub fn recent_failure_patterns(&self, n: usize) -> Vec<String> {
        self.entries.iter().rev().take(n).map(|r| {
            let edit_desc: Vec<String> = r.edits.iter().map(|e| format!("{:?}", e)).collect();
            format!("[iter {}] {} (drop: {:.3})", r.iteration, edit_desc.join(", "), r.score_drop)
        }).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for RejectedEditBuffer {
    fn default() -> Self {
        Self::new(32)
    }
}

/// Validation gate that accepts edits only on strict improvement.
///
/// Maps to SkillOpt's held-out validation: each candidate skill is evaluated
/// on a selection split and accepted only when it strictly improves the score.
#[derive(Debug, Clone)]
pub struct ValidationGate {
    pub min_improvement: f64,
    pub selection_history: Vec<(u64, f64)>,
    pub best_score: f64,
}

impl ValidationGate {
    pub fn new(min_improvement: f64) -> Self {
        Self { min_improvement, selection_history: Vec::new(), best_score: 0.0 }
    }

    pub fn accept(&mut self, iteration: u64, candidate_score: f64) -> bool {
        let improved = candidate_score > self.best_score + self.min_improvement;
        self.selection_history.push((iteration, candidate_score));
        if improved {
            self.best_score = candidate_score;
        }
        improved
    }

    pub fn score_trend(&self, window: usize) -> f64 {
        let n = self.selection_history.len();
        if n < 2 {
            return 0.0;
        }
        let recent = &self.selection_history[n.saturating_sub(window)..];
        if recent.len() < 2 {
            return 0.0;
        }
        let first = recent.first().map(|(_, s)| *s).unwrap_or(0.0);
        let last = recent.last().map(|(_, s)| *s).unwrap_or(0.0);
        last - first
    }
}

impl Default for ValidationGate {
    fn default() -> Self {
        Self::new(0.001)
    }
}

// ==================== Pipeline Stages ====================

pub struct BoundedEditStage;
impl Default for BoundedEditStage { fn default() -> Self { Self } }
impl BoundedEditStage { pub fn new() -> Self { Self } }
impl BrainStage for BoundedEditStage {
    fn name(&self) -> &str { "bounded_edit" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let budget = brain._lr_scheduler.current_budget();
        let edits = brain._take_micro_edits();
        if edits.len() > budget {
            log::info!("[bounded-edit] trimming {} edits to budget {}", edits.len(), budget);
            let bounded: Vec<MicroEdit> = edits.into_iter().take(budget).collect();
            brain._set_micro_edits(bounded);
        } else {
            brain._set_micro_edits(edits);
        }
        brain._lr_scheduler.step();
        Ok(StageDecision::Continue)
    }
}

pub struct ValidationGateStage;
impl Default for ValidationGateStage { fn default() -> Self { Self } }
impl ValidationGateStage { pub fn new() -> Self { Self } }
impl BrainStage for ValidationGateStage {
    fn name(&self) -> &str { "validation_gate" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let candidate_score = brain._reward();
        let iteration = brain.iteration;
        let accepted = brain._validation_gate.accept(iteration, candidate_score);

        if !accepted {
            let edits = brain._take_micro_edits();
            let snapshot_score = brain._snapshot_score();
            let score_drop = snapshot_score - candidate_score;
            let rejected = RejectedEdit {
                edits,
                score_drop: score_drop.max(0.0),
                reason: format!("validation gate: score {:.4} <= best {:.4}", candidate_score, brain._validation_gate.best_score),
                iteration,
            };
            brain._rejected_buffer.push(rejected);
            log::info!("[validation-gate] rejected candidate at iter {}: score={:.4} <= best={:.4}",
                iteration, candidate_score, brain._validation_gate.best_score);
            brain._snapshot_restore();
        } else {
            log::info!("[validation-gate] accepted candidate at iter {}: score={:.4} (best={:.4})",
                iteration, candidate_score, brain._validation_gate.best_score);
        }

        Ok(StageDecision::Continue)
    }
}

pub struct RejectedBufferFeedbackStage;
impl Default for RejectedBufferFeedbackStage { fn default() -> Self { Self } }
impl RejectedBufferFeedbackStage { pub fn new() -> Self { Self } }
impl BrainStage for RejectedBufferFeedbackStage {
    fn name(&self) -> &str { "rejected_feedback" }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let patterns = brain._rejected_buffer.recent_failure_patterns(5);
        if !patterns.is_empty() {
            let insights = patterns.join(" | ");
            let existing = brain._open_source_insights.clone().unwrap_or_default();
            let combined = if existing.is_empty() {
                format!("Rejected patterns: {}", insights)
            } else {
                format!("{} | Rejected patterns: {}", existing, insights)
            };
            brain._set_open_source_insights(Some(combined));
        }
        Ok(StageDecision::Continue)
    }
}

pub struct EpochSlowUpdateStage;
impl Default for EpochSlowUpdateStage { fn default() -> Self { Self } }
impl EpochSlowUpdateStage { pub fn new() -> Self { Self } }
impl BrainStage for EpochSlowUpdateStage {
    fn name(&self) -> &str { "epoch_slow_update" }
    fn frequency(&self) -> usize { 10 }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let trend = brain._validation_gate.score_trend(10);
        log::info!("[epoch-slow] score trend (last 10): {:.4}", trend);
        if trend < -0.05 {
            log::warn!("[epoch-slow] negative trend detected ({:.4}), consider revision", trend);
        }
        Ok(StageDecision::Continue)
    }
}
