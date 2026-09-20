//! 闭环学习引擎 — 基于 Hermes 闭环学习模式
//!
//! 流程: 吸收经验 → 创建技能 → 使用技能 → 收集反馈 → 分析反馈 → 优化技能
//!
//! 与 experience-tree 集成: 吸收的经验自动沉淀为可复用技能,
//! 技能使用反馈驱动版本迭代, 形成自进化闭环。

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 学习技能 — 从经验中提炼的可复用能力单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_used: Option<i64>,
    pub version: u32,
    pub source_experience_ids: Vec<String>,
    pub tags: Vec<String>,
}

/// 学习反馈 — 技能使用后的结果记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningFeedback {
    pub skill_id: String,
    pub success: bool,
    pub context: String,
    pub details: String,
    pub timestamp: i64,
}

/// 技能优化建议 — 基于反馈分析生成
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOptimization {
    pub skill_id: String,
    pub old_version: u32,
    pub new_version: u32,
    pub reason: String,
    pub success_rate_before: f64,
}

/// 闭环学习引擎
///
/// 核心循环:
/// 1. `absorb_experience` — 吸收经验, 自动创建或更新技能
/// 2. `use_skill` — 标记技能被使用
/// 3. `record_feedback` — 记录使用反馈, 触发优化分析
/// 4. `get_optimizations` — 获取待执行的优化建议
/// 5. `apply_optimization` — 应用优化, 技能版本+1
pub struct LearningLoop {
    skills: HashMap<String, LearningSkill>,
    feedbacks: Vec<LearningFeedback>,
    optimizations: Vec<SkillOptimization>,
    /// 成功率低于此阈值时触发优化 (默认 0.7)
    optimize_threshold: f64,
    /// 最小反馈数才触发优化判断 (避免小样本偏差)
    min_feedback_for_optimize: u32,
}

impl Default for LearningLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningLoop {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
            feedbacks: Vec::new(),
            optimizations: Vec::new(),
            optimize_threshold: 0.7,
            min_feedback_for_optimize: 5,
        }
    }

    /// 吸收经验 → 创建或更新技能
    ///
    /// 如果同名技能已存在, 合并 source_experience_ids;
    /// 否则创建新技能 (version=1)。
    pub fn absorb_experience(
        &mut self,
        skill_id: String,
        name: String,
        description: String,
        experience_id: String,
        tags: Vec<String>,
    ) -> &LearningSkill {
        if let Some(skill) = self.skills.get_mut(&skill_id) {
            if !skill.source_experience_ids.contains(&experience_id) {
                skill.source_experience_ids.push(experience_id);
            }
            for tag in &tags {
                if !skill.tags.contains(tag) {
                    skill.tags.push(tag.clone());
                }
            }
        } else {
            let skill = LearningSkill {
                id: skill_id.clone(),
                name,
                description,
                success_count: 0,
                failure_count: 0,
                last_used: None,
                version: 1,
                source_experience_ids: vec![experience_id],
                tags,
            };
            self.skills.insert(skill_id.clone(), skill);
        }
        &self.skills[&skill_id]
    }

    /// 使用技能 — 标记使用时间
    pub fn use_skill(&mut self, skill_id: &str) -> Option<&LearningSkill> {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.last_used = Some(chrono::Utc::now().timestamp());
            Some(skill)
        } else {
            None
        }
    }

    /// 记录反馈 — 更新统计并触发优化分析
    pub fn record_feedback(&mut self, feedback: LearningFeedback) {
        if let Some(skill) = self.skills.get_mut(&feedback.skill_id) {
            if feedback.success {
                skill.success_count += 1;
            } else {
                skill.failure_count += 1;
            }
        }
        self.feedbacks.push(feedback);
        self.analyze_and_optimize();
    }

    /// 分析反馈 → 生成优化建议
    fn analyze_and_optimize(&mut self) {
        let skill_ids: Vec<String> = self.skills.keys().cloned().collect();
        for skill_id in skill_ids {
            let (total, success_count) = {
                let skill = &self.skills[&skill_id];
                let total = skill.success_count + skill.failure_count;
                (total, skill.success_count)
            };

            if total < self.min_feedback_for_optimize {
                continue;
            }

            let success_rate = success_count as f64 / total as f64;
            if success_rate < self.optimize_threshold {
                let old_version = self.skills[&skill_id].version;
                // 避免重复生成同版本优化建议
                let already_pending = self.optimizations.iter().any(|o| {
                    o.skill_id == skill_id && o.new_version == old_version + 1
                });
                if !already_pending {
                    self.optimizations.push(SkillOptimization {
                        skill_id: skill_id.clone(),
                        old_version,
                        new_version: old_version + 1,
                        reason: format!(
                            "成功率 {:.1}% 低于阈值 {:.1}%, 需要优化",
                            success_rate * 100.0,
                            self.optimize_threshold * 100.0,
                        ),
                        success_rate_before: success_rate,
                    });
                }
            }
        }
    }

    /// 获取待执行的优化建议
    pub fn get_optimizations(&self) -> &[SkillOptimization] {
        &self.optimizations
    }

    /// 应用优化 — 技能版本+1, 移除对应优化建议
    pub fn apply_optimization(&mut self, skill_id: &str) -> Option<SkillOptimization> {
        let idx = self.optimizations.iter().position(|o| o.skill_id == skill_id)?;
        let opt = self.optimizations.remove(idx);
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.version = opt.new_version;
        }
        Some(opt)
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

    /// 获取所有技能
    pub fn skills(&self) -> &HashMap<String, LearningSkill> {
        &self.skills
    }

    /// 获取所有反馈
    pub fn feedbacks(&self) -> &[LearningFeedback] {
        &self.feedbacks
    }

    /// 按成功率排序获取技能 (升序, 低的成功率排前面, 方便识别需优化的)
    pub fn skills_by_success_rate(&self) -> Vec<(&str, f64)> {
        let mut rates: Vec<(&str, f64)> = self
            .skills
            .iter()
            .map(|(id, skill)| {
                let total = skill.success_count + skill.failure_count;
                let rate = if total == 0 {
                    0.0
                } else {
                    skill.success_count as f64 / total as f64
                };
                (id.as_str(), rate)
            })
            .collect();
        rates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        rates
    }

    /// 获取统计摘要
    pub fn summary(&self) -> LearningLoopSummary {
        let total_skills = self.skills.len() as u64;
        let total_feedbacks = self.feedbacks.len() as u64;
        let total_success: u64 = self.feedbacks.iter().filter(|f| f.success).count() as u64;
        let pending_optimizations = self.optimizations.len() as u64;
        let overall_success_rate = if total_feedbacks == 0 {
            0.0
        } else {
            total_success as f64 / total_feedbacks as f64
        };
        LearningLoopSummary {
            total_skills,
            total_feedbacks,
            overall_success_rate,
            pending_optimizations,
        }
    }
}

/// 闭环学习统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningLoopSummary {
    pub total_skills: u64,
    pub total_feedbacks: u64,
    pub overall_success_rate: f64,
    pub pending_optimizations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absorb_and_use_skill() {
        let mut loop_engine = LearningLoop::new();
        let skill = loop_engine.absorb_experience(
            "skill-1".into(),
            "测试技能".into(),
            "用于测试".into(),
            "exp-1".into(),
            vec!["test".into()],
        );
        assert_eq!(skill.version, 1);
        assert_eq!(skill.source_experience_ids.len(), 1);

        let used = loop_engine.use_skill("skill-1");
        assert!(used.is_some());
        assert!(used.unwrap().last_used.is_some());
    }

    #[test]
    fn test_feedback_triggers_optimization() {
        let mut loop_engine = LearningLoop::new();
        loop_engine.absorb_experience(
            "skill-1".into(),
            "弱技能".into(),
            "经常失败".into(),
            "exp-1".into(),
            vec![],
        );

        // 记录 6 次反馈, 2 成功 4 失败 → 成功率 33%
        for i in 0..6 {
            loop_engine.record_feedback(LearningFeedback {
                skill_id: "skill-1".into(),
                success: i < 2,
                context: "test".into(),
                details: String::new(),
                timestamp: chrono::Utc::now().timestamp(),
            });
        }

        assert!(!loop_engine.get_optimizations().is_empty());
        let opt = &loop_engine.get_optimizations()[0];
        assert_eq!(opt.new_version, 2);
    }

    #[test]
    fn test_apply_optimization() {
        let mut loop_engine = LearningLoop::new();
        loop_engine.absorb_experience(
            "skill-1".into(),
            "弱技能".into(),
            "经常失败".into(),
            "exp-1".into(),
            vec![],
        );

        for i in 0..6 {
            loop_engine.record_feedback(LearningFeedback {
                skill_id: "skill-1".into(),
                success: i < 2,
                context: "test".into(),
                details: String::new(),
                timestamp: chrono::Utc::now().timestamp(),
            });
        }

        let opt = loop_engine.apply_optimization("skill-1");
        assert!(opt.is_some());
        assert_eq!(opt.unwrap().new_version, 2);
        assert_eq!(loop_engine.skills["skill-1"].version, 2);
        assert!(loop_engine.get_optimizations().is_empty());
    }

    #[test]
    fn test_summary() {
        let mut loop_engine = LearningLoop::new();
        loop_engine.absorb_experience(
            "s1".into(), "A".into(), "desc".into(), "e1".into(), vec![],
        );
        loop_engine.record_feedback(LearningFeedback {
            skill_id: "s1".into(),
            success: true,
            context: "".into(),
            details: "".into(),
            timestamp: 0,
        });
        let s = loop_engine.summary();
        assert_eq!(s.total_skills, 1);
        assert_eq!(s.total_feedbacks, 1);
        assert!((s.overall_success_rate - 1.0).abs() < f64::EPSILON);
    }
}
