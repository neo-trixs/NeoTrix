//! 晶体核心记忆蒸馏模块 (Crystal Memory Distillation)
//!
//! 从原始经验到晶体人格的蒸馏管线
//! 层级蒸馏: L0(原始经验) → L1(原子概念) → L2(晶体知识) → L3(晶体人格)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 晶体蒸馏层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrystalDistillationLevel {
    /// L0: 原始经验 — 未加工的感知流
    L0RawExperience,
    /// L1: 原子概念 — 从经验中提取的最小语义单元
    L1AtomicConcepts,
    /// L2: 晶体知识 — 概念之间的关系网络
    L2CrystalKnowledge,
    /// L3: 晶体人格 — 长期稳定的身份与偏好
    L3CrystalPersonality,
}

/// 原始经验 (L0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExperience {
    /// 经验ID
    pub id: String,
    /// 原始内容
    pub content: String,
    /// 感知上下文 (task_type / domain / source)
    pub context: String,
    /// 感知时间戳 (秒)
    pub timestamp: i64,
    /// 重要性评分 [0.0, 1.0]
    pub importance: f64,
    /// 情绪标签 (可选)
    pub emotion_tag: Option<String>,
    /// 原始元数据
    pub metadata: HashMap<String, String>,
}

/// 原子概念 (L1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicConcept {
    /// 概念ID
    pub id: String,
    /// 概念名称
    pub name: String,
    /// 概念定义 (一句话)
    pub definition: String,
    /// 所属领域
    pub domain: String,
    /// 提取置信度
    pub confidence: f64,
    /// 来源经验ID列表
    pub source_ids: Vec<String>,
    /// 首次出现时间
    pub first_seen: i64,
    /// 最近出现时间
    pub last_seen: i64,
    /// 出现次数
    pub occurrence_count: u32,
}

/// 晶体知识 (L2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalKnowledge {
    /// 知识ID
    pub id: String,
    /// 核心概念名称
    pub concept: String,
    /// 关联概念ID列表
    pub relationships: Vec<String>,
    /// 应用上下文
    pub context: String,
    /// 重要性评分
    pub importance: f64,
    /// 抽象层级 [0.0=具体, 1.0=高度抽象]
    pub abstraction_level: f64,
    /// 证据强度 (支持该知识的经验数)
    pub evidence_strength: u32,
    /// 形成时间
    pub formed_at: i64,
}

/// 晶体人格 (L3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalPersonality {
    /// 人格ID
    pub id: String,
    /// 核心特质 (如 "analytical", "creative")
    pub traits: Vec<String>,
    /// 偏好 (如 "prefers concise output")
    pub preferences: Vec<String>,
    /// 行为模式 (如 "always verifies before claiming")
    pub behaviors: Vec<String>,
    /// 核心价值观 (如 "accuracy over speed")
    pub values: Vec<String>,
    /// 人格置信度
    pub confidence: f64,
    /// 蒸馏轮次
    pub distillation_round: u32,
    /// 最后更新时间
    pub last_updated: i64,
}

/// 晶体蒸馏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalDistillationConfig {
    /// L0→L1: 最小经验长度 (字符)
    pub min_experience_length: usize,
    /// L0→L1: 最小重要性阈值
    pub min_importance_threshold: f64,
    /// L1→L2: 概念聚合最小共现次数
    pub min_cooccurrence: u32,
    /// L1→L2: 概念聚类领域相似度阈值
    pub domain_similarity_threshold: f64,
    /// L2→L3: 人格形成所需最小知识数
    pub min_knowledge_for_personality: usize,
    /// L3: 人格更新衰减因子
    pub personality_decay_factor: f64,
    /// 各层级最大容量
    pub max_l0: usize,
    pub max_l1: usize,
    pub max_l2: usize,
}

impl Default for CrystalDistillationConfig {
    fn default() -> Self {
        Self {
            min_experience_length: 10,
            min_importance_threshold: 0.3,
            min_cooccurrence: 2,
            domain_similarity_threshold: 0.5,
            min_knowledge_for_personality: 3,
            personality_decay_factor: 0.95,
            max_l0: 1000,
            max_l1: 500,
            max_l2: 200,
        }
    }
}

/// 蒸馏统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalDistillationStats {
    /// L0 原始经验数
    pub l0_count: usize,
    /// L1 原子概念数
    pub l1_count: usize,
    /// L2 晶体知识数
    pub l2_count: usize,
    /// L3 晶体人格是否存在
    pub l3_exists: bool,
    /// 总蒸馏次数
    pub total_distillations: u64,
    /// 平均压缩率 (L0字符 → L1概念数)
    pub avg_compression_ratio: f64,
    /// 人格特质数
    pub personality_trait_count: usize,
}

impl std::fmt::Display for CrystalDistillationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        CrystalMemoryDistillation 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "L0 原始经验:     {}", self.l0_count)?;
        writeln!(f, "L1 原子概念:     {}", self.l1_count)?;
        writeln!(f, "L2 晶体知识:     {}", self.l2_count)?;
        writeln!(
            f,
            "L3 晶体人格:     {}",
            if self.l3_exists {
                "已形成"
            } else {
                "未形成"
            }
        )?;
        writeln!(f, "总蒸馏次数:      {}", self.total_distillations)?;
        writeln!(
            f,
            "平均压缩率:      {:.2}%",
            self.avg_compression_ratio * 100.0
        )?;
        writeln!(f, "人格特质数:      {}", self.personality_trait_count)?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

/// 晶体记忆蒸馏管线
pub struct CrystalDistillationPipeline {
    /// L0: 原始经验缓冲
    raw_experiences: Vec<RawExperience>,
    /// L1: 原子概念库
    atomic_concepts: Vec<AtomicConcept>,
    /// L2: 晶体知识网络
    crystal_knowledge: Vec<CrystalKnowledge>,
    /// L3: 晶体人格
    crystal_personality: Option<CrystalPersonality>,
    /// 蒸馏配置
    config: CrystalDistillationConfig,
    /// 统计
    total_distillations: u64,
}

impl CrystalDistillationPipeline {
    /// 创建新的蒸馏管线
    pub fn new(config: CrystalDistillationConfig) -> Self {
        Self {
            raw_experiences: Vec::new(),
            atomic_concepts: Vec::new(),
            crystal_knowledge: Vec::new(),
            crystal_personality: None,
            config,
            total_distillations: 0,
        }
    }

    /// 使用默认配置创建
    pub fn default_pipeline() -> Self {
        Self::new(CrystalDistillationConfig::default())
    }

    /// 添加原始经验并触发蒸馏
    pub fn ingest_experience(&mut self, experience: RawExperience) {
        // 过滤低质量经验
        if experience.content.len() < self.config.min_experience_length {
            return;
        }
        if experience.importance < self.config.min_importance_threshold {
            return;
        }

        self.raw_experiences.push(experience);
        self.trim_l0();
        self.run_distillation();
    }

    /// 批量摄入经验
    pub fn ingest_batch(&mut self, experiences: Vec<RawExperience>) {
        for exp in experiences {
            if exp.content.len() >= self.config.min_experience_length
                && exp.importance >= self.config.min_importance_threshold
            {
                self.raw_experiences.push(exp);
            }
        }
        self.trim_l0();
        self.run_distillation();
    }

    /// 获取当前蒸馏层级
    pub fn current_level(&self) -> CrystalDistillationLevel {
        if self.crystal_personality.is_some() {
            CrystalDistillationLevel::L3CrystalPersonality
        } else if !self.crystal_knowledge.is_empty() {
            CrystalDistillationLevel::L2CrystalKnowledge
        } else if !self.atomic_concepts.is_empty() {
            CrystalDistillationLevel::L1AtomicConcepts
        } else {
            CrystalDistillationLevel::L0RawExperience
        }
    }

    /// 获取蒸馏统计
    pub fn stats(&self) -> CrystalDistillationStats {
        let avg_compression = if !self.raw_experiences.is_empty() {
            let total_chars: usize = self.raw_experiences.iter().map(|e| e.content.len()).sum();
            if total_chars > 0 {
                self.atomic_concepts.len() as f64 / total_chars as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        let trait_count = self
            .crystal_personality
            .as_ref()
            .map(|p| p.traits.len() + p.preferences.len() + p.behaviors.len() + p.values.len())
            .unwrap_or(0);

        CrystalDistillationStats {
            l0_count: self.raw_experiences.len(),
            l1_count: self.atomic_concepts.len(),
            l2_count: self.crystal_knowledge.len(),
            l3_exists: self.crystal_personality.is_some(),
            total_distillations: self.total_distillations,
            avg_compression_ratio: avg_compression,
            personality_trait_count: trait_count,
        }
    }

    /// 获取原子概念 (只读)
    pub fn atomic_concepts(&self) -> &[AtomicConcept] {
        &self.atomic_concepts
    }

    /// 获取晶体知识 (只读)
    pub fn crystal_knowledge(&self) -> &[CrystalKnowledge] {
        &self.crystal_knowledge
    }

    /// 获取晶体人格 (只读)
    pub fn crystal_personality(&self) -> Option<&CrystalPersonality> {
        self.crystal_personality.as_ref()
    }

    // ─── 内部蒸馏逻辑 ───────────────────────────────────

    /// 完整蒸馏流程
    fn run_distillation(&mut self) {
        self.distill_l0_to_l1();
        self.distill_l1_to_l2();
        self.distill_l2_to_l3();
        self.total_distillations += 1;
    }

    /// L0 → L1: 从原始经验提取原子概念
    fn distill_l0_to_l1(&mut self) {
        let new_experiences: Vec<RawExperience> = self.raw_experiences.drain(..).collect();

        for exp in &new_experiences {
            let concepts = self.extract_concepts_from_experience(exp);
            for concept in concepts {
                // 检查是否已存在同名概念，若存在则合并
                if let Some(existing) = self
                    .atomic_concepts
                    .iter_mut()
                    .find(|c| c.name == concept.name && c.domain == concept.domain)
                {
                    existing.occurrence_count += 1;
                    existing.last_seen = exp.timestamp;
                    existing.source_ids.push(exp.id.clone());
                    existing.confidence = (existing.confidence + concept.confidence) / 2.0;
                } else {
                    self.atomic_concepts.push(concept);
                }
            }
        }

        self.trim_l1();
    }

    /// 从单条经验提取原子概念
    fn extract_concepts_from_experience(&self, exp: &RawExperience) -> Vec<AtomicConcept> {
        let mut concepts = Vec::new();

        // 策略1: 基于分词提取关键词
        let words: Vec<&str> = exp.content.split_whitespace().collect();
        let min_word_len = 3;

        for word in &words {
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();

            if cleaned.len() >= min_word_len {
                let concept = AtomicConcept {
                    id: format!("ac_{}", uuid::Uuid::new_v4()),
                    name: cleaned.to_lowercase(),
                    definition: format!("从经验 '{}' 中提取的原子概念", exp.id),
                    domain: exp.context.clone(),
                    confidence: exp.importance * 0.8,
                    source_ids: vec![exp.id.clone()],
                    first_seen: exp.timestamp,
                    last_seen: exp.timestamp,
                    occurrence_count: 1,
                };
                concepts.push(concept);
            }
        }

        // 策略2: 基于标点分割提取短语
        let phrases: Vec<&str> = exp
            .content
            .split(|c| c == ',' || c == ';' || c == '。' || c == '，')
            .collect();
        for phrase in phrases {
            let trimmed = phrase.trim();
            if trimmed.len() >= min_word_len && trimmed.len() <= 50 {
                let concept = AtomicConcept {
                    id: format!("ac_{}", uuid::Uuid::new_v4()),
                    name: trimmed.to_lowercase(),
                    definition: format!("从经验 '{}' 中提取的短语概念", exp.id),
                    domain: exp.context.clone(),
                    confidence: exp.importance * 0.6,
                    source_ids: vec![exp.id.clone()],
                    first_seen: exp.timestamp,
                    last_seen: exp.timestamp,
                    occurrence_count: 1,
                };
                concepts.push(concept);
            }
        }

        concepts
    }

    /// L1 → L2: 聚合原子概念为晶体知识
    fn distill_l1_to_l2(&mut self) {
        if self.atomic_concepts.len() < 2 {
            return;
        }

        // 按领域分组
        let mut domain_groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, concept) in self.atomic_concepts.iter().enumerate() {
            domain_groups
                .entry(concept.domain.clone())
                .or_default()
                .push(idx);
        }

        for (domain, indices) in &domain_groups {
            if indices.len() < self.config.min_cooccurrence as usize {
                continue;
            }

            let concept_names: Vec<String> = indices
                .iter()
                .map(|&i| self.atomic_concepts[i].name.clone())
                .collect();

            let avg_confidence: f64 = indices
                .iter()
                .map(|&i| self.atomic_concepts[i].confidence)
                .sum::<f64>()
                / indices.len() as f64;

            let total_evidence: u32 = indices
                .iter()
                .map(|&i| self.atomic_concepts[i].occurrence_count)
                .sum();

            // 计算抽象层级: 概念数越多越抽象
            let abstraction = (concept_names.len() as f64 / 10.0).min(1.0);

            let knowledge = CrystalKnowledge {
                id: format!("ck_{}", uuid::Uuid::new_v4()),
                concept: domain.clone(),
                relationships: concept_names,
                context: format!("从 {} 个原子概念聚合", indices.len()),
                importance: avg_confidence,
                abstraction_level: abstraction,
                evidence_strength: total_evidence,
                formed_at: chrono::Utc::now().timestamp(),
            };

            // 检查是否已有同领域知识
            if let Some(existing) = self
                .crystal_knowledge
                .iter_mut()
                .find(|k| k.concept == *domain)
            {
                existing.evidence_strength += knowledge.evidence_strength;
                existing.importance = (existing.importance + knowledge.importance) / 2.0;
                // 合并关系
                for rel in &knowledge.relationships {
                    if !existing.relationships.contains(rel) {
                        existing.relationships.push(rel.clone());
                    }
                }
            } else {
                self.crystal_knowledge.push(knowledge);
            }
        }

        self.trim_l2();
    }

    /// L2 → L3: 从晶体知识形成晶体人格
    fn distill_l2_to_l3(&mut self) {
        if self.crystal_knowledge.len() < self.config.min_knowledge_for_personality {
            return;
        }

        // 提取特质: 高重要性知识的概念
        let mut traits: Vec<String> = self
            .crystal_knowledge
            .iter()
            .filter(|k| k.importance >= 0.6)
            .map(|k| k.concept.clone())
            .collect();
        traits.sort();
        traits.dedup();

        // 提取偏好: 高抽象层级的知识
        let preferences: Vec<String> = self
            .crystal_knowledge
            .iter()
            .filter(|k| k.abstraction_level >= 0.5)
            .map(|k| format!("prefers_{}", k.concept))
            .collect();

        // 提取行为模式: 高证据强度的知识
        let behaviors: Vec<String> = self
            .crystal_knowledge
            .iter()
            .filter(|k| k.evidence_strength >= 3)
            .map(|k| format!("systematically_{}", k.concept))
            .collect();

        // 提取价值观: 最高重要性的知识
        let mut values: Vec<String> = self
            .crystal_knowledge
            .iter()
            .filter(|k| k.importance >= 0.8)
            .map(|k| k.concept.clone())
            .collect();
        values.sort();
        values.dedup();

        // 计算人格置信度
        let confidence = if !self.crystal_knowledge.is_empty() {
            let avg_importance: f64 = self
                .crystal_knowledge
                .iter()
                .map(|k| k.importance)
                .sum::<f64>()
                / self.crystal_knowledge.len() as f64;
            let avg_evidence: f64 = self
                .crystal_knowledge
                .iter()
                .map(|k| k.evidence_strength as f64)
                .sum::<f64>()
                / self.crystal_knowledge.len() as f64;
            (avg_importance * 0.6 + (avg_evidence / 10.0).min(1.0) * 0.4).min(1.0)
        } else {
            0.0
        };

        let round = self
            .crystal_personality
            .as_ref()
            .map(|p| p.distillation_round + 1)
            .unwrap_or(1);

        let personality = CrystalPersonality {
            id: format!("cp_{}", uuid::Uuid::new_v4()),
            traits,
            preferences,
            behaviors,
            values,
            confidence,
            distillation_round: round,
            last_updated: chrono::Utc::now().timestamp(),
        };

        self.crystal_personality = Some(personality);
    }

    // ─── 容量管理 ───────────────────────────────────────

    fn trim_l0(&mut self) {
        while self.raw_experiences.len() > self.config.max_l0 {
            // 移除最旧且最低重要性的经验
            let min_idx = self
                .raw_experiences
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.importance
                        .partial_cmp(&b.1.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.1.timestamp.cmp(&b.1.timestamp))
                })
                .map(|(i, _)| i);
            if let Some(idx) = min_idx {
                self.raw_experiences.remove(idx);
            } else {
                break;
            }
        }
    }

    fn trim_l1(&mut self) {
        while self.atomic_concepts.len() > self.config.max_l1 {
            // 移除最低置信度且最少出现的概念
            let min_idx = self
                .atomic_concepts
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.confidence
                        .partial_cmp(&b.1.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.1.occurrence_count.cmp(&b.1.occurrence_count))
                })
                .map(|(i, _)| i);
            if let Some(idx) = min_idx {
                self.atomic_concepts.remove(idx);
            } else {
                break;
            }
        }
    }

    fn trim_l2(&mut self) {
        while self.crystal_knowledge.len() > self.config.max_l2 {
            let min_idx = self
                .crystal_knowledge
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.importance
                        .partial_cmp(&b.1.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i);
            if let Some(idx) = min_idx {
                self.crystal_knowledge.remove(idx);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_experience(id: &str, content: &str, importance: f64) -> RawExperience {
        RawExperience {
            id: id.to_string(),
            content: content.to_string(),
            context: "test_domain".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            importance,
            emotion_tag: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_pipeline_creation() {
        let pipeline = CrystalDistillationPipeline::default_pipeline();
        assert_eq!(
            pipeline.current_level(),
            CrystalDistillationLevel::L0RawExperience
        );
        let stats = pipeline.stats();
        assert_eq!(stats.l0_count, 0);
        assert_eq!(stats.l1_count, 0);
    }

    #[test]
    fn test_low_importance_rejected() {
        let mut pipeline = CrystalDistillationPipeline::default_pipeline();
        let exp = make_experience("e1", "short content that is long enough", 0.1);
        pipeline.ingest_experience(exp);
        assert_eq!(pipeline.stats().l0_count, 0);
    }

    #[test]
    fn test_l0_to_l1_extraction() {
        let mut pipeline = CrystalDistillationPipeline::default_pipeline();
        let exp = make_experience("e1", "this is a test experience with several words", 0.8);
        pipeline.ingest_experience(exp);
        let stats = pipeline.stats();
        assert!(stats.l1_count > 0, "should extract atomic concepts");
    }

    #[test]
    fn test_concept_deduplication() {
        let mut pipeline = CrystalDistillationPipeline::default_pipeline();
        let exp1 = make_experience("e1", "concept_a concept_b concept_a", 0.8);
        let exp2 = make_experience("e2", "concept_a concept_c concept_a", 0.8);
        pipeline.ingest_experience(exp1);
        pipeline.ingest_experience(exp2);

        // concept_a should appear multiple times (occurrence_count > 1)
        let concept_a = pipeline
            .atomic_concepts()
            .iter()
            .find(|c| c.name == "concept_a");
        assert!(concept_a.is_some());
        assert!(concept_a.unwrap().occurrence_count >= 2);
    }

    #[test]
    fn test_distillation_stats_display() {
        let stats = CrystalDistillationStats {
            l0_count: 10,
            l1_count: 25,
            l2_count: 5,
            l3_exists: true,
            total_distillations: 3,
            avg_compression_ratio: 0.15,
            personality_trait_count: 8,
        };
        let display = format!("{}", stats);
        assert!(display.contains("L0 原始经验"));
        assert!(display.contains("L3 晶体人格"));
    }
}
