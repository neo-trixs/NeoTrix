//! NT-SHIELD 侦察 — 资产/攻击面侦察能力叶 (T15 R-P42 新芽)。
//!
//! 这是一次最小化新芽 (bud): 仅声明结构 + 接线桩, 真实行为留 TODO。
//! 强化既有 NT-SHIELD 域, 不平行新建适配器模块。

#![forbid(unsafe_code)]

use nt_core_capability_tree::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};
use nt_core_capability_tree::registry::{CapabilityRegistry, RegistryError};
use serde_json::Value as Json;

/// 侦察扫描器 — 资产枚举/攻击面测绘的占位叶。
#[derive(Debug, Clone, Default)]
pub struct _ReconScanner {
    /// 已扫描目标数 (telemetry, TODO 填充)
    pub scanned_targets: u64,
}

impl _ReconScanner {
    /// 构造空侦察扫描器 (fresh bud)。
    pub fn new() -> Self {
        Self {
            scanned_targets: 0,
        }
    }

    /// 执行侦察扫描 — 资产枚举/攻击面测绘。
    ///
    /// STUB: Returns "not yet implemented" error. No actual scanning.
    /// Real implementation needs:
    /// - Integration with existing nt_shield_* recon/audit paths
    /// - Target scope validation and permission checks
    /// - Passive enumeration fallback for restricted permissions
    /// - Asset inventory output (hosts, services, open ports)
    /// - Attack surface scoring and prioritization
    pub fn scan(&self, _target: &str) -> Result<String, String> {
        // TODO(T15): 接入 nt_shield_* 既有侦察/审计路径, 实现真实侦察行为。
        Err("nt_shield_recon::_ReconScanner::scan not yet implemented (fresh bud)".into())
    }
}

/// 将侦察能力节点注册进能力树 (C1 新芽, 域 NT-SHIELD)。
///
/// 镜像 `nt_core_capability_tree::cad_node::register_cad_capability` 模式:
/// 构造 `CapabilityNode` 后 `tree.register(node)`。wiring_evidence 标注为新芽。
pub fn register_capability(tree: &mut CapabilityRegistry) -> Result<(), RegistryError> {
    let mut node = CapabilityNode::new_primitive(
        "nt_shield::recon::scan".into(),
        Domain::Shield,
        vec!["shield.recon.scan".into(), "shield.recon.enumerate".into()],
    );
    node.layer = NodeLayer::L1Composite;
    node.constellation = ConstellationLevel::C1UnitTest;
    node.metadata.insert(
        "wiring_evidence".into(),
        Json::String(
            "fresh bud (T15 R-P42): nt_shield_recon::_ReconScanner::scan — not yet wired to production"
                .into(),
        ),
    );
    tree.register(node)
}
