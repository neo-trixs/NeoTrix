//! Phase 9.2 — Dynamic Self-Model (动态自我模型 · 持续自评估).
//!
//! MIRROR §5 / Machine Consciousness (2026): beyond the static `SystemIdentity`
//! capability snapshot, the self-model continuously estimates its own running
//! state from the workspace (`h_ws`) and the meta-workspace (`h_meta`):
//!
//!     M_self^(t) = f_self(h_ws^(t), h_meta^(t-1))
//!
//! Outputs:
//!   - a capability vector (how capable the system currently is),
//!   - an uncertainty estimate (how confident it should be),
//!   - a fatigue level (how much it has been pushed this session).
//!
//! The self-model error `R_self = -‖M_self − observed_behavior‖` is the intrinsic
//! reward signal: when the model's self-estimate diverges from observed behavior
//! (e.g. it believed it was capable but kept failing), the discrepancy is used
//! to tighten the model and to drive corrective motivation.

use serde::{Deserialize, Serialize};

/// Number of observed-behavior samples retained for self-error estimation.
pub const SELF_HISTORY: usize = 32;

/// Current self-estimate produced by the dynamic self-model.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SelfState {
    /// Aggregate capability estimate (0..1) — how well the system is performing.
    pub capability: f64,
    /// Uncertainty estimate (0..1) — 0 = confident, 1 = extremely uncertain.
    pub uncertainty: f64,
    /// Fatigue level (0..1) — accumulated load vs. available budget.
    pub fatigue: f64,
    /// Self-model error vs. observed behavior (0..1). Higher = worse self-model.
    pub self_error: f64,
}

impl Default for SelfState {
    fn default() -> Self {
        Self {
            capability: 0.5,
            uncertainty: 0.5,
            fatigue: 0.0,
            self_error: 0.0,
        }
    }
}

/// Phase 9.2 — dynamic self-model.
///
/// Tracks a rolling record of (predicted capability, observed outcome) pairs and
/// updates a running estimate of capability, uncertainty, and fatigue each tick.
/// The self-error term is the discrepancy between the model's own estimate and
/// the observed outcome — used as an intrinsic reward signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    /// Current self-estimate.
    pub state: SelfState,
    /// Rolling predicted-capability history.
    pub predicted_history: std::collections::VecDeque<f64>,
    /// Rolling observed-outcome history.
    pub observed_history: std::collections::VecDeque<f64>,
    /// Running fatigue budget (0..1).
    pub fatigue: f64,
    /// Number of updates performed.
    pub updates: u64,
    /// Recent self-model error (for telemetry).
    pub last_self_error: f64,
    /// Learning rate for the fatigue/capability estimator.
    pub lr: f64,
}

impl Default for SelfModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfModel {
    /// Create a self-model with neutral priors.
    pub fn new() -> Self {
        Self {
            state: SelfState::default(),
            predicted_history: std::collections::VecDeque::with_capacity(SELF_HISTORY),
            observed_history: std::collections::VecDeque::with_capacity(SELF_HISTORY),
            fatigue: 0.0,
            updates: 0,
            last_self_error: 0.0,
            lr: 0.1,
        }
    }

    /// One self-observation tick.
    ///
    /// `workspace_signal` ∈ [0,1] is an aggregate health/quality measure of the
    /// workspace this cycle (higher = better). `load_delta` ∈ [0,1] is the amount
    /// of additional effort spent this cycle (raises fatigue). `meta_alarm` is
    /// the count of meta-observations (over-activation / entropy anomaly etc.)
    /// which raise uncertainty.
    pub fn tick(&mut self, workspace_signal: f64, load_delta: f64, meta_alarm: usize) -> SelfState {
        self.updates += 1;

        // Prior estimate = current capability (used to compute self-error).
        let prior = self.state.capability;
        let observed = workspace_signal.clamp(0.0, 1.0);

        // Push into history.
        self.predicted_history.push_back(prior);
        self.observed_history.push_back(observed);
        while self.predicted_history.len() > SELF_HISTORY {
            self.predicted_history.pop_front();
            self.observed_history.pop_front();
        }

        // Self-model error: |prior estimate − observed outcome| over recent history.
        let n = self.predicted_history.len() as f64;
        let err_sum: f64 = self
            .predicted_history
            .iter()
            .zip(self.observed_history.iter())
            .map(|(p, o)| (p - o).abs())
            .sum();
        let self_error = if n > 0.0 { err_sum / n } else { 0.0 };
        self.last_self_error = self_error;

        // Capability: exponential moving average toward observed behavior.
        self.state.capability += self.lr * (observed - self.state.capability);

        // Fatigue: accumulate load, slowly recover.
        self.fatigue = (self.fatigue + self.lr * load_delta - 0.005).clamp(0.0, 1.0);
        self.state.fatigue = self.fatigue;

        // Uncertainty: baseline + self-error + meta-alarm pressure.
        let u = (0.1 + self_error * 0.5 + (meta_alarm as f64).min(3.0) * 0.1).clamp(0.0, 1.0);
        self.state.uncertainty = u;
        self.state.self_error = self_error;
        self.state
    }

    /// Intrinsic self-model reward `R_self = -‖M_self − observed_behavior‖`.
    ///
    /// Negative (self-model is wrong → corrective pressure) or near-zero.
    pub fn self_reward(&self) -> f64 {
        -self.last_self_error
    }

    /// Estimated intrinsic reward including a fatigue penalty.
    pub fn combined_intrinsic_reward(&self) -> f64 {
        -self.last_self_error - self.fatigue * 0.3
    }

    /// Current self-estimate snapshot.
    pub fn current(&self) -> SelfState {
        self.state
    }

    /// Reset all estimation state to neutral priors.
    pub fn reset(&mut self) {
        self.state = SelfState::default();
        self.predicted_history.clear();
        self.observed_history.clear();
        self.fatigue = 0.0;
        self.updates = 0;
        self.last_self_error = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_neutral() {
        let m = SelfModel::new();
        assert_eq!(m.state.capability, 0.5);
        assert_eq!(m.state.uncertainty, 0.5);
        assert_eq!(m.state.fatigue, 0.0);
    }

    #[test]
    fn test_tick_moves_capability_toward_observation() {
        let mut m = SelfModel::new();
        for _ in 0..20 {
            m.tick(0.95, 0.0, 0);
        }
        assert!(m.state.capability > 0.8, "capability should rise, got {}", m.state.capability);
    }

    #[test]
    fn test_tick_accumulates_fatigue() {
        let mut m = SelfModel::new();
        for _ in 0..50 {
            m.tick(0.6, 0.2, 0);
        }
        assert!(m.state.fatigue > 0.2, "fatigue should accumulate, got {}", m.state.fatigue);
    }

    #[test]
    fn test_self_error_detects_miscalibration() {
        let mut m = SelfModel::new();
        // Prior starts at 0.5 but observed is consistently 0.05 → model is wrong.
        for _ in 0..10 {
            m.tick(0.05, 0.0, 0);
        }
        assert!(m.last_self_error > 0.2, "self-error should be high, got {}", m.last_self_error);
        assert!(m.self_reward() < 0.0);
    }

    #[test]
    fn test_calibrated_model_low_error() {
        let mut m = SelfModel::new();
        // Prior 0.5 matches observation 0.5 → near-zero error after warmup.
        for _ in 0..20 {
            m.tick(0.5, 0.0, 0);
        }
        assert!(m.last_self_error < 0.15, "calibrated self-error should be low, got {}", m.last_self_error);
    }

    #[test]
    fn test_meta_alarm_raises_uncertainty() {
        let mut m = SelfModel::new();
        let no_alarm = m.tick(0.5, 0.0, 0);
        let alarm = m.tick(0.5, 0.0, 3);
        assert!(
            alarm.uncertainty >= no_alarm.uncertainty,
            "meta alarms should not lower uncertainty: {} vs {}",
            alarm.uncertainty,
            no_alarm.uncertainty
        );
    }

    #[test]
    fn test_combined_reward_penalizes_fatigue() {
        let mut m = SelfModel::new();
        m.tick(0.5, 0.0, 0);
        let fresh = m.combined_intrinsic_reward();
        let mut m2 = SelfModel::new();
        for _ in 0..30 {
            m2.tick(0.5, 0.5, 0);
        }
        let tired = m2.combined_intrinsic_reward();
        assert!(tired <= fresh, "fatigued reward ({tired}) should be ≤ fresh ({fresh})");
    }

    #[test]
    fn test_reset_returns_to_neutral() {
        let mut m = SelfModel::new();
        for _ in 0..10 {
            m.tick(0.05, 0.5, 2);
        }
        assert!(m.state.capability < 0.5);
        m.reset();
        assert_eq!(m.state.capability, 0.5);
        assert_eq!(m.state.fatigue, 0.0);
        assert_eq!(m.updates, 0);
    }

    #[test]
    fn test_self_model_accuracy_converges() {
        // self-model 准确性: 模型应能区分"观测稳定 (模型准)"与"观测剧烈波动 (模型不准)"。
        // 恒定观测: 能力估计收敛到观测值 → 预测先验与观测一致 → self-error 收敛到低值。
        let mut stable = SelfModel::new();
        for _ in 0..40 {
            stable.tick(0.9, 0.0, 0);
        }
        assert!(
            stable.state.self_error < 0.15,
            "stable observation should converge to low self-error, got {}",
            stable.state.self_error
        );

        // 周期观测 (10×0.9 后 10×0.1, 重复 4 轮): 能力估计无法跟上切换 →
        // 预测先验与观测持续背离 → self-error 反映真实不一致 (> 0.1)。
        let mut volatile = SelfModel::new();
        for _ in 0..4 {
            for _ in 0..10 {
                volatile.tick(0.9, 0.0, 0);
            }
            for _ in 0..10 {
                volatile.tick(0.1, 0.0, 0);
            }
        }
        assert!(
            volatile.state.self_error > 0.1,
            "periodic observation should keep self-error high (model is wrong), got {}",
            volatile.state.self_error
        );
        // 波动模型的误差必须显著高于稳定模型
        assert!(
            volatile.state.self_error > stable.state.self_error * 2.0,
            "volatile self-error ({}) should exceed stable ({}) substantially",
            volatile.state.self_error,
            stable.state.self_error
        );
    }
}
