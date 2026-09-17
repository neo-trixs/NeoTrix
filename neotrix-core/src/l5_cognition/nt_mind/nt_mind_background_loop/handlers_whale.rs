//! # WHALE Phase Handler (D5 Fix)
//!
//! WHALE (Watt-Hour Adaptive Learning Engine) 循环处理器。
//! 实现 propose → optimize → evaluate → decide 四阶段闭环。
//!
//! 每个 consciousness_tick 周期触发一次 WHALE phase detection，
//! 根据当前系统状态选择最佳优化策略。

use serde::{Deserialize, Serialize};

/// WHALE 循环阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhalePhase {
    /// 监控阶段: 收集系统指标
    Monitor,
    /// 分析阶段: 识别瓶颈
    Analyze,
    /// 优化阶段: 执行优化策略
    Optimize,
    /// 评估阶段: 验证优化效果
    Evaluate,
}

impl std::fmt::Display for WhalePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Monitor => write!(f, "Monitor"),
            Self::Analyze => write!(f, "Analyze"),
            Self::Optimize => write!(f, "Optimize"),
            Self::Evaluate => write!(f, "Evaluate"),
        }
    }
}

/// WHALE 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleState {
    /// 当前阶段
    pub phase: WhalePhase,
    /// 阶段计数
    pub phase_count: u64,
    /// 上次优化效果 (0-1)
    pub last_optimization_effect: f64,
    /// 累计优化次数
    pub total_optimizations: u64,
    /// 累计评估次数
    pub total_evaluations: u64,
}

impl Default for WhaleState {
    fn default() -> Self {
        Self {
            phase: WhalePhase::Monitor,
            phase_count: 0,
            last_optimization_effect: 0.0,
            total_optimizations: 0,
            total_evaluations: 0,
        }
    }
}

/// WHALE Phase Detector — 检测并驱动阶段切换
pub struct WhalePhaseDetector {
    state: WhaleState,
    /// 优化效果阈值: 低于此值触发重新优化
    optimization_threshold: f64,
    /// 最大连续优化次数
    max_consecutive_optimizations: u32,
    /// 当前连续优化计数
    consecutive_optimizations: u32,
}

impl WhalePhaseDetector {
    pub fn new() -> Self {
        Self {
            state: WhaleState::default(),
            optimization_threshold: 0.3,
            max_consecutive_optimizations: 5,
            consecutive_optimizations: 0,
        }
    }

    /// 推进一个阶段
    pub fn advance_phase(&mut self) -> WhalePhase {
        self.state.phase = match self.state.phase {
            WhalePhase::Monitor => WhalePhase::Analyze,
            WhalePhase::Analyze => WhalePhase::Optimize,
            WhalePhase::Optimize => WhalePhase::Evaluate,
            WhalePhase::Evaluate => {
                self.state.total_evaluations += 1;
                // 如果效果差，回到 Monitor 重新开始
                if self.state.last_optimization_effect < self.optimization_threshold {
                    self.consecutive_optimizations += 1;
                } else {
                    self.consecutive_optimizations = 0;
                }
                WhalePhase::Monitor
            }
        };
        self.state.phase_count += 1;
        self.state.phase
    }

    /// 执行 Monitor: 收集系统指标
    pub fn monitor(&self, capability_mean: f64, fatigue: f64, uncertainty: f64) -> WhaleMetrics {
        WhaleMetrics {
            capability_mean,
            fatigue,
            uncertainty,
            needs_optimization: capability_mean < 0.7 || fatigue > 0.5 || uncertainty > 0.6,
        }
    }

    /// 执行 Analyze: 识别瓶颈
    pub fn analyze(&self, metrics: &WhaleMetrics) -> OptimizationStrategy {
        if metrics.fatigue > 0.5 {
            OptimizationStrategy::ReduceLoad
        } else if metrics.uncertainty > 0.6 {
            OptimizationStrategy::IncreaseConfidence
        } else if metrics.capability_mean < 0.5 {
            OptimizationStrategy::BoostCapability
        } else {
            OptimizationStrategy::FineTune
        }
    }

    /// 执行 Optimize: 应用策略
    pub fn optimize(&mut self, strategy: OptimizationStrategy) -> OptimizationResult {
        self.state.total_optimizations += 1;
        OptimizationResult {
            strategy,
            applied: true,
            estimated_improvement: match strategy {
                OptimizationStrategy::ReduceLoad => 0.15,
                OptimizationStrategy::IncreaseConfidence => 0.2,
                OptimizationStrategy::BoostCapability => 0.25,
                OptimizationStrategy::FineTune => 0.1,
            },
        }
    }

    /// 执行 Evaluate: 验证效果
    pub fn evaluate(&mut self, before: f64, after: f64) -> f64 {
        let improvement = (after - before).max(0.0);
        self.state.last_optimization_effect = improvement;
        improvement
    }

    /// 获取当前状态
    pub fn state(&self) -> &WhaleState {
        &self.state
    }

    /// 是否需要优化
    pub fn needs_optimization(&self) -> bool {
        self.consecutive_optimizations < self.max_consecutive_optimizations
    }
}

impl Default for WhalePhaseDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// WHALE 监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleMetrics {
    pub capability_mean: f64,
    pub fatigue: f64,
    pub uncertainty: f64,
    pub needs_optimization: bool,
}

/// 优化策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    ReduceLoad,
    IncreaseConfidence,
    BoostCapability,
    FineTune,
}

/// 优化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub strategy: OptimizationStrategy,
    pub applied: bool,
    pub estimated_improvement: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_phase_advance() {
        let mut detector = WhalePhaseDetector::new();
        assert_eq!(detector.state().phase, WhalePhase::Monitor);

        detector.advance_phase();
        assert_eq!(detector.state().phase, WhalePhase::Analyze);

        detector.advance_phase();
        assert_eq!(detector.state().phase, WhalePhase::Optimize);

        detector.advance_phase();
        assert_eq!(detector.state().phase, WhalePhase::Evaluate);

        detector.advance_phase();
        assert_eq!(detector.state().phase, WhalePhase::Monitor);
    }

    #[test]
    fn test_whale_monitor() {
        let detector = WhalePhaseDetector::new();
        let metrics = detector.monitor(0.8, 0.2, 0.3);
        assert!(!metrics.needs_optimization);

        let metrics = detector.monitor(0.5, 0.6, 0.7);
        assert!(metrics.needs_optimization);
    }

    #[test]
    fn test_whale_analyze() {
        let detector = WhalePhaseDetector::new();
        let metrics = WhaleMetrics {
            capability_mean: 0.5,
            fatigue: 0.6,
            uncertainty: 0.3,
            needs_optimization: true,
        };
        assert_eq!(detector.analyze(&metrics), OptimizationStrategy::ReduceLoad);
    }

    #[test]
    fn test_whale_evaluate() {
        let mut detector = WhalePhaseDetector::new();
        let improvement = detector.evaluate(0.5, 0.7);
        assert!((improvement - 0.2).abs() < 1e-6);
    }
}