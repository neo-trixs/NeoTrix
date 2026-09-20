//! 晶体核心统一管线模块
//!
//! 融合记忆-知识-技能的统一处理管线
//! 输入 → 晶体记忆存储 → 晶体知识提取 → 晶体技能创建 → 使用反馈 → 优化改进

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 晶体记忆层级
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrystalMemoryLevel {
    /// 经验层: 原始经验
    Experience,
    /// 概念层: 原子概念
    Concept,
    /// 知识层: 晶体知识
    Knowledge,
    /// 人格层: 晶体人格
    Personality,
}

/// 晶体时序事实
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalTemporalFact {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub confidence: f64,
    pub level: CrystalMemoryLevel,
}

/// 晶体技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalPipelineSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub version: u32,
}

/// 晶体核心统一管线
pub struct CrystalUnifiedPipeline {
    memories: HashMap<CrystalMemoryLevel, Vec<CrystalTemporalFact>>,
    skills: HashMap<String, CrystalPipelineSkill>,
}

/// 晶体管线统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalPipelineStats {
    pub experience_count: usize,
    pub concept_count: usize,
    pub knowledge_count: usize,
    pub personality_count: usize,
    pub skill_count: usize,
}

impl CrystalUnifiedPipeline {
    pub fn new() -> Self {
        let mut memories = HashMap::new();
        memories.insert(CrystalMemoryLevel::Experience, Vec::new());
        memories.insert(CrystalMemoryLevel::Concept, Vec::new());
        memories.insert(CrystalMemoryLevel::Knowledge, Vec::new());
        memories.insert(CrystalMemoryLevel::Personality, Vec::new());

        Self {
            memories,
            skills: HashMap::new(),
        }
    }

    /// 存储晶体记忆
    pub fn store_memory(&mut self, level: CrystalMemoryLevel, fact: CrystalTemporalFact) {
        self.memories.entry(level).or_default().push(fact);
    }

    /// 提取晶体知识
    pub fn extract_knowledge(&self) -> Vec<CrystalTemporalFact> {
        let all_facts: Vec<CrystalTemporalFact> =
            self.memories.values().flatten().cloned().collect();

        // 按置信度排序
        let mut sorted_facts = all_facts;
        sorted_facts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        sorted_facts.into_iter().take(10).collect()
    }

    /// 创建晶体技能
    pub fn create_skill(&mut self, id: String, name: String, description: String, domain: String) {
        let skill = CrystalPipelineSkill {
            id: id.clone(),
            name,
            description,
            domain,
            success_count: 0,
            failure_count: 0,
            version: 1,
        };
        self.skills.insert(id, skill);
    }

    /// 使用晶体技能并记录反馈
    pub fn use_skill_with_feedback(&mut self, skill_id: &str, success: bool) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            if success {
                skill.success_count += 1;
            } else {
                skill.failure_count += 1;
            }

            // 触发优化
            self.optimize_skill(skill_id);
        }
    }

    /// 优化晶体技能
    fn optimize_skill(&mut self, skill_id: &str) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            let total = skill.success_count + skill.failure_count;
            if total > 10 {
                let success_rate = skill.success_count as f64 / total as f64;
                if success_rate < 0.7 {
                    skill.version += 1;
                    println!(
                        "[CrystalPipeline] 技能 {} 需要优化 (成功率: {:.1}%)",
                        skill.name,
                        success_rate * 100.0
                    );
                }
            }
        }
    }

    /// 获取晶体统计
    pub fn get_stats(&self) -> CrystalPipelineStats {
        CrystalPipelineStats {
            experience_count: self
                .memories
                .get(&CrystalMemoryLevel::Experience)
                .map_or(0, |v| v.len()),
            concept_count: self
                .memories
                .get(&CrystalMemoryLevel::Concept)
                .map_or(0, |v| v.len()),
            knowledge_count: self
                .memories
                .get(&CrystalMemoryLevel::Knowledge)
                .map_or(0, |v| v.len()),
            personality_count: self
                .memories
                .get(&CrystalMemoryLevel::Personality)
                .map_or(0, |v| v.len()),
            skill_count: self.skills.len(),
        }
    }
}

impl Default for CrystalUnifiedPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystal_pipeline_creation() {
        let pipeline = CrystalUnifiedPipeline::new();
        let stats = pipeline.get_stats();
        assert_eq!(stats.experience_count, 0);
        assert_eq!(stats.skill_count, 0);
    }

    #[test]
    fn test_store_memory() {
        let mut pipeline = CrystalUnifiedPipeline::new();
        let fact = CrystalTemporalFact {
            id: "fact_1".to_string(),
            subject: "Rust".to_string(),
            predicate: "is".to_string(),
            object: "programming language".to_string(),
            valid_from: 0,
            valid_to: None,
            confidence: 0.9,
            level: CrystalMemoryLevel::Concept,
        };

        pipeline.store_memory(CrystalMemoryLevel::Concept, fact);
        let stats = pipeline.get_stats();
        assert_eq!(stats.concept_count, 1);
    }

    #[test]
    fn test_create_skill() {
        let mut pipeline = CrystalUnifiedPipeline::new();
        pipeline.create_skill(
            "skill_1".to_string(),
            "Test Skill".to_string(),
            "A test skill".to_string(),
            "testing".to_string(),
        );
        let stats = pipeline.get_stats();
        assert_eq!(stats.skill_count, 1);
    }

    #[test]
    fn test_use_skill_feedback() {
        let mut pipeline = CrystalUnifiedPipeline::new();
        pipeline.create_skill(
            "skill_1".to_string(),
            "Test Skill".to_string(),
            "A test skill".to_string(),
            "testing".to_string(),
        );

        // 使用技能成功
        pipeline.use_skill_with_feedback("skill_1", true);

        // 验证技能统计
        let skill = pipeline.skills.get("skill_1").unwrap();
        assert_eq!(skill.success_count, 1);
        assert_eq!(skill.failure_count, 0);
    }

    #[test]
    fn test_extract_knowledge() {
        let mut pipeline = CrystalUnifiedPipeline::new();

        // 添加不同置信度的事实
        for i in 0..5 {
            let fact = CrystalTemporalFact {
                id: format!("fact_{}", i),
                subject: format!("subject_{}", i),
                predicate: "is".to_string(),
                object: format!("object_{}", i),
                valid_from: 0,
                valid_to: None,
                confidence: i as f64 * 0.2,
                level: CrystalMemoryLevel::Knowledge,
            };
            pipeline.store_memory(CrystalMemoryLevel::Knowledge, fact);
        }

        let knowledge = pipeline.extract_knowledge();
        assert!(knowledge.len() <= 10);
        // 验证按置信度降序排序
        if knowledge.len() >= 2 {
            assert!(knowledge[0].confidence >= knowledge[1].confidence);
        }
    }
}
