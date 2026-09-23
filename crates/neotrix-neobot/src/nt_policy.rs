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
    /// `computer_act` 动作名 (如 `navigate`); 必须命中 `computer_allow`.
    pub computer_action: Option<String>,
    /// `computer_act` 目标 (navigate 时校验 host).
    pub computer_target: Option<String>,
    /// computer 动作 allowlist (空 = 全拒).
    pub computer_allow: Vec<String>,
    /// navigate host allowlist (空 = 全拒).
    pub computer_hosts: Vec<String>,
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
    // 2) computer 受控动作: 动作必须进 allowlist, navigate 再验 host.
    //    默认双空 = 全拒 (fail-closed, 替代旧 blanket-deny 的可配版本).
    if ctx.tool == ToolName::ComputerAct {
        let action = ctx.computer_action.as_deref().unwrap_or("");
        if !ctx.computer_allow.iter().any(|allowed| allowed == action) {
            return deny("computer-allow", "action not in computer_allow");
        }
        if action == "navigate" {
            let target = ctx.computer_target.as_deref().unwrap_or("");
            let host = crate::nt_computer::host_of(target).unwrap_or_default();
            if !ctx.computer_hosts.iter().any(|allowed| allowed == &host) {
                return deny("computer-host", "navigate host not in computer_hosts");
            }
        }
        return PolicyDecision::Allow;
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
    // 5) 纯协议工具默认放行; 未知工具永拒 (fail-closed, 原名进审计).
    match &ctx.tool {
        ToolName::SetTurnStatus | ToolName::ReadFile | ToolName::WriteFile | ToolName::EditFile => {
            PolicyDecision::Allow
        }
        ToolName::Unknown(raw) => deny("unknown-tool", &format!("unknown tool '{raw}'")),
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
            computer_action: None,
            computer_target: None,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
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
            let mut context = ctx(ToolName::ReadFile);
            context.file_path = Some(bad.to_owned());
            assert!(matches!(
                evaluate_policy(&context),
                PolicyDecision::Deny { .. }
            ));
        }
        let mut good = ctx(ToolName::ReadFile);
        good.file_path = Some("notes/todo.md".to_owned());
        assert_eq!(evaluate_policy(&good), PolicyDecision::Allow);
    }

    #[test]
    fn computer_act_default_deny() {
        assert!(matches!(
            evaluate_policy(&ctx(ToolName::ComputerAct)),
            PolicyDecision::Deny { .. }
        ));
    }

    #[test]
    fn computer_allowlist_and_host_gate() {
        let mut context = ctx(ToolName::ComputerAct);
        context.computer_action = Some("navigate".to_owned());
        context.computer_target = Some("https://example.com/a".to_owned());
        context.computer_allow = vec!["navigate".to_owned()];
        // host 未放行 → 拒.
        assert!(matches!(
            evaluate_policy(&context),
            PolicyDecision::Deny { .. }
        ));
        context.computer_hosts = vec!["example.com".to_owned()];
        assert_eq!(evaluate_policy(&context), PolicyDecision::Allow);
        // 动作未放行 → 拒.
        context.computer_action = Some("click".to_owned());
        assert!(matches!(
            evaluate_policy(&context),
            PolicyDecision::Deny { .. }
        ));
    }
}
