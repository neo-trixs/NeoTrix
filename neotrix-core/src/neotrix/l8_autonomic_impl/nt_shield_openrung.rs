//! NT-SHIELD OpenRung — 反审查志愿中继网络接入 (stealth net)
//!
//! 吸收源: github.com/openrung/openrung
//! 能力: 反审查志愿中继网络 → broker 匹配 + volunteer relay + Xray-core
//!       VLESS/REALITY → 映射 NeoTrix NT-SHIELD stealth net / proxy pool。
//!
//! 这是对吸收源的模式吸收 (C1 成熟度): trait + broker/relay 模型 stub +
//! stealth net 接入 stub。编译通过即可 (R-P1: unsafe 禁用)。

use crate::core::nt_core_self_test::SelfTest;

/// 中继节点角色。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayRole {
    /// 撮合志愿中继与请求者的中间人 (broker)。
    Broker,
    /// 志愿中继 (实际转发流量)。
    Volunteer,
    /// 接入客户端 (NeoTrix stealth net 侧)。
    Client,
}

impl RelayRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelayRole::Broker => "broker",
            RelayRole::Volunteer => "volunteer",
            RelayRole::Client => "client",
        }
    }
}

/// 底层传输协议 (Xray-core 系)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportProtocol {
    /// VLESS + REALITY (抗审查首选)。
    VlessReality,
    /// 其他 / 未知。
    Other,
}

impl TransportProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportProtocol::VlessReality => "vless_reality",
            TransportProtocol::Other => "other",
        }
    }
}

/// 单个中继节点 (stub 模型)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayNode {
    pub id: String,
    pub role: RelayRole,
    pub transport: TransportProtocol,
    pub region: String,
}

impl RelayNode {
    pub fn new(id: &str, role: RelayRole, transport: TransportProtocol, region: &str) -> Self {
        Self {
            id: id.to_string(),
            role,
            transport,
            region: region.to_string(),
        }
    }
}

/// 反审查中继网络核心 trait — broker 匹配 + relay 接入 stub。
/// 映射 NeoTrix NT-SHIELD stealth net / proxy pool 接入点。
pub trait AntiCensorRelay {
    /// 登记一个中继节点 (broker / volunteer)。
    fn register_node(&mut self, node: RelayNode);
    /// 通过 broker 匹配一组可用志愿中继 (按 region 过滤)。
    fn match_relays(&self, region: &str) -> Vec<&RelayNode>;
    /// 当前网络内节点总数。
    fn node_count(&self) -> usize;
}

/// 默认 OpenRung 接入实现: 内存 broker/relay 索引。
#[derive(Debug, Default)]
pub struct OpenRungRelay {
    nodes: Vec<RelayNode>,
}

impl OpenRungRelay {
    pub fn new() -> Self {
        Self::default()
    }

    /// 种子网络: 从吸收源归纳的 broker + volunteer 拓扑 stub。
    pub fn with_seed() -> Self {
        let mut r = Self::new();
        r.register_node(RelayNode::new(
            "broker-1",
            RelayRole::Broker,
            TransportProtocol::VlessReality,
            "global",
        ));
        r.register_node(RelayNode::new(
            "relay-eu-1",
            RelayRole::Volunteer,
            TransportProtocol::VlessReality,
            "eu",
        ));
        r.register_node(RelayNode::new(
            "relay-ap-1",
            RelayRole::Volunteer,
            TransportProtocol::VlessReality,
            "ap",
        ));
        r
    }
}

impl AntiCensorRelay for OpenRungRelay {
    fn register_node(&mut self, node: RelayNode) {
        self.nodes.push(node);
    }

    fn match_relays(&self, region: &str) -> Vec<&RelayNode> {
        self.nodes
            .iter()
            .filter(|n| n.role == RelayRole::Volunteer && n.region == region)
            .collect()
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl SelfTest for OpenRungRelay {
    fn name(&self) -> &str {
        "nt_shield_openrung"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.node_count() == 0 {
            failures.push("relay network has no seeded nodes".to_string());
        }
        // broker 是匹配的前提: 无 broker 则 stealth net 无法撮合。
        let has_broker = self.nodes.iter().any(|n| n.role == RelayRole::Broker);
        if !has_broker {
            failures.push("relay network missing broker node".to_string());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_registers_nodes() {
        let mut r = OpenRungRelay::new();
        r.register_node(RelayNode::new("v1", RelayRole::Volunteer, TransportProtocol::VlessReality, "eu"));
        assert_eq!(r.node_count(), 1);
    }

    #[test]
    fn relay_matches_by_region() {
        let r = OpenRungRelay::with_seed();
        let ap = r.match_relays("ap");
        assert_eq!(ap.len(), 1);
        assert_eq!(ap[0].id, "relay-ap-1");
        let eu = r.match_relays("eu");
        assert_eq!(eu.len(), 1);
    }

    #[test]
    fn role_str_roundtrip() {
        assert_eq!(RelayRole::Broker.as_str(), "broker");
        assert_eq!(TransportProtocol::VlessReality.as_str(), "vless_reality");
        assert_eq!(OpenRungRelay::with_seed().node_count(), 3);
    }
}
