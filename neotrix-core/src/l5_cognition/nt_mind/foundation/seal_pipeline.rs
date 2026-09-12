//! L5 SealPipeline trait — abstraction over L1 SEAL pipeline types.
//!
//! Follows dependency inversion: L5 defines the interface, L1 implements.
//! L5 code uses this trait instead of importing L1 SEAL types directly.



/// Oracle decision result (L5层定义)
#[derive(Debug, Clone)]
pub struct OracleDecision {
    pub needs_oracle: bool,
    pub reason: Option<String>,
    pub request: Option<OracleRequest>,
    pub suggested_action: String,
}

/// Oracle request (L5层定义)
#[derive(Debug, Clone)]
pub struct OracleRequest {
    pub id: String,
    pub reason: String,
    pub context: String,
    pub urgency: String,
    pub created_at: u64,
}

/// OracleGate trait — L5 contract for oracle gate operations.
///
/// This trait abstracts the L1 `OracleGate` that L5 needs.
/// The actual implementation lives in L1 and is injected via DI.
pub(crate) trait _OracleGateContract: Send + Sync {
    /// Evaluate failure and decide if oracle intervention is needed.
    fn evaluate_failure(&mut self, attempt_count: u32, dimension: &str) -> OracleDecision;
    
    /// Create a new instance.
    fn new_gate() -> Self where Self: Sized;
}

/// SemanticEntropyGate trait — L5 contract for semantic entropy operations.
///
/// This trait abstracts the L1 `SemanticEntropyGate` that L5 needs.
pub(crate) trait _SemanticEntropyGateContract: Send + Sync {
    /// Compute entropy for a prompt.
    fn compute_entropy(prompt: &str, context: &[String]) -> f64 where Self: Sized;
    
    /// Check if code generation should be deferred.
    fn should_defer(&self, prompt: &str, context: &[String]) -> bool;
    
    /// Record entropy value.
    fn record(&mut self, entropy: f64);
    
    /// Get entropy trend.
    fn entropy_trend(&self) -> _EntropyTrend;
    
    /// Create a new instance.
    fn new_gate() -> Self where Self: Sized;
}

/// Entropy trend direction (L5层定义)
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum _EntropyTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Sandbox verdict (L5层定义)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxVerdict {
    Approved,
    Denied,
    RequiresApproval,
}

/// ActionSandbox trait — L5 contract for action sandbox operations.
///
/// This trait abstracts the L1 `ActionSandbox` that L5 needs.
pub(crate) trait _ActionSandboxContract: Send + Sync {
    /// Evaluate an action against sandbox rules.
    fn evaluate(&mut self, action: &str) -> SandboxVerdict;
    
    /// Evaluate an action with path context.
    fn evaluate_with_path(&mut self, action: &str) -> SandboxVerdict;
    
    /// Get sandbox health score.
    fn health(&self) -> f64;
    
    /// Get sandbox summary.
    fn summary(&self) -> String;
    
    /// Create a new instance.
    fn new_sandbox() -> Self where Self: Sized;
}

/// L1 implementation of _OracleGateContract
pub struct L1OracleGate {
    inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate,
}

impl L1OracleGate {
    pub fn new() -> Self {
        Self {
            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
        }
    }
}

impl _OracleGateContract for L1OracleGate {
    fn evaluate_failure(&mut self, attempt_count: u32, dimension: &str) -> OracleDecision {
        let decision = self.inner.evaluate_failure(attempt_count, dimension);
        OracleDecision {
            needs_oracle: decision.needs_oracle,
            reason: decision.reason.map(|r| format!("{:?}", r)),
            request: decision.request.map(|r| OracleRequest {
                id: r.id,
                reason: format!("{:?}", r.reason),
                context: r.context,
                urgency: format!("{:?}", r.urgency),
                created_at: r.created_at,
            }),
            suggested_action: decision.suggested_action,
        }
    }
    
    fn new_gate() -> Self {
        Self {
            inner: crate::l1_action::nt_act::nt_act_autonomy::oracle_gate::OracleGate::new(),
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for L1OracleGate {
    fn name(&self) -> &str {
        "l1_oracle_gate"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Delegate to inner OracleGate's self_test
        self.inner.self_test()
    }
}

/// L1 implementation of _SemanticEntropyGateContract
pub struct L1SemanticEntropyGate {
    inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate,
}

impl L1SemanticEntropyGate {
    pub fn new() -> Self {
        Self {
            inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
        }
    }
}

impl _SemanticEntropyGateContract for L1SemanticEntropyGate {
    fn compute_entropy(prompt: &str, context: &[String]) -> f64 {
        crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::compute_entropy(prompt, context)
    }
    
    fn should_defer(&self, prompt: &str, context: &[String]) -> bool {
        self.inner.should_defer(prompt, context)
    }
    
    fn record(&mut self, entropy: f64) {
        self.inner.record(entropy);
    }
    
    fn entropy_trend(&self) -> _EntropyTrend {
        match self.inner.entropy_trend() {
            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Increasing => _EntropyTrend::Increasing,
            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Decreasing => _EntropyTrend::Decreasing,
            crate::l1_action::nt_act::nt_act_code::semantic_entropy::TrendDirection::Stable => _EntropyTrend::Stable,
        }
    }
    
    fn new_gate() -> Self {
        Self {
            inner: crate::l1_action::nt_act::nt_act_code::semantic_entropy::SemanticEntropyGate::new(),
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for L1SemanticEntropyGate {
    fn name(&self) -> &str {
        "l1_semantic_entropy_gate"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Delegate to inner SemanticEntropyGate's self_test
        self.inner.self_test()
    }
}

/// L1 implementation of _ActionSandboxContract
pub struct L1ActionSandbox {
    inner: crate::l1_action::nt_act::actions::security::sandbox::ActionSandbox,
}

impl L1ActionSandbox {
    pub fn new() -> Self {
        Self {
            inner: crate::l1_action::nt_act::actions::security::sandbox::ActionSandbox::new(),
        }
    }
}

impl _ActionSandboxContract for L1ActionSandbox {
    fn evaluate(&mut self, action: &str) -> SandboxVerdict {
        match self.inner.evaluate(action) {
            crate::l1_action::nt_act::actions::security::sandbox::SandboxVerdict::Approved => SandboxVerdict::Approved,
            crate::l1_action::nt_act::actions::security::sandbox::SandboxVerdict::Denied => SandboxVerdict::Denied,
            crate::l1_action::nt_act::actions::security::sandbox::SandboxVerdict::RequiresApproval => SandboxVerdict::RequiresApproval,
        }
    }
    
    fn evaluate_with_path(&mut self, action: &str) -> SandboxVerdict {
        match self.inner.evaluate_with_path(action) {
            crate::l1_action::nt_act::actions::security::sandbox::SandboxVerdict::Approved => SandboxVerdict::Approved,
            crate::l1_action::nt_act::actions::security::sandbox::SandboxVerdict::Denied => SandboxVerdict::Denied,
            crate::l1_action::nt_act::actions::security::sandbox::SandboxVerdict::RequiresApproval => SandboxVerdict::RequiresApproval,
        }
    }
    
    fn health(&self) -> f64 {
        self.inner.health()
    }
    
    fn summary(&self) -> String {
        self.inner.summary()
    }
    
    fn new_sandbox() -> Self {
        Self {
            inner: crate::l1_action::nt_act::actions::security::sandbox::ActionSandbox::new(),
        }
    }
}

impl crate::core::nt_core_self_test::SelfTest for L1ActionSandbox {
    fn name(&self) -> &str {
        "l1_action_sandbox"
    }
    
    fn self_test(&self) -> Result<(), Vec<String>> {
        // Delegate to inner ActionSandbox's self_test
        self.inner.self_test()
    }
}
