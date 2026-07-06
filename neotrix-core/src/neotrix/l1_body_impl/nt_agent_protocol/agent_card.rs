use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::discovery::AgentInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub provider: AgentProvider,
    pub version: String,
    pub capabilities: AgentCapabilities,
    pub skills: Vec<SkillSchema>,
    pub authentication: Vec<AuthScheme>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProvider {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub push_notifications: bool,
    pub state_synchronization: bool,
    pub custom_skill_execution: bool,
    pub e8_reasoning_modes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSchema {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthScheme {
    Bearer,
    ApiKey { location: ApiKeyLocation },
    OAuth2 { scopes: Vec<String> },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeyLocation {
    Header,
    Query,
    Cookie,
}

impl AgentCard {
    pub fn new(name: impl Into<String>, description: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            url: url.into(),
            provider: AgentProvider {
                name: "NeoTrix".into(),
                url: "https://neotrix.ai".into(),
                description: "AI-native developer toolkit — self-evolving reasoning engine".into(),
            },
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: AgentCapabilities {
                streaming: true,
                push_notifications: false,
                state_synchronization: true,
                custom_skill_execution: true,
                e8_reasoning_modes: (0..64).collect(),
            },
            skills: Vec::new(),
            authentication: vec![AuthScheme::None],
            metadata: HashMap::new(),
        }
    }

    pub fn from_agent_info(info: &AgentInfo) -> Self {
        let url = format!("neotrix://{}:{}", info.host, info.port);
        let mut card = Self::new(&info.name, &info.name, &url);
        card.capabilities.e8_reasoning_modes = vec![info.hexagram];
        if !info.capabilities.is_empty() {
            for cap in &info.capabilities {
                card.skills.push(SkillSchema {
                    name: cap.clone(),
                    description: format!("Capability: {}", cap),
                    input_schema: serde_json::Value::Object(serde_json::Map::new()),
                    output_schema: serde_json::Value::Object(serde_json::Map::new()),
                });
            }
        }
        card.metadata.insert("agent_id".to_string(), info.id.clone());
        card.metadata.insert("service_type".to_string(), info.service_type.clone());
        card.metadata.insert("instance_name".to_string(), info.instance_name.clone());
        card
    }

    pub fn add_skill(&mut self, name: impl Into<String>, description: impl Into<String>,
                     input: serde_json::Value, output: serde_json::Value) {
        self.skills.push(SkillSchema {
            name: name.into(),
            description: description.into(),
            input_schema: input,
            output_schema: output,
        });
    }

    pub fn register_per_skill(&mut self) {
        self.add_skill(
            "plan_execute_reflect",
            "Plan-Execute-Reflect autonomous task processing loop",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "task": {"type": "string", "description": "Task description"},
                    "max_iterations": {"type": "integer", "default": 5}
                },
                "required": ["task"]
            }),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "outcome": {"type": "string"},
                    "reward": {"type": "number"},
                    "iterations": {"type": "integer"}
                }
            }),
        );
        self.add_skill(
            "awareness_monitor",
            "Self-awareness and capability gap detection",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "capability_vector": {"type": "array", "items": {"type": "number"}}
                }
            }),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "gaps": {"type": "array", "items": {"type": "object"}},
                    "overall_health": {"type": "number"}
                }
            }),
        );
        self.add_skill(
            "knowledge_distillation",
            "Extract principles from session history and absorb into capability vector",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "sessions": {"type": "array", "items": {"type": "object"}}
                }
            }),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "principles_count": {"type": "integer"},
                    "absorbed_dims": {"type": "integer"}
                }
            }),
        );
        self.add_skill(
            "trend_analysis",
            "Linear regression trend analysis of capability evolution over time",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "history": {"type": "array", "items": {"type": "object"}}
                }
            }),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "trends": {"type": "object"},
                    "acceleration": {"type": "number"},
                    "stability": {"type": "number"}
                }
            }),
        );
        self.add_skill(
            "architecture_optimization",
            "Structural codebase analysis and improvement suggestions",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "scan_depth": {"type": "string", "enum": ["quick", "deep"]}
                }
            }),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "issues": {"type": "array"},
                    "recommendations": {"type": "array"}
                }
            }),
        );
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl From<AgentInfo> for AgentCard {
    fn from(info: AgentInfo) -> Self {
        Self::from_agent_info(&info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_new() {
        let card = AgentCard::new("test-agent", "A test agent", "neotrix://localhost:8080");
        assert_eq!(card.name, "test-agent");
        assert_eq!(card.description, "A test agent");
        assert_eq!(card.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(card.capabilities.e8_reasoning_modes.len(), 64);
    }

    #[test]
    fn test_agent_card_from_agent_info() {
        let info = AgentInfo::new("id-42", "my-agent", "192.168.1.1", 9000);
        let card = AgentCard::from(info);
        assert_eq!(card.name, "my-agent");
        assert!(card.url.contains("192.168.1.1:9000"));
        assert_eq!(card.metadata.get("agent_id").unwrap(), "id-42");
    }

    #[test]
    fn test_register_per_skills() {
        let mut card = AgentCard::new("per-agent", "PER loop", "neotrix://localhost:0");
        card.register_per_skill();
        assert_eq!(card.skills.len(), 5);
        assert_eq!(card.skills[0].name, "plan_execute_reflect");
        assert!(card.skills[0].input_schema.as_object().unwrap().contains_key("properties"));
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut card = AgentCard::new("test", "desc", "neotrix://host:0");
        card.register_per_skill();
        let json = serde_json::to_string(&card).unwrap();
        let deserialized: AgentCard = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test");
        assert_eq!(deserialized.skills.len(), 5);
    }

    #[test]
    fn test_to_json_pretty() {
        let card = AgentCard::new("pretty", "pretty card", "neotrix://h:0");
        let json = card.to_json_pretty().unwrap();
        assert!(json.contains("\"name\": \"pretty\""));
        assert!(json.contains("\"url\": \"neotrix://h:0\""));
    }

    #[test]
    fn test_auth_scheme_serialization() {
        let schemes = vec![
            AuthScheme::Bearer,
            AuthScheme::ApiKey { location: ApiKeyLocation::Header },
            AuthScheme::OAuth2 { scopes: vec!["read".into(), "write".into()] },
            AuthScheme::None,
        ];
        let json = serde_json::to_string(&schemes).unwrap();
        let deserialized: Vec<AuthScheme> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 4);
        assert!(matches!(deserialized[0], AuthScheme::Bearer));
        assert!(matches!(deserialized[3], AuthScheme::None));
    }

    #[test]
    fn test_skill_schema_valid_json() {
        let skill = SkillSchema {
            name: "test".into(),
            description: "test skill".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {"x": {"type": "integer"}}}),
            output_schema: serde_json::json!({"type": "string"}),
        };
        let json = serde_json::to_string(&skill).unwrap();
        assert!(json.contains("test"));
    }
}
