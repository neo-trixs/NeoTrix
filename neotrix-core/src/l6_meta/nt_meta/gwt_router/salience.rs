//! SalienceCalculator — computes composite salience score for GWT attention routing.
//!
//! Combines urgency, importance, novelty, and cost_weight into a single [0,1] score.
//! Implements R-P128: cost-aware routing — cheap models preferred when quality is sufficient.

use super::config::{GwtConfig, SalienceWeights};

/// A task to be routed through the Global Workspace.
#[derive(Debug, Clone)]
pub struct Task {
    /// Task type identifier (e.g. "code_review", "summarization", "inference").
    pub task_type: String,
    /// Urgency score in [0,1]. 1 = must execute immediately.
    pub urgency: f64,
    /// Importance score in [0,1]. 1 = critical to system health.
    pub importance: f64,
    /// Novelty score in [0,1]. 1 = never seen this task type before.
    pub novelty: f64,
    /// Estimated cost of the cheapest capable model (tokens * price_per_token).
    pub estimated_cost: f64,
    /// Estimated cost of the strongest capable model.
    pub max_cost: f64,
}

/// Contextual information that modulates salience computation.
#[derive(Debug, Clone)]
pub struct Context {
    /// Current system load [0,1]. High load => prefer cheap models.
    pub system_load: f64,
    /// Remaining budget in the current cycle.
    pub remaining_budget: f64,
    /// Total budget in the current cycle.
    pub total_budget: f64,
}

/// Composite salience score output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SalienceScore {
    /// Final weighted salience [0,1].
    pub score: f64,
    /// Individual components before weighting.
    pub components: SalienceComponents,
}

/// Raw components of a salience score.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SalienceComponents {
    pub urgency: f64,
    pub importance: f64,
    pub novelty: f64,
    pub cost_weight: f64,
}

/// Computes salience scores for tasks based on GWT configuration.
pub struct SalienceCalculator {
    weights: SalienceWeights,
    cost_weight_factor: f64,
}

impl SalienceCalculator {
    pub fn new(config: &GwtConfig) -> Self {
        Self {
            weights: config.salience_weights,
            cost_weight_factor: config.cost_weight_factor,
        }
    }

    /// Compute salience score for a task in the given context.
    ///
    /// Formula:
    ///   cost_component = if max_cost > 0 { 1.0 - (estimated_cost / max_cost) } else { 0.5 }
    ///   cost_component = cost_component * cost_weight_factor
    ///   score = w_u * urgency + w_i * importance + w_n * novelty + w_c * cost_component
    ///   score = score.clamp(0.0, 1.0)
    pub fn compute(&self, task: &Task, context: &Context) -> SalienceScore {
        let cost_component = self.compute_cost_component(task);
        let load_penalty = 0.1 * context.system_load;

        let weighted = self.weights.urgency * task.urgency
            + self.weights.importance * task.importance
            + self.weights.novelty * task.novelty
            + self.weights.cost_weight * cost_component;

        let score = (weighted - load_penalty).clamp(0.0, 1.0);

        SalienceScore {
            score,
            components: SalienceComponents {
                urgency: task.urgency,
                importance: task.importance,
                novelty: task.novelty,
                cost_weight: cost_component,
            },
        }
    }

    /// Compute batch salience for multiple tasks, sorted descending by score.
    pub fn compute_batch(&self, tasks: &[Task], context: &Context) -> Vec<(usize, SalienceScore)> {
        let mut scored: Vec<(usize, SalienceScore)> = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| (i, self.compute(t, context)))
            .collect();
        scored.sort_by(|a, b| b.1.score.partial_cmp(&a.1.score).unwrap_or(std::cmp::Ordering::Equal));
        scored
    }

    fn compute_cost_component(&self, task: &Task) -> f64 {
        let raw = if task.max_cost > 0.0 {
            1.0 - (task.estimated_cost / task.max_cost)
        } else {
            0.5
        };
        (raw * self.cost_weight_factor).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_context() -> Context {
        Context {
            system_load: 0.0,
            remaining_budget: 1000.0,
            total_budget: 1000.0,
        }
    }

    fn cheap_task() -> Task {
        Task {
            task_type: "summarization".into(),
            urgency: 0.8,
            importance: 0.5,
            novelty: 0.1,
            estimated_cost: 0.1,
            max_cost: 1.0,
        }
    }

    #[test]
    fn compute_returns_valid_score() {
        let calc = SalienceCalculator::new(&GwtConfig::default());
        let score = calc.compute(&cheap_task(), &default_context());
        assert!(score.score >= 0.0 && score.score <= 1.0);
    }

    #[test]
    fn high_cost_reduces_salience() {
        let calc = SalienceCalculator::new(&GwtConfig::default());
        let cheap = Task {
            estimated_cost: 0.1,
            ..cheap_task()
        };
        let expensive = Task {
            estimated_cost: 0.9,
            ..cheap_task()
        };
        let ctx = default_context();
        let s_cheap = calc.compute(&cheap, &ctx);
        let s_expensive = calc.compute(&expensive, &ctx);
        assert!(s_cheap.score >= s_expensive.score);
    }

    #[test]
    fn high_urgency_boosts_salience() {
        let calc = SalienceCalculator::new(&GwtConfig::default());
        let low = Task {
            urgency: 0.1,
            ..cheap_task()
        };
        let high = Task {
            urgency: 0.9,
            ..cheap_task()
        };
        let ctx = default_context();
        assert!(calc.compute(&high, &ctx).score > calc.compute(&low, &ctx).score);
    }

    #[test]
    fn system_load_penalizes() {
        let calc = SalienceCalculator::new(&GwtConfig::default());
        let task = cheap_task();
        let low_load = Context {
            system_load: 0.0,
            ..default_context()
        };
        let high_load = Context {
            system_load: 0.9,
            ..default_context()
        };
        assert!(calc.compute(&task, &low_load).score > calc.compute(&task, &high_load).score);
    }

    #[test]
    fn batch_sorted_descending() {
        let calc = SalienceCalculator::new(&GwtConfig::default());
        let tasks = vec![
            Task {
                urgency: 0.1,
                ..cheap_task()
            },
            Task {
                urgency: 0.9,
                ..cheap_task()
            },
        ];
        let ctx = default_context();
        let batch = calc.compute_batch(&tasks, &ctx);
        assert!(batch[0].1.score >= batch[1].1.score);
    }
}
