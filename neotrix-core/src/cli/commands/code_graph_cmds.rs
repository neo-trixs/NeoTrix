//! code-graph CLI — `neotrix_code_graph` MCP 工具的真实 backing (GAP-1 修复)。
//!
//! `nt_agent_mcp_tools::neotrix_code_graph` 声明 `McpTransport::Local { command: "neotrix",
//! args: ["code-graph"] }`, 经 main.rs 未知子命令回退到交互式注册表 → `/code-graph`。
//! 本命令把 `core::nt_core_retrieval::CodeGraphMCP` 的 4 个确定性检索工具暴露为 CLI,
//! 使 CodeGraphMCP 达 T3 生产接线 (R-P79: 公开方法被非测试代码调用并产生行为接地)。

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_retrieval::CodeGraphMCP;
use crate::neotrix::nt_mind::SelfIteratingBrain;

fn parse_str(args: &[String], flag: &str, default: &str) -> String {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

pub struct CodeGraphCmd;

impl CliCommand for CodeGraphCmd {
    fn name(&self) -> &str {
        "/code-graph"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/cg", "/graph"]
    }

    fn description(&self) -> &str {
        "Deterministic code-graph retrieval (G1): /code-graph search_symbols|file_stats|graph_topology|get_node --query <q> --root <dir>"
    }

    fn is_primary(&self) -> bool {
        false
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let Some(action) = args.first() else {
            return CommandOutput::ok(
                "Usage: /code-graph <search_symbols|file_stats|graph_topology|get_node> [--query <q>] [--root <dir>]",
            );
        };
        let root = parse_str(args, "--root", ".");
        let mut mcp = CodeGraphMCP::new();
        if let Err(e) = mcp.build(Path::new(&root)) {
            return CommandOutput::err(&format!("code-graph build failed: {}", e));
        }

        let result = match action.as_str() {
            "search_symbols" => {
                let query = parse_str(args, "--query", "");
                if query.is_empty() {
                    return CommandOutput::err("search_symbols requires --query <q>");
                }
                mcp.tool_search_symbols(&query)
            }
            "file_stats" => mcp.tool_file_stats(),
            "graph_topology" => mcp.tool_graph_topology(),
            "get_node" => {
                let id = parse_str(args, "--query", "");
                if id.is_empty() {
                    return CommandOutput::err("get_node requires --query <node_id>");
                }
                mcp.tool_get_node(&id)
            }
            other => {
                return CommandOutput::err(&format!(
                    "Unknown action '{}'. Use: search_symbols, file_stats, graph_topology, get_node",
                    other
                ));
            }
        };

        if result.ok {
            CommandOutput::ok(&result.summary).with_json(result.detail)
        } else {
            CommandOutput::err(&result.summary)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nt-cgcmd-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/math.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
        dir
    }

    #[test]
    fn search_symbols_returns_json() {
        let dir = fixture();
        let cmd = CodeGraphCmd;
        let out = cmd.execute(&["search_symbols".into(), "--query".into(), "add".into(), "--root".into(), dir.to_string_lossy().to_string()], None);
        assert!(out.success, "msg: {}", out.message);
        let json = out.json.expect("search_symbols should return json");
        assert!(json.as_array().map(|a| a.len() >= 1).unwrap_or(false));
    }

    #[test]
    fn graph_topology_ok() {
        let dir = fixture();
        let cmd = CodeGraphCmd;
        let out = cmd.execute(&["graph_topology".into(), "--root".into(), dir.to_string_lossy().to_string()], None);
        assert!(out.success);
        assert!(out.json.expect("topology json").get("nodes").is_some());
    }

    #[test]
    fn unknown_action_errors() {
        let dir = fixture();
        let cmd = CodeGraphCmd;
        let out = cmd.execute(&["bogus".into(), "--root".into(), dir.to_string_lossy().to_string()], None);
        assert!(!out.success);
        assert!(out.message.contains("Unknown action"));
    }

    #[test]
    fn search_symbols_requires_query() {
        let cmd = CodeGraphCmd;
        let out = cmd.execute(&["search_symbols".into()], None);
        assert!(!out.success);
    }
}