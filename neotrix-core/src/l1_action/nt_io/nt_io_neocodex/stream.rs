// ── Generator-based Agent Loop (Claude Code-inspired: yield events) ──

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;

use super::agent::NeoCodexAgent;
use super::cost::CostTracker;
use super::hooks::{HookDecision, LifecycleHookRegistry, ToolCallContext};
use super::permissions::PermissionSystem;
use super::provider::NeoCodexMode;

#[derive(Debug, Clone)]
pub enum AgentEvent {
    TurnStart {
        turn: u64,
        mode: NeoCodexMode,
    },
    Thinking {
        content: String,
    },
    ToolCallStart {
        name: String,
        args: String,
    },
    ToolCallEnd {
        name: String,
        result: String,
        duration_ms: u64,
        success: bool,
    },
    Chunk {
        text: String,
    },
    TurnEnd {
        response: String,
    },
    Error {
        message: String,
    },
    ModeSwitch {
        from: NeoCodexMode,
        to: NeoCodexMode,
    },
    BudgetWarning {
        remaining: f64,
        limit: f64,
    },
    Done,
}

pub struct AgentStream {
    pub agent: Arc<Mutex<NeoCodexAgent>>,
    pub cost: Arc<Mutex<CostTracker>>,
    pub permissions: Arc<PermissionSystem>,
    pub hooks: Arc<LifecycleHookRegistry>,
    pub events: VecDeque<AgentEvent>,
}

impl AgentStream {
    pub fn new(agent: NeoCodexAgent, max_budget: f64) -> Self {
        Self {
            agent: Arc::new(Mutex::new(agent)),
            cost: Arc::new(Mutex::new(CostTracker::new(max_budget))),
            permissions: Arc::new(PermissionSystem::new()),
            hooks: Arc::new(LifecycleHookRegistry::new()),
            events: VecDeque::new(),
        }
    }

    /// Process input and yield events. Users poll `next_event()`.
    pub async fn process(&mut self, input: &str) {
        let mut agent = self.agent.lock().await;
        let turn = agent.state.turn_count + 1;

        self.events.push_back(AgentEvent::TurnStart {
            turn,
            mode: agent.state.mode,
        });

        // Permission check
        let tool_name = match agent.state.mode {
            NeoCodexMode::Shell => "shell",
            NeoCodexMode::Plan => "plan",
            NeoCodexMode::Agent => "agent",
        };
        if !self.permissions.interactive_check(tool_name, input) {
            self.events.push_back(AgentEvent::Error {
                message: format!("Permission denied for {}: {}", tool_name, input),
            });
            self.events.push_back(AgentEvent::Done);
            return;
        }

        // Lifecycle pre-hook
        let ctx = ToolCallContext {
            tool_name: tool_name.to_string(),
            args: input.to_string(),
            cwd: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            estimated_cost: input.len() as f64 * 0.00001,
        };
        match self.hooks.run_pre(ctx.clone()) {
            HookDecision::Deny(reason) => {
                self.events.push_back(AgentEvent::Error {
                    message: format!("Hook denied: {}", reason),
                });
                self.events.push_back(AgentEvent::Done);
                return;
            }
            HookDecision::RequireConfirm(msg) => {
                self.events.push_back(AgentEvent::Thinking {
                    content: format!("[hook requires confirmation: {}]", msg),
                });
            }
            HookDecision::Allow => {}
        }

        // Cost check
        let estimated_cost = input.len() as f64 * 0.00001;
        {
            let mut cost = self.cost.lock().await;
            if let Err(e) = cost.record(tool_name, estimated_cost, input.len() as u64 / 4) {
                self.events.push_back(AgentEvent::BudgetWarning {
                    remaining: cost.remaining(),
                    limit: cost.max_budget,
                });
                self.events.push_back(AgentEvent::Error { message: e });
                self.events.push_back(AgentEvent::Done);
                return;
            }
            if cost.remaining() < cost.max_budget * 0.1 {
                self.events.push_back(AgentEvent::BudgetWarning {
                    remaining: cost.remaining(),
                    limit: cost.max_budget,
                });
            }
        }

        // Execute
        self.events.push_back(AgentEvent::Thinking {
            content: format!("Processing in {:?} mode...", agent.state.mode),
        });

        let start = Instant::now();
        let response = agent.process(input).await;
        let duration = start.elapsed().as_millis() as u64;

        for line in response.lines() {
            self.events.push_back(AgentEvent::Chunk {
                text: line.to_string(),
            });
        }
        self.events.push_back(AgentEvent::TurnEnd {
            response: response.clone(),
        });
        self.events.push_back(AgentEvent::Done);

        // Lifecycle post-hook
        self.hooks.run_post(ctx, response.clone(), duration);

        // Cost post-record
        {
            let mut cost = self.cost.lock().await;
            let _ = cost.record(tool_name, 0.0, response.len() as u64 / 4);
        }
    }

    pub fn next_event(&mut self) -> Option<AgentEvent> {
        self.events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_stream_event_flow() {
        let agent = NeoCodexAgent::new("stream-test");
        let mut stream = AgentStream::new(agent, 5.0);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            stream.process("hello").await;
            let mut events = Vec::new();
            while let Some(ev) = stream.next_event() {
                events.push(ev);
            }
            assert!(events
                .iter()
                .any(|e| matches!(e, AgentEvent::TurnStart { .. })));
            assert!(events.iter().any(|e| matches!(e, AgentEvent::Done)));
        });
    }
}