//! NT-MEMORY BabelDOC 后端适配 (条目12, 源 funstory-ai/BabelDOC)
//!
//! 参照: github.com/funstory-ai/BabelDOC — 开源双语对照文档翻译后端,
//! 输出对照式翻译 (原文+译文并排)。吸收为 NT-MEMORY 知识层的文档翻译后端适配
//! trait, 将 BabelDOC 双语节点插入 KB (C1: T1 存在级 + 3 单元测)。
//!
//! 机制: `BabelDocBackend` 实现 `DocumentTranslator` trait, 预留双语对照节点
//! 插入钩子。

use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};
use crate::core::nt_core_self_test::SelfTest;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use uuid::Uuid;

use super::shared_utils::now_ts;

/// 文档翻译后端统一接口。
pub(crate) trait DocumentTranslator: Send + Sync {
    /// 后端标识 (如 "babeldoc")。
    fn backend_id(&self) -> &str;
    /// 生成双语对照知识节点 key (原文 + 译文)。
    fn build_bilingual_node(&self, doc_id: &str, source: &str, target: &str) -> Result<String, String>;
    /// C2 接线: 将双语对照文档作为 `Article` 节点写入 KB, 并触发 FTS5 索引。
    /// 原文与译文并排存入 content, 返回新节点 id。
    fn ingest_bilingual_node(
        &self,
        kb: &KnowledgeBase,
        doc_id: &str,
        source: &str,
        target: &str,
    ) -> Result<String, String>;
    /// 后端是否可用 (可达性探测 stub)。
    fn is_available(&self) -> bool;
}

/// BabelDOC 后端实现 (stub)。
pub(crate) struct BabelDocBackend {
    pub endpoint: String,
}

impl Default for BabelDocBackend {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8787".to_string(),
        }
    }
}

impl DocumentTranslator for BabelDocBackend {
    fn backend_id(&self) -> &str {
        "babeldoc"
    }

    fn build_bilingual_node(&self, doc_id: &str, source: &str, target: &str) -> Result<String, String> {
        if doc_id.trim().is_empty() {
            return Err("doc_id must not be empty".to_string());
        }
        if source.trim().is_empty() || target.trim().is_empty() {
            return Err("source/target text must not be empty".to_string());
        }
        Ok(format!(
            "kb:bilingual:{}:{}",
            doc_id.trim().to_lowercase().replace(' ', "_"),
            source.len() + target.len()
        ))
    }

    fn ingest_bilingual_node(
        &self,
        kb: &KnowledgeBase,
        doc_id: &str,
        source: &str,
        target: &str,
    ) -> Result<String, String> {
        if doc_id.trim().is_empty() {
            return Err("doc_id must not be empty".to_string());
        }
        if source.trim().is_empty() || target.trim().is_empty() {
            return Err("source/target text must not be empty".to_string());
        }
        let now = now_ts();
        let content = format!("SOURCE:\n{source}\n\nTARGET:\n{target}");
        let node = KnowledgeNode {
            id: Uuid::new_v4().to_string(),
            node_type: NodeType::Article,
            title: doc_id.trim().to_string(),
            summary: Some(format!("Bilingual doc via {}", self.backend_id())),
            content: Some(content),
            url: None,
            domain: Some("babeldoc".to_string()),
            language: "zh".to_string(),
            confidence: 0.9,
            importance: 0.5,
            recall_weight: 1.0,
            created_at: now,
            updated_at: now,
            access_count: 0,
            metadata: None,
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

/// SelfTest (T1 存在级) — 验证后端 trait 契约与双语节点 key 生成。
impl SelfTest for BabelDocBackend {
    fn name(&self) -> &str {
        "nt_memory_babeldoc"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.backend_id() != "babeldoc" {
            failures.push(format!("unexpected backend_id: {}", self.backend_id()));
        }
        if !self.is_available() {
            failures.push("backend reported unavailable".to_string());
        }
        if let Err(e) = self.build_bilingual_node("doc-1", "原文", "translation") {
            failures.push(format!("bilingual node build failed: {e}"));
        }
        if let Ok(key) = self.build_bilingual_node("Report", "a", "b") {
            if !key.starts_with("kb:bilingual:") {
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
    fn backend_id_is_babeldoc() {
        let b = BabelDocBackend::default();
        assert_eq!(b.backend_id(), "babeldoc");
    }

    #[test]
    fn bilingual_node_key_is_well_formed() {
        let b = BabelDocBackend::default();
        let key = b
            .build_bilingual_node("Quarterly Report", "原文", "译文")
            .expect("build should succeed");
        assert!(key.starts_with("kb:bilingual:quarterly_report:"));
    }

    #[test]
    fn empty_fields_are_rejected() {
        let b = BabelDocBackend::default();
        assert!(b.build_bilingual_node("", "src", "tgt").is_err());
        assert!(b.build_bilingual_node("id", "", "tgt").is_err());
        assert!(b.build_bilingual_node("id", "src", "").is_err());
    }

    #[test]
    fn ingest_bilingual_node_writes_kb_node() {
        use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
        use std::path::PathBuf;
        let Ok(kb) = KnowledgeBase::open(Some(PathBuf::from(":memory:"))) else {
            eprintln!("skip: KB (FTS5) unavailable in this build");
            return;
        };
        let b = BabelDocBackend::default();
        let id = b
            .ingest_bilingual_node(&kb, "Quarterly Report", "原文", "译文")
            .expect("ingest should succeed");
        assert!(!id.is_empty());
        let node = kb.get_node(&id).expect("node present").expect("node exists");
        assert_eq!(node.node_type, crate::core::nt_core_kb_types::NodeType::Article);
        assert!(node.content.as_ref().unwrap().contains("SOURCE:"));
        assert!(node.content.as_ref().unwrap().contains("TARGET:"));
    }
}
