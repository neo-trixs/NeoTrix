//! NT-MEMORY Yopedia 后端适配 (条目10, 源 github.com/yologdev/yopedia)
//!
//! 参照: github.com/yologdev/yopedia — 智能体 wiki / 知识库 (KB) 参考实现,
//! 以条目化 (entry-based) 方式组织智能体可检索的参考知识。吸收为 NT-MEMORY
//! 知识层的 wiki/KB 参考后端 trait, 将 yopedia 条目映射为 KB 的 `WikiPage`
//! 节点 (C1: T1 存在级 + 3 单元测)。
//!
//! 机制: `YopediaBackend` 实现 `AgentWikiRef` trait, 预留条目化知识节点
//! 插入钩子, 与 NeoTrix 现有 `KnowledgeBase` (l3_memory_impl/nt_memory_kb) 对应。

use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};
use crate::core::nt_core_self_test::SelfTest;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use uuid::Uuid;

use super::shared_utils::now_ts;

/// 智能体 wiki / KB 参考后端统一接口。
pub(crate) trait AgentWikiRef: Send + Sync {
    /// 后端标识 (如 "yopedia")。
    fn backend_id(&self) -> &str;
    /// 由条目 (title + body) 生成 KB `WikiPage` 节点 key。
    fn build_entry_node_key(&self, entry_id: &str) -> Result<String, String>;
    /// C2 接线: 将条目化知识作为 `WikiPage` 节点写入 KB, 返回新节点 id。
    fn ingest_entry_node(
        &self,
        kb: &KnowledgeBase,
        entry_id: &str,
        title: &str,
        body: &str,
    ) -> Result<String, String>;
    /// 后端是否可用 (可达性探测 stub)。
    fn is_available(&self) -> bool;
}

/// Yopedia 后端实现 (stub)。
pub(crate) struct YopediaBackend {
    pub endpoint: String,
}

impl Default for YopediaBackend {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8788".to_string(),
        }
    }
}

impl AgentWikiRef for YopediaBackend {
    fn backend_id(&self) -> &str {
        "yopedia"
    }

    fn build_entry_node_key(&self, entry_id: &str) -> Result<String, String> {
        if entry_id.trim().is_empty() {
            return Err("entry_id must not be empty".to_string());
        }
        Ok(format!(
            "kb:wiki:{}",
            entry_id.trim().to_lowercase().replace(' ', "_")
        ))
    }

    fn ingest_entry_node(
        &self,
        kb: &KnowledgeBase,
        entry_id: &str,
        title: &str,
        body: &str,
    ) -> Result<String, String> {
        if entry_id.trim().is_empty() {
            return Err("entry_id must not be empty".to_string());
        }
        if title.trim().is_empty() {
            return Err("title must not be empty".to_string());
        }
        if body.trim().is_empty() {
            return Err("body must not be empty".to_string());
        }
        let now = now_ts();
        let node = KnowledgeNode {
            id: Uuid::new_v4().to_string(),
            node_type: NodeType::WikiPage,
            title: title.trim().to_string(),
            summary: Some(format!("Yopedia entry {}", entry_id.trim())),
            content: Some(body.to_string()),
            url: None,
            domain: Some("yopedia".to_string()),
            language: "en".to_string(),
            confidence: 0.9,
            importance: 0.5,
            recall_weight: 1.0,
            created_at: now,
            updated_at: now,
            access_count: 0,
            metadata: Some(serde_json::json!({ "entry_id": entry_id.trim() })),
            temporal: None,
            supersedes: None,
            source_episode: None,
            parent_id: None,
            depth: 0,
            cluster_id: None,
        };
        kb.insert_node(&node)?;
        Ok(node.id)
    }

    fn is_available(&self) -> bool {
        !self.endpoint.is_empty()
    }
}

/// SelfTest (T1 存在级) — 验证后端 trait 契约与条目节点 key 生成。
impl SelfTest for YopediaBackend {
    fn name(&self) -> &str {
        "nt_memory_yopedia"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.backend_id() != "yopedia" {
            failures.push(format!("unexpected backend_id: {}", self.backend_id()));
        }
        if !self.is_available() {
            failures.push("backend reported unavailable".to_string());
        }
        if let Err(e) = self.build_entry_node_key("entry-1") {
            failures.push(format!("entry node key build failed: {e}"));
        }
        if let Ok(key) = self.build_entry_node_key("Install Guide") {
            if !key.starts_with("kb:wiki:") {
                failures.push(format!("bad node key prefix: {key}"));
            }
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
    fn backend_id_is_yopedia() {
        let b = YopediaBackend::default();
        assert_eq!(b.backend_id(), "yopedia");
    }

    #[test]
    fn entry_node_key_is_well_formed() {
        let b = YopediaBackend::default();
        let key = b.build_entry_node_key("Setup Tips").unwrap();
        assert!(key.starts_with("kb:wiki:"));
        assert!(key.contains("setup_tips"));
    }

    #[test]
    fn empty_entry_id_is_rejected() {
        let b = YopediaBackend::default();
        assert!(b.build_entry_node_key("  ").is_err());
    }
}
