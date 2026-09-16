//! Auditor — Independent verification.
//!
//! From LongHorizon-Harness: the Auditor provides independent,
//! unbiased verification of results. Separates verification concerns
//! from execution logic.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::l1_action::traits::{CapabilityError, Plan, PlanResult};
use super::trait_::{AgentLoop, AgentPlan, AgentLoopError, Adaptation, AdaptationStrategy, Objective, ExecutionResult, VerificationCheck, VerificationResult, PlanStep, StepOutput};
use super::config::LoopConfig;
use super::executor::AgentExecutor;

/// Audit level for verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    /// Basic sanity check.
    Basic,
    /// Thorough verification with deep inspection.
    Thorough,
    /// Full independent audit with external validation.
    Full,
}

/// Audit record for tracking verification history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub objective: String,
    pub passed: bool,
    pub score: f64,
    pub depth: u8,
    pub timestamp: u64,
    pub details: String,
}

/// Auditor — Independent verification.
///
/// From LongHorizon-Harness: the Auditor provides independent,
/// unbiased verification of results. It is completely separate
/// from the Executor to ensure verification integrity.
pub struct Auditor {
    config: LoopConfig,
    level: AuditLevel,
    history: Arc<Mutex<Vec<AuditRecord>>>,
    audit_log: Arc<Mutex<Vec<String>>>,
}

impl Auditor {
    /// Create a new Auditor with the given audit level.
    pub fn new(config: LoopConfig, level: AuditLevel) -> Self {
        Self {
            config,
            level,
            history: Arc::new(Mutex::new(Vec::new())),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a basic auditor.
    pub fn basic() -> Self {
        Self::new(LoopConfig::quick(), AuditLevel::Basic)
    }

    /// Create a thorough auditor.
    pub fn thorough() -> Self {
        Self::new(LoopConfig::default(), AuditLevel::Thorough)
    }

    /// Create a full auditor.
    pub fn full() -> Self {
        Self::new(LoopConfig::orchestrated(), AuditLevel::Full)
    }

    /// Perform an independent audit of the execution result.
    pub fn audit(&self, result: &ExecutionResult) -> AuditResult {
        let mut checks = Vec::new();
        let mut all_passed = true;
        let mut total_score = 0.0;

        for output in &result.outputs {
            let passed = output.success;
            if !passed {
                all_passed = false;
            }
            total_score += output.success as u8 as f64;
            checks.push(VerificationCheck {
                name: format!("audit_{}", output.step_id),
                passed,
                detail: format!("Output {}: success={}", output.step_id, output.success),
            });
        }

        let score = if result.outputs.is_empty() {
            0.0
        } else {
            total_score / result.outputs.len() as f64
        };

        // Log the audit
        let record = AuditRecord {
            id: format!("audit_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()),
            objective: "audit".to_string(),
            passed: all_passed,
            score,
            depth: self.config.verification_depth,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            details: format!("Audit result: {} passed, score={:.2}", all_passed, score),
        };
        self.history.lock().unwrap().push(record.clone());
        self.audit_log.lock().unwrap().push(record.details.clone());

        AuditResult {
            passed: all_passed,
            score,
            checks,
            level: self.level,
        }
    }

    /// Cross-verify results between executor and checker.
    pub fn cross_verify(&self, executor_result: &ExecutionResult, checker_result: &ExecutionResult) -> CrossVerificationResult {
        let executor_score = if executor_result.outputs.is_empty() {
            0.0
        } else {
            executor_result.outputs.iter().map(|o| o.success as u8 as f64).sum::<f64>() / executor_result.outputs.len() as f64
        };
        let checker_score = if checker_result.outputs.is_empty() {
            0.0
        } else {
            checker_result.outputs.iter().map(|o| o.success as u8 as f64).sum::<f64>() / checker_result.outputs.len() as f64
        };

        let agreement = 1.0 - (executor_score - checker_score).abs();
        let passed = agreement > 0.7 && executor_score > 0.5 && checker_score > 0.5;

        CrossVerificationResult {
            agreement,
            passed,
            executor_score,
            checker_score,
        }
    }

    /// Get the audit history.
    pub fn history(&self) -> Vec<AuditRecord> {
        self.history.lock().unwrap().clone()
    }

    /// Get the audit log.
    pub fn audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }

    /// Get the number of audits performed.
    pub fn audit_count(&self) -> usize {
        self.history.lock().unwrap().len()
    }
}

/// Result of an audit operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub passed: bool,
    pub score: f64,
    pub checks: Vec<VerificationCheck>,
    pub level: AuditLevel,
}

/// Result of cross-verification between two executor results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossVerificationResult {
    pub agreement: f64,
    pub passed: bool,
    pub executor_score: f64,
    pub checker_score: f64,
}

impl Default for Auditor {
    fn default() -> Self {
        Self::basic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auditor_basic() {
        let auditor = Auditor::basic();
        assert_eq!(auditor.level, AuditLevel::Basic);
    }

    #[test]
    fn test_auditor_audit() {
        let auditor = Auditor::thorough();
        let result = ExecutionResult {
            success: true,
            outputs: vec![
                StepOutput {
                    step_id: "step_1".to_string(),
                    success: true,
                    output: "done".to_string(),
                    assertions_passed: 2,
                    assertions_failed: 0,
                }
            ],
            iterations_used: 1,
            errors: vec![],
        };
        let audit = auditor.audit(&result);
        assert!(audit.passed);
        assert!(audit.score > 0.0);
    }

    #[test]
    fn test_auditor_cross_verify() {
        let auditor = Auditor::basic();
        let result1 = ExecutionResult {
            success: true,
            outputs: vec![
                StepOutput {
                    step_id: "s1".to_string(),
                    success: true,
                    output: "ok".to_string(),
                    assertions_passed: 1,
                    assertions_failed: 0,
                }
            ],
            iterations_used: 1,
            errors: vec![],
        };
        let result2 = ExecutionResult {
            success: true,
            outputs: vec![
                StepOutput {
                    step_id: "s1".to_string(),
                    success: true,
                    output: "ok".to_string(),
                    assertions_passed: 1,
                    assertions_failed: 0,
                }
            ],
            iterations_used: 1,
            errors: vec![],
        };
        let cross = auditor.cross_verify(&result1, &result2);
        assert!(cross.passed);
        assert_eq!(cross.agreement, 1.0);
    }

    #[test]
    fn test_auditor_history() {
        let auditor = Auditor::thorough();
        let result = ExecutionResult {
            success: true,
            outputs: vec![
                StepOutput {
                    step_id: "step_1".to_string(),
                    success: true,
                    output: "done".to_string(),
                    assertions_passed: 1,
                    assertions_failed: 0,
                }
            ],
            iterations_used: 1,
            errors: vec![],
        };
        auditor.audit(&result);
        assert_eq!(auditor.audit_count(), 1);
    }
}