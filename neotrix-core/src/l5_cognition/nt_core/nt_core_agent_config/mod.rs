use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub role: String,
    pub goal: String,
    pub backstory: Option<String>,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
    pub allowed_functions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl AgentConfig {
    pub fn new(name: &str, role: &str, goal: &str) -> Self {
        Self {
            name: name.to_string(),
            role: role.to_string(),
            goal: goal.to_string(),
            backstory: None,
            tools: Vec::new(),
            model: None,
            temperature: None,
            max_tokens: None,
            system_prompt: None,
            allowed_functions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn with_tool(mut self, tool: &str) -> Self {
        self.tools.push(tool.to_string());
        self
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewConfig {
    pub name: String,
    pub agents: Vec<AgentConfig>,
    pub process: ProcessType,
    pub max_iterations: u32,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessType {
    Sequential,
    Hierarchical,
}

impl CrewConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            agents: Vec::new(),
            process: ProcessType::Sequential,
            max_iterations: 10,
            verbose: false,
        }
    }

    pub fn add_agent(mut self, agent: AgentConfig) -> Self {
        self.agents.push(agent);
        self
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub fn to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(std::fs::write(path, serde_json::to_string_pretty(self)?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config() {
        let a = AgentConfig::new("researcher", "Research", "Find info")
            .with_model("gpt-4")
            .with_temperature(0.7);
        assert_eq!(a.name, "researcher");
        assert_eq!(a.model.unwrap(), "gpt-4");
    }

    #[test]
    fn test_crew_config() {
        let c = CrewConfig::new("team1")
            .add_agent(AgentConfig::new("a1", "r", "g"))
            .add_agent(AgentConfig::new("a2", "r", "g"));
        assert_eq!(c.agents.len(), 2);
    }

    #[test]
    fn test_json_roundtrip() {
        let a = AgentConfig::new("a", "r", "g").with_tool("search");
        let j = a.to_json().unwrap();
        let b = AgentConfig::from_json(&j).unwrap();
        assert_eq!(a.name, b.name);
        assert_eq!(a.tools, b.tools);
    }
}
