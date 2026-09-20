//! # 双轨自治 (CrewAI pattern)
//!
//! 双轨路由器: Crews=自主 + Flows=确定性
//! 基于新颖性×复杂度矩阵决定任务走哪条轨道。

use serde::{Deserialize, Serialize};

/// 轨道类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Track {
    /// 确定性流 (低新颖性+低复杂度)
    Flow,
    /// 自主团队 (高新颖性或高复杂度)
    Crew,
}

/// 新颖性×复杂度矩阵
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoveltyComplexityMatrix {
    pub novelty: f64,
    pub complexity: f64,
}

impl NoveltyComplexityMatrix {
    pub fn new(novelty: f64, complexity: f64) -> Self {
        Self {
            novelty: novelty.clamp(0.0, 1.0),
            complexity: complexity.clamp(0.0, 1.0),
        }
    }
}

/// 双轨路由器
pub struct DualTrackRouter {
    /// 新颖性阈值
    novelty_threshold: f64,
    /// 复杂度阈值
    complexity_threshold: f64,
}

impl DualTrackRouter {
    pub fn new() -> Self {
        Self {
            novelty_threshold: 0.5,
            complexity_threshold: 0.5,
        }
    }
    
    pub fn with_thresholds(novelty: f64, complexity: f64) -> Self {
        Self {
            novelty_threshold: novelty,
            complexity_threshold: complexity,
        }
    }
    
    /// 路由到正确的轨道
    pub fn route(&self, matrix: &NoveltyComplexityMatrix) -> Track {
        if matrix.novelty > self.novelty_threshold || matrix.complexity > self.complexity_threshold {
            Track::Crew
        } else {
            Track::Flow
        }
    }
    
    /// 评估任务并路由
    pub fn evaluate_and_route(&self, task_description: &str) -> (Track, NoveltyComplexityMatrix) {
        // 简单启发式评估
        let novelty = if task_description.contains("new") || task_description.contains("novel") {
            0.8
        } else if task_description.contains("improve") || task_description.contains("optimize") {
            0.4
        } else {
            0.2
        };
        
        let complexity = if task_description.len() > 500 {
            0.8
        } else if task_description.len() > 200 {
            0.5
        } else {
            0.3
        };
        
        let matrix = NoveltyComplexityMatrix::new(novelty, complexity);
        let track = self.route(&matrix);
        (track, matrix)
    }
}

impl Default for DualTrackRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flow_track() {
        let router = DualTrackRouter::new();
        let matrix = NoveltyComplexityMatrix::new(0.2, 0.3);
        assert_eq!(router.route(&matrix), Track::Flow);
    }
    
    #[test]
    fn test_crew_track() {
        let router = DualTrackRouter::new();
        let matrix = NoveltyComplexityMatrix::new(0.8, 0.9);
        assert_eq!(router.route(&matrix), Track::Crew);
    }
}
