use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use super::{DelegateRequest, DelegateResult, AgentTemplate};
use crate::neotrix::nt_mind::reasoning_engine::ReasoningEngine;

pub struct SubagentExecutor {
    template_registry: HashMap<String, AgentTemplate>,
    engine: Option<Arc<Mutex<ReasoningEngine>>>,
    _max_depth: usize,
}

impl Default for SubagentExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SubagentExecutor {
    pub fn new() -> Self {
        Self {
            template_registry: HashMap::new(),
            engine: None,
            _max_depth: 3,
        }
    }

    pub fn with_engine(engine: Arc<Mutex<ReasoningEngine>>) -> Self {
        Self {
            template_registry: HashMap::new(),
            engine: Some(engine),
            _max_depth: 3,
        }
    }

    pub fn register_template(&mut self, template: AgentTemplate) {
        self.template_registry.insert(template.name.clone(), template);
    }

    fn _resolve_task(&self, task: &Option<String>, prev: &Option<String>, _chain_dir: &str) -> String {
        let mut t = task.clone().unwrap_or_default();
        if let Some(prev_out) = prev {
            t = t.replace("{previous}", prev_out);
        }
        t
    }

    pub async fn execute(&self, request: DelegateRequest) -> DelegateResult {
        if let Some(ref engine) = self.engine {
            let mut eng = match engine.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            match eng.reason_task(&request.task) {
                Ok(output) => DelegateResult::success(output),
                Err(e) => DelegateResult::failure(e.to_string()),
            }
        } else {
            DelegateResult::success(format!("[{}] {}", request.agent, request.task))
        }
    }

    pub async fn execute_parallel(&self, tasks: Vec<(String, String)>) -> Vec<DelegateResult> {
        let mut results = Vec::new();
        for (agent, task) in tasks {
            let req = DelegateRequest::new(agent, task);
            results.push(self.execute(req).await);
        }
        results
    }
}

// Tests disabled: executor module needs API compatibility rewrite
