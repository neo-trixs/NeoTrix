//! # Agent Registry
//!
//! Persistent registry of agent metadata: versions, capabilities, configuration.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AgentMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub config: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentMeta>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, meta: AgentMeta) {
        self.agents.insert(meta.name.clone(), meta);
    }

    pub fn unregister(&mut self, name: &str) -> Option<AgentMeta> {
        self.agents.remove(name)
    }

    pub fn get(&self, name: &str) -> Option<&AgentMeta> {
        self.agents.get(name)
    }

    pub fn enable(&mut self, name: &str) {
        if let Some(meta) = self.agents.get_mut(name) {
            meta.enabled = true;
        }
    }

    pub fn disable(&mut self, name: &str) {
        if let Some(meta) = self.agents.get_mut(name) {
            meta.enabled = false;
        }
    }

    pub fn list_enabled(&self) -> Vec<&AgentMeta> {
        self.agents.values().filter(|a| a.enabled).collect()
    }

    pub fn list_all(&self) -> Vec<&AgentMeta> {
        self.agents.values().collect()
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentMeta> {
        self.agents
            .values()
            .filter(|a| a.enabled && a.capabilities.iter().any(|c| c == capability))
            .collect()
    }

    pub fn report(&self) -> String {
        format!("registry:agents_{}_enabled_{}", self.agents.len(), self.list_enabled().len())
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_agent(name: &str) -> AgentMeta {
        AgentMeta {
            name: name.into(),
            version: "1.0.0".into(),
            description: format!("{name} agent"),
            capabilities: vec!["mock".into()],
            config: HashMap::new(),
            enabled: true,
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent("agent_a"));
        assert!(reg.get("agent_a").is_some());
    }

    #[test]
    fn test_enable_disable() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent("agent_a"));
        reg.disable("agent_a");
        assert!(reg.list_enabled().is_empty());
        reg.enable("agent_a");
        assert_eq!(reg.list_enabled().len(), 1);
    }

    #[test]
    fn test_find_by_capability() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent("agent_a"));
        let found = reg.find_by_capability("mock");
        assert_eq!(found.len(), 1);
        let not_found = reg.find_by_capability("nonexistent");
        assert!(not_found.is_empty());
    }

    #[test]
    fn test_unregister() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent("agent_a"));
        assert!(reg.unregister("agent_a").is_some());
        assert!(reg.is_empty());
    }
}
