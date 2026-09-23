//! `nt_policy` — fail-closed 网关策略.
//!
//! 语义移植自 openbot `server/src/computer/{gateway,policy}.ts`:
//! resolve → `evaluate_policy` → 先写 audit → 再执行; deny 优先、
//! 缺省拒绝、损坏规则拒绝; `HumanHasControl` 时拒一切 Bot 动作.
//! `workspace-jail` 越狱拦截移植自 OpenMuse `computer.ts workspacePath`.

use serde::{Deserialize, Serialize};

use crate::nt_types::ToolName;

/// 动作发起方.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Actor {
    Bot,
    Person,
    Routine,
}

/// 策略评估上下文 (openbot `PolicyContext` 本地子集).
#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub tool: ToolName,
    pub actor: Actor,
    pub human_has_control: bool,
    /// 规范化后的相对路径 (如 `notes/todo.md`); `None` 表示非文件工具.
    pub file_path: Option<String>,
    /// `bash` 原始命令 (做越狱启发式检查).
    pub command: Option<String>,
}

/// 策略裁决.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny { rule: String, reason: String },
}

fn deny(rule: &str, reason: &str) -> PolicyDecision {
    PolicyDecision::Deny {
        rule: rule.to_owned(),
        reason: reason.to_owned(),
    }
}

/// fail-closed 评估: 命中任一 deny 即拒; 无显式 allow 即拒.
pub fn evaluate_policy(ctx: &PolicyContext) -> PolicyDecision {
    // 1) 人接管时拒一切 Bot/Routine 动作 (openbot HumanHasControl).
    if ctx.human_has_control && ctx.actor != Actor::Person {
        return deny("human-control", "human has control; bot actions refused");
    }
    // 2) computer 受控动作默认拒 (后续 nt_computer 接入快照+allowlist 后放行).
    if ctx.tool == ToolName::ComputerAct {
        return deny("computer-default-deny", "computer actions need explicit grant");
    }
    // 3) 文件越狱拦截 (OpenMuse workspacePath 语义).
    if let Some(path) = ctx.file_path.as_deref() {
        if is_jailbreak_path(path) {
            return deny("workspace-jail", "path escapes workspace");
        }
        return PolicyDecision::Allow;
    }
    // 4) bash 越狱启发式: 拒绝 `..` / 绝对路径 / 家目录展开.
    if ctx.tool == ToolName::Bash {
        if let Some(cmd) = ctx.command.as_deref() {
            if looks_like_escape(cmd) {
                return deny("workspace-jail", "command escapes workspace");
            }
        }
        return PolicyDecision::Allow;
    }
    // 5) 纯协议工具默认放行.
    match ctx.tool {
        ToolName::SetTurnStatus | ToolName::ReadFile | ToolName::WriteFile | ToolName::EditFile => {
            PolicyDecision::Allow
        }
        ToolName::Bash | ToolName::ComputerAct => {
            deny("default-deny", "no explicit allow rule matched")
        }
    }
}

fn is_jailbreak_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.starts_with('~') {
        return true;
    }
    trimmed.split('/').any(|seg| seg == "..")
}

fn looks_like_escape(cmd: &str) -> bool {
    cmd.contains("..") || cmd.contains("~") || cmd.contains("/etc/") || cmd.contains("/System/")
}

#[cfg(test)]
mod tests {
    use super::{Actor, PolicyContext, PolicyDecision, ToolName, evaluate_policy};

    fn ctx(tool: ToolName) -> PolicyContext {
        PolicyContext {
            tool,
            actor: Actor::Bot,
            human_has_control: false,
            file_path: None,
            command: None,
        }
    }

    #[test]
    fn human_control_denies_bot() {
        let mut context = ctx(ToolName::Bash);
        context.human_has_control = true;
        assert!(matches!(
            evaluate_policy(&context),
            PolicyDecision::Deny { .. }
        ));
    }

    #[test]
    fn jailbreak_paths_denied() {
        for bad in ["../secret", "/etc/passwd", "~/keys", "a/../../b", ""] {
            let context = PolicyContext {
                tool: ToolName::ReadFile,
                actor: Actor::Bot,
                human_has_control: false,
                file_path: Some(bad.to_owned()),
                command: None,
            };
            assert!(matches!(
                evaluate_policy(&context),
                PolicyDecision::Deny { .. }
            ));
        }
        let good = PolicyContext {
            tool: ToolName::ReadFile,
            actor: Actor::Bot,
            human_has_control: false,
            file_path: Some("notes/todo.md".to_owned()),
            command: None,
        };
        assert_eq!(evaluate_policy(&good), PolicyDecision::Allow);
    }

    #[test]
    fn computer_act_default_deny() {
        assert!(matches!(
            evaluate_policy(&ctx(ToolName::ComputerAct)),
            PolicyDecision::Deny { .. }
        ));
    }
}
