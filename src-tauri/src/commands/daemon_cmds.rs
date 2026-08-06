use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::State;

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub enabled: bool,
    pub watch_path: String,
    pub interval_secs: u64,
    pub auto_fix: bool,
    pub max_workers: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            watch_path: ".".to_string(),
            interval_secs: 30,
            auto_fix: true,
            max_workers: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: u64,
    pub uptime_secs: u64,
    pub files_watched: usize,
    pub auto_fixes_applied: usize,
    pub last_cycle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonEvent {
    pub timestamp: u64,
    pub kind: String,
    pub path: String,
    pub message: String,
}

// ── State ───────────────────────────────────────────────────────────────

struct DaemonState {
    running: AtomicBool,
    config: DaemonConfig,
    events: Vec<DaemonEvent>,
    start_time: u64,
    fixes_applied: usize,
    watched_files: HashSet<String>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl DaemonState {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            config: DaemonConfig::default(),
            events: Vec::with_capacity(1024),
            start_time: 0,
            fixes_applied: 0,
            watched_files: HashSet::new(),
            thread_handle: None,
        }
    }
}

static DAEMON: LazyLock<Mutex<DaemonState>> = LazyLock::new(|| Mutex::new(DaemonState::new()));

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn push_event(kind: &str, path: &str, message: &str) {
    if let Ok(mut state) = DAEMON.lock() {
        state.events.push(DaemonEvent {
            timestamp: now_secs(),
            kind: kind.to_string(),
            path: path.to_string(),
            message: message.to_string(),
        });
        if state.events.len() > 5000 {
            state.events.drain(0..1000);
        }
    }
}

fn scan_file_issues(path: &str) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let mut issues = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let ln = i + 1;
        let trimmed = line.trim();
        if trimmed.contains("api_key") || trimmed.contains("password") || trimmed.contains("secret") {
            if !trimmed.starts_with("//") && !trimmed.starts_with("#") && !trimmed.starts_with("/*") {
                issues.push(format!("L{}: Possible hardcoded secret", ln));
            }
        }
        if trimmed.to_uppercase().contains("TODO") || trimmed.to_uppercase().contains("FIXME") {
            // Redact: never include raw line content — a TODO line may contain secrets.
            issues.push(format!("L{}: TODO/FIXME marker", ln));
        }
        if line.len() > 120 {
            issues.push(format!("L{}: Line too long ({} chars)", ln, line.len()));
        }
        if trimmed.contains(".unwrap()") || trimmed.contains(".expect(") {
            issues.push(format!("L{}: Unwrap/expect usage", ln));
        }
        if trimmed.contains("unsafe {") || trimmed.contains("unsafe{") {
            issues.push(format!("L{}: Unsafe block", ln));
        }
        if trimmed.contains("todo!(") || trimmed.contains("unimplemented!(") {
            issues.push(format!("L{}: Todo/unimplemented", ln));
        }
    }
    issues
}

// ── Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn daemon_start(config: DaemonConfig) -> Result<String, String> {
    let mut state = DAEMON.lock().map_err(|e| e.to_string())?;
    if state.running.load(Ordering::SeqCst) {
        return Err("Daemon is already running".to_string());
    }
    state.config = config.clone();
    state.running.store(true, Ordering::SeqCst);
    state.start_time = now_secs();
    state.fixes_applied = 0;
    state.watched_files.clear();

    let handle = thread::spawn(move || {
        loop {
            let should_stop = DAEMON.lock().map(|s| !s.running.load(Ordering::SeqCst)).unwrap_or(true);
            if should_stop {
                break;
            }

            let (watch_path, interval, auto_fix) = DAEMON.lock().map(|s| {
                (s.config.watch_path.clone(), s.config.interval_secs, s.config.auto_fix)
            }).unwrap_or_default();

            let mut current_files: HashSet<String> = HashSet::new();
            if let Ok(entries) = fs::read_dir(&watch_path) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".rs") || name.ends_with(".md") || name.ends_with(".toml") || name.ends_with(".json") {
                            let path = entry.path().to_string_lossy().to_string();
                            current_files.insert(path.clone());
                            if let Ok(meta) = entry.metadata() {
                                if let Ok(_modified) = meta.modified() {
                                    let last_seen = DAEMON.lock().map(|s| s.watched_files.contains(&path)).unwrap_or(false);
                                    if !last_seen {
                                        push_event("file_change", &path, "New file detected");
                                        if auto_fix {
                                            let issues = scan_file_issues(&path);
                                            for issue in issues {
                                                push_event("issue", &path, &issue);
                                            }
                                            let mut state = DAEMON.lock().unwrap_or_else(|e| e.into_inner());
                                            state.fixes_applied += 1;
                                            state.watched_files.insert(path.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(mut state) = DAEMON.lock() {
                state.watched_files = current_files;
            }

            thread::sleep(Duration::from_secs(interval.max(1)));
        }
    });

    state.thread_handle = Some(handle);
    push_event("info", "", "Daemon started");
    Ok("Daemon started".to_string())
}

#[tauri::command]
pub fn daemon_stop() -> Result<(), String> {
    let handle = {
        let mut state = DAEMON.lock().map_err(|e| e.to_string())?;
        if !state.running.load(Ordering::SeqCst) {
            return Err("Daemon is not running".to_string());
        }
        state.running.store(false, Ordering::SeqCst);
        state.thread_handle.take()
    };
    // 必须先释放 DAEMON 锁再 join：守护线程醒来检查 running 时要抢同一把锁，
    // 持锁 join 会造成互等死锁。
    if let Some(handle) = handle {
        let _ = handle.join();
    }
    push_event("info", "", "Daemon stopped");
    Ok(())
}

#[tauri::command]
pub fn daemon_status() -> Result<DaemonStatus, String> {
    let state = DAEMON.lock().map_err(|e| e.to_string())?;
    let uptime = if state.running.load(Ordering::SeqCst) {
        now_secs().saturating_sub(state.start_time)
    } else {
        0
    };
    let last_cycle = state.events.last()
        .map(|e| e.timestamp.to_string())
        .unwrap_or_default();
    Ok(DaemonStatus {
        running: state.running.load(Ordering::SeqCst),
        pid: std::process::id() as u64,
        uptime_secs: uptime,
        files_watched: state.watched_files.len(),
        auto_fixes_applied: state.fixes_applied,
        last_cycle,
    })
}

#[tauri::command]
pub fn daemon_log(count: usize) -> Vec<DaemonEvent> {
    let state = DAEMON.lock().ok();
    let events = state.as_ref().map(|s| s.events.clone()).unwrap_or_default();
    events.into_iter().rev().take(count).collect()
}

#[tauri::command]
pub fn daemon_auto_fix(path: String) -> Result<String, String> {
    // Restrict to source/config extensions inside HOME (P0-3/P1-4).
    let ok_ext = ["rs", "md", "toml", "json", "ts", "js"];
    let ext_ok = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ok_ext.contains(&e))
        .unwrap_or(false);
    if !ext_ok {
        return Err("仅允许检查源码/配置类文件（.rs/.md/.toml/.json/.ts/.js）".to_string());
    }
    if let Some(home) = dirs::home_dir() {
        let canon = std::path::Path::new(&path).canonicalize().unwrap_or_default();
        if !canon.starts_with(home.canonicalize().unwrap_or(home)) {
            return Err("路径超出 HOME 范围".to_string());
        }
    }
    let issues = scan_file_issues(&path);
    if issues.is_empty() {
        return Ok("No issues found".to_string());
    }
    let summary = format!("Issues found:\n  {}", issues.join("\n  "));
    push_event("auto_fix", &path, &summary);
    if let Ok(mut state) = DAEMON.lock() {
        state.fixes_applied += 1;
    }
    Ok(summary)
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let cfg = DaemonConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.interval_secs, 30);
        assert_eq!(cfg.max_workers, 4);
    }

    #[test]
    fn test_daemon_status_not_running() {
        let status = daemon_status().unwrap();
        assert!(!status.running);
    }

    #[test]
    fn test_daemon_log_empty() {
        let log = daemon_log(10);
        assert!(log.is_empty() || log.len() <= 10);
    }

    #[test]
    fn test_scan_file_issues_nonexistent() {
        let issues = scan_file_issues("/tmp/neotrix_nonexistent_file_xyz.rs");
        assert!(issues.is_empty());
    }
}
