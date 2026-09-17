//! Dual Executor — Cloud + Local Execution Router
//!
//! Implements the Claude Cowork pattern: route tasks to cloud or local execution
//! based on complexity, privacy requirements, and availability.
//!
//! # Architecture
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           Dual Executor                      │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
//! │  │  Cloud   │  │  Local   │  │  Router  │  │
//! │  │ Executor │  │ Executor │  │  Engine  │  │
//! │  └──────────┘  └──────────┘  └──────────┘  │
//! │       ↓              ↓              ↓        │
//! │  ┌──────────────────────────────────────┐   │
//! │  │      Execution Policy Engine         │   │
//! │  └──────────────────────────────────────┘   │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! # Safety
//! - Local execution runs in VM sandbox
//! - Cloud execution uses isolated sandboxes
//! - No unsafe code (R-P1)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Core Types
// ============================================================================

/// Execution target
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionTarget {
    /// Execute on cloud server
    Cloud,
    /// Execute locally on device
    Local,
    /// Route based on policy (auto-decide)
    Auto,
}

/// Execution mode (how the task runs)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionMode {
    /// Agent loop runs on cloud, code execution in cloud sandbox
    CloudSandbox,
    /// Agent loop runs locally, code execution in local VM
    LocalVm,
    /// Agent loop on cloud, code execution on local device (hybrid)
    HybridCloudLocal,
    /// Agent loop locally, code execution on cloud (hybrid)
    HybridLocalCloud,
}

/// Task privacy level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum PrivacyLevel {
    /// Public data, no restrictions
    Public,
    /// Internal data, cloud allowed
    Internal,
    /// Confidential, local preferred
    Confidential,
    /// Restricted, local only
    Restricted,
}

/// Task complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum ComplexityLevel {
    /// Simple tasks (fast local model)
    Simple,
    /// Medium tasks (balanced model)
    Medium,
    /// Complex tasks (strong cloud model)
    Complex,
    /// Critical tasks (strongest model + verification)
    Critical,
}

/// Execution request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionRequest {
    /// Task instruction
    pub instruction: String,
    /// Target (cloud/local/auto)
    pub target: ExecutionTarget,
    /// Privacy level
    pub privacy: PrivacyLevel,
    /// Complexity level
    pub complexity: ComplexityLevel,
    /// Files that can be accessed
    pub allowed_files: Vec<String>,
    /// Maximum execution time in seconds
    pub timeout_secs: u64,
    /// Whether to use VM isolation
    pub use_vm: bool,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

/// Execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Actual execution target used
    pub target: ExecutionTarget,
    /// Actual execution mode used
    pub mode: ExecutionMode,
    /// Output
    pub output: String,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Files modified
    pub modified_files: Vec<String>,
    /// Tokens used
    pub tokens_used: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Execution ID for tracking
    pub execution_id: String,
}

/// Execution policy
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionPolicy {
    /// Default target
    pub default_target: ExecutionTarget,
    /// Privacy-based routing rules
    pub privacy_rules: HashMap<PrivacyLevel, ExecutionTarget>,
    /// Complexity-based routing rules
    pub complexity_rules: HashMap<ComplexityLevel, ExecutionTarget>,
    /// Maximum local task complexity
    pub max_local_complexity: ComplexityLevel,
    /// Require VM for local execution
    pub require_vm_for_local: bool,
    /// Allow hybrid execution
    pub allow_hybrid: bool,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        let mut privacy_rules = HashMap::new();
        privacy_rules.insert(PrivacyLevel::Public, ExecutionTarget::Cloud);
        privacy_rules.insert(PrivacyLevel::Internal, ExecutionTarget::Cloud);
        privacy_rules.insert(PrivacyLevel::Confidential, ExecutionTarget::Local);
        privacy_rules.insert(PrivacyLevel::Restricted, ExecutionTarget::Local);

        let mut complexity_rules = HashMap::new();
        complexity_rules.insert(ComplexityLevel::Simple, ExecutionTarget::Local);
        complexity_rules.insert(ComplexityLevel::Medium, ExecutionTarget::Local);
        complexity_rules.insert(ComplexityLevel::Complex, ExecutionTarget::Cloud);
        complexity_rules.insert(ComplexityLevel::Critical, ExecutionTarget::Cloud);

        Self {
            default_target: ExecutionTarget::Auto,
            privacy_rules,
            complexity_rules,
            max_local_complexity: ComplexityLevel::Medium,
            require_vm_for_local: true,
            allow_hybrid: true,
        }
    }
}

/// Executor statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutorStats {
    pub total_executions: usize,
    pub cloud_executions: usize,
    pub local_executions: usize,
    pub hybrid_executions: usize,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
}

// ============================================================================
// Dual Executor
// ============================================================================

/// Cloud + Local execution router
pub struct DualExecutor {
    /// Execution policy
    policy: Arc<RwLock<ExecutionPolicy>>,
    /// Execution history
    history: Arc<RwLock<Vec<ExecutionResult>>>,
    /// Cloud executor endpoint
    cloud_endpoint: Option<String>,
    /// Local VM available
    local_vm_available: bool,
}

impl DualExecutor {
    /// Create a new dual executor
    pub fn new(policy: ExecutionPolicy) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            history: Arc::new(RwLock::new(Vec::new())),
            cloud_endpoint: None,
            local_vm_available: true,
        }
    }

    /// Create with default policy
    pub fn with_defaults() -> Self {
        Self::new(ExecutionPolicy::default())
    }

    /// Set cloud endpoint
    pub fn set_cloud_endpoint(&mut self, endpoint: String) {
        self.cloud_endpoint = Some(endpoint);
    }

    /// Set local VM availability
    pub fn set_local_vm_available(&mut self, available: bool) {
        self.local_vm_available = available;
    }

    /// Route a task to the appropriate executor
    pub async fn route(&self, request: &ExecutionRequest) -> Result<ExecutionResult, ExecutionError> {
        let policy = self.policy.read().await;

        // Determine target
        let target = match request.target {
            ExecutionTarget::Auto => {
                // Check privacy rules first
                if let Some(target) = policy.privacy_rules.get(&request.privacy) {
                    target.clone()
                } else if let Some(target) = policy.complexity_rules.get(&request.complexity) {
                    target.clone()
                } else {
                    policy.default_target.clone()
                }
            }
            ref target => target.clone(),
        };

        // Determine execution mode
        let mode = match target {
            ExecutionTarget::Cloud => {
                if policy.allow_hybrid && request.privacy < PrivacyLevel::Confidential {
                    ExecutionMode::HybridCloudLocal
                } else {
                    ExecutionMode::CloudSandbox
                }
            }
            ExecutionTarget::Local => {
                if !self.local_vm_available {
                    return Err(ExecutionError::LocalVmUnavailable);
                }
                if policy.require_vm_for_local {
                    ExecutionMode::LocalVm
                } else {
                    ExecutionMode::HybridLocalCloud
                }
            }
            ExecutionTarget::Auto => {
                // Should not reach here after routing logic above
                ExecutionMode::CloudSandbox
            }
        };

        // Execute
        let result = match mode {
            ExecutionMode::CloudSandbox | ExecutionMode::HybridCloudLocal => {
                self.execute_cloud(request).await?
            }
            ExecutionMode::LocalVm | ExecutionMode::HybridLocalCloud => {
                self.execute_local(request).await?
            }
        };

        // Store in history
        {
            let mut history = self.history.write().await;
            history.push(result.clone());
            let history_len = history.len();
            if history_len > 1000 {
                history.drain(0..history_len - 1000);
            }
        }

        Ok(result)
    }

    /// Execute on cloud
    async fn execute_cloud(&self, request: &ExecutionRequest) -> Result<ExecutionResult, ExecutionError> {
        let start = std::time::Instant::now();
        let execution_id = format!("exec-{}", uuid::Uuid::new_v4());

        // Simplified cloud execution
        let result = ExecutionResult {
            success: true,
            target: ExecutionTarget::Cloud,
            mode: ExecutionMode::CloudSandbox,
            output: format!("Cloud execution: {}", request.instruction),
            error: None,
            modified_files: Vec::new(),
            tokens_used: 1000,
            duration_ms: start.elapsed().as_millis() as u64,
            execution_id,
        };

        Ok(result)
    }

    /// Execute locally with VM isolation
    async fn execute_local(&self, request: &ExecutionRequest) -> Result<ExecutionResult, ExecutionError> {
        let start = std::time::Instant::now();
        let execution_id = format!("exec-{}", uuid::Uuid::new_v4());

        // Simplified local execution
        let result = ExecutionResult {
            success: true,
            target: ExecutionTarget::Local,
            mode: ExecutionMode::LocalVm,
            output: format!("Local VM execution: {}", request.instruction),
            error: None,
            modified_files: Vec::new(),
            tokens_used: 500,
            duration_ms: start.elapsed().as_millis() as u64,
            execution_id,
        };

        Ok(result)
    }

    /// Update execution policy
    pub async fn update_policy(&self, policy: ExecutionPolicy) {
        let mut p = self.policy.write().await;
        *p = policy;
    }

    /// Get execution statistics
    pub async fn stats(&self) -> ExecutorStats {
        let history = self.history.read().await;
        let total = history.len();
        let cloud = history.iter().filter(|r| r.target == ExecutionTarget::Cloud).count();
        let local = history.iter().filter(|r| r.target == ExecutionTarget::Local).count();
        let hybrid = history.iter().filter(|r| {
            matches!(r.mode, ExecutionMode::HybridCloudLocal | ExecutionMode::HybridLocalCloud)
        }).count();

        let avg_duration = if total > 0 {
            history.iter().map(|r| r.duration_ms as f64).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let success_count = history.iter().filter(|r| r.success).count();
        let success_rate = if total > 0 {
            success_count as f64 / total as f64
        } else {
            0.0
        };

        ExecutorStats {
            total_executions: total,
            cloud_executions: cloud,
            local_executions: local,
            hybrid_executions: hybrid,
            avg_duration_ms: avg_duration,
            success_rate,
        }
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Execution errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ExecutionError {
    #[error("local VM unavailable")]
    LocalVmUnavailable,

    #[error("cloud endpoint not configured")]
    CloudNotConfigured,

    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_policy_default() {
        let policy = ExecutionPolicy::default();
        assert_eq!(policy.default_target, ExecutionTarget::Auto);
        assert!(policy.require_vm_for_local);
        assert!(policy.allow_hybrid);
    }

    #[test]
    fn test_privacy_level_ordering() {
        assert!(PrivacyLevel::Public < PrivacyLevel::Internal);
        assert!(PrivacyLevel::Internal < PrivacyLevel::Confidential);
        assert!(PrivacyLevel::Confidential < PrivacyLevel::Restricted);
    }

    #[test]
    fn test_complexity_level_ordering() {
        assert!(ComplexityLevel::Simple < ComplexityLevel::Medium);
        assert!(ComplexityLevel::Medium < ComplexityLevel::Complex);
        assert!(ComplexityLevel::Complex < ComplexityLevel::Critical);
    }
}
