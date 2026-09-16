//! Social Access 能力节点 — 跨平台社交媒体登录、Feed 获取与聚合
//!
//! 将 social_access 能力注册为能力树的正式 CAPABILITY NODE, 使其参与
//! C1→C2 晋升门禁 (D16 自欺防线): `provides` 非空 + `wiring_evidence`
//! + `evidence_gated='passed'`。
//!
//! - 域 (X): NT-WORLD (`Domain::World`)
//! - 层 (Z): L2World (社交媒体感知层)
//! - 星座 (Y): C0Compile (初始注册)
//! - Rune 槽 (1): Crimson (数据摄取)

use crate::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer, RuneSocket};
use crate::registry::{CapabilityRegistry, RegistryError};

/// Social Access 能力节点描述符。
///
/// 镜像 `CapabilityNode` 契约字段, 作为调用方 introspect 的稳定接口;
/// 实际注册经 [`SocialAccessCapabilityNode::build`] 构建真实 `CapabilityNode`。
#[derive(Default)]
pub struct SocialAccessCapabilityNode;

impl SocialAccessCapabilityNode {
    /// 全局唯一节点 ID
    pub fn node_id(&self) -> &'static str {
        "nt_world::social_access"
    }

    /// 提供的能力标签
    pub fn provides(&self) -> Vec<&'static str> {
        vec![
            "social.login",
            "social.feed.retrieve",
            "social.post.unify",
            "social.metrics.aggregate",
            "social.session.manage",
        ]
    }

    /// 依赖的能力标签
    pub fn requires(&self) -> Vec<&'static str> {
        vec!["http.client", "json.parse"]
    }

    /// 占用的 Rune 槽 (C0: 1 槽 Crimson — 数据摄取)
    pub fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson]
    }

    /// 星座成熟度: C0 (初始注册)
    pub fn constellation_level(&self) -> ConstellationLevel {
        ConstellationLevel::C0Compile
    }

    /// 构建真实 `CapabilityNode`
    pub fn build(&self) -> CapabilityNode {
        let mut node = CapabilityNode::new_composite(
            self.node_id().to_string(),
            Domain::World,
            NodeLayer::L2World,
            self.provides().into_iter().map(|s| s.to_string()).collect(),
            self.requires().into_iter().map(|s| s.to_string()).collect(),
        );
        node.constellation = self.constellation_level();
        node.rune_sockets = self.rune_sockets();
        node.metadata.insert(
            "description".into(),
            serde_json::Value::String(
                "Universal social media login, feed retrieval, and cross-platform aggregation"
                    .into(),
            ),
        );
        node.metadata.insert(
            "platforms".into(),
            serde_json::json!(["Twitter", "Reddit", "Instagram", "TikTok", "Youtube", "Linkedin"]),
        );
        node
    }
}

/// 将 Social Access 能力节点注册进能力树
pub fn register_social_access_capability(
    tree: &mut CapabilityRegistry,
) -> Result<(), RegistryError> {
    tree.register(SocialAccessCapabilityNode.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn social_access_node_builds_correctly() {
        let node = SocialAccessCapabilityNode.build();
        assert_eq!(node.id, "nt_world::social_access");
        assert_eq!(node.domain, Domain::World);
        assert_eq!(node.layer, NodeLayer::L2World);
        assert_eq!(node.constellation, ConstellationLevel::C0Compile);
        assert!(!node.provides.is_empty());
        assert_eq!(node.provides.len(), 5);
        assert_eq!(node.rune_sockets, vec![RuneSocket::Crimson]);
    }

    #[test]
    fn social_access_registers_in_registry() {
        let mut reg = CapabilityRegistry::new();
        register_social_access_capability(&mut reg).expect("register social_access capability");
        assert!(reg.get("nt_world::social_access").is_some());
    }
}
