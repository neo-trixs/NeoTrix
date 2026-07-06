#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityRing { Ring0, Ring1, Ring2, Ring3 }

impl SecurityRing {
    pub fn label(&self) -> &str {
        match self { SecurityRing::Ring0 => "kernel", SecurityRing::Ring1 => "system", SecurityRing::Ring2 => "user", SecurityRing::Ring3 => "untrusted" }
    }
    pub fn allows_escalation_to(&self, target: SecurityRing) -> bool {
        (*self as u8) <= (target as u8)
    }
}

pub struct AgentTier { pub ring: SecurityRing, pub capabilities: Vec<String> }

impl AgentTier {
    pub fn new(ring: SecurityRing) -> Self { AgentTier { ring, capabilities: Vec::new() } }
    pub fn can_access(&self, resource_ring: SecurityRing) -> bool { self.ring.allows_escalation_to(resource_ring) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_ring_label_kernel() { assert_eq!(SecurityRing::Ring0.label(), "kernel"); }
    #[test] fn test_ring_label_untrusted() { assert_eq!(SecurityRing::Ring3.label(), "untrusted"); }
    #[test] fn test_ring_escalation_same() { assert!(SecurityRing::Ring0.allows_escalation_to(SecurityRing::Ring0)); }
    #[test] fn test_ring_escalation_higher() { assert!(SecurityRing::Ring0.allows_escalation_to(SecurityRing::Ring3)); }
    #[test] fn test_agent_tier_new() { let t = AgentTier::new(SecurityRing::Ring1); assert_eq!(t.ring, SecurityRing::Ring1); }
}
