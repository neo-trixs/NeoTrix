//! # NT-MEMORY 记忆操作五原语 (T0.2)
//!
//! 来源: 39 仓库吸收 — Awesome-AI-Memory 记忆系统词汇表
//! (write/retrieve/update/delete/compress 五原语 + 生命周期管理)。
//!
//! 现状: KB 的写/查/改/删/压缩操作隐式分散在 KnowledgeBase 各方法
//! (write_memory_entry / search / update_node_content / delete_node / compact)。
//! 本模块将其收敛为统一 trait 契约, 供上层 (NT-CORE 路由 / NT-MIND 进化 /
//! NT-WORLD 爬取) 以一致语义调用, 强化现有节点 (R-P42), 无平行适配器。

use super::nt_memory_types::{NodeType, SearchResult};
use super::KnowledgeBase;

/// 记忆操作五原语 — 统一记忆契约。
///
/// 对应 Awesome-AI-Memory 记忆操作原语:
/// - write:    写入 (统一写入弧, 含 SVAF 门禁 + 冲突检测 + graphrag 派生)
/// - retrieve: 检索 (统一 search 入口, PQ→semantic→hybrid 三级降级)
/// - update:   更新 (保留代际版本链 generation)
/// - delete:   删除 (级联删除关联边)
/// - compress: 压缩 (VACUUM + 可选过期清理)
pub trait MemoryPrimitives {
    /// write — 写入记忆, 返回主库节点 id (已存在则复用)。
    fn mem_write(
        &self,
        title: &str,
        node_type: NodeType,
        content: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
        evidence: Option<&serde_json::Value>,
    ) -> Result<String, String>;

    /// retrieve — 检索记忆, 返回排序后的搜索结果。
    fn mem_retrieve(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String>;

    /// update — 更新记忆内容 (保留代际版本链)。
    fn mem_update(&self, id: &str, content: &str) -> Result<(), String>;

    /// delete — 删除记忆, 返回是否实际删除。
    fn mem_delete(&self, id: &str) -> Result<bool, String>;

    /// compress — 压缩记忆 (VACUUM 回收物理空间), 返回 (清理节点数, 释放字节)。
    fn mem_compress(&self, prune_stale_days: Option<u32>) -> Result<(usize, i64), String>;
}

impl MemoryPrimitives for KnowledgeBase {
    fn mem_write(
        &self,
        title: &str,
        node_type: NodeType,
        content: Option<&str>,
        url: Option<&str>,
        domain: Option<&str>,
        evidence: Option<&serde_json::Value>,
    ) -> Result<String, String> {
        self.write_memory_entry(title, node_type, content, url, domain, evidence)
    }

    fn mem_retrieve(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        self.search(query, limit)
    }

    fn mem_update(&self, id: &str, content: &str) -> Result<(), String> {
        self.update_node_content(id, content)
    }

    fn mem_delete(&self, id: &str) -> Result<bool, String> {
        self.delete_node(id)
    }

    fn mem_compress(&self, prune_stale_days: Option<u32>) -> Result<(usize, i64), String> {
        self.compact(prune_stale_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType;

    fn test_kb() -> KnowledgeBase {
        let tmp = std::env::temp_dir().join(format!(
            "neotrix_primitives_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        KnowledgeBase::open(Some(tmp)).expect("open temp KB")
    }

    #[test]
    fn five_primitives_roundtrip() {
        let kb = test_kb();

        // write
        let id = kb
            .mem_write(
                "Primitive Write Test",
                NodeType::Concept,
                Some("A test concept about memory primitives and retrieval systems."),
                Some("https://primitives.example/test"),
                Some("test"),
                Some(&serde_json::json!({"source_id": "t0.2"})),
            )
            .expect("write");
        assert!(!id.is_empty());

        // retrieve
        let hits = kb.mem_retrieve("memory primitives", 5).expect("retrieve");
        assert!(
            hits.iter().any(|r| r.node.id == id),
            "written node must be retrievable"
        );

        // update
        kb.mem_update(&id, "Updated content about memory primitives and their lifecycle.")
            .expect("update");
        let node = kb.get_node(&id).expect("get").expect("node exists");
        assert!(node.content.as_deref().unwrap_or("").contains("Updated"));

        // delete
        let deleted = kb.mem_delete(&id).expect("delete");
        assert!(deleted, "node should be deleted");
        assert!(kb.get_node(&id).expect("get").is_none(), "node gone after delete");
    }

    #[test]
    fn compress_returns_report() {
        let kb = test_kb();
        let _ = kb
            .mem_write(
                "Compress Target",
                NodeType::Concept,
                Some("Content that will be compressed away."),
                None,
                Some("test"),
                None,
            )
            .expect("write");
        let (_pruned, _freed) = kb.mem_compress(Some(0)).expect("compress");
    }
}