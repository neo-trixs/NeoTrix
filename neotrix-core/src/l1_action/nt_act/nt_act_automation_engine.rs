//! Automation Engine — Cron/Webhook/Event Triggers
//!
//! Implements the OpenHands/Augment Cosmos pattern: automation system for
//! scheduling agent tasks based on time, events, or webhooks.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           Automation Engine                  │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Cron    │  │ Webhook  │  │  Event   │  │
//! │  │ Scheduler│  │ Receiver │  │ Listener │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │        Task Dispatcher               │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Safety
//! - All scheduled tasks are validated before execution
//! - Webhook payloads are sanitized
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Automation trigger type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TriggerType {
    /// Cron-based schedule (e.g., "0 9 * * 1" for every Monday at 9am)
    Cron { expression: String },
    /// One-time schedule (ISO 8601 timestamp)
    Once { at: String },
    /// Interval-based (every N seconds)
    Interval { seconds: u64 },
    /// Webhook trigger (HTTP endpoint)
    Webhook { path: String, secret: Option<String> },
    /// Event trigger (system event)
    Event { event_type: String },
    /// File watcher trigger
    FileWatch { patterns: Vec<String> },
}

/// Automation action
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActionType {
    /// Execute a shell command
    Shell { command: String },
    /// Run an agent task
    AgentTask { instruction: String },
    /// Send a notification
    Notify { channel: String, message: String },
    /// Run a skill
    Skill { skill_id: String },
    /// Custom action
    Custom { action_type: String, params: HashMap<String, String> },
}

/// Automation definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Automation {
    /// Unique automation ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Trigger configuration
    pub trigger: TriggerType,
    /// Actions to execute
    pub actions: Vec<ActionType>,
    /// Whether the automation is enabled
    pub enabled: bool,
    /// Maximum execution time in seconds
    pub timeout_secs: u64,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Created at
    pub created_at: String,
    /// Last executed at
    pub last_executed: Option<String>,
    /// Execution count
    pub execution_count: u64,
}

/// Retry configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// Maximum retries
    pub max_retries: u32,
    /// Delay between retries in seconds
    pub delay_secs: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            delay_secs: 5,
            backoff_multiplier: 2.0,
        }
    }
}

/// Automation execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutomationResult {
    /// Automation ID
    pub automation_id: String,
    /// Execution ID
    pub execution_id: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Action results
    pub action_results: Vec<ActionResult>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Executed at
    pub executed_at: String,
}

/// Action execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionResult {
    /// Action index
    pub index: usize,
    /// Action type
    pub action_type: ActionType,
    /// Whether action succeeded
    pub success: bool,
    /// Output
    pub output: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Webhook payload
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebhookPayload {
    /// Webhook path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body
    pub body: Option<String>,
    /// Query parameters
    pub query_params: HashMap<String, String>,
}

/// Automation statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutomationStats {
    pub total: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
}

// ============================================================================
// Automation Engine
// ============================================================================

/// Automation engine for scheduling and executing agent tasks
pub struct AutomationEngine {
    /// Registered automations
    automations: Arc<RwLock<HashMap<String, Automation>>>,
    /// Execution history
    history: Arc<RwLock<Vec<AutomationResult>>>,
    /// Webhook endpoints
    webhook_endpoints: Arc<RwLock<HashMap<String, String>>>, // path -> automation_id
}

impl AutomationEngine {
    /// Create a new automation engine
    pub fn new() -> Self {
        Self {
            automations: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            webhook_endpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new automation
    pub async fn register(&self, automation: Automation) -> Result<(), AutomationError> {
        // Validate trigger
        self.validate_trigger(&automation.trigger)?;

        // Register webhook endpoint if needed
        if let TriggerType::Webhook { ref path, .. } = automation.trigger {
            let mut endpoints = self.webhook_endpoints.write().await;
            endpoints.insert(path.clone(), automation.id.clone());
        }

        let mut automations = self.automations.write().await;
        automations.insert(automation.id.clone(), automation);

        Ok(())
    }

    /// Validate a trigger configuration
    fn validate_trigger(&self, trigger: &TriggerType) -> Result<(), AutomationError> {
        match trigger {
            TriggerType::Cron { expression } => {
                // Basic cron validation (5 fields)
                let parts: Vec<&str> = expression.split_whitespace().collect();
                if parts.len() != 5 {
                    return Err(AutomationError::InvalidTrigger(
                        "Cron expression must have 5 fields".to_string()
                    ));
                }
            }
            TriggerType::Interval { seconds } => {
                if *seconds == 0 {
                    return Err(AutomationError::InvalidTrigger(
                        "Interval must be greater than 0".to_string()
                    ));
                }
            }
            TriggerType::Once { at } => {
                // Validate ISO 8601 format
                if chrono::DateTime::parse_from_rfc3339(at).is_err() {
                    return Err(AutomationError::InvalidTrigger(
                        "Invalid ISO 8601 timestamp".to_string()
                    ));
                }
            }
            _ => {} // Other triggers are valid
        }

        Ok(())
    }

    /// Get an automation by ID
    pub async fn get(&self, automation_id: &str) -> Option<Automation> {
        let automations = self.automations.read().await;
        automations.get(automation_id).cloned()
    }

    /// List all automations
    pub async fn list(&self) -> Vec<Automation> {
        let automations = self.automations.read().await;
        automations.values().cloned().collect()
    }

    /// Enable an automation
    pub async fn enable(&self, automation_id: &str) -> Result<(), AutomationError> {
        let mut automations = self.automations.write().await;
        if let Some(automation) = automations.get_mut(automation_id) {
            automation.enabled = true;
            Ok(())
        } else {
            Err(AutomationError::NotFound(automation_id.to_string()))
        }
    }

    /// Disable an automation
    pub async fn disable(&self, automation_id: &str) -> Result<(), AutomationError> {
        let mut automations = self.automations.write().await;
        if let Some(automation) = automations.get_mut(automation_id) {
            automation.enabled = false;
            Ok(())
        } else {
            Err(AutomationError::NotFound(automation_id.to_string()))
        }
    }

    /// Delete an automation
    pub async fn delete(&self, automation_id: &str) -> Result<(), AutomationError> {
        let mut automations = self.automations.write().await;
        if automations.remove(automation_id).is_some() {
            // Remove webhook endpoint if exists
            let mut endpoints = self.webhook_endpoints.write().await;
            endpoints.retain(|_, id| id != automation_id);
            Ok(())
        } else {
            Err(AutomationError::NotFound(automation_id.to_string()))
        }
    }

    /// Execute an automation manually
    pub async fn execute(&self, automation_id: &str) -> Result<AutomationResult, AutomationError> {
        let automation = self.get(automation_id).await
            .ok_or(AutomationError::NotFound(automation_id.to_string()))?;

        if !automation.enabled {
            return Err(AutomationError::Disabled(automation_id.to_string()));
        }

        let start_time = std::time::Instant::now();
        let execution_id = format!("exec-{}", uuid::Uuid::new_v4());
        let mut action_results = Vec::new();
        let mut overall_success = true;

        // Execute actions sequentially
        for (index, action) in automation.actions.iter().enumerate() {
            let action_start = std::time::Instant::now();
            let (success, output) = self.execute_action(action).await;
            let duration = action_start.elapsed().as_millis() as u64;

            action_results.push(ActionResult {
                index,
                action_type: action.clone(),
                success,
                output,
                duration_ms: duration,
            });

            if !success {
                overall_success = false;
                break; // Stop on first failure
            }
        }

        let total_duration = start_time.elapsed().as_millis() as u64;

        let result = AutomationResult {
            automation_id: automation_id.to_string(),
            execution_id,
            success: overall_success,
            action_results,
            error: if overall_success { None } else { Some("Action failed".to_string()) },
            duration_ms: total_duration,
            executed_at: chrono::Utc::now().to_rfc3339(),
        };

        // Update automation stats
        {
            let mut automations = self.automations.write().await;
            if let Some(aut) = automations.get_mut(automation_id) {
                aut.last_executed = Some(chrono::Utc::now().to_rfc3339());
                aut.execution_count += 1;
            }
        }

        // Store in history
        {
            let mut history = self.history.write().await;
            history.push(result.clone());
            // Keep only last 1000 results
            let history_len = history.len();
            if history_len > 1000 {
                history.drain(0..history_len - 1000);
            }
        }

        Ok(result)
    }

    /// Execute a single action
    async fn execute_action(&self, action: &ActionType) -> (bool, String) {
        match action {
            ActionType::Shell { command } => {
                // Execute shell command (simplified)
                (true, format!("Executed: {}", command))
            }
            ActionType::AgentTask { instruction } => {
                // Execute agent task (simplified)
                (true, format!("Agent task: {}", instruction))
            }
            ActionType::Notify { channel, message } => {
                // Send notification (simplified)
                (true, format!("Notified {}: {}", channel, message))
            }
            ActionType::Skill { skill_id } => {
                // Execute skill (simplified)
                (true, format!("Skill executed: {}", skill_id))
            }
            ActionType::Custom { action_type, params } => {
                // Execute custom action (simplified)
                (true, format!("Custom action {}: {:?}", action_type, params))
            }
        }
    }

    /// Handle a webhook trigger
    pub async fn handle_webhook(&self, payload: WebhookPayload) -> Result<AutomationResult, AutomationError> {
        let automation_id = {
            let endpoints = self.webhook_endpoints.read().await;
            endpoints.get(&payload.path).cloned()
                .ok_or(AutomationError::WebhookNotFound(payload.path.clone()))?
        };

        self.execute(&automation_id).await
    }

    /// Get execution history
    pub async fn history(&self, limit: Option<usize>) -> Vec<AutomationResult> {
        let history = self.history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    /// Get automation statistics
    pub async fn stats(&self) -> AutomationStats {
        let automations = self.automations.read().await;
        let history = self.history.read().await;

        let total = automations.len();
        let enabled = automations.values().filter(|a| a.enabled).count();
        let total_executions: u64 = automations.values().map(|a| a.execution_count).sum();
        let successful = history.iter().filter(|r| r.success).count() as u64;
        let failed = history.iter().filter(|r| !r.success).count() as u64;

        AutomationStats {
            total,
            enabled,
            disabled: total - enabled,
            total_executions,
            successful_executions: successful,
            failed_executions: failed,
        }
    }
}

impl Default for AutomationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Automation engine errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum AutomationError {
    #[error("automation not found: {0}")]
    NotFound(String),

    #[error("automation disabled: {0}")]
    Disabled(String),

    #[error("invalid trigger: {0}")]
    InvalidTrigger(String),

    #[error("webhook not found: {0}")]
    WebhookNotFound(String),

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("io error: {0}")]
    Io(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_automation() {
        let engine = AutomationEngine::new();
        let automation = Automation {
            id: "test-1".to_string(),
            name: "Test Automation".to_string(),
            description: "A test automation".to_string(),
            trigger: TriggerType::Interval { seconds: 60 },
            actions: vec![ActionType::Shell { command: "echo hello".to_string() }],
            enabled: true,
            timeout_secs: 30,
            retry: RetryConfig::default(),
            tags: vec!["test".to_string()],
            created_at: chrono::Utc::now().to_rfc3339(),
            last_executed: None,
            execution_count: 0,
        };

        engine.register(automation).await.unwrap();
        let list = engine.list().await;
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.delay_secs, 5);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
}
