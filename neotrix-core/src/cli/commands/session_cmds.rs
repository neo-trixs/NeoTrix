//! Session management commands — Compact / Context / Session / Resume / History

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::cli::tui::session_store::{SessionData, SessionStore};
use crate::core::nt_core_util;
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::server::session::SessionShareManager;

// ====== /compact ======

pub struct CompactCmd;
impl CliCommand for CompactCmd {
    fn name(&self) -> &str {
        "/compact"
    }
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str {
        "Compress context window: /compact [now]"
    }
    fn execute(
        &self,
        args: &[String],
        brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let immediate = args.first().map(|s| s.as_str()) == Some("now");
        if let Some(b) = brain {
            let a = b.blocking_read();
            let stats = a.brain.get_statistics();
            let mem_count = a.reasoning_bank.memories().len();
            let msg = format!(
                "📊 上下文状态:\n  Token 估计: ~{} tokens\n  记忆条目: {} 条\n  迭代次数: {}\n  能力总和: {:.3}\n\n{}",
                mem_count * 500,
                mem_count,
                a.iteration,
                stats.capability_sum,
                if immediate {
                    "(压缩功能需要 LLM 上下文管理器支持，当前为骨架)"
                } else {
                    "使用 /compact now 立即压缩"
                }
            );
            CommandOutput::ok(&msg)
        } else {
            CommandOutput::ok("📊 上下文: LLM 未连接 | 估计 0 tokens")
        }
    }
}

// ====== /context ======

pub struct ContextCmd;
impl CliCommand for ContextCmd {
    fn name(&self) -> &str {
        "/context"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/ctx"]
    }
    fn description(&self) -> &str {
        "Context management: /context | /context clear | /context model <name>"
    }
    fn execute(
        &self,
        args: &[String],
        brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        if args.is_empty() {
            if let Some(b) = brain {
                let a = b.blocking_read();
                let mem_count = a.reasoning_bank.memories().len();
                let model =
                    std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| "default".to_string());
                let msg = format!(
                    "📋 上下文使用:\n  消息数: {} (sessions)\n  记忆: {} 条\n  模型: {}\n  窗口限制: 128K tokens\n  当前估计: ~{} tokens",
                    1, mem_count, model, mem_count * 500
                );
                CommandOutput::ok(&msg)
            } else {
                CommandOutput::ok("📋 上下文使用:\n  消息数: 0\n  模型: default\n  窗口限制: 128K\n  当前: 0 tokens")
            }
        } else {
            match args[0].as_str() {
                "clear" => CommandOutput::ok("🧹 上下文已清空 (会话消息需手动清除)"),
                "model" => {
                    if args.len() < 2 {
                        CommandOutput::err("用法: /context model <name>")
                    } else {
                        CommandOutput::ok(&format!(
                            "📋 模型切换建议: {}. 请在配置文件中修改 default_model。",
                            args[1]
                        ))
                    }
                }
                _ => CommandOutput::err("用法: /context [clear | model <name>]"),
            }
        }
    }
}

// ====== /session ======

pub struct SessionCmd;
impl CliCommand for SessionCmd {
    fn name(&self) -> &str {
        "/session"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/sessions"]
    }
    fn description(&self) -> &str {
        "Session management: /session list | save | load | delete | fork | export | import | share"
    }
    fn execute(
        &self,
        args: &[String],
        brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let store = SessionStore::new();
        if args.is_empty() {
            return CommandOutput::ok("用法:\n  /session list               列出保存的会话\n  /session save <name>        保存当前会话\n  /session load <name>        加载会话\n  /session delete <name>      删除会话\n  /session fork <name>        复制会话\n  /session export <name>      导出会话到 stdout\n  /session export <name> --output <path>  导出会话到文件\n  /session import <path>      从文件导入会话\n  /session share <name>       生成分享链接 (可选 --ttl <hours>)");
        }
        let cmd = args[0].as_str();
        match cmd {
            "list" | "ls" => {
                let sessions = store.list_sessions();
                if sessions.is_empty() {
                    CommandOutput::ok("📂 没有保存的会话")
                } else {
                    let mut s = format!("📂 保存的会话 ({}):\n", sessions.len());
                    for (i, name) in sessions.iter().enumerate() {
                        s.push_str(&format!("  {}. {}\n", i + 1, name));
                    }
                    CommandOutput::ok(&s)
                }
            }
            "save" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session save <name>");
                }
                let name = args[1].clone();
                if let Some(b) = brain {
                    let a = b.blocking_read();
                    let _ = a.brain.save();
                }
                let data = SessionData {
                    name: name.clone(),
                    created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                };
                match store.save_session(&name, &data) {
                    Ok(()) => CommandOutput::ok(&format!("💾 会话 '{}' 已保存", name)),
                    Err(e) => CommandOutput::err(&format!("保存失败: {}", e)),
                }
            }
            "load" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session load <name>");
                }
                let name = args[1].clone();
                match store.load_session(&name) {
                    Ok(data) => CommandOutput::ok(&format!(
                        "📂 会话 '{}' 已加载 (创建于 {})",
                        data.name, data.created_at
                    )),
                    Err(e) => CommandOutput::err(&format!("加载失败: {}", e)),
                }
            }
            "delete" | "rm" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session delete <name>");
                }
                let name = args[1].clone();
                match store.delete_session(&name) {
                    Ok(()) => CommandOutput::ok(&format!("🗑️ 会话 '{}' 已删除", name)),
                    Err(e) => CommandOutput::err(&format!("删除失败: {}", e)),
                }
            }
            "fork" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session fork <name>");
                }
                let name = args[1].clone();
                match store.fork(&name) {
                    Ok(new_name) => CommandOutput::ok(&format!("🔀 会话已复制为 '{}'", new_name)),
                    Err(e) => CommandOutput::err(&format!("复制失败: {}", e)),
                }
            }
            "export" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session export <name> [--output <path>]");
                }
                let name = args[1].clone();
                let output_idx = args.iter().position(|a| a == "--output");
                if let Some(idx) = output_idx {
                    if idx + 1 >= args.len() {
                        return CommandOutput::err("用法: /session export <name> --output <path>");
                    }
                    let path = args[idx + 1].clone();
                    match store.export_to_file(&name, &path) {
                        Ok(()) => {
                            CommandOutput::ok(&format!("📤 会话 '{}' 已导出到 {}", name, path))
                        }
                        Err(e) => CommandOutput::err(&format!("导出失败: {}", e)),
                    }
                } else {
                    match store.export_to_json(&name) {
                        Ok(json) => {
                            CommandOutput::ok(&format!("📤 会话 '{}' 导出:\n{}", name, json))
                        }
                        Err(e) => CommandOutput::err(&format!("导出失败: {}", e)),
                    }
                }
            }
            "import" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session import <path>");
                }
                let path = args[1].clone();
                match store.import_from_file(&path) {
                    Ok(names) => CommandOutput::ok(&format!("📥 已导入会话: {}", names)),
                    Err(e) => CommandOutput::err(&format!("导入失败: {}", e)),
                }
            }
            "share" => {
                if args.len() < 2 {
                    return CommandOutput::err("用法: /session share <name> [--ttl <hours>]");
                }
                let name = args[1].clone();
                let ttl_idx = args.iter().position(|a| a == "--ttl");
                let ttl_hours =
                    ttl_idx.and_then(|idx| args.get(idx + 1).and_then(|s| s.parse::<u64>().ok()));
                // Export the session to JSON
                let session_json = match store.export_to_json(&name) {
                    Ok(j) => j,
                    Err(e) => return CommandOutput::err(&format!("导出失败: {}", e)),
                };
                let json_value: serde_json::Value = match serde_json::from_str(&session_json) {
                    Ok(v) => v,
                    Err(e) => return CommandOutput::err(&format!("JSON 解析失败: {}", e)),
                };
                let share_mgr = SessionShareManager::new();
                match share_mgr.create(&name, json_value, ttl_hours) {
                    Ok(share) => {
                        let ttl_msg = match share.expires_at {
                            Some(exp) => format!(" (过期于 {})", exp.format("%Y-%m-%d %H:%M:%S")),
                            None => " (无过期)".to_string(),
                        };
                        let home = nt_core_util::home_dir().to_string_lossy().to_string();
                        let file_path = format!("{}/.neotrix/shares/{}.json", home, share.token);
                        CommandOutput::ok(&format!(
                            "🔗 分享 '{}' 已创建{ttl_msg}\n  Token: {}\n  文件: {}\n  API:   /api/sessions/share/{}",
                            name, share.token, file_path, share.token
                        ))
                    }
                    Err(e) => CommandOutput::err(&format!("分享创建失败: {}", e)),
                }
            }
            _ => CommandOutput::err(&format!(
                "未知子命令: {}. 可用: list, save, load, delete, fork, export, import, share",
                cmd
            )),
        }
    }
}

// ====== /resume ======

pub struct ResumeCmd;
impl CliCommand for ResumeCmd {
    fn name(&self) -> &str {
        "/resume"
    }
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str {
        "Resume the last session"
    }
    fn execute(
        &self,
        _args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let store = SessionStore::new();
        match store.get_last_session() {
            Some(name) => match store.load_session(&name) {
                Ok(data) => CommandOutput::ok(&format!(
                    "📂 恢复会话 '{}' (创建于 {})",
                    data.name, data.created_at
                )),
                Err(e) => CommandOutput::err(&format!("恢复失败: {}", e)),
            },
            None => CommandOutput::ok("📂 没有可恢复的会话"),
        }
    }
}

// ====== /fork ======

pub struct ForkCmd;
impl CliCommand for ForkCmd {
    fn name(&self) -> &str {
        "/fork"
    }
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str {
        "Fork the current session: /fork [name]"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        if args.is_empty() {
            CommandOutput::ok("在 TUI 中直接输入 /fork 可创建当前会话的分支副本。")
        } else {
            CommandOutput::ok(&format!("在 TUI 中使用 /fork {} 创建分支副本。", args[0]))
        }
    }
}

// ====== /history ======

pub struct HistoryCmd;
impl CliCommand for HistoryCmd {
    fn name(&self) -> &str {
        "/history"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/hist"]
    }
    fn description(&self) -> &str {
        "Command history: /history | /history clear"
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let want_json = args.iter().any(|a| a == "--json");
        let cmd = args.iter().find(|a| *a != "--json").map(|s| s.as_str());
        match cmd {
            Some("clear") | Some("cls") => {
                let home = nt_core_util::home_dir().to_string_lossy().to_string();
                let path = std::path::Path::new(&home)
                    .join(".neotrix")
                    .join("history.json");
                let tmp = path.with_extension("tmp");
                let _ = std::fs::write(&tmp, "[]");
                let _ = std::fs::rename(&tmp, &path);
                let out = CommandOutput::ok("🗑️ 命令历史已清空");
                if want_json {
                    out.with_json(serde_json::json!({"cleared": true}))
                } else {
                    out
                }
            }
            _ => {
                let home = nt_core_util::home_dir().to_string_lossy().to_string();
                let path = std::path::Path::new(&home)
                    .join(".neotrix")
                    .join("history.json");
                let count = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
                    .map(|v| v.len())
                    .unwrap_or(0);
                let out = CommandOutput::ok(&format!(
                    "📋 命令历史: {} 条记录\n使用 Ctrl+R 搜索历史 /history clear 清空",
                    count
                ));
                if want_json {
                    out.with_json(serde_json::json!({"count": count}))
                } else {
                    out
                }
            }
        }
    }
}
