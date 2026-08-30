//! Memory 聚合命令 — agent 调度通道（非人类一级面）

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;

/// /memory 聚合器 — MCP 桥接与 agent 后端调度的记忆域入口。
/// 子命令委派: evidence | wiki | kb | search
pub struct MemoryAggCmd;

impl CliCommand for MemoryAggCmd {
    fn name(&self) -> &str {
        "/memory"
    }

    fn is_primary(&self) -> bool {
        false // agent 工具，不占人类一级面
    }

    fn aliases(&self) -> Vec<&str> {
        vec![]
    }

    fn description(&self) -> &str {
        "Memory aggregator: evidence | wiki | kb | search (agent scheduling)"
    }

    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        match args.first().map(|s| s.as_str()) {
            Some("evidence") => delegate("/evidence", &args[1..], brain),
            Some("wiki") => delegate("/wiki", &args[1..], brain),
            Some("kb") => delegate("/kb", &args[1..], brain),
            Some("search") => delegate("/search", &args[1..], brain),
            _ => CommandOutput::ok("Memory 聚合子命令: evidence | wiki | kb | search"),
        }
    }
}

fn delegate(cmd: &str, rest: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
    let reg = crate::cli::commands::registry::default_registry();
    match reg.find(cmd) {
        Some(_) => {
            let full = format!("{} {}", cmd, rest.join(" "));
            let out = reg.execute(full.trim(), None);
            if out.success {
                out
            } else {
                CommandOutput::err(&out.message)
            }
        }
        None => CommandOutput::err(&format!("未知委派目标: {cmd}")),
    }
}
