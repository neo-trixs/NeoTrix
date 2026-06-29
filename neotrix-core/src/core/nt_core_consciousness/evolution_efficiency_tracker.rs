use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

const DEFAULT_MAX_TASKS: usize = 500;
const DEFAULT_CONVERGENCE_WINDOW: usize = 5;

/// Tracks convergence of execution cost across repeated task patterns.
///
/// Inspired by SEA-Eval (arXiv:2604.08988): distinguishes genuine evolution
/// (cost decreases monotonically across repetitions) from pseudo-evolution
/// (cost stays flat or rises). The T metric tracks token/step convergence.
///
/// When `evolution_convergence_rate` is negative, the system is genuinely
/// becoming more efficient at recurring task patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEfficiencyTracker {
    task_records: VecDeque<TaskRecord>,
    max_records: usize,
    /// Currently computed convergence rate. Negative = improving.
    evolution_convergence_rate: f64,
    /// Number of task families with 3+ repetitions (statistically meaningful)
    converged_families: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TaskRecord {
    task_hash: u64,
    task_domain: String,
    execution_cost: f64,
    cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyReport {
    pub evolution_convergence_rate: f64,
    pub converged_families: usize,
    pub total_unique_families: usize,
    pub total_records: usize,
    pub is_genuine_evolution: bool,
    pub efficiency_trend: String,
}

impl Default for EvolutionEfficiencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EvolutionEfficiencyTracker {
    pub fn new() -> Self {
        Self {
            task_records: VecDeque::with_capacity(DEFAULT_MAX_TASKS),
            max_records: DEFAULT_MAX_TASKS,
            evolution_convergence_rate: 0.0,
            converged_families: 0,
        }
    }

    pub fn with_max_tasks(max: usize) -> Self {
        Self {
            task_records: VecDeque::with_capacity(max),
            max_records: max,
            evolution_convergence_rate: 0.0,
            converged_families: 0,
        }
    }

    /// Record a task execution for efficiency tracking.
    ///
    /// - `task_hash`: deterministic hash identifying the task family
    /// - `task_domain`: domain label (e.g., "reasoning", "search", "memory")
    /// - `execution_cost`: cost measure (steps, tokens, or cycles used)
    /// - `cycle`: consciousness cycle number
    pub fn record_task(
        &mut self,
        task_hash: u64,
        task_domain: &str,
        execution_cost: f64,
        cycle: u64,
    ) {
        if self.task_records.len() >= self.max_records {
            self.task_records.pop_front();
        }
        self.task_records.push_back(TaskRecord {
            task_hash,
            task_domain: task_domain.to_string(),
            execution_cost,
            cycle,
        });
        self.recompute();
    }

    /// Recompute the evolution convergence rate across all task families.
    fn recompute(&mut self) {
        if self.task_records.len() < 6 {
            self.evolution_convergence_rate = 0.0;
            self.converged_families = 0;
            return;
        }

        let mut families: HashMap<u64, Vec<f64>> = HashMap::new();
        for record in &self.task_records {
            families
                .entry(record.task_hash)
                .or_default()
                .push(record.execution_cost);
        }

        let mut convergence_slopes = Vec::new();
        let mut converged = 0usize;

        for (_hash, costs) in &families {
            if costs.len() < 3 {
                continue;
            }
            let recent_window = DEFAULT_CONVERGENCE_WINDOW.min(costs.len());
            let older: f64 = costs[..costs.len() - recent_window].iter().sum::<f64>()
                / (costs.len() - recent_window) as f64;
            let recent: f64 =
                costs[costs.len() - recent_window..].iter().sum::<f64>() / recent_window as f64;

            if older > 0.0 {
                let relative_change = (recent - older) / older;
                convergence_slopes.push(relative_change);
                if relative_change < 0.0 {
                    // Genuine evolution: this family is becoming more efficient
                    converged += 1;
                }
            }
        }

        self.converged_families = converged;

        if convergence_slopes.is_empty() {
            self.evolution_convergence_rate = 0.0;
        } else {
            self.evolution_convergence_rate =
                convergence_slopes.iter().sum::<f64>() / convergence_slopes.len() as f64;
        }
    }

    /// Current evolution convergence rate.
    /// Negative = genuine evolution (cost decreasing across repetitions).
    pub fn evolution_convergence_rate(&self) -> f64 {
        self.evolution_convergence_rate
    }

    /// Number of task families showing genuine convergence (cost decreasing).
    pub fn converged_families(&self) -> usize {
        self.converged_families
    }

    /// Generate a structured efficiency report.
    pub fn report(&self) -> EfficiencyReport {
        let total_records = self.task_records.len();
        let total_unique = self.count_unique_families();
        EfficiencyReport {
            evolution_convergence_rate: self.evolution_convergence_rate,
            converged_families: self.converged_families,
            total_unique_families: total_unique,
            total_records,
            is_genuine_evolution: self.evolution_convergence_rate < -0.01
                && self.converged_families > 0,
            efficiency_trend: if self.evolution_convergence_rate < -0.05 {
                "evolving".to_string()
            } else if self.evolution_convergence_rate < -0.01 {
                "slowly_evolving".to_string()
            } else if self.evolution_convergence_rate.abs() < 0.01 {
                "stable_no_evolution".to_string()
            } else if self.evolution_convergence_rate < 0.1 {
                "slight_degradation".to_string()
            } else {
                "degrading".to_string()
            },
        }
    }

    /// Count unique task families in the record.
    fn count_unique_families(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for record in &self.task_records {
            seen.insert(record.task_hash);
        }
        seen.len()
    }

    pub fn reset(&mut self) {
        self.task_records.clear();
        self.evolution_convergence_rate = 0.0;
        self.converged_families = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_from_str(s: &str) -> u64 {
        let mut h: u64 = 5381;
        for b in s.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        h
    }

    #[test]
    fn test_new_tracker_is_neutral() {
        let t = EvolutionEfficiencyTracker::new();
        assert!((t.evolution_convergence_rate() - 0.0).abs() < 1e-10);
        assert_eq!(t.converged_families(), 0);
    }

    #[test]
    fn test_insufficient_data_returns_neutral() {
        let mut t = EvolutionEfficiencyTracker::new();
        let h = hash_from_str("test_task");
        for i in 0..5 {
            t.record_task(h, "test", 100.0 - (i as f64 * 5.0), i as u64);
        }
        // 5 records < 6 threshold
        assert!((t.evolution_convergence_rate() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_genuine_evolution_detected() {
        let mut t = EvolutionEfficiencyTracker::new();
        let h = hash_from_str("repeated_task");
        // Cost decreases over repetitions: 100, 90, 80, 70, 60, 50, 40
        for i in 0..7 {
            let cost = 100.0 - (i as f64 * 10.0);
            t.record_task(h, "test", cost, i as u64);
        }
        let report = t.report();
        assert!(
            report.evolution_convergence_rate < -0.05,
            "convergence rate should be negative: got {}",
            report.evolution_convergence_rate
        );
        assert!(report.converged_families >= 1);
        assert!(report.is_genuine_evolution);
        assert_eq!(report.efficiency_trend, "evolving");
    }

    #[test]
    fn test_pseudo_evolution_no_convergence() {
        let mut t = EvolutionEfficiencyTracker::new();
        let h = hash_from_str("static_task");
        // Cost stays flat: 100, 100, 100, 100, 100, 100
        for i in 0..7 {
            t.record_task(h, "test", 100.0, i as u64);
        }
        let report = t.report();
        assert!(
            report.evolution_convergence_rate.abs() < 0.01,
            "flat costs should give near-zero convergence: got {}",
            report.evolution_convergence_rate
        );
        assert!(!report.is_genuine_evolution);
        assert_eq!(report.efficiency_trend, "stable_no_evolution");
    }

    #[test]
    fn test_degradation_detected() {
        let mut t = EvolutionEfficiencyTracker::new();
        let h = hash_from_str("degrading_task");
        // Cost increases: 40, 50, 60, 70, 80, 90, 100
        for i in 0..7 {
            let cost = 40.0 + (i as f64 * 10.0);
            t.record_task(h, "test", cost, i as u64);
        }
        let report = t.report();
        assert!(
            report.evolution_convergence_rate > 0.05,
            "increasing costs should give positive convergence: got {}",
            report.evolution_convergence_rate
        );
        assert!(!report.is_genuine_evolution);
        assert_eq!(report.efficiency_trend, "degrading");
    }

    #[test]
    fn test_multiple_families_tracked_independently() {
        let mut t = EvolutionEfficiencyTracker::new();
        let h1 = hash_from_str("evolving_task");
        let h2 = hash_from_str("degrading_task");
        // Evolving family
        for i in 0..7 {
            t.record_task(h1, "evolving", 100.0 - (i as f64 * 10.0), i as u64);
        }
        // Degrading family
        for i in 0..7 {
            t.record_task(h2, "degrading", 40.0 + (i as f64 * 10.0), i as u64 + 10);
        }
        let report = t.report();
        // One converged (the evolving one)
        assert_eq!(report.total_unique_families, 2);
        assert_eq!(report.converged_families, 1);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut t = EvolutionEfficiencyTracker::new();
        let h = hash_from_str("temp_task");
        for i in 0..7 {
            t.record_task(h, "test", 100.0 - (i as f64 * 10.0), i as u64);
        }
        assert!(t.evolution_convergence_rate() < 0.0);
        t.reset();
        assert!((t.evolution_convergence_rate() - 0.0).abs() < 1e-10);
        assert_eq!(t.converged_families(), 0);
    }

    #[test]
    fn test_different_families_different_convergence() {
        let mut t = EvolutionEfficiencyTracker::new();
        // Family with strong convergence
        let h_good = hash_from_str("good");
        for i in 0..10 {
            t.record_task(h_good, "good", 100.0 - (i as f64 * 8.0), i as u64);
        }
        // Family with no convergence
        let h_bad = hash_from_str("bad");
        for i in 0..10 {
            t.record_task(h_bad, "bad", 50.0, i as u64 + 10);
        }
        let report = t.report();
        // At least one family should be converged
        assert!(report.converged_families >= 1);
        // Overall rate should be negative (partially offset by flat family)
        assert!(
            report.evolution_convergence_rate < 0.0,
            "avg rate should be negative with one improving family"
        );
    }
}
