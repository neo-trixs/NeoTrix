//! Experience-Knowledge Bridge — WikiSkill 三层吸收模型
//!
//! 吸收 WikiSkill (arXiv:2608.27454) 核心模式:
//! - Layer 1: Raw Execution Experience (原始执行经验) — 每次调用的原始记录
//! - Layer 2: Accumulated Knowledge (累积知识) — 蒸馏后的模式/规则/反模式
//! - Layer 3: Executable Skill (可执行技能) — 结晶后的正式技能
//!
//! 三层之间通过 Bridge 单向流动，每层有独立的存储和生命周期。
//! 知识层是 Wiki 的持久化核心，技能层通过知识层间接获取经验上下文。

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// WikiSkill 三层模型中的知识层条目 (Layer 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// 知识唯一 ID
    pub id: String,
    /// 知识类型 (pattern / anti_pattern / rule / optimization)
    pub kind: KnowledgeKind,
    /// 知识内容摘要
    pub summary: String,
    /// 完整内容
    pub content: String,
    /// 关联的原始经验 ID 列表
    pub source_experience_ids: Vec<String>,
    /// 关联的技能 ID (如果已结晶)
    pub skill_id: Option<String>,
    /// 置信度 (0.0-1.0)
    pub confidence: f64,
    /// 被引用次数 (知识被技能调用的次数)
    pub citation_count: u32,
    /// 创建时间戳
    pub created_at: i64,
    /// 最后更新时间戳
    pub updated_at: i64,
    /// 来源领域
    pub domain: String,
}

/// 知识类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    /// 成功模式 (从成功轨迹蒸馏)
    Pattern,
    /// 失败反模式 (从失败轨迹提取)
    AntiPattern,
    /// 规则 (从多次经验归纳)
    Rule,
    /// 优化建议 (从对比分析得出)
    Optimization,
}

/// WikiSkill 原始执行经验记录 (Layer 1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExperience {
    /// 经验唯一 ID
    pub id: String,
    /// 任务描述
    pub task: String,
    /// 执行的技能/工具列表
    pub skills_used: Vec<String>,
    /// 输入摘要
    pub input_summary: String,
    /// 输出摘要
    pub output_summary: String,
    /// 是否成功
    pub success: bool,
    /// Token 消耗
    pub tokens_used: u32,
    /// 时间戳
    pub timestamp: i64,
    /// 会话 ID
    pub session_id: String,
    /// 错误信息 (失败时)
    pub error: Option<String>,
}

/// WikiSkill 可执行技能记录 (Layer 3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableSkill {
    /// 技能唯一 ID
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// 输入 schema
    pub input_schema: String,
    /// 输出 schema
    pub output_schema: String,
    /// 关联的知识 ID 列表 (技能依赖的知识)
    pub knowledge_ids: Vec<String>,
    /// 成功率
    pub success_rate: f64,
    /// 版本号
    pub version: u32,
    /// 是否跨会话可用
    pub persistent: bool,
}

/// 吸收统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbsorptionStats {
    pub raw_experiences: u64,
    pub knowledge_entries: u64,
    pub executable_skills: u64,
    pub knowledge_to_skill_ratio: f64,
    pub avg_confidence: f64,
}

/// Bridge 配置
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// 蒸馏阈值: 同类经验 >= N 条时触发知识蒸馏
    pub distillation_threshold: u32,
    /// 结晶阈值: 知识被引用 >= N 次时触发技能结晶
    pub crystallization_threshold: u32,
    /// 最大原始经验保留数
    pub max_raw_experiences: usize,
    /// 最小知识置信度
    pub min_knowledge_confidence: f64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            distillation_threshold: 3,
            crystallization_threshold: 5,
            max_raw_experiences: 1000,
            min_knowledge_confidence: 0.6,
        }
    }
}

/// Experience-Knowledge Bridge — WikiSkill 三层吸收管线
///
/// 核心职责:
/// 1. 原始经验 → 知识蒸馏 (Layer 1 → Layer 2)
/// 2. 知识 → 技能结晶 (Layer 2 → Layer 3)
/// 3. 技能 → 知识回溯 (Layer 3 → Layer 2 查询)
/// 4. 跨会话持久化 (三层独立存储)
pub struct ExperienceKnowledgeBridge {
    /// Layer 1: 原始执行经验
    raw_experiences: Vec<RawExperience>,
    /// Layer 2: 累积知识
    knowledge_entries: Vec<KnowledgeEntry>,
    /// Layer 3: 可执行技能
    executable_skills: Vec<ExecutableSkill>,
    /// 经验→知识映射 (experience_id → knowledge_id)
    experience_to_knowledge: HashMap<String, Vec<String>>,
    /// 知识→技能映射 (knowledge_id → skill_id)
    knowledge_to_skill: HashMap<String, String>,
    /// 配置
    config: BridgeConfig,
}

impl ExperienceKnowledgeBridge {
    /// 创建新的 Bridge
    pub fn new(config: BridgeConfig) -> Self {
        Self {
            raw_experiences: Vec::new(),
            knowledge_entries: Vec::new(),
            executable_skills: Vec::new(),
            experience_to_knowledge: HashMap::new(),
            knowledge_to_skill: HashMap::new(),
            config,
        }
    }

    /// Layer 1: 记录原始执行经验
    pub fn record_experience(&mut self, experience: RawExperience) {
        if self.raw_experiences.len() >= self.config.max_raw_experiences {
            self.raw_experiences.remove(0);
        }
        self.raw_experiences.push(experience);
    }

    /// Layer 1->2: 蒸馏原始经验为知识
    pub fn distill(&mut self) -> Vec<KnowledgeEntry> {
        let mut new_knowledge = Vec::new();

        let mut skill_groups: HashMap<String, Vec<&RawExperience>> = HashMap::new();
        for exp in &self.raw_experiences {
            for skill in &exp.skills_used {
                skill_groups.entry(skill.clone()).or_default().push(exp);
            }
        }

        for (skill_name, experiences) in &skill_groups {
            let successes: Vec<&RawExperience> = experiences.iter().filter(|e| e.success).collect();
            let failures: Vec<&RawExperience> = experiences.iter().filter(|e| !e.success).collect();

            if experiences.len() >= self.config.distillation_threshold as usize {
                let success_rate = successes.len() as f64 / experiences.len() as f64;

                if !successes.is_empty() {
                    let pattern = self.extract_success_pattern(&successes);
                    let knowledge = KnowledgeEntry {
                        id: format!("k_{}_pattern_{}", skill_name, self.knowledge_entries.len()),
                        kind: KnowledgeKind::Pattern,
                        summary: format!("{} success pattern ({:.0}%)", skill_name, success_rate * 100.0),
                        content: pattern,
                        source_experience_ids: successes.iter().map(|e| e.id.clone()).collect(),
                        skill_id: None,
                        confidence: success_rate,
                        citation_count: 0,
                        created_at: timestamp_now(),
                        updated_at: timestamp_now(),
                        domain: "execution".to_string(),
                    };

                    for exp in &successes {
                        self.experience_to_knowledge
                            .entry(exp.id.clone())
                            .or_default()
                            .push(knowledge.id.clone());
                    }

                    new_knowledge.push(knowledge);
                }

                if !failures.is_empty() {
                    let anti_pattern = self.extract_failure_pattern(&failures);
                    let knowledge = KnowledgeEntry {
                        id: format!("k_{}_antipattern_{}", skill_name, self.knowledge_entries.len()),
                        kind: KnowledgeKind::AntiPattern,
                        summary: format!("{} failure anti-pattern ({:.0}%)", skill_name, (1.0 - success_rate) * 100.0),
                        content: anti_pattern,
                        source_experience_ids: failures.iter().map(|e| e.id.clone()).collect(),
                        skill_id: None,
                        confidence: 1.0 - success_rate,
                        citation_count: 0,
                        created_at: timestamp_now(),
                        updated_at: timestamp_now(),
                        domain: "execution".to_string(),
                    };

                    for exp in &failures {
                        self.experience_to_knowledge
                            .entry(exp.id.clone())
                            .or_default()
                            .push(knowledge.id.clone());
                    }

                    new_knowledge.push(knowledge);
                }
            }
        }

        self.knowledge_entries.extend(new_knowledge.clone());
        new_knowledge
    }

    /// Layer 2->3: 将高频知识结晶为可执行技能
    pub fn crystallize_skill(
        &mut self,
        knowledge_id: &str,
        name: &str,
        description: &str,
        input_schema: &str,
        output_schema: &str,
    ) -> Option<ExecutableSkill> {
        let knowledge = self.knowledge_entries.iter_mut().find(|k| k.id == knowledge_id)?;

        if knowledge.citation_count < self.config.crystallization_threshold {
            return None;
        }

        let skill = ExecutableSkill {
            id: format!("skill_{}", self.executable_skills.len() + 1),
            name: name.to_string(),
            description: description.to_string(),
            input_schema: input_schema.to_string(),
            output_schema: output_schema.to_string(),
            knowledge_ids: vec![knowledge_id.to_string()],
            success_rate: knowledge.confidence,
            version: 1,
            persistent: true,
        };

        knowledge.skill_id = Some(skill.id.clone());
        self.knowledge_to_skill.insert(knowledge_id.to_string(), skill.id.clone());
        self.executable_skills.push(skill.clone());
        Some(skill)
    }

    /// Layer 3->2: 技能回溯知识上下文
    pub fn skill_context(&self, skill_id: &str) -> Vec<&KnowledgeEntry> {
        let skill = match self.executable_skills.iter().find(|s| s.id == skill_id) {
            Some(s) => s,
            None => return Vec::new(),
        };
        self.knowledge_entries
            .iter()
            .filter(|k| skill.knowledge_ids.contains(&k.id))
            .collect()
    }

    /// 增加知识引用次数
    pub fn cite_knowledge(&mut self, knowledge_id: &str) {
        if let Some(k) = self.knowledge_entries.iter_mut().find(|e| e.id == knowledge_id) {
            k.citation_count += 1;
            k.updated_at = timestamp_now();
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> AbsorptionStats {
        let total_confidence: f64 = self.knowledge_entries.iter().map(|k| k.confidence).sum();
        let avg_confidence = if self.knowledge_entries.is_empty() {
            0.0
        } else {
            total_confidence / self.knowledge_entries.len() as f64
        };

        AbsorptionStats {
            raw_experiences: self.raw_experiences.len() as u64,
            knowledge_entries: self.knowledge_entries.len() as u64,
            executable_skills: self.executable_skills.len() as u64,
            knowledge_to_skill_ratio: if self.knowledge_entries.is_empty() {
                0.0
            } else {
                self.executable_skills.len() as f64 / self.knowledge_entries.len() as f64
            },
            avg_confidence,
        }
    }

    /// 序列化为 JSON (跨会话持久化)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct PersistableBridge<'a> {
            raw_experiences: &'a [RawExperience],
            knowledge_entries: &'a [KnowledgeEntry],
            executable_skills: &'a [ExecutableSkill],
            experience_to_knowledge: &'a HashMap<String, Vec<String>>,
            knowledge_to_skill: &'a HashMap<String, String>,
        }
        serde_json::to_string(&PersistableBridge {
            raw_experiences: &self.raw_experiences,
            knowledge_entries: &self.knowledge_entries,
            executable_skills: &self.executable_skills,
            experience_to_knowledge: &self.experience_to_knowledge,
            knowledge_to_skill: &self.knowledge_to_skill,
        })
    }

    /// 从 JSON 恢复 (跨会话加载)
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        #[derive(Deserialize)]
        struct PersistableBridge {
            raw_experiences: Vec<RawExperience>,
            knowledge_entries: Vec<KnowledgeEntry>,
            executable_skills: Vec<ExecutableSkill>,
            experience_to_knowledge: HashMap<String, Vec<String>>,
            knowledge_to_skill: HashMap<String, String>,
        }
        let data: PersistableBridge = serde_json::from_str(json)?;
        Ok(Self {
            raw_experiences: data.raw_experiences,
            knowledge_entries: data.knowledge_entries,
            executable_skills: data.executable_skills,
            experience_to_knowledge: data.experience_to_knowledge,
            knowledge_to_skill: data.knowledge_to_skill,
            config: BridgeConfig::default(),
        })
    }

    fn extract_success_pattern(&self, successes: &[&RawExperience]) -> String {
        let skills: Vec<&str> = successes
            .iter()
            .flat_map(|e| e.skills_used.iter().map(|s| s.as_str()))
            .collect();
        let avg_tokens = successes.iter().map(|e| e.tokens_used).sum::<u32>() / successes.len() as u32;
        format!(
            "Success pattern: skill chain {:?}, avg tokens {}, success count {}",
            skills, avg_tokens, successes.len()
        )
    }

    fn extract_failure_pattern(&self, failures: &[&RawExperience]) -> String {
        let errors: Vec<&str> = failures
            .iter()
            .filter_map(|e| e.error.as_deref())
            .collect();
        format!(
            "Failure anti-pattern: error signatures {:?}, failure count {}",
            errors, failures.len()
        )
    }
}

fn timestamp_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_experience(id: &str, skill: &str, success: bool) -> RawExperience {
        RawExperience {
            id: id.to_string(),
            task: "test task".to_string(),
            skills_used: vec![skill.to_string()],
            input_summary: "input".to_string(),
            output_summary: "output".to_string(),
            success,
            tokens_used: 100,
            timestamp: timestamp_now(),
            session_id: "s1".to_string(),
            error: if success { None } else { Some("error".to_string()) },
        }
    }

    #[test]
    fn test_record_and_distill() {
        let mut bridge = ExperienceKnowledgeBridge::new(BridgeConfig {
            distillation_threshold: 2,
            ..Default::default()
        });

        bridge.record_experience(make_experience("e1", "code_gen", true));
        bridge.record_experience(make_experience("e2", "code_gen", true));
        bridge.record_experience(make_experience("e3", "code_gen", false));

        let knowledge = bridge.distill();
        assert_eq!(knowledge.len(), 2);
        assert_eq!(bridge.knowledge_entries.len(), 2);
    }

    #[test]
    fn test_crystallize_and_context() {
        let mut bridge = ExperienceKnowledgeBridge::new(BridgeConfig {
            distillation_threshold: 2,
            crystallization_threshold: 2,
            ..Default::default()
        });

        for i in 0..4 {
            bridge.record_experience(make_experience(&format!("e{}", i), "code_gen", true));
        }
        bridge.distill();

        let kid = bridge.knowledge_entries[0].id.clone();
        bridge.cite_knowledge(&kid);
        bridge.cite_knowledge(&kid);

        let skill = bridge.crystallize_skill(&kid, "CodeGen", "Generates code", "{}", "{}");
        assert!(skill.is_some());
        assert_eq!(bridge.executable_skills.len(), 1);

        let context = bridge.skill_context(&skill.unwrap().id);
        assert_eq!(context.len(), 1);
    }

    #[test]
    fn test_persistence_roundtrip() {
        let mut bridge = ExperienceKnowledgeBridge::new(BridgeConfig::default());
        bridge.record_experience(make_experience("e1", "test", true));
        bridge.record_experience(make_experience("e2", "test", true));
        bridge.record_experience(make_experience("e3", "test", true));
        bridge.distill();

        let json = bridge.to_json().unwrap();
        let restored = ExperienceKnowledgeBridge::from_json(&json).unwrap();
        assert_eq!(restored.knowledge_entries.len(), 1);
        assert_eq!(restored.raw_experiences.len(), 3);
    }

    #[test]
    fn test_stats() {
        let mut bridge = ExperienceKnowledgeBridge::new(BridgeConfig::default());
        bridge.record_experience(make_experience("e1", "test", true));
        bridge.record_experience(make_experience("e2", "test", false));
        let stats = bridge.stats();
        assert_eq!(stats.raw_experiences, 2);
    }
}
