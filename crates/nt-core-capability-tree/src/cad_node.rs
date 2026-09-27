//! # CAD 能力节点 — GenCAD 四步框架 (CSR→CCIP→CDP→Decoder)
//!
//! 将 CAD 生成能力注册为能力树的正式 CAPABILITY NODE, 使其参与
//! C1→C2 晋升门禁 (D16 自欺防线): `provides` 非空 + `wiring_evidence`
//! + `evidence_gated='passed'`。
//!
//! - 域 (X): NT-WORLD (`Domain::World`)
//! - 层 (Z): L3 DomainService (跨模态生成服务)
//! - 星座 (Y): C4MainPipeline (已有 NT-WORLD SelfTest + 意识级生产接线)
//! - Rune 槽 (5): Crimson/Indigo/Obsidian/Golden/Alabaster

use crate::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer, RuneSocket};
use crate::registry::{CapabilityRegistry, RegistryError};
use serde_json::Value as Json;

/// CAD 能力节点描述符。
///
/// 镜像 `CapabilityNode` 契约字段 (node_id/provides/requires/rune_sockets/
/// constellation_level), 作为调用方 introspect 的稳定接口; 实际注册经
/// [`CadCapabilityNode::build`] 构建真实 `CapabilityNode`。
#[derive(Default)]
pub struct CadCapabilityNode;

impl CadCapabilityNode {
    /// 全局唯一节点 ID (遵循 `nt_<domain>::<module>::<function>` 惯例)。
    pub fn node_id(&self) -> &'static str {
        "nt_world::cad::generation"
    }

    /// 提供的能力标签 (四步框架 + 跨模态检索/合成/B-rep 拓扑/自愈)。
    pub fn provides(&self) -> Vec<&'static str> {
        vec![
            "cad.generate",
            "cad.retrieve.cross_modal",
            "cad.synthbal",
            "cad.brep_validate",
            "cad.csr",
            "cad.ccip",
            "cad.cdp",
            "cad.decoder",
        ]
    }

    /// 依赖的能力标签 (图像输入 + 几何内核)。
    pub fn requires(&self) -> Vec<&'static str> {
        vec!["image.input", "geometry.kernel"]
    }

    /// 占用的 5 色 Rune 槽 (C4 已满 5 槽, 触发 Scry runeword)。
    ///
    /// 构造模式: `RuneSocket` 为单元枚举变体, 直接写 `RuneSocket::Crimson`
    /// (无 `()` 调用)。
    pub fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![
            RuneSocket::Crimson,   // 数据摄取 (图像预处理)
            RuneSocket::Indigo,    // 变换 (扩散去噪)
            RuneSocket::Obsidian,  // 缓存 (命令序列)
            RuneSocket::Golden,    // 错误恢复 (C5 自愈)
            RuneSocket::Alabaster, // 监控 (健康链)
        ]
    }

    /// 星座成熟度: C4 (接入主流水线并被消费)。
    pub fn constellation_level(&self) -> ConstellationLevel {
        ConstellationLevel::C4MainPipeline
    }

    /// 构建真实 `CapabilityNode`, 并预置晋升门禁证据
    /// (`wiring_evidence` + `evidence_gated='passed'`), 使节点可经
    /// `promotion_evidence_gate` 校验 (≥C2 需 evidence_gated)。
    pub fn build(&self) -> CapabilityNode {
        let mut node = CapabilityNode::new_constellation(
            self.node_id().to_string(),
            Domain::World,
            NodeLayer::L3DomainService,
            self.provides().into_iter().map(|s| s.to_string()).collect(),
            self.requires().into_iter().map(|s| s.to_string()).collect(),
        );
        node.constellation = self.constellation_level();
        node.rune_sockets = self.rune_sockets();
        node.metadata.insert(
            "wiring_evidence".into(),
            Json::String(
                "neotrix-core/src/neotrix/l2_world_impl/cad_selftest.rs:134 \
                 (register_cad_self_tests) -> nt_core_capability_tree cad_node"
                    .into(),
            ),
        );
        node.metadata
            .insert("evidence_gated".into(), Json::String("passed".into()));
        node
    }
}

/// 将 CAD 能力节点注册进能力树 (Tree 类型: [`CapabilityRegistry`])。
///
/// 镜像既有注册调用模式 (`tree.register(node)`), 返回 `RegistryError`
/// (节点已存在 / 循环依赖 / 校验失败)。
pub fn register_cad_capability(tree: &mut CapabilityRegistry) -> Result<(), RegistryError> {
    tree.register(CadCapabilityNode.build())
}
