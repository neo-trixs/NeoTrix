//! T2.9: 成本阶梯路由模块 (Hermes C2)
//!
//! CostLadder: 按新颖性×复杂度分配模型梯队
//! FatigueDetector: 疲劳检测 + 热交换

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// Rung
// ============================================================================

/// 成本阶梯档位
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rung {
    /// 档位名 ("Tutti" / "Soloist" / "Conductor")
    pub name: String,
    /// 模型层级 ("cheap" / "medium" / "expensive")
    pub model_tier: String,
    /// 每任务最大成本 (USD)
    pub max_cost_per_task: f64,
    /// 支持的能力列表
    pub capabilities: Vec<String>,
}

// ============================================================================
// TaskType
// ============================================================================

/// 任务特征描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskType {
    /// 任务名
    pub name: String,
    /// 新颖性 (0.0-1.0)
    pub novelty: f64,
    /// 复杂度 (0.0-1.0)
    pub complexity: f64,
}

// ============================================================================
// NoveltyComplexityMatrix
// ============================================================================

/// 新颖性×复杂度矩阵
pub struct NoveltyComplexityMatrix {
    /// 自定义评分函数 (可选)
    scorer: Option<Box<dyn Fn(&TaskType) -> (f64, f64) + Send + Sync>>,
}

impl NoveltyComplexityMatrix {
    pub fn new() -> Self {
        Self { scorer: None }
    }

    pub fn with_scorer<F>(scorer: F) -> Self
    where
        F: Fn(&TaskType) -> (f64, f64) + Send + Sync + 'static,
    {
        Self {
            scorer: Some(Box::new(scorer)),
        }
    }

    pub fn novelty(&self, task: &TaskType) -> f64 {
        if let Some(f) = &self.scorer {
            return f(task).0;
        }
        task.novelty
    }

    pub fn complexity(&self, task: &TaskType) -> f64 {
        if let Some(f) = &self.scorer {
            return f(task).1;
        }
        task.complexity
    }
}

impl Default for NoveltyComplexityMatrix {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CostLadder
// ============================================================================

/// 成本阶梯路由器
pub struct CostLadder {
    /// 梯队定义 (按成本升序)
    pub rungs: Vec<Rung>,
    /// 新颖性×复杂度矩阵
    pub matrix: NoveltyComplexityMatrix,
}

impl CostLadder {
    /// 创建默认 3 档阶梯
    pub fn default_three_rung() -> Self {
        Self {
            rungs: vec![
                Rung {
                    name: "Tutti".into(),
                    model_tier: "cheap".into(),
                    max_cost_per_task: 0.01,
                    capabilities: vec!["format".into(), "classify".into(), "dedup".into()],
                },
                Rung {
                    name: "Soloist".into(),
                    model_tier: "medium".into(),
                    max_cost_per_task: 0.10,
                    capabilities: vec![
                        "summarize".into(),
                        "extract".into(),
                        "translate".into(),
                    ],
                },
                Rung {
                    name: "Conductor".into(),
                    model_tier: "expensive".into(),
                    max_cost_per_task: 1.0,
                    capabilities: vec![
                        "reason".into(),
                        "code".into(),
                        "architect".into(),
                        "research".into(),
                    ],
                },
            ],
            matrix: NoveltyComplexityMatrix::new(),
        }
    }

    /// 根据任务特征分配梯队
    pub fn assign_rung(&self, task: &TaskType) -> &Rung {
        let n = self.matrix.novelty(task);
        let c = self.matrix.complexity(task);

        if n < 0.3 && c < 0.3 {
            return &self.rungs[0]; // Tutti
        }
        if n < 0.7 && c < 0.7 {
            return &self.rungs[1]; // Soloist
        }
        &self.rungs[2] // Conductor
    }

    /// 根据梯队名查找
    pub fn find_by_name(&self, name: &str) -> Option<&Rung> {
        self.rungs.iter().find(|r| r.name == name)
    }

    /// 最便宜的梯队
    pub fn cheapest(&self) -> Option<&Rung> {
        self.rungs.first()
    }

    /// 最贵的梯队
    pub fn most_expensive(&self) -> Option<&Rung> {
        self.rungs.last()
    }
}

// ============================================================================
// UsageStats
// ============================================================================

/// 代理使用统计
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_tasks: u64,
    pub recent_failures: u64,
    pub avg_latency: Duration,
    pub cost_accumulated: f64,
    pub last_used: Option<Instant>,
}

// ============================================================================
// FatigueDetector
// ============================================================================

/// 疲劳检测 + 热交换
pub struct FatigueDetector {
    /// 每个代理的使用统计
    usage: HashMap<String, UsageStats>,
    /// 失败率阈值
    failure_threshold: f64,
    /// 延迟阈值
    latency_threshold: Duration,
}

impl FatigueDetector {
    pub fn new() -> Self {
        Self {
            usage: HashMap::new(),
            failure_threshold: 0.3,
            latency_threshold: Duration::from_secs(30),
        }
    }

    /// 记录任务结果
    pub fn record(&mut self, agent_id: &str, success: bool, latency: Duration, cost: f64) {
        let entry = self.usage.entry(agent_id.to_string()).or_insert(UsageStats {
            total_tasks: 0,
            recent_failures: 0,
            avg_latency: Duration::ZERO,
            cost_accumulated: 0.0,
            last_used: None,
        });
        entry.total_tasks += 1;
        if !success {
            entry.recent_failures += 1;
        }
        // 滑动平均
        let n = entry.total_tasks as f64;
        entry.avg_latency =
            Duration::from_secs_f64((entry.avg_latency.as_secs_f64() * (n - 1.0) + latency.as_secs_f64()) / n);
        entry.cost_accumulated += cost;
        entry.last_used = Some(Instant::now());
    }

    /// 检测代理是否疲劳
    pub fn is_fatigued(&self, agent_id: &str) -> bool {
        match self.usage.get(agent_id) {
            None => false,
            Some(s) => {
                if s.total_tasks == 0 {
                    return false;
                }
                let failure_rate = s.recent_failures as f64 / s.total_tasks as f64;
                failure_rate > self.failure_threshold || s.avg_latency > self.latency_threshold
            }
        }
    }

    /// 热交换: 将疲劳代理替换为健康代理
    pub fn hot_swap(&self, fatigued: &str, available: &[String]) -> Option<String> {
        available
            .iter()
            .find(|id| id.as_str() != fatigued && !self.is_fatigued(id))
            .cloned()
    }

    /// 获取代理统计
    pub fn stats(&self, agent_id: &str) -> Option<&UsageStats> {
        self.usage.get(agent_id)
    }

    /// 所有代理 ID
    pub fn agents(&self) -> Vec<&str> {
        self.usage.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FatigueDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_ladder_assignment() {
        let ladder = CostLadder::default_three_rung();

        let simple = TaskType {
            name: "format".into(),
            novelty: 0.1,
            complexity: 0.1,
        };
        assert_eq!(ladder.assign_rung(&simple).name, "Tutti");

        let medium = TaskType {
            name: "summarize".into(),
            novelty: 0.5,
            complexity: 0.5,
        };
        assert_eq!(ladder.assign_rung(&medium).name, "Soloist");

        let hard = TaskType {
            name: "architect".into(),
            novelty: 0.9,
            complexity: 0.9,
        };
        assert_eq!(ladder.assign_rung(&hard).name, "Conductor");
    }

    #[test]
    fn test_fatigue_detection() {
        let mut fd = FatigueDetector::new();
        // 10 tasks, 4 failures → 40% > 30%
        for i in 0..10 {
            fd.record("agent_a", i >= 6, Duration::from_millis(10), 0.01);
        }
        assert!(fd.is_fatigued("agent_a"));
        assert!(!fd.is_fatigued("agent_b"));
    }

    #[test]
    fn test_hot_swap() {
        let mut fd = FatigueDetector::new();
        for _ in 0..10 {
            fd.record("fatigued", false, Duration::from_millis(10), 0.01);
        }
        let available = vec!["fatigued".into(), "healthy".into()];
        let swap = fd.hot_swap("fatigued", &available);
        assert_eq!(swap.as_deref(), Some("healthy"));
    }

    #[test]
    fn test_latency_fatigue() {
        let mut fd = FatigueDetector::new();
        fd.record("slow", true, Duration::from_secs(60), 0.01);
        assert!(fd.is_fatigued("slow"));
    }
}
