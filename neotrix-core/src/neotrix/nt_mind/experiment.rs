use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MAX_HYPOTHESES: usize = 1000;
const MAX_DESIGNS: usize = 500;
const MAX_RESULTS: usize = 1000;
const MAX_ACTIVE: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub statement: String,
    pub null_hypothesis: String,
    pub metrics: Vec<String>,
    pub expected_effect: f64,
    pub confidence_level: f64,
    pub power: f64,
}

impl Hypothesis {
    pub fn new(id: &str, statement: &str, metrics: Vec<String>, expected_effect: f64) -> Self {
        Self {
            id: id.to_string(),
            statement: statement.to_string(),
            null_hypothesis: format!("The null hypothesis: {}", statement),
            metrics,
            expected_effect,
            confidence_level: 0.95,
            power: 0.80,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestDesign {
    pub hypothesis_id: String,
    pub control_description: String,
    pub treatment_description: String,
    pub min_sample_size: u64,
    pub duration_days: u64,
}

impl ABTestDesign {
    pub fn new(
        hypothesis_id: &str,
        control: &str,
        treatment: &str,
        sample_size: u64,
        days: u64,
    ) -> Self {
        Self {
            hypothesis_id: hypothesis_id.to_string(),
            control_description: control.to_string(),
            treatment_description: treatment.to_string(),
            min_sample_size: sample_size,
            duration_days: days,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub hypothesis_id: String,
    pub p_value: f64,
    pub effect_size: f64,
    pub significant: bool,
    pub control_mean: f64,
    pub treatment_mean: f64,
    pub sample_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRegistry {
    pub hypotheses: HashMap<String, Hypothesis>,
    pub designs: HashMap<String, ABTestDesign>,
    pub results: HashMap<String, ExperimentResult>,
    pub active: Vec<String>,
}

impl Default for ExperimentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExperimentRegistry {
    pub fn new() -> Self {
        Self {
            hypotheses: HashMap::new(),
            designs: HashMap::new(),
            results: HashMap::new(),
            active: Vec::new(),
        }
    }

    pub fn register_hypothesis(&mut self, h: Hypothesis) {
        self.hypotheses.insert(h.id.clone(), h);
        if self.hypotheses.len() > MAX_HYPOTHESES {
            let to_remove: Vec<_> = self
                .hypotheses
                .keys()
                .take(MAX_HYPOTHESES / 5)
                .cloned()
                .collect();
            for k in to_remove {
                self.hypotheses.remove(&k);
            }
        }
    }

    pub fn register_design(&mut self, d: ABTestDesign) {
        self.designs.insert(d.hypothesis_id.clone(), d);
        if self.designs.len() > MAX_DESIGNS {
            let to_remove: Vec<_> = self.designs.keys().take(MAX_DESIGNS / 5).cloned().collect();
            for k in to_remove {
                self.designs.remove(&k);
            }
        }
    }

    pub fn record_result(&mut self, r: ExperimentResult) {
        let h_id = r.hypothesis_id.clone();
        self.results.insert(h_id.clone(), r);
        self.active.retain(|id| id != &h_id);
        if self.results.len() > MAX_RESULTS {
            let to_remove: Vec<_> = self.results.keys().take(MAX_RESULTS / 5).cloned().collect();
            for k in to_remove {
                self.results.remove(&k);
            }
        }
    }

    pub fn mark_active(&mut self, hypothesis_id: &str) {
        if !self.active.contains(&hypothesis_id.to_string()) {
            self.active.push(hypothesis_id.to_string());
        }
        while self.active.len() > MAX_ACTIVE {
            self.active.remove(0);
        }
    }
}

pub struct ExperimentDesigner;

impl ExperimentDesigner {
    pub fn design_ab_test(hypothesis: &Hypothesis) -> ABTestDesign {
        let sample_size = Self::estimate_sample_size(
            hypothesis.expected_effect,
            hypothesis.confidence_level,
            hypothesis.power,
        );
        ABTestDesign {
            hypothesis_id: hypothesis.id.clone(),
            control_description: "Current state (no change)".to_string(),
            treatment_description: format!("Apply: {}", hypothesis.statement),
            min_sample_size: sample_size,
            duration_days: 14,
        }
    }

    pub fn estimate_sample_size(effect_size: f64, alpha: f64, power: f64) -> u64 {
        if effect_size <= 0.0 {
            return 1000;
        }
        let z_alpha: f64 = match alpha {
            a if a <= 0.001 => 3.29,
            a if a <= 0.01 => 2.58,
            a if a <= 0.05 => 1.96,
            _ => 1.64,
        };
        let z_beta: f64 = match power {
            p if p >= 0.99 => 2.33,
            p if p >= 0.95 => 1.64,
            p if p >= 0.90 => 1.28,
            p if p >= 0.80 => 0.84,
            _ => 0.67,
        };
        let n = (z_alpha + z_beta).powi(2) / (effect_size * effect_size);
        (n.ceil() as u64).max(30)
    }

    pub fn analyze_results(
        control: &[f64],
        treatment: &[f64],
        hypothesis_id: &str,
    ) -> ExperimentResult {
        let n_control = control.len() as f64;
        let n_treatment = treatment.len() as f64;
        let total_n = (n_control + n_treatment) as u64;

        if control.is_empty() || treatment.is_empty() {
            return ExperimentResult {
                hypothesis_id: hypothesis_id.to_string(),
                p_value: 1.0,
                effect_size: 0.0,
                significant: false,
                control_mean: control.iter().copied().sum::<f64>() / n_control.max(1.0),
                treatment_mean: treatment.iter().copied().sum::<f64>() / n_treatment.max(1.0),
                sample_size: total_n,
            };
        }

        let mean_c = control.iter().sum::<f64>() / n_control;
        let mean_t = treatment.iter().sum::<f64>() / n_treatment;

        let var_c =
            control.iter().map(|v| (v - mean_c).powi(2)).sum::<f64>() / (n_control - 1.0).max(1.0);
        let var_t = treatment.iter().map(|v| (v - mean_t).powi(2)).sum::<f64>()
            / (n_treatment - 1.0).max(1.0);
        let pooled_se = ((var_c / n_control) + (var_t / n_treatment)).sqrt();

        let effect_size = (mean_t - mean_c) / ((var_c + var_t) / 2.0).sqrt().max(0.001);

        let t_stat = (mean_t - mean_c) / pooled_se.max(0.001);
        let p_value = if t_stat.abs() > 3.29 {
            0.001
        } else if t_stat.abs() > 2.58 {
            0.01
        } else if t_stat.abs() > 1.96 {
            0.05
        } else {
            0.5
        };

        ExperimentResult {
            hypothesis_id: hypothesis_id.to_string(),
            p_value,
            effect_size,
            significant: p_value <= 0.05,
            control_mean: mean_c,
            treatment_mean: mean_t,
            sample_size: total_n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_creation() {
        let h = Hypothesis::new(
            "H-001",
            "Adding search will increase user engagement",
            vec![
                "daily_active_users".to_string(),
                "search_per_session".to_string(),
            ],
            0.3,
        );
        assert_eq!(h.id, "H-001");
        assert!((h.confidence_level - 0.95).abs() < 0.01);
        assert!((h.power - 0.80).abs() < 0.01);
        assert_eq!(h.metrics.len(), 2);
    }

    #[test]
    fn test_registry_roundtrip() {
        let mut reg = ExperimentRegistry::new();
        let h = Hypothesis::new("H-001", "Test", vec!["metric".to_string()], 0.5);
        reg.register_hypothesis(h);
        assert!(reg.hypotheses.contains_key("H-001"));
    }

    #[test]
    fn test_sample_size_estimation() {
        let n = ExperimentDesigner::estimate_sample_size(0.5, 0.05, 0.80);
        assert!(n >= 30);
        assert!(n < 500);

        let n_small = ExperimentDesigner::estimate_sample_size(0.8, 0.05, 0.80);
        assert!(n_small < n);
    }

    #[test]
    fn test_sample_size_large_effect() {
        let n = ExperimentDesigner::estimate_sample_size(1.0, 0.05, 0.80);
        assert!(n < 100);
    }

    #[test]
    fn test_ab_test_design_from_hypothesis() {
        let h = Hypothesis::new("H-002", "New feature", vec!["conversion".to_string()], 0.3);
        let design = ExperimentDesigner::design_ab_test(&h);
        assert_eq!(design.hypothesis_id, "H-002");
        assert!(design.min_sample_size > 0);
        assert_eq!(design.duration_days, 14);
    }

    #[test]
    fn test_analyze_results_significant() {
        let control = vec![
            1.0, 1.5, 1.2, 1.3, 1.1, 1.4, 1.0, 1.2, 1.3, 1.1, 1.2, 1.0, 1.3, 1.1, 1.4, 1.2, 1.0,
            1.3, 1.2, 1.1,
        ];
        let treatment = vec![
            5.0, 5.5, 5.2, 5.3, 5.1, 5.4, 5.0, 5.2, 5.3, 5.1, 5.2, 5.0, 5.3, 5.1, 5.4, 5.2, 5.0,
            5.3, 5.2, 5.1,
        ];
        let result = ExperimentDesigner::analyze_results(&control, &treatment, "H-003");
        assert!(result.significant);
        assert!(result.p_value <= 0.05);
        assert!(result.effect_size.abs() > 0.5);
        assert!(result.treatment_mean > result.control_mean);
    }

    #[test]
    fn test_analyze_results_not_significant() {
        let control = vec![1.0, 1.1, 1.0, 1.1, 1.0, 1.1, 1.0, 1.1, 1.0, 1.1];
        let treatment = vec![1.05, 1.15, 1.05, 1.05, 1.1, 1.0, 1.05, 1.1, 1.05, 1.0];
        let result = ExperimentDesigner::analyze_results(&control, &treatment, "H-004");
        assert!(!result.significant);
    }

    #[test]
    fn test_registry_mark_active() {
        let mut reg = ExperimentRegistry::new();
        reg.mark_active("H-001");
        assert_eq!(reg.active.len(), 1);
        reg.mark_active("H-001");
        assert_eq!(reg.active.len(), 1);
    }

    #[test]
    fn test_registry_record_result_clears_active() {
        let mut reg = ExperimentRegistry::new();
        let h = Hypothesis::new("H-005", "Test", vec!["m".to_string()], 0.5);
        reg.register_hypothesis(h);
        reg.mark_active("H-005");

        let result = ExperimentResult {
            hypothesis_id: "H-005".to_string(),
            p_value: 0.01,
            effect_size: 0.5,
            significant: true,
            control_mean: 1.0,
            treatment_mean: 1.5,
            sample_size: 100,
        };
        reg.record_result(result);
        assert!(reg.active.is_empty());
        assert!(reg.results.contains_key("H-005"));
    }
}
