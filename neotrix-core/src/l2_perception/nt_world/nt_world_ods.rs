//! ODS — 开放文档系统解析器适配 (C1)
//!
//! 吸收 github.com/Osmantic/ODS (Open Document System): 将多种开放文档格式
//! 统一解析为规范化节点树。本模块以 trait 定义格式解析契约，提供 stub
//! 实现 (C1 接入点)，后续接入真实解析器 (C2-C4)。

use crate::core::nt_core_kb_types::{KnowledgeNode, NodeType};
use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
use uuid::Uuid;

/// 当前 Unix 时间戳 (秒)。
fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 支持的开放文档格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OdsFormat {
    Markdown,
    Org,
    Rst,
    AsciiDoc,
    Unknown,
}

/// 解析后的文档节点
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdsNode {
    pub path: String,
    pub kind: String,
    pub body: String,
}

/// 开放文档系统解析器适配契约 (ODS 抽象)
pub trait OdsParserAdapter {
    /// 探测输入内容的格式
    fn detect_format(&self, content: &str) -> OdsFormat;
    /// 将内容解析为节点树
    fn parse(&self, content: &str) -> Vec<OdsNode>;
    /// C2 接线: 将解析出的节点树写入 KB (Article 节点) 并触发 FTS5 索引。
    /// 返回成功写入的节点 id 列表。
    fn ingest_nodes(&self, kb: &KnowledgeBase, content: &str) -> Result<Vec<String>, String>;
}

/// 默认 stub 实现
#[derive(Default)]
pub struct OdsAdapter;

impl OdsParserAdapter for OdsAdapter {
    fn detect_format(&self, content: &str) -> OdsFormat {
        let head = content.lines().next().unwrap_or("").trim();
        if head.starts_with("# ") {
            OdsFormat::Markdown
        } else if head.starts_with("* ") {
            OdsFormat::Org
        } else if head.starts_with("== ") {
            OdsFormat::AsciiDoc
        } else if head.starts_with("===") {
            OdsFormat::Rst
        } else {
            OdsFormat::Unknown
        }
    }

    fn parse(&self, content: &str) -> Vec<OdsNode> {
        match self.detect_format(content) {
            OdsFormat::Unknown => Vec::new(),
            fmt => vec![OdsNode {
                path: "root".into(),
                kind: format!("{:?}", fmt),
                body: content.to_string(),
            }],
        }
    }

    fn ingest_nodes(&self, kb: &KnowledgeBase, content: &str) -> Result<Vec<String>, String> {
        let nodes = self.parse(content);
        let mut ids = Vec::with_capacity(nodes.len());
        for n in &nodes {
            let now = now_ts();
            let node = KnowledgeNode {
                id: Uuid::new_v4().to_string(),
                node_type: NodeType::Article,
                title: if n.path.is_empty() { "ods:root".into() } else { n.path.clone() },
                summary: Some(format!("ODS parsed node ({})", n.kind)),
                content: Some(n.body.clone()),
                url: None,
                domain: Some("ods".to_string()),
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
            ids.push(node.id);
        }
        Ok(ids)
    }
}

/// SelfTest (T1): 格式探测 + 解析存在性
pub struct OdsSelfTest;

impl SelfTest for OdsSelfTest {
    fn name(&self) -> &str {
        "nt_world_ods"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let a = OdsAdapter;
        if a.detect_format("# Title") != OdsFormat::Markdown {
            return Err(vec!["ods: '# ' should detect Markdown".into()]);
        }
        let nodes = a.parse("* heading");
        if nodes.len() != 1 {
            return Err(vec![format!("ods: expected 1 node, got {}", nodes.len())]);
        }
        Ok(())
    }
}

/// 注册 ODS SelfTest
pub fn register_ods_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(OdsSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_markdown_and_org() {
        let a = OdsAdapter;
        assert_eq!(a.detect_format("# h"), OdsFormat::Markdown);
        assert_eq!(a.detect_format("* h"), OdsFormat::Org);
    }

    #[test]
    fn test_parse_returns_node() {
        let a = OdsAdapter;
        let nodes = a.parse("== Title");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].kind, "AsciiDoc");
    }

    #[test]
    fn test_unknown_format_empty() {
        let a = OdsAdapter;
        assert_eq!(a.detect_format("plain text"), OdsFormat::Unknown);
        assert!(a.parse("plain text").is_empty());
        let t = OdsSelfTest;
        assert_eq!(t.name(), "nt_world_ods");
        assert!(t.self_test().is_ok());
    }

    #[test]
    fn ingest_nodes_writes_kb_nodes() {
        use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;
        use std::path::PathBuf;
        let Ok(kb) = KnowledgeBase::open(Some(PathBuf::from(":memory:"))) else {
            eprintln!("skip: KB (FTS5) unavailable in this build");
            return;
        };
        let a = OdsAdapter;
        let ids = a
            .ingest_nodes(&kb, "# Title\nbody")
            .expect("ingest should succeed");
        assert_eq!(ids.len(), 1);
        let node = kb.get_node(&ids[0]).expect("node present").expect("node exists");
        assert_eq!(node.node_type, crate::core::nt_core_kb_types::NodeType::Article);
        assert!(node.content.as_ref().unwrap().contains("Title"));
    }
}
