use std::collections::HashMap;

/// Match quality for capability routing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityMatch {
    Exact,
    Prefix,
    Fuzzy,
    None,
}

/// Semantic capability router with priority matching
pub struct SemanticRouter {
    capabilities: HashMap<String, Vec<String>>,
}

impl Default for SemanticRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticRouter {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    pub fn register_agent(&mut self, agent_id: &str, caps: Vec<String>) {
        self.capabilities.insert(agent_id.to_string(), caps);
    }

    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.capabilities.remove(agent_id);
    }

    /// Find the best agent for a capability, with fallback: Exact -> Prefix -> Fuzzy -> None
    pub fn find_best_match(&self, capability: &str) -> Option<(String, CapabilityMatch)> {
        for (agent_id, caps) in &self.capabilities {
            if caps.iter().any(|c| c == capability) {
                return Some((agent_id.clone(), CapabilityMatch::Exact));
            }
        }

        let prefix = capability.split('_').next().unwrap_or("");
        if !prefix.is_empty() {
            for (agent_id, caps) in &self.capabilities {
                if caps.iter().any(|c| c.starts_with(prefix)) {
                    return Some((agent_id.clone(), CapabilityMatch::Prefix));
                }
            }
        }

        let words: Vec<&str> = capability.split('_').collect();
        for (agent_id, caps) in &self.capabilities {
            if caps.iter().any(|c| words.iter().any(|w| c.contains(w))) {
                return Some((agent_id.clone(), CapabilityMatch::Fuzzy));
            }
        }

        None
    }

    pub fn all_agents(&self) -> Vec<String> {
        self.capabilities.keys().cloned().collect()
    }

    pub fn summary(&self) -> String {
        let total: usize = self.capabilities.values().map(|v| v.len()).sum();
        format!(
            "{} agents, {} total capabilities",
            self.capabilities.len(),
            total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let mut router = SemanticRouter::new();
        router.register_agent(
            "agent-1",
            vec!["code_generation".into(), "code_review".into()],
        );
        let result = router.find_best_match("code_generation");
        assert_eq!(result, Some(("agent-1".into(), CapabilityMatch::Exact)));
    }

    #[test]
    fn test_prefix_match() {
        let mut router = SemanticRouter::new();
        router.register_agent("agent-2", vec!["code_review".into()]);
        let result = router.find_best_match("code_generation");
        assert_eq!(result, Some(("agent-2".into(), CapabilityMatch::Prefix)));
    }

    #[test]
    fn test_no_match() {
        let router = SemanticRouter::new();
        let result = router.find_best_match("anything");
        assert_eq!(result, None);
    }

    #[test]
    fn test_unregister_agent() {
        let mut router = SemanticRouter::new();
        router.register_agent("agent-1", vec!["test".into()]);
        assert_eq!(router.all_agents().len(), 1);
        router.unregister_agent("agent-1");
        assert!(router.all_agents().is_empty());
    }
}
