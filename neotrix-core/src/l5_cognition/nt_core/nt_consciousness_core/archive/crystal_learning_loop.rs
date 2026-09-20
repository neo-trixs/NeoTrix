//! 晶体核心学习模块
//!
//! 从经验中学习，在使用中改进
//! 闭环学习：吸收经验 → 使用 → 反馈 → 优化 → 再使用

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 晶体技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_used: Option<i64>,
    pub version: u32,
    pub importance: f64,
}

/// 晶体学习反馈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalLearningFeedback {
    pub skill_id: String,
    pub success: bool,
    pub context: String,
    pub timestamp: i64,
    pub insight: Option<String>,
}

/// 晶体学习模块
pub struct CrystalLearningLoop {
    skills: HashMap<String, CrystalSkill>,
    feedbacks: Vec<CrystalLearningFeedback>,
    optimization_threshold: f64,
    min_feedback_for_optimize: u32,
}

/// 晶体学习统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalLearningStats {
    pub total_skills: usize,
    pub total_feedbacks: usize,
    pub avg_success_rate: f64,
}

impl CrystalLearningLoop {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            feedbacks: Vec::new(),
            optimization_threshold: 0.7,
            min_feedback_for_optimize: 5,
        }
    }

    /// 吸收经验 → 创建晶体技能
    pub fn absorb_experience(
        &mut self,
        id: String,
        name: String,
        description: String,
        domain: String,
    ) {
        let skill = CrystalSkill {
            id: id.clone(),
            name,
            description,
            domain,
            success_count: 0,
            failure_count: 0,
            last_used: None,
            version: 1,
            importance: 0.5,
        };
        self.skills.insert(id, skill);
    }

    /// 使用晶体技能
    pub fn use_skill(&mut self, skill_id: &str) -> Option<&CrystalSkill> {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.last_used = Some(chrono::Utc::now().timestamp());
            Some(skill)
        } else {
            None
        }
    }

    /// 记录反馈
    pub fn record_feedback(&mut self, feedback: CrystalLearningFeedback) {
        // 更新技能统计
        if let Some(skill) = self.skills.get_mut(&feedback.skill_id) {
            if feedback.success {
                skill.success_count += 1;
            } else {
                skill.failure_count += 1;
            }
        }

        // 记录反馈
        self.feedbacks.push(feedback);

        // 触发优化
        self.optimize_skills();
    }

    /// 优化晶体技能
    fn optimize_skills(&mut self) {
        for skill in self.skills.values_mut() {
            let total = skill.success_count + skill.failure_count;
            if total >= self.min_feedback_for_optimize {
                let success_rate = skill.success_count as f64 / total as f64;
                if success_rate < self.optimization_threshold {
                    // 成功率低于阈值，需要优化
                    skill.version += 1;
                    skill.importance *= 0.9;
                    println!(
                        "[CrystalLearning] 技能 {} 需要优化 (成功率: {:.1}%)",
                        skill.name,
                        success_rate * 100.0
                    );
                } else {
                    // 成功率高，提升重要性
                    skill.importance = (skill.importance * 1.1).min(1.0);
                }
            }
        }
    }

    /// 获取晶体技能成功率
    pub fn get_success_rate(&self, skill_id: &str) -> Option<f64> {
        self.skills.get(skill_id).map(|skill| {
            let total = skill.success_count + skill.failure_count;
            if total == 0 {
                0.0
            } else {
                skill.success_count as f64 / total as f64
            }
        })
    }

    /// 获取晶体统计
    pub fn get_stats(&self) -> CrystalLearningStats {
        let total_skills = self.skills.len();
        let total_feedbacks = self.feedbacks.len();
        let avg_success_rate = if total_skills > 0 {
            let sum: f64 = self
                .skills
                .values()
                .map(|s| {
                    let total = s.success_count + s.failure_count;
                    if total == 0 {
                        0.0
                    } else {
                        s.success_count as f64 / total as f64
                    }
                })
                .sum();
            sum / total_skills as f64
        } else {
            0.0
        };

        CrystalLearningStats {
            total_skills,
            total_feedbacks,
            avg_success_rate,
        }
    }

    /// 获取需要优化的技能
    pub fn get_skills_needing_optimization(&self) -> Vec<&CrystalSkill> {
        self.skills
            .values()
            .filter(|s| {
                let total = s.success_count + s.failure_count;
                if total < self.min_feedback_for_optimize {
                    false
                } else {
                    let success_rate = s.success_count as f64 / total as f64;
                    success_rate < self.optimization_threshold
                }
            })
            .collect()
    }

    /// 获取高价值技能
    pub fn get_high_value_skills(&self) -> Vec<&CrystalSkill> {
        self.skills
            .values()
            .filter(|s| s.importance >= 0.7)
            .collect()
    }
}

impl std::fmt::Display for CrystalLearningStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "CrystalLearning: {} skills, {} feedbacks, avg success rate {:.1}%",
            self.total_skills,
            self.total_feedbacks,
            self.avg_success_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystal_learning_absorb() {
        let mut loop_module = CrystalLearningLoop::new();
        loop_module.absorb_experience(
            "skill_1".to_string(),
            "Test Skill".to_string(),
            "A test skill".to_string(),
            "testing".to_string(),
        );
        assert_eq!(loop_module.skills.len(), 1);
    }

    #[test]
    fn test_crystal_learning_feedback() {
        let mut loop_module = CrystalLearningLoop::new();
        loop_module.absorb_experience(
            "skill_1".to_string(),
            "Test Skill".to_string(),
            "A test skill".to_string(),
            "testing".to_string(),
        );

        let feedback = CrystalLearningFeedback {
            skill_id: "skill_1".to_string(),
            success: true,
            context: "test context".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            insight: Some("Works well".to_string()),
        };

        loop_module.record_feedback(feedback);
        assert_eq!(loop_module.feedbacks.len(), 1);

        let rate = loop_module.get_success_rate("skill_1").unwrap();
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_crystal_learning_stats() {
        let mut loop_module = CrystalLearningLoop::new();
        loop_module.absorb_experience(
            "skill_1".to_string(),
            "Test Skill".to_string(),
            "A test skill".to_string(),
            "testing".to_string(),
        );

        let stats = loop_module.get_stats();
        assert_eq!(stats.total_skills, 1);
        assert_eq!(stats.total_feedbacks, 0);
    }
}
