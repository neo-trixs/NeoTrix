//! WikiSkill Three-Layer Knowledge Architecture — WikiSkill三层知识架构
//!
//! 吸收 KB 经验:
//! - arXiv:2608.27454 WikiSkill
//! - raw/knowledge/skills 三层 KB 架构
//! - 知识从原始经验到可执行技能的演进
//! - absorption pipeline 集成

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// WikiSkill 三层知识架构
pub struct WikiSkillThreeLayerKB {
    raw_layer: RawLayer,
    knowledge_layer: KnowledgeLayer,
    skills_layer: SkillsLayer,
    config: WikiSkillConfig,
    stats: WikiSkillStats,
}

/// WikiSkill 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSkillConfig {
    pub max_raw_entries: usize,
    pub max_knowledge_entries: usize,
    pub max_skills: usize,
    pub enable_auto_promotion: bool,
    pub promotion_threshold: f64,
}

impl Default for WikiSkillConfig {
    fn default() -> Self {
        Self {
            max_raw_entries: 10000,
            max_knowledge_entries: 5000,
            max_skills: 1000,
            enable_auto_promotion: true,
            promotion_threshold: 0.7,
        }
    }
}

/// 原始层 (Raw Layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLayer {
    pub entries: Vec<RawEntry>,
    pub stats: LayerStats,
}

/// 原始条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEntry {
    pub entry_id: String,
    pub content: String,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub quality_score: f64,
    pub metadata: HashMap<String, String>,
}

/// 知识层 (Knowledge Layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeLayer {
    pub entries: Vec<KnowledgeEntry>,
    pub relations: Vec<KnowledgeRelation>,
    pub stats: LayerStats,
}

/// 知识条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub entry_id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    pub confidence: f64,
    pub source_entries: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 知识关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRelation {
    pub relation_id: String,
    pub from_entry: String,
    pub to_entry: String,
    pub relation_type: String,
    pub weight: f64,
}

/// 技能层 (Skills Layer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsLayer {
    pub skills: Vec<Skill>,
    pub stats: LayerStats,
}

/// 技能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub skill_type: SkillType,
    pub knowledge_references: Vec<String>,
    pub execution_count: u64,
    pub success_rate: f64,
}

/// 技能类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillType {
    Task,
    Workflow,
    Pattern,
    Rule,
}

/// 层统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStats {
    pub total_entries: u64,
    pub promoted_entries: u64,
    pub avg_quality: f64,
}

/// WikiSkill 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSkillStats {
    pub total_raw: u64,
    pub total_knowledge: u64,
    pub total_skills: u64,
    pub promotion_rate: f64,
    pub avg_skill_success_rate: f64,
}

/// 演进结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionResult {
    pub success: bool,
    pub promoted_count: u64,
    pub new_skills: Vec<String>,
    pub message: String,
}

impl WikiSkillThreeLayerKB {
    /// 创建新的 WikiSkill 三层知识架构
    pub fn new() -> Self {
        Self {
            raw_layer: RawLayer {
                entries: Vec::new(),
                stats: LayerStats {
                    total_entries: 0,
                    promoted_entries: 0,
                    avg_quality: 0.0,
                },
            },
            knowledge_layer: KnowledgeLayer {
                entries: Vec::new(),
                relations: Vec::new(),
                stats: LayerStats {
                    total_entries: 0,
                    promoted_entries: 0,
                    avg_quality: 0.0,
                },
            },
            skills_layer: SkillsLayer {
                skills: Vec::new(),
                stats: LayerStats {
                    total_entries: 0,
                    promoted_entries: 0,
                    avg_quality: 0.0,
                },
            },
            config: WikiSkillConfig::default(),
            stats: WikiSkillStats {
                total_raw: 0,
                total_knowledge: 0,
                total_skills: 0,
                promotion_rate: 0.0,
                avg_skill_success_rate: 0.0,
            },
        }
    }

    /// 添加原始条目
    pub fn add_raw_entry(&mut self, entry: RawEntry) {
        self.raw_layer.entries.push(entry);
        self.stats.total_raw += 1;
    }

    /// 演进: 从原始层提升到知识层
    pub fn promote_to_knowledge(&mut self, entry_id: &str) -> bool {
        if let Some(pos) = self.raw_layer.entries.iter().position(|e| e.entry_id == entry_id) {
            let raw_entry = self.raw_layer.entries.remove(pos);

            if raw_entry.quality_score >= self.config.promotion_threshold {
                let knowledge_entry = KnowledgeEntry {
                    entry_id: raw_entry.entry_id.clone(),
                    title: format!("Knowledge from {}", raw_entry.source),
                    content: raw_entry.content,
                    category: "auto_promoted".into(),
                    confidence: raw_entry.quality_score,
                    source_entries: vec![raw_entry.entry_id],
                    created_at: chrono::Utc::now(),
                };

                self.knowledge_layer.entries.push(knowledge_entry);
                self.stats.total_knowledge += 1;
                self.raw_layer.stats.promoted_entries += 1;

                return true;
            }
        }
        false
    }

    /// 演进: 从知识层提升到技能层
    pub fn promote_to_skill(&mut self, entry_id: &str, skill_type: SkillType) -> bool {
        if let Some(pos) = self.knowledge_layer.entries.iter().position(|e| e.entry_id == entry_id) {
            let knowledge_entry = self.knowledge_layer.entries.remove(pos);

            let skill = Skill {
                skill_id: uuid::Uuid::new_v4().to_string(),
                name: knowledge_entry.title,
                description: knowledge_entry.content,
                skill_type,
                knowledge_references: knowledge_entry.source_entries,
                execution_count: 0,
                success_rate: 0.0,
            };

            self.skills_layer.skills.push(skill);
            self.stats.total_skills += 1;
            self.knowledge_layer.stats.promoted_entries += 1;

            return true;
        }
        false
    }

    /// 自动演进
    pub fn auto_evolve(&mut self) -> EvolutionResult {
        let mut promoted_count = 0;
        let mut new_skills = Vec::new();

        // 从原始层提升到知识层
        let raw_ids: Vec<String> = self.raw_layer.entries.iter()
            .filter(|e| e.quality_score >= self.config.promotion_threshold)
            .map(|e| e.entry_id.clone())
            .collect();

        for entry_id in &raw_ids {
            if self.promote_to_knowledge(entry_id) {
                promoted_count += 1;
            }
        }

        // 从知识层提升到技能层
        let knowledge_ids: Vec<String> = self.knowledge_layer.entries.iter()
            .filter(|e| e.confidence >= self.config.promotion_threshold)
            .map(|e| e.entry_id.clone())
            .collect();

        for entry_id in &knowledge_ids {
            if self.promote_to_skill(entry_id, SkillType::Task) {
                new_skills.push(entry_id.clone());
            }
        }

        EvolutionResult {
            success: true,
            promoted_count,
            new_skills,
            message: format!("演进完成: 提升 {} 条知识, 创建 {} 个技能", promoted_count, new_skills.len()),
        }
    }

    /// 获取所有层
    pub fn raw_layer(&self) -> &RawLayer {
        &self.raw_layer
    }

    pub fn knowledge_layer(&self) -> &KnowledgeLayer {
        &self.knowledge_layer
    }

    pub fn skills_layer(&self) -> &SkillsLayer {
        &self.skills_layer
    }

    /// 获取统计信息
    pub fn stats(&self) -> &WikiSkillStats {
        &self.stats
    }
}
