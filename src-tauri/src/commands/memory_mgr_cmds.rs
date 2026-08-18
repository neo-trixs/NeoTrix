use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::command;
use rusqlite::{Connection, OptionalExtension};

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

// ===== State (SQLite 持久化) =====

const MAX_ENTRIES: usize = 10000;

// 测试时可覆盖数据库路径 (thread-local: 仅影响当前测试线程, 避免并行测试互相干扰)
thread_local! {
    static DB_OVERRIDE: std::cell::Cell<Option<PathBuf>> = const { std::cell::Cell::new(None) };
}

static NEXT_MEM_ID: AtomicU64 = AtomicU64::new(0x1000);

/// 统一数据库路径: ~/.neotrix/desktop.db
fn desktop_db_path() -> PathBuf {
    if let Some(p) = DB_OVERRIDE.with(|c| c.replace(None)) {
        DB_OVERRIDE.with(|c| c.set(Some(p.clone())));
        return p;
    }
    if let Ok(p) = std::env::var("NEOTRIX_DESKTOP_DB") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    dirs::home_dir()
        .map(|h| h.join(".neotrix").join("desktop.db"))
        .unwrap_or_else(|| PathBuf::from(".neotrix/desktop.db"))
}

/// 打开桌面数据库并确保表结构存在 (每次操作独立打开, 由 SQLite 文件锁保证并发安全)
fn open_db() -> Result<Connection, String> {
    let path = desktop_db_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建数据目录 {:?} 失败: {}", parent, e))?;
    }
    let conn = Connection::open(&path).map_err(|e| format!("打开数据库 {:?} 失败: {}", path, e))?;
    conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
    let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT 'general',
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS app_state (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .map_err(|e| e.to_string())?;
    seed_default_entries(&conn)?;
    Ok(conn)
}

// ===== Helpers =====

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// 全量 MemoryEntry 以 JSON 存入 content 列, 读取时反序列化还原
fn load_all(conn: &Connection, kind: Option<&str>) -> Result<Vec<MemoryEntry>, String> {
    let bodies: Vec<String> = match kind.filter(|k| !k.is_empty()) {
        Some(k) => {
            let mut stmt = conn
                .prepare("SELECT content FROM memories WHERE category = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(rusqlite::params![k], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        }
        None => {
            let mut stmt = conn
                .prepare("SELECT content FROM memories")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        }
    };

    let mut entries = Vec::with_capacity(bodies.len());
    for body in bodies {
        entries.push(serde_json::from_str(&body).map_err(|e| e.to_string())?);
    }
    Ok(entries)
}

fn get_entry(conn: &Connection, id: &str) -> Result<MemoryEntry, String> {
    let body: String = conn
        .query_row(
            "SELECT content FROM memories WHERE id = ?1",
            rusqlite::params![id],
            |r| r.get(0),
        )
        .map_err(|_| format!("Memory entry not found: {}", id))?;
    serde_json::from_str(&body).map_err(|e| e.to_string())
}

fn insert_entry(conn: &Connection, entry: &MemoryEntry) -> Result<(), String> {
    let title = if !entry.summary.is_empty() { entry.summary.clone() } else { entry.content.clone() };
    let body = serde_json::to_string(entry).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO memories (id, title, content, category, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![entry.id, title, body, entry.kind, entry.created_at, entry.last_accessed_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn update_entry(conn: &Connection, entry: &MemoryEntry) -> Result<(), String> {
    let title = if !entry.summary.is_empty() { entry.summary.clone() } else { entry.content.clone() };
    let body = serde_json::to_string(entry).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE memories SET title = ?1, content = ?2, category = ?3, updated_at = ?4 WHERE id = ?5",
        rusqlite::params![title, body, entry.kind, entry.last_accessed_at, entry.id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// 生成不冲突的 mem- 前缀 id
fn next_mem_id(conn: &Connection) -> Result<String, String> {
    loop {
        let id = format!("mem-{:016x}", NEXT_MEM_ID.fetch_add(1, Ordering::Relaxed));
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM memories WHERE id = ?1)",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if !exists {
            return Ok(id);
        }
    }
}

/// 首次使用记忆库时写入默认示例条目 (表为空时才播种, 避免重复)
fn seed_default_entries(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Ok(());
    }
    for entry in build_seed_entries() {
        insert_entry(conn, &entry)?;
    }
    Ok(())
}

fn build_seed_entries() -> Vec<MemoryEntry> {
    let now = now_ts();
    let mut out = Vec::new();
    let mut add = |kind: &str, content: &str, summary: &str, source: &str, conf: f64, tags: Vec<&str>| {
        let id = format!("mem-{:016x}", out.len() as u64 + 1);
        out.push(MemoryEntry {
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

    out
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

/// 合并重复内容 + 剔除低置信度 + 裁剪到上限 (与旧内存实现一致)
fn consolidate_entries(entries: Vec<MemoryEntry>, max_entries: usize) -> Vec<MemoryEntry> {
    let kept: Vec<MemoryEntry> =
        entries.into_iter().filter(|e| e.confidence >= 0.1 || e.is_pinned).collect();

    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut merged: Vec<MemoryEntry> = Vec::new();

    for entry in kept {
        let key = entry.content.trim().to_lowercase();
        if let Some(&idx) = seen.get(&key) {
            merged[idx].access_count = merged[idx].access_count.max(entry.access_count);
            merged[idx].confidence = merged[idx].confidence.max(entry.confidence);
            for t in entry.tags {
                if !merged[idx].tags.contains(&t) {
                    merged[idx].tags.push(t);
                }
            }
        } else {
            seen.insert(key, merged.len());
            merged.push(entry);
        }
    }

    if merged.len() > max_entries {
        merged.sort_by(|a, b| {
            if a.is_pinned && !b.is_pinned {
                return std::cmp::Ordering::Less;
            }
            if !a.is_pinned && b.is_pinned {
                return std::cmp::Ordering::Greater;
            }
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });
        merged.truncate(max_entries);
    }

    merged
}

// ===== Commands =====

#[command]
pub fn memory_list(
    kind: Option<String>,
    page: Option<u32>,
    sort: Option<String>,
) -> Result<Vec<MemoryEntry>, String> {
    let conn = open_db()?;
    let sort_by = sort.as_deref().unwrap_or("created");
    let page_size: usize = 50;
    let page_idx = page.unwrap_or(1).max(1) as usize;

    let mut filtered = load_all(&conn, kind.as_deref())?;
    sort_entries(&mut filtered, sort_by);

    let start = (page_idx - 1) * page_size;
    if start >= filtered.len() {
        return Ok(Vec::new());
    }
    Ok(filtered.into_iter().skip(start).take(page_size).collect())
}

#[command]
pub fn memory_get(id: String) -> Result<MemoryEntry, String> {
    let conn = open_db()?;
    let mut entry = get_entry(&conn, &id)?;
    update_access(&mut entry);
    update_entry(&conn, &entry)?;
    Ok(entry)
}

#[command]
pub fn memory_search(
    query: String,
    kind: Option<String>,
) -> Result<MemorySearchResult, String> {
    let conn = open_db()?;
    let entries = load_all(&conn, kind.as_deref())?;
    let q = query.to_lowercase();

    let mut results: Vec<MemoryEntry> = entries
        .into_iter()
        .filter(|e| {
            if q.is_empty() {
                return true;
            }
            e.content.to_lowercase().contains(&q)
                || e.summary.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
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
    let conn = open_db()?;

    if content.trim().is_empty() {
        return Err("Content cannot be empty".to_string());
    }

    // 超限时裁剪最旧一条
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if count as usize >= MAX_ENTRIES {
        conn.execute(
            "DELETE FROM memories WHERE id IN (SELECT id FROM memories ORDER BY created_at ASC LIMIT 1)",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    let id = next_mem_id(&conn)?;
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

    insert_entry(&conn, &entry)?;
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
    let conn = open_db()?;
    let mut entry = get_entry(&conn, &id)?;

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
    update_entry(&conn, &entry)
}

#[command]
pub fn memory_delete(id: String) -> Result<(), String> {
    let conn = open_db()?;
    let affected = conn
        .execute("DELETE FROM memories WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    if affected == 0 {
        return Err(format!("Memory entry not found: {}", id));
    }
    Ok(())
}

#[command]
pub fn memory_pin(id: String) -> Result<(), String> {
    let conn = open_db()?;
    let mut entry = get_entry(&conn, &id)?;
    entry.is_pinned = true;
    entry.last_accessed_at = now_ts();
    update_entry(&conn, &entry)
}

#[command]
pub fn memory_unpin(id: String) -> Result<(), String> {
    let conn = open_db()?;
    let mut entry = get_entry(&conn, &id)?;
    entry.is_pinned = false;
    entry.last_accessed_at = now_ts();
    update_entry(&conn, &entry)
}

#[command]
pub fn memory_categories() -> Result<Vec<MemoryCategory>, String> {
    let conn = open_db()?;
    let entries = load_all(&conn, None)?;
    let mut counts: HashMap<String, u32> = HashMap::new();
    for e in &entries {
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
    let conn = open_db()?;
    let entries = load_all(&conn, None)?;

    let total_entries = entries.len();
    let oldest_entry = entries.iter().map(|e| e.created_at).min().unwrap_or(0);
    let newest_entry = entries.iter().map(|e| e.created_at).max().unwrap_or(0);

    let avg_confidence = if total_entries > 0 {
        let sum: f64 = entries.iter().map(|e| e.confidence).sum();
        sum / total_entries as f64
    } else {
        0.0
    };

    let mut tag_counts: HashMap<String, u32> = HashMap::new();
    for e in &entries {
        for t in &e.tags {
            *tag_counts.entry(t.clone()).or_insert(0) += 1;
        }
    }
    let mut top_tags: Vec<(String, u32)> = tag_counts.into_iter().collect();
    top_tags.sort_by(|a, b| b.1.cmp(&a.1));
    top_tags.truncate(10);

    let memory_usage_bytes = calc_memory_usage(&entries);

    let kind_count = entries.iter().map(|e| e.kind.as_str()).collect::<std::collections::HashSet<_>>().len();

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
    let conn = open_db()?;
    let entries = load_all(&conn, None)?;
    let num_days = days.unwrap_or(30).min(365).max(1);
    let now = now_ts();
    let secs_per_day: i64 = 86400;
    let cutoff = now - (num_days as i64) * secs_per_day;

    let mut day_map: HashMap<String, (u32, u32, Vec<String>)> = HashMap::new();

    for e in &entries {
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
    let mut conn = open_db()?;

    let all = load_all(&conn, None)?;
    let before = all.len();
    let low_conf = all.iter().filter(|e| e.confidence < 0.1 && !e.is_pinned).count();

    let merged = consolidate_entries(all, MAX_ENTRIES);
    let deleted_duplicates = (before - low_conf).saturating_sub(merged.len());

    // 原子化重写数据库
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM memories", []).map_err(|e| e.to_string())?;
    for entry in &merged {
        insert_entry(&tx, entry)?;
    }
    tx.commit().map_err(|e| e.to_string())?;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(serde_json::json!({
        "consolidated": merged.len(),
        "deleted_duplicates": deleted_duplicates,
        "duration_ms": duration_ms,
    }))
}

#[command]
pub fn memory_clear(kind: Option<String>) -> Result<usize, String> {
    let conn = open_db()?;
    let affected = match &kind {
        Some(k) if !k.is_empty() => {
            conn.execute("DELETE FROM memories WHERE category = ?1", rusqlite::params![k])
                .map_err(|e| e.to_string())?
        }
        _ => conn.execute("DELETE FROM memories", []).map_err(|e| e.to_string())?,
    };
    Ok(affected)
}

#[command]
pub fn memory_export(format: Option<String>) -> Result<String, String> {
    let conn = open_db()?;
    let fmt = format.as_deref().unwrap_or("json");
    let entries = load_all(&conn, None)?;

    match fmt {
        "json" => serde_json::to_string_pretty(&entries).map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported export format: {}", fmt)),
    }
}

#[command]
pub fn memory_import(data: String) -> Result<usize, String> {
    let entries: Vec<MemoryEntry> = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    let count = entries.len();

    let conn = open_db()?;
    for entry in entries {
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if total as usize >= MAX_ENTRIES {
            conn.execute(
                "DELETE FROM memories WHERE id IN (SELECT id FROM memories ORDER BY created_at ASC LIMIT 1)",
                [],
            )
            .map_err(|e| e.to_string())?;
        }
        insert_entry(&conn, &entry)?;
    }

    Ok(count)
}

#[command]
pub fn memory_config() -> Result<MemoryConfig, String> {
    let conn = open_db()?;
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM app_state WHERE key = 'memory_config'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    match value {
        Some(v) => serde_json::from_str(&v).map_err(|e| e.to_string()),
        None => Ok(MemoryConfig::default()),
    }
}

#[command]
pub fn memory_set_config(config: MemoryConfig) -> Result<(), String> {
    let conn = open_db()?;
    let value = serde_json::to_string(&config).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO app_state (key, value) VALUES ('memory_config', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    static DB_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 使用临时目录的数据库执行闭包, 验证持久化且不污染真实数据
    fn with_temp_db<T>(f: impl FnOnce() -> T) -> T {
        let n = DB_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("neotrix-memory-test-{}-{}", std::process::id(), n));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("desktop.db");
        DB_OVERRIDE.with(|c| c.set(Some(path)));
        let result = f();
        DB_OVERRIDE.with(|c| c.set(None));
        result
    }

    #[test]
    fn test_memory_create_and_list() {
        with_temp_db(|| {
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
        });
    }

    #[test]
    fn test_memory_get_and_update() {
        with_temp_db(|| {
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
        });
    }

    #[test]
    fn test_memory_search() {
        with_temp_db(|| {
            let result = memory_search("Rust".into(), None).unwrap();
            assert!(result.total > 0);
            assert!(result.results.iter().any(|e| e.content.contains("Rust")));
        });
    }

    #[test]
    fn test_memory_pin_unpin() {
        with_temp_db(|| {
            let id = memory_create("fact".into(), "Pin test".into(), None, None, None).unwrap();
            memory_pin(id.clone()).unwrap();

            let entry = memory_get(id.clone()).unwrap();
            assert!(entry.is_pinned);

            memory_unpin(id.clone()).unwrap();
            let entry = memory_get(id).unwrap();
            assert!(!entry.is_pinned);
        });
    }

    #[test]
    fn test_memory_stats() {
        with_temp_db(|| {
            let stats = memory_stats().unwrap();
            assert!(stats.total_entries >= 15);
            assert!(stats.avg_confidence > 0.0);
            assert!(stats.memory_usage_bytes > 0);
        });
    }

    #[test]
    fn test_memory_categories() {
        with_temp_db(|| {
            let cats = memory_categories().unwrap();
            let prefs = cats.iter().find(|c| c.id == "preference").unwrap();
            assert!(prefs.count >= 5);
        });
    }

    #[test]
    fn test_memory_clear() {
        with_temp_db(|| {
            let id = memory_create("fact".into(), "Clear me".into(), None, None, None).unwrap();
            assert!(memory_get(id.clone()).is_ok());

            memory_delete(id.clone()).unwrap();
            assert!(memory_get(id).is_err());
        });
    }

    #[test]
    fn test_memory_export_import() {
        with_temp_db(|| {
            let exported = memory_export(Some("json".into())).unwrap();
            assert!(exported.contains("kind"));

            let parsed: Vec<MemoryEntry> = serde_json::from_str(&exported).unwrap();
            assert!(parsed.len() >= 15);
        });
    }

    #[test]
    fn test_memory_consolidate_and_persist() {
        with_temp_db(|| {
            let id = memory_create("fact".into(), "Consolidate me".into(), None, None, None).unwrap();
            let result = memory_consolidate_now().unwrap();
            assert!(result["consolidated"].as_u64().unwrap() > 0);

            // 合并重写后 id 仍应从 DB 读到
            assert!(memory_get(id).is_ok());
        });
    }
}
