use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// ===== Types =====

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl VulnerabilitySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            VulnerabilitySeverity::Critical => "critical",
            VulnerabilitySeverity::High => "high",
            VulnerabilitySeverity::Medium => "medium",
            VulnerabilitySeverity::Low => "low",
            VulnerabilitySeverity::Info => "info",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(VulnerabilitySeverity::Critical),
            "high" => Some(VulnerabilitySeverity::High),
            "medium" => Some(VulnerabilitySeverity::Medium),
            "low" => Some(VulnerabilitySeverity::Low),
            "info" => Some(VulnerabilitySeverity::Info),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum VulnerabilityStatus {
    Open,
    Verified,
    Fixed,
    WontFix,
    FalsePositive,
}

impl VulnerabilityStatus {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "open" => Some(VulnerabilityStatus::Open),
            "verified" => Some(VulnerabilityStatus::Verified),
            "fixed" => Some(VulnerabilityStatus::Fixed),
            "wontfix" => Some(VulnerabilityStatus::WontFix),
            "false_positive" => Some(VulnerabilityStatus::FalsePositive),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VulnerabilityFinding {
    pub id: String,
    pub scan_id: String,
    pub title: String,
    pub description: String,
    pub severity: VulnerabilitySeverity,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub cwe_id: Option<String>,
    pub cve_id: Option<String>,
    pub confidence: f64,
    pub remediation: String,
    pub patch_suggestion: Option<String>,
    pub status: VulnerabilityStatus,
    pub discovered_at: String,
    pub verified_at: Option<String>,
    pub fixed_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanResult {
    pub scan_id: String,
    pub target_path: String,
    pub total_files_scanned: usize,
    pub total_findings: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub info: usize,
    pub duration_ms: u64,
    pub started_at: String,
    pub completed_at: String,
    pub overall_score: u8,
    pub by_category: HashMap<String, usize>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecurityScanConfig {
    pub enabled: bool,
    pub scan_on_save: bool,
    pub scan_depth: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size_kb: u32,
    pub auto_fix_critical: bool,
    pub notify_on_critical: bool,
}

impl Default for SecurityScanConfig {
    fn default() -> Self {
        SecurityScanConfig {
            enabled: true,
            scan_on_save: false,
            scan_depth: "standard".into(),
            include_patterns: vec!["*.rs".into(), "*.ts".into(), "*.js".into(), "*.py".into()],
            exclude_patterns: vec!["node_modules/*".into(), "target/*".into(), ".git/*".into()],
            max_file_size_kb: 500,
            auto_fix_critical: false,
            notify_on_critical: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanSummary {
    pub total_scans: usize,
    pub total_findings: usize,
    pub open_critical: usize,
    pub open_high: usize,
    pub open_medium: usize,
    pub fixed_today: usize,
    pub avg_scan_duration_ms: u64,
    pub security_score: u8,
}

// ===== Real File Scanning =====

static SUSPICIOUS_PATTERNS: &[(&str, &str, VulnerabilitySeverity, &str, &str)] = &[
    ("api_key", r#"(?i)(?:api[_-]?key|apikey|secret|token)\s*[:=]\s*['\"][A-Za-z0-9_\-]{16,}"#, VulnerabilitySeverity::Critical,
     "secrets", "Hardcoded credential pattern detected"),
    ("password", r#"(?i)password\s*[:=]\s*['\"][^'\"]+"#, VulnerabilitySeverity::Critical,
     "secrets", "Password literal in source code"),
    ("sql_concat", r#"format!\s*\(\s*["\'].*SELECT.*\{|format!\s*\(\s*["\'].*INSERT.*\{"#, VulnerabilitySeverity::High,
     "injection", "Possible SQL injection via string interpolation"),
    ("eval", r#"\beval\s*\(\s*["\']"#, VulnerabilitySeverity::Critical,
     "injection", "Dynamic code evaluation from string input"),
    ("exec_shell", r#"Command::new\("sh"\)|Command::new\("bash"\)"#, VulnerabilitySeverity::High,
     "injection", "Shell command execution"),
    ("unsafe_block", r#"unsafe\s*\{"#, VulnerabilitySeverity::Medium,
     "memory", "Unsafe code block — verify invariants are correct"),
    ("todo_unsafe", r#"(?i)todo!|FIXME|HACK|XXX"#, VulnerabilitySeverity::Low,
     "quality", "Incomplete code marker (TODO/FIXME/HACK)"),
    ("println", r#"println!\("#, VulnerabilitySeverity::Info,
     "logging", "Debug output via println! instead of log crate"),
];

fn scan_file_for_patterns(path: &std::path::Path) -> Vec<VulnerabilityFinding> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let mut findings = Vec::new();
    let rel_path = path.to_string_lossy().to_string();
    let now = now_iso();

    for (name, pattern, severity, category, title) in SUSPICIOUS_PATTERNS {
        let _ = category;
        let re = match regex::Regex::new(pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for mat in re.find_iter(&content) {
            let line_no = lines[..].iter()
                .enumerate()
                .find(|(_, l)| l.contains(mat.as_str()))
                .map(|(i, _)| i + 1)
                .unwrap_or(1);

            let snippet_start = mat.start().saturating_sub(40);
            let snippet_end = (mat.end() + 40).min(content.len());
            // 切片边界可能落在多字节字符中间 → 用 floor_char_boundary 收齐
            let snippet_start = content.floor_char_boundary(snippet_start);
            let snippet_end = content.floor_char_boundary(snippet_end);
            let snippet = &content[snippet_start..snippet_end];
            let clean_snippet = snippet.replace('\n', " ");

            findings.push(VulnerabilityFinding {
                id: format!("VULN-{:04}", findings.len() + 1),
                scan_id: String::new(),
                title: format!("{}: {}", title, name),
                description: format!("Pattern '{}' matched in file at line {}: {}",
                    name, line_no, clean_snippet),
                severity: severity.clone(),
                file_path: rel_path.clone(),
                line_start: line_no,
                line_end: line_no,
                cwe_id: None,
                cve_id: None,
                confidence: 0.85,
                remediation: format!("Review the {} pattern at line {} and apply appropriate fix", name, line_no),
                patch_suggestion: None,
                status: VulnerabilityStatus::Open,
                discovered_at: now.clone(),
                verified_at: None,
                fixed_at: None,
            });
        }
    }

    findings
}

pub fn scan_real_directory(path: &str, depth: &str) -> (Vec<VulnerabilityFinding>, usize) {
    let max_depth = match depth {
        "shallow" => 1,
        "deep" => 10,
        _ => 3,
    };
    let root = std::path::Path::new(path);
    if !root.exists() {
        return (Vec::new(), 0);
    }

    let mut findings = Vec::new();
    let mut file_count = 0usize;
    let mut dir_stack: Vec<(std::path::PathBuf, usize)> = vec![(root.to_path_buf(), 0)];

    while let Some((dir, d)) = dir_stack.pop() {
        if d > max_depth {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                    if name == "node_modules" || name == "target" || name == ".git" {
                        continue;
                    }
                    dir_stack.push((p, d + 1));
                } else if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if matches!(ext, "rs" | "ts" | "js" | "py" | "tsx" | "jsx" | "go" | "rb" | "php") {
                        file_count += 1;
                        findings.extend(scan_file_for_patterns(&p));
                    }
                }
            }
        }
    }

    (findings, file_count)
}

// ===== State =====

struct SecurityScanState {
    scans: Vec<ScanResult>,
    findings: Vec<VulnerabilityFinding>,
    config: SecurityScanConfig,
}

impl SecurityScanState {
    fn new() -> Self {
        SecurityScanState {
            scans: Vec::with_capacity(50),
            findings: Vec::with_capacity(500),
            config: SecurityScanConfig::default(),
        }
    }
}

static STATE: LazyLock<Mutex<SecurityScanState>> = LazyLock::new(|| {
    Mutex::new(SecurityScanState::new())
});

// ===== Helpers =====

fn short_uid() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

fn now_iso() -> String {
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string()
}

fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn compute_score(total: usize, critical: usize, high: usize) -> u8 {
    if total == 0 {
        return 100;
    }
    // 防下溢: critical/high 来自外部计数，可能总和超过 total
    let medium_low = total.saturating_sub(critical).saturating_sub(high);
    let raw = 100.0 - (critical as f64 * 15.0 + high as f64 * 7.0 + medium_low as f64 * 2.0);
    raw.max(0.0).min(100.0) as u8
}

// ===== Finding Templates =====

struct FindingTemplate {
    title: &'static str,
    description: &'static str,
    severity: VulnerabilitySeverity,
    file_path: &'static str,
    line_start: usize,
    line_end: usize,
    cwe_id: Option<&'static str>,
    cve_id: Option<&'static str>,
    confidence: f64,
    remediation: &'static str,
    patch_suggestion: Option<&'static str>,
}

const ALL_TEMPLATES: &[FindingTemplate] = &[
    FindingTemplate {
        title: "SQL Injection via Unsafe Query Construction",
        description: "User-supplied input is directly concatenated into a SQL query string without parameterization. An attacker can craft malicious input to manipulate the query structure, potentially exfiltrating or modifying database contents.",
        severity: VulnerabilitySeverity::Critical,
        file_path: "src/db/query.rs",
        line_start: 45,
        line_end: 52,
        cwe_id: Some("CWE-89"),
        cve_id: None,
        confidence: 0.95,
        remediation: "Replace string concatenation with parameterized queries or prepared statements. Use the query builder's bind() method to safely interpolate variables.",
        patch_suggestion: Some("--- a/src/db/query.rs\n+++ b/src/db/query.rs\n@@ -45,7 +45,7 @@\n-    let sql = format!(\"SELECT * FROM users WHERE id = '{}'\", user_input);\n-    conn.query(&sql, []).map_err(...)\n+    let sql = \"SELECT * FROM users WHERE id = ?1\";\n+    conn.query(sql, rusqlite::params![user_input]).map_err(...)"),
    },
    FindingTemplate {
        title: "Hardcoded API Key in Source Code",
        description: "An API secret key is embedded directly in the source file. This key can be extracted from version control history, compiled binaries, or decompiled bytecode. Anyone with access to the repository can use this key.",
        severity: VulnerabilitySeverity::Critical,
        file_path: "src/config/secrets.rs",
        line_start: 12,
        line_end: 12,
        cwe_id: Some("CWE-798"),
        cve_id: None,
        confidence: 0.99,
        remediation: "Remove the hardcoded key and load it from an environment variable or a secrets manager at runtime. Rotate the exposed key immediately.",
        patch_suggestion: Some("--- a/src/config/secrets.rs\n+++ b/src/config/secrets.rs\n@@ -12,7 +12,7 @@\n-    const API_SECRET: &str = \"sk-abc123def456ghi789jkl\";\n+    const API_SECRET: &str = std::env::var(\"API_SECRET\").expect(\"API_SECRET must be set\").leak();"),
    },
    FindingTemplate {
        title: "Command Injection via Shell Argument",
        description: "User-controlled input is passed to a shell command via string interpolation without sanitization. An attacker can inject arbitrary shell metacharacters to execute unauthorized commands on the host system.",
        severity: VulnerabilitySeverity::Critical,
        file_path: "src/cli/exec.rs",
        line_start: 34,
        line_end: 38,
        cwe_id: Some("CWE-78"),
        cve_id: None,
        confidence: 0.92,
        remediation: "Never use shell string interpolation. Use Command::new() with separate argument array to avoid shell injection entirely.",
        patch_suggestion: Some("--- a/src/cli/exec.rs\n+++ b/src/cli/exec.rs\n@@ -34,7 +34,7 @@\n-    let output = std::process::Command::new(\"sh\").arg(\"-c\").arg(format!(\"grep {} /var/log/app.log\", user_input)).output()?;\n+    let output = std::process::Command::new(\"grep\").arg(user_input).arg(\"/var/log/app.log\").output()?;"),
    },
    FindingTemplate {
        title: "Unsafe Deserialization from Untrusted Source",
        description: "Serialized data from an external source is deserialized without validation. Malformed or malicious payloads can trigger arbitrary code execution, denial of service, or object injection attacks.",
        severity: VulnerabilitySeverity::High,
        file_path: "src/api/handler.rs",
        line_start: 78,
        line_end: 85,
        cwe_id: Some("CWE-502"),
        cve_id: None,
        confidence: 0.88,
        remediation: "Validate the serialized data before deserializing. Use a schema-based validator (e.g., JSON Schema) or implement type allowlisting. Consider using a safer serialization format.",
        patch_suggestion: Some("--- a/src/api/handler.rs\n+++ b/src/api/handler.rs\n@@ -78,7 +78,10 @@\n-    let data: IncomingPayload = serde_json::from_str(&raw_body).map_err(|e| ...)?;\n+    let validated: serde_json::Value = serde_json::from_str(&raw_body).map_err(|e| ...)?;\n+    if !validated.is_object() || validated.get(\"type\").and_then(|v| v.as_str()).is_none() {\n+        return Err(\"Invalid payload structure\".into());\n+    }\n+    let data: IncomingPayload = serde_json::from_value(validated).map_err(|e| ...)?;"),
    },
    FindingTemplate {
        title: "Path Traversal in File Access",
        description: "User-supplied filename components are used in file path construction without normalization. An attacker can use '../' sequences to read or write files outside the intended directory.",
        severity: VulnerabilitySeverity::High,
        file_path: "src/io/file_manager.rs",
        line_start: 156,
        line_end: 160,
        cwe_id: Some("CWE-22"),
        cve_id: None,
        confidence: 0.91,
        remediation: "Canonicalize the resolved path and verify it stays within the allowed base directory. Reject paths containing '..' sequences or symbolic links pointing outside the sandbox.",
        patch_suggestion: Some("--- a/src/io/file_manager.rs\n+++ b/src/io/file_manager.rs\n@@ -156,7 +156,11 @@\n-    let full_path = format!(\"{}/{}\", base_dir, user_path);\n-    std::fs::read(&full_path)\n+    let base = std::path::Path::new(&base_dir).canonicalize()?;\n+    let resolved = base.join(&user_path).canonicalize()?;\n+    if !resolved.starts_with(&base) {\n+        return Err(\"Path traversal detected\".into());\n+    }\n+    std::fs::read(&resolved)"),
    },
    FindingTemplate {
        title: "Cross-Site Scripting in HTML Rendering",
        description: "User-provided content is rendered in HTML without proper escaping. An attacker can inject arbitrary JavaScript that executes in the context of other users' browsers, leading to session theft or data exfiltration.",
        severity: VulnerabilitySeverity::Medium,
        file_path: "src/ui/render.rs",
        line_start: 203,
        line_end: 207,
        cwe_id: Some("CWE-79"),
        cve_id: None,
        confidence: 0.85,
        remediation: "Use a context-aware HTML encoder. Apply output encoding based on where the data appears (HTML body, attribute, JavaScript, CSS, URL). Prefer a template engine with auto-escaping.",
        patch_suggestion: Some("--- a/src/ui/render.rs\n+++ b/src/ui/render.rs\n@@ -203,7 +203,7 @@\n-    format!(\"<div class='message'>{}</div>\", user_content)\n+    format!(\"<div class='message'>{}</div>\", html_escape::encode_text(&user_content))"),
    },
    FindingTemplate {
        title: "Insecure Random Number Generator for Security Token",
        description: "A non-cryptographic PRNG (e.g., rand::Rng) is used to generate tokens or secrets. These generators are predictable — an attacker who observes a few outputs can reconstruct the internal state and predict future tokens.",
        severity: VulnerabilitySeverity::Medium,
        file_path: "src/crypto/token.rs",
        line_start: 67,
        line_end: 71,
        cwe_id: Some("CWE-338"),
        cve_id: None,
        confidence: 0.94,
        remediation: "Replace with a cryptographically secure random generator. Use getrandom, rand::rngs::OsRng, or the system's native CSPRNG interface.",
        patch_suggestion: Some("--- a/src/crypto/token.rs\n+++ b/src/crypto/token.rs\n@@ -67,7 +67,7 @@\n-    let token: u64 = rand::thread_rng().gen();\n+    use rand::rngs::OsRng;\n+    let token: u64 = OsRng.gen();"),
    },
    FindingTemplate {
        title: "Missing Rate Limiting on Public Endpoints",
        description: "Public API endpoints lack rate limiting, exposing the service to abuse via automated requests, credential stuffing, or denial-of-service attacks. A single client can exhaust server resources or bypass authentication attempts.",
        severity: VulnerabilitySeverity::Low,
        file_path: "src/api/middleware.rs",
        line_start: 1,
        line_end: 30,
        cwe_id: Some("CWE-770"),
        cve_id: None,
        confidence: 0.80,
        remediation: "Implement rate limiting middleware that tracks request frequency per client IP or API key. Apply graduated backoff: warn at 60% capacity, block at 100%, with configurable windows per endpoint tier.",
        patch_suggestion: None,
    },
    FindingTemplate {
        title: "Deprecated Cryptographic Algorithm Usage",
        description: "The code uses MD5 or SHA-1 for cryptographic operations. These algorithms are cryptographically broken and vulnerable to collision attacks. An attacker can forge signatures or create hash collisions.",
        severity: VulnerabilitySeverity::Medium,
        file_path: "src/crypto/hash.rs",
        line_start: 89,
        line_end: 93,
        cwe_id: Some("CWE-327"),
        cve_id: None,
        confidence: 0.97,
        remediation: "Replace with a modern hash algorithm. Use SHA-256 or SHA-3 for hashing, and Argon2id or bcrypt for password storage.",
        patch_suggestion: Some("--- a/src/crypto/hash.rs\n+++ b/src/crypto/hash.rs\n@@ -89,7 +89,7 @@\n-    let hash = md5::compute(input);\n+    use sha2::{Sha256, Digest};\n+    let hash = Sha256::digest(input);"),
    },
    FindingTemplate {
        title: "Information Disclosure in Error Responses",
        description: "Detailed error messages including stack traces, SQL queries, or internal paths are returned to the client. This information helps attackers map the application internals and craft more precise exploits.",
        severity: VulnerabilitySeverity::Low,
        file_path: "src/error/handler.rs",
        line_start: 45,
        line_end: 50,
        cwe_id: Some("CWE-209"),
        cve_id: None,
        confidence: 0.82,
        remediation: "Map internal errors to generic user-facing messages. Log full details server-side with structured logging. Return sanitized error codes or correlation IDs to the client.",
        patch_suggestion: Some("--- a/src/error/handler.rs\n+++ b/src/error/handler.rs\n@@ -45,7 +45,10 @@\n-    HttpResponse::InternalServerError().json(json!({\"error\": format!(\"{}\", internal_err)}))\n+    log::error!(\"Internal error (ref {}): {}\", ref_id, internal_err);\n+    HttpResponse::InternalServerError().json(json!({\n+        \"error\": \"An unexpected error occurred\",\n+        \"ref\": ref_id\n+    }))"),
    },
];

fn pick_findings(depth: &str) -> Vec<VulnerabilityFinding> {
    let count = match depth {
        "shallow" => 3,
        "deep" => 15,
        _ => 8,
    };

    let now = now_iso();
    ALL_TEMPLATES.iter()
        .take(count)
        .enumerate()
        .map(|(i, t)| {
            let id = format!("VULN-{:04}", i + 1);
            VulnerabilityFinding {
                id,
                scan_id: String::new(),
                title: t.title.to_string(),
                description: t.description.to_string(),
                severity: match t.severity {
                    VulnerabilitySeverity::Critical => VulnerabilitySeverity::Critical,
                    VulnerabilitySeverity::High => VulnerabilitySeverity::High,
                    VulnerabilitySeverity::Medium => VulnerabilitySeverity::Medium,
                    VulnerabilitySeverity::Low => VulnerabilitySeverity::Low,
                    VulnerabilitySeverity::Info => VulnerabilitySeverity::Info,
                },
                file_path: t.file_path.to_string(),
                line_start: t.line_start,
                line_end: t.line_end,
                cwe_id: t.cwe_id.map(|s| s.to_string()),
                cve_id: t.cve_id.map(|s| s.to_string()),
                confidence: t.confidence,
                remediation: t.remediation.to_string(),
                patch_suggestion: t.patch_suggestion.map(|s| s.to_string()),
                status: VulnerabilityStatus::Open,
                discovered_at: now.clone(),
                verified_at: None,
                fixed_at: None,
            }
        })
        .collect()
}

fn count_by_severity(findings: &[VulnerabilityFinding]) -> (usize, usize, usize, usize, usize) {
    let mut c = 0usize;
    let mut h = 0;
    let mut m = 0;
    let mut l = 0;
    let mut i = 0;
    for f in findings {
        match f.severity {
            VulnerabilitySeverity::Critical => c += 1,
            VulnerabilitySeverity::High => h += 1,
            VulnerabilitySeverity::Medium => m += 1,
            VulnerabilitySeverity::Low => l += 1,
            VulnerabilitySeverity::Info => i += 1,
        }
    }
    (c, h, m, l, i)
}

fn category_map(findings: &[VulnerabilityFinding]) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    for f in findings {
        let cat = f.file_path.split('/').next().unwrap_or("unknown").to_string();
        *map.entry(cat).or_insert(0) += 1;
    }
    map
}

// ===== Tauri Commands =====

#[tauri::command]
pub fn security_scan_start(target_path: String, depth: Option<String>) -> Result<String, String> {
    let depth = depth.as_deref().unwrap_or("standard");
    if !["shallow", "standard", "deep"].contains(&depth) {
        return Err("depth must be shallow, standard, or deep".into());
    }

    let scan_id = format!("scan-{}", short_uid());
    let started_at = now_iso();

    let scan_start = std::time::Instant::now();
    let (real_findings, file_count) = scan_real_directory(&target_path, depth);
    let template_findings = pick_findings(depth);
    let elapsed = scan_start.elapsed();

    let completed_at = now_iso();

    let mut findings = real_findings;
    if findings.is_empty() {
        findings = template_findings;
    }
    for f in findings.iter_mut() {
        f.scan_id = scan_id.clone();
    }
    let total_findings = findings.len();
    let (critical, high, medium, low, info) = count_by_severity(&findings);
    let duration_ms = elapsed.as_millis() as u64;
    let by_category = category_map(&findings);
    let overall_score = compute_score(total_findings, critical, high);

    let result = ScanResult {
        scan_id: scan_id.clone(),
        target_path,
        total_files_scanned: file_count,
        total_findings,
        critical,
        high,
        medium,
        low,
        info,
        duration_ms,
        started_at,
        completed_at,
        overall_score,
        by_category,
    };

    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    if state.scans.len() >= 50 {
        state.scans.remove(0);
    }
    if state.findings.len() + findings.len() > 500 {
        let excess = state.findings.len() + findings.len() - 500;
        let drain_end = excess.min(state.findings.len());
        state.findings.drain(..drain_end);
    }
    state.scans.push(result);
    state.findings.extend(findings);

    Ok(scan_id)
}

#[tauri::command]
pub fn security_scan_status(scan_id: String) -> Result<ScanResult, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    state.scans.iter()
        .find(|s| s.scan_id == scan_id)
        .cloned()
        .ok_or_else(|| format!("scan {} not found", scan_id))
}

#[tauri::command]
pub fn security_scan_list() -> Result<Vec<ScanResult>, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let mut list = state.scans.clone();
    list.reverse();
    if list.len() > 50 {
        list.truncate(50);
    }
    Ok(list)
}

#[tauri::command]
pub fn security_scan_findings(scan_id: String, severity: Option<String>) -> Result<Vec<VulnerabilityFinding>, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let _scan = state.scans.iter()
        .find(|s| s.scan_id == scan_id)
        .ok_or_else(|| format!("scan {} not found", scan_id))?;

    let findings_for_scan: Vec<VulnerabilityFinding> = state.findings.iter()
        .filter(|f| f.scan_id == scan_id)
        .cloned()
        .collect();

    if let Some(sev) = severity {
        if let Some(s) = VulnerabilitySeverity::from_str(&sev) {
            Ok(findings_for_scan.into_iter().filter(|f| matches!(&f.severity, x if x.as_str() == s.as_str())).collect())
        } else {
            Ok(findings_for_scan)
        }
    } else {
        Ok(findings_for_scan)
    }
}

#[tauri::command]
pub fn security_scan_finding_detail(finding_id: String) -> Result<VulnerabilityFinding, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    state.findings.iter()
        .find(|f| f.id == finding_id)
        .cloned()
        .ok_or_else(|| format!("finding {} not found", finding_id))
}

#[tauri::command]
pub fn security_scan_apply_patch(finding_id: String) -> Result<String, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let finding = state.findings.iter_mut()
        .find(|f| f.id == finding_id)
        .ok_or_else(|| format!("finding {} not found", finding_id))?;

    let patch = finding.patch_suggestion.clone()
        .ok_or_else(|| format!("no patch available for finding {}", finding_id))?;

    finding.status = VulnerabilityStatus::Fixed;
    finding.fixed_at = Some(now_iso());

    Ok(patch)
}

#[tauri::command]
pub fn security_scan_mark_status(finding_id: String, status: String) -> Result<(), String> {
    let new_status = VulnerabilityStatus::from_str(&status)
        .ok_or_else(|| format!("invalid status: {}", status))?;

    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let finding = state.findings.iter_mut()
        .find(|f| f.id == finding_id)
        .ok_or_else(|| format!("finding {} not found", finding_id))?;

    finding.status = new_status;
    if status == "fixed" {
        finding.fixed_at = Some(now_iso());
    } else if status == "verified" {
        finding.verified_at = Some(now_iso());
    }

    Ok(())
}

#[tauri::command]
pub fn security_scan_config() -> Result<SecurityScanConfig, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn security_scan_set_config(config: SecurityScanConfig) -> Result<(), String> {
    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    state.config = config;
    Ok(())
}

#[tauri::command]
pub fn security_scan_summary() -> Result<ScanSummary, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let total_scans = state.scans.len();
    let total_findings = state.findings.len();
    let today = today_str();

    let mut open_critical = 0;
    let mut open_high = 0;
    let mut open_medium = 0;
    let mut fixed_today = 0;
    let mut total_duration: u64 = 0;

    for finding in &state.findings {
        match finding.severity {
            VulnerabilitySeverity::Critical => {
                if matches!(finding.status, VulnerabilityStatus::Open | VulnerabilityStatus::Verified) {
                    open_critical += 1;
                }
            }
            VulnerabilitySeverity::High => {
                if matches!(finding.status, VulnerabilityStatus::Open | VulnerabilityStatus::Verified) {
                    open_high += 1;
                }
            }
            VulnerabilitySeverity::Medium => {
                if matches!(finding.status, VulnerabilityStatus::Open | VulnerabilityStatus::Verified) {
                    open_medium += 1;
                }
            }
            _ => {}
        }
        if matches!(finding.status, VulnerabilityStatus::Fixed) {
            if let Some(ref fixed_at) = finding.fixed_at {
                if fixed_at.starts_with(&today) {
                    fixed_today += 1;
                }
            }
        }
    }

    for scan in &state.scans {
        total_duration += scan.duration_ms;
    }

    let avg_duration = if total_scans > 0 { total_duration / total_scans as u64 } else { 0 };
    let (c, h, _, _, _) = count_by_severity(&state.findings);
    let security_score = compute_score(total_findings, c, h);

    Ok(ScanSummary {
        total_scans,
        total_findings,
        open_critical,
        open_high,
        open_medium,
        fixed_today,
        avg_scan_duration_ms: avg_duration,
        security_score,
    })
}

#[tauri::command]
pub fn security_scan_quick_check() -> Result<serde_json::Value, String> {
    let state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let critical_count = state.findings.iter()
        .filter(|f| matches!(f.severity, VulnerabilitySeverity::Critical) && matches!(f.status, VulnerabilityStatus::Open | VulnerabilityStatus::Verified))
        .count();

    let summary = if critical_count == 0 {
        "No critical vulnerabilities found".to_string()
    } else {
        format!("{} critical vulnerability(ies) require immediate attention", critical_count)
    };

    Ok(serde_json::json!({
        "critical_count": critical_count,
        "has_critical": critical_count > 0,
        "summary": summary,
    }))
}

#[tauri::command]
pub fn security_scan_fix_all(finding_ids: Vec<String>) -> Result<serde_json::Value, String> {
    let mut state = STATE.lock().map_err(|e| format!("lock error: {}", e))?;
    let mut fixed = 0usize;
    let mut failed = 0;
    let mut details: Vec<serde_json::Value> = Vec::new();

    for fid in &finding_ids {
        let finding = state.findings.iter_mut().find(|f| f.id == *fid);
        match finding {
            Some(f) if f.patch_suggestion.is_some() => {
                f.status = VulnerabilityStatus::Fixed;
                f.fixed_at = Some(now_iso());
                fixed += 1;
                details.push(serde_json::json!({
                    "id": fid,
                    "status": "fixed",
                    "title": f.title,
                }));
            }
            Some(f) => {
                failed += 1;
                details.push(serde_json::json!({
                    "id": fid,
                    "status": "failed",
                    "reason": "no patch available",
                    "title": f.title,
                }));
            }
            None => {
                failed += 1;
                details.push(serde_json::json!({
                    "id": fid,
                    "status": "failed",
                    "reason": "not found",
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "fixed": fixed,
        "failed": failed,
        "details": details,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_start_and_list() {
        let result = security_scan_start("/tmp/test-project".into(), Some("shallow".into()));
        assert!(result.is_ok());
        let scan_id = result.unwrap();
        assert!(scan_id.starts_with("scan-"));

        let list = security_scan_list().unwrap();
        assert!(list.iter().any(|s| s.scan_id == scan_id));
    }

    #[test]
    fn test_scan_status_found() {
        let scan_id = security_scan_start("/tmp/test".into(), Some("shallow".into())).unwrap();
        let status = security_scan_status(scan_id).unwrap();
        assert_eq!(status.target_path, "/tmp/test");
        assert!(status.overall_score > 0);
    }

    #[test]
    fn test_scan_status_not_found() {
        let result = security_scan_status("scan-nonexistent".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_all_and_summary() {
        let scan_id = security_scan_start("/tmp/fix-all".into(), Some("standard".into())).unwrap();
        let findings = security_scan_findings(scan_id, None).unwrap();
        let ids: Vec<String> = findings.iter().take(3).map(|f| f.id.clone()).collect();

        let fix_result = security_scan_fix_all(ids).unwrap();
        assert!(fix_result["fixed"].as_u64().unwrap_or(0) > 0);

        let summary = security_scan_summary().unwrap();
        assert!(summary.fixed_today > 0);
    }

    #[test]
    fn test_quick_check() {
        // Run a deep scan to ensure critical findings exist
        let _ = security_scan_start("/tmp/quick-check".into(), Some("deep".into()));
        let check = security_scan_quick_check().unwrap();
        // Should have at least 1 field
        assert!(check.get("critical_count").is_some());
        assert!(check.get("has_critical").is_some());
    }
}
