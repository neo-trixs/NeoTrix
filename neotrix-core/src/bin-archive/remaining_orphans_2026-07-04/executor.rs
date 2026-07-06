use std::collections::HashMap;
use std::time::Instant;

use super::planner::Step;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub step_id: usize,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ToolHandler {
    pub name: String,
    pub description: String,
    handler_fn: fn(&str) -> (bool, String),
}

#[derive(Debug, Clone)]
pub struct Executor {
    tools: HashMap<String, ToolHandler>,
    execution_log: Vec<ExecutionResult>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            execution_log: Vec::new(),
        }
    }

    pub fn register_tool(&mut self, name: &str, description: &str, handler: fn(&str) -> (bool, String)) {
        self.tools.insert(
            name.to_string(),
            ToolHandler {
                name: name.to_string(),
                description: description.to_string(),
                handler_fn: handler,
            },
        );
    }

    pub fn execute(&self, step: &Step) -> ExecutionResult {
        let start = Instant::now();

        let tool_name = match &step.required_tool {
            Some(name) => name.clone(),
            None => {
                let elapsed = start.elapsed().as_millis() as u64;
                return ExecutionResult {
                    step_id: step.id,
                    success: true,
                    output: format!("Executed: {}", step.description),
                    error: None,
                    duration_ms: elapsed,
                };
            }
        };

        match self.tools.get(&tool_name) {
            Some(handler) => {
                let (success, output) = (handler.handler_fn)(&step.description);
                let elapsed = start.elapsed().as_millis() as u64;
                ExecutionResult {
                    step_id: step.id,
                    success,
                    output,
                    error: if success { None } else { Some(output.clone()) },
                    duration_ms: elapsed,
                }
            }
            None => {
                let elapsed = start.elapsed().as_millis() as u64;
                ExecutionResult {
                    step_id: step.id,
                    success: false,
                    output: String::new(),
                    error: Some(format!("Tool '{}' not registered", tool_name)),
                    duration_ms: elapsed,
                }
            }
        }
    }

    pub fn execute_all(&self, steps: &[Step]) -> Vec<ExecutionResult> {
        steps.iter().map(|step| self.execute(step)).collect()
    }

    pub fn execute_parallel(&self, steps: &[Step]) -> Vec<ExecutionResult> {
        let mut results: Vec<ExecutionResult> = Vec::with_capacity(steps.len());
        let mut completed: HashMap<usize, &ExecutionResult> = HashMap::new();

        let mut remaining: Vec<&Step> = steps.iter().collect();
        let mut max_iterations = steps.len() * 2;
        let mut iteration = 0;

        while !remaining.is_empty() && iteration < max_iterations {
            iteration += 1;
            let mut progress = false;

            remaining.retain(|step| {
                let deps_met = step.depends_on.iter().all(|dep_id| {
                    completed.contains_key(dep_id)
                });

                if deps_met {
                    let result = self.execute(step);
                    let result_boxed = result;
                    completed.insert(step.id, results.get(results.len().wrapping_sub(1)).unwrap_or(&ExecutionResult {
                        step_id: usize::MAX,
                        success: false,
                        output: String::new(),
                        error: None,
                        duration_ms: 0,
                    }));
                    results.push(result_boxed);
                    progress = true;
                    false
                } else {
                    true
                }
            });

            if !progress && !remaining.is_empty() {
                let step = remaining[0];
                let result = self.execute(step);
                results.push(result);
                remaining.remove(0);
            }
        }

        results
    }

    pub fn log(&self) -> &[ExecutionResult] {
        &self.execution_log
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn available_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_handler(input: &str) -> (bool, String) {
        (true, format!("Handled: {}", input))
    }

    fn failing_handler(_input: &str) -> (bool, String) {
        (false, "Tool execution failed".to_string())
    }

    #[test]
    fn test_executor_creation() {
        let executor = Executor::new();
        assert_eq!(executor.tool_count(), 0);
    }

    #[test]
    fn test_register_and_execute() {
        let mut executor = Executor::new();
        executor.register_tool("test_tool", "A test tool", mock_handler);

        let step = Step {
            id: 0,
            description: "do something".to_string(),
            required_tool: Some("test_tool".to_string()),
            depends_on: vec![],
        };

        let result = executor.execute(&step);
        assert!(result.success);
        assert!(result.output.contains("do something"));
    }

    #[test]
    fn test_execute_no_tool() {
        let executor = Executor::new();
        let step = Step {
            id: 0,
            description: "simple step".to_string(),
            required_tool: None,
            depends_on: vec![],
        };
        let result = executor.execute(&step);
        assert!(result.success);
    }

    #[test]
    fn test_execute_missing_tool() {
        let executor = Executor::new();
        let step = Step {
            id: 0,
            description: "missing".to_string(),
            required_tool: Some("nonexistent".to_string()),
            depends_on: vec![],
        };
        let result = executor.execute(&step);
        assert!(!result.success);
        assert!(result.error.unwrap().contains("not registered"));
    }

    #[test]
    fn test_failing_handler() {
        let mut executor = Executor::new();
        executor.register_tool("bad_tool", "Fails", failing_handler);
        let step = Step {
            id: 0,
            description: "fail".to_string(),
            required_tool: Some("bad_tool".to_string()),
            depends_on: vec![],
        };
        let result = executor.execute(&step);
        assert!(!result.success);
    }

    #[test]
    fn test_available_tools() {
        let mut executor = Executor::new();
        executor.register_tool("a", "tool a", mock_handler);
        executor.register_tool("b", "tool b", mock_handler);
        let tools = executor.available_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"a".to_string()));
    }

    #[test]
    fn test_execute_all() {
        let mut executor = Executor::new();
        executor.register_tool("t", "tool", mock_handler);
        let steps = vec![
            Step { id: 0, description: "s1".to_string(), required_tool: None, depends_on: vec![] },
            Step { id: 1, description: "s2".to_string(), required_tool: Some("t".to_string()), depends_on: vec![0] },
        ];
        let results = executor.execute_all(&steps);
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }
}
