use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use uuid::Uuid;

use nt_domain::*;

pub trait AgentRouter: Send + Sync {
    fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionResponse, String>;
    fn health_check(&self) -> Result<(), String>;
    fn cancel(&self, request_id: &Uuid) -> Result<(), String>;
    fn capabilities(&self) -> Vec<String>;
}

pub trait HarnessAdapter: Send + Sync {
    fn kind(&self) -> HarnessKind;
    fn execute(&self, task: &str, timeout_secs: u64) -> Result<ExecutionResponse, String>;
    fn is_available(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub default_timeout_secs: u64,
    pub fallback_on_failure: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 60,
            fallback_on_failure: true,
        }
    }
}

pub struct RouterEngine {
    adapters: HashMap<HarnessKind, Box<dyn HarnessAdapter>>,
    config: RouterConfig,
}

impl RouterEngine {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            adapters: HashMap::new(),
            config,
        }
    }

    pub fn register_adapter(&mut self, kind: HarnessKind, adapter: Box<dyn HarnessAdapter>) {
        self.adapters.insert(kind, adapter);
    }

    pub fn select_harness(&self, request: &ExecutionRequest) -> Option<&dyn HarnessAdapter> {
        if let Some(adapter) = self.harness_from_context(request) {
            return Some(adapter);
        }
        for kind in &[
            HarnessKind::ClaudeCode,
            HarnessKind::Codex,
            HarnessKind::OpenAI,
            HarnessKind::Generic,
        ] {
            if let Some(adapter) = self.adapters.get(kind) {
                if adapter.is_available() {
                    return Some(adapter.as_ref());
                }
            }
        }
        None
    }

    pub fn execute(&self, request: &ExecutionRequest) -> ExecutionResponse {
        let request_id = Uuid::new_v4();
        let started_at = Utc::now();

        let timeout_secs = if request.timeout_secs > 0 {
            request.timeout_secs
        } else {
            self.config.default_timeout_secs
        };

        let candidates = self.collect_candidates(request);
        let mut last_error: Option<String> = None;

        for adapter in &candidates {
            if !adapter.is_available() {
                continue;
            }
            match adapter.execute(&request.task, timeout_secs) {
                Ok(response) => {
                    return ExecutionResponse {
                        request_id,
                        status: response.status,
                        output: response.output,
                        error: response.error,
                        started_at,
                        completed_at: response.completed_at,
                    };
                }
                Err(e) => {
                    last_error = Some(e);
                    if !self.config.fallback_on_failure {
                        break;
                    }
                }
            }
        }

        ExecutionResponse {
            request_id,
            status: ExecutionStatus::Failed,
            output: String::new(),
            error: Some(last_error.unwrap_or_else(|| "No adapters available".to_string())),
            started_at,
            completed_at: Some(Utc::now()),
        }
    }

    pub fn health_check_all(&self) -> Vec<(HarnessKind, bool)> {
        self.adapters
            .iter()
            .map(|(kind, adapter)| (kind.clone(), adapter.is_available()))
            .collect()
    }

    fn collect_candidates(&self, request: &ExecutionRequest) -> Vec<&dyn HarnessAdapter> {
        let mut result: Vec<&dyn HarnessAdapter> = Vec::new();
        let mut seen: Vec<HarnessKind> = Vec::new();

        if let Some(adapter) = self.harness_from_context(request) {
            result.push(adapter);
            seen.push(adapter.kind());
        }

        for kind in &[
            HarnessKind::ClaudeCode,
            HarnessKind::Codex,
            HarnessKind::OpenAI,
            HarnessKind::Generic,
        ] {
            if !seen.contains(kind) {
                if let Some(adapter) = self.adapters.get(kind) {
                    result.push(adapter.as_ref());
                    seen.push(kind.clone());
                }
            }
        }

        result
    }

    fn harness_from_context(&self, request: &ExecutionRequest) -> Option<&dyn HarnessAdapter> {
        let context = request.context.as_ref()?;
        let hint = context.get("harness")?;
        let hint_str = hint.as_str()?;
        let kind = match hint_str.to_lowercase().as_str() {
            "claude-code" | "claudecode" => HarnessKind::ClaudeCode,
            "codex" => HarnessKind::Codex,
            "openai" => HarnessKind::OpenAI,
            _ => HarnessKind::Generic,
        };
        self.adapters.get(&kind).map(|a| a.as_ref())
    }
}

impl AgentRouter for RouterEngine {
    fn execute(&self, request: &ExecutionRequest) -> Result<ExecutionResponse, String> {
        Ok(RouterEngine::execute(self, request))
    }

    fn health_check(&self) -> Result<(), String> {
        if self.adapters.is_empty() {
            return Err("No adapters registered".to_string());
        }
        let any_available = self.adapters.values().any(|a| a.is_available());
        if any_available {
            Ok(())
        } else {
            Err("No adapters available".to_string())
        }
    }

    fn cancel(&self, _request_id: &Uuid) -> Result<(), String> {
        Err("Cancellation not implemented".to_string())
    }

    fn capabilities(&self) -> Vec<String> {
        self.adapters
            .keys()
            .map(|k| format!("{:?}", k))
            .collect()
    }
}

pub struct MockAdapter {
    kind: HarnessKind,
    delay_ms: u64,
    should_fail: bool,
    available: bool,
}

impl MockAdapter {
    pub fn new(kind: HarnessKind, delay_ms: u64, should_fail: bool) -> Self {
        Self {
            kind,
            delay_ms,
            should_fail,
            available: true,
        }
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }
}

impl HarnessAdapter for MockAdapter {
    fn kind(&self) -> HarnessKind {
        self.kind.clone()
    }

    fn execute(&self, task: &str, timeout_secs: u64) -> Result<ExecutionResponse, String> {
        let started_at = Utc::now();
        let delay = Duration::from_millis(self.delay_ms);
        let timeout = Duration::from_secs(timeout_secs);

        if delay > timeout {
            thread::sleep(timeout);
            return Ok(ExecutionResponse {
                request_id: Uuid::new_v4(),
                status: ExecutionStatus::Timeout,
                output: String::new(),
                error: Some("Execution timed out".to_string()),
                started_at,
                completed_at: Some(Utc::now()),
            });
        }

        thread::sleep(delay);

        if self.should_fail {
            Ok(ExecutionResponse {
                request_id: Uuid::new_v4(),
                status: ExecutionStatus::Failed,
                output: String::new(),
                error: Some("Mock failure".to_string()),
                started_at,
                completed_at: Some(Utc::now()),
            })
        } else {
            Ok(ExecutionResponse {
                request_id: Uuid::new_v4(),
                status: ExecutionStatus::Completed,
                output: format!("Mock output for: {}", task),
                error: None,
                started_at,
                completed_at: Some(Utc::now()),
            })
        }
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine() -> RouterEngine {
        RouterEngine::new(RouterConfig::default())
    }

    #[test]
    fn test_register_and_execute() {
        let mut engine = test_engine();
        let adapter = MockAdapter::new(HarnessKind::Generic, 1, false);
        engine.register_adapter(HarnessKind::Generic, Box::new(adapter));

        let request = ExecutionRequest {
            agent_id: Uuid::new_v4(),
            task: "hello world".to_string(),
            context: None,
            timeout_secs: 10,
        };

        let response = engine.execute(&request);
        assert_eq!(response.status, ExecutionStatus::Completed);
        assert!(response.output.contains("hello world"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_health_check() {
        let mut engine = test_engine();
        let adapter = MockAdapter::new(HarnessKind::ClaudeCode, 1, false);
        engine.register_adapter(HarnessKind::ClaudeCode, Box::new(adapter));

        let results = engine.health_check_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].1);

        let mut engine2 = test_engine();
        let mut adapter2 = MockAdapter::new(HarnessKind::Codex, 1, false);
        adapter2.set_available(false);
        engine2.register_adapter(HarnessKind::Codex, Box::new(adapter2));

        let results2 = engine2.health_check_all();
        assert!(!results2[0].1);
    }

    #[test]
    fn test_fallback() {
        let mut engine = test_engine();
        let primary = MockAdapter::new(HarnessKind::ClaudeCode, 1, true);
        let fallback = MockAdapter::new(HarnessKind::Generic, 1, false);
        engine.register_adapter(HarnessKind::ClaudeCode, Box::new(primary));
        engine.register_adapter(HarnessKind::Generic, Box::new(fallback));

        let request = ExecutionRequest {
            agent_id: Uuid::new_v4(),
            task: "fallback test".to_string(),
            context: None,
            timeout_secs: 10,
        };

        let response = engine.execute(&request);
        assert_eq!(response.status, ExecutionStatus::Completed);
        assert!(response.output.contains("fallback test"));
    }

    #[test]
    fn test_timeout() {
        let mut engine = test_engine();
        let adapter = MockAdapter::new(HarnessKind::Generic, 2000, false);
        engine.register_adapter(HarnessKind::Generic, Box::new(adapter));

        let request = ExecutionRequest {
            agent_id: Uuid::new_v4(),
            task: "will timeout".to_string(),
            context: None,
            timeout_secs: 1,
        };

        let response = engine.execute(&request);
        assert_eq!(response.status, ExecutionStatus::Timeout);
    }

    #[test]
    fn test_capabilities() {
        let mut engine = test_engine();
        let adapter = MockAdapter::new(HarnessKind::ClaudeCode, 1, false);
        engine.register_adapter(HarnessKind::ClaudeCode, Box::new(adapter));

        let caps = AgentRouter::capabilities(&engine);
        assert_eq!(caps.len(), 1);
        assert!(caps[0].contains("ClaudeCode"));
    }
}
