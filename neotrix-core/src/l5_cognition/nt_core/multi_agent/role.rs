//! Agent Role Definitions (crewAI pattern)
//!
//! Each role defines the capabilities, constraints, and tool access
//! for a specialized agent within a crew.

use serde::{Deserialize, Serialize};

/// Predefined agent roles — each maps to a specific cognitive function
/// within the NeoTrix 6-layer architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    /// Gathers and synthesizes information from external sources
    Researcher,
    /// Produces code, artifacts, or structured output
    Coder,
    /// Validates output against acceptance criteria (R-P130)
    Reviewer,
    /// Decomposes tasks and creates execution plans
    Planner,
    /// Executes concrete actions in the environment
    Executor,
}

impl AgentRole {
    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            AgentRole::Researcher => "Researcher",
            AgentRole::Coder => "Coder",
            AgentRole::Reviewer => "Reviewer",
            AgentRole::Planner => "Planner",
            AgentRole::Executor => "Executor",
        }
    }

    /// Default tools granted to this role
    pub fn default_tools(&self) -> Vec<&'static str> {
        match self {
            AgentRole::Researcher => vec!["web_search", "file_read", "rag_query"],
            AgentRole::Coder => vec!["file_write", "file_read", "shell_exec"],
            AgentRole::Reviewer => vec!["file_read", "diff", "test_runner"],
            AgentRole::Planner => vec!["file_read", "task_decompose", "graph_query"],
            AgentRole::Executor => vec!["shell_exec", "file_write", "http_request"],
        }
    }

    /// Default max iterations before the role agent must yield
    pub fn default_max_iterations(&self) -> u32 {
        match self {
            AgentRole::Researcher => 5,
            AgentRole::Coder => 10,
            AgentRole::Reviewer => 3,
            AgentRole::Planner => 4,
            AgentRole::Executor => 8,
        }
    }

    /// Default context budget (tokens) for this role
    pub fn default_context_budget(&self) -> usize {
        match self {
            AgentRole::Researcher => 16_384,
            AgentRole::Coder => 24_576,
            AgentRole::Reviewer => 8_192,
            AgentRole::Planner => 12_288,
            AgentRole::Executor => 16_384,
        }
    }
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Configuration for a single agent within a crew.
///
/// Combines the role with specific constraints and tool permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// The functional role of this agent
    pub role: AgentRole,
    /// Tools this agent is allowed to use (overrides role defaults if non-empty)
    pub allowed_tools: Vec<String>,
    /// Maximum reasoning iterations before forced yield
    pub max_iterations: u32,
    /// Token budget for context assembly
    pub context_budget: usize,
    /// Optional human-readable name for this agent instance
    pub name: Option<String>,
}

impl RoleConfig {
    /// Create from a role with all defaults
    pub fn new(role: AgentRole) -> Self {
        Self {
            allowed_tools: role.default_tools().into_iter().map(String::from).collect(),
            max_iterations: role.default_max_iterations(),
            context_budget: role.default_context_budget(),
            name: None,
            role,
        }
    }

    /// Override allowed tools
    pub fn with_tools(mut self, tools: Vec<&str>) -> Self {
        self.allowed_tools = tools.into_iter().map(String::from).collect();
        self
    }

    /// Override max iterations
    pub fn with_max_iterations(mut self, n: u32) -> Self {
        self.max_iterations = n;
        self
    }

    /// Override context budget
    pub fn with_context_budget(mut self, budget: usize) -> Self {
        self.context_budget = budget;
        self
    }

    /// Set a display name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Check if a tool is allowed for this role
    pub fn can_use_tool(&self, tool: &str) -> bool {
        self.allowed_tools.iter().any(|t| t == tool)
    }

    /// Display name or role label
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| self.role.label().to_string())
    }
}

impl Default for RoleConfig {
    fn default() -> Self {
        Self::new(AgentRole::Executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_defaults() {
        assert_eq!(AgentRole::Researcher.label(), "Researcher");
        assert!(!AgentRole::Coder.default_tools().is_empty());
        assert!(AgentRole::Reviewer.default_max_iterations() > 0);
        assert!(AgentRole::Planner.default_context_budget() > 0);
    }

    #[test]
    fn role_config_builder() {
        let cfg = RoleConfig::new(AgentRole::Coder)
            .with_tools(vec!["file_write"])
            .with_max_iterations(20)
            .with_context_budget(32_768)
            .with_name("Primary Coder");
        assert_eq!(cfg.role, AgentRole::Coder);
        assert_eq!(cfg.allowed_tools, vec!["file_write"]);
        assert_eq!(cfg.max_iterations, 20);
        assert_eq!(cfg.context_budget, 32_768);
        assert_eq!(cfg.display_name(), "Primary Coder");
    }

    #[test]
    fn can_use_tool_check() {
        let cfg = RoleConfig::new(AgentRole::Researcher);
        assert!(cfg.can_use_tool("web_search"));
        assert!(!cfg.can_use_tool("shell_exec"));
    }

    #[test]
    fn role_config_default_is_executor() {
        let cfg = RoleConfig::default();
        assert_eq!(cfg.role, AgentRole::Executor);
    }

    #[test]
    fn display_name_falls_back_to_role() {
        let cfg = RoleConfig::new(AgentRole::Reviewer);
        assert_eq!(cfg.display_name(), "Reviewer");
    }

    #[test]
    fn role_serialization_roundtrip() {
        let role = AgentRole::Planner;
        let json = serde_json::to_string(&role).unwrap();
        let back: AgentRole = serde_json::from_str(&json).unwrap();
        assert_eq!(role, back);
    }

    #[test]
    fn role_config_serialization_roundtrip() {
        let cfg = RoleConfig::new(AgentRole::Coder).with_name("Test");
        let json = serde_json::to_string(&cfg).unwrap();
        let back: RoleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.role, back.role);
        assert_eq!(cfg.name, back.name);
    }
}
