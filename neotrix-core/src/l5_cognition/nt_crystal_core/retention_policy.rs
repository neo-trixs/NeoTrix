//! 策略进化器 — 自我优化记忆管理
//!
//! 根据性能指标自动调整保留策略，实现记忆管理的自适应进化。

use super::cocoons::{DecayStrategy, RetentionPolicy};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 策略进化器 — 自我优化记忆管理
pub struct StrategyEvolver {
    pub current: RetentionPolicy,
    pub history: Vec<StrategyPerformance>,
    pub meta_strategy: MetaStrategy,
}

/// 策略性能记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub policy: RetentionPolicy,
    pub recall_accuracy: f64,
    pub memory_efficiency: f64,
    pub timestamp: u64,
}

/// 元策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaStrategy {
    pub exploration_rate: f64,
    pub min_samples_for_change: usize,
    pub performance_window: usize,
}

impl StrategyEvolver {
    /// 创建新的策略进化器
    pub fn new(initial: RetentionPolicy) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
            meta_strategy: MetaStrategy {
                exploration_rate: 0.1,
                min_samples_for_change: 10,
                performance_window: 20,
            },
        }
    }

    /// 记录策略性能
    pub fn record_performance(&mut self, accuracy: f64, efficiency: f64) {
        let performance = StrategyPerformance {
            policy: self.current.clone(),
            recall_accuracy: accuracy,
            memory_efficiency: efficiency,
            timestamp: timestamp_now(),
        };

        self.history.push(performance);

        // 保持历史在窗口内
        if self.history.len() > self.meta_strategy.performance_window {
            self.history.remove(0);
        }
    }

    /// 判断是否应该进化
    pub fn should_evolve(&self) -> bool {
        if self.history.len() < self.meta_strategy.min_samples_for_change {
            return false;
        }

        // 计算最近性能的平均值
        let recent: Vec<&StrategyPerformance> = self
            .history
            .iter()
            .rev()
            .take(self.meta_strategy.performance_window)
            .collect();

        let avg_accuracy: f64 =
            recent.iter().map(|p| p.recall_accuracy).sum::<f64>() / recent.len() as f64;

        let avg_efficiency: f64 =
            recent.iter().map(|p| p.memory_efficiency).sum::<f64>() / recent.len() as f64;

        // 如果性能低于阈值，尝试进化
        avg_accuracy < 0.7 || avg_efficiency < 0.6
    }

    /// 执行策略进化
    pub fn evolve(&mut self) -> RetentionPolicy {
        // 探索：以一定概率随机调整
        if rand_f64() < self.meta_strategy.exploration_rate {
            return self.random_mutation();
        }

        // 利用：基于历史数据调整
        self.optimize_based_on_history()
    }

    /// 根据失败类型诊断并生成修复策略
    pub fn diagnose_failure(&self, failure_type: &str) -> RetentionPolicy {
        match failure_type {
            "low_recall" => {
                // 低召回率：降低强度阈值，增加最大记忆数
                RetentionPolicy {
                    max_memories: (self.current.max_memories as f64 * 1.5) as usize,
                    min_strength: (self.current.min_strength * 0.5).max(0.01),
                    domain_weights: self.current.domain_weights.clone(),
                    decay_strategy: DecayStrategy::Exponential { rate: 0.005 },
                }
            }
            "memory_bloat" => {
                // 记忆膨胀：提高强度阈值，减少最大记忆数
                RetentionPolicy {
                    max_memories: (self.current.max_memories as f64 * 0.7) as usize,
                    min_strength: (self.current.min_strength * 1.5).min(0.5),
                    domain_weights: self.current.domain_weights.clone(),
                    decay_strategy: DecayStrategy::Exponential { rate: 0.02 },
                }
            }
            "domain_imbalance" => {
                // 领域不平衡：调整领域权重
                let mut new_weights = self.current.domain_weights.clone();
                for weight in new_weights.values_mut() {
                    *weight = 1.0; // 重置为均衡
                }
                RetentionPolicy {
                    max_memories: self.current.max_memories,
                    min_strength: self.current.min_strength,
                    domain_weights: new_weights,
                    decay_strategy: self.current.decay_strategy.clone(),
                }
            }
            _ => self.current.clone(),
        }
    }

    /// 获取当前策略的性能摘要
    pub fn performance_summary(&self) -> PerformanceSummary {
        if self.history.is_empty() {
            return PerformanceSummary::default();
        }

        let recent: Vec<&StrategyPerformance> = self
            .history
            .iter()
            .rev()
            .take(self.meta_strategy.performance_window)
            .collect();

        let avg_accuracy: f64 =
            recent.iter().map(|p| p.recall_accuracy).sum::<f64>() / recent.len() as f64;

        let avg_efficiency: f64 =
            recent.iter().map(|p| p.memory_efficiency).sum::<f64>() / recent.len() as f64;

        let trend = if self.history.len() >= 2 {
            let last = self.history.last().unwrap().recall_accuracy;
            let prev = self.history[self.history.len() - 2].recall_accuracy;
            if last > prev {
                Trend::Improving
            } else if last < prev {
                Trend::Declining
            } else {
                Trend::Stable
            }
        } else {
            Trend::Stable
        };

        PerformanceSummary {
            avg_recall_accuracy: avg_accuracy,
            avg_memory_efficiency: avg_efficiency,
            trend,
            evolution_count: self.history.len(),
        }
    }

    /// 随机突变策略
    fn random_mutation(&self) -> RetentionPolicy {
        let mut policy = self.current.clone();

        // 随机调整参数
        let mutation_range = 0.2;
        let delta = (rand_f64() - 0.5) * mutation_range;

        policy.max_memories = ((policy.max_memories as f64 * (1.0 + delta)) as usize).max(100);
        policy.min_strength = (policy.min_strength * (1.0 + delta)).clamp(0.01, 0.5);

        // 随机调整衰减率
        match &mut policy.decay_strategy {
            DecayStrategy::Exponential { rate } => {
                *rate = (*rate * (1.0 + delta)).clamp(0.001, 0.1);
            }
            DecayStrategy::Linear { step } => {
                *step = (*step * (1.0 + delta)).clamp(0.001, 0.1);
            }
            DecayStrategy::StepFunction { .. } => {
                // 步进函数不调整
            }
        }

        policy
    }

    /// 基于历史优化策略
    fn optimize_based_on_history(&self) -> RetentionPolicy {
        if self.history.len() < 2 {
            return self.current.clone();
        }

        // 找到最佳性能的策略
        let best = self
            .history
            .iter()
            .max_by(|a, b| {
                let score_a = a.recall_accuracy * 0.6 + a.memory_efficiency * 0.4;
                let score_b = b.recall_accuracy * 0.6 + b.memory_efficiency * 0.4;
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        // 从最佳策略开始，进行微调
        let mut new_policy = best.policy.clone();

        // 如果性能仍然不理想，继续调整
        let avg_accuracy: f64 =
            self.history.iter().map(|p| p.recall_accuracy).sum::<f64>() / self.history.len() as f64;

        if avg_accuracy < 0.6 {
            // 减少记忆数量，提高质量
            new_policy.max_memories = (new_policy.max_memories as f64 * 0.8) as usize;
            new_policy.min_strength = (new_policy.min_strength * 1.2).min(0.5);
        } else if avg_accuracy > 0.9 {
            // 性能很好，可以尝试增加容量
            new_policy.max_memories = (new_policy.max_memories as f64 * 1.2) as usize;
        }

        new_policy
    }
}

/// 趋势方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Declining,
}

impl Default for Trend {
    fn default() -> Self {
        Self::Stable
    }
}

/// 性能摘要
#[derive(Debug, Default)]
pub struct PerformanceSummary {
    pub avg_recall_accuracy: f64,
    pub avg_memory_efficiency: f64,
    pub trend: Trend,
    pub evolution_count: usize,
}

/// 时间戳生成
fn timestamp_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 简单的伪随机数生成（避免引入额外依赖）
fn rand_f64() -> f64 {
    let now = timestamp_now();
    let bits = now
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (bits as f64) / (u64::MAX as f64)
}

impl std::fmt::Display for StrategyEvolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = self.performance_summary();
        writeln!(f, "=== Strategy Evolver ===")?;
        writeln!(f, "History size: {}", self.history.len())?;
        writeln!(f, "Avg recall accuracy: {:.3}", summary.avg_recall_accuracy)?;
        writeln!(
            f,
            "Avg memory efficiency: {:.3}",
            summary.avg_memory_efficiency
        )?;
        writeln!(f, "Trend: {:?}", summary.trend)?;
        Ok(())
    }
}
