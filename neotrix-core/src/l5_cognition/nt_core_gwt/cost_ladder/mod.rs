//! # 成本阶梯 (Hermes pattern)
//!
//! 根据任务新颖性×复杂度分配模型梯队:
//! - Tutti (便宜): 简单任务
//! - Soloist (中等): 中等任务
//! - Conductor (昂贵): 复杂任务

use serde::{Deserialize, Serialize};

/// 模型梯队
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rung {
    pub name: String,
    pub model_tier: String,
    pub max_cost_per_task: f64,
    pub capabilities: Vec<String>,
}

/// 成本阶梯路由器
pub struct CostLadder {
    pub rungs: Vec<Rung>,
    pub novelty_threshold_low: f64,
    pub novelty_threshold_high: f64,
    pub complexity_threshold_low: f64,
    pub complexity_threshold_high: f64,
}

impl CostLadder {
    pub fn new() -> Self {
        Self {
            rungs: vec![
                Rung {
                    name: "Tutti".into(),
                    model_tier: "cheap".into(),
                    max_cost_per_task: 0.01,
                    capabilities: vec!["chat".into(), "simple_qa".into()],
                },
                Rung {
                    name: "Soloist".into(),
                    model_tier: "medium".into(),
                    max_cost_per_task: 0.1,
                    capabilities: vec!["code".into(), "analysis".into()],
                },
                Rung {
                    name: "Conductor".into(),
                    model_tier: "expensive".into(),
                    max_cost_per_task: 1.0,
                    capabilities: vec!["reasoning".into(), "creative".into()],
                },
            ],
            novelty_threshold_low: 0.3,
            novelty_threshold_high: 0.7,
            complexity_threshold_low: 0.3,
            complexity_threshold_high: 0.7,
        }
    }
    
    /// 根据新颖性×复杂度分配梯队
    pub fn assign_rung(&self, novelty: f64, complexity: f64) -> &Rung {
        if novelty < self.novelty_threshold_low && complexity < self.complexity_threshold_low {
            &self.rungs[0] // Tutti
        } else if novelty < self.novelty_threshold_high && complexity < self.complexity_threshold_high {
            &self.rungs[1] // Soloist
        } else {
            &self.rungs[2] // Conductor
        }
    }
    
    /// 获取最低成本梯队
    pub fn cheapest_rung(&self) -> &Rung {
        &self.rungs[0]
    }
    
    /// 获取最高能力梯队
    pub fn most_capable_rung(&self) -> &Rung {
        self.rungs.last().unwrap()
    }
}

impl Default for CostLadder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cheap_rung() {
        let ladder = CostLadder::new();
        let rung = ladder.assign_rung(0.1, 0.1);
        assert_eq!(rung.name, "Tutti");
    }
    
    #[test]
    fn test_expensive_rung() {
        let ladder = CostLadder::new();
        let rung = ladder.assign_rung(0.9, 0.9);
        assert_eq!(rung.name, "Conductor");
    }
}
