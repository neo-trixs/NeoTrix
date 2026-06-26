use std::collections::HashMap;

use super::discovery::AgentInfo;
use crate::core::ReasoningHexagram;

/// Capability routing table for agent-to-agent communication
pub struct CapabilityRouter {
    /// local_capability → handler description
    pub local: HashMap<String, String>,
    /// remote agent ID → capabilities
    pub remote: HashMap<String, Vec<String>>,
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRouter {
    pub fn new() -> Self {
        Self {
            local: HashMap::new(),
            remote: HashMap::new(),
        }
    }

    /// Register a local capability
    pub fn register(&mut self, name: &str, description: &str) {
        self.local.insert(name.to_string(), description.to_string());
    }

    /// Register an agent's remote capabilities
    pub fn register_agent(&mut self, agent_id: &str, capabilities: Vec<String>) {
        self.remote.insert(agent_id.to_string(), capabilities);
    }

    /// Find which agent can handle a capability
    pub fn route(&self, capability: &str) -> Option<String> {
        if self.local.contains_key(capability) {
            return Some("local".to_string());
        }
        for (agent_id, caps) in &self.remote {
            if caps.contains(&capability.to_string()) {
                return Some(agent_id.clone());
            }
        }
        None
    }

    pub fn list_local(&self) -> Vec<String> {
        self.local.keys().cloned().collect()
    }

    pub fn can_handle(&self, capability: &str) -> bool {
        self.route(capability).is_some()
    }

    /// Filter agents whose hexagram != 0, sorted by hamming distance to the given hexagram.
    pub fn route_by_hexagram<'a>(
        &self,
        hexagram: ReasoningHexagram,
        agents: &'a [AgentInfo],
    ) -> Vec<&'a AgentInfo> {
        let mut candidates: Vec<&'a AgentInfo> =
            agents.iter().filter(|a| a.hexagram != 0).collect();
        candidates.sort_by_key(|a| hexagram.hamming_dist(&ReasoningHexagram(a.hexagram)));
        candidates
    }

    /// Returns the top N agents closest by hexagram hamming distance (nonzero only).
    pub fn find_hexagram_affinity<'a>(
        &self,
        hexagram: ReasoningHexagram,
        agents: &'a [AgentInfo],
        top_n: usize,
    ) -> Vec<&'a AgentInfo> {
        let mut candidates: Vec<&'a AgentInfo> =
            agents.iter().filter(|a| a.hexagram != 0).collect();
        candidates.sort_by_key(|a| hexagram.hamming_dist(&ReasoningHexagram(a.hexagram)));
        candidates.into_iter().take(top_n).collect()
    }

    pub fn summary(&self) -> String {
        let local = self.local.len();
        let remote_agents = self.remote.len();
        let remote_caps: usize = self.remote.values().map(|v| v.len()).sum();
        format!(
            "{} local, {} remote agents ({} caps)",
            local, remote_agents, remote_caps
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: &str, hexagram: u8) -> AgentInfo {
        AgentInfo {
            id: id.to_string(),
            name: format!("agent-{}", id),
            host: "127.0.0.1".into(),
            port: 42069,
            capabilities: vec![],
            hexagram,
            service_type: String::new(),
            instance_name: String::new(),
            last_seen: std::time::Instant::now(),
        }
    }

    #[test]
    fn test_agent_info_hexagram_roundtrip() {
        let info = make_agent("a1", 42);
        let json = serde_json::to_string(&info).expect("value should be ok in test");
        let deserialized: AgentInfo =
            serde_json::from_str(&json).expect("value should be ok in test");
        assert_eq!(deserialized.hexagram, 42);
        assert_eq!(deserialized.id, "a1");
    }

    #[test]
    fn test_route_by_hexagram_sorts_by_hamming() {
        let router = CapabilityRouter::new();
        let target = ReasoningHexagram(0b000000);
        let agents = vec![
            make_agent("far", 0b111111),   // hamming = 6
            make_agent("mid", 0b000011),   // hamming = 2
            make_agent("close", 0b000001), // hamming = 1
        ];
        let sorted = router.route_by_hexagram(target, &agents);
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].id, "close");
        assert_eq!(sorted[1].id, "mid");
        assert_eq!(sorted[2].id, "far");
    }

    #[test]
    fn test_route_by_hexagram_excludes_zero() {
        let router = CapabilityRouter::new();
        let target = ReasoningHexagram(5);
        let agents = vec![make_agent("a", 0), make_agent("b", 5), make_agent("c", 0)];
        let sorted = router.route_by_hexagram(target, &agents);
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].id, "b");
    }

    #[test]
    fn test_find_hexagram_affinity_top_n() {
        let router = CapabilityRouter::new();
        let target = ReasoningHexagram(0b101010);
        let agents = vec![
            make_agent("a", 0b101011), // hamming = 1
            make_agent("b", 0b101000), // hamming = 1
            make_agent("c", 0b000000), // hamming = 3
            make_agent("d", 0b111111), // hamming = 6
        ];
        let top2 = router.find_hexagram_affinity(target, &agents, 2);
        assert_eq!(top2.len(), 2);
        for a in &top2 {
            assert_ne!(a.hexagram, 0);
        }
    }
}
