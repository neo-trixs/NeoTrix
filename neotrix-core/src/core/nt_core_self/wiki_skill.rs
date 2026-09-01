use std::collections::HashMap;
use std::time::{Duration, Instant};

/// WikiSkill 3-Layer Knowledge Base — 持久化知识库演化
///
/// 参考: arXiv:2608.27454 "WikiSkill: A Persistent Knowledge Base for Skill Evolution"
/// 核心思想: 三层分离 (原始经验/累积知识/可执行技能),
/// 支持技能跨模型迁移和知识库与技能协同演化。

/// 原始经验条目
#[derive(Debug, Clone)]
pub struct RawExperience {
    pub id: String,
    pub session_id: String,
    pub timestamp: Instant,
    pub content: String,
    pub domain: String,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

/// 累积知识条目
#[derive(Debug, Clone)]
pub struct AccumulatedKnowledge {
    pub id: String,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub content: String,
    pub domain: String,
    pub confidence: f64,
    pub source_experiences: Vec<String>,
    pub related_knowledge: Vec<String>,
    pub version: u32,
}

/// 可执行技能条目
#[derive(Debug, Clone)]
pub struct ExecutableSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub version: String,
    pub domain: String,
    pub code: String,
    pub inputs: Vec<SkillParameter>,
    pub outputs: Vec<SkillOutput>,
    pub dependencies: Vec<String>,
    pub knowledge_refs: Vec<String>,
    pub effectiveness: f64,
    pub usage_count: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SkillParameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SkillOutput {
    pub name: String,
    pub output_type: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgeSkillLink {
    pub knowledge_id: String,
    pub skill_id: String,
    pub link_type: LinkType,
    pub strength: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    Derives,
    DependsOn,
    Validates,
    Generates,
}

#[derive(Debug, Clone)]
pub struct CoEvolutionRecord {
    pub id: String,
    pub timestamp: Instant,
    pub event: CoEvolutionEvent,
    pub knowledge_id: Option<String>,
    pub skill_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoEvolutionEvent {
    KnowledgeCreated,
    KnowledgeUpdated,
    SkillCreated,
    SkillEffectivenessChanged,
    LinkCreated,
}

/// WikiSkill 3层知识库
pub struct WikiSkillKB {
    raw_experiences: HashMap<String, RawExperience>,
    accumulated_knowledge: HashMap<String, AccumulatedKnowledge>,
    executable_skills: HashMap<String, ExecutableSkill>,
    knowledge_skill_links: Vec<KnowledgeSkillLink>,
    co_evolution_log: Vec<CoEvolutionRecord>,
    config: WikiSkillConfig,
}

#[derive(Debug, Clone)]
pub struct WikiSkillConfig {
    pub raw_experience_max_age: Duration,
    pub max_raw_experiences: usize,
    pub confidence_threshold: f64,
    pub effectiveness_threshold: f64,
}

impl Default for WikiSkillConfig {
    fn default() -> Self {
        Self {
            raw_experience_max_age: Duration::from_secs(86400 * 7),
            max_raw_experiences: 10000,
            confidence_threshold: 0.3,
            effectiveness_threshold: 0.2,
        }
    }
}

impl WikiSkillKB {
    pub fn new(config: WikiSkillConfig) -> Self {
        Self {
            raw_experiences: HashMap::new(),
            accumulated_knowledge: HashMap::new(),
            executable_skills: HashMap::new(),
            knowledge_skill_links: Vec::new(),
            co_evolution_log: Vec::new(),
            config,
        }
    }

    pub fn add_raw_experience(&mut self, experience: RawExperience) -> String {
        let id = experience.id.clone();
        if self.raw_experiences.len() >= self.config.max_raw_experiences {
            self.cleanup_old_experiences();
        }
        self.raw_experiences.insert(id.clone(), experience);
        id
    }

    pub fn cleanup_old_experiences(&mut self) {
        let now = Instant::now();
        let max_age = self.config.raw_experience_max_age;
        self.raw_experiences.retain(|_, exp| now.duration_since(exp.timestamp) < max_age);
    }

    pub fn get_raw_experience(&self, id: &str) -> Option<&RawExperience> {
        self.raw_experiences.get(id)
    }

    pub fn create_knowledge(&mut self, knowledge: AccumulatedKnowledge) -> String {
        let id = knowledge.id.clone();
        self.accumulated_knowledge.insert(id.clone(), knowledge);
        id
    }

    pub fn update_knowledge(&mut self, id: &str, content: &str, confidence: f64) -> bool {
        if let Some(k) = self.accumulated_knowledge.get_mut(id) {
            k.content = content.to_string();
            k.confidence = confidence;
            k.updated_at = Instant::now();
            k.version += 1;
            true
        } else {
            false
        }
    }

    pub fn get_knowledge(&self, id: &str) -> Option<&AccumulatedKnowledge> {
        self.accumulated_knowledge.get(id)
    }

    pub fn low_confidence_knowledge(&self) -> Vec<&AccumulatedKnowledge> {
        self.accumulated_knowledge.values().filter(|k| k.confidence < self.config.confidence_threshold).collect()
    }

    pub fn create_skill(&mut self, skill: ExecutableSkill) -> String {
        let id = skill.id.clone();
        self.executable_skills.insert(id.clone(), skill);
        id
    }

    pub fn update_skill_effectiveness(&mut self, id: &str, effectiveness: f64) -> bool {
        if let Some(s) = self.executable_skills.get_mut(id) {
            s.effectiveness = effectiveness;
            s.updated_at = Instant::now();
            true
        } else {
            false
        }
    }

    pub fn get_skill(&self, id: &str) -> Option<&ExecutableSkill> {
        self.executable_skills.get(id)
    }

    pub fn low_effectiveness_skills(&self) -> Vec<&ExecutableSkill> {
        self.executable_skills.values().filter(|s| s.effectiveness < self.config.effectiveness_threshold).collect()
    }

    pub fn link_knowledge_skill(&mut self, knowledge_id: &str, skill_id: &str, link_type: LinkType, strength: f64) -> bool {
        if !self.accumulated_knowledge.contains_key(knowledge_id) || !self.executable_skills.contains_key(skill_id) {
            return false;
        }
        self.knowledge_skill_links.push(KnowledgeSkillLink {
            knowledge_id: knowledge_id.to_string(),
            skill_id: skill_id.to_string(),
            link_type,
            strength,
        });
        true
    }

    pub fn get_skills_for_knowledge(&self, knowledge_id: &str) -> Vec<&ExecutableSkill> {
        self.knowledge_skill_links.iter().filter(|l| l.knowledge_id == knowledge_id).filter_map(|l| self.executable_skills.get(&l.skill_id)).collect()
    }

    pub fn get_knowledge_for_skill(&self, skill_id: &str) -> Vec<&AccumulatedKnowledge> {
        self.knowledge_skill_links.iter().filter(|l| l.skill_id == skill_id).filter_map(|l| self.accumulated_knowledge.get(&l.knowledge_id)).collect()
    }

    pub fn stats(&self) -> (usize, usize, usize, usize) {
        (self.raw_experiences.len(), self.accumulated_knowledge.len(), self.executable_skills.len(), self.knowledge_skill_links.len())
    }

    pub fn get_co_evolution_log(&self) -> &[CoEvolutionRecord] {
        &self.co_evolution_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_exp(id: &str) -> RawExperience {
        RawExperience { id: id.to_string(), session_id: "s1".to_string(), timestamp: Instant::now(), content: "test".to_string(), domain: "NT-CORE".to_string(), source: "dialogue".to_string(), metadata: HashMap::new() }
    }

    fn make_know(id: &str) -> AccumulatedKnowledge {
        AccumulatedKnowledge { id: id.to_string(), created_at: Instant::now(), updated_at: Instant::now(), content: "test".to_string(), domain: "NT-MEMORY".to_string(), confidence: 0.8, source_experiences: vec![], related_knowledge: vec![], version: 1 }
    }

    fn make_skill(id: &str) -> ExecutableSkill {
        ExecutableSkill { id: id.to_string(), name: format!("Skill {}", id), description: "test".to_string(), created_at: Instant::now(), updated_at: Instant::now(), version: "1.0".to_string(), domain: "NT-ACT".to_string(), code: "fn exec(){}".to_string(), inputs: vec![], outputs: vec![], dependencies: vec![], knowledge_refs: vec![], effectiveness: 0.8, usage_count: 0, tags: vec![] }
    }

    #[test]
    fn test_raw_experience_lifecycle() {
        let mut kb = WikiSkillKB::new(WikiSkillConfig::default());
        let id = kb.add_raw_experience(make_exp("e1"));
        assert_eq!(id, "e1");
        assert!(kb.get_raw_experience("e1").is_some());
    }

    #[test]
    fn test_knowledge_lifecycle() {
        let mut kb = WikiSkillKB::new(WikiSkillConfig::default());
        let id = kb.create_knowledge(make_know("k1"));
        assert_eq!(id, "k1");
        assert!(kb.update_knowledge("k1", "updated", 0.9));
        let k = kb.get_knowledge("k1").unwrap();
        assert_eq!(k.version, 2);
        assert_eq!(k.confidence, 0.9);
    }

    #[test]
    fn test_skill_lifecycle() {
        let mut kb = WikiSkillKB::new(WikiSkillConfig::default());
        let id = kb.create_skill(make_skill("s1"));
        assert_eq!(id, "s1");
        assert!(kb.update_skill_effectiveness("s1", 0.5));
        assert_eq!(kb.get_skill("s1").unwrap().effectiveness, 0.5);
    }

    #[test]
    fn test_knowledge_skill_link() {
        let mut kb = WikiSkillKB::new(WikiSkillConfig::default());
        kb.create_knowledge(make_know("k1"));
        kb.create_skill(make_skill("s1"));
        assert!(kb.link_knowledge_skill("k1", "s1", LinkType::Derives, 0.9));
        assert_eq!(kb.get_skills_for_knowledge("k1").len(), 1);
        assert_eq!(kb.get_knowledge_for_skill("s1").len(), 1);
    }

    #[test]
    fn test_low_confidence_and_effectiveness() {
        let mut kb = WikiSkillKB::new(WikiSkillConfig::default());
        let mut k = make_know("k1");
        k.confidence = 0.1;
        kb.create_knowledge(k);
        assert_eq!(kb.low_confidence_knowledge().len(), 1);

        let mut s = make_skill("s1");
        s.effectiveness = 0.1;
        kb.create_skill(s);
        assert_eq!(kb.low_effectiveness_skills().len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut kb = WikiSkillKB::new(WikiSkillConfig::default());
        kb.add_raw_experience(make_exp("e1"));
        kb.create_knowledge(make_know("k1"));
        kb.create_skill(make_skill("s1"));
        let (raw, know, skill, links) = kb.stats();
        assert_eq!(raw, 1);
        assert_eq!(know, 1);
        assert_eq!(skill, 1);
        assert_eq!(links, 0);
    }
}
