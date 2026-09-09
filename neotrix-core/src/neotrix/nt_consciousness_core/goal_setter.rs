//! 目标设定器 (GoalSetter)
//! 
//! 自主设定目标、优先级排序、动态调整

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 目标设定器
pub struct GoalSetter {
    /// 目标库
    pub goals: Vec<Goal>,
    /// 目标历史
    pub history: Vec<GoalRecord>,
}

/// 目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub goal_type: GoalType,
    pub priority: GoalPriority,
    pub status: GoalStatus,
    pub progress: f64,
    pub deadline: Option<String>,
}

/// 目标类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    Learning,
    Performance,
    Exploration,
    Creation,
    Maintenance,
}

/// 目标优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// 目标状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoalStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Abandoned,
}

/// 目标记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRecord {
    pub id: String,
    pub cycle: u32,
    pub action: GoalAction,
    pub goal: Goal,
    pub timestamp: String,
}

/// 目标动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalAction {
    Created,
    Updated,
    Completed,
    Failed,
    Adjusted,
}

impl GoalSetter {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 设定目标
    pub fn set_goal(&mut self, cycle: u32, goal: Goal) {
        let record = GoalRecord {
            id: format!("goal_{}", uuid::Uuid::new_v4()),
            cycle,
            action: GoalAction::Created,
            goal: goal.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);
        self.goals.push(goal);
    }

    /// 更新进度
    pub fn update_progress(&mut self, cycle: u32, goal_id: &str, progress: f64) {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.progress = progress;
            
            let record = GoalRecord {
                id: format!("goal_{}", uuid::Uuid::new_v4()),
                cycle,
                action: GoalAction::Updated,
                goal: goal.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            self.history.push(record);
        }
    }

    /// 获取活跃目标
    pub fn active_goals(&self) -> Vec<&Goal> {
        self.goals.iter()
            .filter(|g| g.status == GoalStatus::InProgress || g.status == GoalStatus::Pending)
            .collect()
    }

    /// 获取统计
    pub fn stats(&self) -> GoalStats {
        GoalStats {
            total_goals: self.goals.len(),
            active_goals: self.active_goals().len(),
            completed_goals: self.goals.iter().filter(|g| g.status == GoalStatus::Completed).count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalStats {
    pub total_goals: usize,
    pub active_goals: usize,
    pub completed_goals: usize,
}

impl std::fmt::Display for GoalStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GoalSetter: {} total, {} active, {} completed",
            self.total_goals, self.active_goals, self.completed_goals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_setter() {
        let mut setter = GoalSetter::new();
        let goal = Goal {
            id: "g1".to_string(),
            name: "Test Goal".to_string(),
            description: "Test".to_string(),
            goal_type: GoalType::Learning,
            priority: GoalPriority::Medium,
            status: GoalStatus::Pending,
            progress: 0.0,
            deadline: None,
        };
        setter.set_goal(0, goal);
        assert_eq!(setter.goals.len(), 1);
    }
}
