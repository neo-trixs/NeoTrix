use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    pub task_id: String,
    pub agent_id: String,
    pub input: String,
    pub tool_calls: Vec<String>,
    pub tool_results: Vec<String>,
    pub errors: Vec<String>,
    pub model_calls: usize,
    pub started_at: Instant,
    pub metadata: HashMap<String, String>,
}

impl MiddlewareContext {
    pub fn new(task_id: &str, agent_id: &str, input: &str) -> Self {
        Self {
            task_id: task_id.into(),
            agent_id: agent_id.into(),
            input: input.into(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            errors: Vec::new(),
            model_calls: 0,
            started_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }
}

#[derive(Debug, Clone)]
pub enum MiddlewareResult {
    Continue(MiddlewareContext),
    Halt(MiddlewareContext, String),
}

pub trait MiddlewareHook: Send + Sync {
    fn name(&self) -> &'static str;
    fn before_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult;
    fn after_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult;
    fn on_error(&self, ctx: &mut MiddlewareContext, error: &str) -> MiddlewareResult;
}

pub struct MessageQueueMiddleware {
    pub queue_check_enabled: bool,
}

impl MiddlewareHook for MessageQueueMiddleware {
    fn name(&self) -> &'static str {
        "message_queue"
    }

    fn before_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        if self.queue_check_enabled {
            ctx.metadata.insert("queue_checked".into(), "true".into());
        }
        MiddlewareResult::Continue(ctx.clone())
    }

    fn after_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        MiddlewareResult::Continue(ctx.clone())
    }

    fn on_error(&self, ctx: &mut MiddlewareContext, error: &str) -> MiddlewareResult {
        ctx.errors.push(format!("queue: {}", error));
        MiddlewareResult::Continue(ctx.clone())
    }
}

pub struct StepLimitMiddleware {
    pub max_model_calls: usize,
}

impl MiddlewareHook for StepLimitMiddleware {
    fn name(&self) -> &'static str {
        "step_limit"
    }

    fn before_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        if ctx.model_calls >= self.max_model_calls {
            return MiddlewareResult::Halt(
                ctx.clone(),
                format!("step limit reached ({})", self.max_model_calls),
            );
        }
        ctx.model_calls += 1;
        MiddlewareResult::Continue(ctx.clone())
    }

    fn after_model(&self, _ctx: &mut MiddlewareContext) -> MiddlewareResult {
        MiddlewareResult::Continue(_ctx.clone())
    }

    fn on_error(&self, ctx: &mut MiddlewareContext, error: &str) -> MiddlewareResult {
        ctx.errors.push(format!("step_limit: {}", error));
        MiddlewareResult::Continue(ctx.clone())
    }
}

pub struct ToolErrorMiddleware;

impl MiddlewareHook for ToolErrorMiddleware {
    fn name(&self) -> &'static str {
        "tool_error"
    }

    fn before_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        MiddlewareResult::Continue(ctx.clone())
    }

    fn after_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        MiddlewareResult::Continue(ctx.clone())
    }

    fn on_error(&self, ctx: &mut MiddlewareContext, error: &str) -> MiddlewareResult {
        ctx.errors.push(format!("tool_error: {}", error));
        MiddlewareResult::Continue(ctx.clone())
    }
}

pub struct MiddlewareChain {
    pub hooks: Vec<Box<dyn MiddlewareHook>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add(&mut self, hook: Box<dyn MiddlewareHook>) {
        self.hooks.push(hook);
    }

    pub fn run_before_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        for hook in &self.hooks {
            match hook.before_model(ctx) {
                MiddlewareResult::Continue(c) => *ctx = c,
                halt @ MiddlewareResult::Halt(..) => return halt,
            }
        }
        MiddlewareResult::Continue(ctx.clone())
    }

    pub fn run_after_model(&self, ctx: &mut MiddlewareContext) -> MiddlewareResult {
        for hook in &self.hooks {
            match hook.after_model(ctx) {
                MiddlewareResult::Continue(c) => *ctx = c,
                halt @ MiddlewareResult::Halt(..) => return halt,
            }
        }
        MiddlewareResult::Continue(ctx.clone())
    }

    pub fn run_on_error(&self, ctx: &mut MiddlewareContext, error: &str) -> MiddlewareResult {
        for hook in &self.hooks {
            match hook.on_error(ctx, error) {
                MiddlewareResult::Continue(c) => *ctx = c,
                halt @ MiddlewareResult::Halt(..) => return halt,
            }
        }
        MiddlewareResult::Continue(ctx.clone())
    }
}

pub fn default_middleware_chain(max_steps: usize) -> MiddlewareChain {
    let mut chain = MiddlewareChain::new();
    chain.add(Box::new(StepLimitMiddleware {
        max_model_calls: max_steps,
    }));
    chain.add(Box::new(MessageQueueMiddleware {
        queue_check_enabled: true,
    }));
    chain.add(Box::new(ToolErrorMiddleware));
    chain
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentConfig {
    pub agent_id: String,
    pub model: String,
    pub system_prompt: String,
    pub max_steps: usize,
    pub sandbox_enabled: bool,
    pub middleware: Vec<String>,
}

impl Default for SubAgentConfig {
    fn default() -> Self {
        Self {
            agent_id: "default".into(),
            model: "gpt-4o".into(),
            system_prompt: "You are a helpful coding agent.".into(),
            max_steps: 25,
            sandbox_enabled: true,
            middleware: vec![
                "step_limit".into(),
                "message_queue".into(),
                "tool_error".into(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubAgentResult {
    pub agent_id: String,
    pub task_id: String,
    pub success: bool,
    pub output: String,
    pub model_calls: usize,
    pub elapsed_ms: u128,
    pub error_count: usize,
    pub tool_calls: Vec<String>,
}

pub struct SubAgentOrchestrator {
    pub agents: HashMap<String, SubAgentConfig>,
    pub middleware: MiddlewareChain,
    pub results: Vec<SubAgentResult>,
}

impl SubAgentOrchestrator {
    pub fn new(middleware: MiddlewareChain) -> Self {
        Self {
            agents: HashMap::new(),
            middleware,
            results: Vec::new(),
        }
    }

    pub fn register_agent(&mut self, config: SubAgentConfig) {
        self.agents.insert(config.agent_id.clone(), config);
    }

    pub fn spawn(&mut self, agent_id: &str, task_id: &str, input: &str) -> SubAgentResult {
        let _config = match self.agents.get(agent_id) {
            Some(c) => c.clone(),
            None => {
                return SubAgentResult {
                    agent_id: agent_id.into(),
                    task_id: task_id.into(),
                    success: false,
                    output: format!("unknown agent: {}", agent_id),
                    model_calls: 0,
                    elapsed_ms: 0,
                    error_count: 1,
                    tool_calls: Vec::new(),
                }
            }
        };

        let mut ctx = MiddlewareContext::new(task_id, agent_id, input);

        match self.middleware.run_before_model(&mut ctx) {
            MiddlewareResult::Continue(c) => ctx = c,
            MiddlewareResult::Halt(c, reason) => {
                return SubAgentResult {
                    agent_id: agent_id.into(),
                    task_id: task_id.into(),
                    success: false,
                    output: reason,
                    model_calls: c.model_calls,
                    elapsed_ms: c.elapsed_ms(),
                    error_count: c.errors.len(),
                    tool_calls: c.tool_calls,
                };
            }
        }

        let output = format!("executed task '{}' with agent '{}'", task_id, agent_id);
        ctx.tool_calls.push("execute".into());
        ctx.tool_results.push("completed".into());

        match self.middleware.run_after_model(&mut ctx) {
            MiddlewareResult::Continue(c) => ctx = c,
            MiddlewareResult::Halt(c, reason) => {
                return SubAgentResult {
                    agent_id: agent_id.into(),
                    task_id: task_id.into(),
                    success: false,
                    output: reason,
                    model_calls: c.model_calls,
                    elapsed_ms: c.elapsed_ms(),
                    error_count: c.errors.len(),
                    tool_calls: c.tool_calls,
                };
            }
        }

        let result = SubAgentResult {
            agent_id: agent_id.into(),
            task_id: task_id.into(),
            success: true,
            output,
            model_calls: ctx.model_calls,
            elapsed_ms: ctx.elapsed_ms(),
            error_count: ctx.errors.len(),
            tool_calls: ctx.tool_calls,
        };
        self.results.push(result.clone());
        result
    }

    pub fn spawn_parallel(&mut self, tasks: Vec<(&str, &str, &str)>) -> Vec<SubAgentResult> {
        tasks
            .into_iter()
            .map(|(agent, task, input)| self.spawn(agent, task, input))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_chain_default() {
        let chain = default_middleware_chain(25);
        assert_eq!(chain.hooks.len(), 3);
    }

    #[test]
    fn test_step_limit_halt() {
        let chain = default_middleware_chain(2);
        let mut ctx = MiddlewareContext::new("t1", "a1", "test");
        let r1 = chain.run_before_model(&mut ctx);
        assert!(matches!(r1, MiddlewareResult::Continue(_)));
        let r2 = chain.run_before_model(&mut ctx);
        assert!(matches!(r2, MiddlewareResult::Continue(_)));
        let r3 = chain.run_before_model(&mut ctx);
        assert!(matches!(r3, MiddlewareResult::Halt(..)));
    }

    #[test]
    fn test_subagent_orchestrator() {
        let mut orch = SubAgentOrchestrator::new(default_middleware_chain(25));
        orch.register_agent(SubAgentConfig {
            agent_id: "coder".into(),
            model: "gpt-4o".into(),
            ..Default::default()
        });
        let result = orch.spawn("coder", "task-1", "fix bug in main.rs");
        assert!(result.success);
        assert_eq!(result.agent_id, "coder");
    }

    #[test]
    fn test_unknown_agent() {
        let mut orch = SubAgentOrchestrator::new(default_middleware_chain(25));
        let result = orch.spawn("unknown", "t1", "input");
        assert!(!result.success);
        assert!(result.output.contains("unknown agent"));
    }

    #[test]
    fn test_parallel_spawn() {
        let mut orch = SubAgentOrchestrator::new(default_middleware_chain(25));
        orch.register_agent(SubAgentConfig::default());
        let results = orch.spawn_parallel(vec![
            ("default", "t1", "task one"),
            ("default", "t2", "task two"),
        ]);
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
        assert_eq!(orch.results.len(), 2);
    }

    #[test]
    fn test_middleware_context() {
        let ctx = MiddlewareContext::new("t1", "a1", "hello");
        assert_eq!(ctx.task_id, "t1");
        assert!(ctx.elapsed_ms() < 100);
    }

    #[test]
    fn test_tool_error_middleware() {
        let hook = ToolErrorMiddleware;
        let mut ctx = MiddlewareContext::new("t1", "a1", "test");
        let result = hook.on_error(&mut ctx, "connection refused");
        assert!(matches!(result, MiddlewareResult::Continue(_)));
        assert_eq!(ctx.errors.len(), 1);
        assert!(ctx.errors[0].contains("connection refused"));
    }
}

#[cfg(test)]
mod middleware_tests {
    use super::*;

    #[test]
    fn test_message_queue_middleware_disabled() {
        let hook = MessageQueueMiddleware {
            queue_check_enabled: false,
        };
        let mut ctx = MiddlewareContext::new("t1", "a1", "test");
        let result = hook.before_model(&mut ctx);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
        assert_eq!(ctx.metadata.get("queue_checked"), None);
    }

    #[test]
    fn test_chain_error_propagation() {
        let chain = default_middleware_chain(25);
        let mut ctx = MiddlewareContext::new("t1", "a1", "test");
        let result = chain.run_on_error(&mut ctx, "api failure");
        assert!(matches!(result, MiddlewareResult::Continue(_)));
        assert_eq!(ctx.errors.len(), 3);
    }
}
