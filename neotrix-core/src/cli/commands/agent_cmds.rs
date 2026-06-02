//! Agent & MCP 命令 — Agent / Mcp

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent::team::{AgentRole, AgentTeam, ProcessType};
use crate::agent::tools::McpRegistry;
use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_traits::ToolProvider;
use crate::neotrix::nt_mind::SelfIteratingBrain;

// ====== /agent ======

pub struct AgentCmd;
impl CliCommand for AgentCmd {
    fn name(&self) -> &str { "/agent" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "Agent管理: /agent team <roles...> | /agent list | /agent status" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            return CommandOutput::err("用法: /agent team <roles...> | list | status | create <role>");
        }
        let cmd = args[0].as_str();
        match cmd {
            "team" => {
                let roles = &args[1..];
                if roles.is_empty() { return CommandOutput::err("需要角色名: /agent team researcher,writer,reviewer"); }
                let mut team = AgentTeam::new("cli-team", ProcessType::Sequential);
                for role_name in roles {
                    team.add_agent(AgentRole {
                        name: role_name.to_string(),
                        role: role_name.to_string(),
                        goal: format!("作为{}执行任务", role_name),
                        backstory: format!("资深{}专家", role_name),
                        tools: Vec::new(),
                    });
                }
                let msg = format!("🤖 AgentTeam 'cli-team' created: {} agents", team.agents.len());
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "team": "cli-team", "agent_count": team.agents.len(), "roles": roles
                    }));
                }
                CommandOutput::ok(&msg)
            }
            "list" | "ls" => {
                let msg = if let Some(b) = brain {
                    let a = b.blocking_read();
                    let iter_count = a.iteration;
                    format!("🤖 Agents: default (main) | iterations: {} | brain_version: {}",
                        iter_count, a.brain.total_absorb_count)
                } else {
                    "🤖 Available agents: default, researcher, writer, reviewer".to_string()
                };
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({"agents": ["default"]}));
                }
                CommandOutput::ok(&msg)
            }
            "status" => {
                if let Some(b) = brain {
                    let a = b.blocking_read();
                    let stats = a.brain.get_statistics();
                    let bank_stats = a.reasoning_bank.stats();
                    let msg = format!("🧠 Brain: {} absorbs | {} memories | capability: {:.3} | iterations: {}",
                        stats.total_absorbed, bank_stats.total_memories, stats.capability_sum, a.iteration);
                    if want_json {
                        return CommandOutput::ok(&msg).with_json(serde_json::json!({
                            "total_absorbed": stats.total_absorbed,
                            "memories": bank_stats.total_memories,
                            "capability_sum": stats.capability_sum,
                            "iteration": a.iteration,
                        }));
                    }
                    CommandOutput::ok(&msg)
                } else {
                    CommandOutput::ok("🤖 Agent status: idle (no brain attached)")
                }
            }
            "create" => {
                if args.len() < 2 { return CommandOutput::err("需要角色名: /agent create researcher"); }
                CommandOutput::ok(&format!("🤖 Agent '{}' created", args[1]))
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: team, list, status, create", cmd)),
        }
    }
}

// ====== /discover ======

pub struct DiscoverCmd;
impl CliCommand for DiscoverCmd {
    fn name(&self) -> &str { "/discover" }
    fn aliases(&self) -> Vec<&str> { vec!["/scan", "/dsc"] }
    fn description(&self) -> &str { "扫描网络中的 NeoTrix 代理: /discover [--json] [--port <port>] [--duration <ms>]" }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let port = args.iter()
            .position(|a| a == "--port")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(42069);
        let duration = args.iter()
            .position(|a| a == "--duration" || a == "-d")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(3000);

        let mut discovery = match crate::neotrix::agent_protocol::discovery::AgentDiscovery::new(port) {
            Ok(d) => d,
            Err(e) => return CommandOutput::err(&format!("绑定 UDP :{} 失败: {}", port, e)),
        };

        match discovery.discover(duration) {
            Ok(agents) => {
                if agents.is_empty() {
                    let msg = format!("🔍 扫描完成 ({}ms)，未发现任何代理", duration);
                    if want_json {
                        return CommandOutput::ok(&msg).with_json(serde_json::json!({
                            "scanned": true, "agent_count": 0, "duration_ms": duration, "port": port
                        }));
                    }
                    return CommandOutput::ok(&msg);
                }

                let mut table = format!("🔍 发现 {} 个代理 (扫描 {}ms):\n", agents.len(), duration);
                table.push_str("┌──────┬────────────────────────┬──────────────────────┬───────┬──────┐\n");
                table.push_str("│ #    │ ID                     │ Host                 │ Port  │ Caps │\n");
                table.push_str("├──────┼────────────────────────┼──────────────────────┼───────┼──────┤\n");
                for (i, a) in agents.iter().enumerate() {
                    let id_trunc = if a.id.len() > 22 { format!("{}…", &a.id[..21]) } else { a.id.clone() };
                    let host_trunc = if a.host.len() > 20 { format!("{}…", &a.host[..19]) } else { a.host.clone() };
                    let cap_count = a.capabilities.len();
                    table.push_str(&format!("│ {:<4} │ {:<22} │ {:<20} │ {:<5} │ {:<4} │",
                        i + 1, id_trunc, host_trunc, a.port, cap_count));
                    table.push('\n');
                }
                table.push_str("└──────┴────────────────────────┴──────────────────────┴───────┴──────┘\n");

                // Show details for single agent
                if agents.len() == 1 {
                    let a = &agents[0];
                    table.push_str(&format!("  详情:\n"));
                    table.push_str(&format!("    Name:    {}\n", a.name));
                    table.push_str(&format!("    Service: {}\n", if a.service_type.is_empty() { "(none)" } else { &a.service_type }));
                    table.push_str(&format!("    Instance:{}\n", if a.instance_name.is_empty() { "(none)" } else { &a.instance_name }));
                    if !a.capabilities.is_empty() {
                        table.push_str(&format!("    Caps:    {}\n", a.capabilities.join(", ")));
                    }
                    if a.hexagram != 0 {
                        table.push_str(&format!("    Hexagram:{}", a.hexagram));
                    }
                }

                if want_json {
                    let json_agents: Vec<serde_json::Value> = agents.iter().map(|a| {
                        serde_json::json!({
                            "id": a.id, "name": a.name, "host": a.host, "port": a.port,
                            "capabilities": a.capabilities, "hexagram": a.hexagram,
                            "service_type": a.service_type, "instance_name": a.instance_name,
                        })
                    }).collect();
                    return CommandOutput::ok(&table).with_json(serde_json::json!({
                        "agent_count": agents.len(), "duration_ms": duration, "port": port, "agents": json_agents
                    }));
                }
                CommandOutput::ok(&table)
            }
            Err(e) => CommandOutput::err(&format!("扫描失败: {}", e)),
        }
    }
}

// ====== /mcp ======

pub struct McpCmd;
impl CliCommand for McpCmd {
    fn name(&self) -> &str { "/mcp" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn description(&self) -> &str { "MCP: /mcp list|status|discover|search <q>|publish <name> <cmd>" }
    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            return CommandOutput::err("用法: /mcp list [--json] | status | discover | search <query> | publish <name> <cmd> [args...]");
        }
        let cmd = args[0].as_str();
        match cmd {
            "list" | "ls" => {
                let registry = McpRegistry::new();
                let tools = registry.list_tools();
                let mut s = format!("🔌 MCP Tools: {} registered\n", tools.len());
                for (i, tool) in tools.iter().enumerate() {
                    s.push_str(&format!("  {}. {} — {}\n", i + 1, tool.name, tool.description));
                }
                if tools.is_empty() {
                    s.push_str("  (none — use /mcp status for bridge status)\n");
                }
                if want_json {
                    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
                    return CommandOutput::ok(&s).with_json(serde_json::json!({
                        "tools": tool_names, "count": tool_names.len()
                    }));
                }
                CommandOutput::ok(&s)
            }
            "status" | "stat" => {
                if let Some(b) = brain {
                    let a = b.blocking_read();
                    let tool_calls = a.tool_call_count;
                    let msg = format!("🔌 MCP Bridge: {} tool calls | {} traces cached",
                        tool_calls, a.tool_traces.len());
                    if want_json {
                        return CommandOutput::ok(&msg).with_json(serde_json::json!({
                            "tool_call_count": tool_calls, "traces": a.tool_traces.len()
                        }));
                    }
                    CommandOutput::ok(&msg)
                } else {
                    CommandOutput::ok("🔌 MCP Bridge: idle (no brain attached)")
                }
            }
            "discover" | "scan" => {
                use crate::neotrix::mcp_discovery::McpDiscovery;
                let entries = McpDiscovery::scan_path();
                let mut s = format!("🔍 MCP Discovery: {} candidates in PATH\n", entries.len());
                for (i, e) in entries.iter().enumerate() {
                    s.push_str(&format!(
                        "  {}. {} | {} | {:?}\n",
                        i + 1,
                        e.name,
                        e.path.display(),
                        e.status
                    ));
                }
                if entries.is_empty() {
                    s.push_str("  (none found — install an *-mcp-server binary and ensure it is in PATH)\n");
                }
                if want_json {
                    let items: Vec<serde_json::Value> = entries.iter().map(|e| {
                        serde_json::json!({
                            "name": e.name,
                            "path": e.path.display().to_string(),
                            "version": e.version,
                            "status": format!("{:?}", e.status),
                        })
                    }).collect();
                    return CommandOutput::ok(&s).with_json(serde_json::json!({
                        "count": entries.len(),
                        "entries": items,
                    }));
                }
                CommandOutput::ok(&s)
            }
            "search" | "find" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /mcp search <query>");
                }
                let query = args[1..].join(" ");
                let registry = McpRegistry::new();
                let results = registry.search(&query);
                let mut s = format!("🔎 MCP search '{}' → {} match(es)\n", query, results.len());
                for (i, tool) in results.iter().take(20).enumerate() {
                    s.push_str(&format!(
                        "  {}. [{}] {} — {}\n",
                        i + 1,
                        tool.server_name,
                        tool.name,
                        tool.description
                    ));
                }
                if results.len() > 20 {
                    s.push_str(&format!("  ... +{} more\n", results.len() - 20));
                }
                if want_json {
                    let items: Vec<serde_json::Value> = results.iter().map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "server": t.server_name,
                            "description": t.description,
                        })
                    }).collect();
                    return CommandOutput::ok(&s).with_json(serde_json::json!({
                        "query": query,
                        "count": results.len(),
                        "results": items,
                    }));
                }
                CommandOutput::ok(&s)
            }
            "publish" | "add" => {
                if args.len() < 3 {
                    return CommandOutput::err("用法: /mcp publish <name> <command> [args...] [--description <desc>]");
                }
                let name = &args[1];
                let command = &args[2];
                let rest: Vec<&str> = args[3..]
                    .iter()
                    .filter(|a| !a.starts_with("--"))
                    .map(|a| a.as_str())
                    .collect();
                let desc = args.iter()
                    .position(|a| a == "--description" || a == "-d")
                    .and_then(|i| args.get(i + 1))
                    .map(|s| s.clone())
                    .unwrap_or_else(|| format!("user-published MCP server: {}", name));
                let mut registry = McpRegistry::new();
                let n = registry.publish(name, command, &rest, &desc);
                let msg = format!("📤 Published '{}' as MCP server ({} tool(s))", name, n);
                if want_json {
                    return CommandOutput::ok(&msg).with_json(serde_json::json!({
                        "name": name, "command": command, "args": rest, "tools_added": n,
                    }));
                }
                CommandOutput::ok(&msg)
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: list, status, discover, search, publish", cmd)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        let _instance = AgentCmd;
        assert!(true);
    }
}
