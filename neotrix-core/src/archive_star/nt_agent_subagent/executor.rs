use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::{DelegateRequest, DelegateResult, AgentTemplate};

pub trait ReasoningProvider: Send + Sync {
    fn reason(&mut self, task: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct SubagentExecutor {
    template_registry: HashMap<String, AgentTemplate>,
    engine: Option<Arc<Mutex<Box<dyn ReasoningProvider>>>>,
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

    pub fn with_engine(engine: Arc<Mutex<Box<dyn ReasoningProvider>>>) -> Self {
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
            match eng.reason(&request.task) {
                Ok(output) => DelegateResult::success(output),
                Err(e) => DelegateResult::failure(e.to_string()),
            }
        } else {
            DelegateResult::success(format!("[{}] {}", request.agent, request.task))
        }
    }

    pub async fn execute_parallel(&self, tasks: Vec<(String, String)>) -> Vec<DelegateResult> {
        use futures::future::join_all;
        let futs: Vec<_> = tasks.into_iter().map(|(agent, task)| {
            let req = DelegateRequest::new(agent, task);
            self.execute(req)
        }).collect();
        join_all(futs).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_empty_engine() {
        let exec = SubagentExecutor::new();
        let req = DelegateRequest::new("test_agent".into(), "do_something".into());
        let result = futures::executor::block_on(exec.execute(req));
        assert!(result.success);
        assert!(result.output.unwrap().contains("test_agent"));
    }

    #[test]
    fn test_execute_parallel_no_engine() {
        let exec = SubagentExecutor::new();
        let tasks = vec![
            ("a1".into(), "task1".into()),
            ("a2".into(), "task2".into()),
        ];
        let results = futures::executor::block_on(exec.execute_parallel(tasks));
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_register_template() {
        let mut exec = SubagentExecutor::new();
        let t = AgentTemplate {
            name: "researcher".into(),
            description: "Research agent".into(),
            capabilities: vec!["search".into()],
            model: None,
        };
        exec.register_template(t);
        assert!(exec.template_registry.contains_key("researcher"));
    }
}
