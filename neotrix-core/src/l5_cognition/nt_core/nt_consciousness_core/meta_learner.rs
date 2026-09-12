//! 元学习器 (MetaLearner)
//! 
//! 学习如何学习、优化策略、迁移知识
//! 
//! 参考: Intrinsic Metacognitive Learning (ICML 2025)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 元学习器
pub struct MetaLearner {
    /// 元知识
    pub meta_knowledge: _MetaKnowledge,
    /// 学习策略
    pub strategies: Vec<_LearningStrategy>,
    /// 学习历史
    pub history: Vec<_MetaLearningRecord>,
}

/// 元知识
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct _MetaKnowledge {
    /// 学习方法有效性
    pub method_effectiveness: HashMap<String, f64>,
    /// 任务类型适配
    pub task_adaptation: HashMap<String, String>,
    /// 错误模式
    pub error_patterns: Vec<_ErrorPattern>,
}

/// 错误模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _ErrorPattern {
    pub pattern_type: String,
    pub frequency: u32,
    pub correction_strategy: String,
}

/// 学习策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _LearningStrategy {
    pub id: String,
    pub name: String,
    pub strategy_type: StrategyType,
    pub effectiveness: f64,
    pub applicable_tasks: Vec<String>,
}

/// 策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Supervised,
    Unsupervised,
    Reinforcement,
    Transfer,
    Meta,
}

/// 元学习记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _MetaLearningRecord {
    pub id: String,
    pub cycle: u32,
    pub learning_task: String,
    pub strategy_used: String,
    pub effectiveness: f64,
    pub insights: Vec<String>,
    pub timestamp: String,
}

impl MetaLearner {
    pub fn new() -> Self {
        Self {
            meta_knowledge: _MetaKnowledge::default(),
            strategies: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 学习如何学习
    pub(crate) fn _meta_learn(&mut self, cycle: u32, task: &str, results: &[String]) -> Vec<String> {
        let insights = vec![
            format!("Learned from task: {}", task),
            format!("Results analyzed: {}", results.len()),
        ];

        // 更新元知识
        let effectiveness = if results.is_empty() { 0.5 } else { 0.7 };
        self.meta_knowledge.method_effectiveness.insert(
            task.to_string(),
            effectiveness,
        );

        let record = _MetaLearningRecord {
            id: format!("meta_{}", uuid::Uuid::new_v4()),
            cycle,
            learning_task: task.to_string(),
            strategy_used: "meta_learning".to_string(),
            effectiveness,
            insights: insights.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);
        insights
    }

    /// 选择最佳策略
    pub fn select_strategy(&self, task_type: &str) -> Option<&_LearningStrategy> {
        self.strategies.iter()
            .filter(|s| s.applicable_tasks.contains(&task_type.to_string()))
            .max_by(|a, b| a.effectiveness.partial_cmp(&b.effectiveness).unwrap())
    }

    /// 获取统计
    pub fn stats(&self) -> _MetaLearnerStats {
        _MetaLearnerStats {
            total_meta_learnings: self.history.len(),
            total_strategies: self.strategies.len(),
            avg_effectiveness: if self.history.is_empty() {
                0.0
            } else {
                self.history.iter().map(|r| r.effectiveness).sum::<f64>() / self.history.len() as f64
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _MetaLearnerStats {
    pub total_meta_learnings: usize,
    pub total_strategies: usize,
    pub avg_effectiveness: f64,
}

impl std::fmt::Display for _MetaLearnerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "MetaLearner: {} learnings, {} strategies, avg effectiveness {:.4}",
            self.total_meta_learnings, self.total_strategies, self.avg_effectiveness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_learner() {
        let mut learner = MetaLearner::new();
        let insights = learner._meta_learn(0, "test_task", &["result1".to_string()]);
        assert_eq!(insights.len(), 2);
    }
}
