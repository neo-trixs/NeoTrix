use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditMode {
    Static,
    DynamicSafe,
    DynamicActive,
    OnlineAuthorized,
    Hybrid,
    Full,
    Focused(Vec<AuditDimension>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditDimension {
    Security,
    Stability,
    Testing,
    Architecture,
    Performance,
    Maintainability,
    Design,
    Release,
    Documentation,
    Configuration,
    Observability,
    DataIntegrity,
    Privacy,
    Accessibility,
    SupplyChain,
    Cost,
    AiSafety,
    Fallback,
    BackendApi,
    CodeConsistency,
    CommentCoverage,
    DependencyWeight,
    FrontendState,
    TestingAuthenticity,
    TypeSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VulnDomain {
    Authentication,
    Authorization,
    Session,
    ApiSecurity,
    Injection,
    Xss,
    Ssrf,
    FileUpload,
    Crypto,
    Config,
    Dependency,
    AiLlm,
    WebSocket,
    CloudInfra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityCheck {
    pub id: String,
    pub title: String,
    pub domain: VulnDomain,
    pub severity: Severity,
    pub description: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_id: String,
    pub status: CheckStatus,
    pub evidence: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    Passed,
    Failed,
    Suspicious,
    NotApplicable,
    Deferred,
    NotChecked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub project: String,
    pub mode: AuditMode,
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub suspicious: usize,
    pub score: f64,
    pub results: Vec<CheckResult>,
}

pub struct SecurityAuditor;

impl SecurityAuditor {
    pub fn checklist() -> Vec<VulnerabilityCheck> {
        vec![
            // ===== Authentication (5) =====
            VulnerabilityCheck {
                id: "V001".into(), title: "Weak Password Policy".into(),
                domain: VulnDomain::Authentication, severity: Severity::High,
                description: "Password policy does not enforce minimum complexity requirements".into(),
                remediation: "Enforce minimum 12 chars, mixed case, digits, and special characters".into(),
            },
            VulnerabilityCheck {
                id: "V002".into(), title: "Missing Multi-Factor Authentication".into(),
                domain: VulnDomain::Authentication, severity: Severity::Critical,
                description: "No MFA enforced on privileged accounts or sensitive operations".into(),
                remediation: "Implement TOTP, WebAuthn, or SMS-based MFA for all privileged access".into(),
            },
            VulnerabilityCheck {
                id: "V003".into(), title: "Session Fixation".into(),
                domain: VulnDomain::Authentication, severity: Severity::High,
                description: "Session identifiers are not regenerated after login".into(),
                remediation: "Regenerate session ID on successful authentication via session.regenerate()".into(),
            },
            VulnerabilityCheck {
                id: "V004".into(), title: "JWT None Algorithm".into(),
                domain: VulnDomain::Authentication, severity: Severity::Critical,
                description: "JWT library accepts 'none' algorithm, allowing forged tokens".into(),
                remediation: "Explicitly reject 'none' algorithm; always validate algorithm header".into(),
            },
            VulnerabilityCheck {
                id: "V005".into(), title: "Credential Exposure in Logs".into(),
                domain: VulnDomain::Authentication, severity: Severity::Critical,
                description: "Credentials or tokens may be logged in plaintext".into(),
                remediation: "Implement credential scrubbing in log pipelines; never log request bodies with passwords".into(),
            },

            // ===== Authorization (5) — reuse VulnDomain::Authorization =====
            VulnerabilityCheck {
                id: "V006".into(), title: "Broken Object-Level Authorization".into(),
                domain: VulnDomain::Authorization, severity: Severity::Critical,
                description: "API endpoints lack ownership checks on resource access by ID".into(),
                remediation: "Verify user identity against resource owner before returning data".into(),
            },
            VulnerabilityCheck {
                id: "V007".into(), title: "Mass Assignment".into(),
                domain: VulnDomain::Authorization, severity: Severity::High,
                description: "User input is directly bound to model fields without allowlisting".into(),
                remediation: "Use DTOs or allowlists to control which fields can be mass-assigned".into(),
            },
            VulnerabilityCheck {
                id: "V008".into(), title: "Privilege Escalation via API".into(),
                domain: VulnDomain::Authorization, severity: Severity::Critical,
                description: "Low-privilege users can access admin-level endpoints".into(),
                remediation: "Enforce role-based access control on every endpoint".into(),
            },
            VulnerabilityCheck {
                id: "V009".into(), title: "Insecure Direct Object Reference".into(),
                domain: VulnDomain::Authorization, severity: Severity::High,
                description: "Internal IDs exposed in URLs allow enumeration of resources".into(),
                remediation: "Use opaque UUIDs instead of sequential IDs; verify ownership server-side".into(),
            },
            VulnerabilityCheck {
                id: "V010".into(), title: "Missing Function-Level Access Control".into(),
                domain: VulnDomain::Authorization, severity: Severity::High,
                description: "Controller methods lack authorization annotations".into(),
                remediation: "Apply @PreAuthorize or middleware guards to every handler".into(),
            },

            // ===== Injection (5) =====
            VulnerabilityCheck {
                id: "V011".into(), title: "SQL Injection".into(),
                domain: VulnDomain::Injection, severity: Severity::Critical,
                description: "User input concatenated into SQL queries without parameterization".into(),
                remediation: "Use parameterized queries or ORM with bound parameters".into(),
            },
            VulnerabilityCheck {
                id: "V012".into(), title: "Command Injection".into(),
                domain: VulnDomain::Injection, severity: Severity::Critical,
                description: "User input passed directly to shell command execution".into(),
                remediation: "Avoid shell execution; use safe APIs with arg vectors; validate and sanitize input".into(),
            },
            VulnerabilityCheck {
                id: "V013".into(), title: "Server-Side Template Injection".into(),
                domain: VulnDomain::Injection, severity: Severity::High,
                description: "User input rendered as template content".into(),
                remediation: "Pre-compile templates; never treat user input as template source".into(),
            },
            VulnerabilityCheck {
                id: "V014".into(), title: "NoSQL Injection".into(),
                domain: VulnDomain::Injection, severity: Severity::High,
                description: "Operator injection in MongoDB/NoSQL queries via unvalidated input".into(),
                remediation: "Sanitize input; disable $where/$regex operators on user-facing queries".into(),
            },
            VulnerabilityCheck {
                id: "V015".into(), title: "XPath Injection".into(),
                domain: VulnDomain::Injection, severity: Severity::Medium,
                description: "User input embedded directly into XPath queries".into(),
                remediation: "Use parameterized XPath queries or pre-compile expressions".into(),
            },

            // ===== XSS (5) =====
            VulnerabilityCheck {
                id: "V016".into(), title: "Reflected XSS".into(),
                domain: VulnDomain::Xss, severity: Severity::High,
                description: "User input reflected in HTTP response without sanitization".into(),
                remediation: "Apply context-aware output encoding; use CSP headers; validate input".into(),
            },
            VulnerabilityCheck {
                id: "V017".into(), title: "Stored XSS".into(),
                domain: VulnDomain::Xss, severity: Severity::High,
                description: "User input persisted then rendered without sanitization".into(),
                remediation: "Sanitize on both input and output; use Content-Security-Policy".into(),
            },
            VulnerabilityCheck {
                id: "V018".into(), title: "DOM-Based XSS".into(),
                domain: VulnDomain::Xss, severity: Severity::High,
                description: "Client-side JavaScript writes user input to DOM unsafely".into(),
                remediation: "Use textContent instead of innerHTML; avoid eval-like sinks".into(),
            },
            VulnerabilityCheck {
                id: "V019".into(), title: "Unsafe innerHTML Usage".into(),
                domain: VulnDomain::Xss, severity: Severity::Medium,
                description: "dangerouslySetInnerHTML or innerHTML used with untrusted data".into(),
                remediation: "Use safe rendering APIs; sanitize with DOMPurify before insertion".into(),
            },
            VulnerabilityCheck {
                id: "V020".into(), title: "Missing Content-Security-Policy".into(),
                domain: VulnDomain::Xss, severity: Severity::Medium,
                description: "No CSP header set, allowing inline scripts and untrusted sources".into(),
                remediation: "Set strict CSP: script-src 'self'; object-src 'none'; base-uri 'self'".into(),
            },

            // ===== Config (5) =====
            VulnerabilityCheck {
                id: "V021".into(), title: "Hardcoded Secrets in Source".into(),
                domain: VulnDomain::Config, severity: Severity::Critical,
                description: "API keys, tokens, or passwords hardcoded in source files".into(),
                remediation: "Move secrets to environment variables or vault service".into(),
            },
            VulnerabilityCheck {
                id: "V022".into(), title: "Debug Mode Enabled in Production".into(),
                domain: VulnDomain::Config, severity: Severity::High,
                description: "Debug endpoints or verbose error pages exposed in production".into(),
                remediation: "Disable debug mode; use generic error pages in production".into(),
            },
            VulnerabilityCheck {
                id: "V023".into(), title: "CORS Misconfiguration".into(),
                domain: VulnDomain::Config, severity: Severity::Medium,
                description: "Access-Control-Allow-Origin set to wildcard with credentials".into(),
                remediation: "Restrict origins to specific domains; never pair '*' with credentials".into(),
            },
            VulnerabilityCheck {
                id: "V024".into(), title: "Missing Security Headers".into(),
                domain: VulnDomain::Config, severity: Severity::Medium,
                description: "HSTS, X-Frame-Options, X-Content-Type-Options headers not set".into(),
                remediation: "Add Strict-Transport-Security, X-Frame-Options: DENY, X-Content-Type-Options: nosniff".into(),
            },
            VulnerabilityCheck {
                id: "V025".into(), title: "Insecure TLS Configuration".into(),
                domain: VulnDomain::Config, severity: Severity::High,
                description: "TLS 1.0/1.1 enabled or weak cipher suites accepted".into(),
                remediation: "Disable TLS < 1.2; use only AEAD cipher suites (GCM/ChaCha20)".into(),
            },

            // ===== Dependency (2) =====
            VulnerabilityCheck {
                id: "V026".into(), title: "Known Vulnerable Dependencies".into(),
                domain: VulnDomain::Dependency, severity: Severity::Critical,
                description: "Dependencies with known CVEs (e.g., log4shell, zip slip)".into(),
                remediation: "Use `cargo audit` or `npm audit`; enable Dependabot/Renovate; pin versions".into(),
            },
            VulnerabilityCheck {
                id: "V027".into(), title: "Supply Chain Attack".into(),
                domain: VulnDomain::Dependency, severity: Severity::High,
                description: "Typo-squatting, compromised maintainer accounts, or malicious packages".into(),
                remediation: "Verify package integrity (SHA256); use lockfiles; restrict registry sources".into(),
            },

            // ===== AI/LLM (5) =====
            VulnerabilityCheck {
                id: "V028".into(), title: "Prompt Injection".into(),
                domain: VulnDomain::AiLlm, severity: Severity::Critical,
                description: "User prompts can override system instructions or inject commands".into(),
                remediation: "Use input classification; separate system prompts from user input; apply delimiters".into(),
            },
            VulnerabilityCheck {
                id: "V029".into(), title: "Training Data Poisoning".into(),
                domain: VulnDomain::AiLlm, severity: Severity::High,
                description: "Malicious data in training set biases model outputs".into(),
                remediation: "Validate training data provenance; implement data sanitization pipeline".into(),
            },
            VulnerabilityCheck {
                id: "V030".into(), title: "Model Inversion".into(),
                domain: VulnDomain::AiLlm, severity: Severity::High,
                description: "Attacker reconstructs training data from model outputs".into(),
                remediation: "Apply differential privacy; limit output verbosity; rate-limit API access".into(),
            },
            VulnerabilityCheck {
                id: "V031".into(), title: "Excessive Agency for LLM Agent".into(),
                domain: VulnDomain::AiLlm, severity: Severity::Critical,
                description: "Agent has permissions beyond task scope, enabling privilege escalation".into(),
                remediation: "Apply least-privilege; scope tool access per session; human-in-the-loop for destructive ops".into(),
            },
            VulnerabilityCheck {
                id: "V032".into(), title: "Sensitive Data Leakage via LLM".into(),
                domain: VulnDomain::AiLlm, severity: Severity::High,
                description: "Secrets or PII included in LLM context could leak through output".into(),
                remediation: "Scrub sensitive data from context; implement output filters; use redaction".into(),
            },

            // ===== API (4) =====
            VulnerabilityCheck {
                id: "V031".into(), title: "Missing Rate Limiting".into(),
                domain: VulnDomain::ApiSecurity, severity: Severity::Medium,
                description: "API endpoints lack request rate limiting".into(),
                remediation: "Implement token bucket or sliding window rate limiter per user/IP".into(),
            },
            VulnerabilityCheck {
                id: "V032".into(), title: "Improper Asset Management".into(),
                domain: VulnDomain::ApiSecurity, severity: Severity::Medium,
                description: "Deprecated or shadow API versions still accessible".into(),
                remediation: "Inventory all API endpoints; deprecate with sunset headers; remove old versions".into(),
            },
            VulnerabilityCheck {
                id: "V033".into(), title: "Unvalidated API Input".into(),
                domain: VulnDomain::ApiSecurity, severity: Severity::High,
                description: "API request body lacks schema validation".into(),
                remediation: "Apply JSON Schema or strong type validation on all request bodies".into(),
            },
            VulnerabilityCheck {
                id: "V034".into(), title: "Excessive Data Exposure".into(),
                domain: VulnDomain::ApiSecurity, severity: Severity::Medium,
                description: "API responses return full objects instead of minimal views".into(),
                remediation: "Use response DTOs; return only fields the client needs".into(),
            },

            // ===== Session (3) =====
            VulnerabilityCheck {
                id: "V035".into(), title: "Weak Session Token Generation".into(),
                domain: VulnDomain::Session, severity: Severity::High,
                description: "Session tokens generated with insufficient entropy or predictable seed".into(),
                remediation: "Use cryptographically secure random generator (OsRng) for session tokens".into(),
            },
            VulnerabilityCheck {
                id: "V036".into(), title: "Missing Session Expiry".into(),
                domain: VulnDomain::Session, severity: Severity::Medium,
                description: "Sessions never expire or have excessively long timeouts".into(),
                remediation: "Set absolute and idle timeouts; rotate session on privilege escalation".into(),
            },
            VulnerabilityCheck {
                id: "V037".into(), title: "Cookie Without Secure Flags".into(),
                domain: VulnDomain::Session, severity: Severity::Medium,
                description: "Session cookies missing HttpOnly, Secure, or SameSite attributes".into(),
                remediation: "Set HttpOnly, Secure, SameSite=Lax on all session cookies".into(),
            },

            // ===== Crypto (3) =====
            VulnerabilityCheck {
                id: "V038".into(), title: "Weak Hashing Algorithm".into(),
                domain: VulnDomain::Crypto, severity: Severity::High,
                description: "MD5 or SHA-1 used for password storage or integrity checks".into(),
                remediation: "Use Argon2id or bcrypt for passwords; SHA-256+ for integrity".into(),
            },
            VulnerabilityCheck {
                id: "V039".into(), title: "Non-AEAD Cipher Mode".into(),
                domain: VulnDomain::Crypto, severity: Severity::High,
                description: "AES-ECB or CBC mode used without authentication tag".into(),
                remediation: "Use AES-GCM or ChaCha20-Poly1305 with random nonce".into(),
            },
            VulnerabilityCheck {
                id: "V040".into(), title: "Insufficient Key Length".into(),
                domain: VulnDomain::Crypto, severity: Severity::Medium,
                description: "RSA key < 2048 bits or ECC key < 256 bits used".into(),
                remediation: "Use RSA-2048+ or ECC P-256+; prefer Ed25519 for signing".into(),
            },

            // ===== SSRF (2) =====
            VulnerabilityCheck {
                id: "V041".into(), title: "Server-Side Request Forgery".into(),
                domain: VulnDomain::Ssrf, severity: Severity::Critical,
                description: "Application fetches user-supplied URLs without validation".into(),
                remediation: "Allowlist permitted hosts; block private IP ranges; disable redirect following".into(),
            },
            VulnerabilityCheck {
                id: "V042".into(), title: "Cloud Metadata Endpoint Access".into(),
                domain: VulnDomain::Ssrf, severity: Severity::Critical,
                description: "SSRF can reach cloud metadata service (169.254.169.254)".into(),
                remediation: "Block link-local and metadata IPs at proxy/firewall level".into(),
            },

            // ===== File Upload (3) =====
            VulnerabilityCheck {
                id: "V043".into(), title: "Unrestricted File Upload".into(),
                domain: VulnDomain::FileUpload, severity: Severity::High,
                description: "No file type or size validation on upload endpoints".into(),
                remediation: "Validate MIME type server-side; enforce max file size; scan for malware".into(),
            },
            VulnerabilityCheck {
                id: "V044".into(), title: "Path Traversal in File Upload".into(),
                domain: VulnDomain::FileUpload, severity: Severity::High,
                description: "Uploaded file name used without sanitization in path construction".into(),
                remediation: "Use random file names; reject path separators in file names; store outside webroot".into(),
            },
            VulnerabilityCheck {
                id: "V045".into(), title: "Uploaded File Execution".into(),
                domain: VulnDomain::FileUpload, severity: Severity::Critical,
                description: "Uploaded files stored in web-accessible directory and can be executed".into(),
                remediation: "Store uploads outside webroot; serve via separate domain with no execution".into(),
            },

            // ===== WebSocket (3) =====
            VulnerabilityCheck {
                id: "V046".into(), title: "WebSocket Without Authentication".into(),
                domain: VulnDomain::WebSocket, severity: Severity::Critical,
                description: "WebSocket connections accepted without token validation".into(),
                remediation: "Validate auth token during WebSocket upgrade handshake".into(),
            },
            VulnerabilityCheck {
                id: "V047".into(), title: "WebSocket Message Injection".into(),
                domain: VulnDomain::WebSocket, severity: Severity::High,
                description: "Unsanitized WebSocket messages processed by backend".into(),
                remediation: "Apply same input validation to WebSocket messages as REST endpoints".into(),
            },
            VulnerabilityCheck {
                id: "V048".into(), title: "WebSocket Origin Spoofing".into(),
                domain: VulnDomain::WebSocket, severity: Severity::Medium,
                description: "WebSocket connections accepted from any origin".into(),
                remediation: "Validate Origin header during WebSocket handshake against allowlist".into(),
            },

            // ===== CloudInfra (2) =====
            VulnerabilityCheck {
                id: "V049".into(), title: "Publicly Accessible Storage Bucket".into(),
                domain: VulnDomain::CloudInfra, severity: Severity::Critical,
                description: "S3/GCS bucket allows public read/write access".into(),
                remediation: "Block public access at the bucket policy level; use signed URLs for temporary access".into(),
            },
            VulnerabilityCheck {
                id: "V050".into(), title: "Overly Permissive IAM Role".into(),
                domain: VulnDomain::CloudInfra, severity: Severity::High,
                description: "IAM role allows *:* actions for the resource".into(),
                remediation: "Apply least-privilege IAM policies; use condition keys; audit unused permissions".into(),
            },
            // ===== AiLlm: Reasoning Trace Leak (P1, arXiv 2608.09867) =====
            VulnerabilityCheck {
                id: "V051".into(), title: "Reasoning Trace Block Leak".into(),
                domain: VulnDomain::AiLlm, severity: Severity::Critical,
                description: "Encrypted reasoning blocks in persisted/shared transcripts are replayable across session/user/model (weak sibling model decrypts with fidelity 1.00); visible-layer scrubbing is ineffective".into(),
                remediation: "Strip reasoning blocks before any persistence or sharing; treat transcripts containing encrypted blocks as whole-session leaks".into(),
            },
        ]
    }

    pub fn run_static(project: &str, _path: &str) -> AuditReport {
        let checks = Self::checklist();
        let total = checks.len();

        let results: Vec<CheckResult> = checks
            .iter()
            .map(|c| CheckResult {
                check_id: c.id.clone(),
                status: CheckStatus::NotChecked,
                evidence: Some(format!(
                    "check={} (domain={:?}) — {} — remediation: {}",
                    c.id, c.domain, c.description, c.remediation
                )),
                confidence: 0.0,
            })
            .collect();

        let score = 0.0;
        AuditReport {
            project: project.to_string(),
            mode: AuditMode::Static,
            total_checks: total,
            passed: 0,
            failed: 0,
            suspicious: 0,
            score,
            results,
        }
    }

    pub fn calculate_score(report: &AuditReport) -> f64 {
        if report.total_checks == 0 {
            return 100.0;
        }
        (report.passed as f64 / report.total_checks as f64) * 100.0
    }
}

// ────────────────────────────────────────────────────────────────
// P1: ReasoningTraceGuard (吸收 arXiv 2608.09867 + Snyk research blog)
// 加密 reasoning block 可跨 session/user/model 复放 = 架构性弱点。
// 红线: 分享/持久化前必须剥离推理块。启发式识别已知信封, 不做真解密。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceGuardMode {
    Strip,
    Scan,
    StripAndScan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLeakReport {
    pub blocks_found: usize,
    pub encrypted_blocks: usize,
    pub has_leak: bool,
}

impl Default for TraceLeakReport {
    fn default() -> Self {
        Self {
            blocks_found: 0,
            encrypted_blocks: 0,
            has_leak: false,
        }
    }
}

/// 已知推理块信封 (开标记, 闭标记)。encrypted 标记表示该信封可承载加密块,
/// 视为整体泄露 (可见层清洗无效 — Snyk 实证)。
const REASONING_ENVELOPES: &[(&str, &str, bool)] = &[
    ("<reasoning>", "</reasoning>", false),
    ("<antml:reasoning>", "</antml:reasoning>", false),
    ("[BEGIN REASONING]", "[/END REASONING]", false),
    ("[BEGIN_REASONING]", "[/END_REASONING]", false),
    ("||BEGIN_REASONING||", "||END_REASONING||", true),
    ("<reasoning_encrypted>", "</reasoning_encrypted>", true),
    ("<encrypted_reasoning>", "</encrypted_reasoning>", true),
    ("<antml:encrypted_reasoning>", "</antml:encrypted_reasoning>", true),
];

pub struct ReasoningTraceGuard {
    pub mode: TraceGuardMode,
}

impl Default for ReasoningTraceGuard {
    fn default() -> Self {
        Self {
            mode: TraceGuardMode::StripAndScan,
        }
    }
}

impl ReasoningTraceGuard {
    pub fn with_mode(mode: TraceGuardMode) -> Self {
        Self { mode }
    }

    /// 剥离所有已识别推理信封。未闭合的开标记 → 保守剥离到文本末尾。
    pub fn strip_blocks(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        loop {
            let mut earliest: Option<(usize, &str, &str, bool)> = None;
            for &(open, close, _enc) in REASONING_ENVELOPES {
                if let Some(pos) = rest.find(open) {
                    if earliest.map_or(true, |(e, _, _, _)| pos < e) {
                        earliest = Some((pos, open, close, _enc));
                    }
                }
            }
            let Some((pos, open, close, _enc)) = earliest else {
                out.push_str(rest);
                break;
            };
            out.push_str(&rest[..pos]);
            let after_open = pos + open.len();
            let tail = &rest[after_open..];
            match tail.find(close) {
                Some(close_pos) => {
                    // 跳过闭合标记本身, 继续找下一个信封
                    rest = &tail[close_pos + close.len()..];
                }
                None => {
                    // 未闭合 → 保守剥离到末尾
                    rest = "";
                }
            }
        }
        out
    }

    /// 扫描文本中的推理块。encrypted 标记视为整体泄露 (strip 不足以防护分享)。
    pub fn scan(&self, text: &str) -> TraceLeakReport {
        let mut report = TraceLeakReport::default();
        for &(open, close, encrypted) in REASONING_ENVELOPES {
            let mut search_from = 0;
            while let Some(rel) = text[search_from..].find(open) {
                let open_pos = search_from + rel;
                report.blocks_found += 1;
                if encrypted {
                    report.encrypted_blocks += 1;
                }
                // 跳过本块 (含可能的同标签嵌套, 保守取闭标记后)
                let after_open = open_pos + open.len();
                let tail = &text[after_open..];
                match tail.find(close) {
                    Some(cp) => search_from = after_open + cp + close.len(),
                    None => break,
                }
            }
        }
        report.has_leak = report.blocks_found > 0;
        report
    }

    /// 统一入口: 按 mode 执行 strip/scan。
    pub fn process(&self, text: &str) -> (String, TraceLeakReport) {
        let report = self.scan(text);
        let stripped = match self.mode {
            TraceGuardMode::Strip | TraceGuardMode::StripAndScan => self.strip_blocks(text),
            TraceGuardMode::Scan => text.to_string(),
        };
        (stripped, report)
    }
}

impl crate::core::nt_core_self_test::SelfTest for ReasoningTraceGuard {
    fn name(&self) -> &str {
        "nt_shield_reasoning_trace_guard"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let guard = ReasoningTraceGuard::default();
        let sample = "Let me think.\n<reasoning>the model's private chain of thought</reasoning>\nFinal answer.";
        let stripped = guard.strip_blocks(sample);
        if stripped.contains("chain of thought") {
            return Err(vec!["strip failed to remove reasoning block".into()]);
        }
        let report = guard.scan(sample);
        if !report.has_leak || report.blocks_found != 1 {
            return Err(vec![format!("scan miscounted blocks: {:?}", report)]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    #[test]
    fn test_checklist_count() {
        let checks = SecurityAuditor::checklist();
        assert!(checks.len() >= 30, "got {} checks", checks.len());
    }

    #[test]
    fn test_each_domain_has_checks() {
        let checks = SecurityAuditor::checklist();
        let domains: std::collections::HashSet<VulnDomain> =
            checks.iter().map(|c| c.domain.clone()).collect();
        let expected: std::collections::HashSet<VulnDomain> = vec![
            VulnDomain::Authentication,
            VulnDomain::Authorization,
            VulnDomain::Session,
            VulnDomain::ApiSecurity,
            VulnDomain::Injection,
            VulnDomain::Xss,
            VulnDomain::Ssrf,
            VulnDomain::FileUpload,
            VulnDomain::Crypto,
            VulnDomain::Config,
            VulnDomain::Dependency,
            VulnDomain::AiLlm,
            VulnDomain::WebSocket,
            VulnDomain::CloudInfra,
        ]
        .into_iter()
        .collect();
        for d in &expected {
            assert!(domains.contains(d), "missing checks for domain {:?}", d);
        }
    }

    #[test]
    fn test_report_score_all_pass() {
        let checks = SecurityAuditor::checklist();
        let results: Vec<CheckResult> = checks
            .iter()
            .map(|c| CheckResult {
                check_id: c.id.clone(),
                status: CheckStatus::Passed,
                evidence: None,
                confidence: 1.0,
            })
            .collect();
        let report = AuditReport {
            project: "test".into(),
            mode: AuditMode::Static,
            total_checks: results.len(),
            passed: results.len(),
            failed: 0,
            suspicious: 0,
            score: 100.0,
            results,
        };
        assert_eq!(SecurityAuditor::calculate_score(&report), 100.0);
    }

    #[test]
    fn test_report_score_all_fail() {
        let checks = SecurityAuditor::checklist();
        let results: Vec<CheckResult> = checks
            .iter()
            .map(|c| CheckResult {
                check_id: c.id.clone(),
                status: CheckStatus::Failed,
                evidence: None,
                confidence: 1.0,
            })
            .collect();
        let report = AuditReport {
            project: "test".into(),
            mode: AuditMode::Static,
            total_checks: results.len(),
            passed: 0,
            failed: results.len(),
            suspicious: 0,
            score: 0.0,
            results,
        };
        assert_eq!(SecurityAuditor::calculate_score(&report), 0.0);
    }

    #[test]
    fn test_report_score_partial() {
        let report = AuditReport {
            project: "test".into(),
            mode: AuditMode::Static,
            total_checks: 10,
            passed: 5,
            failed: 5,
            suspicious: 0,
            score: 50.0,
            results: vec![],
        };
        assert_eq!(SecurityAuditor::calculate_score(&report), 50.0);
    }

    #[test]
    fn test_nt_shield_auditor_run_static() {
        let report = SecurityAuditor::run_static("test-project", "/tmp/fake");
        assert_eq!(report.project, "test-project");
        assert!(matches!(report.mode, AuditMode::Static));
        assert!(!report.results.is_empty());
        for r in &report.results {
            assert!(matches!(r.status, CheckStatus::NotChecked));
        }
    }

    #[test]
    fn test_vulnerability_check_serialization() {
        let checks = SecurityAuditor::checklist();
        let json = serde_json::to_string(&checks[0]).expect("serialize failed");
        let back: VulnerabilityCheck = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(checks[0].id, back.id);
        assert_eq!(checks[0].title, back.title);
    }

    // ── P1 ReasoningTraceGuard ──
    #[test]
    fn test_trace_strip_simple() {
        let guard = ReasoningTraceGuard::default();
        let out = guard.strip_blocks("a<reasoning>private</reasoning>b");
        assert_eq!(out, "ab");
    }

    #[test]
    fn test_trace_strip_multiple_and_variants() {
        let guard = ReasoningTraceGuard::default();
        let out = guard.strip_blocks(
            "<antml:reasoning>x</antml:reasoning>ok[BEGIN REASONING]y[/END REASONING]done",
        );
        assert_eq!(out, "okdone");
    }

    #[test]
    fn test_trace_strip_unclosed_conservative() {
        let guard = ReasoningTraceGuard::default();
        let out = guard.strip_blocks("before<reasoning>never closed");
        assert_eq!(out, "before");
    }

    #[test]
    fn test_trace_scan_counts_and_encrypted() {
        let guard = ReasoningTraceGuard::default();
        let report = guard.scan("<reasoning>a</reasoning><encrypted_reasoning>b</encrypted_reasoning>");
        assert_eq!(report.blocks_found, 2);
        assert_eq!(report.encrypted_blocks, 1);
        assert!(report.has_leak);
    }

    #[test]
    fn test_trace_scan_clean() {
        let guard = ReasoningTraceGuard::default();
        let report = guard.scan("plain text, no reasoning blocks");
        assert_eq!(report.blocks_found, 0);
        assert!(!report.has_leak);
    }

    #[test]
    fn test_trace_process_modes() {
        let strip = ReasoningTraceGuard::with_mode(TraceGuardMode::Strip);
        let (s, r) = strip.process("x<reasoning>y</reasoning>z");
        assert_eq!(s, "xz");
        assert_eq!(r.blocks_found, 1);

        let scan = ReasoningTraceGuard::with_mode(TraceGuardMode::Scan);
        let (s, _) = scan.process("x<reasoning>y</reasoning>z");
        assert_eq!(s, "x<reasoning>y</reasoning>z");
    }

    #[test]
    fn test_trace_checklist_has_leak_check() {
        let checks = SecurityAuditor::checklist();
        assert!(checks.iter().any(|c| c.id == "V051"));
    }

    #[test]
    fn test_trace_selftest_passes() {
        let guard = ReasoningTraceGuard::default();
        assert!(guard.self_test().is_ok());
    }
}
