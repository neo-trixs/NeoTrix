use std::fmt;

/// Hook execution point — when in the tool lifecycle the hook fires.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPoint {
    PreToolUse,
    PostToolUse,
}

impl fmt::Display for HookPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookPoint::PreToolUse => write!(f, "PreToolUse"),
            HookPoint::PostToolUse => write!(f, "PostToolUse"),
        }
    }
}

/// Action returned by a hook after execution.
#[derive(Debug, Clone)]
pub enum HookAction {
    /// Allow execution to proceed unchanged.
    Allow,
    /// Block execution with a reason string.
    Deny(String),
    /// Modify the input (pre) or output (post).
    Modify(String),
}

/// Trait for tool lifecycle hooks.
///
/// Each hook declares a single `hook_point` and implements the corresponding
/// callback. The unused callback has a default no-op implementation.
pub trait ToolHook: Send + Sync {
    /// Which hook point this hook attaches to.
    fn hook_point(&self) -> HookPoint;

    /// Unique name for identification / debugging.
    fn name(&self) -> &str;

    /// Called before tool execution (only when `hook_point == PreToolUse`).
    fn on_pre_tool_use(&self, _tool_name: &str, _input: &str) -> HookAction {
        HookAction::Allow
    }

    /// Called after tool execution (only when `hook_point == PostToolUse`).
    fn on_post_tool_use(&self, _tool_name: &str, _output: &str) -> HookAction {
        HookAction::Allow
    }
}

/// Registry that manages and executes all registered tool hooks.
pub struct ToolHookRegistry {
    hooks: Vec<Box<dyn ToolHook>>,
}

impl ToolHookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Register a new hook.
    pub fn register(&mut self, hook: Box<dyn ToolHook>) {
        self.hooks.push(hook);
    }

    /// Run all pre-tool-use hooks.
    ///
    /// Returns the *first* `Deny` immediately (fail-fast), or the *last*
    /// `Modify` if any, or `Allow` if all passed.
    pub fn run_pre_hooks(&self, tool_name: &str, input: &str) -> HookAction {
        let mut last_modify: Option<String> = None;
        for hook in &self.hooks {
            if hook.hook_point() != HookPoint::PreToolUse {
                continue;
            }
            let current_input = last_modify.as_deref().unwrap_or(input);
            match hook.on_pre_tool_use(tool_name, current_input) {
                HookAction::Deny(reason) => return HookAction::Deny(reason),
                HookAction::Modify(modified) => last_modify = Some(modified),
                HookAction::Allow => {}
            }
        }
        last_modify
            .map(HookAction::Modify)
            .unwrap_or(HookAction::Allow)
    }

    /// Run all post-tool-use hooks.
    ///
    /// Same aggregation semantics as `run_pre_hooks`.
    pub fn run_post_hooks(&self, tool_name: &str, output: &str) -> HookAction {
        let mut last_modify: Option<String> = None;
        for hook in &self.hooks {
            if hook.hook_point() != HookPoint::PostToolUse {
                continue;
            }
            let current_output = last_modify.as_deref().unwrap_or(output);
            match hook.on_post_tool_use(tool_name, current_output) {
                HookAction::Deny(reason) => return HookAction::Deny(reason),
                HookAction::Modify(modified) => last_modify = Some(modified),
                HookAction::Allow => {}
            }
        }
        last_modify
            .map(HookAction::Modify)
            .unwrap_or(HookAction::Allow)
    }

    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    pub fn list_hooks(&self) -> Vec<String> {
        self.hooks.iter().map(|h| h.name().to_string()).collect()
    }
}

impl Default for ToolHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Security allowlist hook — denies any tool whose name is not in the
/// pre-configured allowlist. Only fires on `PreToolUse`.
pub struct SecurityAllowlistHook {
    name: String,
    allowed_tools: Vec<String>,
}

impl SecurityAllowlistHook {
    pub fn new(name: &str, allowed: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            allowed_tools: allowed,
        }
    }
}

impl ToolHook for SecurityAllowlistHook {
    fn hook_point(&self) -> HookPoint {
        HookPoint::PreToolUse
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn on_pre_tool_use(&self, tool_name: &str, _input: &str) -> HookAction {
        if self.allowed_tools.iter().any(|t| t == tool_name) {
            HookAction::Allow
        } else {
            HookAction::Deny(format!(
                "Tool '{}' is not in the allowlist",
                tool_name
            ))
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helper hooks ---------------------------------------------------

    struct AllowHook;

    impl ToolHook for AllowHook {
        fn hook_point(&self) -> HookPoint {
            HookPoint::PreToolUse
        }
        fn name(&self) -> &str {
            "allow-hook"
        }
        fn on_pre_tool_use(&self, _tool_name: &str, _input: &str) -> HookAction {
            HookAction::Allow
        }
    }

    struct DenyHook;

    impl ToolHook for DenyHook {
        fn hook_point(&self) -> HookPoint {
            HookPoint::PreToolUse
        }
        fn name(&self) -> &str {
            "deny-hook"
        }
        fn on_pre_tool_use(&self, tool_name: &str, _input: &str) -> HookAction {
            HookAction::Deny(format!("denied: {}", tool_name))
        }
    }

    struct ModifyInputHook;

    impl ToolHook for ModifyInputHook {
        fn hook_point(&self) -> HookPoint {
            HookPoint::PreToolUse
        }
        fn name(&self) -> &str {
            "modify-input-hook"
        }
        fn on_pre_tool_use(&self, _tool_name: &str, input: &str) -> HookAction {
            HookAction::Modify(format!("modified: {}", input))
        }
    }

    struct PostModifyHook;

    impl ToolHook for PostModifyHook {
        fn hook_point(&self) -> HookPoint {
            HookPoint::PostToolUse
        }
        fn name(&self) -> &str {
            "post-modify-hook"
        }
        fn on_post_tool_use(&self, _tool_name: &str, _output: &str) -> HookAction {
            HookAction::Modify("post-processed".to_string())
        }
    }

    struct PostDenyHook;

    impl ToolHook for PostDenyHook {
        fn hook_point(&self) -> HookPoint {
            HookPoint::PostToolUse
        }
        fn name(&self) -> &str {
            "post-deny-hook"
        }
        fn on_post_tool_use(&self, tool_name: &str, _output: &str) -> HookAction {
            HookAction::Deny(format!("post-deny: {}", tool_name))
        }
    }

    // -- Tests ----------------------------------------------------------

    #[test]
    fn test_hook_registry_register_and_run() {
        let mut reg = ToolHookRegistry::new();
        assert_eq!(reg.hook_count(), 0);

        reg.register(Box::new(AllowHook));
        assert_eq!(reg.hook_count(), 1);

        let action = reg.run_pre_hooks("some_tool", "input");
        assert!(matches!(action, HookAction::Allow));
    }

    #[test]
    fn test_allowlist_hook_allows_known_tool() {
        let hook = SecurityAllowlistHook::new(
            "test-allowlist",
            vec!["safe_tool".to_string(), "reader".to_string()],
        );

        assert!(
            matches!(hook.on_pre_tool_use("safe_tool", ""), HookAction::Allow),
            "Expected Allow for known tool"
        );
    }

    #[test]
    fn test_allowlist_hook_denies_unknown_tool() {
        let hook = SecurityAllowlistHook::new(
            "test-allowlist",
            vec!["safe_tool".to_string()],
        );

        let action = hook.on_pre_tool_use("malicious_tool", "");
        assert!(
            matches!(&action, HookAction::Deny(ref msg) if msg.contains("malicious_tool")),
            "Expected Deny for unknown tool"
        );
    }

    #[test]
    fn test_deny_stops_execution_immediately() {
        let mut reg = ToolHookRegistry::new();
        reg.register(Box::new(AllowHook));
        reg.register(Box::new(DenyHook));
        reg.register(Box::new(ModifyInputHook));

        let action = reg.run_pre_hooks("danger", "payload");
        assert!(matches!(action, HookAction::Deny(_)));
        if let HookAction::Deny(msg) = action {
            assert!(msg.contains("danger"));
        }
    }

    #[test]
    fn test_modify_hook_changes_input() {
        let mut reg = ToolHookRegistry::new();
        reg.register(Box::new(ModifyInputHook));

        let action = reg.run_pre_hooks("echo", "hello");
        assert!(
            matches!(&action, HookAction::Modify(ref s) if s == "modified: hello"),
            "Expected Modify"
        );
    }

    #[test]
    fn test_multiple_hooks_run_in_order() {
        let mut reg = ToolHookRegistry::new();
        reg.register(Box::new(AllowHook));
        reg.register(Box::new(ModifyInputHook));

        let action = reg.run_pre_hooks("tool", "raw");
        assert!(
            matches!(&action, HookAction::Modify(ref s) if s == "modified: raw"),
            "Expected Modify from last hook"
        );
    }

    #[test]
    fn test_post_hook_modify() {
        let mut reg = ToolHookRegistry::new();
        reg.register(Box::new(PostModifyHook));

        let action = reg.run_post_hooks("tool", "raw_output");
        assert!(
            matches!(&action, HookAction::Modify(ref s) if s == "post-processed"),
            "Expected Modify"
        );
    }

    #[test]
    fn test_post_hook_deny() {
        let mut reg = ToolHookRegistry::new();
        reg.register(Box::new(PostDenyHook));

        let action = reg.run_post_hooks("bad_tool", "some output");
        assert!(
            matches!(&action, HookAction::Deny(ref msg) if msg.contains("bad_tool")),
            "Expected Deny"
        );
    }

    #[test]
    fn test_hook_point_display() {
        assert_eq!(HookPoint::PreToolUse.to_string(), "PreToolUse");
        assert_eq!(HookPoint::PostToolUse.to_string(), "PostToolUse");
    }

    #[test]
    fn test_list_hooks() {
        let mut reg = ToolHookRegistry::new();
        reg.register(Box::new(AllowHook));
        reg.register(Box::new(DenyHook));

        let names = reg.list_hooks();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"allow-hook".to_string()));
        assert!(names.contains(&"deny-hook".to_string()));
    }
}
