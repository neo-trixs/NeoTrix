use serde::{Deserialize, Serialize};

/// Skills system — Grok Bot pattern.
///
/// Skills are reusable instruction sets that agents can load on demand.
/// Each skill has a trigger condition, instructions, and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub triggers: Vec<SkillTrigger>,
    pub instructions: SkillInstructions,
    pub validation: SkillValidation,
    pub visibility: SkillVisibility,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    pub kind: TriggerKind,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerKind {
    Message,
    File,
    Tool,
    Schedule,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInstructions {
    pub system_prompt: String,
    pub required_tools: Vec<String>,
    pub context_template: Option<String>,
    pub max_tokens: Option<u32>,
    pub tier_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillValidation {
    pub output_schema: Option<serde_json::Value>,
    pub required_patterns: Vec<String>,
    pub max_output_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillVisibility {
    Global,
    Agents(Vec<String>),
    Private,
}

impl Skill {
    pub fn matches_message(&self, message: &str) -> bool {
        self.triggers.iter().any(|t| match t.kind {
            TriggerKind::Message => message.to_lowercase().contains(&t.pattern.to_lowercase()),
            _ => false,
        })
    }

    pub fn matches_file(&self, path: &str) -> bool {
        self.triggers.iter().any(|t| match t.kind {
            TriggerKind::File => path.contains(&t.pattern),
            _ => false,
        })
    }
}

pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn register(&mut self, skill: Skill) {
        tracing::info!("registered skill: {} ({})", skill.name, skill.id);
        self.skills.push(skill);
    }

    pub fn find_triggered(&self, message: &str) -> Vec<&Skill> {
        self.skills
            .iter()
            .filter(|s| s.matches_message(message))
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    pub fn for_agent(&self, agent_id: &str) -> Vec<&Skill> {
        self.skills
            .iter()
            .filter(|s| match &s.visibility {
                SkillVisibility::Global => true,
                SkillVisibility::Agents(ids) => ids.contains(&agent_id.to_string()),
                SkillVisibility::Private => false,
            })
            .collect()
    }

    pub fn list_all(&self) -> &[Skill] {
        &self.skills
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
