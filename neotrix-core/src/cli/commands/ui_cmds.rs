//! UI commands — Side / WorkSpace / Router / Background

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::core::nt_core_router::{TaskComplexity, TaskContext, SMART_ROUTER};
use crate::core::WORKSPACE_MANAGER;
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_mind_background_loop::always_on::ALWAYS_ON_ENGINE;

// ====== /side ======

pub struct SideCmd;
impl CliCommand for SideCmd {
    fn name(&self) -> &str {
        "/side"
    }
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str {
        "Quick side question (non-blocking): /side <question> | /side clear"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        if args.is_empty() || args[0] == "--json" {
            return CommandOutput::ok("用法:\n  /side <question>    快速提问，结果以 [Side] 前缀显示\n  /side clear         清除侧边会话历史");
        }
        if args[0] == "clear" {
            return CommandOutput::ok("[Side] clear — handled in app loop");
        }
        CommandOutput::ok(&format!("[Side] queued: {}", args.join(" ")))
    }
}

// ====== /workspace ======

pub struct WorkSpaceCmd;
impl CliCommand for WorkSpaceCmd {
    fn name(&self) -> &str {
        "/workspace"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/ws"]
    }
    fn description(&self) -> &str {
        "Workspace management: create | list | switch | delete | rename | status"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let clean_args: Vec<&str> = args
            .iter()
            .map(|s| s.as_str())
            .filter(|a| *a != "--json")
            .collect();

        if clean_args.is_empty() {
            let msg = "用法:\n  /workspace create <name> [--path <dir>] [--desc <text>]  创建新 WorkSpace\n  /workspace list                                          列出所有 WorkSpace\n  /workspace switch <id>                                   切换 WorkSpace\n  /workspace delete <id>                                   删除 WorkSpace\n  /workspace rename <id> <new_name>                        重命名 WorkSpace\n  /workspace status                                        显示当前 WorkSpace";
            let out = CommandOutput::ok(msg);
            return if want_json {
                out.with_json(serde_json::json!({"subcommands": ["create", "list", "switch", "delete", "rename", "status"]}))
            } else {
                out
            };
        }

        let mut mgr = WORKSPACE_MANAGER.lock().unwrap_or_else(|e| e.into_inner());
        let sub = clean_args[0];

        match sub {
            "create" | "new" => {
                if clean_args.len() < 2 {
                    return CommandOutput::err(
                        "用法: /workspace create <name> [--path <dir>] [--desc <text>]",
                    );
                }
                let name = clean_args[1];
                let path_idx = clean_args.iter().position(|a| *a == "--path" || *a == "-p");
                let desc_idx = clean_args.iter().position(|a| *a == "--desc" || *a == "-d");
                let project_root = path_idx
                    .and_then(|i| clean_args.get(i + 1))
                    .map(|s| std::path::PathBuf::from(s));
                let description = desc_idx
                    .and_then(|i| clean_args.get(i + 1))
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let ws = mgr.create(name, project_root, &description);
                let _ = mgr.save();
                let msg = format!("✅ WorkSpace created: {} (id={})", ws.name, ws.id);
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "created": true, "id": ws.id, "name": ws.name,
                        "project_root": ws.project_root, "description": ws.description,
                    }))
                } else {
                    out
                }
            }
            "list" | "ls" => {
                let spaces = mgr.list();
                if spaces.is_empty() {
                    return CommandOutput::ok(
                        "📂 No workspaces. Create one with /workspace create <name>",
                    );
                }
                let mut s = format!("📂 WorkSpaces ({}):\n", spaces.len());
                for ws in spaces {
                    let active_mark = if Some(ws.id.as_str()) == mgr.active_id.as_deref() {
                        " *"
                    } else {
                        "  "
                    };
                    let root = ws
                        .project_root
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    s.push_str(&format!(
                        "  {}{} {} (root: {})\n",
                        active_mark, ws.id, ws.name, root
                    ));
                }
                let out = CommandOutput::ok(&s);
                if want_json {
                    let json_list: Vec<serde_json::Value> = spaces.iter().map(|ws| serde_json::json!({
                        "id": ws.id, "name": ws.name, "active": Some(ws.id.as_str()) == mgr.active_id.as_deref(),
                        "project_root": ws.project_root, "description": ws.description,
                    })).collect();
                    out.with_json(
                        serde_json::json!({"workspaces": json_list, "count": json_list.len()}),
                    )
                } else {
                    out
                }
            }
            "switch" | "use" | "activate" => {
                if clean_args.len() < 2 {
                    return CommandOutput::err("用法: /workspace switch <id>");
                }
                match mgr.switch(clean_args[1]) {
                    Ok(()) => {
                        let _ = mgr.save();
                        let out = CommandOutput::ok(&format!(
                            "➡️ Switched to workspace: {}",
                            clean_args[1]
                        ));
                        if want_json {
                            out.with_json(
                                serde_json::json!({"switched": true, "id": clean_args[1]}),
                            )
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "delete" | "rm" => {
                if clean_args.len() < 2 {
                    return CommandOutput::err("用法: /workspace delete <id>");
                }
                match mgr.delete(clean_args[1]) {
                    Ok(()) => {
                        let _ = mgr.save();
                        let out =
                            CommandOutput::ok(&format!("🗑️ Deleted workspace: {}", clean_args[1]));
                        if want_json {
                            out.with_json(serde_json::json!({"deleted": true, "id": clean_args[1]}))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "rename" | "mv" => {
                if clean_args.len() < 3 {
                    return CommandOutput::err("用法: /workspace rename <id> <new_name>");
                }
                match mgr.rename(clean_args[1], clean_args[2]) {
                    Ok(()) => {
                        let _ = mgr.save();
                        let out = CommandOutput::ok(&format!(
                            "✏️ Renamed {} → {}",
                            clean_args[1], clean_args[2]
                        ));
                        if want_json {
                            out.with_json(serde_json::json!({"renamed": true, "id": clean_args[1], "new_name": clean_args[2]}))
                        } else {
                            out
                        }
                    }
                    Err(e) => CommandOutput::err(&e),
                }
            }
            "status" | "info" | "current" => match mgr.active() {
                Some(ws) => {
                    let msg = format!(
                            "📌 Active WorkSpace: {} (id={})\n  Created: {}\n  Root: {}\n  Description: {}\n  Tags: {}\n  Memories: {} | Goals: {} | Skills: {}",
                            ws.name, ws.id, ws.created_at.format("%Y-%m-%d %H:%M:%S"),
                            ws.project_root.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "(none)".to_string()),
                            ws.description,
                            if ws.tags.is_empty() { "(none)".to_string() } else { ws.tags.join(", ") },
                            ws.memory_count, ws.goal_count, ws.skill_count,
                        );
                    let out = CommandOutput::ok(&msg);
                    if want_json {
                        out.with_json(serde_json::json!({
                            "active": true, "id": ws.id, "name": ws.name,
                            "created_at": ws.created_at.to_rfc3339(),
                            "project_root": ws.project_root, "description": ws.description,
                            "tags": ws.tags, "memory_count": ws.memory_count,
                            "goal_count": ws.goal_count, "skill_count": ws.skill_count,
                        }))
                    } else {
                        out
                    }
                }
                None => {
                    let out = CommandOutput::ok(
                        "📌 No active workspace. Create one with /workspace create <name>",
                    );
                    if want_json {
                        out.with_json(serde_json::json!({"active": false}))
                    } else {
                        out
                    }
                }
            },
            _ => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: create, list, switch, delete, rename, status",
                sub
            )),
        }
    }
}

// ====== /route ======

pub struct RouterCmd;
impl CliCommand for RouterCmd {
    fn name(&self) -> &str {
        "/route"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/router"]
    }
    fn description(&self) -> &str {
        "Smart router: status | enable | disable | reset | set <complexity> <provider> <model> | classify <text>"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let plain_args: Vec<&str> = args
            .iter()
            .map(|s| s.as_str())
            .filter(|a| *a != "--json")
            .collect();

        if plain_args.is_empty() {
            let msg = "用法:\n  /route status             显示路由状态\n  /route enable             启用智能路由\n  /route disable            禁用智能路由(使用默认provider)\n  /route reset              重置路由统计\n  /route set <complexity> <provider> <model> [cost_in cost_out]  设置路由规则\n  /route classify <text>    测试分类器";
            let out = CommandOutput::ok(msg);
            return if want_json {
                out.with_json(serde_json::json!({"subcommands": ["status", "enable", "disable", "reset", "set", "classify"]}))
            } else {
                out
            };
        }

        let sub = plain_args[0];
        match sub {
            "status" | "stats" => {
                let router = SMART_ROUTER.lock().unwrap_or_else(|e| e.into_inner());
                let msg = router.savings_report();
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "enabled": router.enabled,
                        "total_routes": router.stats.total_routes,
                        "estimated_savings": router.stats.estimated_savings,
                        "actual_cost": router.stats.actual_cost,
                        "flagship_cost": router.stats.flagship_cost,
                    }))
                } else {
                    out
                }
            }
            "enable" | "on" => {
                let mut router = SMART_ROUTER.lock().unwrap_or_else(|e| e.into_inner());
                router.set_enabled(true);
                let _ = router.save();
                let out = CommandOutput::ok("🔀 智能路由已启用");
                if want_json {
                    out.with_json(serde_json::json!({"smart_router": "enabled"}))
                } else {
                    out
                }
            }
            "disable" | "off" => {
                let mut router = SMART_ROUTER.lock().unwrap_or_else(|e| e.into_inner());
                router.set_enabled(false);
                let _ = router.save();
                let out = CommandOutput::ok("🔀 智能路由已禁用，将使用默认 flagship provider");
                if want_json {
                    out.with_json(serde_json::json!({"smart_router": "disabled"}))
                } else {
                    out
                }
            }
            "reset" => {
                let mut router = SMART_ROUTER.lock().unwrap_or_else(|e| e.into_inner());
                router.reset_stats();
                let out = CommandOutput::ok("🔀 路由统计已重置");
                if want_json {
                    out.with_json(serde_json::json!({"routing_stats": "reset"}))
                } else {
                    out
                }
            }
            "set" | "rule" => {
                if plain_args.len() < 4 {
                    return CommandOutput::err(
                        "用法: /route set <complexity> <provider> <model> [cost_in cost_out]",
                    );
                }
                let complexity =
                    match TaskComplexity::from_str(plain_args[1]) {
                        Some(c) => c,
                        None => return CommandOutput::err(&format!(
                            "无效复杂度: {}。可用: trivial, simple, moderate, complex, critical",
                            plain_args[1]
                        )),
                    };
                let provider = plain_args[2];
                let model = plain_args[3];
                let cost_in = plain_args
                    .get(4)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.01);
                let cost_out = plain_args
                    .get(5)
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.03);
                {
                    let mut router = SMART_ROUTER.lock().unwrap_or_else(|e| e.into_inner());
                    router.set_rule(complexity, provider, model, cost_in, cost_out);
                    let _ = router.save();
                }
                let msg = format!(
                    "🔀 路由规则已设置: {} → {} / {} (${:.4} in, ${:.4} out)",
                    complexity.label(),
                    provider,
                    model,
                    cost_in,
                    cost_out
                );
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "complexity": complexity.label(),
                        "provider": provider,
                        "model": model,
                        "cost_in": cost_in,
                        "cost_out": cost_out,
                    }))
                } else {
                    out
                }
            }
            "classify" | "test" => {
                if plain_args.len() < 2 {
                    return CommandOutput::err("用法: /route classify <text>");
                }
                let text = plain_args[1..].join(" ");
                let ctx = TaskContext::new(&text);
                let complexity = TaskComplexity::classify(&text, &ctx);
                let msg = format!(
                    "🔀 分类结果: {}\n\n特征分析:\n  文本长度: {} 字符\n  文件引用: {} (提及 {} 个文件)\n  Git 上下文: {}\n  关键词: {:?}",
                    complexity.label(),
                    ctx.prompt_length,
                    if ctx.mentions_files { "是" } else { "否" },
                    ctx.file_count,
                    if ctx.has_git_context { "是" } else { "否" },
                    ctx.keywords,
                );
                let out = CommandOutput::ok(&msg);
                if want_json {
                    out.with_json(serde_json::json!({
                        "complexity": complexity.label(),
                        "prompt_length": ctx.prompt_length,
                        "mentions_files": ctx.mentions_files,
                        "file_count": ctx.file_count,
                        "has_git_context": ctx.has_git_context,
                        "keywords": ctx.keywords,
                    }))
                } else {
                    out
                }
            }
            _ => CommandOutput::err(&format!(
                "未知子命令: {}。可用: status, enable, disable, reset, set, classify",
                sub
            )),
        }
    }
}

// ====== /vim ======

pub struct VimCmd;
impl CliCommand for VimCmd {
    fn name(&self) -> &str {
        "/vim"
    }
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str {
        "Vim mode: /vim toggle | /vim on | /vim off (works in TUI)"
    }
    fn execute(
        &self,
        _args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        CommandOutput::ok("Vim 模式 — 在 TUI 中使用 /vim toggle 切换。\n  /vim toggle  切换开关\n  /vim on      启用\n  /vim off     禁用")
    }
}

// ====== /background ======

pub struct BackgroundCommand;
impl CliCommand for BackgroundCommand {
    fn name(&self) -> &str {
        "/background"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/bg"]
    }
    fn description(&self) -> &str {
        "Manage always-on background engine: start | stop | status | task | cycle"
    }

    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("status");

        match subcmd {
            "start" => {
                let mut engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                match engine.start() {
                    Ok(()) => {
                        let _ = engine.save();
                        CommandOutput::ok("Always-on engine started")
                    }
                    Err(e) => CommandOutput::err(&format!("Failed to start: {}", e)),
                }
            }
            "stop" => {
                let mut engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                match engine.stop() {
                    Ok(()) => {
                        let _ = engine.save();
                        CommandOutput::ok("Always-on engine stopped")
                    }
                    Err(e) => CommandOutput::err(&format!("Failed to stop: {}", e)),
                }
            }
            "status" => {
                let engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                let s = engine.status();
                let msg = format!(
                    "Always-On Engine:\n  Enabled: {}\n  State: {}\n  Tasks: {} total, {} active, {} completed\n  Uptime: {}s\n  Last Cycle: {}",
                    s.enabled, s.state, s.total_tasks, s.active_tasks, s.completed_tasks, s.uptime_secs,
                    s.last_cycle.as_deref().unwrap_or("never"),
                );
                CommandOutput::ok(&msg)
            }
            "task" => {
                let task_args: Vec<String> = args.iter().skip(1).cloned().collect();
                let task_subcmd = task_args.first().map(|s| s.as_str()).unwrap_or("list");
                match task_subcmd {
                    "add" => {
                        let desc: String = task_args
                            .iter()
                            .skip(1)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" ");
                        if desc.is_empty() {
                            return CommandOutput::err("Usage: /background task add <description>");
                        }
                        let clean_desc: String = desc
                            .split("--interval")
                            .next()
                            .unwrap_or(&desc)
                            .trim()
                            .to_string();
                        let mut engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                        let id = engine.add_oneshot(&clean_desc);
                        let _ = engine.save();
                        drop(engine);
                        CommandOutput::ok(&format!("Added task: {} (id={})", clean_desc, id))
                    }
                    "list" => {
                        let engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                        let filter = task_args.get(1).map(|s| s.as_str());
                        let tasks = engine.list_tasks(filter);
                        if tasks.is_empty() {
                            drop(engine);
                            return CommandOutput::ok("No tasks");
                        }
                        let mut msg = String::from("Tasks:\n");
                        for t in &tasks {
                            let runs = format!("{}/{}", t.run_count, t.max_runs);
                            let interval = if t.interval_secs > 0 {
                                format!("every {}s", t.interval_secs)
                            } else {
                                "oneshot".into()
                            };
                            msg.push_str(&format!(
                                "  [{}] {} (runs={}, {})\n",
                                t.id, t.description, runs, interval
                            ));
                        }
                        CommandOutput::ok(msg.trim())
                    }
                    "remove" => {
                        let id = task_args.get(1).map(|s| s.as_str()).unwrap_or("");
                        if id.is_empty() {
                            return CommandOutput::err("Usage: /background task remove <id>");
                        }
                        let mut engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                        match engine.remove_task(id) {
                            Ok(()) => {
                                let _ = engine.save();
                                CommandOutput::ok(&format!("Removed task: {}", id))
                            }
                            Err(e) => CommandOutput::err(&e),
                        }
                    }
                    _ => CommandOutput::err("Usage: /background task add|list|remove"),
                }
            }
            "cycle" => {
                let mut engine = ALWAYS_ON_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
                match engine.full_cycle() {
                    Ok(report) => {
                        let _ = engine.save();
                        CommandOutput::ok(&format!(
                            "Cycle complete: scanned={}, executed={}, took={}ms",
                            report.scan_count, report.tasks_executed, report.duration_ms
                        ))
                    }
                    Err(e) => CommandOutput::err(&format!("Cycle failed: {}", e)),
                }
            }
            _ => CommandOutput::err("Usage: /background start|stop|status|task|cycle"),
        }
    }
}
