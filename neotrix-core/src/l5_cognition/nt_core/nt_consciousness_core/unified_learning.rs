//! 统一学习层 (UnifiedLearningEngine)
//! 按 FUSION-ARCHITECTURE-v4 设计

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 学习策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStrategy {
    /// 技能学习
    SkillLearning,
    /// 经验蒸馏
    ExperienceDistillation,
    /// 元学习
    MetaLearning,
    /// 跨域迁移
    CrossDomainTransfer,
}

/// 学习风格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningStyle {
    /// 被动学习
    Passive,
    /// 主动探索
    Active,
    /// 混合学习
    Hybrid,
}

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

/// 学习反馈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningFeedback {
    pub skill_id: String,
    pub success: bool,
    pub context: String,
    pub timestamp: i64,
    pub insight: Option<String>,
}

/// 学习协调器
pub struct LearningCoordination {
    /// 技能↔蒸馏桥接
    skill_distillation_bridge: HashMap<String, String>,
    /// 反馈路由
    feedback_routing: Vec<String>,
}

impl LearningCoordination {
    pub fn new() -> Self {
        Self {
            skill_distillation_bridge: HashMap::new(),
            feedback_routing: Vec::new(),
        }
    }

    /// 桥接技能和蒸馏
    pub fn bridge_skill_distillation(&mut self, skill_id: String, distillation_id: String) {
        self.skill_distillation_bridge
            .insert(skill_id, distillation_id);
    }

    /// 路由反馈
    pub fn route_feedback(&mut self, feedback: &LearningFeedback) {
        self.feedback_routing.push(feedback.skill_id.clone());
    }
}

/// 统一学习引擎
pub struct UnifiedLearningEngine {
    skills: HashMap<String, CrystalSkill>,
    feedbacks: Vec<LearningFeedback>,
    strategies: Vec<LearningStrategy>,
    style: LearningStyle,
    coordination: LearningCoordination,
    optimization_threshold: f64,
    min_feedback_for_optimize: u32,
}

impl UnifiedLearningEngine {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            feedbacks: Vec::new(),
            strategies: vec![
                LearningStrategy::SkillLearning,
                LearningStrategy::ExperienceDistillation,
                LearningStrategy::MetaLearning,
            ],
            style: LearningStyle::Hybrid,
            coordination: LearningCoordination::new(),
            optimization_threshold: 0.7,
            min_feedback_for_optimize: 5,
        }
    }

    /// 吸收经验 → 创建技能
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

    /// 使用技能
    pub fn use_skill(&mut self, skill_id: &str) -> Option<&CrystalSkill> {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.last_used = Some(chrono::Utc::now().timestamp());
            Some(skill)
        } else {
            None
        }
    }

    /// 记录反馈
    pub fn record_feedback(&mut self, feedback: LearningFeedback) {
        // 更新技能统计
        if let Some(skill) = self.skills.get_mut(&feedback.skill_id) {
            if feedback.success {
                skill.success_count += 1;
            } else {
                skill.failure_count += 1;
            }
        }

        // 路由反馈
        self.coordination.route_feedback(&feedback);

        // 记录反馈
        self.feedbacks.push(feedback);

        // 触发优化
        self.optimize_skills();
    }

    /// 优化技能
    fn optimize_skills(&mut self) {
        for skill in self.skills.values_mut() {
            let total = skill.success_count + skill.failure_count;
            if total >= self.min_feedback_for_optimize {
                let success_rate = skill.success_count as f64 / total as f64;
                if success_rate < self.optimization_threshold {
                    skill.version += 1;
                    skill.importance *= 0.9;
                    println!(
                        "[UnifiedLearning] 技能 {} 需要优化 (成功率: {:.1}%)",
                        skill.name,
                        success_rate * 100.0
                    );
                } else {
                    skill.importance = (skill.importance * 1.1).min(1.0);
                }
            }
        }
    }

    /// 获取技能成功率
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

    /// 获取需要优化的技能
    pub fn get_skills_needing_optimization(&self) -> Vec<&CrystalSkill> {
        self.skills
            .values()
            .filter(|s| {
                let total = s.success_count + s.failure_count;
                if total >= self.min_feedback_for_optimize {
                    let success_rate = s.success_count as f64 / total as f64;
                    success_rate < self.optimization_threshold
                } else {
                    false
                }
            })
            .collect()
    }

    /// 获取高价值技能
    pub fn get_high_value_skills(&self) -> Vec<&CrystalSkill> {
        self.skills
            .values()
            .filter(|s| s.importance > 0.7)
            .collect()
    }

    /// 获取统计
    pub fn get_stats(&self) -> UnifiedLearningStats {
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

        UnifiedLearningStats {
            total_skills,
            total_feedbacks,
            avg_success_rate,
            strategies_count: self.strategies.len(),
        }
    }
}

/// 统一学习统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedLearningStats {
    pub total_skills: usize,
    pub total_feedbacks: usize,
    pub avg_success_rate: f64,
    pub strategies_count: usize,
}
