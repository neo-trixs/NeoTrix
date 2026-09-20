//! 统一记忆层 (UnifiedMemorySystem)
//!
//! 按 FUSION-ARCHITECTURE-v4 设计
//! 整合 L0(原始经验) → L1(原子概念) → L2(晶体知识) → L3(晶体人格) 蒸馏管线
//! 与现有 `crystal_memory_distillation` 模块共享类型，不重复定义

use super::crystal_memory_distillation::{
    AtomicConcept, CrystalDistillationLevel, CrystalKnowledge, CrystalPersonality, RawExperience,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆层级标签
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryLevel {
    /// L0: 原始经验
    Experience,
    /// L1: 原子概念
    Concept,
    /// L2: 晶体知识
    Knowledge,
    /// L3: 晶体人格
    Personality,
}

impl From<CrystalDistillationLevel> for MemoryLevel {
    fn from(level: CrystalDistillationLevel) -> Self {
        match level {
            CrystalDistillationLevel::L0RawExperience => MemoryLevel::Experience,
            CrystalDistillationLevel::L1AtomicConcepts => MemoryLevel::Concept,
            CrystalDistillationLevel::L2CrystalKnowledge => MemoryLevel::Knowledge,
            CrystalDistillationLevel::L3CrystalPersonality => MemoryLevel::Personality,
        }
    }
}

/// 时序记忆 — 支持有效时间区间的四元组记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalMemory {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
    pub confidence: f64,
    pub level: MemoryLevel,
}

/// 记忆协调器 — 跨层级查询与压缩
pub struct MemoryCoordination {
    cross_level_queries: Vec<String>,
    compaction_threshold: usize,
}

impl MemoryCoordination {
    pub fn new() -> Self {
        Self {
            cross_level_queries: Vec::new(),
            compaction_threshold: 1000,
        }
    }

    pub fn cross_level_query(&mut self, query: &str) -> Vec<String> {
        self.cross_level_queries.push(query.to_string());
        Vec::new()
    }

    pub fn compact(&self, memories: &mut Vec<TemporalMemory>) {
        if memories.len() > self.compaction_threshold {
            memories.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            memories.truncate(self.compaction_threshold);
        }
    }
}

/// 统一记忆系统 — 顶层协调器
pub struct UnifiedMemorySystem {
    raw_experiences: Vec<RawExperience>,
    atomic_concepts: Vec<AtomicConcept>,
    crystal_knowledge: Vec<CrystalKnowledge>,
    crystal_personality: Option<CrystalPersonality>,
    temporal_memories: Vec<TemporalMemory>,
    coordination: MemoryCoordination,
}

impl UnifiedMemorySystem {
    pub fn new() -> Self {
        Self {
            raw_experiences: Vec::new(),
            atomic_concepts: Vec::new(),
            crystal_knowledge: Vec::new(),
            crystal_personality: None,
            temporal_memories: Vec::new(),
            coordination: MemoryCoordination::new(),
        }
    }

    /// 添加原始经验并触发蒸馏
    pub fn add_raw_experience(&mut self, experience: RawExperience) {
        self.raw_experiences.push(experience);
        self.distill();
    }

    /// 蒸馏 L0 → L1 — 从经验中提取原子概念
    fn distill_l0_to_l1(&mut self) {
        for exp in &self.raw_experiences {
            let words: Vec<&str> = exp.content.split_whitespace().collect();
            for word in words {
                if word.len() > 3 {
                    let concept = AtomicConcept {
                        id: format!("concept_{}", uuid::Uuid::new_v4()),
                        name: word.to_string(),
                        definition: format!("从经验 '{}' 中提取", exp.id),
                        domain: exp.context.clone(),
                        confidence: exp.importance * 0.8,
                        source_ids: vec![exp.id.clone()],
                        first_seen: exp.timestamp,
                        last_seen: exp.timestamp,
                        occurrence_count: 1,
                    };
                    self.atomic_concepts.push(concept);
                }
            }
        }
    }

    /// 蒸馏 L1 → L2 — 按领域聚合为晶体知识
    fn distill_l1_to_l2(&mut self) {
        let mut concepts_by_domain: HashMap<String, Vec<String>> = HashMap::new();

        for concept in &self.atomic_concepts {
            concepts_by_domain
                .entry(concept.domain.clone())
                .or_default()
                .push(concept.name.clone());
        }

        for (domain, concept_names) in concepts_by_domain {
            let knowledge = CrystalKnowledge {
                id: format!("knowledge_{}", uuid::Uuid::new_v4()),
                concept: domain,
                relationships: concept_names,
                context: "从原子概念聚合".to_string(),
                importance: 0.7,
                abstraction_level: 0.6,
                evidence_strength: 0,
                formed_at: 0,
            };
            self.crystal_knowledge.push(knowledge);
        }
    }

    /// 蒸馏 L2 → L3 — 晶体人格涌现
    fn distill_l2_to_l3(&mut self) {
        let traits: Vec<String> = self
            .crystal_knowledge
            .iter()
            .map(|k| k.concept.clone())
            .collect();

        if !traits.is_empty() {
            let personality = CrystalPersonality {
                id: format!("personality_{}", uuid::Uuid::new_v4()),
                traits,
                preferences: Vec::new(),
                behaviors: Vec::new(),
                values: Vec::new(),
                confidence: 0.6,
                distillation_round: 0,
                last_updated: 0,
            };
            self.crystal_personality = Some(personality);
        }
    }

    /// 完整蒸馏流程 L0 → L1 → L2 → L3
    fn distill(&mut self) {
        self.distill_l0_to_l1();
        self.distill_l1_to_l2();
        self.distill_l2_to_l3();
    }

    /// 存储时序记忆
    pub fn store_temporal(&mut self, memory: TemporalMemory) {
        self.temporal_memories.push(memory);
    }

    /// 查询时序记忆
    pub fn query_temporal(&self, query: &str) -> Vec<&TemporalMemory> {
        self.temporal_memories
            .iter()
            .filter(|m| m.subject.contains(query) || m.object.contains(query))
            .collect()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> UnifiedMemoryStats {
        UnifiedMemoryStats {
            l0_count: self.raw_experiences.len(),
            l1_count: self.atomic_concepts.len(),
            l2_count: self.crystal_knowledge.len(),
            l3_count: if self.crystal_personality.is_some() {
                1
            } else {
                0
            },
            temporal_count: self.temporal_memories.len(),
        }
    }
}

/// 统一记忆统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMemoryStats {
    pub l0_count: usize,
    pub l1_count: usize,
    pub l2_count: usize,
    pub l3_count: usize,
    pub temporal_count: usize,
}
