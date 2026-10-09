//! `nt_mcp_bridge` — 把 mcp.json 里配置的 MCP server 动态热插拔进插件清单。
//!
//! 语义形状（沿用 `nt_plugins` 的 fail-closed 原则）:
//! - `data_dir/mcp.json` 形如 `{ "servers": [{"name","command","args","description"}] }`;
//! - 每个 server 注册一条 `PluginManifest`, name=`mcp__<server>`, 同一条
//!   `nt_plugins::invoke` 通道（stdin 写 JSON、stdout 回字节串）调用;
//! - 真实 MCP stdio 握手/`tools/list` 由 `nt_qwen_mm::McpStdioSession` 处理,
//!   本模块只暴露薄封装 `list_server_tools`, 不复制 handshake 代码;
//! - 未登记到清单的 server 不可触达（默认 deny）。

use std::path::Path;

use crate::nt_error::NtBotError;
use crate::nt_plugins::PluginManifest;
use crate::nt_qwen_mm::{McpListedTool, McpSessionError, McpStdioSession};

/// mcp.json 中的单条 server 配置。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServerSpec {
    /// server 名（注册进清单时前缀为 `mcp__`）。
    pub name: String,
    /// 可执行命令（在 PATH 中搜索）。
    pub command: String,
    /// 命令固定前缀参数。
    #[serde(default)]
    pub args: Vec<String>,
    /// 给模型/人看的说明。
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct McpConfigFile {
    servers: Vec<McpServerSpec>,
}

/// 读取 `data_dir/mcp.json`；文件不存在返回空表。
pub fn load_mcp_servers(data_dir: &Path) -> Result<Vec<McpServerSpec>, NtBotError> {
    let path = data_dir.join("mcp.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| NtBotError::Invalid(format!("mcp.json unreadable: {e}")))?;
    let parsed: McpConfigFile = serde_json::from_str(&text)
        .map_err(|e| NtBotError::Invalid(format!("mcp.json parse failed: {e}")))?;
    Ok(parsed.servers)
}

/// 把一个 server spec 包装成单条 `PluginManifest`（name=`mcp__<server>`）。
pub fn manifest_for_server(spec: &McpServerSpec) -> PluginManifest {
    PluginManifest {
        name: format!("mcp__{}", spec.name),
        description: spec.description.clone(),
        command: spec.command.clone(),
        args: spec.args.clone(),
    }
}

/// 把所有 server 注册进全局插件表；同名重复的条目跳过（幂等），其余错误上抛。
pub fn register_mcp_servers(servers: &[McpServerSpec]) -> Result<usize, NtBotError> {
    let mut registered = 0usize;
    for spec in servers {
        let manifest = manifest_for_server(spec);
        match crate::nt_plugins::register_one(manifest) {
            Ok(()) => registered += 1,
            Err(NtBotError::Invalid(msg)) if msg.contains("already registered") => {}
            Err(e) => return Err(e),
        }
    }
    Ok(registered)
}

/// 启动钩子：load + register 一条龙。
pub fn init_mcp_bridge(data_dir: &Path) -> Result<usize, NtBotError> {
    let servers = load_mcp_servers(data_dir)?;
    register_mcp_servers(&servers)
}

/// 经现有 `McpStdioSession` 对某个 server 跑一次 `tools/list`
///（每调用一次 spawn, 不驻留常驻进程）。
pub fn list_server_tools(
    spec: &McpServerSpec,
    timeout_ms: u64,
) -> Result<Vec<McpListedTool>, McpSessionError> {
    let session = McpStdioSession::new(spec.command.clone(), spec.args.clone())
        .with_timeout_ms(timeout_ms);
    session.list_tools()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_missing_mcp_json_yields_empty() {
        let tmp = crate::nt_testutil::temp_dir("nt_mcp_bridge_empty");
        std::fs::create_dir_all(&tmp).unwrap();
        let servers = load_mcp_servers(&tmp).expect("missing mcp.json");
        assert!(servers.is_empty());
    }

    #[test]
    fn load_parses_servers_array() {
        let tmp = crate::nt_testutil::temp_dir("nt_mcp_bridge_parse");
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("mcp.json"),
            r#"{"servers":[{"name":"demo","command":"/bin/cat","args":[],"description":"mock"}]}"#,
        )
        .unwrap();
        let servers = load_mcp_servers(&tmp).expect("parse");
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "demo");
        assert_eq!(manifest_for_server(&servers[0]).name, "mcp__demo");
    }

    #[test]
    fn list_server_tools_with_mock_stdio_mcp_server() {
        let tmp = crate::nt_testutil::temp_dir("nt_mcp_bridge_mock_e2e");
        std::fs::create_dir_all(&tmp).unwrap();
        let script = tmp.join("mock_mcp.py");
        std::fs::write(
            &script,
            r#"import sys, json
for line in sys.stdin:
    try:
        req = json.loads(line)
    except Exception:
        continue
    method = req.get("method")
    if method == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":req.get("id"),"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"mock","version":"0"}}}), flush=True)
    elif method == "notifications/initialized":
        continue
    elif method == "tools/list":
        print(json.dumps({"jsonrpc":"2.0","id":req.get("id"),"result":{"tools":[{"name":"mock_echo","description":"echo","inputSchema":{"type":"object"}}]}}), flush=True)
    elif method == "tools/call":
        print(json.dumps({"jsonrpc":"2.0","id":req.get("id"),"result":{"content":[{"type":"text","text":"mock mcp echo"}],"isError":False}}), flush=True)
    else:
        print(json.dumps({"jsonrpc":"2.0","id":req.get("id"),"error":{"code":-32601,"message":"unknown"}}), flush=True)
"#,
        )
        .unwrap();
        let spec = McpServerSpec {
            name: "mock".into(),
            command: "python3".into(),
            args: vec![script.to_str().unwrap().to_owned()],
            description: None,
        };
        let tools = list_server_tools(&spec, 5000).expect("tools/list through mock");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "mock_echo");
    }

    #[test]
    fn register_mock_server_via_register_one_and_invoke_cat() {
        let m = PluginManifest {
            name: "mcp__mock_cat".to_owned(),
            description: Some("mock".to_owned()),
            command: "/bin/cat".to_owned(),
            args: vec![],
        };
        crate::nt_plugins::register_one(m).expect("register");
        assert!(crate::nt_plugins::is_registered("mcp__mock_cat"));
        let found = crate::nt_plugins::lookup("mcp__mock_cat").expect("lookup");
        let tmp = crate::nt_testutil::temp_dir("nt_mcp_bridge_invoke");
        std::fs::create_dir_all(&tmp).unwrap();
        let args = serde_json::json!({"hello": "world"});
        let out = crate::nt_plugins::invoke(&found, &args, &tmp).expect("invoke");
        assert_eq!(out, serde_json::to_string(&args).unwrap());
    }
}
