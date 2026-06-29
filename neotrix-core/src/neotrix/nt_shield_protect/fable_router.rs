use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};

/// Fable 5 security tiers — strict atomic delivery, fail-secure for Unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FableTier {
    Root,
    Kernel,
    User,
    Unknown,
}

impl FableTier {
    pub fn name(&self) -> &'static str {
        match self {
            FableTier::Root => "root",
            FableTier::Kernel => "kernel",
            FableTier::User => "user",
            FableTier::Unknown => "unknown",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            FableTier::Root => 3,
            FableTier::Kernel => 2,
            FableTier::User => 1,
            FableTier::Unknown => 0,
        }
    }
}

/// A Fable 5 security packet with metadata and payload.
#[derive(Debug, Clone)]
pub struct FablePacket {
    pub id: u64,
    pub source_tier: FableTier,
    pub target_tier: FableTier,
    pub operation: String,
    pub payload_vsa: Vec<u8>,
    pub integrity_hash: u64,
    pub is_atomic: bool,
}

impl FablePacket {
    pub fn new(id: u64, source: FableTier, target: FableTier, operation: &str) -> Self {
        Self {
            id,
            source_tier: source,
            target_tier: target,
            operation: operation.to_string(),
            payload_vsa: QuantizedVSA::random_vector(),
            integrity_hash: 0,
            is_atomic: true,
        }
    }

    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload_vsa = payload;
        self
    }

    pub fn default_decision(&self) -> FableRoutingDecision {
        if self.source_tier == FableTier::Unknown || self.target_tier == FableTier::Unknown {
            return FableRoutingDecision::Deny("unknown tier — fail secure".to_string());
        }
        if self.source_tier.priority() >= self.target_tier.priority() {
            FableRoutingDecision::Allow
        } else {
            FableRoutingDecision::Deny(format!(
                "insufficient privilege: {:?} → {:?}",
                self.source_tier, self.target_tier
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub enum FableRoutingDecision {
    Allow,
    Deny(String),
    Escalate(String),
}

/// VSA-native Fable 5 router: uses VSA similarity to tier profile vectors for
/// content-aware routing decisions.
#[derive(Debug, Clone)]
pub struct VsaFableRouter {
    root_profile: Vec<u8>,
    kernel_profile: Vec<u8>,
    user_profile: Vec<u8>,
    decisions: Vec<(u64, FableRoutingDecision)>,
    max_history: usize,
    pub monitor_only: bool,
    packets_routed: u64,
    packets_denied: u64,
}

impl VsaFableRouter {
    pub fn new() -> Self {
        Self {
            root_profile: QuantizedVSA::seeded_random(0xFAB1_E001, VSA_DIM),
            kernel_profile: QuantizedVSA::seeded_random(0xFAB1_E002, VSA_DIM),
            user_profile: QuantizedVSA::seeded_random(0xFAB1_E003, VSA_DIM),
            decisions: Vec::with_capacity(128),
            max_history: 1000,
            monitor_only: false,
            packets_routed: 0,
            packets_denied: 0,
        }
    }

    pub fn tier_profile(&self, tier: FableTier) -> &[u8] {
        match tier {
            FableTier::Root => &self.root_profile,
            FableTier::Kernel => &self.kernel_profile,
            FableTier::User => &self.user_profile,
            FableTier::Unknown => &self.user_profile,
        }
    }

    pub fn route(&mut self, packet: &FablePacket) -> FableRoutingDecision {
        self.packets_routed += 1;

        if packet.source_tier == FableTier::Unknown {
            self.packets_denied += 1;
            let d = FableRoutingDecision::Deny("unknown source tier — fail secure".to_string());
            self.record(packet.id, &d);
            return d;
        }
        if packet.target_tier == FableTier::Unknown {
            self.packets_denied += 1;
            let d = FableRoutingDecision::Deny("unknown target tier — fail secure".to_string());
            self.record(packet.id, &d);
            return d;
        }

        let sim = QuantizedVSA::cosine(&packet.payload_vsa, self.tier_profile(packet.target_tier));

        if sim < 0.3 {
            let d = FableRoutingDecision::Escalate(format!(
                "VSA content mismatch (sim={:.3}) for {:?}",
                sim, packet.target_tier
            ));
            self.record(packet.id, &d);
            return d;
        }

        let decision = packet.default_decision();
        match &decision {
            FableRoutingDecision::Deny(_) => {
                self.packets_denied += 1;
                if self.monitor_only {
                    self.record(packet.id, &decision);
                    return FableRoutingDecision::Allow;
                }
            }
            _ => {}
        }
        self.record(packet.id, &decision);
        decision
    }

    fn record(&mut self, id: u64, decision: &FableRoutingDecision) {
        if self.decisions.len() >= self.max_history {
            self.decisions.remove(0);
        }
        self.decisions.push((id, decision.clone()));
    }

    pub fn stats(&self) -> FableRouterStats {
        let denied = self
            .decisions
            .iter()
            .filter(|(_, d)| matches!(d, FableRoutingDecision::Deny(_)))
            .count();
        let escalated = self
            .decisions
            .iter()
            .filter(|(_, d)| matches!(d, FableRoutingDecision::Escalate(_)))
            .count();
        FableRouterStats {
            packets_routed: self.packets_routed,
            packets_denied: self.packets_denied,
            recent_denials: denied,
            recent_escalations: escalated,
            monitor_only: self.monitor_only,
        }
    }

    pub fn reset_stats(&mut self) {
        self.decisions.clear();
        self.packets_routed = 0;
        self.packets_denied = 0;
    }
}

impl Default for VsaFableRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FableRouterStats {
    pub packets_routed: u64,
    pub packets_denied: u64,
    pub recent_denials: usize,
    pub recent_escalations: usize,
    pub monitor_only: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_priority() {
        assert!(FableTier::Root.priority() > FableTier::Kernel.priority());
        assert!(FableTier::Kernel.priority() > FableTier::User.priority());
        assert_eq!(FableTier::Unknown.priority(), 0);
    }

    #[test]
    fn test_fable_packet_default_decision() {
        let r = FablePacket::new(1, FableTier::Root, FableTier::Kernel, "syscall");
        assert!(matches!(r.default_decision(), FableRoutingDecision::Allow));

        let u = FablePacket::new(2, FableTier::User, FableTier::Root, "raw_mem");
        assert!(matches!(
            u.default_decision(),
            FableRoutingDecision::Deny(_)
        ));

        let uk = FablePacket::new(3, FableTier::Unknown, FableTier::User, "unknown");
        assert!(matches!(
            uk.default_decision(),
            FableRoutingDecision::Deny(_)
        ));
    }

    #[test]
    fn test_route_root_to_kernel_allowed() {
        let mut router = VsaFableRouter::new();
        let p = FablePacket::new(1, FableTier::Root, FableTier::Kernel, "syscall");
        assert!(matches!(router.route(&p), FableRoutingDecision::Allow));
    }

    #[test]
    fn test_route_user_to_root_denied() {
        let mut router = VsaFableRouter::new();
        let p = FablePacket::new(2, FableTier::User, FableTier::Root, "raw_mem");
        assert!(matches!(router.route(&p), FableRoutingDecision::Deny(_)));
    }

    #[test]
    fn test_route_unknown_denied() {
        let mut router = VsaFableRouter::new();
        let p = FablePacket::new(3, FableTier::Unknown, FableTier::User, "unknown");
        assert!(matches!(router.route(&p), FableRoutingDecision::Deny(_)));
    }

    #[test]
    fn test_route_unknown_target_denied() {
        let mut router = VsaFableRouter::new();
        let p = FablePacket::new(4, FableTier::Root, FableTier::Unknown, "x");
        assert!(matches!(router.route(&p), FableRoutingDecision::Deny(_)));
    }

    #[test]
    fn test_stats() {
        let mut router = VsaFableRouter::new();
        router.route(&FablePacket::new(
            1,
            FableTier::Root,
            FableTier::Kernel,
            "ok",
        ));
        router.route(&FablePacket::new(
            2,
            FableTier::User,
            FableTier::Root,
            "deny",
        ));
        router.route(&FablePacket::new(
            3,
            FableTier::Unknown,
            FableTier::User,
            "deny",
        ));
        let s = router.stats();
        assert_eq!(s.packets_routed, 3);
        assert_eq!(s.packets_denied, 2);
    }

    #[test]
    fn test_monitor_mode_bypasses() {
        let mut router = VsaFableRouter::new();
        router.monitor_only = true;
        let p = FablePacket::new(2, FableTier::User, FableTier::Root, "raw_mem");
        assert!(matches!(router.route(&p), FableRoutingDecision::Allow));
    }

    #[test]
    fn test_reset_stats() {
        let mut router = VsaFableRouter::new();
        router.route(&FablePacket::new(
            1,
            FableTier::Root,
            FableTier::Kernel,
            "ok",
        ));
        router.reset_stats();
        assert_eq!(router.stats().packets_routed, 0);
    }

    #[test]
    fn test_packet_builder() {
        let p = FablePacket::new(5, FableTier::Kernel, FableTier::User, "ioctl")
            .with_payload(vec![1, 2, 3, 4]);
        assert_eq!(p.id, 5);
        assert_eq!(p.source_tier, FableTier::Kernel);
        assert_eq!(p.payload_vsa, vec![1, 2, 3, 4]);
        assert!(p.is_atomic);
    }
}
