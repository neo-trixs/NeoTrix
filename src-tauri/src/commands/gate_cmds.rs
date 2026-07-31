use std::fs;
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    pub auto_check: bool,
    pub strict_mode: bool,
    pub checks_enabled: Vec<String>,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            strict_mode: false,
            checks_enabled: vec![
                "UnresolvedFIXME".into(),
                "HardcodedSecrets".into(),
                "DebugArtifacts".into(),
                "UnwrapUsage".into(),
                "UnsafeCode".into(),
                "FileSize".into(),
                "VulnerabilityScan".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    pub name: String,
    pub status: String,
    pub message: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    pub overall: String,
    pub checks: Vec<GateCheck>,
    pub score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatePolicy {
    pub block_on_fail: bool,
    pub require_review: bool,
    pub auto_fix_trivial: bool,
}

impl Default for GatePolicy {
    fn default() -> Self {
        Self {
            block_on_fail: true,
            require_review: false,
            auto_fix_trivial: true,
        }
    }
}

// ── State ───────────────────────────────────────────────────────────────

struct GateState {
    config: GateConfig,
    policy: GatePolicy,
    audit_log: Vec<serde_json::Value>,
}

impl GateState {
    fn new() -> Self {
        Self {
            config: GateConfig::default(),
            policy: GatePolicy::default(),
            audit_log: Vec::new(),
        }
    }
}

static GATE: LazyLock<Mutex<GateState>> = LazyLock::new(|| Mutex::new(GateState::new()));

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn push_audit(kind: &str, detail: &str) {
    if let Ok(mut state) = GATE.lock() {
        state.audit_log.push(json!({
            "timestamp": now_secs(),
            "kind": kind,
            "detail": detail,
        }));
        if state.audit_log.len() > 100 {
            state.audit_log.remove(0);
        }
    }
}

fn collect_rs_files(path: &str) -> Result<Vec<String>, String> {
    let meta = fs::metadata(path).map_err(|e| format!("Cannot access {}: {}", path, e))?;
    if meta.is_file() {
        return Ok(vec![path.to_string()]);
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(path).map_err(|e| format!("Cannot read directory {}: {}", path, e))?;
    for entry in entries.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        if p.is_dir() && (name_str == "target" || name_str == "node_modules" || name_str == ".git" || name_str == ".vscode") {
            continue;
        }
        if p.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(p.to_string_lossy().to_string());
        } else if p.is_dir() {
            if let Ok(sub) = collect_rs_files(&p.to_string_lossy().to_string()) {
                files.extend(sub);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn run_fixme_check(files: &[String]) -> GateCheck {
    let mut details = Vec::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            for (i, line) in content.lines().enumerate() {
                let t = line.trim().to_uppercase();
                if t.contains("TODO") || t.contains("FIXME") || t.contains("HACK") || t.contains("XXX") {
                    details.push(format!("{}:L{}: {}", file, i + 1, line.trim()));
                }
            }
        }
    }
    let status = if details.is_empty() { "pass" } else { "fail" };
    GateCheck {
        name: "UnresolvedFIXME".into(),
        status: status.into(),
        message: if details.is_empty() { "No unresolved markers found".into() } else { format!("Found {} unresolved marker(s)", details.len()) },
        details: details.join("\n"),
    }
}

fn run_secrets_check(files: &[String]) -> GateCheck {
    let patterns = ["password=", "password:", "api_key=", "api_key:", "secret=", "secret:", "token=", "token:"];
    let mut details = Vec::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            for (i, line) in content.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("//") || t.starts_with('#') || t.starts_with("/*") { continue; }
                let lower = t.to_lowercase();
                for pat in &patterns {
                    if lower.contains(pat) {
                        let preview: String = t.chars().take(60).collect();
                        details.push(format!("{}:L{}: {}", file, i + 1, preview));
                        break;
                    }
                }
            }
        }
    }
    let status = if details.is_empty() { "pass" } else { "fail" };
    GateCheck {
        name: "HardcodedSecrets".into(),
        status: status.into(),
        message: if details.is_empty() { "No hardcoded secrets detected".into() } else { format!("Found {} potential secret(s)", details.len()) },
        details: details.join("\n"),
    }
}

fn run_debug_check(files: &[String]) -> GateCheck {
    let mut details = Vec::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            for (i, line) in content.lines().enumerate() {
                let t = line.trim();
                if t.contains("dbg!(") || t.contains("println!(") || t.contains("console.log") || t.starts_with("print(") {
                    details.push(format!("{}:L{}: {}", file, i + 1, t));
                }
            }
        }
    }
    let status = if details.is_empty() { "pass" } else { "warn" };
    GateCheck {
        name: "DebugArtifacts".into(),
        status: status.into(),
        message: if details.is_empty() { "No debug artifacts found".into() } else { format!("Found {} debug artifact(s)", details.len()) },
        details: details.join("\n"),
    }
}

fn run_unwrap_check(files: &[String]) -> GateCheck {
    let mut details = Vec::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            for (i, line) in content.lines().enumerate() {
                let t = line.trim();
                if t.contains(".unwrap()") || t.contains(".expect(") {
                    details.push(format!("{}:L{}: {}", file, i + 1, t));
                }
            }
        }
    }
    let status = if details.is_empty() { "pass" } else { "warn" };
    GateCheck {
        name: "UnwrapUsage".into(),
        status: status.into(),
        message: if details.is_empty() { "No unwrap/expect usage found".into() } else { format!("Found {} unwrap/expect usage(s)", details.len()) },
        details: details.join("\n"),
    }
}

fn run_unsafe_check(files: &[String]) -> GateCheck {
    let mut details = Vec::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            for (i, line) in content.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("unsafe") || t.contains("unsafe {") || t.contains("unsafe{") {
                    details.push(format!("{}:L{}: {}", file, i + 1, t));
                }
            }
        }
    }
    let status = if details.is_empty() { "pass" } else { "fail" };
    GateCheck {
        name: "UnsafeCode".into(),
        status: status.into(),
        message: if details.is_empty() { "No unsafe code found".into() } else { format!("Found {} unsafe block(s)", details.len()) },
        details: details.join("\n"),
    }
}

fn run_filesize_check(files: &[String]) -> GateCheck {
    let mut details = Vec::new();
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            let count = content.lines().count();
            if count > 500 {
                details.push(format!("{}: {} lines (limit: 500)", file, count));
            }
        }
    }
    let status = if details.is_empty() { "pass" } else { "warn" };
    GateCheck {
        name: "FileSize".into(),
        status: status.into(),
        message: if details.is_empty() { "All files within size limit".into() } else { format!("{} file(s) exceed 500 lines", details.len()) },
        details: details.join("\n"),
    }
}

fn run_vuln_scan_check(path: &str) -> GateCheck {
    let (findings, _file_count) = super::security_scan_cmds::scan_real_directory(path, "standard");
    if findings.is_empty() {
        return GateCheck {
            name: "VulnerabilityScan".into(),
            status: "pass".into(),
            message: "No security vulnerabilities detected".into(),
            details: "Pattern scan clean across project files".into(),
        };
    }
    let critical = findings.iter().filter(|f| matches!(f.severity, super::security_scan_cmds::VulnerabilitySeverity::Critical)).count();
    let high = findings.iter().filter(|f| matches!(f.severity, super::security_scan_cmds::VulnerabilitySeverity::High)).count();
    let medium = findings.iter().filter(|f| matches!(f.severity, super::security_scan_cmds::VulnerabilitySeverity::Medium)).count();
    let details: Vec<String> = findings.iter().take(20)
        .map(|f| format!("{}:{}: [{}] {}",
            f.file_path.rsplit('/').next().unwrap_or(&f.file_path),
            f.line_start,
            f.severity.as_str().to_uppercase(),
            f.title))
        .collect();
    let status = if critical + high > 0 { "fail" } else { "warn" };
    GateCheck {
        name: "VulnerabilityScan".into(),
        status: status.into(),
        message: format!("Found {} critical, {} high, {} medium vulnerability pattern(s)", critical, high, medium),
        details: details.join("\n"),
    }
}

// ── Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn gate_run_check(path: String) -> Result<GateResult, String> {
    let files = collect_rs_files(&path)?;

    let config = GATE.lock().map(|s| s.config.clone()).unwrap_or_default();
    let enabled: Vec<String> = config.checks_enabled.iter().map(|c| c.to_lowercase()).collect();
    let mut checks: Vec<GateCheck> = Vec::new();

    if enabled.contains(&"unresolvedfixme".to_string()) {
        checks.push(run_fixme_check(&files));
    }
    if enabled.contains(&"hardcodedsecrets".to_string()) {
        checks.push(run_secrets_check(&files));
    }
    if enabled.contains(&"debugartifacts".to_string()) {
        checks.push(run_debug_check(&files));
    }
    if enabled.contains(&"unwrapusage".to_string()) {
        checks.push(run_unwrap_check(&files));
    }
    if enabled.contains(&"unsafecode".to_string()) {
        checks.push(run_unsafe_check(&files));
    }
    if enabled.contains(&"filesize".to_string()) {
        checks.push(run_filesize_check(&files));
    }
    if enabled.contains(&"vulnerabilityscan".to_string()) {
        checks.push(run_vuln_scan_check(&path));
    }

    let failures = checks.iter().filter(|c| c.status == "fail").count();
    let warnings = checks.iter().filter(|c| c.status == "warn").count();
    let score = (100u16).saturating_sub((failures * 20 + warnings * 5) as u16) as u8;
    let overall = if failures > 0 { "fail" } else if warnings > 0 { "warn" } else { "pass" };

    let result = GateResult { overall: overall.into(), checks, score };
    push_audit("gate_run_check", &format!("path={} score={} failures={} warnings={}", path, result.score, failures, warnings));
    Ok(result)
}

#[tauri::command]
pub fn gate_set_config(config: GateConfig) -> Result<(), String> {
    let mut state = GATE.lock().map_err(|e| e.to_string())?;
    state.config = config;
    push_audit("gate_set_config", "Configuration updated");
    Ok(())
}

#[tauri::command]
pub fn gate_get_config() -> Result<GateConfig, String> {
    let state = GATE.lock().map_err(|e| e.to_string())?;
    Ok(state.config.clone())
}

#[tauri::command]
pub fn gate_set_policy(policy: GatePolicy) -> Result<(), String> {
    let mut state = GATE.lock().map_err(|e| e.to_string())?;
    state.policy = policy;
    push_audit("gate_set_policy", "Policy updated");
    Ok(())
}

#[tauri::command]
pub fn gate_get_policy() -> Result<GatePolicy, String> {
    let state = GATE.lock().map_err(|e| e.to_string())?;
    Ok(state.policy.clone())
}

#[tauri::command]
pub fn gate_approve(override_reason: String) -> Result<String, String> {
    push_audit("gate_approve", &override_reason);
    Ok(format!("Gate overridden: {}", override_reason))
}

#[tauri::command]
pub fn gate_audit_log(count: usize) -> Vec<serde_json::Value> {
    let state = GATE.lock().ok();
    let log = state.as_ref().map(|s| s.audit_log.clone()).unwrap_or_default();
    log.into_iter().rev().take(count).collect()
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_fixme_check_detects_markers() {
        let file = std::env::temp_dir().join("neotrix_gate_fixme_test.rs");
        fs::write(&file, "// FIXME: broken\nfn main() {}\n// TODO: refactor\n").unwrap();
        let result = run_fixme_check(&[file.to_string_lossy().to_string()]);
        assert_eq!(result.status, "fail");
        assert!(result.details.contains("FIXME"));
        let _ = fs::remove_file(&file);
    }

    #[test]
    fn test_run_secrets_check_detects_password() {
        let file = std::env::temp_dir().join("neotrix_gate_secret_test.rs");
        fs::write(&file, "password=\"secret\"\napi_key=sk-xxx\nfn main() {}\n").unwrap();
        let result = run_secrets_check(&[file.to_string_lossy().to_string()]);
        assert_eq!(result.status, "fail");
        assert!(result.details.contains("api_key") || result.details.contains("password"));
        let _ = fs::remove_file(&file);
    }

    #[test]
    fn test_run_unsafe_check_detects_unsafe() {
        let file = std::env::temp_dir().join("neotrix_gate_unsafe_test.rs");
        fs::write(&file, "fn main() {\n    unsafe { let x = &42 as *const i32; }\n}\n").unwrap();
        let result = run_unsafe_check(&[file.to_string_lossy().to_string()]);
        assert_eq!(result.status, "fail");
        assert!(result.details.contains("unsafe"));
        let _ = fs::remove_file(&file);
    }

    #[test]
    fn test_run_debug_check_detects_dbg() {
        let file = std::env::temp_dir().join("neotrix_gate_debug_test.rs");
        fs::write(&file, "fn main() {\n    dbg!(42);\n    println!(\"x: {}\", x);\n}\n").unwrap();
        let result = run_debug_check(&[file.to_string_lossy().to_string()]);
        assert_eq!(result.status, "warn");
        let _ = fs::remove_file(&file);
    }

    #[test]
    fn test_gate_config_default() {
        let config = GateConfig::default();
        assert!(config.auto_check);
        assert!(!config.strict_mode);
        assert_eq!(config.checks_enabled.len(), 7);
    }

    #[test]
    fn test_gate_result_scoring() {
        let result = GateResult {
            overall: "pass".into(),
            checks: vec![],
            score: 100,
        };
        assert_eq!(result.score, 100);
        assert_eq!(result.overall, "pass");
    }

    #[test]
    fn test_gate_check_struct() {
        let check = GateCheck {
            name: "TestCheck".into(),
            status: "fail".into(),
            message: "Test message".into(),
            details: "line 5: error".into(),
        };
        assert_eq!(check.status, "fail");
        assert!(check.details.contains("error"));
    }
}
