//! 统一记忆-学习管线
//! 融合 Hermes + Mem0 + Graphiti 优势

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆层级 (Mem0)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryLevel {
    User,
    Session,
    Agent,
}

/// 时序事实 (Graphiti)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFact {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub confidence: f64,
}

/// 学习技能 (Hermes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub version: u32,
}

/// 统一记忆-学习管线
pub struct UnifiedPipeline {
    memories: HashMap<MemoryLevel, Vec<TemporalFact>>,
    skills: HashMap<String, LearningSkill>,
}

impl UnifiedPipeline {
    pub fn new() -> Self {
        let mut memories = HashMap::new();
        memories.insert(MemoryLevel::User, Vec::new());
        memories.insert(MemoryLevel::Session, Vec::new());
        memories.insert(MemoryLevel::Agent, Vec::new());

        Self {
            memories,
            skills: HashMap::new(),
        }
    }

    /// 存储记忆 (Mem0)
    pub fn store_memory(&mut self, level: MemoryLevel, fact: TemporalFact) {
        self.memories.entry(level).or_default().push(fact);
    }

    /// 查询记忆 (Mem0) - 按关键词过滤
    pub fn query_memories(&self, keyword: &str) -> Vec<&TemporalFact> {
        self.memories
            .values()
            .flatten()
            .filter(|f| {
                f.subject.contains(keyword)
                    || f.predicate.contains(keyword)
                    || f.object.contains(keyword)
            })
            .collect()
    }

    /// 提取知识 (Graphiti) - 按置信度排序返回 top N
    pub fn extract_knowledge(&self, top_n: usize) -> Vec<TemporalFact> {
        let mut all_facts: Vec<TemporalFact> = self.memories.values().flatten().cloned().collect();

        all_facts.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        all_facts.into_iter().take(top_n).collect()
    }

    /// 创建技能 (Hermes)
    pub fn create_skill(&mut self, id: String, name: String, description: String) {
        let skill = LearningSkill {
            id: id.clone(),
            name,
            description,
            success_count: 0,
            failure_count: 0,
            version: 1,
        };
        self.skills.insert(id, skill);
    }

    /// 使用技能并记录反馈 (Hermes)
    pub fn use_skill_with_feedback(&mut self, skill_id: &str, success: bool) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            if success {
                skill.success_count += 1;
            } else {
                skill.failure_count += 1;
            }

            self.optimize_skill(skill_id);
        }
    }

    /// 优化技能 (Hermes)
    fn optimize_skill(&mut self, skill_id: &str) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            let total = skill.success_count + skill.failure_count;
            if total >= 10 {
                let success_rate = skill.success_count as f64 / total as f64;
                if success_rate < 0.7 {
                    skill.version += 1;
                }
            }
        }
    }

    /// 获取技能优化建议
    pub fn get_optimization_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        for skill in self.skills.values() {
            let total = skill.success_count + skill.failure_count;
            if total >= 10 {
                let success_rate = skill.success_count as f64 / total as f64;
                if success_rate < 0.7 {
                    suggestions.push(format!(
                        "技能 '{}' (v{}) 成功率 {:.1}%，建议优化",
                        skill.name,
                        skill.version,
                        success_rate * 100.0
                    ));
                }
            }
        }

        suggestions
    }

    /// 获取统计
    pub fn get_stats(&self) -> PipelineStats {
        PipelineStats {
            user_memories: self.memories.get(&MemoryLevel::User).map_or(0, |v| v.len()),
            session_memories: self.memories.get(&MemoryLevel::Session).map_or(0, |v| v.len()),
            agent_memories: self.memories.get(&MemoryLevel::Agent).map_or(0, |v| v.len()),
            skill_count: self.skills.len(),
        }
    }
}

impl Default for UnifiedPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// 管线统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    pub user_memories: usize,
    pub session_memories: usize,
    pub agent_memories: usize,
    pub skill_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_pipeline_basic() {
        let mut pipeline = UnifiedPipeline::new();

        // Store memories
        pipeline.store_memory(
            MemoryLevel::User,
            TemporalFact {
                id: "f1".into(),
                subject: "Alice".into(),
                predicate: "likes".into(),
                object: "Rust".into(),
                valid_from: 1000,
                valid_to: None,
                confidence: 0.9,
            },
        );

        // Query
        let results = pipeline.query_memories("Alice");
        assert_eq!(results.len(), 1);

        // Stats
        let stats = pipeline.get_stats();
        assert_eq!(stats.user_memories, 1);
    }

    #[test]
    fn test_skill_optimization() {
        let mut pipeline = UnifiedPipeline::new();
        pipeline.create_skill("s1".into(), "TestSkill".into(), "desc".into());

        for _ in 0..7 {
            pipeline.use_skill_with_feedback("s1", false);
        }
        for _ in 0..3 {
            pipeline.use_skill_with_feedback("s1", true);
        }

        let skill = pipeline.skills.get("s1").unwrap();
        assert_eq!(skill.version, 2);
    }
}
