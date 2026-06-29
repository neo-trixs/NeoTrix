#![forbid(unsafe_code)]

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use super::a2a::ProtocolBinding;

// ── ProtocolVersion ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ProtocolVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(format!("invalid version format: {}", s));
        }
        let major = parts[0]
            .parse::<u16>()
            .map_err(|e| format!("invalid major version: {}", e))?;
        let minor = parts[1]
            .parse::<u16>()
            .map_err(|e| format!("invalid minor version: {}", e))?;
        Ok(Self { major, minor })
    }
}

impl PartialOrd for ProtocolVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProtocolVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
    }
}

// ── CapabilityFlag ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum CapabilityFlag {
    TaskSend = 0,
    TaskGet = 1,
    TaskCancel = 2,
    TaskStream = 3,
    TaskSubscribe = 4,
    AgentCard = 5,
    SignedCard = 6,
    BatchTasks = 7,
    LatencyBroadcast = 8,
    Streaming = 9,
    PushNotification = 10,
}

impl CapabilityFlag {
    pub fn all() -> &'static [CapabilityFlag] {
        &[
            CapabilityFlag::TaskSend,
            CapabilityFlag::TaskGet,
            CapabilityFlag::TaskCancel,
            CapabilityFlag::TaskStream,
            CapabilityFlag::TaskSubscribe,
            CapabilityFlag::AgentCard,
            CapabilityFlag::SignedCard,
            CapabilityFlag::BatchTasks,
            CapabilityFlag::LatencyBroadcast,
            CapabilityFlag::Streaming,
            CapabilityFlag::PushNotification,
        ]
    }

    fn bit_mask(&self) -> u128 {
        1u128 << (*self as u32)
    }
}

// ── CapabilityVector ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityVector {
    pub bits: u128,
}

impl CapabilityVector {
    pub const fn new(bits: u128) -> Self {
        Self { bits }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn all() -> Self {
        let mut v = Self::empty();
        for flag in CapabilityFlag::all() {
            v.set(*flag);
        }
        v
    }

    pub fn supports(&self, cap: CapabilityFlag) -> bool {
        (self.bits & cap.bit_mask()) != 0
    }

    pub fn set(&mut self, cap: CapabilityFlag) {
        self.bits |= cap.bit_mask();
    }

    pub fn clear(&mut self, cap: CapabilityFlag) {
        self.bits &= !cap.bit_mask();
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    pub fn intersection(self, other: Self) -> Self {
        Self {
            bits: self.bits & other.bits,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

impl Default for CapabilityVector {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for CapabilityVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags: Vec<String> = CapabilityFlag::all()
            .iter()
            .filter(|flag| self.supports(**flag))
            .map(|flag| format!("{:?}", flag))
            .collect();
        write!(f, "[{}]", flags.join(", "))
    }
}

// ── NegotiationOffer ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegotiationOffer {
    #[serde(rename = "versions")]
    pub versions: Vec<ProtocolVersion>,
    #[serde(rename = "bindings")]
    pub bindings: Vec<ProtocolBinding>,
    #[serde(rename = "capabilities")]
    pub capabilities: CapabilityVector,
    #[serde(rename = "agentName")]
    pub agent_name: String,
    #[serde(rename = "negotiationId")]
    pub negotiation_id: String,
}

impl NegotiationOffer {
    pub fn new(
        versions: Vec<ProtocolVersion>,
        bindings: Vec<ProtocolBinding>,
        capabilities: CapabilityVector,
        agent_name: &str,
    ) -> Self {
        Self {
            versions,
            bindings,
            capabilities,
            agent_name: agent_name.to_string(),
            negotiation_id: Uuid::new_v4().to_string(),
        }
    }
}

// ── NegotiationResponse ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegotiationResponse {
    #[serde(rename = "selectedVersion")]
    pub selected_version: ProtocolVersion,
    #[serde(rename = "selectedBinding")]
    pub selected_binding: ProtocolBinding,
    #[serde(rename = "commonCapabilities")]
    pub common_capabilities: CapabilityVector,
    #[serde(rename = "negotiationId")]
    pub negotiation_id: String,
    #[serde(rename = "accepted")]
    pub accepted: bool,
}

// ── A2ANegotiator ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct A2ANegotiator {
    pub supported_versions: Vec<ProtocolVersion>,
    pub supported_bindings: Vec<ProtocolBinding>,
    pub capabilities: CapabilityVector,
    pub agent_name: String,
}

impl A2ANegotiator {
    pub fn new(
        supported_versions: Vec<ProtocolVersion>,
        supported_bindings: Vec<ProtocolBinding>,
        capabilities: CapabilityVector,
        agent_name: &str,
    ) -> Self {
        Self {
            supported_versions,
            supported_bindings,
            capabilities,
            agent_name: agent_name.to_string(),
        }
    }

    /// Negotiate with a remote offer: find highest common version, first
    /// common binding, intersect capabilities.  Returns `accepted: false`
    /// when no common version or binding exists.
    pub fn negotiate(&self, offer: &NegotiationOffer) -> NegotiationResponse {
        // Find highest mutually supported version (our sorted descending list)
        let selected_version = self
            .supported_versions
            .iter()
            .find(|our_ver| offer.versions.contains(our_ver))
            .cloned();

        // Find first mutually supported binding (our priority order)
        let selected_binding = self
            .supported_bindings
            .iter()
            .find(|our_binding| offer.bindings.contains(our_binding))
            .cloned();

        match (selected_version, selected_binding) {
            (Some(ver), Some(binding)) => {
                let common = self.capabilities.intersection(offer.capabilities);
                NegotiationResponse {
                    selected_version: ver,
                    selected_binding: binding,
                    common_capabilities: common,
                    negotiation_id: offer.negotiation_id.clone(),
                    accepted: true,
                }
            }
            (ver, binding) => NegotiationResponse {
                selected_version: ver.unwrap_or(ProtocolVersion::new(0, 0)),
                selected_binding: binding.unwrap_or(ProtocolBinding::JsonRpc),
                common_capabilities: CapabilityVector::empty(),
                negotiation_id: offer.negotiation_id.clone(),
                accepted: false,
            },
        }
    }

    /// Create a negotiation offer from our own capabilities.
    pub fn make_offer(&self) -> NegotiationOffer {
        NegotiationOffer::new(
            self.supported_versions.clone(),
            self.supported_bindings.clone(),
            self.capabilities,
            &self.agent_name,
        )
    }

    /// Verify that a response matches our offer: the selected version and
    /// binding are ones we support, and the negotiation_id echoes the offer.
    pub fn verify_response(
        &self,
        response: &NegotiationResponse,
        offer: &NegotiationOffer,
    ) -> bool {
        if response.negotiation_id != offer.negotiation_id {
            return false;
        }
        if !self.supported_versions.contains(&response.selected_version) {
            return false;
        }
        if !self.supported_bindings.contains(&response.selected_binding) {
            return false;
        }
        true
    }

    /// Convenience: A2A v1.2 agent with all capabilities.
    pub fn default_v1_2() -> Self {
        let versions = vec![
            ProtocolVersion::new(1, 2),
            ProtocolVersion::new(1, 1),
            ProtocolVersion::new(1, 0),
        ];
        let bindings = vec![
            ProtocolBinding::HttpJsonRest,
            ProtocolBinding::Grpc,
            ProtocolBinding::JsonRpc,
        ];
        Self::new(versions, bindings, CapabilityVector::all(), "neotrix")
    }

    /// Convenience: minimal A2A v1.0 agent.
    pub fn minimal_v1_0() -> Self {
        let versions = vec![ProtocolVersion::new(1, 0)];
        let bindings = vec![ProtocolBinding::JsonRpc];
        let mut caps = CapabilityVector::empty();
        caps.set(CapabilityFlag::TaskSend);
        caps.set(CapabilityFlag::TaskGet);
        caps.set(CapabilityFlag::AgentCard);
        Self::new(versions, bindings, caps, "neotrix-minimal")
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // -- ProtocolVersion ---------------------------------------------------

    #[test]
    fn test_version_display() {
        let v = ProtocolVersion::new(1, 2);
        assert_eq!(v.to_string(), "1.2");
    }

    #[test]
    fn test_version_from_str() {
        let v: ProtocolVersion = "1.2".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
    }

    #[test]
    fn test_version_from_str_invalid() {
        assert!("1".parse::<ProtocolVersion>().is_err());
        assert!("abc".parse::<ProtocolVersion>().is_err());
        assert!("1.2.3".parse::<ProtocolVersion>().is_err());
    }

    #[test]
    fn test_version_ordering() {
        let v10 = ProtocolVersion::new(1, 0);
        let v11 = ProtocolVersion::new(1, 1);
        let v12 = ProtocolVersion::new(1, 2);
        let v20 = ProtocolVersion::new(2, 0);

        assert!(v12 > v11);
        assert!(v11 > v10);
        assert!(v20 > v12);
        assert!(v10 < v11);
        assert_eq!(v11, v11);
    }

    // -- CapabilityVector --------------------------------------------------

    #[test]
    fn test_capability_set_and_supports() {
        let mut cv = CapabilityVector::empty();
        assert!(!cv.supports(CapabilityFlag::TaskSend));
        cv.set(CapabilityFlag::TaskSend);
        assert!(cv.supports(CapabilityFlag::TaskSend));
        assert!(!cv.supports(CapabilityFlag::TaskGet));
    }

    #[test]
    fn test_capability_clear() {
        let mut cv = CapabilityVector::all();
        cv.clear(CapabilityFlag::PushNotification);
        assert!(!cv.supports(CapabilityFlag::PushNotification));
        assert!(cv.supports(CapabilityFlag::TaskSend));
    }

    #[test]
    fn test_capability_intersection() {
        let mut a = CapabilityVector::empty();
        a.set(CapabilityFlag::TaskSend);
        a.set(CapabilityFlag::TaskGet);
        a.set(CapabilityFlag::TaskCancel);

        let mut b = CapabilityVector::empty();
        b.set(CapabilityFlag::TaskGet);
        b.set(CapabilityFlag::TaskCancel);
        b.set(CapabilityFlag::TaskStream);

        let c = a.intersection(b);
        assert!(!c.supports(CapabilityFlag::TaskSend));
        assert!(c.supports(CapabilityFlag::TaskGet));
        assert!(c.supports(CapabilityFlag::TaskCancel));
        assert!(!c.supports(CapabilityFlag::TaskStream));
    }

    #[test]
    fn test_capability_union() {
        let mut a = CapabilityVector::empty();
        a.set(CapabilityFlag::TaskSend);
        let mut b = CapabilityVector::empty();
        b.set(CapabilityFlag::TaskGet);
        let c = a.union(b);
        assert!(c.supports(CapabilityFlag::TaskSend));
        assert!(c.supports(CapabilityFlag::TaskGet));
    }

    // -- Negotiation -------------------------------------------------------

    fn make_negotiator_v12() -> A2ANegotiator {
        A2ANegotiator::default_v1_2()
    }

    #[test]
    fn test_negotiation_matching_versions() {
        let negotiator = make_negotiator_v12();
        let offer = NegotiationOffer::new(
            vec![ProtocolVersion::new(1, 2), ProtocolVersion::new(1, 0)],
            vec![ProtocolBinding::HttpJsonRest],
            CapabilityVector::all(),
            "remote",
        );
        let response = negotiator.negotiate(&offer);
        assert!(response.accepted);
        assert_eq!(response.selected_version, ProtocolVersion::new(1, 2));
        assert_eq!(response.selected_binding, ProtocolBinding::HttpJsonRest);
    }

    #[test]
    fn test_negotiation_partial_overlap() {
        let negotiator = make_negotiator_v12();
        let offer = NegotiationOffer::new(
            vec![ProtocolVersion::new(1, 1), ProtocolVersion::new(1, 0)],
            vec![ProtocolBinding::Grpc],
            CapabilityVector::all(),
            "remote",
        );
        let response = negotiator.negotiate(&offer);
        assert!(response.accepted);
        assert_eq!(response.selected_version, ProtocolVersion::new(1, 1));
        assert_eq!(response.selected_binding, ProtocolBinding::Grpc);
    }

    #[test]
    fn test_negotiation_no_common_version() {
        let negotiator = make_negotiator_v12();
        let offer = NegotiationOffer::new(
            vec![ProtocolVersion::new(2, 0)],
            vec![ProtocolBinding::HttpJsonRest],
            CapabilityVector::all(),
            "remote",
        );
        let response = negotiator.negotiate(&offer);
        assert!(!response.accepted);
    }

    #[test]
    fn test_negotiation_no_common_binding() {
        let negotiator = make_negotiator_v12();
        let offer = NegotiationOffer::new(
            vec![ProtocolVersion::new(1, 2)],
            vec![],
            CapabilityVector::all(),
            "remote",
        );
        let response = negotiator.negotiate(&offer);
        assert!(!response.accepted);
    }

    #[test]
    fn test_verify_response_passes() {
        let negotiator = make_negotiator_v12();
        let offer = negotiator.make_offer();
        let response = negotiator.negotiate(&offer);
        assert!(negotiator.verify_response(&response, &offer));
    }

    #[test]
    fn test_verify_response_fails_mismatched_id() {
        let negotiator = make_negotiator_v12();
        let offer = negotiator.make_offer();
        let mut response = negotiator.negotiate(&offer);
        response.negotiation_id = "wrong-id".into();
        assert!(!negotiator.verify_response(&response, &offer));
    }

    #[test]
    fn test_verify_response_fails_unsupported_version() {
        let negotiator = make_negotiator_v12();
        let offer = negotiator.make_offer();
        let mut response = negotiator.negotiate(&offer);
        response.selected_version = ProtocolVersion::new(3, 0);
        assert!(!negotiator.verify_response(&response, &offer));
    }

    #[test]
    fn test_default_v1_2_vs_minimal_v1_0() {
        let v12 = A2ANegotiator::default_v1_2();
        let v10 = A2ANegotiator::minimal_v1_0();

        assert_eq!(v12.supported_versions.len(), 3);
        assert_eq!(v10.supported_versions.len(), 1);
        assert!(v12.capabilities.supports(CapabilityFlag::TaskSubscribe));
        assert!(!v10.capabilities.supports(CapabilityFlag::TaskSubscribe));
        assert!(v12.supported_bindings.contains(&ProtocolBinding::Grpc));
        assert!(!v10.supported_bindings.contains(&ProtocolBinding::Grpc));
    }

    // -- Serialization -----------------------------------------------------

    #[test]
    fn test_negotiation_offer_roundtrip() {
        let offer = NegotiationOffer::new(
            vec![ProtocolVersion::new(1, 2), ProtocolVersion::new(1, 0)],
            vec![ProtocolBinding::HttpJsonRest, ProtocolBinding::JsonRpc],
            CapabilityVector::all(),
            "test-agent",
        );
        let json = serde_json::to_string(&offer).expect("serialize offer");
        let deserialized: NegotiationOffer =
            serde_json::from_str(&json).expect("deserialize offer");
        assert_eq!(deserialized.agent_name, "test-agent");
        assert_eq!(deserialized.versions.len(), 2);
        assert_eq!(deserialized.bindings.len(), 2);
        assert!(deserialized.capabilities.supports(CapabilityFlag::TaskSend));
        // negotiation_id is preserved across roundtrip
        assert_eq!(deserialized.negotiation_id, offer.negotiation_id);
    }

    #[test]
    fn test_negotiation_response_roundtrip() {
        let response = NegotiationResponse {
            selected_version: ProtocolVersion::new(1, 2),
            selected_binding: ProtocolBinding::Grpc,
            common_capabilities: CapabilityVector::all(),
            negotiation_id: "test-id".into(),
            accepted: true,
        };
        let json = serde_json::to_string(&response).expect("serialize response");
        let deserialized: NegotiationResponse =
            serde_json::from_str(&json).expect("deserialize response");
        assert_eq!(deserialized.selected_version, ProtocolVersion::new(1, 2));
        assert_eq!(deserialized.selected_binding, ProtocolBinding::Grpc);
        assert!(deserialized.accepted);
    }

    #[test]
    fn test_capability_vector_roundtrip() {
        let mut cv = CapabilityVector::empty();
        cv.set(CapabilityFlag::TaskSend);
        cv.set(CapabilityFlag::TaskGet);
        cv.set(CapabilityFlag::AgentCard);
        let json = serde_json::to_string(&cv).expect("serialize cv");
        let deserialized: CapabilityVector = serde_json::from_str(&json).expect("deserialize cv");
        assert!(deserialized.supports(CapabilityFlag::TaskSend));
        assert!(!deserialized.supports(CapabilityFlag::TaskCancel));
    }

    #[test]
    fn test_protocol_version_roundtrip() {
        let v = ProtocolVersion::new(1, 2);
        let json = serde_json::to_string(&v).expect("serialize version");
        let deserialized: ProtocolVersion =
            serde_json::from_str(&json).expect("deserialize version");
        assert_eq!(deserialized, ProtocolVersion::new(1, 2));
    }

    #[test]
    fn test_make_offer_contains_negotiation_id() {
        let negotiator = make_negotiator_v12();
        let offer = negotiator.make_offer();
        assert!(!offer.negotiation_id.is_empty());
        // UUID v4 format: 8-4-4-4-12 hex chars
        assert_eq!(offer.negotiation_id.len(), 36);
    }

    #[test]
    fn test_negotiation_capability_intersection() {
        let mut local_caps = CapabilityVector::empty();
        local_caps.set(CapabilityFlag::TaskSend);
        local_caps.set(CapabilityFlag::TaskGet);
        local_caps.set(CapabilityFlag::TaskStream);

        let mut remote_caps = CapabilityVector::empty();
        remote_caps.set(CapabilityFlag::TaskGet);
        remote_caps.set(CapabilityFlag::TaskStream);
        remote_caps.set(CapabilityFlag::TaskCancel);

        let negotiator = A2ANegotiator::new(
            vec![ProtocolVersion::new(1, 2)],
            vec![ProtocolBinding::HttpJsonRest],
            local_caps,
            "local",
        );
        let offer = NegotiationOffer::new(
            vec![ProtocolVersion::new(1, 2)],
            vec![ProtocolBinding::HttpJsonRest],
            remote_caps,
            "remote",
        );
        let response = negotiator.negotiate(&offer);
        assert!(response.accepted);
        assert!(response
            .common_capabilities
            .supports(CapabilityFlag::TaskGet));
        assert!(response
            .common_capabilities
            .supports(CapabilityFlag::TaskStream));
        assert!(!response
            .common_capabilities
            .supports(CapabilityFlag::TaskSend));
        assert!(!response
            .common_capabilities
            .supports(CapabilityFlag::TaskCancel));
    }
}
