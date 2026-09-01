//! NT-MEMORY 科学论文翻译后端适配 (条目7, 源 PDFMathTranslate/PDFMathTranslate)
//!
//! 参照: github.com/PDFMathTranslate/PDFMathTranslate — 科学论文翻译, 保留
//! 数学公式/图表/排版格式。吸收为 NT-MEMORY 知识层的一个翻译后端适配 trait,
//! 将已翻译论文节点插入 KB FTS5 索引 (C1: T1 存在级 + 3 单元测)。
//!
//! 机制: `TranslatorBackend` trait 定义统一翻译后端接口; `PdfMathTranslateBackend`
//! 为 PDFMathTranslate CLI/HTTP 后端的 stub 实现, 预留 FTS5 节点插入钩子。

use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};
use crate::core::nt_core_self_test::SelfTest;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use uuid::Uuid;

/// 当前 Unix 时间戳 (秒)。
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 翻译后端统一接口 — 任何论文/文档翻译源均实现此 trait。
pub trait TranslatorBackend: Send + Sync {
    /// 后端标识 (如 "pdfmathtranslate")。
    fn backend_id(&self) -> &str;
    /// 计算已翻译论文的 KB 节点 key (无副作用, 纯标识生成)。
    /// 保留为无 KB 句柄的轻量入口, 真实写入见 `ingest_translated_paper`。
    fn insert_translated_node(&self, title: &str, body: &str) -> Result<String, String>;
    /// C2 接线: 将一篇已翻译论文作为 `Paper` 节点写入 KB, 并触发 FTS5 索引。
    /// 返回新插入节点的 id。
    fn ingest_translated_paper(
        &self,
        kb: &KnowledgeBase,
        title: &str,
        body: &str,
    ) -> Result<String, String>;
    /// 后端是否可用 (可达性探测 stub)。
    fn is_available(&self) -> bool;
}

/// PDFMathTranslate 后端实现 (stub)。
pub struct PdfMathTranslateBackend {
    pub endpoint: String,
}

impl Default for PdfMathTranslateBackend {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:7860".to_string(),
        }
    }
}

impl TranslatorBackend for PdfMathTranslateBackend {
    fn backend_id(&self) -> &str {
        "pdfmathtranslate"
    }

    fn insert_translated_node(&self, title: &str, body: &str) -> Result<String, String> {
        if title.trim().is_empty() {
            return Err("title must not be empty".to_string());
        }
        if body.trim().is_empty() {
            return Err("body must not be empty".to_string());
        }
        Ok(format!("kb:paper:{}", title.trim().to_lowercase().replace(' ', "_")))
    }

    fn ingest_translated_paper(
        &self,
        kb: &KnowledgeBase,
        title: &str,
        body: &str,
    ) -> Result<String, String> {
        if title.trim().is_empty() {
            return Err("title must not be empty".to_string());
        }
        if body.trim().is_empty() {
            return Err("body must not be empty".to_string());
        }
        let now = now_ts();
        let node = KnowledgeNode {
            id: Uuid::new_v4().to_string(),
            node_type: NodeType::Paper,
            title: title.trim().to_string(),
            summary: Some(format!("Translated paper via {}", self.backend_id())),
            content: Some(body.to_string()),
            url: None,
            domain: Some("pdfmathtranslate".to_string()),
            language: "en".to_string(),
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
        };
        kb.insert_node(&node)?;
        Ok(node.id)
    }

    fn is_available(&self) -> bool {
        !self.endpoint.is_empty()
    }
}

/// SelfTest (T1 存在级) — 验证后端 trait 契约与 FTS5 节点 key 生成。
impl SelfTest for PdfMathTranslateBackend {
    fn name(&self) -> &str {
        "nt_memory_pdf_math_translate"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.backend_id() != "pdfmathtranslate" {
            failures.push(format!("unexpected backend_id: {}", self.backend_id()));
        }
        if !self.is_available() {
            failures.push("backend reported unavailable".to_string());
        }
        if let Err(e) = self.insert_translated_node("Test Paper", "translated body") {
            failures.push(format!("node insert failed: {e}"));
        }
        if let Ok(key) = self.insert_translated_node("Deep Learning", "body") {
            if !key.starts_with("kb:paper:") {
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
    fn backend_id_is_pdfmathtranslate() {
        let b = PdfMathTranslateBackend::default();
        assert_eq!(b.backend_id(), "pdfmathtranslate");
    }

    #[test]
    fn insert_node_generates_fts5_key() {
        let b = PdfMathTranslateBackend::default();
        let key = b
            .insert_translated_node("Attention Is All You Need", "body")
            .expect("insert should succeed");
        assert!(key.starts_with("kb:paper:"));
        assert!(key.contains("attention_is_all_you_need"));
    }

    #[test]
    fn empty_title_or_body_is_rejected() {
        let b = PdfMathTranslateBackend::default();
        assert!(b.insert_translated_node("", "body").is_err());
        assert!(b.insert_translated_node("Title", "").is_err());
    }

    #[test]
    fn ingest_translated_paper_writes_kb_node() {
        use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
        use std::path::PathBuf;
        // FTS5 虚拟表在本构建可能不可用 → 优雅跳过, 仅验证编译/接线。
        let Ok(kb) = KnowledgeBase::open(Some(PathBuf::from(":memory:"))) else {
            eprintln!("skip: KB (FTS5) unavailable in this build");
            return;
        };
        let b = PdfMathTranslateBackend::default();
        let id = b
            .ingest_translated_paper(&kb, "Attention Is All You Need", "translated body")
            .expect("ingest should succeed");
        assert!(!id.is_empty());
        let node = kb.get_node(&id).expect("node present").expect("node exists");
        assert_eq!(node.node_type, crate::core::nt_core_kb_types::NodeType::Paper);
        assert_eq!(node.title, "Attention Is All You Need");
    }
}
