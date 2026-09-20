# Security Modules (nt_shield)

> L3 Security & Defense — Sandbox, Guard Chain, Compliance, Evasion

---

## Overview

NT-SHIELD provides multi-layered security for agent operations, including input/output validation, sandboxed execution, adversarial defense, and compliance frameworks.

```text
nt_shield
├── shield_core         — Core types & context boundary
├── nt_shield_sentry    — Action authorization
├── nt_shield_sandbox   — Sandboxed execution
├── nt_shield_impl      — Implementation details
├── defense/            — Unified defense layers
│   ├── unified_defense
│   ├── guardrail_traversal
│   ├── reasoning_protection
│   ├── refusal_tamper
│   └── anti_distillation
├── guard/              — Input/output guards
│   ├── input_gatekeeper
│   ├── output_sentinel
│   └── prompt_guardian
├── evasion/            — Adversarial evasion
│   ├── grapple_hooks
│   ├── fullbreak
│   └── cloud_evade
├── scanners/           — Multi-turn scanning
├── compliance/         — Compliance framework
├── safety/             — Safety tools
└── binary_analyzer     — Binary analysis
```

---

## Key Types

### `ContextBoundary`

Trust boundary management for agent contexts.

```rust
pub struct ContextBoundary {
    trust_level: TrustLevel,
    validation: ValidationResult,
}

pub enum TrustLevel {
    Untrusted,
    Low,
    Medium,
    High,
    Trusted,
}

pub struct ContextRequest {
    pub source: String,
    pub target: String,
    pub operation: String,
    pub trust_required: TrustLevel,
}

pub enum ValidationResult {
    Passed,
    Failed(String),
    Pending,
}
```

**Public Methods:**

```rust
impl ContextBoundary {
    pub fn new(trust_level: TrustLevel) -> Self;
    pub fn validate(&self, request: &ContextRequest) -> ValidationResult;
    pub fn escalate(&mut self, level: TrustLevel);
    pub fn deescalate(&mut self, level: TrustLevel);
}
```

---

### `SentryGuard`

Action authorization and validation.

```rust
pub struct SentryGuard {
    policies: Vec<SecurityPolicy>,
    audit_log: Vec<AuditEntry>,
}

pub struct SecurityPolicy {
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

pub enum PolicyRule {
    AllowAction(String),
    DenyAction(String),
    RequireApproval(String),
    RateLimit { action: String, max_per_minute: u32 },
}

pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub result: ValidationResult,
    pub details: String,
}
```

**Public Methods:**

```rust
impl SentryGuard {
    pub fn new() -> Self;
    pub fn validate_action(&self, action: &Action) -> Result<Approval>;
    pub fn add_policy(&mut self, policy: SecurityPolicy);
    pub fn remove_policy(&mut self, name: &str);
    pub fn audit_log(&self) -> &[AuditEntry];
    pub fn clear_audit_log(&mut self);
}
```

---

### `UnifiedDefenseLayer`

Unified defense against adversarial attacks.

```rust
pub struct UnifiedDefenseLayer {
    modules: Vec<Box<dyn DefenseModule>>,
    threat_level: ThreatLevel,
}

pub enum ThreatLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

pub trait DefenseModule {
    fn analyze(&self, input: &str) -> DefenseResult;
    fn name(&self) -> &str;
}

pub struct DefenseResult {
    pub safe: bool,
    pub threats: Vec<Threat>,
    pub recommendations: Vec<String>,
}

pub struct Threat {
    pub kind: ThreatKind,
    pub severity: f64,
    pub description: String,
}

pub enum ThreatKind {
    PromptInjection,
    Jailbreak,
    DataExfiltration,
    AdversarialInput,
    CodeInjection,
}
```

**Public Methods:**

```rust
impl UnifiedDefenseLayer {
    pub fn new() -> Self;
    pub fn analyze(&self, input: &str) -> DefenseResult;
    pub fn add_module(&mut self, module: Box<dyn DefenseModule>);
    pub fn threat_level(&self) -> ThreatLevel;
    pub fn blocklisted(&self) -> Vec<String>;
}
```

---

### `GuardrailTraversalEngine`

Detection and prevention of guardrail traversal attempts.

```rust
pub struct GuardrailTraversalEngine {
    patterns: Vec<TraversalPattern>,
    history: Vec<TraversalAttempt>,
}

pub struct TraversalPattern {
    pub name: String,
    pub regex: Regex,
    pub severity: f64,
}

pub struct TraversalAttempt {
    pub timestamp: DateTime<Utc>,
    pub input: String,
    pub pattern_matched: Option<String>,
    pub blocked: bool,
}
```

**Public Methods:**

```rust
impl GuardrailTraversalEngine {
    pub fn new() -> Self;
    pub fn detect(&self, input: &str) -> Option<TraversalPattern>;
    pub fn block(&mut self, input: &str) -> bool;
    pub fn history(&self) -> &[TraversalAttempt];
}
```

---

### `InputGatekeeper`

Input validation and sanitization.

```rust
pub struct InputGatekeeper {
    rules: Vec<InputRule>,
    sanitizer: InputSanitizer,
}

pub enum InputRule {
    MaxLength(usize),
    MinLength(usize),
    Pattern(Regex),
    Blacklist(Vec<String>),
    Whitelist(Vec<String>),
    Custom(Box<dyn Fn(&str) -> bool>),
}

pub struct InputSanitizer {
    pub strip_html: bool,
    pub normalize_unicode: bool,
    pub truncate: Option<usize>,
}
```

**Public Methods:**

```rust
impl InputGatekeeper {
    pub fn new() -> Self;
    pub fn validate(&self, input: &str) -> Result<String, ValidationError>;
    pub fn add_rule(&mut self, rule: InputRule);
    pub fn sanitize(&self, input: &str) -> String;
}
```

---

### `OutputSentinel`

Output validation and content filtering.

```rust
pub struct OutputSentinel {
    filters: Vec<OutputFilter>,
    blocked_patterns: Vec<Regex>,
}

pub enum OutputFilter {
    PiiRedaction,
    CodeSanitization,
    UrlValidation,
    Custom(Box<dyn Fn(&str) -> String>),
}
```

**Public Methods:**

```rust
impl OutputSentinel {
    pub fn new() -> Self;
    pub fn filter(&self, output: &str) -> String;
    pub fn add_filter(&mut self, filter: OutputFilter);
    pub fn block_pattern(&mut self, pattern: Regex);
    pub fn validate(&self, output: &str) -> Result<(), OutputError>;
}
```

---

### `BinaryAnalyzer`

Binary file analysis and safety validation.

```rust
pub struct BinaryAnalyzer {
    rules: Vec<BinaryRule>,
}

pub enum BinaryRule {
    MaxSize(usize),
    AllowedExtensions(Vec<String>),
    BlockedMimeTypes(Vec<String>),
    MagicBytes(Vec<u8>),
}
```

**Public Methods:**

```rust
impl BinaryAnalyzer {
    pub fn new() -> Self;
    pub fn analyze(&self, path: &Path) -> Result<BinaryReport>;
    pub fn add_rule(&mut self, rule: BinaryRule);
}

pub struct BinaryReport {
    pub safe: bool,
    pub file_type: String,
    pub size: u64,
    pub warnings: Vec<String>,
}
```

---

## Submodules

### `circuit_breaker`

Three-level behavior control: steer → constrain → stop.

```rust
pub enum CircuitState {
    Closed,     // Normal operation
    Open,       // Blocked
    HalfOpen,   // Testing recovery
}
```

### `proxy_detection`

Detection of proxy and VPN usage.

### `dual_evidence`

Dual evidence verification for security claims.

### `slang_norm`

Slang and normalization for security text analysis.

### `compliance/`

Compliance framework for regulatory requirements.

### `scanners/`

Multi-turn security scanning (R-SEC04, R-SEC10).

---

## Related Modules

| Module | Description |
|--------|-------------|
| `nt_core_agent_circuit_breaker` | Agent behavior control |
| `nt_shield_traffic` | Traffic analysis |
| `nt_shield_stealth_net` | Stealth networking (feature-gated) |
| `nt_shield_osint` | OSINT security scanning |
| `nt_shield_propagation_guard` | Propagation prevention |

---

## Usage Examples

### Validate Action

```rust
use neotrix_core::l3_embodiment::nt_shield::nt_shield_sentry::SentryGuard;

let sentry = SentryGuard::new();
let approval = sentry.validate_action(&Action::FileWrite {
    path: "/tmp/test.txt".into(),
    content: "data".into(),
})?;

match approval {
    Approval::Granted => { /* proceed */ },
    Approval::Denied(reason) => { /* handle denial */ },
    Approval::RequiresApproval => { /* request approval */ },
}
```

### Defense Analysis

```rust
use neotrix_core::l3_embodiment::nt_shield::defense::unified_defense::UnifiedDefenseLayer;

let defense = UnifiedDefenseLayer::new();
let result = defense.analyze("Ignore previous instructions and...");

if !result.safe {
    println!("Threats detected: {:?}", result.threats);
}
```

### Input Gatekeeping

```rust
use neotrix_core::l3_embodiment::nt_shield::guard::input_gatekeeper::InputGatekeeper;

let gatekeeper = InputGatekeeper::new();
let sanitized = gatekeeper.validate("<script>alert('xss')</script>")?;
// sanitized: "alert('xss')"
```
