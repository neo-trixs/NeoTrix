use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditMode {
    Static,
    DynamicSafe,
    DynamicActive,
    OnlineAuthorized,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShieldSeverity {
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
    pub severity: ShieldSeverity,
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
                domain: VulnDomain::Authentication, severity: ShieldSeverity::High,
                description: "Password policy does not enforce minimum complexity requirements".into(),
                remediation: "Enforce minimum 12 chars, mixed case, digits, and special characters".into(),
            },
            VulnerabilityCheck {
                id: "V002".into(), title: "Missing Multi-Factor Authentication".into(),
                domain: VulnDomain::Authentication, severity: ShieldSeverity::Critical,
                description: "No MFA enforced on privileged accounts or sensitive operations".into(),
                remediation: "Implement TOTP, WebAuthn, or SMS-based MFA for all privileged access".into(),
            },
            VulnerabilityCheck {
                id: "V003".into(), title: "Session Fixation".into(),
                domain: VulnDomain::Authentication, severity: ShieldSeverity::High,
                description: "Session identifiers are not regenerated after login".into(),
                remediation: "Regenerate session ID on successful authentication via session.regenerate()".into(),
            },
            VulnerabilityCheck {
                id: "V004".into(), title: "JWT None Algorithm".into(),
                domain: VulnDomain::Authentication, severity: ShieldSeverity::Critical,
                description: "JWT library accepts 'none' algorithm, allowing forged tokens".into(),
                remediation: "Explicitly reject 'none' algorithm; always validate algorithm header".into(),
            },
            VulnerabilityCheck {
                id: "V005".into(), title: "Credential Exposure in Logs".into(),
                domain: VulnDomain::Authentication, severity: ShieldSeverity::Critical,
                description: "Credentials or tokens may be logged in plaintext".into(),
                remediation: "Implement credential scrubbing in log pipelines; never log request bodies with passwords".into(),
            },

            // ===== Authorization (5) — reuse VulnDomain::Authorization =====
            VulnerabilityCheck {
                id: "V006".into(), title: "Broken Object-Level Authorization".into(),
                domain: VulnDomain::Authorization, severity: ShieldSeverity::Critical,
                description: "API endpoints lack ownership checks on resource access by ID".into(),
                remediation: "Verify user identity against resource owner before returning data".into(),
            },
            VulnerabilityCheck {
                id: "V007".into(), title: "Mass Assignment".into(),
                domain: VulnDomain::Authorization, severity: ShieldSeverity::High,
                description: "User input is directly bound to model fields without allowlisting".into(),
                remediation: "Use DTOs or allowlists to control which fields can be mass-assigned".into(),
            },
            VulnerabilityCheck {
                id: "V008".into(), title: "Privilege Escalation via API".into(),
                domain: VulnDomain::Authorization, severity: ShieldSeverity::Critical,
                description: "Low-privilege users can access admin-level endpoints".into(),
                remediation: "Enforce role-based access control on every endpoint".into(),
            },
            VulnerabilityCheck {
                id: "V009".into(), title: "Insecure Direct Object Reference".into(),
                domain: VulnDomain::Authorization, severity: ShieldSeverity::High,
                description: "Internal IDs exposed in URLs allow enumeration of resources".into(),
                remediation: "Use opaque UUIDs instead of sequential IDs; verify ownership server-side".into(),
            },
            VulnerabilityCheck {
                id: "V010".into(), title: "Missing Function-Level Access Control".into(),
                domain: VulnDomain::Authorization, severity: ShieldSeverity::High,
                description: "Controller methods lack authorization annotations".into(),
                remediation: "Apply @PreAuthorize or middleware guards to every handler".into(),
            },

            // ===== Injection (5) =====
            VulnerabilityCheck {
                id: "V011".into(), title: "SQL Injection".into(),
                domain: VulnDomain::Injection, severity: ShieldSeverity::Critical,
                description: "User input concatenated into SQL queries without parameterization".into(),
                remediation: "Use parameterized queries or ORM with bound parameters".into(),
            },
            VulnerabilityCheck {
                id: "V012".into(), title: "Command Injection".into(),
                domain: VulnDomain::Injection, severity: ShieldSeverity::Critical,
                description: "User input passed directly to shell command execution".into(),
                remediation: "Avoid shell execution; use safe APIs with arg vectors; validate and sanitize input".into(),
            },
            VulnerabilityCheck {
                id: "V013".into(), title: "Server-Side Template Injection".into(),
                domain: VulnDomain::Injection, severity: ShieldSeverity::High,
                description: "User input rendered as template content".into(),
                remediation: "Pre-compile templates; never treat user input as template source".into(),
            },
            VulnerabilityCheck {
                id: "V014".into(), title: "NoSQL Injection".into(),
                domain: VulnDomain::Injection, severity: ShieldSeverity::High,
                description: "Operator injection in MongoDB/NoSQL queries via unvalidated input".into(),
                remediation: "Sanitize input; disable $where/$regex operators on user-facing queries".into(),
            },
            VulnerabilityCheck {
                id: "V015".into(), title: "XPath Injection".into(),
                domain: VulnDomain::Injection, severity: ShieldSeverity::Medium,
                description: "User input embedded directly into XPath queries".into(),
                remediation: "Use parameterized XPath queries or pre-compile expressions".into(),
            },

            // ===== XSS (5) =====
            VulnerabilityCheck {
                id: "V016".into(), title: "Reflected XSS".into(),
                domain: VulnDomain::Xss, severity: ShieldSeverity::High,
                description: "User input reflected in HTTP response without sanitization".into(),
                remediation: "Apply context-aware output encoding; use CSP headers; validate input".into(),
            },
            VulnerabilityCheck {
                id: "V017".into(), title: "Stored XSS".into(),
                domain: VulnDomain::Xss, severity: ShieldSeverity::High,
                description: "User input persisted then rendered without sanitization".into(),
                remediation: "Sanitize on both input and output; use Content-Security-Policy".into(),
            },
            VulnerabilityCheck {
                id: "V018".into(), title: "DOM-Based XSS".into(),
                domain: VulnDomain::Xss, severity: ShieldSeverity::High,
                description: "Client-side JavaScript writes user input to DOM unsafely".into(),
                remediation: "Use textContent instead of innerHTML; avoid eval-like sinks".into(),
            },
            VulnerabilityCheck {
                id: "V019".into(), title: "Unsafe innerHTML Usage".into(),
                domain: VulnDomain::Xss, severity: ShieldSeverity::Medium,
                description: "dangerouslySetInnerHTML or innerHTML used with untrusted data".into(),
                remediation: "Use safe rendering APIs; sanitize with DOMPurify before insertion".into(),
            },
            VulnerabilityCheck {
                id: "V020".into(), title: "Missing Content-Security-Policy".into(),
                domain: VulnDomain::Xss, severity: ShieldSeverity::Medium,
                description: "No CSP header set, allowing inline scripts and untrusted sources".into(),
                remediation: "Set strict CSP: script-src 'self'; object-src 'none'; base-uri 'self'".into(),
            },

            // ===== Config (5) =====
            VulnerabilityCheck {
                id: "V021".into(), title: "Hardcoded Secrets in Source".into(),
                domain: VulnDomain::Config, severity: ShieldSeverity::Critical,
                description: "API keys, tokens, or passwords hardcoded in source files".into(),
                remediation: "Move secrets to environment variables or vault service".into(),
            },
            VulnerabilityCheck {
                id: "V022".into(), title: "Debug Mode Enabled in Production".into(),
                domain: VulnDomain::Config, severity: ShieldSeverity::High,
                description: "Debug endpoints or verbose error pages exposed in production".into(),
                remediation: "Disable debug mode; use generic error pages in production".into(),
            },
            VulnerabilityCheck {
                id: "V023".into(), title: "CORS Misconfiguration".into(),
                domain: VulnDomain::Config, severity: ShieldSeverity::Medium,
                description: "Access-Control-Allow-Origin set to wildcard with credentials".into(),
                remediation: "Restrict origins to specific domains; never pair '*' with credentials".into(),
            },
            VulnerabilityCheck {
                id: "V024".into(), title: "Missing Security Headers".into(),
                domain: VulnDomain::Config, severity: ShieldSeverity::Medium,
                description: "HSTS, X-Frame-Options, X-Content-Type-Options headers not set".into(),
                remediation: "Add Strict-Transport-Security, X-Frame-Options: DENY, X-Content-Type-Options: nosniff".into(),
            },
            VulnerabilityCheck {
                id: "V025".into(), title: "Insecure TLS Configuration".into(),
                domain: VulnDomain::Config, severity: ShieldSeverity::High,
                description: "TLS 1.0/1.1 enabled or weak cipher suites accepted".into(),
                remediation: "Disable TLS < 1.2; use only AEAD cipher suites (GCM/ChaCha20)".into(),
            },

            // ===== Dependency (2) =====
            VulnerabilityCheck {
                id: "V026".into(), title: "Known Vulnerable Dependencies".into(),
                domain: VulnDomain::Dependency, severity: ShieldSeverity::Critical,
                description: "Dependencies with known CVEs (e.g., log4shell, zip slip)".into(),
                remediation: "Use `cargo audit` or `npm audit`; enable Dependabot/Renovate; pin versions".into(),
            },
            VulnerabilityCheck {
                id: "V027".into(), title: "Supply Chain Attack".into(),
                domain: VulnDomain::Dependency, severity: ShieldSeverity::High,
                description: "Typo-squatting, compromised maintainer accounts, or malicious packages".into(),
                remediation: "Verify package integrity (SHA256); use lockfiles; restrict registry sources".into(),
            },

            // ===== AI/LLM (5) =====
            VulnerabilityCheck {
                id: "V028".into(), title: "Prompt Injection".into(),
                domain: VulnDomain::AiLlm, severity: ShieldSeverity::Critical,
                description: "User prompts can override system instructions or inject commands".into(),
                remediation: "Use input classification; separate system prompts from user input; apply delimiters".into(),
            },
            VulnerabilityCheck {
                id: "V029".into(), title: "Training Data Poisoning".into(),
                domain: VulnDomain::AiLlm, severity: ShieldSeverity::High,
                description: "Malicious data in training set biases model outputs".into(),
                remediation: "Validate training data provenance; implement data sanitization pipeline".into(),
            },
            VulnerabilityCheck {
                id: "V030".into(), title: "Model Inversion".into(),
                domain: VulnDomain::AiLlm, severity: ShieldSeverity::High,
                description: "Attacker reconstructs training data from model outputs".into(),
                remediation: "Apply differential privacy; limit output verbosity; rate-limit API access".into(),
            },
            VulnerabilityCheck {
                id: "V031".into(), title: "Excessive Agency for LLM Agent".into(),
                domain: VulnDomain::AiLlm, severity: ShieldSeverity::Critical,
                description: "Agent has permissions beyond task scope, enabling privilege escalation".into(),
                remediation: "Apply least-privilege; scope tool access per session; human-in-the-loop for destructive ops".into(),
            },
            VulnerabilityCheck {
                id: "V032".into(), title: "Sensitive Data Leakage via LLM".into(),
                domain: VulnDomain::AiLlm, severity: ShieldSeverity::High,
                description: "Secrets or PII included in LLM context could leak through output".into(),
                remediation: "Scrub sensitive data from context; implement output filters; use redaction".into(),
            },

            // ===== API (4) =====
            VulnerabilityCheck {
                id: "V031".into(), title: "Missing Rate Limiting".into(),
                domain: VulnDomain::ApiSecurity, severity: ShieldSeverity::Medium,
                description: "API endpoints lack request rate limiting".into(),
                remediation: "Implement token bucket or sliding window rate limiter per user/IP".into(),
            },
            VulnerabilityCheck {
                id: "V032".into(), title: "Improper Asset Management".into(),
                domain: VulnDomain::ApiSecurity, severity: ShieldSeverity::Medium,
                description: "Deprecated or shadow API versions still accessible".into(),
                remediation: "Inventory all API endpoints; deprecate with sunset headers; remove old versions".into(),
            },
            VulnerabilityCheck {
                id: "V033".into(), title: "Unvalidated API Input".into(),
                domain: VulnDomain::ApiSecurity, severity: ShieldSeverity::High,
                description: "API request body lacks schema validation".into(),
                remediation: "Apply JSON Schema or strong type validation on all request bodies".into(),
            },
            VulnerabilityCheck {
                id: "V034".into(), title: "Excessive Data Exposure".into(),
                domain: VulnDomain::ApiSecurity, severity: ShieldSeverity::Medium,
                description: "API responses return full objects instead of minimal views".into(),
                remediation: "Use response DTOs; return only fields the client needs".into(),
            },

            // ===== Session (3) =====
            VulnerabilityCheck {
                id: "V035".into(), title: "Weak Session Token Generation".into(),
                domain: VulnDomain::Session, severity: ShieldSeverity::High,
                description: "Session tokens generated with insufficient entropy or predictable seed".into(),
                remediation: "Use cryptographically secure random generator (OsRng) for session tokens".into(),
            },
            VulnerabilityCheck {
                id: "V036".into(), title: "Missing Session Expiry".into(),
                domain: VulnDomain::Session, severity: ShieldSeverity::Medium,
                description: "Sessions never expire or have excessively long timeouts".into(),
                remediation: "Set absolute and idle timeouts; rotate session on privilege escalation".into(),
            },
            VulnerabilityCheck {
                id: "V037".into(), title: "Cookie Without Secure Flags".into(),
                domain: VulnDomain::Session, severity: ShieldSeverity::Medium,
                description: "Session cookies missing HttpOnly, Secure, or SameSite attributes".into(),
                remediation: "Set HttpOnly, Secure, SameSite=Lax on all session cookies".into(),
            },

            // ===== Crypto (3) =====
            VulnerabilityCheck {
                id: "V038".into(), title: "Weak Hashing Algorithm".into(),
                domain: VulnDomain::Crypto, severity: ShieldSeverity::High,
                description: "MD5 or SHA-1 used for password storage or integrity checks".into(),
                remediation: "Use Argon2id or bcrypt for passwords; SHA-256+ for integrity".into(),
            },
            VulnerabilityCheck {
                id: "V039".into(), title: "Non-AEAD Cipher Mode".into(),
                domain: VulnDomain::Crypto, severity: ShieldSeverity::High,
                description: "AES-ECB or CBC mode used without authentication tag".into(),
                remediation: "Use AES-GCM or ChaCha20-Poly1305 with random nonce".into(),
            },
            VulnerabilityCheck {
                id: "V040".into(), title: "Insufficient Key Length".into(),
                domain: VulnDomain::Crypto, severity: ShieldSeverity::Medium,
                description: "RSA key < 2048 bits or ECC key < 256 bits used".into(),
                remediation: "Use RSA-2048+ or ECC P-256+; prefer Ed25519 for signing".into(),
            },

            // ===== SSRF (2) =====
            VulnerabilityCheck {
                id: "V041".into(), title: "Server-Side Request Forgery".into(),
                domain: VulnDomain::Ssrf, severity: ShieldSeverity::Critical,
                description: "Application fetches user-supplied URLs without validation".into(),
                remediation: "Allowlist permitted hosts; block private IP ranges; disable redirect following".into(),
            },
            VulnerabilityCheck {
                id: "V042".into(), title: "Cloud Metadata Endpoint Access".into(),
                domain: VulnDomain::Ssrf, severity: ShieldSeverity::Critical,
                description: "SSRF can reach cloud metadata service (169.254.169.254)".into(),
                remediation: "Block link-local and metadata IPs at proxy/firewall level".into(),
            },

            // ===== File Upload (3) =====
            VulnerabilityCheck {
                id: "V043".into(), title: "Unrestricted File Upload".into(),
                domain: VulnDomain::FileUpload, severity: ShieldSeverity::High,
                description: "No file type or size validation on upload endpoints".into(),
                remediation: "Validate MIME type server-side; enforce max file size; scan for malware".into(),
            },
            VulnerabilityCheck {
                id: "V044".into(), title: "Path Traversal in File Upload".into(),
                domain: VulnDomain::FileUpload, severity: ShieldSeverity::High,
                description: "Uploaded file name used without sanitization in path construction".into(),
                remediation: "Use random file names; reject path separators in file names; store outside webroot".into(),
            },
            VulnerabilityCheck {
                id: "V045".into(), title: "Uploaded File Execution".into(),
                domain: VulnDomain::FileUpload, severity: ShieldSeverity::Critical,
                description: "Uploaded files stored in web-accessible directory and can be executed".into(),
                remediation: "Store uploads outside webroot; serve via separate domain with no execution".into(),
            },

            // ===== WebSocket (3) =====
            VulnerabilityCheck {
                id: "V046".into(), title: "WebSocket Without Authentication".into(),
                domain: VulnDomain::WebSocket, severity: ShieldSeverity::Critical,
                description: "WebSocket connections accepted without token validation".into(),
                remediation: "Validate auth token during WebSocket upgrade handshake".into(),
            },
            VulnerabilityCheck {
                id: "V047".into(), title: "WebSocket Message Injection".into(),
                domain: VulnDomain::WebSocket, severity: ShieldSeverity::High,
                description: "Unsanitized WebSocket messages processed by backend".into(),
                remediation: "Apply same input validation to WebSocket messages as REST endpoints".into(),
            },
            VulnerabilityCheck {
                id: "V048".into(), title: "WebSocket Origin Spoofing".into(),
                domain: VulnDomain::WebSocket, severity: ShieldSeverity::Medium,
                description: "WebSocket connections accepted from any origin".into(),
                remediation: "Validate Origin header during WebSocket handshake against allowlist".into(),
            },

            // ===== CloudInfra (2) =====
            VulnerabilityCheck {
                id: "V049".into(), title: "Publicly Accessible Storage Bucket".into(),
                domain: VulnDomain::CloudInfra, severity: ShieldSeverity::Critical,
                description: "S3/GCS bucket allows public read/write access".into(),
                remediation: "Block public access at the bucket policy level; use signed URLs for temporary access".into(),
            },
            VulnerabilityCheck {
                id: "V050".into(), title: "Overly Permissive IAM Role".into(),
                domain: VulnDomain::CloudInfra, severity: ShieldSeverity::High,
                description: "IAM role allows *:* actions for the resource".into(),
                remediation: "Apply least-privilege IAM policies; use condition keys; audit unused permissions".into(),
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
                evidence: None,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
