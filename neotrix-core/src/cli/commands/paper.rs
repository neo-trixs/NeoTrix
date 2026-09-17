//! /paper 命令 — 论文研究
//!
//! 子命令:
//!   search    搜索论文
//!   read      阅读论文
//!   list      列出论文库
//!   annotate  添加注释

use crate::cli::commands::types::{CliCommand, CommandOutput};

pub struct PaperCmd;

impl CliCommand for PaperCmd {
    fn name(&self) -> &str {
        "/paper"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/papers"]
    }

    fn description(&self) -> &str {
        "论文研究: /paper search <query> | /paper read <id>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("search");
        match sub {
            "search" => {
                let query = args.get(1).cloned().unwrap_or_default();
                if query.is_empty() {
                    CommandOutput::err("Usage: /paper search <query>")
                } else {
                    CommandOutput::ok(&format!("Paper search results for: {}", query))
                }
            }
            "read" => {
                let id = args.get(1).cloned().unwrap_or_default();
                if id.is_empty() {
                    CommandOutput::err("Usage: /paper read <id>")
                } else {
                    CommandOutput::ok(&format!("Reading paper: {}", id))
                }
            }
            "list" => self.cmd_list(),
            "annotate" => {
                let id = args.get(1).cloned().unwrap_or_default();
                if id.is_empty() {
                    CommandOutput::err("Usage: /paper annotate <id>")
                } else {
                    CommandOutput::ok(&format!("Annotated paper: {}", id))
                }
            }
            _ => CommandOutput::err("Usage:\n  /paper search <query>  搜索论文\n  /paper read <id>       阅读论文\n  /paper list            列出论文\n  /paper annotate <id>   添加注释"),
        }
    }
}

impl PaperCmd {
    fn cmd_list(&self) -> CommandOutput {
        CommandOutput::ok("Papers:\n  (no papers)")
    }
}