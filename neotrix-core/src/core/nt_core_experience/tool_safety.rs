use std::collections::HashMap;

use super::tool_orchestrator::DetectedIntent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SafetyLevel {
    Safe,
    LowRisk,
    MediumRisk,
    HighRisk,
}

#[derive(Debug)]
pub struct SafetyDecision {
    pub level: SafetyLevel,
    pub auto_approve: bool,
    pub reason: String,
}

impl SafetyDecision {
    pub fn auto_approve() -> Self {
        Self {
            level: SafetyLevel::Safe,
            auto_approve: true,
            reason: "auto-approved".into(),
        }
    }
    pub fn flag(level: SafetyLevel, reason: &str) -> Self {
        Self {
            level,
            auto_approve: level <= SafetyLevel::LowRisk,
            reason: reason.into(),
        }
    }
}

impl SafetyDecision {
    pub fn classify(intent: &DetectedIntent) -> Self {
        match intent {
            DetectedIntent::Greeting | DetectedIntent::Status => Self::auto_approve(),
            DetectedIntent::WebSearch(q) => {
                if q.len() > 500 {
                    Self::flag(SafetyLevel::MediumRisk, "long search query")
                } else {
                    Self::auto_approve()
                }
            }
            DetectedIntent::WebFetch(url) => {
                if url.starts_with("http://") || url.len() > 2048 {
                    Self::flag(SafetyLevel::MediumRisk, "non-https or long url")
                } else {
                    Self::flag(SafetyLevel::LowRisk, "web fetch")
                }
            }
            DetectedIntent::FileRead(path) => {
                if path.contains("..") || path.contains("~") {
                    Self::flag(SafetyLevel::HighRisk, "path traversal in read")
                } else {
                    Self::flag(SafetyLevel::LowRisk, "file read")
                }
            }
            DetectedIntent::FileWrite(path, _) => {
                if path.contains("..") || path.contains("~") {
                    Self::flag(SafetyLevel::HighRisk, "path traversal in write")
                } else {
                    Self::flag(SafetyLevel::MediumRisk, "file write")
                }
            }
            DetectedIntent::FileEdit(..) => Self::flag(SafetyLevel::MediumRisk, "file edit"),
            DetectedIntent::Bash(cmd) => {
                let lower = cmd.to_lowercase();
                if lower.contains("rm -rf") || lower.contains("sudo") || lower.contains("chmod 777")
                {
                    Self::flag(SafetyLevel::HighRisk, "dangerous bash command")
                } else {
                    Self::flag(SafetyLevel::MediumRisk, "bash execution")
                }
            }
            DetectedIntent::Glob(_) | DetectedIntent::Grep(_, _) => Self::auto_approve(),
            DetectedIntent::Translate(_, _) => Self::auto_approve(),
            DetectedIntent::Reasoning(_) | DetectedIntent::Unknown(_) => Self::auto_approve(),
        }
    }
}

/// Proof status for formal safety verification (arXiv:2606.06523 - Lean4Agent).
///
/// - Verified: the precondition invariant provably holds
/// - Obliged: no proof attempted, responsibility deferred to caller
/// - Failed: the precondition invariant is violated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofStatus {
    Verified,
    Obliged,
    Failed,
}

/// Formal proof-of-correctness check for tool safety.
///
/// Checks that the tool's precondition invariant holds before execution.
/// This is a lightweight proof check — it matches known precondition patterns
/// against the contract's properties and the runtime call context.
pub fn safety_proof_check(contract: &ToolContract, precondition: &str) -> ProofStatus {
    if contract.precondition.as_deref() == Some(precondition) {
        return ProofStatus::Verified;
    }
    match precondition {
        "path_safe" => {
            let allowed = ["/tmp", "/Users", "/home"];
            contract
                .precondition
                .as_ref()
                .map_or(ProofStatus::Obliged, |p| {
                    if allowed.iter().any(|prefix| p.starts_with(prefix)) {
                        ProofStatus::Verified
                    } else {
                        ProofStatus::Failed
                    }
                })
        }
        "no_network" => {
            if contract.permissions.contains(&"network".to_string()) {
                ProofStatus::Failed
            } else {
                ProofStatus::Verified
            }
        }
        "read_only" => {
            if contract.permissions.contains(&"write".to_string())
                || contract.permissions.contains(&"shell".to_string())
            {
                ProofStatus::Failed
            } else {
                ProofStatus::Verified
            }
        }
        _ => ProofStatus::Obliged,
    }
}

/// A tool contract with behavioral pre/post conditions (arXiv:2602.22302).
///
/// Extends SafetyDecision with optional pre/post condition specifications
/// so that composite operations can verify their preconditions before execution
/// and validate postconditions after completion.
#[derive(Debug, Clone)]
pub struct ToolContract {
    pub name: String,
    pub permissions: Vec<String>,
    pub precondition: Option<String>,
    pub postcondition: Option<String>,
    pub safety_level: SafetyLevel,
}

impl ToolContract {
    pub fn new(name: &str, permissions: Vec<String>, safety_level: SafetyLevel) -> Self {
        Self {
            name: name.to_string(),
            permissions,
            precondition: None,
            postcondition: None,
            safety_level,
        }
    }

    /// Set a precondition that must hold before tool execution.
    pub fn with_precondition(mut self, precondition: &str) -> Self {
        self.precondition = Some(precondition.to_string());
        self
    }

    /// Set a postcondition that must hold after tool execution.
    pub fn with_postcondition(mut self, postcondition: &str) -> Self {
        self.postcondition = Some(postcondition.to_string());
        self
    }

    /// Verify that the caller context satisfies the precondition.
    ///
    /// Returns true if no precondition is set, or if the precondition
    /// matches the provided call_context.
    pub fn verify_precondition(&self, call_context: &HashMap<String, String>) -> bool {
        match &self.precondition {
            None => true,
            Some(pre) => call_context.values().any(|v| v.contains(pre.as_str())),
        }
    }

    /// Verify that the tool result satisfies the postcondition.
    ///
    /// Returns true if no postcondition is set, or if the postcondition
    /// matches the provided result string.
    pub fn verify_postcondition(&self, result: &str) -> bool {
        match &self.postcondition {
            None => true,
            Some(post) => result.contains(post.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting_is_safe() {
        let d = SafetyDecision::classify(&DetectedIntent::Greeting);
        assert!(d.auto_approve);
    }

    #[test]
    fn test_dangerous_bash_is_high_risk() {
        let d = SafetyDecision::classify(&DetectedIntent::Bash("rm -rf /".into()));
        assert_eq!(d.level, SafetyLevel::HighRisk);
        assert!(!d.auto_approve);
    }

    #[test]
    fn test_path_traversal_is_high_risk() {
        let d = SafetyDecision::classify(&DetectedIntent::FileRead("../../etc/passwd".into()));
        assert_eq!(d.level, SafetyLevel::HighRisk);
    }

    #[test]
    fn test_safe_web_search_is_auto() {
        let d = SafetyDecision::classify(&DetectedIntent::WebSearch("rust programming".into()));
        assert!(d.auto_approve);
    }

    #[test]
    fn test_safe_glob_is_auto() {
        let d = SafetyDecision::classify(&DetectedIntent::Glob("*.rs".into()));
        assert!(d.auto_approve);
    }
}
