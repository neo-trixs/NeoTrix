//! OpenKnowledge Markdown IDE
//!
//! 吸收 OpenKnowledge (3.8K★) markdown IDE + LLM wiki:
//! - WYSIWYG markdown 编辑
//! - LLM wiki 系统 (知识库/规格/笔记)
//! - MCP + skills + agentic search
//! - Git/GitHub 同步
//! - 图谱 wiki link viewer
//! - 可嵌入 HTML + 丰富组件

#![allow(dead_code)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Markdown IDE — OpenKnowledge 核心
pub struct MarkdownIDE {
    documents: HashMap<String, MarkdownDocument>,
    wiki_links: Vec<WikiLink>,
    sync_config: Option<SyncConfig>,
}

/// Markdown 文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    pub id: String,
    pub path: String,
    pub content: String,
    pub frontmatter: HashMap<String, serde_json::Value>,
    pub links: Vec<WikiLink>,
    pub backlinks: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
}

/// Wiki 链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiLink {
    pub source: String,
    pub target: String,
    pub text: Option<String>,
    pub link_type: WikiLinkType,
}

/// 链接类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WikiLinkType {
    Internal,    // [[target]]
    Embed,       // ![[target]]
    Reference,   // [[target|text]]
    External,    // [text](url)
}

/// 同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub provider: String, // "git", "github"
    pub remote: Option<String>,
    pub branch: String,
    pub auto_sync: bool,
    pub sync_interval_secs: u64,
}

/// LLM Wiki — 知识库系统
pub struct LLMWiki {
    documents: Vec<MarkdownDocument>,
    search_index: SearchIndex,
    knowledge_graph: KnowledgeGraph,
}

/// 搜索索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub entries: Vec<SearchEntry>,
    pub embeddings: bool,
}

/// 搜索条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    pub doc_id: String,
    pub title: String,
    pub content: String,
    pub score: f64,
}

/// 知识图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// 图边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: String,
    pub weight: f64,
}

/// MCP 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub tool: String,
    pub output: serde_json::Value,
    pub success: bool,
}

impl MarkdownIDE {
    /// 创建新的 IDE
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            wiki_links: vec![],
            sync_config: None,
        }
    }

    /// 打开文档
    pub(crate) fn _open_document(&mut self, path: &str, content: &str) -> Result<MarkdownDocument, String> {
        let doc = MarkdownDocument {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string(),
            content: content.to_string(),
            frontmatter: HashMap::new(),
            links: self.extract_wiki_links(content),
            backlinks: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: vec![],
        };

        // 更新反向链接
        for link in &doc.links {
            if let Some(target_doc) = self.documents.values_mut().find(|d| d.path == link.target) {
                target_doc.backlinks.push(doc.path.clone());
            }
        }

        self.wiki_links.extend(doc.links.clone());
        self.documents.insert(doc.id.clone(), doc.clone());

        Ok(doc)
    }

    /// 提取 wiki 链接
    fn extract_wiki_links(&self, content: &str) -> Vec<WikiLink> {
        let mut links = Vec::new();
        let mut chars = content.char_indices().peekable();

        while let Some((i, c)) = chars.next() {
            if c == '[' && chars.peek().map(|(_, c)| *c) == Some('[') {
                chars.next(); // skip second [
                let start = i + 2;
                let mut end = start;
                let mut text = None;

                while let Some((j, c)) = chars.next() {
                    if c == ']' && chars.peek().map(|(_, c)| *c) == Some(']') {
                        end = j;
                        chars.next(); // skip second ]
                        break;
                    }
                    if c == '|' {
                        text = Some(content[start..j].to_string());
                        end = j;
                    }
                }

                let target = if text.is_some() {
                    content[start..end].to_string()
                } else {
                    content[start..end].to_string()
                };

                links.push(WikiLink {
                    source: String::new(), // 由调用者设置
                    target,
                    text,
                    link_type: WikiLinkType::Internal,
                });
            }
        }

        links
    }

    /// 搜索文档
    pub fn search(&self, query: &str) -> Vec<SearchEntry> {
        let query_lower = query.to_lowercase();
        self.documents.values()
            .filter(|doc| {
                doc.content.to_lowercase().contains(&query_lower)
                    || doc.path.to_lowercase().contains(&query_lower)
            })
            .map(|doc| SearchEntry {
                doc_id: doc.id.clone(),
                title: doc.path.clone(),
                content: doc.content.chars().take(200).collect(),
                score: 0.5,
            })
            .collect()
    }

    /// 获取文档图谱
    pub fn get_graph(&self) -> KnowledgeGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for doc in self.documents.values() {
            nodes.push(GraphNode {
                id: doc.id.clone(),
                label: doc.path.clone(),
                node_type: "document".into(),
                properties: HashMap::new(),
            });

            for link in &doc.links {
                edges.push(GraphEdge {
                    source: doc.id.clone(),
                    target: link.target.clone(),
                    edge_type: "wiki_link".into(),
                    weight: 1.0,
                });
            }
        }

        KnowledgeGraph { nodes, edges }
    }

    /// 同步到 Git
    pub(crate) fn _sync_to_git(&self) -> Result<String, String> {
        match &self.sync_config {
            Some(config) => {
                // 实际实现会调用 git 命令
                Ok(format!("Synced to {} branch {}", config.provider, config.branch))
            }
            None => Err("No sync config set".into()),
        }
    }

    /// MCP 工具: search
    pub fn mcp_search(&self, query: &str) -> McpToolResult {
        let results = self.search(query);
        McpToolResult {
            tool: "search".into(),
            output: serde_json::json!({
                "results": results,
                "count": results.len(),
            }),
            success: true,
        }
    }

    /// MCP 工具: open
    pub(crate) fn _mcp_open(&self, path: &str) -> McpToolResult {
        match self.documents.values().find(|d| d.path == path) {
            Some(doc) => McpToolResult {
                tool: "open".into(),
                output: serde_json::json!({
                    "id": doc.id,
                    "path": doc.path,
                    "content": doc.content,
                    "links": doc.links.len(),
                    "backlinks": doc.backlinks.len(),
                }),
                success: true,
            },
            None => McpToolResult {
                tool: "open".into(),
                output: serde_json::json!({"error": "Document not found"}),
                success: false,
            },
        }
    }

    /// MCP 工具: graph
    pub(crate) fn _mcp_graph(&self) -> McpToolResult {
        let graph = self.get_graph();
        McpToolResult {
            tool: "graph".into(),
            output: serde_json::json!({
                "nodes": graph.nodes.len(),
                "edges": graph.edges.len(),
                "graph": graph,
            }),
            success: true,
        }
    }
}
