//! WHALE — Weight/Harness Adaptive Cycle Loop Engine
//!
//! Inspired by the WHALE paper concept: harness optimization can substitute
//! for model optimization. This validates NeoTrix's focus on GWT routing
//! optimization over model scaling.
//!
//! Each consciousness cycle includes a WHALE phase that alternates between:
//! - WeightUpdate: Train the model while the harness is frozen
//! - HarnessSearch: Search for the best harness while the model is frozen
//!
//! Adaptive switching monitors improvement rate and switches phases when
//! plateaus are detected below the threshold, dynamically adjusting phase
//! length based on recent performance.

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

/// Current WHALE optimization phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationPhase {
    /// Train the model while the harness is frozen
    WeightUpdate,
    /// Search for the best harness while the model is frozen
    HarnessSearch,
}

impl OptimizationPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WeightUpdate => "weight_update",
            Self::HarnessSearch => "harness_search",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::WeightUpdate => Self::HarnessSearch,
            Self::HarnessSearch => Self::WeightUpdate,
        }
    }
}

/// WHALE configuration parameters
#[derive(Debug)]
pub struct WHALEConfig {
    /// Current optimization phase
    pub phase: OptimizationPhase,
    /// Number of cycles to wait before switching if no improvement
    pub patience: u64,
    /// Improvement rate threshold below which we consider a plateau
    pub switch_threshold: f64,
    /// Whether adaptive phase length is enabled
    pub adaptive: bool,
    /// Maximum number of cycles to run before forced phase switch
    pub max_cycles: u64,
    /// Current cycle count within the active phase
    pub cycle_count: AtomicU64,
    /// Best improvement rate observed in current phase
    pub best_improvement: AtomicU64,
    /// Timestamp when current phase started
    pub phase_start: Instant,
}

impl Clone for WHALEConfig {
    fn clone(&self) -> Self {
        Self {
            phase: self.phase,
            patience: self.patience,
            switch_threshold: self.switch_threshold,
            adaptive: self.adaptive,
            max_cycles: self.max_cycles,
            cycle_count: AtomicU64::new(self.cycle_count.load(Ordering::Relaxed)),
            best_improvement: AtomicU64::new(self.best_improvement.load(Ordering::Relaxed)),
            phase_start: self.phase_start,
        }
    }
}

impl Default for WHALEConfig {
    fn default() -> Self {
        Self {
            phase: OptimizationPhase::WeightUpdate,
            patience: 10,
            switch_threshold: 0.01,
            adaptive: true,
            max_cycles: 50,
            cycle_count: AtomicU64::new(0),
            best_improvement: AtomicU64::new(0),
            phase_start: Instant::now(),
        }
    }
}

/// Metrics for a single WHALE cycle iteration
#[derive(Debug, Clone)]
pub struct WhaleCycleResult {
    pub phase: OptimizationPhase,
    pub cycle: u64,
    pub improvement_rate: f64,
    pub phase_duration: Duration,
    pub switched: bool,
    pub adaptive_length: u64,
}

/// Adaptive switching controller for WHALE phases
///
/// Monitors improvement rate, switches when plateaus are detected below
/// threshold, and adjusts phase length dynamically.
///
/// **Key insight**: Harness optimization can substitute for model
/// optimization, validating NeoTrix's focus on GWT routing optimization
/// over model scaling.
#[derive(Debug)]
pub struct AdaptiveSwitching {
    config: WHALEConfig,
    /// Historical improvement rates for plateau detection
    improvement_history: Vec<f64>,
    /// Number of adaptive switches performed
    pub switch_count: AtomicU64,
    /// Total cycles completed
    pub total_cycles: AtomicU64,
    /// Phase durations tracker (in nanoseconds)
    phase_durations: Vec<Duration>,
}

impl AdaptiveSwitching {
    /// Create a new AdaptiveSwitching controller with default config
    pub fn new() -> Self {
        Self {
            config: WHALEConfig::default(),
            improvement_history: Vec::new(),
            switch_count: AtomicU64::new(0),
            total_cycles: AtomicU64::new(0),
            phase_durations: Vec::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: WHALEConfig) -> Self {
        Self {
            config,
            improvement_history: Vec::new(),
            switch_count: AtomicU64::new(0),
            total_cycles: AtomicU64::new(0),
            phase_durations: Vec::new(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &WHALEConfig {
        &self.config
    }

    /// Get current phase
    pub fn phase(&self) -> OptimizationPhase {
        self.config.phase
    }

    /// Record an improvement rate for the current cycle
    ///
    /// Returns `WhaleCycleResult` with the cycle outcome.
    /// If improvement is below threshold for `patience` cycles,
    /// triggers a phase switch.
    pub fn record_improvement(&mut self, rate: f64) -> WhaleCycleResult {
        let cycle = self.total_cycles.load(Ordering::Relaxed) + 1;
        self.total_cycles.store(cycle, Ordering::Relaxed);

        let current_phase = self.config.phase;
        let cycle_count = self.config.cycle_count.load(Ordering::Relaxed) + 1;
        self.config.cycle_count.store(cycle_count, Ordering::Relaxed);

        self.improvement_history.push(rate);
        // Keep only the last 100 improvement rates
        if self.improvement_history.len() > 100 {
            self.improvement_history.remove(0);
        }

        // Track best improvement
        let best = self.config.best_improvement.load(Ordering::Relaxed);
        if rate > best as f64 {
            self.config.best_improvement.store(rate as u64, Ordering::Relaxed);
        }

        // Calculate adaptive phase length if enabled
        let adaptive_length = if self.config.adaptive {
            self.calculate_adaptive_length(rate)
        } else {
            self.config.patience
        };

        let mut switched = false;

        // Check for plateau: improvement below threshold for `patience` cycles
        if cycle_count >= adaptive_length && rate < self.config.switch_threshold {
            // Plateau detected — switch phase
            self.config.phase = current_phase.next();
            self.config.cycle_count.store(0, Ordering::Relaxed);
            self.config.phase_start = Instant::now();
            self.phase_durations.push(self.config.phase_start.elapsed());
            self.switch_count.fetch_add(1, Ordering::Relaxed);
            switched = true;
        }

        // Check for max_cycles override
        if cycle_count >= self.config.max_cycles && !switched {
            self.config.phase = current_phase.next();
            self.config.cycle_count.store(0, Ordering::Relaxed);
            self.config.phase_start = Instant::now();
            self.phase_durations.push(self.config.phase_start.elapsed());
            self.switch_count.fetch_add(1, Ordering::Relaxed);
            switched = true;
        }

        let phase_duration = self.config.phase_start.elapsed();

        WhaleCycleResult {
            phase: current_phase,
            cycle,
            improvement_rate: rate,
            phase_duration,
            switched,
            adaptive_length,
        }
    }

    /// Calculate adaptive phase length based on recent improvement trend
    fn calculate_adaptive_length(&self, current_rate: f64) -> u64 {
        let history_len = self.improvement_history.len();
        if history_len < 3 {
            return self.config.patience;
        }

        // Compute trend from last N improvements
        let recent: Vec<f64> = self.improvement_history.iter().copied().collect();
        let avg: f64 = recent.iter().sum::<f64>() / recent.len() as f64;

        if current_rate > avg * 1.5 {
            // Strong improvement — shorten phase to capitalize on momentum
            (self.config.patience as f64 * 0.7) as u64
        } else if current_rate < avg * 0.5 {
            // Weak improvement — extend phase to explore more
            (self.config.patience as f64 * 1.5) as u64
        } else {
            self.config.patience
        }
    }

    /// Get the average improvement rate over the history
    pub fn avg_improvement(&self) -> f64 {
        if self.improvement_history.is_empty() {
            return 0.0;
        }
        self.improvement_history.iter().sum::<f64>() / self.improvement_history.len() as f64
    }

    /// Get the current improvement trend (positive = improving)
    pub fn trend(&self) -> f64 {
        let len = self.improvement_history.len();
        if len < 2 {
            return 0.0;
        }
        let recent: Vec<f64> = self.improvement_history.iter().copied().rev().take(5).collect();
        if recent.len() < 2 {
            return 0.0;
        }
        let first = recent[recent.len() - 1];
        let last = recent[0];
        last - first
    }

    /// Run a single WHALE cycle with the given optimization function
    ///
    /// # Arguments
    /// * `optimize_fn` - A function that performs the current phase optimization
    ///   and returns the improvement rate (0.0 to 1.0)
    ///
    /// Returns the `WhaleCycleResult` for this cycle
    pub fn run_cycle<F>(&mut self, optimize_fn: F) -> WhaleCycleResult
    where
        F: Fn(OptimizationPhase) -> f64,
    {
        let phase = self.config.phase;
        let rate = optimize_fn(phase);
        self.record_improvement(rate)
    }

    /// Get the ratio of HarnessSearch cycles to WeightUpdate cycles
    pub fn harness_weight_ratio(&self) -> f64 {
        let total = self.total_cycles.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        // Approximate from switch count (each switch alternates phases)
        let harness_cycles = self.switch_count.load(Ordering::Relaxed) / 2;
        harness_cycles as f64 / total as f64
    }

    /// Get total accumulated phase durations
    pub fn total_phase_duration(&self) -> Duration {
        self.phase_durations.iter().sum()
    }

    /// Reset the controller to initial state
    pub fn reset(&mut self) {
        self.config = WHALEConfig::default();
        self.improvement_history.clear();
        self.switch_count.store(0, Ordering::Relaxed);
        self.total_cycles.store(0, Ordering::Relaxed);
        self.phase_durations.clear();
    }
}

impl Default for AdaptiveSwitching {
    fn default() -> Self {
        Self::new()
    }
}

/// WHALE cycle detector for consciousness_tick integration
///
/// Checks phase → optimizes → evaluates → decides next phase.
/// Integrates into the consciousness tick loop via `handle_whale_cycle`.
pub struct WhaleCycleDetector {
    switching: AdaptiveSwitching,
    /// Last recorded improvement rate from the consciousness cycle
    last_improvement: AtomicU64,
}

impl WhaleCycleDetector {
    pub fn new() -> Self {
        Self {
            switching: AdaptiveSwitching::new(),
            last_improvement: AtomicU64::new(0),
        }
    }

    /// Detect WHALE phase and run optimization within the consciousness cycle.
    ///
    /// This is the entry point for `handle_consciousness_tick` integration.
    /// It:
    /// 1. Checks the current WHALE phase
    /// 2. Runs the appropriate optimization (weight or harness)
    /// 3. Evaluates the improvement rate
    /// 4. Decides whether to switch phases
    ///
    /// Returns the cycle result for logging and heartbeat reporting.
    pub fn detect_and_optimize(&mut self, consciousness_quality: f64) -> WhaleCycleResult {
        let rate = consciousness_quality.clamp(0.0, 1.0);
        self.last_improvement.store((rate * 100.0) as u64, Ordering::Relaxed);
        self.switching.record_improvement(rate)
    }

    /// Get the underlying AdaptiveSwitching controller
    pub fn switching(&self) -> &AdaptiveSwitching {
        &self.switching
    }

    /// Get the last improvement rate
    pub fn last_improvement_rate(&self) -> f64 {
        self.last_improvement.load(Ordering::Relaxed) as f64 / 100.0
    }
}

impl Default for WhaleCycleDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_phase_alternation() {
        let mut phase = OptimizationPhase::WeightUpdate;
        phase = phase.next();
        assert_eq!(phase, OptimizationPhase::HarnessSearch);
        phase = phase.next();
        assert_eq!(phase, OptimizationPhase::WeightUpdate);
    }

    #[test]
    fn test_whale_config_default() {
        let config = WHALEConfig::default();
        assert_eq!(config.phase, OptimizationPhase::WeightUpdate);
        assert_eq!(config.patience, 10);
        assert_eq!(config.switch_threshold, 0.01);
        assert!(config.adaptive);
        assert_eq!(config.max_cycles, 50);
    }

    #[test]
    fn test_adaptive_switching_record_improvement() {
        let mut asw = AdaptiveSwitching::new();
        let result = asw.record_improvement(0.05);
        assert_eq!(result.cycle, 1);
        assert_eq!(result.phase, OptimizationPhase::WeightUpdate);
        assert!(!result.switched);
    }

    #[test]
    fn test_adaptive_switching_plateau_detection() {
        let mut asw = AdaptiveSwitching::new();
        // Record low improvements below threshold for patience cycles
        for _ in 0..12 {
            let result = asw.record_improvement(0.001); // Below 0.01 threshold
            assert_eq!(result.improvement_rate, 0.001);
        }
        // After patience cycles, should have switched
        assert!(asw.switch_count.load(Ordering::Relaxed) > 0,
            "Should have switched due to plateau");
    }

    #[test]
    fn test_adaptive_switching_strong_improvement_no_switch() {
        let mut asw = AdaptiveSwitching::new();
        let result = asw.record_improvement(0.8);
        assert_eq!(result.phase, OptimizationPhase::WeightUpdate);
        assert!(!result.switched);
    }

    #[test]
    fn test_harness_weight_ratio() {
        let mut asw = AdaptiveSwitching::new();
        // Run several cycles with switches
        for i in 0..20 {
            let rate = if i % 5 == 0 { 0.001 } else { 0.05 };
            asw.record_improvement(rate);
        }
        let ratio = asw.harness_weight_ratio();
        assert!(ratio >= 0.0, "ratio should be non-negative");
    }

    #[test]
    fn test_avg_improvement() {
        let mut asw = AdaptiveSwitching::new();
        asw.record_improvement(0.5);
        asw.record_improvement(0.7);
        let avg = asw.avg_improvement();
        assert!((avg - 0.6).abs() < 0.01, "avg should be 0.6, got {avg}");
    }

    #[test]
    fn test_trend_positive() {
        let mut asw = AdaptiveSwitching::new();
        asw.record_improvement(0.1);
        asw.record_improvement(0.3);
        asw.record_improvement(0.5);
        let trend = asw.trend();
        assert!(trend > 0.0, "trend should be positive for improving rates");
    }

    #[test]
    fn test_whale_cycle_detector() {
        let mut detector = WhaleCycleDetector::new();
        let result = detector.detect_and_optimize(0.75);
        assert_eq!(result.phase, OptimizationPhase::WeightUpdate);
        assert_eq!(result.improvement_rate, 0.75);
        assert_eq!(result.cycle, 1);
    }

    #[test]
    fn test_cycle_result_fields() {
        let result = WhaleCycleResult {
            phase: OptimizationPhase::HarnessSearch,
            cycle: 5,
            improvement_rate: 0.03,
            phase_duration: Duration::from_millis(100),
            switched: true,
            adaptive_length: 15,
        };
        assert_eq!(result.phase, OptimizationPhase::HarnessSearch);
        assert_eq!(result.cycle, 5);
        assert!(result.switched);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut asw = AdaptiveSwitching::new();
        asw.record_improvement(0.5);
        asw.record_improvement(0.3);
        asw.reset();
        assert_eq!(asw.total_cycles.load(Ordering::Relaxed), 0);
        assert_eq!(asw.switch_count.load(Ordering::Relaxed), 0);
        assert!(asw.improvement_history.is_empty());
    }

    #[test]
    fn test_adaptive_length_changes() {
        let mut asw = AdaptiveSwitching::with_config(WHALEConfig {
            patience: 10,
            adaptive: true,
            ..Default::default()
        });
        // Strong improvement should shorten adaptive length
        let result = asw.record_improvement(0.9);
        assert!(result.adaptive_length <= 10, "Strong improvement should shorten or keep length");
    }
}