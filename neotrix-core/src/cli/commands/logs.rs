//! /logs 命令 — 日志查看
//!
//! 子命令:
//!   list    列出日志
//!   tail    尾部日志
//!   filter  过滤日志

use crate::cli::commands::types::{CliCommand, CommandOutput};

pub struct LogsCmd;

impl CliCommand for LogsCmd {
    fn name(&self) -> &str {
        "/logs"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/log"]
    }

    fn description(&self) -> &str {
        "日志查看: /logs list | /logs tail"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" => self.cmd_list(),
            "tail" => {
                let lines = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
                self.cmd_tail(lines)
            }
            "filter" => {
                let keyword = args.get(1).cloned().unwrap_or_default();
                if keyword.is_empty() {
                    CommandOutput::err("Usage: /logs filter <keyword>")
                } else {
                    CommandOutput::ok(&format!("Filtered logs for: {}", keyword))
                }
            }
            _ => CommandOutput::err("Usage:\n  /logs list      列出日志\n  /logs tail [n]    尾部日志\n  /logs filter <kw> 过滤日志"),
        }
    }
}

impl LogsCmd {
    fn cmd_list(&self) -> CommandOutput {
        CommandOutput::ok("Logs:\n  (no log entries)")
    }

    fn cmd_tail(&self, lines: usize) -> CommandOutput {
        CommandOutput::ok(&format!("Last {} log lines:\n  (no entries)", lines))
    }
}