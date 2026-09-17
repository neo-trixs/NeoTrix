//! /exp 命令 — 实验管理
//!
//! 子命令:
//!   list     列出实验
//!   create   创建实验
//!   status   查看实验状态
//!   archive  归档实验

use crate::cli::commands::types::{CliCommand, CommandOutput};

pub struct ExpCmd;

impl CliCommand for ExpCmd {
    fn name(&self) -> &str {
        "/exp"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/experiment"]
    }

    fn description(&self) -> &str {
        "实验管理: /exp list | /exp create <name>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" => self.cmd_list(),
            "create" => {
                let name = args.get(1).cloned().unwrap_or_default();
                if name.is_empty() {
                    CommandOutput::err("Usage: /exp create <name>")
                } else {
                    CommandOutput::ok(&format!("Created experiment: {}", name))
                }
            }
            "status" => {
                let id = args.get(1).cloned().unwrap_or_default();
                if id.is_empty() {
                    CommandOutput::err("Usage: /exp status <id>")
                } else {
                    CommandOutput::ok(&format!("Experiment status: {}", id))
                }
            }
            "archive" => {
                let id = args.get(1).cloned().unwrap_or_default();
                if id.is_empty() {
                    CommandOutput::err("Usage: /exp archive <id>")
                } else {
                    CommandOutput::ok(&format!("Archived experiment: {}", id))
                }
            }
            _ => CommandOutput::err("Usage:\n  /exp list       列出实验\n  /exp create <name>  创建实验\n  /exp status <id>    查看状态\n  /exp archive <id>   归档"),
        }
    }
}

impl ExpCmd {
    fn cmd_list(&self) -> CommandOutput {
        CommandOutput::ok("Experiments:\n  (no experiments)")
    }
}