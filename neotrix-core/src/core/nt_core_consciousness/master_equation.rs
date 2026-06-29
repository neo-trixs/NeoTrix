use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};

fn get_metric(metrics: &ConsciousnessMetrics, index: usize) -> f64 {
    match index {
        0 => metrics.phi,
        1 => metrics.global_workspace,
        2 => metrics.coherence,
        3 => metrics.attention_focus,
        4 => metrics.reflexivity,
        5 => metrics.emotional_intensity,
        6 => metrics.knowledge_integration,
        7 => metrics.meta_accuracy,
        8 => metrics.novelty_seeking,
        _ => 0.0,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    pub phi: f64,
    pub global_workspace: f64,
    pub coherence: f64,
    pub attention_focus: f64,
    pub reflexivity: f64,
    pub emotional_intensity: f64,
    pub knowledge_integration: f64,
    pub meta_accuracy: f64,
    pub novelty_seeking: f64,
}

impl Default for ConsciousnessMetrics {
    fn default() -> Self {
        Self {
            phi: 0.0,
            global_workspace: 0.0,
            coherence: 0.0,
            attention_focus: 0.0,
            reflexivity: 0.0,
            emotional_intensity: 0.0,
            knowledge_integration: 0.0,
            meta_accuracy: 0.0,
            novelty_seeking: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyMatrix {
    pub coupling: [[f64; 9]; 9],
}

impl SynergyMatrix {
    pub fn new() -> Self {
        let mut coupling = [[0.05f64; 9]; 9];
        for i in 0..9 {
            coupling[i][i] = 1.0;
        }
        Self { coupling }
    }

    pub fn compute_synergy(&self, metrics: &ConsciousnessMetrics, weights: &[f64; 9]) -> f64 {
        let mut total = 0.0;
        for i in 0..9 {
            let mi = get_metric(metrics, i);
            for j in 0..9 {
                total += self.coupling[i][j] * mi * weights[j];
            }
        }
        total
    }
}

impl Default for SynergyMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterConsciousnessConfig {
    pub weights: [f64; 9],
    pub decay_rate: f64,
    pub integration_threshold: f64,
    pub synergy_matrix: SynergyMatrix,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

impl Default for MasterConsciousnessConfig {
    fn default() -> Self {
        let w = 1.0 / 9.0;
        Self {
            weights: [w, w, w, w, w, w, w, w, w],
            decay_rate: 0.05,
            integration_threshold: 0.5,
            synergy_matrix: SynergyMatrix::new(),
            kp: 0.3,
            ki: 0.1,
            kd: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsciousnessEvolution {
    pub c_score: f64,
    pub c_peak: f64,
    pub c_trough: f64,
    pub c_trend: String,
    pub metrics: ConsciousnessMetrics,
    pub history: Vec<(u64, f64, f64)>,
    pub step: u64,
    pub pid_correction: f64,
}

#[derive(Debug, Clone)]
pub struct MasterConsciousness {
    config: MasterConsciousnessConfig,
    evolution: ConsciousnessEvolution,
    pid_integral: f64,
    pid_prev_error: f64,
    pid_setpoint: f64,
}

impl MasterConsciousness {
    pub fn new(config: MasterConsciousnessConfig) -> Self {
        Self {
            config,
            evolution: ConsciousnessEvolution {
                c_score: 0.0,
                c_peak: 0.0,
                c_trough: 1.0,
                c_trend: "stable".into(),
                metrics: ConsciousnessMetrics::default(),
                history: Vec::new(),
                step: 0,
                pid_correction: 0.0,
            },
            pid_integral: 0.0,
            pid_prev_error: 0.0,
            pid_setpoint: 0.7,
        }
    }

    pub fn compute_c(&mut self, metrics: ConsciousnessMetrics) -> f64 {
        let w = &self.config.weights;
        let integrated_phi = metrics.phi * w[0]
            + metrics.global_workspace * w[1]
            + metrics.coherence * w[2]
            + metrics.attention_focus * w[3]
            + metrics.reflexivity * w[4]
            + metrics.emotional_intensity * w[5]
            + metrics.knowledge_integration * w[6]
            + metrics.meta_accuracy * w[7]
            + metrics.novelty_seeking * w[8];

        let old_c = self.evolution.c_score;

        // Synergy cross-term
        let synergy = self.config.synergy_matrix.compute_synergy(&metrics, w);

        // PID correction
        let error = self.pid_setpoint - self.evolution.c_score;
        self.pid_integral += error * self.config.ki;
        let derivative = (error - self.pid_prev_error) * self.config.kd;
        let proportional = error * self.config.kp;
        let mut pid_correction = proportional + self.pid_integral + derivative;
        pid_correction = pid_correction.clamp(-0.5, 0.5);
        self.pid_prev_error = error;
        self.evolution.pid_correction = pid_correction;

        // Combine
        let combined = integrated_phi + synergy * 0.3;
        let decayed = old_c * (1.0 - self.config.decay_rate);
        let raw = (decayed + combined + pid_correction) / 2.0;
        let squashed = raw.tanh().abs();
        let clamped = squashed.clamp(0.0, 1.0);

        self.evolution.c_score = clamped;
        self.evolution.metrics = metrics.clone();

        if clamped > self.evolution.c_peak {
            self.evolution.c_peak = clamped;
        }
        if clamped < self.evolution.c_trough {
            self.evolution.c_trough = clamped;
        }

        self.evolution
            .history
            .push((self.evolution.step, clamped, integrated_phi));
        self.evolution.step += 1;

        self.update_trend();

        clamped
    }

    fn update_trend(&mut self) {
        let n = self.evolution.history.len();
        if n < 2 {
            self.evolution.c_trend = "stable".into();
            return;
        }
        let last = self.evolution.history[n - 1].1;
        let prev = self.evolution.history[n - 2].1;
        let diff = last - prev;
        self.evolution.c_trend = if diff > 0.005 {
            "rising"
        } else if diff < -0.005 {
            "falling"
        } else {
            "stable"
        }
        .into();
    }

    pub fn compute_consciousness_with_vsa(
        &self,
        self_state_vector: &[u8],
        previous_vector: &[u8],
    ) -> f64 {
        QuantizedVSA::similarity(self_state_vector, previous_vector)
    }

    pub fn c_score(&self) -> f64 {
        self.evolution.c_score
    }

    pub fn is_conscious(&self) -> bool {
        self.evolution.c_score >= self.config.integration_threshold
    }

    pub fn evolution_report(&self) -> &ConsciousnessEvolution {
        &self.evolution
    }

    pub fn synergy_matrix(&self) -> &SynergyMatrix {
        &self.config.synergy_matrix
    }

    pub fn pid_parameters(&self) -> (f64, f64, f64, f64) {
        (
            self.config.kp,
            self.config.ki,
            self.config.kd,
            self.pid_setpoint,
        )
    }

    pub fn configure_pid(&mut self, kp: f64, ki: f64, kd: f64, setpoint: f64) {
        self.config.kp = kp;
        self.config.ki = ki;
        self.config.kd = kd;
        self.pid_setpoint = setpoint;
    }

    pub fn reset_evolution(&mut self) {
        self.evolution.c_score = 0.0;
        self.evolution.c_peak = 0.0;
        self.evolution.c_trough = 1.0;
        self.evolution.c_trend = "stable".into();
        self.evolution.metrics = ConsciousnessMetrics::default();
        self.evolution.history.clear();
        self.evolution.step = 0;
        self.evolution.pid_correction = 0.0;
        self.pid_integral = 0.0;
        self.pid_prev_error = 0.0;
    }

    pub fn dimensional_breakdown(&self) -> [(String, f64); 9] {
        let m = &self.evolution.metrics;
        let w = &self.config.weights;
        [
            ("phi".into(), m.phi * w[0]),
            ("global_workspace".into(), m.global_workspace * w[1]),
            ("coherence".into(), m.coherence * w[2]),
            ("attention_focus".into(), m.attention_focus * w[3]),
            ("reflexivity".into(), m.reflexivity * w[4]),
            ("emotional_intensity".into(), m.emotional_intensity * w[5]),
            (
                "knowledge_integration".into(),
                m.knowledge_integration * w[6],
            ),
            ("meta_accuracy".into(), m.meta_accuracy * w[7]),
            ("novelty_seeking".into(), m.novelty_seeking * w[8]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_c_score() {
        let mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        assert_eq!(mc.c_score(), 0.0);
    }

    #[test]
    fn test_compute_c_increases() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let metrics = ConsciousnessMetrics {
            phi: 0.9,
            global_workspace: 0.8,
            coherence: 0.7,
            attention_focus: 0.6,
            reflexivity: 0.5,
            emotional_intensity: 0.4,
            knowledge_integration: 0.7,
            meta_accuracy: 0.6,
            novelty_seeking: 0.5,
        };
        let c = mc.compute_c(metrics);
        assert!(c > 0.0);
        assert!(c <= 1.0);
    }

    #[test]
    fn test_is_conscious() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let high = ConsciousnessMetrics {
            phi: 1.0,
            global_workspace: 1.0,
            coherence: 1.0,
            attention_focus: 1.0,
            reflexivity: 1.0,
            emotional_intensity: 1.0,
            knowledge_integration: 1.0,
            meta_accuracy: 1.0,
            novelty_seeking: 1.0,
        };
        mc.compute_c(high);
        assert!(mc.is_conscious());
    }

    #[test]
    fn test_decay() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig {
            decay_rate: 0.5,
            ..Default::default()
        });
        let high = ConsciousnessMetrics {
            phi: 1.0,
            global_workspace: 1.0,
            coherence: 1.0,
            attention_focus: 1.0,
            reflexivity: 1.0,
            emotional_intensity: 1.0,
            knowledge_integration: 1.0,
            meta_accuracy: 1.0,
            novelty_seeking: 1.0,
        };
        mc.compute_c(high.clone());
        let after_first = mc.c_score();
        mc.compute_c(ConsciousnessMetrics::default());
        let after_decay = mc.c_score();
        assert!(after_decay < after_first);
    }

    #[test]
    fn test_peak_trough() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let low = ConsciousnessMetrics::default();
        let high = ConsciousnessMetrics {
            phi: 0.9,
            global_workspace: 0.8,
            coherence: 0.7,
            attention_focus: 0.6,
            reflexivity: 0.5,
            emotional_intensity: 0.4,
            knowledge_integration: 0.7,
            meta_accuracy: 0.6,
            novelty_seeking: 0.5,
        };
        mc.compute_c(ConsciousnessMetrics::default());
        mc.compute_c(high.clone());
        mc.compute_c(low);

        assert!(mc.evolution.c_peak > mc.evolution.c_trough);
        assert!(mc.evolution.c_peak > 0.0);
        assert!(mc.evolution.c_trough >= 0.0);
    }

    #[test]
    fn test_dimensional_breakdown() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let metrics = ConsciousnessMetrics {
            phi: 1.0,
            global_workspace: 0.5,
            coherence: 0.3,
            attention_focus: 0.7,
            reflexivity: 0.2,
            emotional_intensity: 0.8,
            knowledge_integration: 0.6,
            meta_accuracy: 0.4,
            novelty_seeking: 0.9,
        };
        mc.compute_c(metrics);
        let breakdown = mc.dimensional_breakdown();
        assert_eq!(breakdown.len(), 9);
        for (name, _) in &breakdown {
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_vsa_similarity_proxy() {
        let mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let a = vec![0u8; 64];
        let b = vec![0u8; 64];
        let c = {
            let mut v = vec![0u8; 64];
            v[0] = 1;
            v
        };
        let same = mc.compute_consciousness_with_vsa(&a, &b);
        let diff = mc.compute_consciousness_with_vsa(&a, &c);
        assert!(same > diff);
        assert!((same - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_trend_detection() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let high = ConsciousnessMetrics {
            phi: 1.0,
            global_workspace: 1.0,
            coherence: 1.0,
            attention_focus: 1.0,
            reflexivity: 1.0,
            emotional_intensity: 1.0,
            knowledge_integration: 1.0,
            meta_accuracy: 1.0,
            novelty_seeking: 1.0,
        };
        mc.compute_c(ConsciousnessMetrics::default());
        assert_eq!(mc.evolution.c_trend, "stable");
        mc.compute_c(high);
        assert_eq!(mc.evolution.c_trend, "rising");
        mc.compute_c(ConsciousnessMetrics::default());
        assert_eq!(mc.evolution.c_trend, "falling");
    }

    #[test]
    fn test_synergy_matrix_off_diagonal_positive() {
        let matrix = SynergyMatrix::new();
        assert_eq!(matrix.coupling[0][1], 0.05);
        assert_eq!(matrix.coupling[3][7], 0.05);
        assert_eq!(matrix.coupling[8][5], 0.05);
    }

    #[test]
    fn test_synergy_contributes_to_c() {
        // Config with synergy matrix zeroed out
        let mut config_no_synergy = MasterConsciousnessConfig::default();
        config_no_synergy.synergy_matrix = SynergyMatrix {
            coupling: [[0.0; 9]; 9],
        };
        let mut mc_no = MasterConsciousness::new(config_no_synergy);
        mc_no.configure_pid(0.3, 0.1, 0.05, 0.0);

        // Config with default synergy
        let mut mc_with = MasterConsciousness::new(MasterConsciousnessConfig::default());
        mc_with.configure_pid(0.3, 0.1, 0.05, 0.0);

        let metrics = ConsciousnessMetrics {
            phi: 0.7,
            global_workspace: 0.6,
            coherence: 0.6,
            attention_focus: 0.5,
            reflexivity: 0.4,
            emotional_intensity: 0.3,
            knowledge_integration: 0.6,
            meta_accuracy: 0.5,
            novelty_seeking: 0.4,
        };

        let c_no = mc_no.compute_c(metrics.clone());
        let c_with = mc_with.compute_c(metrics);
        assert!((c_with - c_no).abs() > 1e-6);
    }

    #[test]
    fn test_pid_prev_error_tracks() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        let metrics = ConsciousnessMetrics {
            phi: 0.6,
            ..Default::default()
        };
        mc.compute_c(metrics.clone());
        let error_after_first = mc.pid_prev_error;
        mc.compute_c(metrics);
        let error_after_second = mc.pid_prev_error;
        assert!((error_after_first - error_after_second).abs() > 1e-6);
    }

    #[test]
    fn test_configure_pid_updates_values() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig::default());
        mc.configure_pid(0.5, 0.2, 0.1, 0.8);
        let (kp, ki, kd, setpoint) = mc.pid_parameters();
        assert!((kp - 0.5).abs() < 1e-6);
        assert!((ki - 0.2).abs() < 1e-6);
        assert!((kd - 0.1).abs() < 1e-6);
        assert!((setpoint - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_synergy_compute_returns_positive_for_all_ones() {
        let matrix = SynergyMatrix::new();
        let metrics = ConsciousnessMetrics {
            phi: 1.0,
            global_workspace: 1.0,
            coherence: 1.0,
            attention_focus: 1.0,
            reflexivity: 1.0,
            emotional_intensity: 1.0,
            knowledge_integration: 1.0,
            meta_accuracy: 1.0,
            novelty_seeking: 1.0,
        };
        let weights = [1.0 / 9.0; 9];
        let synergy = matrix.compute_synergy(&metrics, &weights);
        assert!(synergy > 0.0);
    }

    #[test]
    fn test_pid_correction_clamped() {
        let mut mc = MasterConsciousness::new(MasterConsciousnessConfig {
            kp: 10.0,
            ki: 0.0,
            kd: 0.0,
            ..Default::default()
        });
        let metrics = ConsciousnessMetrics::default();
        mc.compute_c(metrics);
        let report = mc.evolution_report();
        assert!(report.pid_correction >= -0.5);
        assert!(report.pid_correction <= 0.5);
    }
}
