//! /discover 命令 — 技能/模型发现
//!
//! 子命令:
//!   skills    发现可用技能
//!   models    发现可用模型
//!   plugins   发现可用插件
//!   search    搜索发现

use crate::cli::commands::types::{CliCommand, CommandOutput};

pub struct DiscoverCmd;

impl CliCommand for DiscoverCmd {
    fn name(&self) -> &str {
        "/discover"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/find"]
    }

    fn description(&self) -> &str {
        "发现: /discover skills | /discover models | /discover plugins"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let sub = args.first().map(|s| s.as_str()).unwrap_or("skills");
        match sub {
            "skills" => self.cmd_skills(),
            "models" => self.cmd_models(),
            "plugins" => self.cmd_plugins(),
            "search" => {
                let query = args.get(1).cloned().unwrap_or_default();
                if query.is_empty() {
                    CommandOutput::err("Usage: /discover search <query>")
                } else {
                    CommandOutput::ok(&format!("Search results for: {}", query))
                }
            }
            _ => CommandOutput::err("Usage:\n  /discover skills    发现技能\n  /discover models    发现模型\n  /discover plugins   发现插件\n  /discover search <q> 搜索"),
        }
    }
}

impl DiscoverCmd {
    fn cmd_skills(&self) -> CommandOutput {
        CommandOutput::ok("Skills:\n  (no skills discovered)")
    }

    fn cmd_models(&self) -> CommandOutput {
        CommandOutput::ok("Models:\n  (no models discovered)")
    }

    fn cmd_plugins(&self) -> CommandOutput {
        CommandOutput::ok("Plugins:\n  (no plugins discovered)")
    }
}