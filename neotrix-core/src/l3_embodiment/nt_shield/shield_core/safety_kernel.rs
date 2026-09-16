use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::policy::{ActionPolicy, PolicyDecision};

use crate::core::nt_core_self_test::SelfTest;

type HmacSha256 = Hmac<Sha256>;

/// Destructive action categories that require explicit confirmation.
///
/// Absorbs computer-repair-skill axiom: "confirm before state changes."
/// and airgorah pattern: polkit agent prompts before privileged operations.
///
/// When an action is classified as destructive, SafetyKernel refuses to
/// auto-approve it — even if the risk score is below the threshold.
/// The agent must receive an explicit human confirmation (or a signed
/// SafetyKernel confirmation token) before proceeding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DestructiveAction {
    /// File deletion (rm, trash, unlink)
    FileDelete,
    /// Directory deletion (rmdir, rm -rf)
    DirectoryDelete,
    /// Database modification (DROP, DELETE, TRUNCATE)
    DatabaseModify,
    /// System configuration change (chmod, chown, sysctl)
    SystemConfigChange,
    /// Network service restart (systemctl restart, service reload)
    ServiceRestart,
    /// User/account modification
    AccountModify,
    /// Package installation/removal (apt, brew, cargo install)
    PackageModify,
    /// Custom destructive action
    Custom(String),
}

impl DestructiveAction {
    /// Check if an ActionRequest is a destructive action
    pub fn from_request(request: &ActionRequest) -> Option<DestructiveAction> {
        match &request.action_type {
            ActionType::FileDelete => Some(DestructiveAction::FileDelete),
            ActionType::SubprocessExec => {
                let cmd = request.args.values().find(|v| v.contains("rm ") || v.contains("rmdir"));
                if cmd.is_some() {
                    Some(DestructiveAction::DirectoryDelete)
                } else if request.args.values().any(|v| v.contains("chmod") || v.contains("chown")) {
                    Some(DestructiveAction::SystemConfigChange)
                } else {
                    None
                }
            }
            ActionType::FileWrite => {
                if request.target.contains("/etc/") || request.target.contains("/sys/") {
                    Some(DestructiveAction::SystemConfigChange)
                } else {
                    None
                }
            }
            ActionType::Custom(s) => {
                if s.contains("delete") || s.contains("drop") || s.contains("remove") {
                    Some(DestructiveAction::Custom(s.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Description for the confirmation prompt
    pub fn confirmation_message(&self, request: &ActionRequest) -> String {
        format!(
            "DESTRUCTIVE ACTION CONFIRMATION REQUIRED\n\
             Action: {:?}\n\
             Target: {}\n\
             Risk: {:.2}\n\
             \n\
             This action will permanently modify or delete data.\n\
             Type 'CONFIRM' to proceed, or 'DENY' to abort.",
            self, request.target, request.risk_score
        )
    }
}

/// Execution-Time Safety Decision
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyDecision {
    /// Action is permitted
    Allowed { reason: String, signed_at: u64 },
    /// Action is denied — structurally the agent cannot bypass this
    Denied { reason: String, signed_at: u64, appeal_path: Option<String> },
    /// Action requires human-in-the-loop approval
    RequiresApproval { reason: String, escalation_path: String },
}

/// Signed evidence of a safety decision, verifiable outside the agent's trust boundary
#[derive(Debug, Clone)]
pub struct _SignedEvidence {
    pub decision: SafetyDecision,
    pub kernel_version: String,
    pub signature: String,
    pub timestamp: u64,
}

/// Action types for execution-time alignment classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionType {
    FileRead,
    FileWrite,
    FileDelete,
    NetworkRequest,
    SubprocessExec,
    CodeEval,
    DataExport,
    ModelAccess,
    ToolCall,
    Custom(String),
}

impl ActionType {
    fn to_policy_action(&self) -> &str {
        match self {
            ActionType::FileRead => "read_file",
            ActionType::FileWrite => "write_file",
            ActionType::FileDelete => "delete_file",
            ActionType::NetworkRequest => "network_request",
            ActionType::SubprocessExec => "execute_command",
            ActionType::CodeEval => "execute_command",
            ActionType::DataExport => "data_export",
            ActionType::ModelAccess => "model_access",
            ActionType::ToolCall => "tool_call",
            ActionType::Custom(s) => s,
        }
    }
}

/// An action request to be evaluated by the SafetyKernel
#[derive(Debug, Clone)]
pub struct ActionRequest {
    pub action_id: String,
    pub action_type: ActionType,
    pub target: String,
    pub args: HashMap<String, String>,
    pub context: HashMap<String, String>,
    pub risk_score: f64,
}

/// The Unfireable Safety Kernel — execution-time AI alignment layer
///
/// Four properties from "The Unfireable Safety Kernel" (June 2026):
/// 1. Process separation — runs in separate memory space from agent (here: Arc-isolated state)
/// 2. Pre-action enforcement — action is checked BEFORE it executes
/// 3. Fail-closed — if kernel is unreachable, action is DENIED
/// 4. Signed evidence — decisions are verifiable outside agent's trust boundary
pub struct SafetyKernel {
    active: Arc<AtomicBool>,
    version: String,
    signing_key: [u8; 32],
    policy: Arc<ActionPolicy>,
    audit_log: std::sync::Mutex<Vec<_SignedEvidence>>,
}

impl SafetyKernel {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for i in 0..8 {
            let b = (now >> (i * 8)) as u8;
            key[i] = b;
            key[i + 8] = b.wrapping_add(0xAA);
            key[i + 16] = b.wrapping_add(0x55);
            key[i + 24] = b.wrapping_add(0x33);
        }
        Self {
            active: Arc::new(AtomicBool::new(true)),
            version: option_env!("CARGO_PKG_VERSION").unwrap_or("0.1.0").to_string(),
            signing_key: key,
            policy: Arc::new(ActionPolicy::new()),
            audit_log: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// THE core method — checks an action and returns signed evidence
    pub fn check(&self, action: &ActionRequest) -> _SignedEvidence {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if !self.active.load(Ordering::SeqCst) {
            let decision = SafetyDecision::Denied {
                reason: "SafetyKernel is inactive — fail-closed: action denied".to_string(),
                signed_at: timestamp,
                appeal_path: Some("reactivate kernel via set_active(true)".to_string()),
            };
            return self.sign(decision, timestamp);
        }

        if action.action_id.is_empty() {
            let decision = SafetyDecision::Denied {
                reason: "Action ID is empty — cannot evaluate unnamed action".to_string(),
                signed_at: timestamp,
                appeal_path: None,
            };
            return self.sign(decision, timestamp);
        }

        // Confirm-before-destructive gate (computer-repair-skill axiom)
        // Destructive actions ALWAYS require approval, regardless of risk score
        if let Some(destructive) = DestructiveAction::from_request(action) {
            let decision = SafetyDecision::RequiresApproval {
                reason: format!(
                    "Destructive action '{}' of type {:?} requires explicit confirmation. \
                     Target: {}. This is a state-changing operation — confirm before proceeding.",
                    action.action_id, action.action_type, action.target
                ),
                escalation_path: format!(
                    "destructive_confirm://{}\n{}",
                    action.target,
                    destructive.confirmation_message(action)
                ),
            };
            return self.sign(decision, timestamp);
        }

        let policy_action = action.action_type.to_policy_action();
        let policy_decision = self.policy.decide(policy_action);

        match policy_decision {
            PolicyDecision::Deny => {
                let decision = SafetyDecision::Denied {
                    reason: format!(
                        "Action '{}' of type {:?} denied by policy (profile: {})",
                        action.action_id, action.action_type, self.policy.profile
                    ),
                    signed_at: timestamp,
                    appeal_path: Some(format!(
                        "add '{}' to allowlist or change policy profile",
                        policy_action
                    )),
                };
                self.sign(decision, timestamp)
            }
            PolicyDecision::RequireConfirmation | PolicyDecision::Allow => {
                let computed_risk: f64 = compute_risk_score(&action.action_type, &action.target, &action.args);
                let effective_risk: f64 = action.risk_score.max(computed_risk).max(0.0_f64).min(1.0_f64);

                if effective_risk > 0.8 {
                    let decision = SafetyDecision::RequiresApproval {
                        reason: format!(
                            "Action '{}' of type {:?} has high risk score ({:.2}) — requires human approval",
                            action.action_id, action.action_type, effective_risk
                        ),
                        escalation_path: "human_in_the_loop://approve".to_string(),
                    };
                    self.sign(decision, timestamp)
                } else {
                    let decision = SafetyDecision::Allowed {
                        reason: format!(
                            "Action '{}' of type {:?} allowed (risk: {:.2}, policy: {:?})",
                            action.action_id, action.action_type, effective_risk, policy_decision
                        ),
                        signed_at: timestamp,
                    };
                    self.sign(decision, timestamp)
                }
            }
        }
    }

    /// Convenience: check whether an action requires destructive confirmation
    pub fn requires_destructive_confirmation(&self, action: &ActionRequest) -> bool {
        DestructiveAction::from_request(action).is_some()
    }

    /// External verification — re-computes HMAC and compares
    /// Can be called OUTSIDE the agent's process with a shared secret
    pub fn verify(&self, evidence: &_SignedEvidence, _action: &ActionRequest) -> bool {
        let canonical = Self::canonical_string_static(&evidence.decision, evidence.timestamp, &evidence.kernel_version);
        let expected_sig = compute_hmac(&canonical, &self.signing_key);
        expected_sig == evidence.signature
    }

    /// Verify signed evidence with an external key (cross-process verification)
    pub fn _verify_with_key(evidence: &_SignedEvidence, key: &[u8; 32], _action: &ActionRequest) -> bool {
        let canonical = Self::canonical_string_static(&evidence.decision, evidence.timestamp, &evidence.kernel_version);
        let expected_sig = compute_hmac(&canonical, key);
        expected_sig == evidence.signature
    }

    fn canonical_string(&self, decision: &SafetyDecision, timestamp: u64) -> String {
        Self::canonical_string_static(decision, timestamp, &self.version)
    }

    fn canonical_string_static(decision: &SafetyDecision, timestamp: u64, kernel_version: &str) -> String {
        match decision {
            SafetyDecision::Allowed { reason, signed_at } => {
                format!("ALLOWED:{}:{}:{}:{}", reason, signed_at, timestamp, kernel_version)
            }
            SafetyDecision::Denied { reason, signed_at, appeal_path } => {
                format!("DENIED:{}:{}:{:?}:{}:{}", reason, signed_at, appeal_path, timestamp, kernel_version)
            }
            SafetyDecision::RequiresApproval { reason, escalation_path } => {
                format!("REQUIRES_APPROVAL:{}:{}:{}:{}", reason, escalation_path, timestamp, kernel_version)
            }
        }
    }

    fn sign(&self, decision: SafetyDecision, timestamp: u64) -> _SignedEvidence {
        let canonical = self.canonical_string(&decision, timestamp);
        let signature = compute_hmac(&canonical, &self.signing_key);

        let evidence = _SignedEvidence {
            decision,
            kernel_version: self.version.clone(),
            signature,
            timestamp,
        };

        if let Ok(mut log) = self.audit_log.lock() {
            log.push(evidence.clone());
            if log.len() > 1000 {
                log.remove(0);
            }
        }

        evidence
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn audit_log(&self) -> Vec<_SignedEvidence> {
        self.audit_log.lock().map_or_else(|_| Vec::new(), |log| log.clone())
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for SafetyKernel {
    fn default() -> Self {
        Self::new()
    }
}

/// _ExecutionTimeGuard wraps SafetyKernel with ergonomic integration hooks
pub struct _ExecutionTimeGuard {
    kernel: Arc<SafetyKernel>,
}

impl _ExecutionTimeGuard {
    pub fn new(kernel: Arc<SafetyKernel>) -> Self {
        Self { kernel }
    }

    /// Guard an action from string parameters
    pub fn _guard_action(
        &self,
        action: &str,
        target: &str,
        args: HashMap<String, String>,
    ) -> Result<_SignedEvidence, String> {
        if action.is_empty() {
            return Err("Action string is empty".to_string());
        }

        let action_type = match action {
            "read_file" | "file_read" => ActionType::FileRead,
            "write_file" | "file_write" => ActionType::FileWrite,
            "delete_file" | "file_delete" => ActionType::FileDelete,
            "network_request" | "network" => ActionType::NetworkRequest,
            "execute_command" | "subprocess" | "exec" => ActionType::SubprocessExec,
            "code_eval" | "eval" | "code_exec" => ActionType::CodeEval,
            "data_export" | "export" => ActionType::DataExport,
            "model_access" | "model" => ActionType::ModelAccess,
            "tool_call" | "tool" => ActionType::ToolCall,
            other => ActionType::Custom(other.to_string()),
        };

        let risk_score = compute_risk_score(&action_type, target, &args);

        let request = ActionRequest {
            action_id: format!("{}-{}", action, target),
            action_type,
            target: target.to_string(),
            args,
            context: HashMap::new(),
            risk_score,
        };

        let evidence = self.kernel.check(&request);

        match &evidence.decision {
            SafetyDecision::Allowed { .. } => Ok(evidence),
            SafetyDecision::Denied { reason, .. } => Err(format!("Denied: {}", reason)),
            SafetyDecision::RequiresApproval { reason, .. } => {
                Err(format!("RequiresApproval: {}", reason))
            }
        }
    }

    /// Verify a signed decision matches an action ID
    pub fn _verify_decision(&self, evidence: &_SignedEvidence, action_id: &str) -> bool {
        let dummy = ActionRequest {
            action_id: action_id.to_string(),
            action_type: ActionType::ToolCall,
            target: String::new(),
            args: HashMap::new(),
            context: HashMap::new(),
            risk_score: 0.0,
        };
        if !self.kernel.verify(evidence, &dummy) {
            return false;
        }
        match &evidence.decision {
            SafetyDecision::Allowed { reason, .. }
            | SafetyDecision::Denied { reason, .. } => reason.contains(action_id),
            SafetyDecision::RequiresApproval { reason, .. } => reason.contains(action_id),
        }
    }

    pub fn kernel(&self) -> &Arc<SafetyKernel> {
        &self.kernel
    }
}

/// Compute HMAC-SHA256 of a canonical string
fn compute_hmac(canonical: &str, key: &[u8; 32]) -> String {
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else {
        log::error!("[safety_kernel] Impossible: HMAC rejected 32-byte key");
        return String::new();
    };
    mac.update(canonical.as_bytes());
    let result = mac.finalize();
    let code = result.into_bytes();
    hex::encode(code)
}

/// Compute a risk score from action type, target, and args
fn compute_risk_score(action_type: &ActionType, target: &str, args: &HashMap<String, String>) -> f64 {
    let base: f64 = match action_type {
        ActionType::FileRead => 0.15_f64,
        ActionType::FileWrite => 0.45_f64,
        ActionType::FileDelete => 0.75_f64,
        ActionType::NetworkRequest => 0.50_f64,
        ActionType::SubprocessExec => 0.70_f64,
        ActionType::CodeEval => 0.85_f64,
        ActionType::DataExport => 0.65_f64,
        ActionType::ModelAccess => 0.30_f64,
        ActionType::ToolCall => 0.40_f64,
        ActionType::Custom(_) => 0.60_f64,
    };

    let target_boost: f64 = if target.contains("secret")
        || target.contains("key")
        || target.contains("password")
        || target.contains("token")
        || target.contains("credential")
        || target.contains(".env")
        || target.contains("..")
        || target.starts_with("/etc")
        || target.starts_with("/var")
        || target.contains("/sys")
    {
        0.30_f64
    } else if target.contains("config")
        || target.contains(".json")
        || target.contains(".toml")
        || target.contains(".yaml")
    {
        0.10_f64
    } else {
        0.0_f64
    };

    let arg_boost: f64 = if args.values().any(|v| {
        v.contains("rm -rf")
            || v.contains("sudo")
            || v.contains("chmod 777")
            || v.contains("curl")
            || v.contains("wget")
            || v.contains("eval(")
            || v.contains("exec(")
    }) {
        0.20_f64
    } else {
        0.0_f64
    };

    let total: f64 = base + target_boost + arg_boost;
    total.max(0.0).min(1.0)
}

impl SelfTest for SafetyKernel {
    fn name(&self) -> &str { "safety_kernel" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        if !self.is_active() {
            return Err(vec!["SafetyKernel should be active by default".into()]);
        }
        if self.version().is_empty() {
            return Err(vec!["SafetyKernel version should not be empty".into()]);
        }
        if !self.audit_log().is_empty() {
            return Err(vec!["SafetyKernel audit log should be empty initially".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_request(action_type: ActionType, target: &str, risk: f64) -> ActionRequest {
        ActionRequest {
            action_id: format!("{:?}-{}", action_type, target),
            action_type,
            target: target.to_string(),
            args: HashMap::new(),
            context: HashMap::new(),
            risk_score: risk,
        }
    }

    #[test]
    fn test_safety_kernel_creation() {
        let kernel = SafetyKernel::new();
        assert!(kernel.is_active());
        assert_eq!(kernel.audit_log().len(), 0);
        assert!(!kernel.version().is_empty());
    }

    #[test]
    fn test_fail_closed_when_inactive() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        kernel.set_active(false);
        assert!(!kernel.is_active());

        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::Denied { reason, .. } => {
                assert!(reason.contains("inactive"), "Denial should mention inactive: {}", reason);
            }
            _ => return Err("Should be denied when kernel is inactive (fail-closed)".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_allow_low_risk_action() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::Allowed { reason, .. } => {
                assert!(reason.contains("allowed"), "Should be allowed: {}", reason);
            }
            other => return Err(format!("Low-risk FileRead should be allowed, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_deny_blocked_action() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::NetworkRequest, "evil.example.com", 0.3);
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::Denied { reason, .. } => {
                assert!(reason.contains("denied"), "Denial should mention denied: {}", reason);
            }
            other => return Err(format!("NetworkRequest to unknown domain should be denied, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_require_approval_high_risk() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileDelete, "/etc/secrets/password.db", 0.0);
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::RequiresApproval { reason, .. } => {
                assert!(
                    reason.contains("high risk") || reason.contains("approval"),
                    "Should mention high risk/approval: {}", reason
                );
            }
            other => return Err(format!("High-risk FileDelete should require approval, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_signed_evidence_verification_pass() {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        let evidence = kernel.check(&request);
        assert!(kernel.verify(&evidence, &request), "Signature should verify");
    }

    #[test]
    fn test_signed_evidence_verification_fail_tampered() {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        let mut evidence = kernel.check(&request);
        evidence.signature = "tampered_signature".to_string();
        assert!(!kernel.verify(&evidence, &request), "Tampered signature should fail");
    }

    #[test]
    fn test_execution_time_guard_basic() {
        let kernel = Arc::new(SafetyKernel::new());
        let guard = _ExecutionTimeGuard::new(kernel);

        let result = guard._guard_action("read_file", "/tmp/test.txt", HashMap::new());
        assert!(result.is_ok(), "read_file should be allowed");

        let result2 = guard._guard_action("network_request", "evil.com", HashMap::new());
        assert!(result2.is_err(), "network_request should be denied");
    }

    #[test]
    fn test_kernel_rejects_missing_action() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = ActionRequest {
            action_id: String::new(),
            action_type: ActionType::FileRead,
            target: "/tmp/test.txt".to_string(),
            args: HashMap::new(),
            context: HashMap::new(),
            risk_score: 0.1,
        };
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::Denied { reason, .. } => {
                assert!(reason.contains("empty"), "Should mention empty: {}", reason);
            }
            other => return Err(format!("Empty action ID should be denied, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_kernel_rejects_empty_args() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = ActionRequest {
            action_id: "test-action".to_string(),
            action_type: ActionType::FileRead,
            target: String::new(),
            args: HashMap::new(),
            context: HashMap::new(),
            risk_score: 0.1,
        };
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::Allowed { .. } => {}
            SafetyDecision::RequiresApproval { .. } => {}
            SafetyDecision::Denied { .. } => {
                return Err("Empty args should not cause denial for allowed action type".to_string());
            }
        }
        Ok(())
    }

    #[test]
    fn test_kernel_requires_approval_for_unknown_high_risk() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = ActionRequest {
            action_id: "dangerous-code-eval".to_string(),
            action_type: ActionType::CodeEval,
            target: "/etc/secrets/decrypt.sh".to_string(),
            args: HashMap::new(),
            context: HashMap::new(),
            risk_score: 0.9,
        };
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::RequiresApproval { .. } => {}
            other => return Err(format!("High-risk CodeEval should require approval, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_evidence_signature_tamper_detection() {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/safe/file.txt", 0.1);
        let evidence = kernel.check(&request);

        assert!(kernel.verify(&evidence, &request));

        let mut tampered = evidence.clone();
        tampered.decision = SafetyDecision::Denied {
            reason: "Forged denial".to_string(),
            signed_at: tampered.timestamp,
            appeal_path: None,
        };
        assert!(!kernel.verify(&tampered, &request), "Tampered decision should fail");

        let mut tampered2 = evidence.clone();
        tampered2.timestamp = 999999999;
        assert!(!kernel.verify(&tampered2, &request), "Tampered timestamp should fail");

        let mut tampered3 = evidence.clone();
        tampered3.kernel_version = "99.99.99".to_string();
        assert!(!kernel.verify(&tampered3, &request), "Tampered version should fail");
    }

    #[test]
    fn test_audit_log_records_decisions() {
        let kernel = SafetyKernel::new();
        assert_eq!(kernel.audit_log().len(), 0);

        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        kernel.check(&request);
        assert_eq!(kernel.audit_log().len(), 1);

        kernel.check(&request);
        assert_eq!(kernel.audit_log().len(), 2);
    }

    #[test]
    fn test_evidence_contains_kernel_version_and_signature() {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        let evidence = kernel.check(&request);
        assert!(!evidence.kernel_version.is_empty());
        assert!(!evidence.signature.is_empty());
    }

    #[test]
    fn test_guard_action_rejects_empty_action() {
        let kernel = Arc::new(SafetyKernel::new());
        let guard = _ExecutionTimeGuard::new(kernel);
        let result = guard._guard_action("", "/tmp/test.txt", HashMap::new());
        assert!(result.is_err(), "Empty action string should be rejected");
    }

    #[test]
    fn test_verify_with_external_key() {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/tmp/test.txt", 0.1);
        let evidence = kernel.check(&request);

        let key = kernel.signing_key;
        assert!(SafetyKernel::_verify_with_key(&evidence, &key, &request));
    }

    #[test]
    fn test_confirm_before_destructive_file_delete() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileDelete, "/tmp/important.txt", 0.3);
        let evidence = kernel.check(&request);
        match &evidence.decision {
            SafetyDecision::RequiresApproval { reason, escalation_path } => {
                assert!(reason.contains("Destructive action"));
                assert!(escalation_path.contains("destructive_confirm://"));
            }
            other => return Err(format!("FileDelete should always require confirmation, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_confirm_before_destructive_subprocess_rm() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let mut args = HashMap::new();
        args.insert("command".into(), "rm -rf /tmp/data".into());
        let request = ActionRequest {
            action_id: "shell-rm".into(),
            action_type: ActionType::SubprocessExec,
            target: "/tmp/data".into(),
            args,
            context: HashMap::new(),
            risk_score: 0.1,
        };
        let evidence = kernel.check(&request);
        match &evidence.decision {
            SafetyDecision::RequiresApproval { reason, .. } => {
                assert!(reason.contains("Destructive action"));
            }
            other => return Err(format!("rm command should require destructive confirmation, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_confirm_before_destructive_system_config_write() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileWrite, "/etc/nginx.conf", 0.2);
        let evidence = kernel.check(&request);
        match &evidence.decision {
            SafetyDecision::RequiresApproval { reason, .. } => {
                assert!(reason.contains("Destructive action"));
            }
            other => return Err(format!("Writing to /etc/ should require destructive confirmation, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_non_destructive_allows_low_risk() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileRead, "/tmp/safe.txt", 0.1);
        let evidence = kernel.check(&request);
        match &evidence.decision {
            SafetyDecision::Allowed { .. } => {}
            other => return Err(format!("Safe read should be allowed, got: {:?}", other)),
        }
        Ok(())
    }

    #[test]
    fn test_requires_destructive_confirmation_api() {
        let kernel = SafetyKernel::new();
        let delete_req = create_request(ActionType::FileDelete, "/tmp/x.txt", 0.1);
        assert!(kernel.requires_destructive_confirmation(&delete_req));

        let read_req = create_request(ActionType::FileRead, "/tmp/x.txt", 0.1);
        assert!(!kernel.requires_destructive_confirmation(&read_req));
    }

    #[test]
    fn test_destructive_action_detection_from_request() {
        let req = create_request(ActionType::FileDelete, "/tmp/test.txt", 0.5);
        assert!(DestructiveAction::from_request(&req).is_some());

        let req2 = create_request(ActionType::FileRead, "/tmp/test.txt", 0.5);
        assert!(DestructiveAction::from_request(&req2).is_none());
    }

    #[test]
    fn test_allow_confirmation_action() -> Result<(), String> {
        let kernel = SafetyKernel::new();
        let request = create_request(ActionType::FileWrite, "/tmp/output.txt", 0.2);
        let evidence = kernel.check(&request);
        match evidence.decision {
            SafetyDecision::Allowed { reason, .. } => {
                assert!(reason.contains("allowed"), "FileWrite (confirmation) with low risk should be allowed: {}", reason);
            }
            SafetyDecision::RequiresApproval { .. } => {
                // 风险可能 > 0.8 如果目标匹配敏感模式, 但 /tmp/output.txt 是安全的
                // FileWrite base 0.45 + target 0.0 + arg 0.0 = 0.45 < 0.8 => Allowed
            }
            other => return Err(format!("FileWrite with low risk should not be denied, got: {:?}", other)),
        }
        Ok(())
    }
}
