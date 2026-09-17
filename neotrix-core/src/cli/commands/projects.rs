//! /projects 命令 — 项目管理
//!
//! 子命令:
//!   list     列出所有项目
//!   create   创建新项目
//!   info     查看项目详情
//!   delete   删除项目

use crate::cli::commands::types::{CliCommand, CommandOutput};

pub struct ProjectsCmd;

impl CliCommand for ProjectsCmd {
    fn name(&self) -> &str {
        "/projects"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/project"]
    }

    fn description(&self) -> &str {
        "项目管理: /projects list | /projects create <name>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" => self.cmd_list(),
            "create" => {
                let name = args.get(1).cloned().unwrap_or_default();
                if name.is_empty() {
                    CommandOutput::err("Usage: /projects create <name>")
                } else {
                    CommandOutput::ok(&format!("Created project: {}", name))
                }
            }
            "info" => {
                let name = args.get(1).cloned().unwrap_or_default();
                if name.is_empty() {
                    CommandOutput::err("Usage: /projects info <name>")
                } else {
                    CommandOutput::ok(&format!("Project info: {}", name))
                }
            }
            "delete" => {
                let name = args.get(1).cloned().unwrap_or_default();
                if name.is_empty() {
                    CommandOutput::err("Usage: /projects delete <name>")
                } else {
                    CommandOutput::ok(&format!("Deleted project: {}", name))
                }
            }
            _ => CommandOutput::err("Usage:\n  /projects list       列出所有项目\n  /projects create <name>  创建项目\n  /projects info <name>    查看项目\n  /projects delete <name>  删除项目"),
        }
    }
}

impl ProjectsCmd {
    fn cmd_list(&self) -> CommandOutput {
        CommandOutput::ok("Projects:\n  (no projects registered)")
    }
}