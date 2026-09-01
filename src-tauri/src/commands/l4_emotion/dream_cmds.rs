use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

// ── Types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub auto_run: bool,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 15,
            auto_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamStatus {
    pub phase: String,
    pub progress: f64,
    pub last_run: u64,
    pub total_consolidations: u64,
    pub memories_harvested: u64,
    pub contradictions_removed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamMemoryEntry {
    pub id: String,
    pub content: String,
    pub source: String,
    pub timestamp: u64,
    pub confidence: f64,
    pub tags: Vec<String>,
}

// ── State ───────────────────────────────────────────────────────────────

struct DreamState {
    running: AtomicBool,
    config: DreamConfig,
    entries: Vec<DreamMemoryEntry>,
    status: DreamStatus,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl DreamState {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            config: DreamConfig::default(),
            entries: Vec::with_capacity(512),
            status: DreamStatus {
                phase: "idle".to_string(),
                progress: 0.0,
                last_run: 0,
                total_consolidations: 0,
                memories_harvested: 0,
                contradictions_removed: 0,
            },
            thread_handle: None,
        }
    }
}

static DREAM: LazyLock<Mutex<DreamState>> = LazyLock::new(|| Mutex::new(DreamState::new()));

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn dream_source_dirs() -> Vec<String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let candidates = vec![
        format!("{}/.agents/skills", home),
        format!("{}/.agents/experience", home),
        "/tmp/neotrix_dream".to_string(),
    ];
    candidates.into_iter().filter(|d| Path::new(d).exists()).collect()
}

fn extract_tags_from_path(path: &str) -> Vec<String> {
    let p = path.to_lowercase();
    let mut tags = Vec::new();
    if p.contains("skill") || p.ends_with(".md") { tags.push("skill".to_string()); }
    if p.contains("experience") { tags.push("experience".to_string()); }
    if p.contains("review") || p.contains("audit") { tags.push("audit".to_string()); }
    if p.contains("agent") { tags.push("agent".to_string()); }
    if p.contains("memory") { tags.push("memory".to_string()); }
    if p.contains("config") || p.contains("json") { tags.push("config".to_string()); }
    if p.ends_with(".rs") { tags.push("rust".to_string()); }
    if p.ends_with(".toml") { tags.push("config".to_string()); }
    if tags.is_empty() { tags.push("general".to_string()); }
    tags
}

fn orient_phase() -> Vec<(String, String)> {
    let dirs = dream_source_dirs();
    let mut sources = Vec::new();
    for dir in &dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext == "rs" || ext == "md" || ext == "json" || ext == "toml" {
                        if let Some(p) = path.to_str() {
                            sources.push((p.to_string(), ext.to_string()));
                        }
                    }
                }
            }
        }
    }
    sources
}

fn gather_phase(sources: &[(String, String)]) -> Vec<DreamMemoryEntry> {
    let now = now_secs();
    let mut entries = Vec::new();
    let mut counter = 0u64;

    for (path, _ext) in sources {
        if let Ok(content) = fs::read_to_string(path) {
            let tags = extract_tags_from_path(path);
            let meaningful: Vec<&str> = content.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#") && !l.starts_with("/*"))
                .collect();

            for chunk in meaningful.chunks(5) {
                let text = chunk.join(" ");
                if text.len() < 20 || text.len() > 2000 {
                    continue;
                }
                counter += 1;
                entries.push(DreamMemoryEntry {
                    id: format!("mem-{}-{}", now, counter),
                    content: text,
                    source: path.clone(),
                    timestamp: now,
                    confidence: 0.5,
                    tags: tags.clone(),
                });
            }
        }
    }
    entries
}

fn consolidate_phase(entries: &mut Vec<DreamMemoryEntry>) -> (u64, u64) {
    let before = entries.len() as u64;

    // Merge entries with overlapping tags
    let mut merged: Vec<DreamMemoryEntry> = Vec::new();
    for entry in entries.drain(..) {
        let mut found = false;
        for existing in &mut merged {
            let common_tags: Vec<&String> = entry.tags.iter().filter(|t| existing.tags.contains(t)).collect();
            if !common_tags.is_empty() && entry.source == existing.source {
                existing.content.push_str(&format!("\n---\n{}", entry.content));
                existing.confidence = existing.confidence.max(entry.confidence);
                existing.tags.extend(entry.tags.clone());
                existing.tags.sort();
                existing.tags.dedup();
                found = true;
                break;
            }
        }
        if !found {
            merged.push(entry);
        }
    }
    *entries = merged;

    // Simple near-duplicate removal (identical content)
    let mut seen: HashMap<String, bool> = HashMap::new();
    entries.retain(|e| {
        if seen.contains_key(&e.content) {
            false
        } else {
            seen.insert(e.content.clone(), true);
            true
        }
    });

    let removed = before.saturating_sub(entries.len() as u64);
    (entries.len() as u64, removed)
}

fn prune_phase(entries: &mut Vec<DreamMemoryEntry>) -> u64 {
    let before = entries.len();
    entries.retain(|e| e.confidence >= 0.2);
    entries.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    if entries.len() > 500 {
        entries.truncate(500);
    }
    before.saturating_sub(entries.len()) as u64
}

fn run_dream_cycle() -> String {
    if let Ok(mut state) = DREAM.lock() {
        state.status.progress = 0.0;
        state.status.phase = "orient".to_string();
    }
    let sources = orient_phase();

    if let Ok(mut state) = DREAM.lock() {
        state.status.progress = 0.25;
        state.status.phase = "gather".to_string();
    }
    let gathered: Vec<DreamMemoryEntry> = gather_phase(&sources);
    let harvested = gathered.len() as u64;

    if let Ok(mut state) = DREAM.lock() {
        state.status.progress = 0.5;
        state.status.phase = "consolidate".to_string();
        state.entries.extend(gathered);
    }

    if let Ok(mut state) = DREAM.lock() {
        let (_total, removed) = consolidate_phase(&mut state.entries);
        state.status.progress = 0.75;
        state.status.phase = "prune".to_string();
        state.status.contradictions_removed += removed;
        state.status.memories_harvested += harvested;
    }

    if let Ok(mut state) = DREAM.lock() {
        let pruned = prune_phase(&mut state.entries);
        state.status.contradictions_removed += pruned;
        state.status.total_consolidations += 1;
        state.status.last_run = now_secs();
        state.status.progress = 1.0;
        state.status.phase = "idle".to_string();
    }

    format!(
        "Dream cycle complete: {} sources, {} harvested, {} entries after consolidation",
        sources.len(),
        harvested,
        DREAM.lock().map(|s| s.entries.len()).unwrap_or(0)
    )
}

// ── Commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn dream_start(config: DreamConfig) -> Result<String, String> {
    let mut state = DREAM.lock().map_err(|e| e.to_string())?;
    if state.running.load(Ordering::SeqCst) {
        return Err("Dream cycle is already running".to_string());
    }
    state.config = config.clone();
    state.running.store(true, Ordering::SeqCst);

    let interval_secs = config.interval_minutes.max(1) * 60;
    let handle = thread::spawn(move || {
        loop {
            let should_stop = DREAM.lock().map(|s| !s.running.load(Ordering::SeqCst)).unwrap_or(true);
            if should_stop {
                break;
            }
            run_dream_cycle();
            thread::sleep(Duration::from_secs(interval_secs));
        }
    });

    state.thread_handle = Some(handle);
    Ok("Dream cycle started".to_string())
}

#[tauri::command]
pub fn dream_stop() -> Result<(), String> {
    let handle = {
        let mut state = DREAM.lock().map_err(|e| e.to_string())?;
        if !state.running.load(Ordering::SeqCst) {
            return Err("Dream cycle is not running".to_string());
        }
        state.running.store(false, Ordering::SeqCst);
        state.thread_handle.take()
    };
    // 必须先释放 DREAM 锁再 join：守护线程醒来检查 running 时要抢同一把锁。
    if let Some(handle) = handle {
        let _ = handle.join();
    }
    Ok(())
}

#[tauri::command]
pub fn dream_status() -> Result<DreamStatus, String> {
    let state = DREAM.lock().map_err(|e| e.to_string())?;
    Ok(state.status.clone())
}

#[tauri::command]
pub fn dream_entries() -> Vec<DreamMemoryEntry> {
    DREAM.lock().map(|s| s.entries.clone()).unwrap_or_default()
}

#[tauri::command]
pub fn dream_consolidate_now() -> Result<String, String> {
    Ok(run_dream_cycle())
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dream_config_default() {
        let cfg = DreamConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.interval_minutes, 15);
        assert!(!cfg.auto_run);
    }

    #[test]
    fn test_dream_status_idle() {
        let status = dream_status().unwrap();
        assert_eq!(status.phase, "idle");
    }

    #[test]
    fn test_dream_entries_empty() {
        let entries = dream_entries();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_extract_tags_from_path() {
        let tags = extract_tags_from_path("/home/user/.agents/skills/audit_agent.rs");
        assert!(tags.contains(&"skill".to_string()));
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"audit".to_string()));
    }
}
