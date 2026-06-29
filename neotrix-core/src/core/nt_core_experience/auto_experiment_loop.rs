#![forbid(unsafe_code)]

use std::collections::VecDeque;

/// Configuration for the autonomous experiment loop.
#[derive(Debug, Clone)]
pub struct AutoExperimentConfig {
    /// Maximum number of experiments to run per tick.
    pub max_experiments_per_tick: usize,
    /// Total budget in seconds for experiment execution.
    pub budget_seconds: f64,
    /// Minimum delta to keep the change (default 0.05).
    pub keep_threshold: f64,
    /// Max consecutive failures before entering cooldown.
    pub max_consecutive_failures: u32,
    /// How many recent deltas to track in results.
    pub metrics_history: usize,
}

impl Default for AutoExperimentConfig {
    fn default() -> Self {
        Self {
            max_experiments_per_tick: 3,
            budget_seconds: 30.0,
            keep_threshold: 0.05,
            max_consecutive_failures: 5,
            metrics_history: 20,
        }
    }
}

/// A single experiment record.
#[derive(Debug, Clone)]
pub struct ExperimentRecord {
    /// Unique identifier.
    pub id: u64,
    /// Description of what was tried.
    pub hypothesis: String,
    /// Composite metric before the experiment.
    pub metric_before: f64,
    /// Composite metric after the simulated experiment.
    pub metric_after: f64,
    /// Change in composite metric (after - before).
    pub delta: f64,
    /// Whether the change was kept (delta > keep_threshold).
    pub kept: bool,
    /// Cycle at which the experiment was run.
    pub timestamp_cycle: u64,
}

/// Summary result from one tick of experiments.
#[derive(Debug, Clone)]
pub struct AutoExperimentResult {
    /// Number of experiments run this tick.
    pub experiments_run: usize,
    /// Number of experiments kept this tick.
    pub experiments_kept: usize,
    /// Best delta observed this tick.
    pub best_delta: f64,
    /// Current consecutive failure count.
    pub consecutive_failures: u32,
    /// Recent deltas (up to metrics_history entries).
    pub recent_deltas: Vec<f64>,
}

/// Karpathy-style autonomous experiment loop.
///
/// On each tick, generates a hypothesis based on the dominant metric weakness,
/// simulates a bounded perturbation, records the outcome, and tracks
/// consecutive failures with an automatic cooldown.
#[derive(Debug, Clone)]
pub struct AutoExperimentLoop {
    pub config: AutoExperimentConfig,
    /// Rolling history of experiment records (capped at 100).
    experiments: VecDeque<ExperimentRecord>,
    /// How many consecutive experiments have failed the keep threshold.
    consecutive_failures: u32,
    /// Cycle of the last tick execution.
    last_run_cycle: u64,
    /// Currently active hypothesis.
    active_hypothesis: Option<String>,
}

impl AutoExperimentLoop {
    /// Tick wrapper around run_experiment.
    pub fn tick(&mut self, cycle: u64, meta_accuracy: f64, ece: f64, loss: f64) -> AutoExperimentResult {
        self.run_experiment(ece, meta_accuracy, loss, cycle)
    }

    pub fn new(config: AutoExperimentConfig) -> Self {
        Self {
            config,
            experiments: VecDeque::new(),
            consecutive_failures: 0,
            last_run_cycle: 0,
            active_hypothesis: None,
        }
    }

    /// Run a batch of experiments for this tick.
    ///
    /// Generates hypotheses from metric weakness, simulates bounded
    /// perturbations, records keep/revert decisions. Returns zero experiments
    /// when consecutive failures exceed `max_consecutive_failures` (cooldown).
    pub fn run_experiment(
        &mut self,
        current_ece: f64,
        current_meta_acc: f64,
        current_loss: f64,
        cycle: u64,
    ) -> AutoExperimentResult {
        self.last_run_cycle = cycle;

        // ── cooldown gate ─────────────────────────────────────────────
        if self.consecutive_failures > self.config.max_consecutive_failures {
            log::warn!(
                "auto_experiment.cooldown: {} consecutive failures, skipping",
                self.consecutive_failures,
            );
            let recent: Vec<f64> = self
                .experiments
                .iter()
                .rev()
                .take(self.config.metrics_history)
                .map(|e| e.delta)
                .collect();
            return AutoExperimentResult {
                experiments_run: 0,
                experiments_kept: 0,
                best_delta: 0.0,
                consecutive_failures: self.consecutive_failures,
                recent_deltas: recent,
            };
        }

        let hypothesis = self.generate_hypothesis(current_ece, current_meta_acc, current_loss);
        self.active_hypothesis = Some(hypothesis.clone());
        let baseline = Self::composite_metric(current_ece, current_meta_acc, current_loss);

        let max = self.config.max_experiments_per_tick;
        let mut run = 0usize;
        let mut kept = 0usize;
        let mut best_delta = f64::NEG_INFINITY;
        let mut deltas = Vec::with_capacity(max);

        for i in 0..max {
            let metric_before = baseline;
            let metric_after = Self::simulate_outcome(metric_before, i);
            let delta = metric_after - metric_before;
            let is_kept = delta > self.config.keep_threshold;

            if is_kept {
                kept += 1;
            }
            if delta > best_delta {
                best_delta = delta;
            }

            let id = self.next_id();
            self.experiments.push_back(ExperimentRecord {
                id,
                hypothesis: hypothesis.clone(),
                metric_before,
                metric_after,
                delta,
                kept: is_kept,
                timestamp_cycle: cycle,
            });

            if self.experiments.len() > 100 {
                self.experiments.pop_front();
            }

            deltas.push(delta);
            run += 1;

            log::info!(
                "auto_experiment: #{} hyp={} before={:.4} after={:.4} delta={:+.4} kept={}",
                id,
                hypothesis,
                metric_before,
                metric_after,
                delta,
                is_kept,
            );
        }

        if kept == 0 {
            self.consecutive_failures += 1;
        } else {
            self.consecutive_failures = 0;
        }

        let recent: Vec<f64> = self
            .experiments
            .iter()
            .rev()
            .take(self.config.metrics_history)
            .map(|e| e.delta)
            .collect();

        AutoExperimentResult {
            experiments_run: run,
            experiments_kept: kept,
            best_delta,
            consecutive_failures: self.consecutive_failures,
            recent_deltas: recent,
        }
    }

    /// Return a formatted stats string suitable for logging or `summary()`.
    pub fn stats(&self) -> String {
        let total = self.experiments.len();
        let kept = self.experiments.iter().filter(|e| e.kept).count();
        let avg = if total > 0 {
            self.experiments.iter().map(|e| e.delta).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let best = self
            .experiments
            .iter()
            .map(|e| e.delta)
            .fold(f64::NEG_INFINITY, f64::max);

        format!(
            "exp:{}_total|{}_kept|{:.4}_avg_delta|{:.4}_best_delta|{}_failures|{}_active",
            total,
            kept,
            avg,
            if best == f64::NEG_INFINITY { 0.0 } else { best },
            self.consecutive_failures,
            self.active_hypothesis.is_some(),
        )
    }

    /// Number of experiment records stored.
    pub fn experiment_count(&self) -> usize {
        self.experiments.len()
    }

    /// Return the N most recent experiment records.
    pub fn recent_experiments(&self, n: usize) -> Vec<&ExperimentRecord> {
        self.experiments.iter().rev().take(n).collect()
    }

    // ─── private helpers ──────────────────────────────────────────────

    /// Generate a hypothesis by selecting the metric that needs the most help.
    fn generate_hypothesis(&self, ece: f64, meta_acc: f64, loss: f64) -> String {
        if loss > 0.5 {
            format!("reduce_loss:loss={:.3}", loss)
        } else if ece > 0.15 {
            format!("calibrate_ece:ece={:.3}", ece)
        } else if meta_acc < 0.7 {
            format!("boost_meta_acc:meta={:.3}", meta_acc)
        } else {
            let pool = [
                "adjust_learning_rate",
                "increase_exploration",
                "reduce_threshold",
                "tune_regularization",
                "adjust_temperature",
                "modify_decay_rate",
            ];
            let idx = self.experiments.len() % pool.len();
            pool[idx].to_string()
        }
    }

    /// Deterministic bounded perturbation in [-0.1, +0.1].
    fn simulate_outcome(base: f64, seed: usize) -> f64 {
        let h = (base.to_bits() as u64)
            .wrapping_mul(1_103_515_245)
            .wrapping_add(seed as u64 * 6_366_136_223_846_793_005);
        let frac = ((h >> 16) & 0x7FFF) as f64 / 32767.0;
        let perturbation = (frac - 0.5) * 0.2;
        (base + perturbation).clamp(0.0, 1.0)
    }

    /// Combine ECE / meta-accuracy / loss into a single "higher is better" score.
    fn composite_metric(ece: f64, meta_acc: f64, loss: f64) -> f64 {
        let ece_good = 1.0 - ece.clamp(0.0, 1.0);
        let loss_good = 1.0 - loss.clamp(0.0, 1.0);
        ece_good * 0.3 + meta_acc.clamp(0.0, 1.0) * 0.4 + loss_good * 0.3
    }

    /// Next monotonic experiment ID.
    fn next_id(&self) -> u64 {
        self.experiments
            .back()
            .map(|e| e.id.wrapping_add(1))
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_loop() -> AutoExperimentLoop {
        AutoExperimentLoop::new(AutoExperimentConfig {
            max_experiments_per_tick: 3,
            budget_seconds: 30.0,
            keep_threshold: 0.05,
            max_consecutive_failures: 5,
            metrics_history: 20,
        })
    }

    fn populate(n: usize) -> AutoExperimentLoop {
        let mut loop_ = make_loop();
        for i in 0..n {
            loop_.experiments.push_back(ExperimentRecord {
                id: i as u64,
                hypothesis: "seed".into(),
                metric_before: 0.5,
                metric_after: 0.5,
                delta: 0.0,
                kept: false,
                timestamp_cycle: 0,
            });
        }
        loop_
    }

    // ── 1. basic run ─────────────────────────────────────────────────

    #[test]
    fn test_empty_after_new() {
        let loop_ = AutoExperimentLoop::new(AutoExperimentConfig::default());
        assert_eq!(loop_.experiment_count(), 0);
        assert!(loop_.active_hypothesis.is_none());
        assert_eq!(loop_.consecutive_failures, 0);
    }

    // ── 2. basic run ─────────────────────────────────────────────────

    #[test]
    fn test_run_experiment_basic() {
        let mut loop_ = make_loop();
        let result = loop_.run_experiment(0.1, 0.85, 0.2, 1);
        assert!(result.experiments_run > 0, "should run experiments");
        assert_eq!(loop_.experiment_count(), result.experiments_run);
    }

    // ── 3. keep threshold ──────────────────────────────────────────────

    #[test]
    fn test_experiment_keep_threshold() {
        let mut loop_ = AutoExperimentLoop::new(AutoExperimentConfig {
            keep_threshold: 1.0, // impossible to keep (max perturbation is ±0.1)
            ..AutoExperimentConfig::default()
        });
        let result = loop_.run_experiment(0.5, 0.5, 0.5, 1);
        assert_eq!(result.experiments_kept, 0);
        assert!(loop_.experiments.iter().all(|e| !e.kept));
    }

    // ── 4. revert (delta below threshold) ────────────────────────────────

    #[test]
    fn test_experiment_revert() {
        let mut loop_ = AutoExperimentLoop::new(AutoExperimentConfig {
            keep_threshold: 1.0, // nothing will exceed ±0.1
            ..AutoExperimentConfig::default()
        });
        let result = loop_.run_experiment(0.3, 0.8, 0.2, 1);
        assert_eq!(result.experiments_kept, 0);
        for record in loop_.experiments.iter() {
            assert!(!record.kept, "record {} should be reverted", record.id);
        }
    }

    // ── 5. cooldown ──────────────────────────────────────────────────────

    #[test]
    fn test_consecutive_failures_cooldown() {
        let mut loop_ = make_loop();
        loop_.consecutive_failures = 6; // exceeds max_consecutive_failures = 5
        let result = loop_.run_experiment(0.5, 0.5, 0.5, 1);
        assert_eq!(result.experiments_run, 0, "cooldown should skip all");
        assert_eq!(result.consecutive_failures, 6);
    }

    // ── 6. max experiments per tick ──────────────────────────────────────

    #[test]
    fn test_max_experiments_per_tick() {
        let mut loop_ = AutoExperimentLoop::new(AutoExperimentConfig {
            max_experiments_per_tick: 5,
            ..AutoExperimentConfig::default()
        });
        let result = loop_.run_experiment(0.1, 0.9, 0.1, 1);
        assert_eq!(result.experiments_run, 5);
        assert_eq!(loop_.experiment_count(), 5);
    }

    // ── 7. cap at 100 ────────────────────────────────────────────────────

    #[test]
    fn test_experiments_capped_at_100() {
        let mut loop_ = populate(99);
        let _ = loop_.run_experiment(0.1, 0.9, 0.1, 1);
        assert!(
            loop_.experiment_count() <= 100,
            "got {}",
            loop_.experiment_count()
        );
    }

    // ── 8. stats output ──────────────────────────────────────────────────

    #[test]
    fn test_stats_output() {
        let loop_ = make_loop();
        let s = loop_.stats();
        assert!(!s.is_empty());
        assert!(s.contains("exp:"));

        let mut loop_ = make_loop();
        let _ = loop_.run_experiment(0.1, 0.9, 0.1, 1);
        let s = loop_.stats();
        assert!(s.contains("total|"));
    }

    // ── extra: record fields ─────────────────────────────────────────────

    #[test]
    fn test_record_fields_consistency() {
        let mut loop_ = make_loop();
        let result = loop_.run_experiment(0.2, 0.75, 0.3, 42);
        for record in loop_.experiments.iter() {
            assert_eq!(record.timestamp_cycle, 42);
            assert!((record.metric_after - record.metric_before - record.delta).abs() < 1e-9);
        }
        assert!(result.recent_deltas.len() <= 20);
    }

    // ── extra: hypothesis selection ──────────────────────────────────────

    #[test]
    fn test_hypothesis_generation_is_deterministic() {
        let loop_ = make_loop();
        // healthy metrics → rotation pool
        let h1 = loop_.generate_hypothesis(0.05, 0.95, 0.1);
        assert!(!h1.is_empty());

        // high loss → loss hypothesis
        let h2 = loop_.generate_hypothesis(0.05, 0.95, 0.8);
        assert!(h2.contains("loss"));

        // high ECE → ECE hypothesis
        let h3 = loop_.generate_hypothesis(0.3, 0.95, 0.1);
        assert!(h3.contains("ece"));

        // low meta-accuracy → meta hypothesis
        let h4 = loop_.generate_hypothesis(0.05, 0.4, 0.1);
        assert!(h4.contains("meta"));
    }

    // ── extra: failures reset on success ──────────────────────────────────

    #[test]
    fn test_consecutive_failures_reset_on_success() {
        let mut loop_ = make_loop();
        loop_.consecutive_failures = 4;
        loop_.config.keep_threshold = -1.0; // everything is kept
        let result = loop_.run_experiment(0.2, 0.8, 0.2, 1);
        assert_eq!(result.consecutive_failures, 0);
        assert_eq!(loop_.consecutive_failures, 0);
    }

    // ── extra: simulate_outcome bounds ───────────────────────────────────

    #[test]
    fn test_simulate_outcome_bounds() {
        for base in [0.0, 0.25, 0.5, 0.75, 1.0] {
            for seed in 0..10 {
                let val = AutoExperimentLoop::simulate_outcome(base, seed);
                assert!(
                    (0.0..=1.0).contains(&val),
                    "outcome {val} out of [0,1] (base={base} seed={seed})"
                );
            }
        }
    }
}
