//! L2 Perception Layer — trait abstractions for decoupled dependency
//!
//! 定义 L2 感知层对 L1 最小接口依赖的 trait，
//! 打断 L2→L1 concrete type 直接依赖。
//! 实现留在 L1，此处仅定义 trait contract。

pub use neotrix_types::knowledge_access::KnowledgeNode;
pub use crate::l2_perception::nt_world::l1_facade::{NodeType, KnowledgeBase};

/// L2 感知层对 KB 的最小读写接口 — 数据源入库只依赖此 trait，不依赖 KnowledgeBase concrete type。
pub trait KnowledgeStore: Send + Sync {
    /// 按 URL 查找节点 (去重用)。
    fn find_node_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String>;
    /// 插入或复用节点 (幂等写入)。
    fn insert_or_get_node(
        &self,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String>;
}

impl KnowledgeStore for KnowledgeBase {
    fn find_node_by_url(&self, url: &str) -> Result<Option<KnowledgeNode>, String> {
        KnowledgeBase::find_node_by_url(self, url)
    }
    fn insert_or_get_node(
        &self,
        title: &str,
        node_type: NodeType,
        summary: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
    ) -> Result<String, String> {
        KnowledgeBase::insert_or_get_node(
            self, title, node_type, summary, url, domain,
        )
    }
}
