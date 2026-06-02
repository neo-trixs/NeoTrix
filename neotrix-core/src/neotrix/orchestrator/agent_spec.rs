use serde::{Serialize, Deserialize};

/// A declarative agent specification, inspired by CrewAI's Agent class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub role: String,
    pub goal: String,
    pub backstory: String,
    pub allowed_tools: Vec<String>,
    pub llm_config: Option<String>,
    pub allow_delegation: bool,
    pub max_iterations: Option<u32>,
}

impl AgentSpec {
    pub fn new(role: &str, goal: &str, backstory: &str) -> Self {
        Self {
            role: role.to_string(),
            goal: goal.to_string(),
            backstory: backstory.to_string(),
            allowed_tools: Vec::new(),
            llm_config: None,
            allow_delegation: false,
            max_iterations: None,
        }
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    pub fn with_llm(mut self, llm: &str) -> Self {
        self.llm_config = Some(llm.to_string());
        self
    }

    pub fn with_delegation(mut self) -> Self {
        self.allow_delegation = true;
        self
    }

    pub fn with_max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = Some(n);
        self
    }
}

/// Registry of agent specs for quick lookup by role name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentSpecRegistry {
    specs: std::collections::HashMap<String, AgentSpec>,
}

impl AgentSpecRegistry {
    pub fn new() -> Self {
        Self { specs: std::collections::HashMap::new() }
    }

    pub fn register(&mut self, spec: AgentSpec) {
        self.specs.insert(spec.role.clone(), spec);
    }

    pub fn get(&self, role: &str) -> Option<&AgentSpec> {
        self.specs.get(role)
    }

    pub fn all(&self) -> Vec<&AgentSpec> {
        self.specs.values().collect()
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_spec_creation() {
        let spec = AgentSpec::new("researcher", "Find information", "Expert researcher")
            .with_tools(vec!["nt_world_search".into(), "read_url".into()])
            .with_llm("gpt-4o");
        assert_eq!(spec.role, "researcher");
        assert_eq!(spec.allowed_tools.len(), 2);
        assert!(spec.llm_config.is_some());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = AgentSpecRegistry::new();
        registry.register(AgentSpec::new("coder", "Write code", "Expert coder"));
        registry.register(AgentSpec::new("reviewer", "Review code", "Expert reviewer"));
        assert_eq!(registry.len(), 2);
        assert!(registry.get("coder").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let spec = AgentSpec::new("dev", "Develop", "Dev")
            .with_tools(vec!["rustc".into()])
            .with_delegation()
            .with_max_iterations(10);
        assert!(spec.allow_delegation);
        assert_eq!(spec.max_iterations, Some(10));
    }

    #[test]
    fn test_default_values() {
        let spec = AgentSpec::new("minimal", "Just do it", "Minimalist");
        assert!(!spec.allow_delegation);
        assert!(spec.max_iterations.is_none());
        assert!(spec.allowed_tools.is_empty());
    }
}
