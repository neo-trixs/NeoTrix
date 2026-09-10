
use std::collections::HashMap;

pub type LlmPromptFn = Box<dyn Fn(&str, &str) -> String + Send + Sync>;

pub struct LlmHooks {
    enabled: bool,
    hooks: HashMap<String, LlmPromptFn>,
    call_count: u64,
    token_budget: u64,
    tokens_used: u64,
}

impl LlmHooks {
    pub fn new(token_budget: u64) -> Self {
        Self {
            enabled: false,
            hooks: HashMap::new(),
            call_count: 0,
            token_budget,
            tokens_used: 0,
        }
    }

    pub fn enable(&mut self) { self.enabled = true; }

    pub fn register_hook(&mut self, name: String, prompt_fn: LlmPromptFn) {
        self.hooks.insert(name, prompt_fn);
    }

    pub fn invoke(&mut self, hook_name: &str, context: &str, system_prompt: &str) -> Option<String> {
        if !self.enabled { return None; }
        if self.tokens_used >= self.token_budget { return None; }

        let hook = self.hooks.get(hook_name)?;
        let prompt = hook(context, system_prompt);

        // Estimate tokens (rough: 1 token per 4 chars)
        let estimated = (prompt.len() / 4) as u64;
        if self.tokens_used + estimated > self.token_budget { return None; }

        self.tokens_used += estimated;
        self.call_count += 1;

        // In production, this would call an actual LLM API
        // For simulation, return a synthetic response
        Some(format!("LLM response for '{}' (simulated)", hook_name))
    }

    pub fn call_count(&self) -> u64 { self.call_count }
    pub fn tokens_used(&self) -> u64 { self.tokens_used }
    pub fn token_budget(&self) -> u64 { self.token_budget }
    pub fn remaining_budget(&self) -> u64 { self.token_budget.saturating_sub(self.tokens_used) }
    pub fn is_enabled(&self) -> bool { self.enabled }
}

/// Default hooks for common LLM tasks
pub fn default_hooks() -> HashMap<String, String> {
    let mut hooks = HashMap::new();
    hooks.insert("reflect".into(), "You are a reflective agent. Analyze your recent actions and suggest improvements.".into());
    hooks.insert("plan".into(), "You are a planning agent. Create a step-by-step plan to achieve the goal.".into());
    hooks.insert("social".into(), "You are a social agent. Consider the perspectives and needs of others.".into());
    hooks.insert("explore".into(), "You are an explorer. Identify interesting things to investigate.".into());
    hooks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_returns_none() {
        let mut hooks = LlmHooks::new(1000);
        let result = hooks.invoke("reflect", "context", "sys");
        assert!(result.is_none());
    }

    #[test]
    fn enabled_invokes() {
        let mut hooks = LlmHooks::new(10000);
        hooks.enable();
        hooks.register_hook("test".into(), Box::new(|ctx, _| format!("resp: {}", ctx)));
        let result = hooks.invoke("test", "hello", "sys");
        assert!(result.is_some());
        assert_eq!(hooks.call_count(), 1);
    }

    #[test]
    fn budget_enforced() {
        let mut hooks = LlmHooks::new(20);
        hooks.enable();
        hooks.register_hook("test".into(), Box::new(|_, _| "x".repeat(100)));
        let _ = hooks.invoke("test", "", "");
        let result = hooks.invoke("test", "", "");
        assert!(result.is_none()); // budget exhausted
    }

    #[test]
    fn unknown_hook_returns_none() {
        let mut hooks = LlmHooks::new(1000);
        hooks.enable();
        let result = hooks.invoke("nonexistent", "", "");
        assert!(result.is_none());
    }

    #[test]
    fn default_hooks_has_common() {
        let h = default_hooks();
        assert!(h.contains_key("reflect"));
        assert!(h.contains_key("plan"));
    }
}
