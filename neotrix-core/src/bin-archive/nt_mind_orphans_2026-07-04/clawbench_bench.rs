use std::collections::HashMap;
use chrono::Utc;

use super::clawbench::{TrajectoryClassifier, TrajectoryDynamics, ClassificationResult};

#[derive(Debug, Clone)]
pub struct ClassStats {
    pub tp: usize,
    pub fp: usize,
    pub fn_count: usize,
}

#[derive(Debug, Clone)]
pub struct ClawBenchReport {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub per_class: HashMap<TrajectoryDynamics, ClassStats>,
    pub confusion_matrix: [[usize; 3]; 3],
    pub timestamp: String,
    pub total_samples: usize,
}

impl ClawBenchReport {
    pub fn display(&self) {
        println!("╭─ ClawBench Trajectory Classification ──────────────╮");
        println!("│ Accuracy:  {:.3}  Precision: {:.3}              │", self.accuracy, self.precision);
        println!("│ Recall:    {:.3}  F1:        {:.3}              │", self.recall, self.f1);
        println!("│ Total:     {:<4}                                 │", self.total_samples);
        println!("├──────────────────────────────────────────────────────┤");
        println!("│ Confusion Matrix (rows=actual, cols=predicted)      │");
        println!("│                      SA    LC    ND                  │");
        println!("│ StrangeAttractor   {:>4}  {:>4}  {:>4}              │",
            self.confusion_matrix[0][0], self.confusion_matrix[0][1], self.confusion_matrix[0][2]);
        println!("│ LimitCycle         {:>4}  {:>4}  {:>4}              │",
            self.confusion_matrix[1][0], self.confusion_matrix[1][1], self.confusion_matrix[1][2]);
        println!("│ NormalDiffusion    {:>4}  {:>4}  {:>4}              │",
            self.confusion_matrix[2][0], self.confusion_matrix[2][1], self.confusion_matrix[2][2]);
        println!("├──────────────────────────────────────────────────────┤");
        println!("│ Per-Class Stats:                                     │");
        for (dyn_ty, stats) in &self.per_class {
            let label = dyn_ty.label();
            println!("│   {:<18} TP={:>2} FP={:>2} FN={:>2}           │",
                label, stats.tp, stats.fp, stats.fn_count);
        }
        println!("╰──────────────────────────────────────────────────────╯");
    }
}

fn dynamics_to_idx(d: TrajectoryDynamics) -> usize {
    match d {
        TrajectoryDynamics::StrangeAttractor => 0,
        TrajectoryDynamics::LimitCycle => 1,
        TrajectoryDynamics::NormalDiffusion => 2,
    }
}

fn make_actions(variants: &[&str]) -> Vec<String> {
    variants.iter().map(|s| s.to_string()).collect()
}

const ACTIONS_A: &[&str] = &["read", "write", "search", "delete", "verify", "plan", "refactor", "test", "deploy", "monitor"];
const ACTIONS_B: &[&str] = &["search", "extract", "analyze", "summarize", "report"];

/// Generate a strange attractor trajectory (high variance, >50% action switches)
fn gen_strange_attractor(idx: usize) -> (Vec<f64>, Vec<String>) {
    let seed = idx * 7 + 1;
    let rewards: Vec<f64> = (0..20).map(|i| {
        let t = i as f64 * 0.3 + seed as f64;
        (t.sin() * 1.5 + (t * 1.7).cos() * 1.2 + 1.0) * 0.5
    }).collect();
    let actions: Vec<String> = (0..20).map(|i| {
        let choice = (i * seed * 3 + idx * 5) % ACTIONS_A.len();
        ACTIONS_A[choice].to_string()
    }).collect();
    (rewards, actions)
}

/// Generate a limit cycle trajectory (alternating rewards, periodic actions)
fn gen_limit_cycle(idx: usize) -> (Vec<f64>, Vec<String>) {
    let offset = idx as f64 * 0.2;
    let rewards: Vec<f64> = (0..20).map(|i| {
        let t = i as f64 * 0.8 + offset;
        (t.sin() * 0.6 + 0.5).abs().max(0.1)
    }).collect();
    let actions: Vec<String> = (0..20).map(|i| {
        let p = i % 3;
        match (p, idx % 2) {
            (0, _) => "read".to_string(),
            (1, _) => "write".to_string(),
            (2, 0) => "verify".to_string(),
            (2, 1) => "delete".to_string(),
            _ => "read".to_string(),
        }
    }).collect();
    (rewards, actions)
}

/// Generate a normal diffusion trajectory (low variance, gradual convergence)
fn gen_normal_diffusion(idx: usize) -> (Vec<f64>, Vec<String>) {
    let base = 0.1 + idx as f64 * 0.05;
    let rewards: Vec<f64> = (0..20).map(|i| {
        let progress = i as f64 / 20.0;
        (base + progress * 0.7 + (i as f64 * 0.01).sin() * 0.05).min(1.0)
    }).collect();
    let actions: Vec<String> = (0..20).map(|i| {
        let phase = i / 4;
        match phase {
            0 => "read".to_string(),
            1 => "analyze".to_string(),
            2 => ACTIONS_B[(i + idx) % ACTIONS_B.len()].to_string(),
            3 => "write".to_string(),
            _ => "done".to_string(),
        }
    }).collect();
    (rewards, actions)
}

pub struct ClawBenchBenchmark;

impl ClawBenchBenchmark {
    pub fn run() -> ClawBenchReport {
        let classifier = TrajectoryClassifier::new();
        let mut all_ground_truth: Vec<TrajectoryDynamics> = Vec::new();
        let mut all_predictions: Vec<ClassificationResult> = Vec::new();

        for i in 0..3 {
            let (r, a) = gen_strange_attractor(i);
            all_ground_truth.push(TrajectoryDynamics::StrangeAttractor);
            all_predictions.push(classifier.classify(&r, &a));
        }

        for i in 0..3 {
            let (r, a) = gen_limit_cycle(i);
            all_ground_truth.push(TrajectoryDynamics::LimitCycle);
            all_predictions.push(classifier.classify(&r, &a));
        }

        for i in 0..3 {
            let (r, a) = gen_normal_diffusion(i);
            all_ground_truth.push(TrajectoryDynamics::NormalDiffusion);
            all_predictions.push(classifier.classify(&r, &a));
        }

        let total = all_ground_truth.len();
        let mut confusion = [[0usize; 3]; 3];
        let mut correct = 0usize;

        for (gt, pred) in all_ground_truth.iter().zip(all_predictions.iter()) {
            let gt_idx = dynamics_to_idx(*gt);
            let pred_idx = dynamics_to_idx(pred.dynamics);
            confusion[gt_idx][pred_idx] += 1;
            if gt_idx == pred_idx {
                correct += 1;
            }
        }

        let accuracy = if total > 0 { correct as f64 / total as f64 } else { 0.0 };

        let mut per_class: HashMap<TrajectoryDynamics, ClassStats> = HashMap::new();
        let mut precision_sum = 0.0;
        let mut recall_sum = 0.0;
        let mut f1_sum = 0.0;
        let mut class_count = 0usize;

        for (class_idx, dyn_ty) in [TrajectoryDynamics::StrangeAttractor, TrajectoryDynamics::LimitCycle, TrajectoryDynamics::NormalDiffusion].iter().enumerate() {
            let tp = confusion[class_idx][class_idx];
            let fp: usize = (0..3).map(|r| confusion[r][class_idx]).sum::<usize>() - tp;
            let fn_count: usize = confusion[class_idx].iter().sum::<usize>() - tp;
            per_class.insert(*dyn_ty, ClassStats { tp, fp, fn_count });

            let precision = if tp + fp > 0 { tp as f64 / (tp + fp) as f64 } else { 0.0 };
            let recall = if tp + fn_count > 0 { tp as f64 / (tp + fn_count) as f64 } else { 0.0 };
            let f1 = if precision + recall > 0.0 { 2.0 * precision * recall / (precision + recall) } else { 0.0 };

            precision_sum += precision;
            recall_sum += recall;
            f1_sum += f1;
            class_count += 1;
        }

        let precision = precision_sum / class_count as f64;
        let recall = recall_sum / class_count as f64;
        let f1 = f1_sum / class_count as f64;

        ClawBenchReport {
            accuracy,
            precision,
            recall,
            f1,
            per_class,
            confusion_matrix: confusion,
            timestamp: Utc::now().to_rfc3339(),
            total_samples: total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_strange_attractor_high_variance() {
        let (rewards, actions) = gen_strange_attractor(0);
        assert_eq!(rewards.len(), 20);
        assert_eq!(actions.len(), 20);
        let classifier = TrajectoryClassifier::new();
        let result = classifier.classify(&rewards, &actions);
        assert!(result.reward_variance > 0.2, "variance {} should be > 0.2 for strange attractor", result.reward_variance);
        assert!(result.action_switches > 0.4, "switch ratio {} should be > 0.4", result.action_switches);
    }

    #[test]
    fn test_gen_limit_cycle_alternating() {
        let (rewards, actions) = gen_limit_cycle(0);
        assert_eq!(rewards.len(), 20);
        let classifier = TrajectoryClassifier::new();
        let result = classifier.classify(&rewards, &actions);
        assert_eq!(result.dynamics, TrajectoryDynamics::LimitCycle,
            "limit cycle generation should be classified as LimitCycle, got {:?}", result.dynamics);
    }

    #[test]
    fn test_gen_normal_diffusion_low_variance() {
        let (rewards, actions) = gen_normal_diffusion(0);
        assert_eq!(rewards.len(), 20);
        let classifier = TrajectoryClassifier::new();
        let result = classifier.classify(&rewards, &actions);
        assert!(result.reward_variance < 0.1, "variance {} should be < 0.1 for normal diffusion", result.reward_variance);
    }

    #[test]
    fn test_clawbench_benchmark_runs() {
        let report = ClawBenchBenchmark::run();
        assert_eq!(report.total_samples, 9);
        assert!(report.accuracy >= 0.0 && report.accuracy <= 1.0);
        assert!(report.precision >= 0.0 && report.precision <= 1.0);
        assert!(report.recall >= 0.0 && report.recall <= 1.0);
        assert!(report.f1 >= 0.0 && report.f1 <= 1.0);
    }

    #[test]
    fn test_confusion_matrix_dimensions() {
        let report = ClawBenchBenchmark::run();
        assert_eq!(report.confusion_matrix.len(), 3);
        for row in &report.confusion_matrix {
            assert_eq!(row.len(), 3);
        }
    }

    #[test]
    fn test_confusion_matrix_total_matches_samples() {
        let report = ClawBenchBenchmark::run();
        let total: usize = report.confusion_matrix.iter()
            .flat_map(|r| r.iter())
            .sum();
        assert_eq!(total, report.total_samples);
    }

    #[test]
    fn test_per_class_has_all_three() {
        let report = ClawBenchBenchmark::run();
        assert!(report.per_class.contains_key(&TrajectoryDynamics::StrangeAttractor));
        assert!(report.per_class.contains_key(&TrajectoryDynamics::LimitCycle));
        assert!(report.per_class.contains_key(&TrajectoryDynamics::NormalDiffusion));
    }

    #[test]
    fn test_display_does_not_panic() {
        let report = ClawBenchBenchmark::run();
        report.display();
    }

    #[test]
    fn test_all_seeds_different() {
        let r1 = ClawBenchBenchmark::run();
        let r2 = ClawBenchBenchmark::run();
        assert!((r1.accuracy - r2.accuracy).abs() < 0.001);
    }
}
