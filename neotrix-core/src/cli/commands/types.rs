//! Base types — ExitCode / CommandOutput / CliCommand trait / CommandRegistry

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::agent::hooks::{HookContext, HookEvent, HookRegistry};
use crate::cli::sandbox::check_sandbox;
use crate::cli::shield_enforcer::global_shield;
pub(crate) use crate::neotrix::nt_mind::SelfIteratingBrain;

/// Exit code convention (0=clean / 1=warning / 2=notfound / 3=permission / 4=invalid)
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
        Self {
            success: true,
            message: msg.to_string(),
            exit_code: ExitCode::Success,
            json: None,
        }
    }

    pub fn warn(msg: &str) -> Self {
        Self {
            success: true,
            message: msg.to_string(),
            exit_code: ExitCode::Warning,
            json: None,
        }
    }

    pub fn err(msg: &str) -> Self {
        Self {
            success: false,
            message: msg.to_string(),
            exit_code: ExitCode::InvalidInput,
            json: None,
        }
    }

    pub fn not_found(msg: &str) -> Self {
        Self {
            success: false,
            message: msg.to_string(),
            exit_code: ExitCode::NotFound,
            json: None,
        }
    }

    pub fn with_json(mut self, value: serde_json::Value) -> Self {
        self.json = Some(value);
        self
    }
}

pub trait CliCommand {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str> {
        vec![]
    }
    fn description(&self) -> &str;
    fn help_detail(&self) -> Option<String> {
        None
    }
    fn execute(
        &self,
        args: &[String],
        _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput;
}

pub struct CommandRegistry {
    commands: Vec<Box<dyn CliCommand>>,
    hooks: Option<HookRegistry>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            hooks: None,
        }
    }

    /// Attach a HookRegistry for PreToolUse/PostToolUse hook calls
    pub fn with_hooks(mut self, hooks: HookRegistry) -> Self {
        self.hooks = Some(hooks);
        self
    }

    pub fn set_hooks(&mut self, hooks: HookRegistry) {
        self.hooks = Some(hooks);
    }

    pub fn register(&mut self, cmd: Box<dyn CliCommand>) {
        self.commands.push(cmd);
    }

    pub fn get(&self, name: &str) -> Option<&dyn CliCommand> {
        self.commands
            .iter()
            .find(|cmd| cmd.name() == name)
            .map(|b| b.as_ref())
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

    pub fn execute(
        &self,
        input: &str,
        brain: Option<&Arc<RwLock<SelfIteratingBrain>>>,
    ) -> CommandOutput {
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
        let args: Vec<String> = parts
            .get(1)
            .map(|s| s.split(' ').map(String::from).collect())
            .unwrap_or_default();
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
                if let Some(block_reason) = HookRegistry::check_blocked(&actions) {
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

            result
        } else {
            CommandOutput::err(&format!("Unknown command: {}", parts[0]))
        }
    }
}

/// Git subcommands that modify state.
const DESTRUCTIVE_GIT_SUBCMDS: &[&str] = &[
    "add", "commit", "push", "pull", "merge", "rebase", "branch", "checkout", "switch", "restore",
    "reset", "rm", "mv", "tag", "worktree", "gc", "prune",
];

/// Returns true if a command name corresponds to a write/modify operation.
/// Single source of truth — no more separate hardcoded lists.
fn is_write_command(name: &str) -> bool {
    matches!(
        name,
        "/write"
            | "/create"
            | "/edit"
            | "/patch"
            | "/commit"
            | "/pr"
            | "/approve"
            | "/swap"
            | "/transfer"
            | "/features"
            | "/wallet"
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
        Err(decision) => match decision {
            crate::cli::ShieldDecision::Block(msg) => Some(CommandOutput::err(&format!(
                "{} (blocked by nt_shield)",
                msg
            ))),
            _ => None,
        },
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
