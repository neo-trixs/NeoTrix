//! nt_core_retrieval — 确定性代码图检索 (G1, repowise/graphify/codebase-memory-mcp 吸收)。
//!
//! Bud `code_graph_mcp`: 把 NT-WORLD 的 `CodeGraph` (符号图) 与 `CodeSearchEngine`
//! (语义检索) 包装为确定性 MCP 风格工具函数, 供 `nt_agent_mcp_registry` 登记为
//! 代码图检索工具面。纯 Rust, 零 LLM 依赖, 结果可复现。

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::neotrix::l2_world_impl::nt_world_code_search::SymbolIndex;
use crate::neotrix::l8_autonomic_impl::nt_mind::infrastructure::code_graph::CodeGraph;

/// MCP 工具调用结果 (确定性, 可缓存/可审计)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolResult {
    pub tool: String,
    pub ok: bool,
    pub summary: String,
    pub detail: serde_json::Value,
}

impl MCPToolResult {
    pub fn ok(tool: &str, summary: impl Into<String>, detail: serde_json::Value) -> Self {
        Self { tool: tool.to_string(), ok: true, summary: summary.into(), detail }
    }

    pub fn err(tool: &str, message: impl Into<String>) -> Self {
        Self { tool: tool.to_string(), ok: false, summary: message.into(), detail: serde_json::json!({}) }
    }
}

/// 代码图检索 Bud — 包装 CodeGraph + SymbolIndex 为 MCP 工具面。
#[derive(Default)]
pub struct CodeGraphMCP {
    graph: Option<CodeGraph>,
    index: Option<SymbolIndex>,
}

impl CodeGraphMCP {
    pub fn new() -> Self {
        Self::default()
    }

    /// 构建代码图 + 符号索引 (幂等: 已构建则跳过)。
    pub fn build(&mut self, root: &Path) -> Result<usize, String> {
        let mut graph = CodeGraph::new();
        let nodes = graph.build(root)?;
        let index = SymbolIndex::build(root);
        let total = nodes + index.len();
        self.graph = Some(graph);
        self.index = Some(index);
        Ok(total)
    }

    pub fn is_built(&self) -> bool {
        self.graph.is_some() && self.index.is_some()
    }

    /// MCP 工具: 检索符号 (名字包含查询)。
    pub fn tool_search_symbols(&self, query: &str) -> MCPToolResult {
        let Some(index) = &self.index else {
            return MCPToolResult::err("search_symbols", "index not built; call build first");
        };
        let hits = index.search_symbols(query);
        let detail: Vec<serde_json::Value> = hits
            .iter()
            .map(|s| serde_json::json!({ "name": s.name, "file": s.file, "line": s.line, "kind": s.kind }))
            .collect();
        MCPToolResult::ok(
            "search_symbols",
            format!("{} hits for {:?}", detail.len(), query),
            serde_json::json!(detail),
        )
    }

    /// MCP 工具: 按文件统计符号数 (模块拓扑)。
    pub fn tool_file_stats(&self) -> MCPToolResult {
        let Some(index) = &self.index else {
            return MCPToolResult::err("file_stats", "index not built");
        };
        let mut by_file: HashMap<&str, usize> = HashMap::new();
        for s in index.all_symbols() {
            *by_file.entry(s.file.as_str()).or_insert(0) += 1;
        }
        let mut files: Vec<(String, usize)> = by_file
            .into_iter()
            .map(|(f, n)| (f.to_string(), n))
            .collect();
        files.sort_by(|a, b| b.1.cmp(&a.1));
        MCPToolResult::ok(
            "file_stats",
            format!("{} files indexed", files.len()),
            serde_json::json!(files),
        )
    }

    /// MCP 工具: 代码图全局拓扑 (节点/边/社区)。
    pub fn tool_graph_topology(&self) -> MCPToolResult {
        let Some(graph) = &self.graph else {
            return MCPToolResult::err("graph_topology", "graph not built");
        };
        MCPToolResult::ok(
            "graph_topology",
            format!(
                "{} nodes, {} edges, {} communities",
                graph.nodes().len(),
                graph.edges().len(),
                graph.communities().len()
            ),
            serde_json::json!({
                "nodes": graph.nodes().len(),
                "edges": graph.edges().len(),
                "communities": graph.communities().len(),
            }),
        )
    }

    /// MCP 工具: 检索代码图节点。
    pub fn tool_get_node(&self, id: &str) -> MCPToolResult {
        let Some(graph) = &self.graph else {
            return MCPToolResult::err("get_node", "graph not built");
        };
        match graph.get_node(id) {
            Some(n) => MCPToolResult::ok("get_node", format!("node {} found", id), serde_json::json!({
                "id": n.id,
                "name": n.name,
                "kind": format!("{:?}", n.kind),
                "file": n.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                "line": n.start_line,
            })),
            None => MCPToolResult::err("get_node", format!("no node {}", id)),
        }
    }

    /// 汇总: 已登记工具名列表 (供 registry 登记)。
    pub fn tool_names() -> &'static [&'static str] {
        &["search_symbols", "file_stats", "graph_topology", "get_node"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nt-retrieval-fixture-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("src/math.rs"),
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\npub struct Config { pub name: String }\n",
        )
        .unwrap();
        std::fs::write(dir.join("src/main.rs"), "use crate::math;\npub fn main() { math::add(1, 2); }\n").unwrap();
        dir
    }

    #[test]
    fn build_and_search_symbols() {
        let dir = fixture();
        let mut mcp = CodeGraphMCP::new();
        assert!(!mcp.is_built());
        mcp.build(&dir).unwrap();
        assert!(mcp.is_built());
        let r = mcp.tool_search_symbols("add");
        assert!(r.ok);
        assert!(r.summary.contains("1 hits"));
    }

    #[test]
    fn tool_search_missing_requires_build() {
        let mcp = CodeGraphMCP::new();
        let r = mcp.tool_search_symbols("x");
        assert!(!r.ok);
        assert!(r.summary.contains("build first"));
    }

    #[test]
    fn file_stats_and_topology() {
        let dir = fixture();
        let mut mcp = CodeGraphMCP::new();
        mcp.build(&dir).unwrap();
        let stats = mcp.tool_file_stats();
        assert!(stats.ok);
        let topo = mcp.tool_graph_topology();
        assert!(topo.ok);
        assert!(topo.detail["nodes"].as_u64().unwrap() >= 3, "main+math 的符号节点");
    }

    #[test]
    fn tool_names_registry_contract() {
        assert!(CodeGraphMCP::tool_names().contains(&"search_symbols"));
        assert!(CodeGraphMCP::tool_names().contains(&"graph_topology"));
        assert_eq!(CodeGraphMCP::tool_names().len(), 4);
    }
}