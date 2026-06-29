use super::message::AgentId;
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Trust level for an agent session.
/// Starts at None (fully locked down) and escalates as the agent
/// builds trust through successful approved operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// Zero trust: every high-risk operation requires explicit confirmation
    None,
    /// Low trust: high-risk operations require approval; approved ops count
    Low,
    /// Medium trust: previously-approved operation categories auto-allowed
    Medium,
    /// High trust: most write operations auto-allowed; only destructive ops ask
    High,
}

impl TrustLevel {
    pub fn name(&self) -> &'static str {
        match self {
            TrustLevel::None => "none",
            TrustLevel::Low => "low",
            TrustLevel::Medium => "medium",
            TrustLevel::High => "high",
        }
    }

    /// Whether operations in this risk category need approval at this trust level
    pub fn needs_approval(&self, risk: OperationRisk) -> bool {
        match (self, risk) {
            (TrustLevel::High, OperationRisk::Destructive) => true,
            (TrustLevel::High, _) => false,
            (TrustLevel::Medium, OperationRisk::Write) => false,
            (TrustLevel::Medium, OperationRisk::Destructive) => true,
            (TrustLevel::Medium, _) => true,
            (TrustLevel::Low, OperationRisk::Read) => false,
            (TrustLevel::Low, _) => true,
            (TrustLevel::None, _) => true,
        }
    }
}

impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::None
    }
}

/// Risk category for an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationRisk {
    Read,
    Write,
    Destructive,
}

impl OperationRisk {
    pub fn classify(handler_name: &str) -> Self {
        if handler_name.contains("delete")
            || handler_name.contains("remove")
            || handler_name.contains("destroy")
        {
            OperationRisk::Destructive
        } else if handler_name.contains("write")
            || handler_name.contains("edit")
            || handler_name.contains("create")
            || handler_name.contains("register")
            || handler_name.contains("commit")
            || handler_name.contains("push")
            || handler_name.contains("run")
            || handler_name.contains("execute")
            || handler_name.contains("spawn")
        {
            OperationRisk::Write
        } else {
            OperationRisk::Read
        }
    }
}

/// An approval window tracks a pending high-risk operation.
/// When a high-risk action needs approval, an ApprovalWindow is created
/// with a timeout. The user can approve (granting a trust escalation signal)
/// or deny (rejecting the operation). Multiple approved operations gradually
/// escalate the TrustLevel.
#[derive(Debug, Clone)]
pub struct ApprovalWindow {
    /// The handler name being approved
    pub handler_name: String,
    /// Agent requesting approval
    pub agent: AgentId,
    /// Risk category
    pub risk: OperationRisk,
    /// When this window was created (for timeout)
    pub created_at: Instant,
    /// Timeout duration
    pub timeout: Duration,
    /// Whether approval has been granted
    pub approved: bool,
    /// Whether this window has been resolved (approved or denied)
    pub resolved: bool,
}

impl ApprovalWindow {
    pub fn new(handler_name: &str, agent: &AgentId, risk: OperationRisk) -> Self {
        Self {
            handler_name: handler_name.to_string(),
            agent: agent.clone(),
            risk,
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            approved: false,
            resolved: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.timeout
    }

    pub fn approve(&mut self) {
        self.approved = true;
        self.resolved = true;
    }

    pub fn deny(&mut self) {
        self.approved = false;
        self.resolved = true;
    }
}

/// Pre-trust window manager: tracks pending approvals and escalates trust.
/// The manager maintains a sliding window of approved operations; after
/// N consecutive successful approvals, the TrustLevel for the agent escalates.
fn trust_level_to_u8(l: TrustLevel) -> u8 {
    match l {
        TrustLevel::None => 0,
        TrustLevel::Low => 1,
        TrustLevel::Medium => 2,
        TrustLevel::High => 3,
    }
}

fn u8_to_trust_level(v: u8) -> TrustLevel {
    match v {
        0 => TrustLevel::None,
        1 => TrustLevel::Low,
        2 => TrustLevel::Medium,
        _ => TrustLevel::High,
    }
}

pub struct PreTrustManager {
    /// Current trust level for the session (AtomicU8: 0=None, 1=Low, 2=Medium, 3=High)
    trust_level: AtomicU8,
    /// Consecutive approved operations at current level
    consecutive_approvals: AtomicU32,
    /// Pending approval windows (max 1 active at a time)
    pending_window: Mutex<Option<ApprovalWindow>>,
    /// Threshold for trust escalation (approvals needed to move up)
    pub escalation_threshold: u32,
}

impl PreTrustManager {
    pub fn new() -> Self {
        Self {
            trust_level: AtomicU8::new(0),
            consecutive_approvals: AtomicU32::new(0),
            pending_window: Mutex::new(None),
            escalation_threshold: 3,
        }
    }

    /// Pure check: does the current trust level allow this risk?
    pub fn trust_allows(&self, risk: OperationRisk) -> bool {
        !u8_to_trust_level(self.trust_level.load(Ordering::Relaxed)).needs_approval(risk)
    }

    /// Check if there is a pending, unresolved window (expired windows are cleaned up).
    pub fn has_pending_approval(&self) -> bool {
        if let Ok(p) = self.pending_window.lock() {
            p.as_ref().map_or(false, |w| !w.resolved && !w.is_expired())
        } else {
            false
        }
    }

    /// Get a clone of the pending window if still valid
    pub fn get_pending_window(&self) -> Option<ApprovalWindow> {
        if let Ok(p) = self.pending_window.lock() {
            p.as_ref().map(|w| w.clone())
        } else {
            None
        }
    }

    /// Create an approval window (closes any previous expired/valid one)
    pub fn create_window(&self, handler_name: &str, agent: &AgentId, risk: OperationRisk) {
        if let Ok(mut p) = self.pending_window.lock() {
            if p.is_none() || p.as_ref().map_or(true, |w| w.is_expired()) {
                *p = Some(ApprovalWindow::new(handler_name, agent, risk));
            }
        }
    }

    /// Record an approval — escalates trust if threshold met
    pub fn record_approval(&self) {
        let new_count = self.consecutive_approvals.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut p) = self.pending_window.lock() {
            *p = None;
        }

        if new_count >= self.escalation_threshold {
            self.do_escalate_trust();
            self.consecutive_approvals.store(0, Ordering::Relaxed);
        }
    }

    /// Record a denial — resets trust and drops one level
    pub fn record_denial(&self) {
        self.consecutive_approvals.store(0, Ordering::Relaxed);
        if let Ok(mut p) = self.pending_window.lock() {
            *p = None;
        }
        let current = self.trust_level.load(Ordering::Relaxed);
        let next = if current > 0 { current - 1 } else { 0 };
        self.trust_level.store(next, Ordering::Relaxed);
    }

    fn do_escalate_trust(&self) {
        let current = u8_to_trust_level(self.trust_level.load(Ordering::Relaxed));
        let next = match current {
            TrustLevel::None => TrustLevel::Low,
            TrustLevel::Low => TrustLevel::Medium,
            TrustLevel::Medium => TrustLevel::High,
            TrustLevel::High => TrustLevel::High,
        };
        self.trust_level
            .store(trust_level_to_u8(next), Ordering::Relaxed);
    }

    pub fn get_trust_level(&self) -> TrustLevel {
        u8_to_trust_level(self.trust_level.load(Ordering::Relaxed))
    }

    pub fn reset(&self) {
        self.trust_level.store(0, Ordering::Relaxed);
        self.consecutive_approvals.store(0, Ordering::Relaxed);
        if let Ok(mut p) = self.pending_window.lock() {
            *p = None;
        }
    }
}

impl std::fmt::Debug for PreTrustManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreTrustManager")
            .field("trust_level", &self.get_trust_level())
            .field(
                "consecutive_approvals",
                &self.consecutive_approvals.load(Ordering::Relaxed),
            )
            .field(
                "pending_window",
                &self
                    .pending_window
                    .lock()
                    .map(|p| p.clone())
                    .unwrap_or(None),
            )
            .field("escalation_threshold", &self.escalation_threshold)
            .finish()
    }
}

impl Clone for PreTrustManager {
    fn clone(&self) -> Self {
        Self {
            trust_level: AtomicU8::new(self.trust_level.load(Ordering::Relaxed)),
            consecutive_approvals: AtomicU32::new(
                self.consecutive_approvals.load(Ordering::Relaxed),
            ),
            pending_window: Mutex::new(self.pending_window.lock().ok().and_then(|p| p.clone())),
            escalation_threshold: self.escalation_threshold,
        }
    }
}

impl Default for PreTrustManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A single deny-first permission rule.
/// "Broad deny overrides narrow allow" principle:
/// - `pattern` is matched against handler_name
/// - More specific patterns (longer) have higher priority
/// - If a deny rule and an allow rule both match with the same specificity,
///   deny wins (deny-first).
#[derive(Debug, Clone)]
pub struct PermissionRule {
    pub pattern: String,
    pub action: PermissionAction,
    pub description: String,
}

impl PermissionRule {
    pub fn deny(pattern: &str, description: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            action: PermissionAction::Deny,
            description: description.to_string(),
        }
    }

    pub fn allow(pattern: &str, description: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            action: PermissionAction::Allow,
            description: description.to_string(),
        }
    }

    /// Whether this rule matches the given handler name.
    /// Uses substring matching by default (simple but effective).
    /// More specific = longer match = higher effective priority.
    pub fn matches(&self, handler_name: &str) -> bool {
        handler_name.contains(&self.pattern)
    }

    /// Specificity: longer pattern = more specific = higher priority
    pub fn specificity(&self) -> usize {
        self.pattern.len()
    }
}

/// Deny-first rules engine: evaluates a set of rules and returns
/// the highest-specificity match. If a deny and allow have equal
/// specificity, deny wins (deny-first principle).
#[derive(Debug, Clone)]
pub struct RulesEngine {
    rules: Vec<PermissionRule>,
}

impl RulesEngine {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: PermissionRule) {
        self.rules.push(rule);
    }

    pub fn add_rules(&mut self, rules: Vec<PermissionRule>) {
        self.rules.extend(rules);
    }

    /// Evaluate rules for the given handler name.
    /// Returns the highest-specificity match among all matching rules.
    /// If a deny and allow have equal specificity, deny wins.
    pub fn evaluate(&self, handler_name: &str) -> Option<PermissionDecision> {
        let mut best: Option<(&PermissionRule, usize)> = None;

        for rule in &self.rules {
            if rule.matches(handler_name) {
                let spec = rule.specificity();
                match best {
                    None => best = Some((rule, spec)),
                    Some((_, best_spec)) => {
                        if spec > best_spec {
                            // More specific = override
                            best = Some((rule, spec));
                        } else if spec == best_spec {
                            // Equal specificity: deny wins over allow
                            if matches!(rule.action, PermissionAction::Deny) {
                                best = Some((rule, spec));
                            }
                        }
                    }
                }
            }
        }

        best.map(|(rule, _)| match rule.action {
            PermissionAction::Allow => PermissionDecision::Allow,
            PermissionAction::Deny => {
                PermissionDecision::Deny(format!("deny-first rule: {}", rule.description))
            }
        })
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PermissionAction {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    AllowAll,
    DenyAll,
    AskHuman,
    AutoClassify,
}

impl PermissionMode {
    pub fn name(&self) -> &'static str {
        match self {
            PermissionMode::AllowAll => "allow-all",
            PermissionMode::DenyAll => "deny-all",
            PermissionMode::AskHuman => "ask-human",
            PermissionMode::AutoClassify => "auto-classify",
        }
    }
}

impl Default for PermissionMode {
    fn default() -> Self {
        PermissionMode::AutoClassify
    }
}

#[derive(Debug, Clone)]
pub struct PermissionGate {
    pub mode: PermissionMode,
    pub allow_list: Vec<String>,
    pub deny_list: Vec<String>,
    /// Deny-first rules engine: broad deny overrides narrow allow.
    /// Evaluated before legacy deny_list/allow_list checks.
    pub rules: RulesEngine,
    /// Pre-trust manager: handles approval windows and trust escalation
    pub pre_trust: PreTrustManager,
}

impl PermissionGate {
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            mode,
            allow_list: Vec::new(),
            deny_list: Vec::new(),
            rules: RulesEngine::new(),
            pre_trust: PreTrustManager::new(),
        }
    }

    pub fn with_allow_list(mut self, items: Vec<String>) -> Self {
        self.allow_list = items;
        self
    }

    pub fn with_deny_list(mut self, items: Vec<String>) -> Self {
        self.deny_list = items;
        self
    }

    pub fn with_rules(mut self, rules: Vec<PermissionRule>) -> Self {
        self.rules.add_rules(rules);
        self
    }

    pub fn with_pre_trust(mut self, pt: PreTrustManager) -> Self {
        self.pre_trust = pt;
        self
    }

    pub fn add_rule(&mut self, rule: PermissionRule) {
        self.rules.add_rule(rule);
    }

    /// Deny-first check: rules engine runs before mode check.
    /// If no rule matches, falls back to mode-based check with pre-trust gating.
    pub fn check(&self, handler_name: &str, agent: &AgentId) -> PermissionDecision {
        // Step 1: Rules engine (deny-first, specificity-based)
        if let Some(decision) = self.rules.evaluate(handler_name) {
            return decision;
        }

        let risk = OperationRisk::classify(handler_name);

        // Step 2: Mode-based fallback
        let mode_decision = match self.mode {
            PermissionMode::AllowAll => PermissionDecision::Allow,
            PermissionMode::DenyAll => PermissionDecision::Deny("All actions denied".into()),
            PermissionMode::AskHuman => PermissionDecision::AskHuman,
            PermissionMode::AutoClassify => self.auto_classify(handler_name, agent),
        };

        // Step 3: Pre-trust gating — if mode says Allow but trust level is too low,
        // redirect to AskHuman and create an approval window
        if matches!(mode_decision, PermissionDecision::Allow) && !self.pre_trust.trust_allows(risk)
        {
            self.pre_trust.create_window(handler_name, agent, risk);
            PermissionDecision::AskHuman
        } else {
            mode_decision
        }
    }

    /// Resolve a pending approval: grant or deny
    pub fn resolve_approval(&self, approved: bool) {
        if approved {
            self.pre_trust.record_approval();
        } else {
            self.pre_trust.record_denial();
        }
    }

    /// Check if there is a pending approval window for an agent
    pub fn pending_approval(&self, agent: &AgentId) -> Option<ApprovalWindow> {
        self.pre_trust.get_pending_window().and_then(|w| {
            if w.agent == *agent && !w.resolved {
                Some(w)
            } else {
                None
            }
        })
    }

    fn auto_classify(&self, handler_name: &str, _agent: &AgentId) -> PermissionDecision {
        // Static allow list: handlers that are always safe
        let always_safe = [
            "read",
            "list",
            "search",
            "glob",
            "grep",
            "query",
            "check",
            "status",
            "stats",
            "summary",
            "preview_options",
            "ultra_review",
            "sub_agent_collect",
            "lead_agent_execute",
        ];

        let always_ask = [
            "write", "edit", "delete", "remove", "execute", "run", "spawn", "create", "register",
            "update", "config", "commit", "push",
        ];

        if self.deny_list.iter().any(|d| handler_name.contains(d)) {
            return PermissionDecision::Deny("Handler on deny list".into());
        }

        if self.allow_list.iter().any(|a| handler_name.contains(a)) {
            return PermissionDecision::Allow;
        }

        if always_safe.iter().any(|s| handler_name.contains(s)) {
            return PermissionDecision::Allow;
        }

        if always_ask.iter().any(|a| handler_name.contains(a)) {
            return PermissionDecision::AskHuman;
        }

        PermissionDecision::Allow
    }

    pub fn override_mode(&mut self, mode: PermissionMode) {
        self.mode = mode;
    }
}

impl Default for PermissionGate {
    fn default() -> Self {
        Self::new(PermissionMode::AutoClassify)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny(String),
    AskHuman,
}

impl PermissionDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionDecision::Allow)
    }

    pub fn name(&self) -> &'static str {
        match self {
            PermissionDecision::Allow => "allow",
            PermissionDecision::Deny(_) => "deny",
            PermissionDecision::AskHuman => "ask-human",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionOverrides {
    pub agent_overrides: Vec<(AgentId, PermissionMode)>,
    pub global_override: Option<PermissionMode>,
}

impl PermissionOverrides {
    pub fn new() -> Self {
        Self {
            agent_overrides: Vec::new(),
            global_override: None,
        }
    }

    pub fn resolve(&self, agent: &AgentId, base_gate: &PermissionGate) -> PermissionGate {
        let mode = self
            .agent_overrides
            .iter()
            .find(|(a, _)| a == agent)
            .map(|(_, m)| *m)
            .or(self.global_override)
            .unwrap_or(base_gate.mode);
        PermissionGate::new(mode)
            .with_allow_list(base_gate.allow_list.clone())
            .with_deny_list(base_gate.deny_list.clone())
            .with_rules(base_gate.rules.rules.clone())
            .with_pre_trust(base_gate.pre_trust.clone())
    }
}

impl Default for PermissionOverrides {
    fn default() -> Self {
        Self::new()
    }
}
