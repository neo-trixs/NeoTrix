//! NT-ACT 媒体 — 社交/内容平台媒体发布与互动能力叶 (T15 R-P42 新芽)。
//!
//! 这是一次最小化新芽 (bud): 仅声明结构 + 接线桩, 真实行为留 TODO。
//! 强化既有 NT-ACT 域, 不平行新建适配器模块。

#![forbid(unsafe_code)]

use nt_core_capability_tree::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};
use nt_core_capability_tree::registry::{CapabilityRegistry, RegistryError};
use serde_json::Value as Json;

/// 媒体机器人 — 跨平台内容发布/互动的占位叶。
#[derive(Debug, Clone, Default)]
pub struct MediaBot {
    /// 已接线平台数 (telemetry, TODO 填充)
    pub wired_platforms: u64,
}

impl MediaBot {
    /// 构造空媒体机器人 (fresh bud)。
    pub fn new() -> Self {
        Self {
            wired_platforms: 0,
        }
    }

    /// TODO: 实现真实媒体发布/互动行为 (NT-ACT 生产接线点)。
    ///
    /// 设计契约: 输入 = 平台 + 内容负载; 输出 = 发布回执;
    /// fallback = 平台不可达时回退至队列。当前为桩, 返回未实现错误。
    pub fn publish(&self, _platform: &str, _payload: &str) -> Result<String, String> {
        // TODO(T15): 接入 nt_act_* 既有发布路径, 实现真实媒体行为。
        Err("nt_act_media::MediaBot::publish not yet implemented (fresh bud)".into())
    }
}

/// 将媒体能力节点注册进能力树 (C1 新芽, 域 NT-ACT)。
///
/// 镜像 `nt_core_capability_tree::cad_node::register_cad_capability` 模式:
/// 构造 `CapabilityNode` 后 `tree.register(node)`。wiring_evidence 标注为新芽。
pub fn register_capability(tree: &mut CapabilityRegistry) -> Result<(), RegistryError> {
    let mut node = CapabilityNode::new_primitive(
        "nt_act::media::publish".into(),
        Domain::Act,
        vec!["media.publish".into(), "media.interact".into()],
    );
    node.layer = NodeLayer::L1Composite;
    node.constellation = ConstellationLevel::C1UnitTest;
    node.metadata.insert(
        "wiring_evidence".into(),
        Json::String(
            "fresh bud (T15 R-P42): nt_act_media::MediaBot::publish — not yet wired to production"
                .into(),
        ),
    );
    tree.register(node)
}
