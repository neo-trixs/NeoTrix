pub mod agent_template;
pub mod executor;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AgentTemplate {
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub model: Option<String>,
}

#[derive(Debug)]
pub struct DelegateRequest {
    pub agent: String,
    pub task: String,
    pub context: ExecutionContext,
    pub clarify: bool,
    pub async_exec: bool,
    pub chain: Option<Chain>,
    pub variables: HashMap<String, String>,
}

impl DelegateRequest {
    pub fn new(agent: String, task: String) -> Self {
        Self {
            agent,
            task,
            context: ExecutionContext::Fresh,
            clarify: false,
            async_exec: false,
            chain: None,
            variables: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct DelegateResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl DelegateResult {
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output: Some(output),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionContext {
    Fresh,
    Fork,
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub steps: Vec<ChainStep>,
}

#[derive(Debug, Clone)]
pub struct ChainStep {
    pub agent: String,
    pub task: Option<String>,
}

pub const TEMPLATE_VARIABLE_TASK: &str = "task";
pub const TEMPLATE_VARIABLE_PREVIOUS: &str = "previous";
pub const TEMPLATE_VARIABLE_CHAIN_DIR: &str = "chain_dir";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = AgentTemplate {
            name: "test_agent".to_string(),
            description: "A test agent".to_string(),
            capabilities: vec!["rust".to_string(), "testing".to_string()],
            model: None,
        };
        assert_eq!(template.name, "test_agent");
    }

    #[test]
    fn test_delegate_request_creation() {
        let req = DelegateRequest::new("agent1".to_string(), "do something".to_string());
        assert_eq!(req.agent, "agent1");
        assert!(req.chain.is_none());
    }

    #[test]
    fn test_delegate_result_success() {
        let r = DelegateResult::success("done".to_string());
        assert!(r.success);
        assert_eq!(r.output, Some("done".to_string()));
    }
}
