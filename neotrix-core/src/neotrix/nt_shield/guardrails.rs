/// Guardrails system — inspired by CrewAI's Task and Agent-level validation.
/// Pre-execution checks, post-execution validation, and output screening.

use serde::{Deserialize, Serialize};
use crate::neotrix::nt_shield::tool_permissions::ToolPermission;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub max_tool_calls: usize,         // max tool calls per task
    pub max_input_length: usize,       // max input length for any tool
    pub max_output_length: usize,      // max output length (truncation)
    pub allowed_paths: Vec<String>,    // allowed filesystem prefixes
    pub blocked_paths: Vec<String>,    // blocked filesystem prefixes
    pub allowed_domains: Vec<String>,  // allowed network domains
    pub blocked_domains: Vec<String>,  // blocked network domains
    pub require_confirmation: Vec<ToolPermission>, // permissions needing user ok
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        Self {
            max_tool_calls: 200,
            max_input_length: 100_000,
            max_output_length: 500_000,
            allowed_paths: vec!["/tmp".into(), ".".into()],
            blocked_paths: vec!["/etc".into(), "/sys".into()],
            allowed_domains: vec![],
            blocked_domains: vec!["localhost".into(), "127.0.0.1".into()],
            require_confirmation: vec![ToolPermission::Shell, ToolPermission::SystemConfig],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailResult {
    pub passed: bool,
    pub violations: Vec<GuardrailViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailViolation {
    pub rule: String,
    pub detail: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Blocked,
    Critical,
}

/// Pre-execution guard: checks if a tool call should be allowed
pub struct InputGuardrail {
    config: GuardrailConfig,
}

impl InputGuardrail {
    pub fn new(config: GuardrailConfig) -> Self {
        Self { config }
    }

    /// Validate a tool call before execution
    pub fn validate(&self, tool_id: &str, input: &str, permission: Option<&ToolPermission>) -> GuardrailResult {
        let mut violations = Vec::new();

        // Check input length
        if input.len() > self.config.max_input_length {
            violations.push(GuardrailViolation {
                rule: "max_input_length".into(),
                detail: format!("Input length {} exceeds max {}", input.len(), self.config.max_input_length),
                severity: ViolationSeverity::Blocked,
            });
        }

        // Check permission
        if let Some(perm) = permission {
            if self.config.require_confirmation.contains(perm) {
                violations.push(GuardrailViolation {
                    rule: "require_confirmation".into(),
                    detail: format!("Tool '{}' requires confirmation for {:?}", tool_id, perm),
                    severity: ViolationSeverity::Warning,
                });
            }
        }

        // Check paths (if input looks like a path)
        if input.contains('/') || input.contains('\\') {
            for blocked in &self.config.blocked_paths {
                if input.contains(blocked.as_str()) {
                    violations.push(GuardrailViolation {
                        rule: "blocked_path".into(),
                        detail: format!("Input references blocked path '{}'", blocked),
                        severity: ViolationSeverity::Blocked,
                    });
                }
            }
        }

        GuardrailResult {
            passed: violations.iter().all(|v| v.severity != ViolationSeverity::Blocked),
            violations,
        }
    }

    /// Check if a tool call count exceeds the limit
    pub fn check_tool_limit(&self, call_count: usize) -> GuardrailResult {
        if call_count > self.config.max_tool_calls {
            GuardrailResult {
                passed: false,
                violations: vec![GuardrailViolation {
                    rule: "max_tool_calls".into(),
                    detail: format!("Tool call count {} exceeds max {}", call_count, self.config.max_tool_calls),
                    severity: ViolationSeverity::Critical,
                }],
            }
        } else {
            GuardrailResult {
                passed: true,
                violations: vec![],
            }
        }
    }
}

/// Post-execution guard: validates tool output
pub struct OutputGuardrail {
    config: GuardrailConfig,
}

impl OutputGuardrail {
    pub fn new(config: GuardrailConfig) -> Self {
        Self { config }
    }

    /// Validate tool output before returning to LLM
    pub fn validate(&self, output: &str) -> GuardrailResult {
        let mut violations = Vec::new();

        // Check output length (only warn, don't block)
        if output.len() > self.config.max_output_length {
            violations.push(GuardrailViolation {
                rule: "max_output_length".into(),
                detail: format!("Output {} exceeds max {}", output.len(), self.config.max_output_length),
                severity: ViolationSeverity::Warning,
            });
        }

        GuardrailResult {
            passed: violations.iter().all(|v| v.severity == ViolationSeverity::Warning),
            violations,
        }
    }
}

/// Combined guardrail system
pub struct GuardrailSystem {
    pub input: InputGuardrail,
    pub output: OutputGuardrail,
    pub config: GuardrailConfig,
    tool_call_count: std::sync::atomic::AtomicUsize,
}

impl GuardrailSystem {
    pub fn new(config: GuardrailConfig) -> Self {
        Self {
            input: InputGuardrail::new(config.clone()),
            output: OutputGuardrail::new(config.clone()),
            tool_call_count: std::sync::atomic::AtomicUsize::new(0),
            config,
        }
    }

    pub fn check_tool_call(&self, tool_id: &str, input: &str, permission: Option<&ToolPermission>) -> GuardrailResult {
        let prev_count = self.tool_call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let call_count = prev_count + 1; // post-increment — actual call number
        let mut violations = Vec::new();

        // Tool call limit
        let limit_result = self.input.check_tool_limit(call_count);
        violations.extend(limit_result.violations);

        // Input validation
        let input_result = self.input.validate(tool_id, input, permission);
        violations.extend(input_result.violations);

        GuardrailResult {
            passed: violations.iter().all(|v| v.severity != ViolationSeverity::Blocked),
            violations,
        }
    }

    pub fn check_output(&self, output: &str) -> GuardrailResult {
        self.output.validate(output)
    }

    pub fn reset_call_count(&self) {
        self.tool_call_count.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GuardrailConfig::default();
        assert_eq!(config.max_tool_calls, 200);
    }

    #[test]
    fn test_input_guardrail_blocks_long_input() {
        let config = GuardrailConfig {
            max_input_length: 10,
            ..Default::default()
        };
        let guard = InputGuardrail::new(config);
        let result = guard.validate("test", "this is a very long input that exceeds the limit", None);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "max_input_length"));
    }

    #[test]
    fn test_input_guardrail_allows_short_input() {
        let config = GuardrailConfig::default();
        let guard = InputGuardrail::new(config);
        let result = guard.validate("test", "short", None);
        assert!(result.passed);
    }

    #[test]
    fn test_input_guardrail_blocks_path_traversal() {
        let config = GuardrailConfig::default();
        let guard = InputGuardrail::new(config);
        let result = guard.validate("read_file", "../../etc/passwd", None);
        assert!(!result.passed);
    }

    #[test]
    fn test_tool_call_limit() {
        let config = GuardrailConfig {
            max_tool_calls: 1,
            ..Default::default()
        };
        let guard = InputGuardrail::new(config);
        // 0 calls → ok
        assert!(guard.check_tool_limit(0).passed);
        // 1 call → exactly at limit
        assert!(guard.check_tool_limit(1).passed);
        // 2 calls → exceeds
        assert!(!guard.check_tool_limit(2).passed);
    }

    #[test]
    fn test_guardrail_system() {
        let system = GuardrailSystem::new(GuardrailConfig::default());
        let result = system.check_tool_call("web_scrape", "https://example.com", Some(&ToolPermission::Network));
        // Should pass (no blocking violations for a normal URL)
        assert!(result.passed);
    }

    #[test]
    fn test_output_guardrail_warns_on_long() {
        let config = GuardrailConfig {
            max_output_length: 5,
            ..Default::default()
        };
        let guard = OutputGuardrail::new(config);
        let result = guard.validate("very long output");
        assert!(result.passed); // Warnings don't block
        assert!(result.violations.iter().any(|v| v.rule == "max_output_length"));
    }
}
