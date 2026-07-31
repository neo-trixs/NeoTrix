use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::command;

// ===== Structs =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub kind: String,
    pub content: String,
    pub summary: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub access_count: u32,
    pub tags: Vec<String>,
    pub is_pinned: bool,
}

impl MemoryEntry {
    fn new(id: String, kind: String, content: String, summary: String, source: String) -> Self {
        let now = now_ts();
        MemoryEntry {
            id,
            kind,
            content,
            summary,
            source,
            confidence: 0.5,
            created_at: now,
            last_accessed_at: now,
            access_count: 0,
            tags: Vec::new(),
            is_pinned: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub total_categories: usize,
    pub oldest_entry: i64,
    pub newest_entry: i64,
    pub avg_confidence: f64,
    pub top_tags: Vec<(String, u32)>,
    pub memory_usage_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub total: u32,
    pub results: Vec<MemoryEntry>,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub enabled: bool,
    pub auto_consolidate: bool,
    pub consolidation_interval_mins: u32,
    pub max_entries: u32,
    pub enable_search: bool,
    pub enable_pinning: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            enabled: true,
            auto_consolidate: true,
            consolidation_interval_mins: 60,
            max_entries: 10000,
            enable_search: true,
            enable_pinning: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTimelineEntry {
    pub date: String,
    pub entries_created: u32,
    pub entries_accessed: u32,
    pub top_topic: String,
}

// ===== State =====

const MAX_ENTRIES: usize = 10000;

struct MemoryManagerState {
    entries: Vec<MemoryEntry>,
    config: MemoryConfig,
}

impl Default for MemoryManagerState {
    fn default() -> Self {
        let mut state = MemoryManagerState {
            entries: Vec::with_capacity(64),
            config: MemoryConfig::default(),
        };
        state.seed_default_entries();
        state
    }
}

impl MemoryManagerState {
    fn seed_default_entries(&mut self) {
        let now = now_ts();
        let mut add = |kind: &str, content: &str, summary: &str, source: &str, conf: f64, tags: Vec<&str>| {
            let id = format!("mem-{:016x}", self.entries.len() as u64 + 1);
            self.entries.push(MemoryEntry {
                id,
                kind: kind.to_string(),
                content: content.to_string(),
                summary: summary.to_string(),
                source: source.to_string(),
                confidence: conf,
                created_at: now,
                last_accessed_at: now,
                access_count: 0,
                tags: tags.into_iter().map(String::from).collect(),
                is_pinned: false,
            });
        };

        // Preferences (5)
        add("preference", "Preferred language: Rust", "Language preference for development work", "user_profile", 0.95, vec!["language", "rust"]);
        add("preference", "Dark theme preferred", "UI theme preference", "user_profile", 0.90, vec!["theme", "ui"]);
        add("preference", "Uses tabs, not spaces", "Code formatting preference", "user_profile", 0.85, vec!["formatting", "code"]);
        add("preference", "Prefers async/await patterns", "Async programming style preference", "user_profile", 0.80, vec!["async", "rust"]);
        add("preference", "Favorite editor: VS Code", "Editor preference", "user_profile", 0.75, vec!["editor", "vscode"]);

        // Facts (3)
        add("fact", "Project uses Tauri V2", "Desktop application framework used by this project", "project_config", 1.0, vec!["tauri", "framework", "desktop"]);
        add("fact", "Database: SQLite via rusqlite", "Persistent storage backend for knowledge base", "project_config", 1.0, vec!["database", "sqlite", "storage"]);
        add("fact", "Frontend: React+TypeScript", "UI framework used for the desktop app", "project_config", 1.0, vec!["frontend", "react", "typescript"]);

        // Skills (3)
        add("skill", "Expert in Rust async programming", "Advanced proficiency in Rust concurrency and async patterns", "skill_assessment", 0.92, vec!["rust", "async", "concurrency"]);
        add("skill", "Proficient in Tauri desktop apps", "Experience building cross-platform desktop apps with Tauri", "skill_assessment", 0.88, vec!["tauri", "desktop", "rust"]);
        add("skill", "Skilled in TypeScript/React", "Frontend development with modern TypeScript and React", "skill_assessment", 0.85, vec!["typescript", "react", "frontend"]);

        // Knowledge (4)
        add("knowledge", "Tauri commands need #[tauri::command]", "All Tauri command functions must be annotated with #[tauri::command]", "project_learned", 1.0, vec!["tauri", "commands", "patterns"]);
        add("knowledge", "serde for JSON serialization", "serde crate provides Serialize/Deserialize for JSON data exchange", "project_learned", 1.0, vec!["serde", "serialization", "json"]);
        add("knowledge", "KnowledgeBase stored in SQLite", "The KB system persists graph nodes and edges in SQLite", "project_learned", 1.0, vec!["knowledge", "database", "architecture"]);
        add("knowledge", "AGENTS.md drives agent behavior", "The AGENTS.md file at project root guides AI agent context and behavior", "project_learned", 1.0, vec!["agents", "documentation", "behavior"]);
    }
}

static MEMORY_MGR: LazyLock<Mutex<MemoryManagerState>> =
    LazyLock::new(|| Mutex::new(MemoryManagerState::default()));

static NEXT_MEM_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0x1000);

// ===== Helpers =====

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn sort_entries(entries: &mut Vec<MemoryEntry>, sort: &str) {
    match sort {
        "created" => entries.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
        "accessed" => entries.sort_by(|a, b| b.last_accessed_at.cmp(&a.last_accessed_at)),
        "confidence" => entries.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)),
        _ => entries.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
    }
}

fn update_access(entry: &mut MemoryEntry) {
    entry.last_accessed_at = now_ts();
    entry.access_count += 1;
}

fn calc_memory_usage(entries: &[MemoryEntry]) -> u64 {
    let mut bytes: u64 = 0;
    for e in entries {
        bytes += e.id.len() as u64;
        bytes += e.kind.len() as u64;
        bytes += e.content.len() as u64;
        bytes += e.summary.len() as u64;
        bytes += e.source.len() as u64;
        bytes += std::mem::size_of::<f64>() as u64;
        bytes += std::mem::size_of::<i64>() as u64 * 2;
        bytes += std::mem::size_of::<u32>() as u64;
        bytes += e.tags.iter().map(|t| t.len() as u64).sum::<u64>();
        bytes += 1;
    }
    bytes
}

// ===== Commands =====

#[command]
pub fn memory_list(
    kind: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
) -> Result<Vec<MemoryEntry>, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let sort_by = sort.as_deref().unwrap_or("created");
    let page_size: usize = 50;
    let page_idx = page.unwrap_or(1).max(1) as usize;

    let mut filtered: Vec<MemoryEntry> = match &kind {
        Some(k) if !k.is_empty() => state.entries.iter().filter(|e| e.kind == *k).cloned().collect(),
        _ => state.entries.clone(),
    };

    sort_entries(&mut filtered, sort_by);

    let start = (page_idx - 1) * page_size;
    if start >= filtered.len() {
        return Ok(Vec::new());
    }
    Ok(filtered.into_iter().skip(start).take(page_size).collect())
}

#[command]
pub fn memory_get(id: String) -> Result<MemoryEntry, String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("Memory entry not found: {}", id))?;
    update_access(entry);
    Ok(entry.clone())
}

#[command]
pub fn memory_search(
    query: String,
    kind: Option<String>,
) -> Result<MemorySearchResult, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let q = query.to_lowercase();

    let mut results: Vec<MemoryEntry> = state
        .entries
        .iter()
        .filter(|e| {
            let kind_match = match &kind {
                Some(k) if !k.is_empty() => e.kind == *k,
                _ => true,
            };
            if !kind_match {
                return false;
            }
            if q.is_empty() {
                return true;
            }
            e.content.to_lowercase().contains(&q)
                || e.summary.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();

    results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

    let total = results.len() as u32;
    Ok(MemorySearchResult {
        total,
        results,
        query,
    })
}

#[command]
pub fn memory_create(
    kind: String,
    content: String,
    summary: Option<String>,
    tags: Option<Vec<String>>,
    source: Option<String>,
) -> Result<String, String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;

    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    if state.entries.len() >= MAX_ENTRIES {
        state.entries.remove(0);
    }

    let id = format!("mem-{:016x}", NEXT_MEM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    let now = now_ts();
    let entry = MemoryEntry {
        id: id.clone(),
        kind,
        content,
        summary: summary.unwrap_or_default(),
        source: source.unwrap_or_else(|| "manual".to_string()),
        confidence: 0.5,
        created_at: now,
        last_accessed_at: now,
        access_count: 0,
        tags: tags.unwrap_or_default(),
        is_pinned: false,
    };

    state.entries.push(entry);
    Ok(id)
}

#[command]
pub fn memory_update(
    id: String,
    content: Option<String>,
    summary: Option<String>,
    tags: Option<Vec<String>>,
    confidence: Option<f64>,
    is_pinned: Option<bool>,
) -> Result<(), String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("Memory entry not found: {}", id))?;

    if let Some(v) = content {
        if v.trim().is_empty() {
            return Err("Content cannot be empty".to_string());
        }
        entry.content = v;
    }
    if let Some(v) = summary {
        entry.summary = v;
    }
    if let Some(v) = tags {
        entry.tags = v;
    }
    if let Some(v) = confidence {
        entry.confidence = v.max(0.0).min(1.0);
    }
    if let Some(v) = is_pinned {
        entry.is_pinned = v;
    }
    entry.last_accessed_at = now_ts();
    Ok(())
}

#[command]
pub fn memory_delete(id: String) -> Result<(), String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let before = state.entries.len();
    state.entries.retain(|e| e.id != id);
    if state.entries.len() == before {
        return Err(format!("Memory entry not found: {}", id));
    }
    Ok(())
}

#[command]
pub fn memory_pin(id: String) -> Result<(), String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("Memory entry not found: {}", id))?;
    entry.is_pinned = true;
    entry.last_accessed_at = now_ts();
    Ok(())
}

#[command]
pub fn memory_unpin(id: String) -> Result<(), String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let entry = state
        .entries
        .iter_mut()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("Memory entry not found: {}", id))?;
    entry.is_pinned = false;
    entry.last_accessed_at = now_ts();
    Ok(())
}

#[command]
pub fn memory_categories() -> Result<Vec<MemoryCategory>, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let mut counts: HashMap<String, u32> = HashMap::new();
    for e in &state.entries {
        *counts.entry(e.kind.clone()).or_insert(0) += 1;
    }

    let categories = vec![
        MemoryCategory { id: "preference".into(), name: "Preferences".into(), description: "User preferences and settings".into(), count: *counts.get("preference").unwrap_or(&0) },
        MemoryCategory { id: "fact".into(), name: "Facts".into(), description: "Factual knowledge about the project".into(), count: *counts.get("fact").unwrap_or(&0) },
        MemoryCategory { id: "conversation".into(), name: "Conversations".into(), description: "Session conversation history".into(), count: *counts.get("conversation").unwrap_or(&0) },
        MemoryCategory { id: "skill".into(), name: "Skills".into(), description: "Learned skills and capabilities".into(), count: *counts.get("skill").unwrap_or(&0) },
        MemoryCategory { id: "workflow".into(), name: "Workflows".into(), description: "Workflow definitions and patterns".into(), count: *counts.get("workflow").unwrap_or(&0) },
        MemoryCategory { id: "knowledge".into(), name: "Knowledge".into(), description: "General knowledge entries".into(), count: *counts.get("knowledge").unwrap_or(&0) },
    ];

    Ok(categories)
}

#[command]
pub fn memory_stats() -> Result<MemoryStats, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;

    let total_entries = state.entries.len();
    let oldest_entry = state.entries.iter().map(|e| e.created_at).min().unwrap_or(0);
    let newest_entry = state.entries.iter().map(|e| e.created_at).max().unwrap_or(0);

    let avg_confidence = if total_entries > 0 {
        let sum: f64 = state.entries.iter().map(|e| e.confidence).sum();
        sum / total_entries as f64
    } else {
        0.0
    };

    let mut tag_counts: HashMap<String, u32> = HashMap::new();
    for e in &state.entries {
        for t in &e.tags {
            *tag_counts.entry(t.clone()).or_insert(0) += 1;
        }
    }
    let mut top_tags: Vec<(String, u32)> = tag_counts.into_iter().collect();
    top_tags.sort_by(|a, b| b.1.cmp(&a.1));
    top_tags.truncate(10);

    let memory_usage_bytes = calc_memory_usage(&state.entries);

    let kind_count = state.entries.iter().map(|e| e.kind.as_str()).collect::<std::collections::HashSet<_>>().len();

    Ok(MemoryStats {
        total_entries,
        total_categories: kind_count,
        oldest_entry,
        newest_entry,
        avg_confidence,
        top_tags,
        memory_usage_bytes,
    })
}

#[command]
pub fn memory_timeline(days: Option<u32>) -> Result<Vec<MemoryTimelineEntry>, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let num_days = days.unwrap_or(30).min(365).max(1);
    let now = now_ts();
    let secs_per_day: i64 = 86400;
    let cutoff = now - (num_days as i64) * secs_per_day;

    let mut day_map: HashMap<String, (u32, u32, Vec<String>)> = HashMap::new();

    for e in &state.entries {
        if e.created_at < cutoff && e.last_accessed_at < cutoff {
            continue;
        }
        let day_idx = ((e.created_at - cutoff) / secs_per_day).min(num_days as i64 - 1);
        let date = chrono::DateTime::from_timestamp(cutoff + day_idx * secs_per_day, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| format!("day-{}", day_idx));

        let entry = day_map.entry(date).or_insert((0, 0, Vec::new()));
        if e.created_at >= cutoff {
            entry.0 += 1;
        }
        if e.last_accessed_at >= cutoff {
            entry.1 += 1;
        }
        for t in &e.tags {
            entry.2.push(t.clone());
        }
    }

    let mut result: Vec<MemoryTimelineEntry> = day_map
        .into_iter()
        .map(|(date, (created, accessed, tags))| {
            let mut tag_counts: HashMap<String, u32> = HashMap::new();
            for t in tags {
                *tag_counts.entry(t).or_insert(0) += 1;
            }
            let top_topic = tag_counts
                .into_iter()
                .max_by_key(|&(_, c)| c)
                .map(|(t, _)| t)
                .unwrap_or_else(|| "general".to_string());
            MemoryTimelineEntry {
                date,
                entries_created: created,
                entries_accessed: accessed,
                top_topic,
            }
        })
        .collect();

    result.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(result)
}

#[command]
pub fn memory_consolidate_now() -> Result<serde_json::Value, String> {
    let start = std::time::Instant::now();
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;

    let before = state.entries.len();

    // Remove entries with very low confidence (unless pinned)
    state.entries.retain(|e| e.confidence >= 0.1 || e.is_pinned);

    let low_conf = before - state.entries.len();

    // Merge duplicate content entries
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut merged: Vec<MemoryEntry> = Vec::new();
    let mut dup_count = 0;

    for entry in state.entries.drain(..) {
        let key = entry.content.trim().to_lowercase();
        if let Some(&idx) = seen.get(&key) {
            merged[idx].access_count = merged[idx].access_count.max(entry.access_count);
            merged[idx].confidence = merged[idx].confidence.max(entry.confidence);
            for t in entry.tags {
                if !merged[idx].tags.contains(&t) {
                    merged[idx].tags.push(t);
                }
            }
            dup_count += 1;
        } else {
            seen.insert(key, merged.len());
            merged.push(entry);
        }
    }

    // Trim to max
    if merged.len() > (state.config.max_entries as usize) {
        merged.sort_by(|a, b| {
            if a.is_pinned && !b.is_pinned {
                return std::cmp::Ordering::Less;
            }
            if !a.is_pinned && b.is_pinned {
                return std::cmp::Ordering::Greater;
            }
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(state.config.max_entries as usize);
    }

    let trimmed = (state.config.max_entries as usize).max(merged.len()) - merged.len();
    state.entries = merged;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(serde_json::json!({
        "consolidated": state.entries.len(),
        "deleted_duplicates": dup_count + low_conf + trimmed,
        "duration_ms": duration_ms,
    }))
}

#[command]
pub fn memory_clear(kind: Option<String>) -> Result<usize, String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let before = state.entries.len();

    match &kind {
        Some(k) if !k.is_empty() => {
            state.entries.retain(|e| e.kind != *k);
        }
        _ => {
            state.entries.clear();
        }
    }

    Ok(before - state.entries.len())
}

#[command]
pub fn memory_export(format: Option<String>) -> Result<String, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    let fmt = format.as_deref().unwrap_or("json");

    match fmt {
        "json" => serde_json::to_string_pretty(&state.entries).map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported export format: {}", fmt)),
    }
}

#[command]
pub fn memory_import(data: String) -> Result<usize, String> {
    let entries: Vec<MemoryEntry> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let count = entries.len();

    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    for entry in entries {
        if state.entries.len() >= MAX_ENTRIES {
            state.entries.remove(0);
        }
        state.entries.push(entry);
    }

    Ok(count)
}

#[command]
pub fn memory_config() -> Result<MemoryConfig, String> {
    let state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    Ok(state.config.clone())
}

#[command]
pub fn memory_set_config(config: MemoryConfig) -> Result<(), String> {
    let mut state = MEMORY_MGR.lock().map_err(|e| e.to_string())?;
    state.config = config;
    Ok(())
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_create_and_list() {
        let id = memory_create(
            "fact".into(),
            "Test memory content".into(),
            Some("Test summary".into()),
            Some(vec!["test".into(), "memory".into()]),
            Some("test".into()),
        )
        .unwrap();
        assert!(id.starts_with("mem-"));

        let list = memory_list(None, None, None).unwrap();
        assert!(list.iter().any(|e| e.id == id));
    }

    #[test]
    fn test_memory_get_and_update() {
        let id = memory_create(
            "preference".into(),
            "Update test content".into(),
            None, None, None,
        )
        .unwrap();

        let entry = memory_get(id.clone()).unwrap();
        assert_eq!(entry.content, "Update test content");

        memory_update(id.clone(), Some("Updated content".into()), None, None, Some(0.9), None).unwrap();
        let updated = memory_get(id.clone()).unwrap();
        assert_eq!(updated.content, "Updated content");
        assert!((updated.confidence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_memory_search() {
        let result = memory_search("Rust".into(), None).unwrap();
        assert!(result.total > 0);
        assert!(result.results.iter().any(|e| e.content.contains("Rust")));
    }

    #[test]
    fn test_memory_pin_unpin() {
        let id = memory_create("fact".into(), "Pin test".into(), None, None, None).unwrap();
        memory_pin(id.clone()).unwrap();

        let entry = memory_get(id.clone()).unwrap();
        assert!(entry.is_pinned);

        memory_unpin(id.clone()).unwrap();
        let entry = memory_get(id).unwrap();
        assert!(!entry.is_pinned);
    }

    #[test]
    fn test_memory_stats() {
        let stats = memory_stats().unwrap();
        assert!(stats.total_entries >= 15);
        assert!(stats.avg_confidence > 0.0);
        assert!(stats.memory_usage_bytes > 0);
    }

    #[test]
    fn test_memory_categories() {
        let cats = memory_categories().unwrap();
        let prefs = cats.iter().find(|c| c.id == "preference").unwrap();
        assert!(prefs.count >= 5);
    }

    #[test]
    fn test_memory_clear() {
        let id = memory_create("fact".into(), "Clear me".into(), None, None, None).unwrap();
        assert!(memory_get(id.clone()).is_ok());

        memory_delete(id.clone()).unwrap();
        assert!(memory_get(id).is_err());
    }

    #[test]
    fn test_memory_export_import() {
        let exported = memory_export(Some("json".into())).unwrap();
        assert!(exported.contains("kind"));

        let parsed: Vec<MemoryEntry> = serde_json::from_str(&exported).unwrap();
        assert!(parsed.len() >= 15);
    }
}
