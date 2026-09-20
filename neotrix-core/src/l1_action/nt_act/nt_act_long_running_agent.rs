//! # Long-running Agent Support
//!
//! Inspired by Grok 4.6's sustained multi-step work capabilities and
//! GPT-5.6's agentic harness for managing context bloat and repeated work.
//!
//! ## Design Principles
//! - Sustained execution: Agents can work for minutes/hours
//! - Context management: Prevent bloat in long sessions
//! - Self-verification: Models check their own work
//! - Checkpoint/restore: Save state for recovery
//! - Progress tracking: Real-time progress updates

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Agent task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentTaskStatus {
    /// Task is queued
    Queued,
    /// Task is running
    Running,
    /// Task is paused (user requested)
    Paused,
    /// Task is checkpointed (for resume)
    Checkpointed,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed { error: String },
    /// Task was cancelled
    Cancelled,
}

/// Long-running agent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongRunningAgentTask {
    pub id: String,
    pub description: String,
    pub status: AgentTaskStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub progress: f32, // 0.0 - 1.0
    pub steps_completed: u32,
    pub steps_total: Option<u32>,
    pub current_step: Option<String>,
    pub result: Option<AgentOutput>,
    pub error: Option<String>,
    pub checkpoints: Vec<Checkpoint>,
    pub context: AgentContext,
    pub config: AgentConfig,
}

/// Agent output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub content: String,
    pub artifacts: Vec<Artifact>,
    pub metrics: AgentMetrics,
    pub self_verification: Option<VerificationResult>,
}

/// Agent artifact (code, documents, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub artifact_type: ArtifactType,
    pub content: String,
    pub created_at: String,
    pub version: u32,
}

/// Artifact type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Code,
    Document,
    Data,
    Image,
    Report,
    Plan,
}

/// Agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tokens_used: u64,
    pub cost_incurred: f64,
    pub execution_time_ms: u64,
    pub steps_completed: u32,
    pub self_corrections: u32,
    pub tool_calls: u32,
}

/// Verification result (Grok 4.6 self-testing pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub checks: Vec<VerificationCheck>,
    pub confidence: f32,
    pub issues_found: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Individual verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub details: String,
    pub severity: CheckSeverity,
}

/// Check severity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CheckSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Agent checkpoint for resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub step: u32,
    pub timestamp: String,
    pub state: HashMap<String, serde_json::Value>,
    pub context_snapshot: ContextSnapshot,
}

/// Context snapshot at checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub messages: Vec<ContextMessage>,
    pub tokens_used: u64,
    pub summary: Option<String>,
}

/// Context message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
    pub tokens: u32,
    pub timestamp: String,
}

/// Agent context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub messages: Vec<ContextMessage>,
    pub total_tokens: u64,
    pub max_tokens: u64,
    pub summary: Option<String>,
    pub key_facts: Vec<String>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_steps: Option<u32>,
    pub max_tokens: u64,
    pub max_cost: f64,
    pub timeout_ms: u64,
    pub checkpoint_interval: u32, // Checkpoint every N steps
    pub self_verify: bool,
    pub auto_continue: bool,
}

/// Context bloat management (GPT-5.6 pattern)
pub struct ContextBloatManager {
    max_context_tokens: u64,
    compaction_threshold: f64, // Trigger compaction at this % full
}

impl ContextBloatManager {
    pub fn new(max_context_tokens: u64) -> Self {
        Self {
            max_context_tokens,
            compaction_threshold: 0.8, // 80% full triggers compaction
        }
    }

    /// Check if context needs compaction
    pub fn needs_compaction(&self, context: &AgentContext) -> bool {
        let usage_ratio = context.total_tokens as f64 / self.max_context_tokens as f64;
        usage_ratio >= self.compaction_threshold
    }

    /// Compact context by summarizing old messages
    pub fn compact_context(&mut self, context: &mut AgentContext) -> CompactionResult {
        let original_tokens = context.total_tokens;
        let messages = &context.messages;

        if messages.len() < 10 {
            return CompactionResult {
                tokens_before: original_tokens,
                tokens_after: original_tokens,
                messages_removed: 0,
                summary: None,
            };
        }

        // Keep last 5 messages intact, summarize the rest
        let split_point = messages.len() - 5;
        let old_messages = &messages[..split_point];
        let new_messages = messages[split_point..].to_vec();

        // Create summary of old messages
        let summary = self.summarize_messages(old_messages);

        // Update context
        context.messages = new_messages;
        context.summary = Some(summary.clone());
        context.total_tokens = context.messages.iter().map(|m| m.tokens as u64).sum();

        let _tokens_removed = original_tokens - context.total_tokens;

        CompactionResult {
            tokens_before: original_tokens,
            tokens_after: context.total_tokens,
            messages_removed: split_point as u32,
            summary: Some(summary),
        }
    }

    /// Summarize messages (placeholder - in production, use LLM)
    fn summarize_messages(&self, messages: &[ContextMessage]) -> String {
        let user_messages: Vec<&str> = messages
            .iter()
            .filter(|m| m.role == "user")
            .map(|m| m.content.as_str())
            .collect();

        format!(
            "Previous conversation covered {} messages discussing: {}",
            messages.len(),
            user_messages.join("; ")
        )
    }
}

/// Compaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    pub tokens_before: u64,
    pub tokens_after: u64,
    pub messages_removed: u32,
    pub summary: Option<String>,
}

/// Long-running Agent Manager
pub struct LongRunningAgentManager {
    tasks: Arc<Mutex<HashMap<String, LongRunningAgentTask>>>,
}

impl LongRunningAgentManager {
    pub fn new(max_context_tokens: u64) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a new long-running agent task
    pub fn start_task(
        &self,
        description: String,
        config: AgentConfig,
    ) -> Result<LongRunningAgentTask, AgentError> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let task = LongRunningAgentTask {
            id: task_id.clone(),
            description,
            status: AgentTaskStatus::Queued,
            created_at: now.clone(),
            started_at: None,
            completed_at: None,
            progress: 0.0,
            steps_completed: 0,
            steps_total: None,
            current_step: None,
            result: None,
            error: None,
            checkpoints: Vec::new(),
            context: AgentContext {
                messages: Vec::new(),
                total_tokens: 0,
                max_tokens: config.max_tokens,
                summary: None,
                key_facts: Vec::new(),
            },
            config,
        };

        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| AgentError::LockError(e.to_string()))?;
        tasks.insert(task_id.clone(), task.clone());

        Ok(task)
    }

    /// Get task status
    pub fn get_task(&self, task_id: &str) -> Result<LongRunningAgentTask, AgentError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|e| AgentError::LockError(e.to_string()))?;
        tasks
            .get(task_id)
            .cloned()
            .ok_or_else(|| AgentError::TaskNotFound(task_id.to_string()))
    }

    /// Update task progress
    pub fn update_progress(
        &self,
        task_id: &str,
        progress: f32,
        step: Option<String>,
    ) -> Result<(), AgentError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| AgentError::LockError(e.to_string()))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentError::TaskNotFound(task_id.to_string()))?;

        task.progress = progress;
        task.current_step = step;
        task.steps_completed += 1;

        Ok(())
    }

    /// Create checkpoint
    pub fn create_checkpoint(&self, task_id: &str) -> Result<Checkpoint, AgentError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| AgentError::LockError(e.to_string()))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentError::TaskNotFound(task_id.to_string()))?;

        let checkpoint = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            step: task.steps_completed,
            timestamp: chrono::Utc::now().to_rfc3339(),
            state: HashMap::new(),
            context_snapshot: ContextSnapshot {
                messages: task.context.messages.clone(),
                tokens_used: task.context.total_tokens,
                summary: task.context.summary.clone(),
            },
        };

        task.checkpoints.push(checkpoint.clone());
        task.status = AgentTaskStatus::Checkpointed;

        Ok(checkpoint)
    }

    /// Self-verify agent output (Grok 4.6 pattern)
    pub fn self_verify(&self, output: &AgentOutput) -> VerificationResult {
        let mut checks = Vec::new();
        let mut issues = Vec::new();

        // Check 1: Content is not empty
        let content_not_empty = !output.content.is_empty();
        checks.push(VerificationCheck {
            name: "content_not_empty".into(),
            passed: content_not_empty,
            details: if content_not_empty {
                "Output contains content".into()
            } else {
                "Output is empty".into()
            },
            severity: if content_not_empty {
                CheckSeverity::Info
            } else {
                CheckSeverity::Error
            },
        });
        if !content_not_empty {
            issues.push("Output is empty".into());
        }

        // Check 2: No obvious errors in code
        let no_errors =
            !output.content.contains("error") || output.content.contains("// error handled");
        checks.push(VerificationCheck {
            name: "no_obvious_errors".into(),
            passed: no_errors,
            details: if no_errors {
                "No obvious error patterns found".into()
            } else {
                "Potential error patterns detected".into()
            },
            severity: if no_errors {
                CheckSeverity::Info
            } else {
                CheckSeverity::Warning
            },
        });

        // Check 3: Metrics are reasonable
        let metrics_ok = output.metrics.tokens_used > 0 && output.metrics.execution_time_ms > 0;
        checks.push(VerificationCheck {
            name: "metrics_reasonable".into(),
            passed: metrics_ok,
            details: format!(
                "Tokens: {}, Time: {}ms",
                output.metrics.tokens_used, output.metrics.execution_time_ms
            ),
            severity: if metrics_ok {
                CheckSeverity::Info
            } else {
                CheckSeverity::Warning
            },
        });

        let passed = checks
            .iter()
            .all(|c| c.passed || c.severity != CheckSeverity::Error);
        let confidence = checks.iter().filter(|c| c.passed).count() as f32 / checks.len() as f32;

        VerificationResult {
            passed,
            checks,
            confidence,
            issues_found: issues,
            suggestions: Vec::new(),
        }
    }

    /// Get all tasks
    pub fn list_tasks(&self) -> Result<Vec<LongRunningAgentTask>, AgentError> {
        let tasks = self
            .tasks
            .lock()
            .map_err(|e| AgentError::LockError(e.to_string()))?;
        Ok(tasks.values().cloned().collect())
    }

    /// Cancel task
    pub fn cancel_task(&self, task_id: &str) -> Result<(), AgentError> {
        let mut tasks = self
            .tasks
            .lock()
            .map_err(|e| AgentError::LockError(e.to_string()))?;
        let task = tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentError::TaskNotFound(task_id.to_string()))?;

        task.status = AgentTaskStatus::Cancelled;
        Ok(())
    }
}

/// Agent errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentError {
    TaskNotFound(String),
    LockError(String),
    InvalidState(String),
    CheckpointError(String),
    VerificationError(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            AgentError::LockError(e) => write!(f, "Lock error: {}", e),
            AgentError::InvalidState(s) => write!(f, "Invalid state: {}", s),
            AgentError::CheckpointError(e) => write!(f, "Checkpoint error: {}", e),
            AgentError::VerificationError(e) => write!(f, "Verification error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_task() {
        let manager = LongRunningAgentManager::new(128000);
        let config = AgentConfig {
            max_steps: Some(100),
            max_tokens: 128000,
            max_cost: 10.0,
            timeout_ms: 3600000,
            checkpoint_interval: 10,
            self_verify: true,
            auto_continue: true,
        };

        let task = manager.start_task("Test task".into(), config).unwrap();
        assert_eq!(task.status, AgentTaskStatus::Queued);
    }

    #[test]
    fn test_context_compaction() {
        let mut manager = ContextBloatManager::new(128000);
        let mut context = AgentContext {
            messages: (0..20)
                .map(|i| ContextMessage {
                    role: if i % 2 == 0 {
                        "user".into()
                    } else {
                        "assistant".into()
                    },
                    content: format!("Message {}", i),
                    tokens: 100,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
                .collect(),
            total_tokens: 2000,
            max_tokens: 128000,
            summary: None,
            key_facts: Vec::new(),
        };

        let result = manager.compact_context(&mut context);
        assert!(result.messages_removed > 0);
        assert!(result.summary.is_some());
    }

    #[test]
    fn test_self_verify() {
        let manager = LongRunningAgentManager::new(128000);
        let output = AgentOutput {
            content: "Hello, world!".into(),
            artifacts: Vec::new(),
            metrics: AgentMetrics {
                tokens_used: 100,
                cost_incurred: 0.001,
                execution_time_ms: 500,
                steps_completed: 1,
                self_corrections: 0,
                tool_calls: 0,
            },
            self_verification: None,
        };

        let result = manager.self_verify(&output);
        assert!(result.passed);
        assert!(result.confidence > 0.8);
    }
}
