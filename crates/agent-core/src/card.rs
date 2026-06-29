use serde::{Deserialize, Serialize};

/// A2A AgentCard — self-description that every agent exposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    pub capabilities: AgentCapabilities,
    pub skills: Vec<SkillDecl>,
    pub registry_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub push_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDecl {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
}

impl AgentCard {
    pub fn new(
        name: &str,
        description: &str,
        url: &str,
        version: &str,
        skills: Vec<SkillDecl>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            url: url.to_string(),
            version: version.to_string(),
            capabilities: AgentCapabilities {
                streaming: true,
                push_notifications: false,
            },
            skills,
            registry_url: None,
        }
    }

    pub fn skill_ids(&self) -> Vec<&str> {
        self.skills.iter().map(|s| s.id.as_str()).collect()
    }

    pub fn has_skill(&self, skill_id: &str) -> bool {
        self.skills.iter().any(|s| s.id == skill_id)
    }
}
