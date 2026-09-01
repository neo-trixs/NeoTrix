use serde::{Deserialize, Serialize};

/// #9 — HPA Axis: stress-cognition feedback loop.
/// Models how sustained cognitive load triggers a stress response that
/// modulates learning rate, exploration vs exploitation, and decision quality.
/// Biological analog: Hypothalamic-Pituitary-Adrenal axis (cortisol feedback).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpaAxisState {
    /// Current cortisol-equivalent stress level [0, 1]
    pub cortisol: f64,
    /// Stress accumulation rate per unit cognitive load
    pub sensitivity: f64,
    /// Recovery rate when load is low
    pub recovery_rate: f64,
    /// Basal (minimum) cortisol level
    pub basal: f64,
    /// Maximum sustainable cortisol before impairment
    pub ceiling: f64,
    /// Number of consecutive high-stress ticks
    pub stress_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HpaPhase {
    /// Low stress, optimal cognition
    Baseline,
    /// Moderate stress, enhanced focus
    Alert,
    /// High stress, impaired cognition
    Overload,
    /// Recovery phase after overload
    Recovery,
}

impl HpaAxisState {
    pub fn new(sensitivity: f64, recovery_rate: f64) -> Self {
        Self {
            cortisol: 0.2,
            sensitivity,
            recovery_rate,
            basal: 0.1,
            ceiling: 0.85,
            stress_duration: 0,
        }
    }

    /// Update cortisol based on current cognitive load [0, 1].
    pub fn update(&mut self, cognitive_load: f64, dt: f64) {
        let production = cognitive_load * self.sensitivity;
        let clearance = (self.cortisol - self.basal) * self.recovery_rate;
        let delta = (production - clearance) * dt;
        self.cortisol = (self.cortisol + delta).clamp(0.0, 1.0);
        if self.cortisol > self.basal + 0.3 {
            self.stress_duration += 1;
        } else {
            self.stress_duration = 0;
        }
    }

    /// Current phase based on cortisol level and trend.
    pub fn phase(&self) -> HpaPhase {
        if self.cortisol > self.ceiling {
            HpaPhase::Overload
        } else if self.stress_duration > 0 && self.cortisol <= self.basal + 0.05 {
            HpaPhase::Recovery
        } else if self.cortisol > self.basal + 0.3 {
            HpaPhase::Alert
        } else {
            HpaPhase::Baseline
        }
    }

    /// Modulate learning rate based on HPA phase.
    /// Baseline → normal LR, Alert → increased, Overload → decreased.
    pub fn modulated_learning_rate(&self, base_lr: f64) -> f64 {
        match self.phase() {
            HpaPhase::Baseline => base_lr,
            HpaPhase::Alert => base_lr * (1.0 + (self.cortisol - self.basal - 0.3) * 2.0).min(1.5),
            HpaPhase::Overload => base_lr * (1.0 - (self.cortisol - self.ceiling) * 3.0).max(0.1),
            HpaPhase::Recovery => base_lr * 0.5,
        }
    }

    /// Modulate exploration rate: higher cortisol = more exploitation (conservatism).
    pub fn modulated_exploration(&self, base_explore: f64) -> f64 {
        match self.phase() {
            HpaPhase::Baseline => base_explore,
            HpaPhase::Alert => base_explore * (1.0 - (self.cortisol - self.basal - 0.3) * 0.5).max(0.3),
            HpaPhase::Overload => base_explore * 0.1,
            HpaPhase::Recovery => base_explore * 0.5,
        }
    }

    /// Decision quality penalty: 0 = no penalty, 1 = total impairment.
    pub fn decision_impairment(&self) -> f64 {
        match self.phase() {
            HpaPhase::Baseline => 0.0,
            HpaPhase::Alert => ((self.cortisol - self.basal - 0.3) * 0.5).max(0.0),
            HpaPhase::Overload => ((self.cortisol - self.ceiling) / (1.0 - self.ceiling) * 0.8).min(0.8),
            HpaPhase::Recovery => 0.2,
        }
    }
}

impl Default for HpaAxisState {
    fn default() -> Self {
        Self::new(1.5, 0.8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpa_default_is_baseline() {
        let hpa = HpaAxisState::default();
        assert!(matches!(hpa.phase(), HpaPhase::Baseline));
    }

    #[test]
    fn test_hpa_high_load_raises_cortisol() {
        let mut hpa = HpaAxisState::new(2.0, 0.5);
        for _ in 0..10 {
            hpa.update(0.9, 1.0);
        }
        assert!(hpa.cortisol > 0.5);
        assert!(matches!(hpa.phase(), HpaPhase::Alert | HpaPhase::Overload));
    }

    #[test]
    fn test_hpa_overload_reduces_learning_rate() {
        let mut hpa = HpaAxisState::new(3.0, 0.2);
        for _ in 0..20 {
            hpa.update(1.0, 1.0);
        }
        let lr = hpa.modulated_learning_rate(0.1);
        assert!(lr < 0.09, "Overload should reduce learning rate, got {}", lr);
    }

    #[test]
    fn test_hpa_overload_reduces_exploration() {
        let mut hpa = HpaAxisState::new(3.0, 0.2);
        for _ in 0..20 {
            hpa.update(1.0, 1.0);
        }
        let explore = hpa.modulated_exploration(0.5);
        assert!(explore < 0.2, "Overload should reduce exploration, got {}", explore);
    }

    #[test]
    fn test_hpa_recovers_after_load_drops() {
        let mut hpa = HpaAxisState::new(2.0, 0.8);
        for _ in 0..10 {
            hpa.update(1.0, 1.0);
        }
        assert!(hpa.cortisol > 0.5);
        for _ in 0..30 {
            hpa.update(0.0, 1.0);
        }
        assert!(hpa.cortisol < 0.25, "Cortisol should recover, got {}", hpa.cortisol);
        assert!(matches!(hpa.phase(), HpaPhase::Baseline | HpaPhase::Recovery));
    }
}
