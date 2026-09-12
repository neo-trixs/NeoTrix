//! NT-ACT SEO — 搜索引擎优化分析与内容可见性能力叶 (T15 R-P42 新芽)。
//!
//! 这是一次最小化新芽 (bud): 仅声明结构 + 接线桩, 真实行为留 TODO。
//! 强化既有 NT-ACT 域, 不平行新建适配器模块。

#![forbid(unsafe_code)]

use nt_core_capability_tree::node::{CapabilityNode, ConstellationLevel, Domain, NodeLayer};
use nt_core_capability_tree::registry::{CapabilityRegistry, RegistryError};
use serde_json::Value as Json;

/// SEO 分析器 — 内容可见性/关键词/排名分析的占位叶。
#[derive(Debug, Clone, Default)]
pub(crate) struct SeoAnalyzer {
    /// 已分析页面数 (telemetry, TODO 填充)
    pub analyzed_pages: u64,
}

impl SeoAnalyzer {
    /// 构造空 SEO 分析器 (fresh bud)。
    pub fn new() -> Self {
        Self { analyzed_pages: 0 }
    }

    /// TODO: 实现真实 SEO 分析行为 (NT-ACT 生产接线点)。
    ///
    /// 设计契约: 输入 = 内容/URL; 输出 = 可见性评分 + 建议;
    /// fallback = 无索引数据时回退至启发式基线。当前为桩, 返回未实现错误。
    pub fn analyze(&self, _content: &str) -> Result<String, String> {
        // TODO(T15): 接入 nt_act_* 既有分析与发布路径, 实现真实 SEO 行为。
        Err("nt_act_seo::SeoAnalyzer::analyze not yet implemented (fresh bud)".into())
    }
}

/// 将 SEO 能力节点注册进能力树 (C1 新芽, 域 NT-ACT)。
///
/// 镜像 `nt_core_capability_tree::cad_node::register_cad_capability` 模式:
/// 构造 `CapabilityNode` 后 `tree.register(node)`。wiring_evidence 标注为新芽。
pub fn register_capability(tree: &mut CapabilityRegistry) -> Result<(), RegistryError> {
    let mut node = CapabilityNode::new_primitive(
        "nt_act::seo::analyze".into(),
        Domain::Act,
        vec!["seo.analyze".into(), "seo.visibility".into()],
    );
    node.layer = NodeLayer::L1Composite;
    node.constellation = ConstellationLevel::C1UnitTest;
    node.metadata.insert(
        "wiring_evidence".into(),
        Json::String(
            "fresh bud (T15 R-P42): nt_act_seo::SeoAnalyzer::analyze — not yet wired to production"
                .into(),
        ),
    );
    tree.register(node)
}
