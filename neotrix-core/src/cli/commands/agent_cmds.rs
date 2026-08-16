//! Agent & MCP 命令 — Agent / Mcp

use std::sync::{Arc, LazyLock, OnceLock};
use tokio::sync::RwLock;

use crate::agent::tool::mcp::McpRegistry;
use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::l7_capability::nt_core_orch_agent::{SubagentConfig, SubagentManager, MessageType};
use crate::neotrix::nt_mind::SelfIteratingBrain;

static AGENT_MANAGER: LazyLock<Arc<RwLock<SubagentManager>>> =
    LazyLock::new(|| Arc::new(RwLock::new(SubagentManager::new())));
static MCP_REGISTRY: OnceLock<Arc<RwLock<McpRegistry>>> = OnceLock::new();
static TOOL_ORCHESTRATOR: OnceLock<Arc<RwLock<crate::agent::tool::ToolOrchestrator>>> = OnceLock::new();

/// Shared subagent registry — single owner across /agent and /board todo.
pub fn shared_subagent_manager() -> Arc<RwLock<SubagentManager>> {
    AGENT_MANAGER.clone()
}

pub fn set_mcp_registry(registry: McpRegistry) {
    MCP_REGISTRY.set(Arc::new(RwLock::new(registry))).ok();
}

pub fn get_mcp_registry() -> Arc<RwLock<McpRegistry>> {
    MCP_REGISTRY.get()
        .cloned()
        .unwrap_or_else(|| Arc::new(RwLock::new(McpRegistry::new())))
}

/// 注入生产初始化时构建的 ToolOrchestrator（吸收管线终点）。
pub fn set_tool_orchestrator(orch: crate::agent::tool::ToolOrchestrator) {
    TOOL_ORCHESTRATOR.set(Arc::new(RwLock::new(orch))).ok();
}

pub fn get_tool_orchestrator() -> Arc<RwLock<crate::agent::tool::ToolOrchestrator>> {
    TOOL_ORCHESTRATOR.get()
        .cloned()
        .unwrap_or_else(|| Arc::new(RwLock::new(crate::agent::tool::ToolOrchestrator::default())))
}

// ====== /agent ======

pub struct AgentCmd;
impl CliCommand for AgentCmd {
    fn name(&self) -> &str { "/agent" }
    fn aliases(&self) -> Vec<&str> { vec!["/agents"] }
    fn description(&self) -> &str {
        "Subagent管理: /agent spawn <name> <mode> | /agent list | /agent talk <id> <message> | /agent kill <id> | /agent status <id> | /agent background <name> <mode>"
    }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::ok(
                "Subagent管理:\n  /agent catalog                  查看内置 agent 目录\n  /agent spawn <name> <mode>        创建新子代理 (mode: 0-63)，name 为内置档案名时自动套用档案\n  /agent list                        列出所有活跃子代理\n  /agent talk <id> <message>         向子代理发送消息\n  /agent kill <id>                   终止子代理\n  /agent status <id>                 查看子代理状态\n  /agent background <name> <mode>    创建后台异步任务\n  /agent tasks                       列出所有后台任务"
            );
        }
        match args[0].as_str() {
            "catalog" => {
                if args.len() > 1 && args[1] == "file" {
                    // 文件驱动 agent 目录（~/.neotrix/agents/ + 项目 .neotrix/agents/）
                    return CommandOutput::ok(&crate::core::l7_capability::nt_core_orch_agent::AgentCatalog::catalog_full_text());
                }
                CommandOutput::ok(&crate::core::l7_capability::nt_core_orch_agent::AgentCatalog::catalog_text())
            }
            "spawn" => {
                if args.len() < 3 {
                    return CommandOutput::err("用法: /agent spawn <name> <mode>");
                }
                let mut mgr = AGENT_MANAGER.blocking_write();
                let name = &args[1];
                // 档案命中判定：文件驱动定义（~/.neotrix/agents）优先，其次内置静态档案。
                // 命中则套用档案的工具权限矩阵与分级，而非裸 E8 模式。
                use crate::core::nt_core_subagent::SubAgentRegistry;
                let mut file_reg = SubAgentRegistry::new();
                file_reg.scan_all();
                let is_known_agent = file_reg.get(name).is_some()
                    || crate::core::l7_capability::nt_core_orch_agent::AgentCatalog::by_name(name).is_some();
                if is_known_agent {
                    // 先去掉挡在前面阻塞写锁的临时借用再 spawn
                    return match mgr.spawn_from_profile(name) {
                        Ok(id) => CommandOutput::ok(&format!(
                            "Subagent spawned from catalog: {} (id: {}, E8 mode: {})",
                            name, id,
                            mgr.get(&id).map(|a| a.config.e8_mode).unwrap_or(0)
                        )),
                        Err(e) => CommandOutput::err(&e),
                    };
                }
                let mode: u8 = match args[2].parse() {
                    Ok(m) if m <= 63 => m,
                    _ => return CommandOutput::err("mode 必须是 0-63 之间的整数"),
                };
                let config = SubagentConfig {
                    name: name.to_string(),
                    e8_mode: mode,
                    description: format!("E8 mode {} subagent: {}", mode, name),
                    goal: format!("Execute tasks as {}", name),
                    capabilities: vec!["reason".into(), "search".into(), "communicate".into()],
                    max_context: 4096,
                    autostart: true,
                };
                let id = mgr.spawn(config);
                CommandOutput::ok(&format!("Subagent spawned: {} (id: {}, E8 mode: {})", name, id, mode))
            }
            "list" | "ls" => {
                let mgr = AGENT_MANAGER.blocking_read();
                let agents = mgr.list();
                if agents.is_empty() {
                    return CommandOutput::ok("No active subagents.");
                }
                let mut out = format!("Active subagents ({}):\n", agents.len());
                for a in &agents {
                    let status_str = match &a.status {
                        crate::core::l7_capability::nt_core_orch_agent::SubagentStatus::Idle => "idle",
                        crate::core::l7_capability::nt_core_orch_agent::SubagentStatus::Running { .. } => "running",
                        crate::core::l7_capability::nt_core_orch_agent::SubagentStatus::Completed { .. } => "completed",
                        crate::core::l7_capability::nt_core_orch_agent::SubagentStatus::Failed { .. } => "failed",
                        crate::core::l7_capability::nt_core_orch_agent::SubagentStatus::Paused => "paused",
                        crate::core::l7_capability::nt_core_orch_agent::SubagentStatus::Stale => "stale",
                    };
                    out.push_str(&format!("  {} | {} | E8:{} | {} | msgs:{}\n",
                        a.id, a.config.name, a.config.e8_mode, status_str, a.messages.len()));
                }
                CommandOutput::ok(&out)
            }
            "talk" => {
                if args.len() < 3 {
                    return CommandOutput::err("用法: /agent talk <id> <message>");
                }
                let id = &args[1];
                let message = args[2..].join(" ");
                let mut mgr = AGENT_MANAGER.blocking_write();
                match mgr.send_message("cli", id, &message, MessageType::Task) {
                    Ok(()) => CommandOutput::ok(&format!("Message sent to {}: {}", id, message)),
                    Err(e) => CommandOutput::err(&format!("Failed to send: {}", e)),
                }
            }
            "kill" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /agent kill <id>");
                }
                let id = &args[1];
                let mut mgr = AGENT_MANAGER.blocking_write();
                match mgr.kill(id) {
                    Some(agent) => CommandOutput::ok(&format!("Subagent '{}' ({}) terminated.", agent.config.name, id)),
                    None => CommandOutput::err(&format!("Subagent '{}' not found.", id)),
                }
            }
            "background" | "bg" => {
                if args.len() < 3 {
                    return CommandOutput::err("用法: /agent background <name> <mode>");
                }
                let name = &args[1];
                let mode: u8 = match args[2].parse() {
                    Ok(m) if m <= 63 => m,
                    _ => return CommandOutput::err("mode 必须是 0-63 之间的整数"),
                };
                let mut mgr = AGENT_MANAGER.blocking_write();
                let id = mgr.spawn_background(name, mode);
                CommandOutput::ok(&format!("Background task created: {} (id: {}, E8 mode: {})", name, id, mode))
            }
            "tasks" | "bglist" => {
                let mgr = AGENT_MANAGER.blocking_read();
                let tasks = mgr.list_tasks();
                if tasks.is_empty() {
                    return CommandOutput::ok("No background tasks.");
                }
                let mut out = format!("Background tasks ({}):\n", tasks.len());
                for t in &tasks {
                    let status_str = match &t.status {
                        crate::core::l7_capability::nt_core_orch_agent::TaskStatus::Pending => "pending",
                        crate::core::l7_capability::nt_core_orch_agent::TaskStatus::Running => "running",
                        crate::core::l7_capability::nt_core_orch_agent::TaskStatus::Completed(_) => "completed",
                        crate::core::l7_capability::nt_core_orch_agent::TaskStatus::Failed(_) => "failed",
                    };
                    out.push_str(&format!("  {} | {} | E8:{} | {}\n", t.id, t.name, t.e8_mode, status_str));
                }
                CommandOutput::ok(&out)
            }
            "status" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /agent status <id>");
                }
                let id = &args[1];
                let mgr = AGENT_MANAGER.blocking_read();
                match mgr.get(id) {
                    Some(agent) => {
                        let status_str = format!("{:?}", agent.status);
                        let mut out = format!("Subagent: {} ({})\n", agent.config.name, id);
                        out.push_str(&format!("  E8 Mode:     {}\n", agent.config.e8_mode));
                        out.push_str(&format!("  Status:      {}\n", status_str));
                        out.push_str(&format!("  Goal:        {}\n", agent.config.goal));
                        out.push_str(&format!("  Messages:    {}\n", agent.messages.len()));
                        out.push_str(&format!("  Created:     {}\n", agent.created_at));
                        out.push_str(&format!("  Last Active: {}\n", agent.last_active));
                        out.push_str(&format!("  Executions:  {}\n", agent.execution_count));
                        if let Some(plan) = &agent.current_plan {
                            out.push_str(&format!("  Plan:        {} ({} steps)\n", &plan.id[..8.min(plan.id.len())], plan.metrics.total_steps));
                        }
                        CommandOutput::ok(&out)
                    }
                    None => CommandOutput::err(&format!("Subagent '{}' not found.", id)),
                }
            }
            _ => CommandOutput::err(&format!("未知子命令: {}. 可用: spawn, list, talk, kill, status, background, tasks", args[0])),
        }
    }
}

// ====== /discover ======

pub struct DiscoverCmd;
impl CliCommand for DiscoverCmd {
    fn name(&self) -> &str { "/discover" }
    fn aliases(&self) -> Vec<&str> { vec!["/scan", "/dsc"] }
    fn description(&self) -> &str { "Scan for NeoTrix agents on the network: /discover [--json] [--port <port>] [--duration <ms>]" }
    fn is_primary(&self) -> bool { false }

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

        let mut discovery = match crate::neotrix::nt_agent_protocol::discovery::AgentDiscovery::new(port) {
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

                if agents.len() == 1 {
                    let a = &agents[0];
                    table.push_str("  详情:\n");
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
    fn description(&self) -> &str { "MCP: /mcp list|status|stubs|discover|search <q>|publish <name> <cmd>" }
    fn is_primary(&self) -> bool { false }

    fn execute(&self, args: &[String], brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        if args.is_empty() || (args.len() == 1 && args[0] == "--json") {
            return CommandOutput::err("用法: /mcp list [--json] | status | discover | search <query> | publish <name> <cmd> [args...]");
        }
        let cmd = args[0].as_str();
        match cmd {
            "list" | "ls" => {
                let registry = get_mcp_registry();
                let registry = registry.blocking_read();
                let tools = registry.list_tools();
                let mut s = format!("🔌 MCP Tools: {} registered\n", tools.len());
                for (i, tool) in tools.iter().enumerate() {
                    s.push_str(&format!("  {}. {} — {}\n", i + 1, tool.name, tool.description));
                }
                if tools.is_empty() {
                    s.push_str("  (none — use /mcp status for bridge status)\n");
                }
                let orch = get_tool_orchestrator();
                let orch = orch.blocking_read();
                let absorbed = orch.list_defs();
                s.push_str(&format!("🧩 Absorbed NativeTools: {}\n", absorbed.len()));
                for (i, def) in absorbed.iter().enumerate() {
                    s.push_str(&format!("  {}. {} — {}\n", i + 1, def.name, def.description));
                }
                if want_json {
                    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
                    let absorbed_names: Vec<&str> = absorbed.iter().map(|t| t.name.as_str()).collect();
                    return CommandOutput::ok(&s).with_json(serde_json::json!({
                        "tools": tool_names, "count": tool_names.len(),
                        "absorbed": absorbed_names, "absorbed_count": absorbed_names.len()
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
            "stubs" => {
                // PTC 接线 (programmatic_tool_calling): 渲染 Python 类型签名桩,
                // 供 agent 单 turn 内链式/并行调用 (typed-stub 工具调用)。
                let registry = get_mcp_registry();
                let registry = registry.blocking_read();
                let stubs = registry.gateway().tool_stubs();
                let mut s = format!("🐍 PTC stubs: {} typed signatures\n", stubs.len());
                for stub in stubs {
                    s.push_str(&format!("  def {}({}) -> str  # {}\n", stub.name, stub.signature, stub.doc));
                }
                if want_json {
                    let json: Vec<serde_json::Value> = registry.gateway().tool_stubs()
                        .iter()
                        .map(|t| serde_json::json!({ "name": t.name, "signature": t.signature, "doc": t.doc }))
                        .collect();
                    return CommandOutput::ok(&s).with_json(serde_json::json!({ "stubs": json, "count": json.len() }));
                }
                CommandOutput::ok(&s)
            }
            "discover" | "scan" => {
                use crate::neotrix::nt_agent_mcp_discovery::McpDiscovery;
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
                let registry = get_mcp_registry();
                let registry = registry.blocking_read();
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
                    .and_then(|i| args.get(i + 1)).cloned()
                    .unwrap_or_else(|| format!("user-published MCP server: {}", name));
                let registry = get_mcp_registry();
                let mut registry = registry.blocking_write();
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

    fn spawn_test_agent(name: &str, mode: u8) -> String {
        let cmd = AgentCmd;
        let r = cmd.execute(&["spawn".into(), name.into(), mode.to_string()], None);
        assert!(r.success, "spawn should succeed: {}", r.message);
        // Extract id from "Subagent spawned: name (id: agent-NNNN, E8 mode: N)"
        let id_part = r.message.split("(id: ").nth(1)
            .and_then(|s| s.split(", ").next())
            .unwrap();
        id_part.trim().to_string()
    }

    #[test]
    fn test_agent_spawn_and_list() {
        let _id = spawn_test_agent("test-agent", 7);
        let cmd = AgentCmd;
        let r = cmd.execute(&["list".into()], None);
        assert!(r.success, "list should succeed");
        assert!(r.message.contains("test-agent"), "list should show spawned agent");
        assert!(r.message.contains("E8:7"), "list should show E8 mode");
    }

    #[test]
    fn test_agent_spawn_invalid_mode() {
        let cmd = AgentCmd;
        let r = cmd.execute(&["spawn".into(), "bad".into(), "99".into()], None);
        assert!(!r.success, "invalid mode should fail");
    }

    #[test]
    fn test_agent_talk_and_kill() {
        let id = spawn_test_agent("chatty", 3);
        let cmd = AgentCmd;

        let r = cmd.execute(&["talk".into(), id.clone(), "hello".into()], None);
        assert!(r.success, "talk should succeed: {:?}", r.message);

        let r = cmd.execute(&["kill".into(), id.clone()], None);
        assert!(r.success, "kill should succeed: {:?}", r.message);

        let r = cmd.execute(&["status".into(), id.clone()], None);
        assert!(!r.success, "status after kill should fail");
    }

    #[test]
    fn test_agent_talk_unknown() {
        let cmd = AgentCmd;
        let r = cmd.execute(&["talk".into(), "nonexistent".into(), "hi".into()], None);
        assert!(!r.success, "talk to unknown agent should fail");
    }

    #[test]
    fn test_agent_kill_unknown() {
        let cmd = AgentCmd;
        let r = cmd.execute(&["kill".into(), "nonexistent".into()], None);
        assert!(!r.success, "kill unknown agent should fail");
    }

    #[test]
    fn test_agent_no_args() {
        let cmd = AgentCmd;
        let r = cmd.execute(&[], None);
        assert!(r.success, "no args should show help");
        assert!(r.message.contains("spawn"), "help should mention spawn");
        assert!(r.message.contains("list"), "help should mention list");
        assert!(r.message.contains("talk"), "help should mention talk");
        assert!(r.message.contains("kill"), "help should mention kill");
        assert!(r.message.contains("status"), "help should mention status");
        assert!(r.message.contains("background"), "help should mention background");
    }

    #[test]
    fn test_agent_status() {
        let id = spawn_test_agent("status-check", 15);
        let cmd = AgentCmd;
        let r = cmd.execute(&["status".into(), id.clone()], None);
        assert!(r.success, "status should succeed: {:?}", r.message);
        assert!(r.message.contains("status-check"));
        assert!(r.message.contains("E8 Mode:     15"));
    }
}
