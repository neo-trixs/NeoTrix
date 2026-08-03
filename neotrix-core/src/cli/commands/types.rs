//! 基类型 — ExitCode / CommandOutput / CliCommand trait / CommandRegistry

use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::agent::hooks::{EccHookRegistry, HookEvent, HookContext};
use crate::cli::sandbox::check_sandbox;
use crate::cli::shield_enforcer::global_shield;
use crate::neotrix::nt_memory_kb::KnowledgeBase;

/// 退出码约定（参考 witr: 0=clean / 1=warning / 2=notfound / 3=permission / 4=invalid）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Warning = 1,
    NotFound = 2,
    PermissionDenied = 3,
    InvalidInput = 4,
    InternalError = 5,
}

impl ExitCode {
    pub fn to_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub success: bool,
    pub message: String,
    pub exit_code: ExitCode,
    pub json: Option<serde_json::Value>,
}

impl CommandOutput {
    pub fn ok(msg: &str) -> Self {
        Self { success: true, message: msg.to_string(), exit_code: ExitCode::Success, json: None }
    }

    pub fn warn(msg: &str) -> Self {
        Self { success: true, message: msg.to_string(), exit_code: ExitCode::Warning, json: None }
    }

    pub fn err(msg: &str) -> Self {
        Self { success: false, message: msg.to_string(), exit_code: ExitCode::InvalidInput, json: None }
    }

    pub fn not_found(msg: &str) -> Self {
        Self { success: false, message: msg.to_string(), exit_code: ExitCode::NotFound, json: None }
    }

    pub fn with_json(mut self, value: serde_json::Value) -> Self {
        self.json = Some(value);
        self
    }
}

/// 命令分类 — 用于智能帮助分组
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum CommandCategory {
    System,      // 系统核心: /e8, /trace, /brain, /help, /status
    File,        // 文件操作: /read, /write, /edit, /glob, /grep
    Git,         // 版本控制: /git, /diff, /log
    Session,     // 会话: /session-recovery
    Brain,       // 推理进化: /reason, /learn, /plan, /pipe, /seal, /agent, /avatar
    Agent,       // 子代理/MCP
    Crypto,      // 加密金融
    Goal,        // 目标规划
    Explore,     // 探索摄入: /explore, /absorb
    Memory,      // 知识记忆: /kb, /memory
    Automation,  // 自动化: automation, cleanup
    Ui,          // 界面: background, side, router, vim, workspace, theme
    Provider,    // 提供者/模型: provider, model
    Sandbox,     // 沙箱: sandbox
    Connector,   // 连接器: connector
    Other,       // 其他
}

impl CommandCategory {
    pub fn label(&self) -> &str {
        match self {
            CommandCategory::System => "系统核心",
            CommandCategory::File => "文件操作",
            CommandCategory::Git => "版本控制",
            CommandCategory::Session => "会话管理",
            CommandCategory::Brain => "推理进化",
            CommandCategory::Agent => "子代理/MCP",
            CommandCategory::Crypto => "加密金融",
            CommandCategory::Goal => "目标规划",
            CommandCategory::Explore => "探索摄入",
            CommandCategory::Memory => "知识记忆",
            CommandCategory::Automation => "自动化维护",
            CommandCategory::Ui => "界面布局",
            CommandCategory::Provider => "提供者模型",
            CommandCategory::Sandbox => "沙箱安全",
            CommandCategory::Connector => "连接器",
            CommandCategory::Other => "其他",
        }
    }
}

pub trait CliCommand {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str;
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput;
}

/// 名称 → 分类映射（无需每个命令单独实现 category()）
pub fn category_for(name: &str) -> CommandCategory {
    match name.trim_start_matches('/') {
        "help" | "stats" | "exit" | "clear" | "version" | "completions" | "doctor" | "config" | "bench"
            => CommandCategory::System,
        "read" | "write" | "create" | "edit" | "patch" | "diff" | "file"
            => CommandCategory::File,
        "git" | "commit" | "pr"
            => CommandCategory::Git,
        "session" | "resume" | "fork" | "history" | "context" | "compact" | "session-all" | "sess"
            => CommandCategory::Session,
        "e8"
            => CommandCategory::System,
        "agent" | "agents" | "agent-all" | "agents-all" | "discover" | "mcp"
            => CommandCategory::Agent,
        "wallet" | "swap" | "approve" | "transfer" | "cost" | "budget" | "crypto" | "finance"
            => CommandCategory::Crypto,
        "goal" | "plan" | "schedule"
            => CommandCategory::Goal,
        "evidence" | "hypothesis" | "search" | "board" | "kb" | "knowledge" | "knowledge-base"
            => CommandCategory::Memory,
        "background" | "side" | "router" | "vim" | "workspace" | "theme" | "layout" | "display"
            => CommandCategory::Ui,
        "provider" | "model"
            => CommandCategory::Provider,
        "sandbox"
            => CommandCategory::Sandbox,
        "connector"
            => CommandCategory::Connector,
        "approval" | "review" | "plugin" | "profile" | "session-recovery" | "recover" | "snap" | "vc" | "vcs"
            => CommandCategory::Other,
        _ => CommandCategory::Other,
    }
}

pub struct CommandRegistry {
    commands: Vec<Box<dyn CliCommand>>,
    hooks: Option<EccHookRegistry>,
    kb: Option<KnowledgeBase>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self { commands: Vec::new(), hooks: None, kb: None }
    }

    /// Attach a EccHookRegistry for PreToolUse/PostToolUse hook calls
    pub fn with_hooks(mut self, hooks: EccHookRegistry) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn set_hooks(&mut self, hooks: EccHookRegistry) {
        self.hooks = Some(hooks);
    }

    /// Enable session logging to KB
    pub fn with_session_logging(mut self) -> Self {
        self.kb = KnowledgeBase::open(None).ok();
        self
    }

    pub fn enable_session_logging(&mut self) {
        self.kb = KnowledgeBase::open(None).ok();
    }

    pub fn register(&mut self, cmd: Box<dyn CliCommand>) {
        self.commands.push(cmd);
    }

    pub fn get(&self, name: &str) -> Option<&dyn CliCommand> {
        self.commands.iter().find(|cmd| cmd.name() == name).map(|b| b.as_ref())
    }

    pub fn find(&self, name: &str) -> Option<&dyn CliCommand> {
        self.commands
            .iter()
            .find(|cmd| cmd.name() == name || cmd.aliases().contains(&name))
            .map(|b| b.as_ref())
    }

    pub fn list(&self) -> Vec<&str> {
        self.commands.iter().map(|cmd| cmd.name()).collect()
    }

    pub fn complete(&self, prefix: &str) -> Vec<String> {
        self.commands
            .iter()
            .map(|cmd| cmd.name().to_string())
            .filter(|n| n.starts_with(prefix))
            .collect()
    }

    /// 按分类列出命令
    pub fn list_by_category(&self) -> std::collections::BTreeMap<CommandCategory, Vec<&str>> {
        let mut map: std::collections::BTreeMap<CommandCategory, Vec<&str>> = std::collections::BTreeMap::new();
        for cmd in &self.commands {
            let cat = category_for(cmd.name());
            map.entry(cat).or_default().push(cmd.name());
        }
        map
    }

    /// 生成分类帮助文本
    pub fn help_by_category(&self) -> String {
        let by_cat = self.list_by_category();
        let mut out = String::from("━━ NeoTrix 智能命令分类 ━━\n\n");
        for (cat, cmds) in &by_cat {
            out.push_str(&format!("▸ {} ({}):\n", cat.label(), cmds.len()));
            for name in cmds {
                if let Some(cmd) = self.get(name) {
                    let aliases = cmd.aliases();
                    let alias_str = if aliases.is_empty() {
                        String::new()
                    } else {
                        format!(" [{}]", aliases.join(", "))
                    };
                    let desc = cmd.description();
                    let first_line = desc.lines().next().unwrap_or(desc);
                    out.push_str(&format!("  {:<20}{} {}\n", name, alias_str, first_line));
                }
            }
            out.push('\n');
        }
        out.push_str("使用 /help <命令名> 查看详细帮助, /help all 查看全部\n");
        out
    }

    pub fn execute(&self, input: &str, brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
        let args: Vec<String> = parts.get(1).map(|s| s.split(' ').map(String::from).collect()).unwrap_or_default();
        if let Some(cmd) = self.find(parts[0]) {
            // Sandbox check: block write commands in read-only mode
            if let Some(blocked) = check_sandbox_for_command(cmd.name(), &args) {
                return blocked;
            }
            // ShieldEnforcer check: unified policy + guardrails + laws
            if let Some(blocked) = check_shield_for_command(cmd.name(), &args) {
                return blocked;
            }
            // PreToolUse hook
            if let Some(ref hooks) = self.hooks {
                let pre_ctx = HookContext {
                    event: HookEvent::PreToolUse,
                    tool_name: Some(cmd.name().to_string()),
                    tool_input: Some(input.to_string()),
                    tool_output: None,
                    file_path: None,
                    session_id: None,
                    timestamp: std::time::Instant::now(),
                };
                let actions = hooks.execute_event(&pre_ctx);
                if let Some(block_reason) = EccHookRegistry::check_blocked(&actions) {
                    return CommandOutput::err(&format!("Hook blocked: {}", block_reason));
                }
            }

            let result = cmd.execute(&args, brain);

            // PostToolUse hook
            if let Some(ref hooks) = self.hooks {
                let post_ctx = HookContext {
                    event: HookEvent::PostToolUse,
                    tool_name: Some(cmd.name().to_string()),
                    tool_input: Some(input.to_string()),
                    tool_output: Some(result.message.clone()),
                    file_path: None,
                    session_id: None,
                    timestamp: std::time::Instant::now(),
                };
                let _ = hooks.execute_event(&post_ctx);
            }

            // Session log to KB
            if let Some(ref kb) = self.kb {
                let _ = kb.session_log_append(
                    "cli::session",
                    &format!("> {}\n{}", input, result.message),
                    "command",
                    None,
                );
            }

            result
        } else {
            CommandOutput::err(&format!("Unknown command: {}", parts[0]))
        }
    }
}

/// Git subcommands that modify state.
const DESTRUCTIVE_GIT_SUBCMDS: &[&str] = &[
    "add", "commit", "push", "pull", "merge", "rebase",
    "branch", "checkout", "switch", "restore", "reset",
    "rm", "mv", "tag", "worktree", "gc", "prune",
];

/// Returns true if a command name corresponds to a write/modify operation.
/// Single source of truth — no more separate hardcoded lists.
fn is_write_command(name: &str) -> bool {
    matches!(name,
        "/write" | "/create" | "/edit" | "/patch"
        | "/commit" | "/pr"
        | "/approve" | "/swap" | "/transfer"
        | "/features"
        | "/file" | "/git" | "/wallet" | "/session-recovery"
    )
}

fn check_sandbox_for_command(name: &str, args: &[String]) -> Option<CommandOutput> {
    if name == "/git" {
        if let Some(sub) = args.first() {
            if DESTRUCTIVE_GIT_SUBCMDS.contains(&sub.as_str()) {
                return check_sandbox();
            }
        }
        return None;
    }
    if is_write_command(name) {
        return check_sandbox();
    }
    None
}

fn check_shield_for_command(name: &str, _args: &[String]) -> Option<CommandOutput> {
    let shield = global_shield();
    let s = shield.lock().unwrap_or_else(|e| e.into_inner());
    let action = name.trim_start_matches('/');
    let result = s.check_cli_command(action, action);
    match result {
        Ok(()) => None,
        Err(decision) => {
            match decision {
                crate::cli::ShieldDecision::Block(msg) =>
                    Some(CommandOutput::err(&format!("{} (blocked by nt_shield)", msg))),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success.to_i32(), 0);
        assert_eq!(ExitCode::Warning.to_i32(), 1);
        assert_eq!(ExitCode::NotFound.to_i32(), 2);
        assert_eq!(ExitCode::PermissionDenied.to_i32(), 3);
        assert_eq!(ExitCode::InvalidInput.to_i32(), 4);
        assert_eq!(ExitCode::InternalError.to_i32(), 5);
    }

    #[test]
    fn test_exit_code_equality() {
        assert_eq!(ExitCode::Success, ExitCode::Success);
        assert_ne!(ExitCode::Success, ExitCode::Warning);
        assert_ne!(ExitCode::NotFound, ExitCode::InvalidInput);
    }

    #[test]
    fn test_command_output_ok() {
        let out = CommandOutput::ok("all good");
        assert!(out.success);
        assert_eq!(out.message, "all good");
        assert_eq!(out.exit_code, ExitCode::Success);
        assert!(out.json.is_none());
    }

    #[test]
    fn test_command_output_err() {
        let out = CommandOutput::err("error occurred");
        assert!(!out.success);
        assert_eq!(out.message, "error occurred");
        assert_eq!(out.exit_code, ExitCode::InvalidInput);
        assert!(out.json.is_none());
    }

    #[test]
    fn test_command_output_warn() {
        let out = CommandOutput::warn("caution");
        assert!(out.success);
        assert_eq!(out.message, "caution");
    }

    #[test]
    fn test_command_output_not_found() {
        let out = CommandOutput::not_found("not found");
        assert!(!out.success);
        assert_eq!(out.message, "not found");
        assert_eq!(out.exit_code, ExitCode::NotFound);
    }

    #[test]
    fn test_command_output_with_json() {
        let json_val = serde_json::json!({"key": "value", "count": 42});
        let out = CommandOutput::ok("json output").with_json(json_val.clone());
        assert_eq!(out.json, Some(json_val));
        assert!(out.success);
    }

    #[test]
    fn test_command_registry_empty() {
        let reg = CommandRegistry::new();
        assert!(reg.list().is_empty());
        assert!(reg.get("/help").is_none());
        assert!(reg.find("/help").is_none());
    }
}
