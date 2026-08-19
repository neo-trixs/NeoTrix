// ── Lifecycle Hooks (from Kimi Code: pre/post tool gates) ──

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum HookDecision {
    Allow,
    Deny(String),
    RequireConfirm(String),
}

#[derive(Debug, Clone)]
pub struct ToolCallContext {
    pub tool_name: String,
    pub args: String,
    pub cwd: String,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone)]
pub struct HookResult {
    pub decision: HookDecision,
    pub duration_ms: u64,
}

pub type PreToolHook = Arc<dyn Fn(ToolCallContext) -> HookResult + Send + Sync>;
pub type PostToolHook = Arc<dyn Fn(ToolCallContext, String, u64) + Send + Sync>;

pub struct LifecycleHookRegistry {
    pub pre_hooks: Vec<(String, PreToolHook)>,
    pub post_hooks: Vec<(String, PostToolHook)>,
}

impl Default for LifecycleHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleHookRegistry {
    pub fn new() -> Self {
        Self {
            pre_hooks: Vec::new(),
            post_hooks: Vec::new(),
        }
    }

    pub fn register_pre(&mut self, name: &str, hook: PreToolHook) {
        self.pre_hooks.push((name.to_string(), hook));
    }

    pub fn register_post(&mut self, name: &str, hook: PostToolHook) {
        self.post_hooks.push((name.to_string(), hook));
    }

    pub fn run_pre(&self, ctx: ToolCallContext) -> HookDecision {
        for (_, hook) in &self.pre_hooks {
            let result = hook(ctx.clone());
            match result.decision {
                HookDecision::Deny(reason) => return HookDecision::Deny(reason),
                HookDecision::RequireConfirm(msg) => {
                    eprint!("[hook] {} — Allow? (y/N): ", msg);
                    let mut buf = String::new();
                    let _ = std::io::stdin().read_line(&mut buf);
                    match buf.trim().to_lowercase().as_str() {
                        "y" | "yes" => continue,
                        _ => return HookDecision::Deny("User denied at confirmation hook".into()),
                    }
                }
                HookDecision::Allow => continue,
            }
        }
        HookDecision::Allow
    }

    pub fn run_post(&self, ctx: ToolCallContext, output: String, duration_ms: u64) {
        for (_, hook) in &self.post_hooks {
            hook(ctx.clone(), output.clone(), duration_ms);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_hooks() {
        let mut registry = LifecycleHookRegistry::new();
        let deny_hook: PreToolHook = Arc::new(|ctx| HookResult {
            decision: if ctx.tool_name == "dangerous" {
                HookDecision::Deny("blocked".into())
            } else {
                HookDecision::Allow
            },
            duration_ms: 0,
        });
        registry.register_pre("deny_checker", deny_hook);
        let ctx_safe = ToolCallContext {
            tool_name: "read".into(),
            args: String::new(),
            cwd: "/".into(),
            estimated_cost: 0.0,
        };
        let ctx_danger = ToolCallContext {
            tool_name: "dangerous".into(),
            args: String::new(),
            cwd: "/".into(),
            estimated_cost: 0.0,
        };
        match registry.run_pre(ctx_safe) {
            HookDecision::Allow => {}
            _ => panic!("safe tool should be allowed"),
        }
        match registry.run_pre(ctx_danger) {
            HookDecision::Deny(_) => {}
            _ => panic!("dangerous tool should be denied"),
        }
    }
}