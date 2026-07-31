use std::fs;
use std::sync::{LazyLock, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProfile {
    pub name: String,
    pub email: String,
    pub use_for_commits: bool,
    pub strip_co_auth: bool,
    pub custom_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndercoverConfig {
    pub enabled: bool,
    pub default_profile: String,
    pub profiles: Vec<IdentityProfile>,
    pub strip_traces: bool,
}

impl Default for UndercoverConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_profile: "default".into(),
            profiles: vec![],
            strip_traces: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTrace {
    pub commit_hash: String,
    pub repo: String,
    pub original_author: String,
    pub committed_as: String,
    pub timestamp: u64,
    pub trace_hash: String,
}

// ── State ───────────────────────────────────────────────────────────────

struct UndercoverState {
    config: UndercoverConfig,
    commit_log: Vec<CommitTrace>,
    active_profile: Option<String>,
}

impl UndercoverState {
    fn new() -> Self {
        Self {
            config: UndercoverConfig::default(),
            commit_log: Vec::with_capacity(64),
            active_profile: None,
        }
    }
}

static UNDERCOVER: LazyLock<Mutex<UndercoverState>> = LazyLock::new(|| Mutex::new(UndercoverState::new()));

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn simple_hash(s: &str) -> String {
    let mut h = 0u64;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u64);
    }
    format!("{:x}", h)
}

fn push_commit_log(commit_hash: &str, repo: &str, original_author: &str, committed_as: &str) {
    if let Ok(mut state) = UNDERCOVER.lock() {
        state.commit_log.push(CommitTrace {
            commit_hash: commit_hash.to_string(),
            repo: repo.to_string(),
            original_author: original_author.to_string(),
            committed_as: committed_as.to_string(),
            timestamp: now_secs(),
            trace_hash: simple_hash(&format!("{}:{}:{}", commit_hash, original_author, committed_as)),
        });
        if state.commit_log.len() > 50 {
            state.commit_log.remove(0);
        }
    }
}

// ── Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn undercover_status() -> Result<serde_json::Value, String> {
    let state = UNDERCOVER.lock().map_err(|e| e.to_string())?;
    Ok(json!({
        "enabled": state.config.enabled,
        "active_profile": state.active_profile,
        "total_profiles": state.config.profiles.len(),
        "commits_processed": state.commit_log.len(),
        "strip_traces": state.config.strip_traces,
    }))
}

#[tauri::command]
pub fn undercover_set_profile(name: String, email: String, strip_co_auth: bool) -> Result<(), String> {
    let mut state = UNDERCOVER.lock().map_err(|e| e.to_string())?;
    let custom_prefix = format!("{} <{}>", &name, &email);

    // Remove existing profile with same name
    state.config.profiles.retain(|p| p.name != name);

    state.config.profiles.push(IdentityProfile {
        name: name.clone(),
        email,
        use_for_commits: true,
        strip_co_auth,
        custom_prefix,
    });

    if state.active_profile.is_none() {
        state.active_profile = Some(name);
    }

    Ok(())
}

#[tauri::command]
pub fn undercover_get_profiles() -> Result<Vec<IdentityProfile>, String> {
    let state = UNDERCOVER.lock().map_err(|e| e.to_string())?;
    Ok(state.config.profiles.clone())
}

#[tauri::command]
pub fn undercover_activate_profile(name: String) -> Result<(), String> {
    let mut state = UNDERCOVER.lock().map_err(|e| e.to_string())?;
    let exists = state.config.profiles.iter().any(|p| p.name == name);
    if !exists {
        return Err(format!("Profile '{}' not found", name));
    }
    state.active_profile = Some(name.clone());
    state.config.enabled = true;
    push_commit_log("pending", "runtime", "unknown", &name);
    Ok(())
}

#[tauri::command]
pub fn undercover_strip_metadata(file_path: String) -> Result<String, String> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Cannot read {}: {}", file_path, e))?;

    let patterns = [
        "Co-Authored-By:",
        "Co-authored-by:",
        "Co-authored-By:",
        "co-authored-by:",
        "Signed-off-by:",
        "Signed-Off-By:",
    ];

    let mut removed = 0u32;
    let filtered: Vec<&str> = content.lines()
        .filter(|line| {
            let trimmed = line.trim();
            let is_metadata = patterns.iter().any(|p| trimmed.starts_with(p));
            if is_metadata { removed += 1; }
            !is_metadata
        })
        .collect();

    let new_content = filtered.join("\n");

    // Preserve trailing newline if original had one
    let new_content = if content.ends_with('\n') {
        format!("{}\n", new_content)
    } else {
        new_content
    };

    fs::write(&file_path, &new_content)
        .map_err(|e| format!("Cannot write {}: {}", file_path, e))?;

    Ok(format!("Removed {} metadata line(s) from {}", removed, file_path))
}

#[tauri::command]
pub fn undercover_commit_log(count: usize) -> Vec<CommitTrace> {
    let state = UNDERCOVER.lock().ok();
    let log = state.as_ref().map(|s| s.commit_log.clone()).unwrap_or_default();
    log.into_iter().rev().take(count).collect()
}

#[tauri::command]
pub fn undercover_verify_anonymity(path: String) -> Result<serde_json::Value, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read {}: {}", path, e))?;

    let mut findings: Vec<serde_json::Value> = Vec::new();

    // Check for email patterns
    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        if t.contains('@') && (t.contains(".com") || t.contains(".org") || t.contains(".net") || t.contains(".edu")) {
            let preview: String = t.chars().take(80).collect();
            findings.push(json!({
                "line": i + 1,
                "type": "email",
                "preview": preview,
            }));
        }
    }

    // Check for API keys / tokens
    let key_patterns = ["sk-", "pk-", "api-", "ghp_", "gho_", "xoxb-", "xoxp-"];
    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        let lower = t.to_lowercase();
        for pat in &key_patterns {
            if lower.contains(pat) {
                let preview: String = t.chars().take(60).collect();
                findings.push(json!({
                    "line": i + 1,
                    "type": "api_key",
                    "preview": preview,
                }));
                break;
            }
        }
    }

    // Check for internal URLs
    let url_patterns = ["localhost", "127.0.0.1", "internal", "10.0.", "192.168."];
    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        let lower = t.to_lowercase();
        for pat in &url_patterns {
            if lower.contains(pat) {
                let preview: String = t.chars().take(60).collect();
                findings.push(json!({
                    "line": i + 1,
                    "type": "internal_url",
                    "preview": preview,
                }));
                break;
            }
        }
    }

    let total_lines = content.lines().count();
    let risk = if findings.is_empty() {
        "low"
    } else if findings.len() > 5 {
        "high"
    } else {
        "medium"
    };

    Ok(json!({
        "file": path,
        "total_lines": total_lines,
        "findings_count": findings.len(),
        "risk_level": risk,
        "findings": findings,
    }))
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_file(content: &str) -> (std::path::PathBuf, String) {
        let dir = std::env::temp_dir().join("neotrix_undercover_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(format!("test_anonymity_{}.txt", now_secs()));
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (path.clone(), path.to_string_lossy().to_string())
    }

    #[test]
    fn test_set_and_get_profile() {
        undercover_set_profile("TestDev".into(), "test@example.com".into(), true).unwrap();
        let profiles = undercover_get_profiles().unwrap();
        let found = profiles.iter().find(|p| p.name == "TestDev");
        assert!(found.is_some(), "Profile should exist");
        assert_eq!(found.unwrap().email, "test@example.com");
    }

    #[test]
    fn test_strip_metadata_removes_co_authored() {
        let content = "First line\nCo-Authored-By: Someone <someone@example.com>\nThird line\n";
        let (_path, path_str) = write_temp_file(content);
        let result = undercover_strip_metadata(path_str.clone()).unwrap();
        assert!(result.contains("Removed 1"), "Should report 1 line removed");

        let remaining = fs::read_to_string(&path_str).unwrap();
        assert!(!remaining.contains("Co-Authored-By"));
        assert!(remaining.contains("First line"));
        assert!(remaining.contains("Third line"));
    }

    #[test]
    fn test_verify_finds_email() {
        let content = "fn main() {\n    let email = \"user@example.com\";\n    println!(\"done\");\n}\n";
        let (_path, path_str) = write_temp_file(content);
        let result = undercover_verify_anonymity(path_str).unwrap();
        let findings = result["findings"].as_array().unwrap();
        let emails: Vec<&serde_json::Value> = findings.iter().filter(|f| f["type"] == "email").collect();
        assert!(!emails.is_empty(), "Should detect email pattern");
    }

    #[test]
    fn test_commit_log_bounded() {
        for i in 0..60 {
            push_commit_log(&format!("hash-{}", i), "test-repo", "author", "alias");
        }
        let state = UNDERCOVER.lock().unwrap();
        assert!(state.commit_log.len() <= 50, "Commit log should be bounded to 50, got {}", state.commit_log.len());
    }
}
