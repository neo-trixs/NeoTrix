//! ClawBench Trajectory Diagnostics
//!
//! Classifies agent trajectories into 3 power system dynamics categories:
//! - StrangeAttractor: agent jumps between different approaches chaotically
//! - LimitCycle: agent oscillates between 2+ states without convergence
//! - NormalDiffusion: healthy exploration with eventual convergence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 3 种轨迹动力学类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrajectoryDynamics {
    /// 奇怪吸引子: 混沌行为, agent 在不同方法间跳跃
    StrangeAttractor,
    /// 极限环: agent 在 2+ 状态间振荡, 无法收敛
    LimitCycle,
    /// 正常扩散: 健康探索并最终收敛
    NormalDiffusion,
}

impl TrajectoryDynamics {
    pub fn label(&self) -> &'static str {
        match self {
            TrajectoryDynamics::StrangeAttractor => "StrangeAttractor",
            TrajectoryDynamics::LimitCycle => "LimitCycle",
            TrajectoryDynamics::NormalDiffusion => "NormalDiffusion",
        }
    }
}

/// 轨迹分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub dynamics: TrajectoryDynamics,
    pub confidence: f64,
    pub action_switches: f64,
    pub reward_variance: f64,
    pub trajectory_length: usize,
}

/// 轨迹分类器
#[derive(Debug, Clone)]
pub struct TrajectoryClassifier {
    pub action_switch_threshold: f64,
    pub reward_variance_threshold: f64,
    pub min_cycle_count: usize,
}

impl Default for TrajectoryClassifier {
    fn default() -> Self {
        Self {
            action_switch_threshold: 0.50,
            reward_variance_threshold: 0.30,
            min_cycle_count: 3,
        }
    }
}

impl TrajectoryClassifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// 分析奖励序列和动作序列, 分类轨迹类型
    pub fn classify(&self, rewards: &[f64], actions: &[String]) -> ClassificationResult {
        let trajectory_length = rewards.len().min(actions.len());
        if trajectory_length == 0 {
            return ClassificationResult {
                dynamics: TrajectoryDynamics::NormalDiffusion,
                confidence: 1.0,
                action_switches: 0.0,
                reward_variance: 0.0,
                trajectory_length: 0,
            };
        }

        let action_switches = self.compute_action_switch_ratio(actions);
        let reward_variance = self.compute_variance(rewards);
        let has_limit_cycle = self.detect_limit_cycle(rewards);

        let mut dynamics = TrajectoryDynamics::NormalDiffusion;
        let mut confidence = 0.5;

        if reward_variance > self.reward_variance_threshold
            && action_switches > self.action_switch_threshold
        {
            dynamics = TrajectoryDynamics::StrangeAttractor;
            confidence = (reward_variance + action_switches) / 2.0;
        } else if has_limit_cycle {
            dynamics = TrajectoryDynamics::LimitCycle;
            confidence = 0.7 + (reward_variance.min(0.5) / 0.5) * 0.2;
        }

        confidence = confidence.clamp(0.0, 1.0);

        ClassificationResult {
            dynamics,
            confidence,
            action_switches,
            reward_variance,
            trajectory_length,
        }
    }

    /// 计算动作切换比例 (相邻动作不同的比例)
    fn compute_action_switch_ratio(&self, actions: &[String]) -> f64 {
        if actions.len() < 2 {
            return 0.0;
        }
        let switches = actions.windows(2).filter(|w| w[0] != w[1]).count();
        switches as f64 / (actions.len() - 1) as f64
    }

    /// 计算奖励方差
    fn compute_variance(&self, rewards: &[f64]) -> f64 {
        if rewards.len() < 2 {
            return 0.0;
        }
        let mean = rewards.iter().sum::<f64>() / rewards.len() as f64;
        let variance = rewards.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rewards.len() as f64;
        variance
    }

    /// 检测极限环: 奖励交替上升/下降模式超过 min_cycle_count 次
    fn detect_limit_cycle(&self, rewards: &[f64]) -> bool {
        if rewards.len() < 4 {
            return false;
        }

        // 计算差分方向: +1 = 上升, -1 = 下降, 0 = 持平
        let directions: Vec<i32> = rewards.windows(2).map(|w| {
            if w[1] > w[0] { 1 } else if w[1] < w[0] { -1 } else { 0 }
        }).collect();

        if directions.len() < 3 {
            return false;
        }

        // 检测交替模式: +1, -1, +1, -1, ... 或 -1, +1, -1, +1, ...
        let mut cycles = 0;
        let mut i = 0;
        while i < directions.len() - 1 {
            if directions[i] != 0 && directions[i + 1] != 0 && directions[i] != directions[i + 1] {
                cycles += 1;
                i += 2;
            } else {
                i += 1;
            }
        }

        cycles >= self.min_cycle_count
    }
}

/// 聚合分析结果, 支持批量轨迹分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryAnalysisReport {
    pub total_trajectories: usize,
    pub dynamics_counts: HashMap<TrajectoryDynamics, usize>,
    pub avg_confidence: f64,
    pub avg_action_switches: f64,
    pub avg_reward_variance: f64,
}

impl TrajectoryAnalysisReport {
    pub fn new(results: &[ClassificationResult]) -> Self {
        let total = results.len();
        if total == 0 {
            return Self {
                total_trajectories: 0,
                dynamics_counts: HashMap::new(),
                avg_confidence: 0.0,
                avg_action_switches: 0.0,
                avg_reward_variance: 0.0,
            };
        }

        let mut counts = HashMap::new();
        let mut conf_sum = 0.0;
        let mut switch_sum = 0.0;
        let mut var_sum = 0.0;

        for r in results {
            *counts.entry(r.dynamics).or_insert(0) += 1;
            conf_sum += r.confidence;
            switch_sum += r.action_switches;
            var_sum += r.reward_variance;
        }

        Self {
            total_trajectories: total,
            dynamics_counts: counts,
            avg_confidence: conf_sum / total as f64,
            avg_action_switches: switch_sum / total as f64,
            avg_reward_variance: var_sum / total as f64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_actions(variants: &[&str]) -> Vec<String> {
        variants.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_normal_diffusion() {
        let classifier = TrajectoryClassifier::new();
        let rewards = vec![0.1, 0.3, 0.5, 0.6, 0.7, 0.75, 0.8, 0.82, 0.85, 0.9];
        let actions = make_actions(&["read", "read", "search", "read", "write", "write", "write", "verify", "verify", "done"]);
        let result = classifier.classify(&rewards, &actions);
        assert_eq!(result.dynamics, TrajectoryDynamics::NormalDiffusion);
        assert!(result.confidence >= 0.0);
    }

    #[test]
    fn test_strange_attractor() {
        let classifier = TrajectoryClassifier::new();
        // Extreme swings to get variance > 0.3 (0.0 and 1.0 → variance = 0.25, so use negative)
        let rewards = vec![-0.5, 1.0, -0.5, 1.0, -0.5, 1.0, -0.5, 1.0, -0.5, 1.0];
        let actions = make_actions(&["write", "delete", "search", "write", "delete", "search", "write", "delete", "search", "write"]);
        let result = classifier.classify(&rewards, &actions);
        assert_eq!(result.dynamics, TrajectoryDynamics::StrangeAttractor);
        assert!(result.reward_variance > 0.3);
        assert!(result.action_switches > 0.5);
    }

    #[test]
    fn test_limit_cycle() {
        let classifier = TrajectoryClassifier::new();
        // 交替上升/下降模式: +0.5, -0.3, +0.5, -0.3, ...
        let rewards = vec![0.2, 0.7, 0.4, 0.9, 0.6, 0.8, 0.5, 0.9, 0.6];
        let actions = make_actions(&["search", "search", "write", "write", "search", "search", "write", "write", "search"]);
        let result = classifier.classify(&rewards, &actions);
        assert_eq!(result.dynamics, TrajectoryDynamics::LimitCycle);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_empty_input() {
        let classifier = TrajectoryClassifier::new();
        let result = classifier.classify(&[], &[]);
        assert_eq!(result.dynamics, TrajectoryDynamics::NormalDiffusion);
        assert_eq!(result.trajectory_length, 0);
    }

    #[test]
    fn test_single_action() {
        let classifier = TrajectoryClassifier::new();
        let result = classifier.classify(&[0.5], &["done".into()]);
        assert_eq!(result.dynamics, TrajectoryDynamics::NormalDiffusion);
        assert_eq!(result.action_switches, 0.0);
        assert_eq!(result.reward_variance, 0.0);
    }

    #[test]
    fn test_action_switch_ratio() {
        let classifier = TrajectoryClassifier::new();
        let actions = vec!["a".into(), "b".into(), "a".into(), "b".into()];
        let ratio = classifier.compute_action_switch_ratio(&actions);
        assert!((ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_variance() {
        let classifier = TrajectoryClassifier::new();
        let rewards = vec![1.0, 1.0, 1.0, 1.0];
        assert!((classifier.compute_variance(&rewards)).abs() < 1e-6);
    }

    #[test]
    fn test_trajectory_analysis_report() {
        let results = vec![
            ClassificationResult {
                dynamics: TrajectoryDynamics::NormalDiffusion,
                confidence: 0.9,
                action_switches: 0.2,
                reward_variance: 0.1,
                trajectory_length: 10,
            },
            ClassificationResult {
                dynamics: TrajectoryDynamics::StrangeAttractor,
                confidence: 0.7,
                action_switches: 0.8,
                reward_variance: 0.5,
                trajectory_length: 8,
            },
        ];
        let report = TrajectoryAnalysisReport::new(&results);
        assert_eq!(report.total_trajectories, 2);
        assert_eq!(*report.dynamics_counts.get(&TrajectoryDynamics::NormalDiffusion).expect("value should be ok in test"), 1);
        assert_eq!(*report.dynamics_counts.get(&TrajectoryDynamics::StrangeAttractor).expect("value should be ok in test"), 1);
        assert!((report.avg_confidence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_limit_cycle_not_normal_diffusion() {
        let classifier = TrajectoryClassifier::new();
        let rewards = vec![0.5, 0.1, 0.5, 0.1, 0.5, 0.1, 0.5, 0.1, 0.5];
        let actions = make_actions(&["a", "b", "a", "b", "a", "b", "a", "b", "a"]);
        let result = classifier.classify(&rewards, &actions);
        assert_eq!(result.dynamics, TrajectoryDynamics::LimitCycle);
    }
}
