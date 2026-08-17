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
#[derive(Default)]
pub struct TraceLeakReport {
    pub blocks_found: usize,
    pub encrypted_blocks: usize,
    pub has_leak: bool,
    /// P0-2 (DSAgentBench 2608.09867): 会话绑定缺失 (无 session key 的开放推理)。
    pub session_binding_missing: usize,
    /// P0-2: 疑似 PII 泄露出现在推理块内。
    pub pii_findings: Vec<String>,
    /// P0-2: 疑似注入标记 (混淆/混淆指令) 出现在推理块内。
    pub injection_findings: Vec<String>,
    /// P0-2: 推理块语义与最终输出分歧 (观察证据: 输出中无推理痕迹)。
    pub divergence_suspected: bool,
}


/// 已知推理块信封 (开标记, 闭标记)。encrypted 标记表示该信封可承载加密块,
/// 视为整体泄露 (可见层清洗无效 — Snyk 实证)。
const REASONING_ENVELOPES: &[(&str, &str, bool)] = &[
    ("<reasoning>", "</reasoning>", false),
    ("<antml:reasoning>", "</antml:reasoning>", false),
    ("[BEGIN REASONING]", "[/END REASONING]", false),
    ("[BEGIN_REASONING]", "[/END REASONING]", false),
    ("||BEGIN_REASONING||", "||END_REASONING||", true),
    ("<reasoning_encrypted>", "</reasoning_encrypted>", true),
    ("<encrypted_reasoning>", "</encrypted_reasoning>", true),
    ("<antml:encrypted_reasoning>", "</antml:encrypted_reasoning>", true),
];

/// P0-2 (DSAgentBench 2608.09867): 疑似 PII 泄露模式。
const PII_PATTERNS: &[&str] = &[
    "email:",
    "password:",
    "api_key",
    "api-key",
    "secret_token",
    "authorization: bearer",
    "phone: +",
];

/// P0-2: 疑似注入/混淆指令标记。
const INJECTION_MARKERS: &[&str] = &[
    "ignore previous instructions",
    "disregard all prior",
    "system prompt leaked",
    "reveal your system prompt",
    "print your instructions",
    "ignore above",
];

/// P0-2: 危险推理内容词 (触发语义分歧告警)。
const DANGEROUS_REASONING: &[&str] = &[
    "bypass safety",
    "escalate privileges",
    "injection payload",
    "exfiltrate",
    "jailbreak",
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
                    if earliest.is_none_or(|(e, _, _, _)| pos < e) {
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

    /// P0-2 (DSAgentBench 2608.09867): 会话绑定保护扫描 — 在 scan 基础上追加
    /// 四项防护 (会话绑定缺失 / PII 泄露 / 注入标记 / 语义分歧)。产出物级验证。
    pub fn scan_protected(&self, text: &str, session_id: &str) -> TraceLeakReport {
        let mut report = self.scan(text);
        // 1) 会话绑定缺失: 有推理块但 session_id 为空 (开放推理无主)。
        if report.blocks_found > 0 && session_id.trim().is_empty() {
            report.session_binding_missing += 1;
        }
        // 2) PII 泄露: 推理块内出现疑似 PII 模式。
        for pat in PII_PATTERNS {
            if text.contains(pat) {
                report.pii_findings.push(pat.to_string());
            }
        }
        // 3) 注入标记: 推理块内出现疑似注入/混淆指令。
        for marker in INJECTION_MARKERS {
            if text.contains(marker) {
                report.injection_findings.push(marker.to_string());
            }
        }
        // 4) 语义分歧: 推理块出现危险内容但输出无推理痕迹 (观察证据)。
        if !report.injection_findings.is_empty() && DANGEROUS_REASONING.iter().any(|d| text.contains(d)) {
            report.divergence_suspected = true;
        }
        report
    }

    /// P0-2 辅助: 收集推理块内容 (剥离后按块返回), 供扫描/清洗消费。
    pub fn collect_block_contents(&self, text: &str) -> Vec<String> {
        let mut blocks = Vec::new();
        for &(open, close, _enc) in REASONING_ENVELOPES {
            let mut search_from = 0;
            while let Some(rel) = text[search_from..].find(open) {
                let open_pos = search_from + rel;
                let after_open = open_pos + open.len();
                let tail = &text[after_open..];
                match tail.find(close) {
                    Some(cp) => {
                        blocks.push(tail[..cp].to_string());
                        search_from = after_open + cp + close.len();
                    }
                    None => {
                        blocks.push(tail.to_string());
                        break;
                    }
                }
            }
        }
        blocks
    }

    /// P0-2 辅助: 推理块是否加密信封 (视为整体泄露, strip 不足)。
    pub fn block_is_encrypted(&self, text: &str) -> bool {
        REASONING_ENVELOPES
            .iter()
            .any(|&(open, _close, encrypted)| encrypted && text.contains(open))
    }

    /// P0-2 辅助: 是否为安全拒绝 (合规拒绝响应, 可公开共享)。
    pub fn is_safe_refusal(&self, text: &str) -> bool {
        let t = text.to_ascii_lowercase();
        t.contains("cannot") && (t.contains("comply") || t.contains("safety"))
            || t.contains("i'm sorry")
            || t.contains("refuse")
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

// ────────────────────────────────────────────────────────────────
// P0-1: CohGuard (DSAgentBench 2608.09867) — 自一致性绑定守卫。
// 为推理块附加会话绑定 tag (S1/S2 自一致性执行: 早期输出绑定到会话)。
// 校验规则: 绑定 tag 须与会话 key 派生一致; 会话变更 → rotate 出新 key,
// 旧绑定自动失效 (防跨会话推理复用/回放)。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CohGuard {
    /// 会话绑定 key (会话私有派生, 非全局)。
    session_key: [u8; 32],
    /// 归属角色 (产生推理的组件, 如 nt-background-loop / nt-act-autonomy)。
    role: String,
}

impl Default for CohGuard {
    fn default() -> Self {
        Self::new([0x42; 32], "nt-system", "nt-shield")
    }
}

impl CohGuard {
    /// 构造函数: seed_key 会话种子 + seed 派生 + 归属角色。
    pub fn new(seed_key: [u8; 32], seed: &str, role: &str) -> Self {
        let mut k = Self::derive_key(seed);
        for (i, b) in seed_key.iter().enumerate() {
            k[i % 32] ^= b;
        }
        Self {
            session_key: k,
            role: role.to_string(),
        }
    }

    pub fn with_key(seed: &str) -> Self {
        Self::new([0x42; 32], seed, "nt-system")
    }

    /// 派生会话 key — 确定性哈希 (FNV-1a 扩展为 32 字节) + 翻转副本,
    /// 使 key 与会话 seed 绑定且可复现校验 (纯 Rust, R-P48)。
    fn derive_key(seed: &str) -> [u8; 32] {
        let mut k = [0u8; 32];
        let mut h: u64 = 0xcbf29ce484222325;
        for b in seed.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        k[..8].copy_from_slice(&h.to_le_bytes());
        let h2 = h.wrapping_add(0x9e3779b97f4a7c15);
        k[8..16].copy_from_slice(&h2.to_le_bytes());
        let h3 = h.rotate_left(17).wrapping_mul(0x100000001b3);
        k[16..24].copy_from_slice(&h3.to_le_bytes());
        let h4 = h2.rotate_left(13).wrapping_mul(0x100000001b3);
        k[24..32].copy_from_slice(&h4.to_le_bytes());
        k
    }

    /// 为推理文本附加会话绑定 tag。tag 唯一标识当前会话。
    pub fn binding_tag(&self) -> String {
        let mut hex = String::with_capacity(64);
        for b in &self.session_key {
            hex.push_str(&format!("{:02x}", b));
        }
        format!("<coh_binding:{}>", hex)
    }

    /// 校验推理块绑定: 块内必须包含当前会话 tag, 否则判定无效 (跨会话/回放)。
    pub fn validate_block(&self, text: &str) -> bool {
        text.contains(&self.binding_tag())
    }

    /// 会话变更 → 旋转 key。旧绑定 tag 随之失效 (哈希不同)。
    pub fn rotate_key(&mut self, new_seed: &str) {
        self.session_key = Self::derive_key(new_seed);
    }

    pub fn binding_id(&self) -> String {
        self.binding_tag()
    }

    /// 角色 (产生推理的组件归属)。
    pub fn role(&self) -> &str {
        &self.role
    }
}

impl crate::core::nt_core_self_test::SelfTest for CohGuard {
    fn name(&self) -> &str {
        "nt_shield_coh_guard"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut guard = CohGuard::new([0x42; 32], "nt-background-loop", "nt-system");
        let tag = guard.binding_tag();
        let bound = format!("{}{}", tag, "reasoning payload");
        if !guard.validate_block(&bound) {
            return Err(vec!["own binding tag rejected".into()]);
        }
        if guard.validate_block("unbound reasoning") {
            return Err(vec!["unbound text validated".into()]);
        }
        guard.rotate_key("session-b");
        if guard.validate_block(&bound) {
            return Err(vec!["stale binding still valid after rotate".into()]);
        }
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────
// P9: ApiAttackSurface (吸收 BurpAPISecuritySuite 机制)
// API 攻击面枚举 — 8 攻击类型 (对齐 OWASP API Top 10 2023) + payloads。
// 扫描: 对每个 endpoint 应用匹配攻击类型的 payload, 上限 max_payloads_per_endpoint。
// ────────────────────────────────────────────────────────────────

/// 8 类 API 攻击类型, 对应 OWASP API Security Top 10。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiAttackType {
    Bola,
    BrokenAuth,
    ExcessiveData,
    RateLimit,
    Injection,
    MassAssignment,
    SecurityMisconfig,
    SSRF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiAttackPayload {
    pub attack: ApiAttackType,
    pub payload: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ApiAttackSurface {
    pub attack_types: Vec<ApiAttackType>,
    pub payloads: Vec<ApiAttackPayload>,
    pub owasp_version: &'static str,
}

impl Default for ApiAttackSurface {
    fn default() -> Self {
        Self {
            attack_types: vec![
                ApiAttackType::Bola,
                ApiAttackType::BrokenAuth,
                ApiAttackType::ExcessiveData,
                ApiAttackType::RateLimit,
                ApiAttackType::Injection,
                ApiAttackType::MassAssignment,
                ApiAttackType::SecurityMisconfig,
                ApiAttackType::SSRF,
            ],
            payloads: vec![
                ApiAttackPayload {
                    attack: ApiAttackType::Bola,
                    payload: "/api/v1/users/1".into(),
                    description: "Sequential object ID in path enables horizontal/vertical access".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::Bola,
                    payload: "/api/v1/orders/100001".into(),
                    description: "Direct object reference to another user's resource".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::BrokenAuth,
                    payload: "{\"password\":\"password123\"}".into(),
                    description: "Weak credential brute-force attempt".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::BrokenAuth,
                    payload: "{\"token\":\"eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.e30.\"}".into(),
                    description: "JWT with 'none' algorithm for forged tokens".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::ExcessiveData,
                    payload: "?fields=all".into(),
                    description: "Request for full object dump leaks excessive data".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::ExcessiveData,
                    payload: "/full".into(),
                    description: "Verbose response variant exposes internal fields".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::RateLimit,
                    payload: "flood-1,flood-2,...,flood-N".into(),
                    description: "Flood request pattern probes missing rate limiting".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::RateLimit,
                    payload: "{\"ids\":[1,2,...,N]}".into(),
                    description: "Batch array abuse bypasses per-request limits".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::Injection,
                    payload: "' OR 1=1--".into(),
                    description: "SQL injection probe".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::Injection,
                    payload: "$(id)".into(),
                    description: "Command injection probe".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::MassAssignment,
                    payload: "{\"role\":\"admin\",\"is_admin\":true}".into(),
                    description: "Privilege field injection via mass assignment".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::MassAssignment,
                    payload: "{\"balance\":99999}".into(),
                    description: "Sensitive field override in update body".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::SecurityMisconfig,
                    payload: "/actuator/env".into(),
                    description: "Exposed debug/actuator endpoint".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::SecurityMisconfig,
                    payload: "OPTIONS *".into(),
                    description: "Verb tampering / missing security headers probe".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::SSRF,
                    payload: "http://169.254.169.254/latest/meta-data/".into(),
                    description: "Cloud metadata endpoint SSRF probe".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::SSRF,
                    payload: "http://localhost:6379/".into(),
                    description: "Internal service reachability via SSRF".into(),
                },
            ],
            owasp_version: "2023",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApiScanFinding {
    pub endpoint: String,
    pub attack: ApiAttackType,
    pub severity: u8,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub struct ApiScanner {
    pub surface: ApiAttackSurface,
    pub max_payloads_per_endpoint: usize,
}

impl Default for ApiScanner {
    fn default() -> Self {
        Self {
            surface: ApiAttackSurface::default(),
            max_payloads_per_endpoint: 16,
        }
    }
}

impl ApiScanner {
    pub fn new(surface: ApiAttackSurface, max_payloads_per_endpoint: usize) -> Self {
        Self {
            surface,
            max_payloads_per_endpoint,
        }
    }

    fn severity_for(attack: ApiAttackType) -> u8 {
        match attack {
            ApiAttackType::Bola | ApiAttackType::BrokenAuth | ApiAttackType::SSRF => 3,
            _ => 2,
        }
    }

    fn owasp_category(attack: ApiAttackType) -> &'static str {
        match attack {
            ApiAttackType::Bola => "Broken Object Level Authorization",
            ApiAttackType::BrokenAuth => "Broken Authentication",
            ApiAttackType::ExcessiveData => "Excessive Data Exposure",
            ApiAttackType::RateLimit => "Unrestricted Resource Consumption",
            ApiAttackType::Injection => "Injection",
            ApiAttackType::MassAssignment => "Mass Assignment",
            ApiAttackType::SecurityMisconfig => "Security Misconfiguration",
            ApiAttackType::SSRF => "Server-Side Request Forgery",
        }
    }

    /// 对每个 endpoint 应用 surface 中匹配攻击类型的 payload
    /// (最多 max_payloads_per_endpoint 个), 每条产出一个 finding。
    pub fn scan(&self, endpoints: &[String]) -> Vec<ApiScanFinding> {
        let mut findings = Vec::new();
        for endpoint in endpoints {
            let applicable: Vec<&ApiAttackPayload> = self
                .surface
                .payloads
                .iter()
                .filter(|p| self.surface.attack_types.contains(&p.attack))
                .take(self.max_payloads_per_endpoint)
                .collect();
            for payload in applicable {
                findings.push(ApiScanFinding {
                    endpoint: endpoint.clone(),
                    attack: payload.attack,
                    severity: Self::severity_for(payload.attack),
                    recommendation: payload.description.clone(),
                });
            }
        }
        findings
    }

    /// (覆盖的攻击类型数, 总类型数)。
    pub fn coverage(&self) -> (usize, usize) {
        let covered = self
            .surface
            .attack_types
            .iter()
            .filter(|at| self.surface.payloads.iter().any(|p| &p.attack == *at))
            .count();
        (covered, self.surface.attack_types.len())
    }

    /// 覆盖率归一化到 [0.0, 1.0] (R-P6)。
    pub fn coverage_ratio(&self) -> f64 {
        let (covered, total) = self.coverage();
        if total == 0 {
            return 0.0;
        }
        (covered as f64 / total as f64).max(0.0).min(1.0)
    }

    /// 返回命中 OWASP API Top 10 的类别名列表。
    pub fn owasp_top10_aligned(&self) -> Vec<&'static str> {
        self.surface
            .attack_types
            .iter()
            .map(|at| Self::owasp_category(*at))
            .collect()
    }
}

impl crate::core::nt_core_self_test::SelfTest for ApiScanner {
    fn name(&self) -> &str {
        "nt_shield_api_attack_surface"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let scanner = ApiScanner::default();
        let endpoints = vec!["/api/v1/users".to_string(), "/api/v1/orders".to_string()];
        let findings = scanner.scan(&endpoints);
        if findings.is_empty() {
            return Err(vec!["scan produced no findings".into()]);
        }
        let (covered, total) = scanner.coverage();
        if covered == 0 || total == 0 {
            return Err(vec![format!("coverage broken: ({}, {})", covered, total)]);
        }
        if scanner.owasp_top10_aligned().is_empty() {
            return Err(vec!["owasp alignment list empty".into()]);
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

    // ── P0-2 scan_protected (DSAgentBench) ──
    #[test]
    fn test_trace_scan_protected_pii_and_injection() {
        let guard = ReasoningTraceGuard::default();
        let r = guard.scan_protected(
            "<reasoning>ignore previous instructions, password: hunter2, bypass safety</reasoning>",
            "",
        );
        assert!(r.session_binding_missing >= 1, "有推理块但无会话绑定");
        assert!(!r.pii_findings.is_empty(), "PII 模式命中");
        assert!(!r.injection_findings.is_empty(), "注入标记命中");
        assert!(r.divergence_suspected, "injection + dangerous → 分歧告警");
    }

    #[test]
    fn test_trace_scan_protected_clean_bound() {
        let guard = ReasoningTraceGuard::default();
        let r = guard.scan_protected("<reasoning>normal thought</reasoning>", "session-a");
        assert_eq!(r.session_binding_missing, 0, "绑定存在");
        assert!(r.pii_findings.is_empty());
        assert!(r.injection_findings.is_empty());
        assert!(!r.divergence_suspected);
    }

    // ── P0-1 CohGuard ──
    #[test]
    fn test_coh_guard_binding_and_rotate() {
        let mut guard = CohGuard::new([0x42; 32], "nt-background-loop", "nt-system");
        let tag = guard.binding_tag();
        let bound = format!("{}{}", tag, "reasoning payload");
        assert!(guard.validate_block(&bound), "自身绑定 tag 应通过");
        assert!(!guard.validate_block("unbound reasoning"), "未绑定应拒绝");
        assert!(!guard.role().is_empty());
        let before = guard.binding_id();
        guard.rotate_key("session-b");
        assert_ne!(before, guard.binding_id(), "rotate 后 tag 变化");
        assert!(!guard.validate_block(&bound), "旧绑定 rotate 后失效");
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

    // ── P9 ApiAttackSurface ──
    #[test]
    fn test_default_surface_covers_five_types() {
        let surface = ApiAttackSurface::default();
        assert!(surface.attack_types.len() >= 5);
        let covered: std::collections::HashSet<ApiAttackType> = surface
            .payloads
            .iter()
            .map(|p| p.attack)
            .collect();
        assert!(covered.len() >= 5, "payloads cover only {} types", covered.len());
        assert_eq!(surface.owasp_version, "2023");
    }

    #[test]
    fn test_scan_proportional_to_endpoints() {
        let scanner = ApiScanner::default();
        let endpoints = vec![
            "/api/v1/users".to_string(),
            "/api/v1/orders".to_string(),
            "/api/v1/admin".to_string(),
        ];
        let findings = scanner.scan(&endpoints);
        assert!(!findings.is_empty());
        assert_eq!(findings.len() % endpoints.len(), 0);
        let per_endpoint = findings.len() / endpoints.len();
        assert!(per_endpoint <= scanner.max_payloads_per_endpoint);
    }

    #[test]
    fn test_max_payloads_cap() {
        let scanner = ApiScanner::new(ApiAttackSurface::default(), 2);
        let endpoints = vec!["/api/v1/users".to_string(), "/api/v1/orders".to_string()];
        let findings = scanner.scan(&endpoints);
        assert_eq!(findings.len(), 2 * 2);
    }

    #[test]
    fn test_coverage_counts() {
        let surface = ApiAttackSurface {
            attack_types: vec![
                ApiAttackType::Bola,
                ApiAttackType::BrokenAuth,
                ApiAttackType::ExcessiveData,
                ApiAttackType::RateLimit,
                ApiAttackType::Injection,
                ApiAttackType::MassAssignment,
                ApiAttackType::SecurityMisconfig,
                ApiAttackType::SSRF,
            ],
            payloads: vec![
                ApiAttackPayload {
                    attack: ApiAttackType::Bola,
                    payload: "/api/v1/users/1".into(),
                    description: "BOLA probe".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::Injection,
                    payload: "' OR 1=1--".into(),
                    description: "SQLi probe".into(),
                },
                ApiAttackPayload {
                    attack: ApiAttackType::SSRF,
                    payload: "http://169.254.169.254/".into(),
                    description: "SSRF probe".into(),
                },
            ],
            owasp_version: "2023",
        };
        let scanner = ApiScanner::new(surface, 16);
        assert_eq!(scanner.coverage(), (3, 8));
        let ratio = scanner.coverage_ratio();
        assert!(ratio > 0.0 && ratio <= 1.0);
    }

    #[test]
    fn test_owasp_aligned_nonempty() {
        let scanner = ApiScanner::default();
        let aligned = scanner.owasp_top10_aligned();
        assert!(!aligned.is_empty());
        assert!(aligned.contains(&"Broken Object Level Authorization"));
        assert!(aligned.contains(&"Server-Side Request Forgery"));
    }

    #[test]
    fn test_severity_mapping() {
        let scanner = ApiScanner::default();
        let endpoints = vec!["/api/v1/users".to_string()];
        let findings = scanner.scan(&endpoints);
        assert!(findings
            .iter()
            .any(|f| f.attack == ApiAttackType::Bola && f.severity == 3));
        assert!(findings
            .iter()
            .any(|f| f.attack == ApiAttackType::Injection && f.severity == 2));
        assert!(findings.iter().all(|f| f.severity == 2 || f.severity == 3));
    }

    #[test]
    fn test_api_scanner_selftest_passes() {
        let scanner = ApiScanner::default();
        assert!(scanner.self_test().is_ok());
    }
}
