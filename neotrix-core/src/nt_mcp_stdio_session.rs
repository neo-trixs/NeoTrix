//! MCP stdio 会话客户端 —— core 侧门面（2026-09-29 起实现在 neobot 侧）。
//!
//! 实现在 `crates/neotrix-neobot/src/nt_qwen_mm.rs`：那里是**唯一真在执行
//! 工具的环**（`nt_agent::execute_tool`）。2026-09-29 实测：
//! `neotrix-core` 的 `ToolOrchestrator.call` 仅测试在调、`AgentLoop.with_tools`
//! 仅测试在用、crystal 活环零消费 `NativeTool`；而 neobot 的 turn loop
//! 真的逐个执行模型点名的工具。**模型自主执行链因此必须落在 neobot 侧** ——
//! 客户端也随执行环住。
//!
//! 本文件做两件事：
//! 1. re-export 客户端类型（零成本，`neotrix::nt_mcp_stdio_session::*` 旧路径
//!    与 `nt_qwen_mm_manifests.rs` 的 core 侧注册继续编译）；
//! 2. 保留 `McpSessionTool` —— `NativeTool` 是 **core 侧**的 trait
//!    （`l0_substrate::nt_core_traits`），所以这个适配器归 core，不进 neobot。
//!    它服务 core 的 MCP 注册表（`as_native_tools` 会话分支），
//!    即"人类操作员经 CLI 执行"这条路径；模型自主执行走 neobot 直调。

use std::path::PathBuf;

pub use neotrix_neobot::nt_qwen_mm::{
    DEFAULT_TIMEOUT_MS, MCP_CLIENT_NAME, MCP_PROTOCOL_VERSION, McpCallResult, McpListedTool,
    McpSessionError, McpStdioSession,
};

/// 会话式 MCP 工具的 `NativeTool` 适配（core 侧）。
///
/// `agent.rs` 的 `McpServer.use_session == true` 时由此执行；旧
/// `StdioNativeTool` 保持不动（短命令式工具仍走它）。
#[derive(Debug, Clone)]
pub struct McpSessionTool {
    /// 工具名（`tools/call` 的 `name`）。
    pub def_name: String,
    /// 工具描述（透传清单）。
    pub def_description: String,
    /// 工具输入 schema（透传清单）。
    pub def_schema: serde_json::Value,
    /// 服务器启动命令。
    pub command: String,
    /// 服务器启动参数。
    pub args: Vec<String>,
    /// 单次调用超时（毫秒）。
    pub timeout_ms: u64,
    /// `image` 块落盘目录（`None` = 默认临时目录）。
    pub artifacts_dir: Option<PathBuf>,
}

impl crate::l0_substrate::nt_core_traits::NativeTool for McpSessionTool {
    fn id(&self) -> &str {
        &self.def_name
    }

    fn description(&self) -> &str {
        &self.def_description
    }

    fn input_schema(&self) -> serde_json::Value {
        self.def_schema.clone()
    }

    fn capability_tags(&self) -> Vec<&'static str> {
        Vec::new()
    }

    fn execute(
        &self,
        args: &serde_json::Value,
    ) -> Result<crate::l0_substrate::nt_core_traits::ToolOutput, String> {
        let mut session =
            McpStdioSession::new(self.command.clone(), self.args.clone())
                .with_timeout_ms(self.timeout_ms);
        if let Some(dir) = &self.artifacts_dir {
            session = session.with_artifacts_dir(dir.clone());
        }
        match session.call_tool(&self.def_name, args) {
            Ok(r) => {
                let mut content = r.text;
                if !r.saved_images.is_empty() {
                    content.push_str("\n[saved images]");
                    for p in &r.saved_images {
                        content.push_str(&format!("\n- {}", p.display()));
                    }
                }
                Ok(crate::l0_substrate::nt_core_traits::ToolOutput {
                    success: !r.is_error,
                    content,
                })
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    const FAKE_SERVER: &str = r#"#!/usr/bin/env bash
while IFS= read -r line; do
  case "$line" in
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"fake","version":"0"}}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"echo-ok"}],"isError":false}}'
      ;;
  esac
done
"#;

    fn bash_cmd() -> String {
        if Path::new("/bin/bash").is_file() {
            "/bin/bash".to_string()
        } else {
            "bash".to_string()
        }
    }

    /// core 侧适配器仍能驱动会话（`/mcp call` 路径的回归保护）。
    #[test]
    fn test_native_adapter_drives_session() {
        use crate::l0_substrate::nt_core_traits::NativeTool;
        let dir = std::env::temp_dir().join(format!(
            "nt_core_adapter_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        let script = dir.join("fake.sh");
        std::fs::write(&script, FAKE_SERVER).expect("write fake server");
        let tool = McpSessionTool {
            def_name: "echo_text".to_string(),
            def_description: "echo".to_string(),
            def_schema: serde_json::json!({"type": "object"}),
            command: bash_cmd(),
            args: vec![script.to_string_lossy().to_string()],
            timeout_ms: 10_000,
            artifacts_dir: None,
        };
        assert_eq!(tool.id(), "echo_text");
        let out = tool.execute(&serde_json::json!({})).expect("execute");
        assert!(out.success);
        assert!(out.content.contains("echo-ok"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
