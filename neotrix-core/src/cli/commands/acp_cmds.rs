//! /acp — ACP Server (Agent Client Protocol) for editor integration.
//!
//! Runs the JSON-RPC 2.0 stdio server backed by a NeoCodexAgent, so editors
//! (VS Code / Neovim / Emacs) can drive the agent over the Agent Client
//! Protocol. Methods: ping, agent/process, agent/status, agent/mode,
//! tools/list, shutdown.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::l1_body_impl::nt_io_neocodex::{AcpServer, NeoCodexAgent};

pub struct AcpCmd;

impl CliCommand for AcpCmd {
    fn name(&self) -> &str { "/acp" }
    fn aliases(&self) -> Vec<&str> { vec!["acp-server"] }
    fn description(&self) -> &str {
        "ACP Server: /acp run — start JSON-RPC stdio server for editor integration"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("run");
        match sub {
            "run" | "start" => {
                // Build a fresh agent and run the stdio loop. This blocks until
                // stdin closes (client sends "shutdown" or EOF).
                let agent = NeoCodexAgent::new("acp-stdio");
                let server = AcpServer::new(std::sync::Arc::new(tokio::sync::Mutex::new(agent)));
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => return CommandOutput::err(&format!("failed to start tokio runtime: {}", e)),
                };
                let _guard = rt.enter();
                let _ = rt.block_on(server.run_stdio());
                CommandOutput::ok("ACP server exited")
            }
            "help" | "--help" | "-h" => {
                CommandOutput::ok(
                    "用法: /acp run\n\n\
                     在 stdio 上运行 ACP (Agent Client Protocol) JSON-RPC 2.0 服务器。\n\
                     支持方法: ping, agent/process, agent/status, agent/mode, tools/list, shutdown\n\
                     编辑器通过子进程 stdin/stdout 逐行 JSON 交互。",
                )
            }
            _ => CommandOutput::err(&format!("未知子命令: {} (用法: /acp run)", sub)),
        }
    }
}