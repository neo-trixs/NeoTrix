//! /runs 命令 — 运行管理
//!
//! 子命令:
//!   list    列出所有运行
//!   start   启动新运行
//!   stop    停止运行
//!   logs    查看运行日志

use crate::cli::commands::types::{CliCommand, CommandOutput};

pub struct RunsCmd;

impl CliCommand for RunsCmd {
    fn name(&self) -> &str {
        "/runs"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/run"]
    }

    fn description(&self) -> &str {
        "运行管理: /runs list | /runs start <name>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" => self.cmd_list(),
            "start" => {
                let name = args.get(1).cloned().unwrap_or_default();
                if name.is_empty() {
                    CommandOutput::err("Usage: /runs start <name>")
                } else {
                    CommandOutput::ok(&format!("Started run: {}", name))
                }
            }
            "stop" => {
                let id = args.get(1).cloned().unwrap_or_default();
                if id.is_empty() {
                    CommandOutput::err("Usage: /runs stop <id>")
                } else {
                    CommandOutput::ok(&format!("Stopped run: {}", id))
                }
            }
            _ => CommandOutput::err("Usage:\n  /runs list      列出所有运行\n  /runs start <name>  启动运行\n  /runs stop <id>     停止运行"),
        }
    }
}

impl RunsCmd {
    fn cmd_list(&self) -> CommandOutput {
        CommandOutput::ok("Runs:\n  (no active runs)")
    }
}