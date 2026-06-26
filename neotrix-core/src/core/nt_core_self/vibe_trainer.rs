use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct ConfigDimension {
    pub name: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
}

impl ConfigDimension {
    pub fn normalized(&self) -> f64 {
        if self.max <= self.min {
            return 0.0;
        }
        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub params: Vec<ConfigDimension>,
}

impl TrainingConfig {
    pub fn new() -> Self {
        TrainingConfig { params: Vec::new() }
    }

    pub fn add(&mut self, name: &str, value: f64, min: f64, max: f64) {
        self.params.push(ConfigDimension {
            name: name.to_string(),
            value,
            min,
            max,
        });
    }

    pub fn get(&self, name: &str) -> Option<&ConfigDimension> {
        self.params.iter().find(|p| p.name == name)
    }

    pub fn as_feature_vector(&self) -> Vec<f64> {
        self.params.iter().map(|p| p.normalized()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct TrainingOutcome {
    pub config_id: u64,
    pub final_loss: f64,
    pub final_accuracy: f64,
    pub convergence_step: u64,
    pub loss_curve_slope: f64,
    pub generalization_gap: f64,
}

#[derive(Debug, Clone)]
pub struct DynamicsPrediction {
    pub predicted_final_loss: f64,
    pub predicted_convergence_step: u64,
    pub predicted_slope: f64,
    pub confidence: f64,
}

impl DynamicsPrediction {
    pub fn is_optimistic(&self) -> bool {
        self.predicted_final_loss < 0.1
    }
}

#[derive(Debug, Clone)]
pub struct VibeTrainer {
    pub experience: Vec<(Vec<f64>, TrainingOutcome)>,
    pub max_experiences: usize,
    pub k_neighbors: usize,
    pub inverse_distance_weight: bool,
}

impl VibeTrainer {
    pub fn new() -> Self {
        VibeTrainer {
            experience: Vec::new(),
            max_experiences: 5000,
            k_neighbors: 5,
            inverse_distance_weight: true,
        }
    }

    pub fn record(&mut self, config: &TrainingConfig, outcome: TrainingOutcome) {
        let fv = config.as_feature_vector();
        if self.experience.len() >= self.max_experiences {
            self.experience.remove(0);
        }
        self.experience.push((fv, outcome));
    }

    pub fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f64>()
            .sqrt()
    }

    pub fn nearest_neighbors(&self, query: &[f64], k: usize) -> Vec<(usize, f64)> {
        if self.experience.is_empty() || k == 0 {
            return Vec::new();
        }
        let mut heap: BinaryHeap<(DistanceOrd, usize)> = BinaryHeap::new();
        for (i, (fv, _)) in self.experience.iter().enumerate() {
            let d = self.distance(query, fv);
            if heap.len() < k {
                heap.push((DistanceOrd(d), i));
            } else if d < heap.peek().unwrap().0 .0 {
                heap.pop();
                heap.push((DistanceOrd(d), i));
            }
        }
        let mut result: Vec<(usize, f64)> = heap.into_iter().map(|(d, i)| (i, d.0)).collect();
        result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn predict(&self, config: &TrainingConfig) -> DynamicsPrediction {
        let fv = config.as_feature_vector();
        let nns = self.nearest_neighbors(&fv, self.k_neighbors);
        if nns.is_empty() {
            return DynamicsPrediction {
                predicted_final_loss: 0.0,
                predicted_convergence_step: 0,
                predicted_slope: 0.0,
                confidence: 0.0,
            };
        }
        let total_dist: f64 = nns.iter().map(|(_, d)| d).sum();
        let avg_dist = total_dist / nns.len() as f64;
        let confidence = 1.0 / (1.0 + avg_dist);
        let weights: Vec<f64> = if self.inverse_distance_weight {
            nns.iter()
                .map(|(_, d)| if *d < 1e-12 { 1e12 } else { 1.0 / d })
                .collect()
        } else {
            nns.iter().map(|_| 1.0).collect()
        };
        let wsum: f64 = weights.iter().sum();
        if wsum <= 0.0 {
            return DynamicsPrediction {
                predicted_final_loss: 0.0,
                predicted_convergence_step: 0,
                predicted_slope: 0.0,
                confidence,
            };
        }
        let pred_loss: f64 = nns
            .iter()
            .zip(weights.iter())
            .map(|((i, _), w)| self.experience[*i].1.final_loss * w)
            .sum::<f64>()
            / wsum;
        let pred_step: f64 = nns
            .iter()
            .zip(weights.iter())
            .map(|((i, _), w)| self.experience[*i].1.convergence_step as f64 * w)
            .sum::<f64>()
            / wsum;
        let pred_slope: f64 = nns
            .iter()
            .zip(weights.iter())
            .map(|((i, _), w)| self.experience[*i].1.loss_curve_slope * w)
            .sum::<f64>()
            / wsum;
        DynamicsPrediction {
            predicted_final_loss: pred_loss,
            predicted_convergence_step: pred_step.round() as u64,
            predicted_slope: pred_slope,
            confidence,
        }
    }

    pub fn predict_loss_curve(&self, config: &TrainingConfig, steps: usize) -> Vec<f64> {
        let fv = config.as_feature_vector();
        let nns = self.nearest_neighbors(&fv, self.k_neighbors);
        if nns.is_empty() || steps == 0 {
            return Vec::new();
        }
        let weights: Vec<f64> = if self.inverse_distance_weight {
            nns.iter()
                .map(|(_, d)| if *d < 1e-12 { 1e12 } else { 1.0 / d })
                .collect()
        } else {
            nns.iter().map(|_| 1.0).collect()
        };
        let wsum: f64 = weights.iter().sum();
        if wsum <= 0.0 {
            return vec![0.0; steps];
        }
        let avg_slope: f64 = nns
            .iter()
            .zip(weights.iter())
            .map(|((i, _), w)| self.experience[*i].1.loss_curve_slope * w)
            .sum::<f64>()
            / wsum;
        let avg_final: f64 = nns
            .iter()
            .zip(weights.iter())
            .map(|((i, _), w)| self.experience[*i].1.final_loss * w)
            .sum::<f64>()
            / wsum;
        let avg_conv: f64 = nns
            .iter()
            .zip(weights.iter())
            .map(|((i, _), w)| self.experience[*i].1.convergence_step as f64 * w)
            .sum::<f64>()
            / wsum;
        let mut curve = Vec::with_capacity(steps);
        for t in 0..steps {
            let t_norm = t as f64 / steps.max(1) as f64;
            let decay = (-avg_slope * t_norm * 10.0).exp();
            let loss = avg_final + (1.0 - avg_final) * decay;
            curve.push(loss);
        }
        if avg_conv > 0.0 && avg_conv < steps as f64 {
            let conv_idx = avg_conv as usize;
            for t in conv_idx..steps {
                curve[t] = curve[t].min(avg_final * 1.05);
            }
        }
        curve
    }

    pub fn prediction_error(&self, config: &TrainingConfig, actual: &TrainingOutcome) -> f64 {
        let pred = self.predict(config);
        let d_loss = pred.predicted_final_loss - actual.final_loss;
        let d_step = pred.predicted_convergence_step as f64 - actual.convergence_step as f64;
        let d_slope = pred.predicted_slope - actual.loss_curve_slope;
        (d_loss * d_loss + d_step * d_step + d_slope * d_slope).sqrt()
    }

    pub fn uncertainty(&self, config: &TrainingConfig) -> f64 {
        let fv = config.as_feature_vector();
        let nns = self.nearest_neighbors(&fv, self.k_neighbors);
        if nns.is_empty() {
            return f64::MAX;
        }
        nns.iter().map(|(_, d)| d).sum::<f64>() / nns.len() as f64
    }

    pub fn suggest_experiment(&self) -> Option<TrainingConfig> {
        if self.experience.is_empty() {
            let mut cfg = TrainingConfig::new();
            cfg.add("learning_rate", 0.5, 0.0, 1.0);
            cfg.add("batch_size", 0.5, 0.0, 1.0);
            return Some(cfg);
        }
        let dim = self.experience[0].0.len();
        if dim == 0 {
            return None;
        }
        let mut best_score = -1.0f64;
        let mut best_cfg = None;
        for _ in 0..100 {
            let mut cfg = TrainingConfig::new();
            let mut fv = Vec::with_capacity(dim);
            for d in 0..dim {
                let val: f64 = fastrand();
                cfg.add(&format!("param_{}", d), val, 0.0, 1.0);
                fv.push(val);
            }
            let min_dist = self
                .experience
                .iter()
                .map(|(efv, _)| self.distance(&fv, efv))
                .fold(f64::MAX, f64::min);
            if min_dist > best_score {
                best_score = min_dist;
                best_cfg = Some(cfg);
            }
        }
        best_cfg
    }

    pub fn info_gain(&self, config: &TrainingConfig) -> f64 {
        let before = self.uncertainty(config);
        if before <= 0.0 {
            return 0.0;
        }
        let fv = config.as_feature_vector();
        let nns = self.nearest_neighbors(&fv, self.k_neighbors);
        let min_new_dist: f64 = if nns.is_empty() {
            0.0
        } else {
            nns.iter().map(|(_, d)| d).fold(f64::MAX, |a, &b| a.min(b))
        };
        let after = if nns.is_empty() {
            0.0
        } else {
            let sum_d: f64 = nns.iter().map(|(_, d)| d).sum::<f64>() + min_new_dist;
            let count = nns.len() as f64 + 1.0;
            sum_d / count
        };
        let gain = before - after;
        if gain < 0.0 {
            0.0
        } else {
            gain
        }
    }

    pub fn reset(&mut self) {
        self.experience.clear();
    }
}

pub struct VibeReport {
    pub num_experiences: usize,
    pub avg_prediction_error: f64,
    pub most_uncertain_config: Option<String>,
    pub suggested_experiment: Option<TrainingConfig>,
}

impl VibeReport {
    pub fn generate(&self, trainer: &VibeTrainer) -> String {
        let mut report = String::new();
        report.push_str(&format!("Vibe Training Report\n"));
        report.push_str(&format!("====================\n"));
        report.push_str(&format!("Experiences: {}\n", self.num_experiences));
        report.push_str(&format!(
            "Avg Prediction Error: {:.6}\n",
            self.avg_prediction_error
        ));
        if let Some(ref name) = self.most_uncertain_config {
            report.push_str(&format!("Most Uncertain Config: {}\n", name));
        }
        if let Some(ref cfg) = self.suggested_experiment {
            report.push_str("Suggested Experiment:\n");
            for p in &cfg.params {
                report.push_str(&format!(
                    "  {} = {:.4} (norm: {:.4})\n",
                    p.name,
                    p.value,
                    p.normalized()
                ));
            }
        }
        report.push_str(&format!(
            "Current uncertainty: {:.6}\n",
            trainer.uncertainty(&self.suggested_experiment.clone().unwrap_or_else(|| {
                let mut c = TrainingConfig::new();
                c.add("default", 0.5, 0.0, 1.0);
                c
            }))
        ));
        report
    }
}

#[derive(Debug, Clone, PartialEq)]
struct DistanceOrd(f64);

impl Eq for DistanceOrd {}

impl PartialOrd for DistanceOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for DistanceOrd {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn fastrand() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as f64;
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as f64;
    let seed = (nanos * 7919.0 + micros * 104729.0).fract().abs();
    seed.fract()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(values: &[f64]) -> TrainingConfig {
        let mut c = TrainingConfig::new();
        for (i, &v) in values.iter().enumerate() {
            c.add(&format!("p{}", i), v, 0.0, 1.0);
        }
        c
    }

    fn make_outcome(config_id: u64, loss: f64, step: u64, slope: f64) -> TrainingOutcome {
        TrainingOutcome {
            config_id,
            final_loss: loss,
            final_accuracy: 1.0 - loss,
            convergence_step: step,
            loss_curve_slope: slope,
            generalization_gap: loss * 0.1,
        }
    }

    #[test]
    fn test_record_and_predict() {
        let mut trainer = VibeTrainer::new();
        let c1 = make_config(&[0.1, 0.2]);
        let o1 = make_outcome(1, 0.05, 100, -0.5);
        let c2 = make_config(&[0.9, 0.8]);
        let o2 = make_outcome(2, 0.50, 500, -0.1);
        trainer.record(&c1, o1);
        trainer.record(&c2, o2);
        let near = make_config(&[0.12, 0.21]);
        let pred = trainer.predict(&near);
        assert!(pred.predicted_final_loss < 0.3);
        assert!(pred.confidence > 0.0);
    }

    #[test]
    fn test_nearest_neighbors_ordering() {
        let mut trainer = VibeTrainer::new();
        for i in 0..10 {
            let c = make_config(&[i as f64 / 10.0]);
            trainer.record(&c, make_outcome(i as u64, 0.1, 100, -0.5));
        }
        let query = make_config(&[0.0]);
        let nns = trainer.nearest_neighbors(&query.as_feature_vector(), 3);
        assert_eq!(nns.len(), 3);
        assert!(nns[0].1 <= nns[1].1);
        assert!(nns[1].1 <= nns[2].1);
        assert_eq!(nns[0].0, 0);
    }

    #[test]
    fn test_distance_correctness() {
        let trainer = VibeTrainer::new();
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let d = trainer.distance(&a, &b);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_uncertainty_increases_with_novelty() {
        let mut trainer = VibeTrainer::new();
        for i in 0..10 {
            let c = make_config(&[i as f64 / 10.0]);
            trainer.record(&c, make_outcome(i as u64, 0.1, 100, -0.5));
        }
        let known = make_config(&[0.5]);
        let novel = make_config(&[2.0]);
        let u_known = trainer.uncertainty(&known);
        let u_novel = trainer.uncertainty(&novel);
        assert!(u_novel > u_known);
    }

    #[test]
    fn test_suggest_experiment_explores_sparse() {
        let mut trainer = VibeTrainer::new();
        for i in 0..5 {
            let c = make_config(&[0.1 + i as f64 * 0.02]);
            trainer.record(&c, make_outcome(i as u64, 0.1, 100, -0.5));
        }
        let suggestion = trainer.suggest_experiment();
        assert!(suggestion.is_some());
        let cfg = suggestion.unwrap();
        let fv = cfg.as_feature_vector();
        let min_d = trainer
            .experience
            .iter()
            .map(|(efv, _)| trainer.distance(&fv, efv))
            .fold(f64::MAX, f64::min);
        assert!(min_d > 0.01);
    }

    #[test]
    fn test_info_gain() {
        let mut trainer = VibeTrainer::new();
        for i in 0..5 {
            let c = make_config(&[i as f64 / 5.0]);
            trainer.record(&c, make_outcome(i as u64, 0.1, 100, -0.5));
        }
        let known = make_config(&[0.2]);
        let novel = make_config(&[10.0]);
        let ig_known = trainer.info_gain(&known);
        let ig_novel = trainer.info_gain(&novel);
        assert!(ig_novel >= ig_known - 1e-6);
    }

    #[test]
    fn test_prediction_error() {
        let mut trainer = VibeTrainer::new();
        let c = make_config(&[0.5]);
        let o = make_outcome(1, 0.2, 200, -0.3);
        trainer.record(&c, o.clone());
        let err = trainer.prediction_error(&c, &o);
        assert!(err < 1e-6);
    }

    #[test]
    fn test_loss_curve_interpolation() {
        let mut trainer = VibeTrainer::new();
        for i in 0..3 {
            let c = make_config(&[i as f64 / 3.0]);
            trainer.record(
                &c,
                make_outcome(i as u64, 0.1 + i as f64 * 0.1, 100 + i as u64 * 50, -0.5),
            );
        }
        let c = make_config(&[0.5]);
        let curve = trainer.predict_loss_curve(&c, 10);
        assert_eq!(curve.len(), 10);
        for v in &curve {
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn test_empty_trainer_edge_case() {
        let trainer = VibeTrainer::new();
        let c = make_config(&[0.5]);
        let pred = trainer.predict(&c);
        assert_eq!(pred.confidence, 0.0);
        assert!(!pred.is_optimistic());
        let nns = trainer.nearest_neighbors(&c.as_feature_vector(), 3);
        assert!(nns.is_empty());
        let unc = trainer.uncertainty(&c);
        assert_eq!(unc, f64::MAX);
        let curve = trainer.predict_loss_curve(&c, 5);
        assert!(curve.is_empty());
        assert!(trainer.suggest_experiment().is_some());
    }

    #[test]
    fn test_single_experience_edge_case() {
        let mut trainer = VibeTrainer::new();
        let c = make_config(&[0.5]);
        let o = make_outcome(1, 0.2, 200, -0.4);
        trainer.record(&c, o);
        let pred = trainer.predict(&c);
        assert!((pred.predicted_final_loss - 0.2).abs() < 1e-6);
        assert_eq!(pred.predicted_convergence_step, 200);
        assert!(pred.confidence > 0.0);
        let curve = trainer.predict_loss_curve(&c, 5);
        assert_eq!(curve.len(), 5);
    }

    #[test]
    fn test_normalized_clamping() {
        let d = ConfigDimension {
            name: "test".into(),
            value: 1.5,
            min: 0.0,
            max: 1.0,
        };
        assert!((d.normalized() - 1.0).abs() < 1e-6);
        let d2 = ConfigDimension {
            name: "test".into(),
            value: -0.5,
            min: 0.0,
            max: 1.0,
        };
        assert!((d2.normalized() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset_clears_experience() {
        let mut trainer = VibeTrainer::new();
        let c = make_config(&[0.5]);
        trainer.record(&c, make_outcome(1, 0.2, 100, -0.5));
        assert_eq!(trainer.experience.len(), 1);
        trainer.reset();
        assert_eq!(trainer.experience.len(), 0);
    }

    #[test]
    fn test_as_feature_vector() {
        let mut c = TrainingConfig::new();
        c.add("lr", 0.5, 0.0, 1.0);
        c.add("wd", 0.25, 0.0, 0.5);
        let fv = c.as_feature_vector();
        assert_eq!(fv.len(), 2);
        assert!((fv[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_get_returns_correct_dimension() {
        let mut c = TrainingConfig::new();
        c.add("lr", 0.1, 0.0, 1.0);
        let d = c.get("lr").unwrap();
        assert!((d.value - 0.1).abs() < 1e-6);
        assert!(c.get("nonexistent").is_none());
    }
}
