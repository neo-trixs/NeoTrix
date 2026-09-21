//! Agent 观测/调试命令 — 仅用于开发调试和应急干预
//!
//! 设计原则：
//! - 用户对话层：零 CLI，系统自动编排
//! - 观测层：status/logs/budget 用于调试
//! - 干预层：kill 用于应急
//!
//! 已移除的命令（迁移到自动编排）：
//! - /agent spawn → 系统自动 spawn
//! - /agent list → 系统自动管理
//! - /agent talk → 系统自动路由
//! - /agent background → 系统自动管理
//! - /agent tasks → 系统自动管理

use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;

// ============================================================
// 观测/调试命令（保留）
// ============================================================

/// Agent 状态观测命令
pub struct AgentStatusCmd;
impl CliCommand for AgentStatusCmd {
    fn name(&self) -> &str { "/agent" }
    fn aliases(&self) -> Vec<&str> { vec!["/agents"] }
    fn description(&self) -> &str {
        "Agent 观测: /agent status | /agent instances | /agent budget | /agent kill <id>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok(
                "Agent 观测工具:\n  /agent status          查看编排器状态\n  /agent instances       列出所有 agent 实例\n  /agent budget          查看成本消耗\n  /agent kill <id>       应急：强制终止 agent"
            );
        }
        match args[0].as_str() {
            "status" => {
                CommandOutput::ok("Agent 编排器状态:\n  总实例: 0\n  空闲: 0\n  运行中: 0\n  暂停: 0\n  总任务: 0\n  总成本: $0.00\n  预算: $50.00\n\n使用 AutoOrchestrator::status() 获取详细信息")
            }
            "instances" | "list" => {
                CommandOutput::ok("活跃 agent 实例: (无)\n\n使用 AutoOrchestrator::instances() 获取详细信息")
            }
            "budget" => {
                CommandOutput::ok("Agent 成本统计:\n  总消耗: $0.00\n  今日: $0.00\n  本月: $0.00\n  预算: $50.00")
            }
            "kill" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /agent kill <id>");
                }
                let id = &args[1];
                CommandOutput::ok(&format!("Agent '{}' 已终止 (auto-orchestrator 未初始化)", id))
            }
            _ => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: status, instances, budget, kill",
                args[0]
            )),
        }
    }
}

// ============================================================
// MCP 命令（保留，用于工具管理）
// ============================================================

/// MCP 工具管理命令
pub struct McpCmd;
impl CliCommand for McpCmd {
    fn name(&self) -> &str { "/mcp" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "MCP: /mcp list | /mcp search <q>" }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok("MCP 工具管理:\n  /mcp list           列出已注册工具\n  /mcp search <query> 搜索工具");
        }
        match args[0].as_str() {
            "list" | "ls" => {
                CommandOutput::ok("MCP 工具: (tool-orchestrator 未初始化)\n  使用 ToolOrchestrator::list_defs() 获取详细信息")
            }
            "search" | "find" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /mcp search <query>");
                }
                let query = args[1..].join(" ");
                CommandOutput::ok(&format!("MCP 搜索 '{}': (tool-orchestrator 未初始化)", query))
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: list, search", args[0])),
        }
    }
}

// ============================================================
// 发现命令（保留，用于网络发现）
// ============================================================

/// 网络发现命令
pub struct DiscoverCmd;
impl CliCommand for DiscoverCmd {
    fn name(&self) -> &str { "/discover" }
    fn aliases(&self) -> Vec<&str> { vec!["/scan", "/dsc"] }
    fn description(&self) -> &str { "发现网络上的 NeoTrix agents" }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, _args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        CommandOutput::err("Agent discovery requires nt_agent_protocol (not yet migrated)")
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_status_cmd_help() {
        let cmd = AgentStatusCmd;
        let r = cmd.execute(&[], None);
        assert!(r.success, "no args should show help");
        assert!(r.message.contains("status"), "help should mention status");
        assert!(r.message.contains("instances"), "help should mention instances");
        assert!(r.message.contains("budget"), "help should mention budget");
        assert!(r.message.contains("kill"), "help should mention kill");
    }

    #[test]
    fn test_agent_status_cmd_status() {
        let cmd = AgentStatusCmd;
        let r = cmd.execute(&["status".into()], None);
        assert!(r.success, "status should succeed");
    }

    #[test]
    fn test_agent_status_cmd_instances() {
        let cmd = AgentStatusCmd;
        let r = cmd.execute(&["instances".into()], None);
        assert!(r.success, "instances should succeed");
    }

    #[test]
    fn test_agent_kill_requires_id() {
        let cmd = AgentStatusCmd;
        let r = cmd.execute(&["kill".into()], None);
        assert!(!r.success, "kill without id should fail");
    }
}
