//! Persistent crawl queue and checkpoint system (G307.1-2)
//!
//! Provides FIFO-with-priority queue management for crawl frontiers and
//! JSON-file-based checkpoint save/load for session persistence.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Status of a queue entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Entry in the crawl frontier
#[derive(Debug, Clone)]
pub struct CrawlQueueEntry {
    pub url: String,
    pub depth: u32,
    pub priority: u8,
    pub retry_count: u32,
    pub max_retries: u32,
    pub status: QueueStatus,
    pub added_at: String,
    pub next_attempt_at: Option<String>,
    pub referrer: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Configuration for a crawl session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlSessionConfig {
    pub max_pages: u32,
    pub max_depth: u32,
    pub same_domain: bool,
    pub respect_robots: bool,
    pub politeness_ms: u64,
}

impl Default for CrawlSessionConfig {
    fn default() -> Self {
        Self {
            max_pages: 50,
            max_depth: 3,
            same_domain: true,
            respect_robots: true,
            politeness_ms: 1000,
        }
    }
}

/// Crawl statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlStats {
    pub pages_crawled: u32,
    pub bytes_downloaded: u64,
    pub total_time_ms: u64,
    pub errors: u32,
    pub avg_latency_ms: f64,
}

impl Default for CrawlStats {
    fn default() -> Self {
        Self {
            pages_crawled: 0,
            bytes_downloaded: 0,
            total_time_ms: 0,
            errors: 0,
            avg_latency_ms: 0.0,
        }
    }
}

/// Serializable entry for checkpoint persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SerializableEntry {
    pub url: String,
    pub depth: u32,
    pub priority: u8,
    pub retry_count: u32,
    pub max_retries: u32,
    pub status: String,
    pub added_at: String,
    pub next_attempt_at: Option<String>,
    pub referrer: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl From<&CrawlQueueEntry> for SerializableEntry {
    fn from(e: &CrawlQueueEntry) -> Self {
        SerializableEntry {
            url: e.url.clone(),
            depth: e.depth,
            priority: e.priority,
            retry_count: e.retry_count,
            max_retries: e.max_retries,
            status: format!("{:?}", e.status).to_lowercase(),
            added_at: e.added_at.clone(),
            next_attempt_at: e.next_attempt_at.clone(),
            referrer: e.referrer.clone(),
            metadata: e.metadata.clone(),
        }
    }
}

impl From<SerializableEntry> for CrawlQueueEntry {
    fn from(s: SerializableEntry) -> Self {
        let status = match s.status.as_str() {
            "inprogress" => QueueStatus::InProgress,
            "completed" => QueueStatus::Completed,
            "failed" => QueueStatus::Failed,
            "skipped" => QueueStatus::Skipped,
            _ => QueueStatus::Pending,
        };
        CrawlQueueEntry {
            url: s.url,
            depth: s.depth,
            priority: s.priority,
            retry_count: s.retry_count,
            max_retries: s.max_retries,
            status,
            added_at: s.added_at,
            next_attempt_at: s.next_attempt_at,
            referrer: s.referrer,
            metadata: s.metadata,
        }
    }
}

/// Serializable session state snapshot for checkpoint
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SerializableSession {
    pub session_id: String,
    pub start_url: String,
    pub config: CrawlSessionConfig,
    pub frontier: Vec<SerializableEntry>,
    pub visited: Vec<String>,
    pub completed: Vec<String>,
    pub failed: Vec<FailRecord>,
    pub stats: CrawlStats,
    pub checkpoint_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FailRecord {
    pub url: String,
    pub reason: String,
}

/// Crawl state snapshot for checkpointing
#[derive(Debug, Clone)]
pub struct CrawlSessionState {
    pub session_id: String,
    pub start_url: String,
    pub config: CrawlSessionConfig,
    pub frontier: Vec<CrawlQueueEntry>,
    pub visited: Vec<String>,
    pub completed: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub stats: CrawlStats,
    pub checkpoint_at: String,
}

/// Persistent crawl queue manager with checkpoint support
pub struct CrawlQueueManager {
    pub session: CrawlSessionState,
    pub checkpoint_dir: String,
    visited_set: HashSet<String>,
    queue: VecDeque<usize>,
}

impl CrawlQueueManager {
    pub fn new(start_url: &str, config: CrawlSessionConfig, checkpoint_dir: &str) -> Self {
        let now = iso_now();
        let session_id = format!("crawl-{}", now.replace(|c: char| !c.is_ascii_alphanumeric() && c != '-', "_"));

        let start_entry = CrawlQueueEntry {
            url: start_url.to_string(),
            depth: 0,
            priority: 5,
            retry_count: 0,
            max_retries: 3,
            status: QueueStatus::Pending,
            added_at: now.clone(),
            next_attempt_at: None,
            referrer: None,
            metadata: HashMap::new(),
        };

        let mut visited_set = HashSet::new();
        visited_set.insert(start_url.to_string());

        let session = CrawlSessionState {
            session_id: session_id.clone(),
            start_url: start_url.to_string(),
            config,
            frontier: vec![start_entry],
            visited: vec![start_url.to_string()],
            completed: vec![],
            failed: vec![],
            stats: CrawlStats::default(),
            checkpoint_at: now,
        };

        let mut queue = VecDeque::new();
        queue.push_back(0);

        Self {
            session,
            checkpoint_dir: checkpoint_dir.to_string(),
            visited_set,
            queue,
        }
    }

    pub fn enqueue(&mut self, url: &str, depth: u32, referrer: Option<&str>) {
        if self.visited_set.contains(url) {
            return;
        }
        if depth > self.session.config.max_depth {
            return;
        }
        if self.session.frontier.len() as u32 >= self.session.config.max_pages * 2 {
            return;
        }

        self.visited_set.insert(url.to_string());

        let entry = CrawlQueueEntry {
            url: url.to_string(),
            depth,
            priority: 3.min(5u8.saturating_sub(depth as u8)),
            retry_count: 0,
            max_retries: 3,
            status: QueueStatus::Pending,
            added_at: iso_now(),
            next_attempt_at: None,
            referrer: referrer.map(String::from),
            metadata: HashMap::new(),
        };

        let idx = self.session.frontier.len();
        self.session.frontier.push(entry);
        self.session.visited.push(url.to_string());
        self.queue.push_back(idx);
    }

    pub fn dequeue(&mut self) -> Option<CrawlQueueEntry> {
        while let Some(idx) = self.queue.pop_front() {
            let entry = &mut self.session.frontier[idx];
            if entry.status == QueueStatus::Pending {
                entry.status = QueueStatus::InProgress;
                if let Some(ref at) = entry.next_attempt_at {
                    if at.as_str() > iso_now().as_str() {
                        // Not ready yet, put back
                        entry.status = QueueStatus::Pending;
                        self.queue.push_back(idx);
                        continue;
                    }
                }
                return Some(entry.clone());
            }
        }
        None
    }

    pub fn mark_completed(&mut self, url: &str) {
        if let Some(entry) = self.session.frontier.iter_mut().find(|e| e.url == url) {
            entry.status = QueueStatus::Completed;
        }
        self.session.completed.push(url.to_string());
        self.session.stats.pages_crawled += 1;
    }

    pub fn mark_failed(&mut self, url: &str, reason: &str) {
        let should_skip = if let Some(entry) = self.session.frontier.iter_mut().find(|e| e.url == url) {
            entry.retry_count += 1;
            if entry.retry_count >= entry.max_retries {
                entry.status = QueueStatus::Failed;
                true
            } else {
                entry.status = QueueStatus::Pending;
                // Schedule retry after delay
                let delay_secs = 30u64 * entry.retry_count as u64;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let next = now + delay_secs;
                entry.next_attempt_at = Some(timestamp_iso(next));
                // Re-enqueue
                if let Some(idx) = self.session.frontier.iter().position(|e| e.url == url) {
                    self.queue.push_back(idx);
                }
                false
            }
        } else {
            true
        };

        if should_skip {
            self.session.failed.push((url.to_string(), reason.to_string()));
            self.session.stats.errors += 1;
        }
    }

    pub fn is_visited(&self, url: &str) -> bool {
        self.visited_set.contains(url)
    }

    pub fn remaining(&self) -> u32 {
        self.session
            .frontier
            .iter()
            .filter(|e| e.status == QueueStatus::Pending)
            .count() as u32
    }

    pub fn progress(&self) -> f64 {
        let total = self.session.frontier.len().max(1);
        let done = self.session.frontier
            .iter()
            .filter(|e| matches!(e.status, QueueStatus::Completed | QueueStatus::Failed | QueueStatus::Skipped))
            .count();
        done as f64 / total as f64
    }

    pub fn save_checkpoint(&self) -> Result<String, String> {
        let dir = Path::new(&self.checkpoint_dir);
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create checkpoint dir: {}", e))?;

        let s_entries: Vec<SerializableEntry> = self.session.frontier.iter().map(|e| e.into()).collect();
        let s_failed: Vec<FailRecord> = self.session.failed.iter().map(|(u, r)| FailRecord {
            url: u.clone(),
            reason: r.clone(),
        }).collect();

        let s_session = SerializableSession {
            session_id: self.session.session_id.clone(),
            start_url: self.session.start_url.clone(),
            config: self.session.config.clone(),
            frontier: s_entries,
            visited: self.session.visited.clone(),
            completed: self.session.completed.clone(),
            failed: s_failed,
            stats: self.session.stats.clone(),
            checkpoint_at: iso_now(),
        };

        let json = serde_json::to_string_pretty(&s_session)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let filepath = dir.join(format!("{}.json", self.session.session_id));
        fs::write(&filepath, &json).map_err(|e| format!("Failed to write checkpoint: {}", e))?;

        Ok(filepath.to_string_lossy().to_string())
    }

    pub fn load_checkpoint(session_id: &str, dir: &str) -> Result<Self, String> {
        let filepath = Path::new(dir).join(format!("{}.json", session_id));
        let json = fs::read_to_string(&filepath)
            .map_err(|e| format!("Failed to read checkpoint {}: {}", filepath.display(), e))?;

        let s_session: SerializableSession =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {}", e))?;

        let mut visited_set = HashSet::new();
        for url in &s_session.visited {
            visited_set.insert(url.clone());
        }

        let entries: Vec<CrawlQueueEntry> = s_session.frontier.into_iter().map(|e| e.into()).collect();
        let failed: Vec<(String, String)> = s_session.failed.into_iter().map(|f| (f.url, f.reason)).collect();

        let mut queue = VecDeque::new();
        for (idx, entry) in entries.iter().enumerate() {
            if entry.status == QueueStatus::Pending || entry.status == QueueStatus::InProgress {
                queue.push_back(idx);
            }
        }

        let session = CrawlSessionState {
            session_id: s_session.session_id,
            start_url: s_session.start_url,
            config: s_session.config,
            frontier: entries,
            visited: s_session.visited,
            completed: s_session.completed,
            failed,
            stats: s_session.stats,
            checkpoint_at: s_session.checkpoint_at,
        };

        Ok(Self {
            session,
            checkpoint_dir: dir.to_string(),
            visited_set,
            queue,
        })
    }

    pub fn list_checkpoints(dir: &str) -> Result<Vec<String>, String> {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            return Ok(vec![]);
        }
        let mut sessions = Vec::new();
        for entry in fs::read_dir(dir_path).map_err(|e| format!("Failed to read dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    sessions.push(stem.to_string());
                }
            }
        }
        sessions.sort();
        Ok(sessions)
    }

    pub fn cleanup(&self) -> Result<(), String> {
        let filepath = Path::new(&self.checkpoint_dir).join(format!("{}.json", self.session.session_id));
        if filepath.exists() {
            fs::remove_file(&filepath).map_err(|e| format!("Failed to remove checkpoint: {}", e))?;
        }
        Ok(())
    }

    pub fn stats(&self) -> CrawlStats {
        self.session.stats.clone()
    }
}

fn iso_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        year, month, day, hours, minutes, seconds, nanos / 1000
    )
}

fn timestamp_iso(unix_secs: u64) -> String {
    let days = unix_secs / 86400;
    let time_secs = unix_secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000000Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for &md in months_days {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }
    (y as u64, m, (d + 1) as u64)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_config() -> CrawlSessionConfig {
        CrawlSessionConfig {
            max_pages: 20,
            max_depth: 2,
            same_domain: true,
            respect_robots: true,
            politeness_ms: 100,
        }
    }

    fn test_manager(dir: &str) -> CrawlQueueManager {
        CrawlQueueManager::new("https://example.com", test_config(), dir)
    }

    #[test]
    fn test_queue_enqueue_dequeue() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);

        manager.enqueue("https://example.com/page1", 1, None);
        manager.enqueue("https://example.com/page2", 1, None);

        let entry1 = manager.dequeue();
        assert!(entry1.is_some());
        assert_eq!(entry1.unwrap().url, "https://example.com");

        let entry2 = manager.dequeue();
        assert!(entry2.is_some());
        assert_eq!(entry2.unwrap().url, "https://example.com/page1");

        let entry3 = manager.dequeue();
        assert!(entry3.is_some());
        assert_eq!(entry3.unwrap().url, "https://example.com/page2");

        assert!(manager.dequeue().is_none());
    }

    #[test]
    fn test_queue_dedup() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);

        manager.enqueue("https://example.com/page1", 1, None);
        manager.enqueue("https://example.com/page1", 1, None); // duplicate
        manager.enqueue("https://example.com/page2", 1, None);

        assert_eq!(manager.remaining(), 2);
    }

    #[test]
    fn test_queue_fifo_order() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);

        manager.enqueue("https://example.com/a", 1, None);
        manager.enqueue("https://example.com/b", 1, None);
        manager.enqueue("https://example.com/c", 1, None);

        // First dequeue is the start_url
        let first = manager.dequeue().unwrap();
        assert_eq!(first.url, "https://example.com");

        let second = manager.dequeue().unwrap();
        assert_eq!(second.url, "https://example.com/a");

        let third = manager.dequeue().unwrap();
        assert_eq!(third.url, "https://example.com/b");
    }

    #[test]
    fn test_queue_mark_completed() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);
        manager.enqueue("https://example.com/page", 1, None);

        let entry = manager.dequeue().unwrap();
        manager.mark_completed(&entry.url);

        let stats = manager.stats();
        assert_eq!(stats.pages_crawled, 1);
        assert!(manager.session.completed.contains(&entry.url));
    }

    #[test]
    fn test_queue_mark_failed_retries() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);

        // Add a page with max_retries = 2
        let entry = CrawlQueueEntry {
            url: "https://example.com/failpage".to_string(),
            depth: 1,
            priority: 3,
            retry_count: 0,
            max_retries: 2,
            status: QueueStatus::Pending,
            added_at: iso_now(),
            next_attempt_at: None,
            referrer: None,
            metadata: HashMap::new(),
        };
        manager.session.frontier.push(entry);
        manager.visited_set.insert("https://example.com/failpage".to_string());
        manager.session.visited.push("https://example.com/failpage".to_string());
        manager.queue.push_back(manager.session.frontier.len() - 1);

        // First failure (retry)
        manager.mark_failed("https://example.com/failpage", "timeout");
        assert!(manager.session.failed.is_empty()); // Not yet failed, retrying
        assert_eq!(manager.remaining(), 1);

        // Second failure (still retry since retry_count=1, max_retries=2)
        manager.mark_failed("https://example.com/failpage", "timeout again");
        assert!(manager.session.failed.is_empty());
        assert_eq!(manager.remaining(), 1);

        // Third failure (now should fail, retry_count=2 >= max_retries=2)
        manager.mark_failed("https://example.com/failpage", "third time");
        assert_eq!(manager.session.failed.len(), 1);
    }

    #[test]
    fn test_queue_is_visited() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);

        assert!(manager.is_visited("https://example.com")); // start_url
        assert!(!manager.is_visited("https://other.com"));

        manager.enqueue("https://example.com/newpage", 1, None);
        assert!(manager.is_visited("https://example.com/newpage"));
    }

    #[test]
    fn test_queue_progress() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);
        manager.enqueue("https://example.com/a", 1, None);
        manager.enqueue("https://example.com/b", 1, None);

        assert!((manager.progress() - 0.0).abs() < 0.01);

        let e1 = manager.dequeue().unwrap();
        manager.mark_completed(&e1.url);
        assert!(manager.progress() > 0.0);
        assert!(manager.progress() < 1.0);

        // Finish all
        while let Some(e) = manager.dequeue() {
            manager.mark_completed(&e.url);
        }
        assert!((manager.progress() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_queue_remaining() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);
        assert_eq!(manager.remaining(), 1); // start_url

        manager.enqueue("https://example.com/a", 1, None);
        assert_eq!(manager.remaining(), 2);

        manager.dequeue();
        assert_eq!(manager.remaining(), 1);
    }

    #[test]
    fn test_save_and_load_checkpoint() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);
        manager.enqueue("https://example.com/page1", 1, Some("https://example.com"));
        manager.enqueue("https://example.com/page2", 1, None);

        // Process one
        let entry = manager.dequeue().unwrap();
        manager.mark_completed(&entry.url);

        let saved_path = manager.save_checkpoint().unwrap();
        assert!(std::path::Path::new(&saved_path).exists());

        // Load into new manager
        let loaded = CrawlQueueManager::load_checkpoint(&manager.session.session_id, &dir).unwrap();
        assert_eq!(loaded.session.start_url, "https://example.com");
        assert_eq!(loaded.session.completed.len(), 1);
        assert_eq!(loaded.session.frontier.len(), 3); // start_url + 2 pages
        assert!(loaded.is_visited("https://example.com/page1"));

        // Cleanup
        let _ = manager.cleanup();
        assert!(!std::path::Path::new(&saved_path).exists());
    }

    #[test]
    fn test_session_state_serialization() {
        let dir = temp_dir();
        let mut manager = CrawlQueueManager::new(
            "https://start.com",
            CrawlSessionConfig {
                max_pages: 100,
                max_depth: 5,
                same_domain: false,
                respect_robots: true,
                politeness_ms: 500,
            },
            &dir,
        );

        manager.enqueue("https://start.com/page1", 1, None);
        manager.enqueue("https://other.com/page", 2, None);
        manager.mark_completed("https://start.com");
        manager.mark_failed("https://other.com/page", "not found");

        let path = manager.save_checkpoint().unwrap();
        let loaded = CrawlQueueManager::load_checkpoint(&manager.session.session_id, &dir).unwrap();

        assert_eq!(loaded.session.session_id, manager.session.session_id);
        assert_eq!(loaded.session.config.max_pages, 100);
        assert_eq!(loaded.session.config.max_depth, 5);
        assert_eq!(loaded.session.config.same_domain, false);
        assert_eq!(loaded.session.completed.len(), 1);
        assert_eq!(loaded.session.failed.len(), 1);
        assert_eq!(loaded.session.failed[0].0, "https://other.com/page");

        let _ = manager.cleanup();
    }

    #[test]
    fn test_config_default() {
        let config = CrawlSessionConfig::default();
        assert_eq!(config.max_pages, 50);
        assert_eq!(config.max_depth, 3);
        assert!(config.same_domain);
        assert!(config.respect_robots);
        assert_eq!(config.politeness_ms, 1000);
    }

    #[test]
    fn test_enqueue_respects_max_depth() {
        let dir = temp_dir();
        let mut manager = test_manager(&dir);
        // max_depth = 2 from test_config

        manager.enqueue("https://example.com/deep1", 2, None);
        manager.enqueue("https://example.com/too_deep", 3, None); // Should be rejected

        assert_eq!(manager.remaining(), 1); // start_url + deep1
    }

    #[test]
    fn test_list_checkpoints() {
        let dir = temp_dir();
        let mut m1 = test_manager(&dir);
        let mut m2 = CrawlQueueManager::new("https://other.com", test_config(), &dir);

        m1.save_checkpoint().unwrap();
        m2.save_checkpoint().unwrap();

        let sessions = CrawlQueueManager::list_checkpoints(&dir).unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().any(|s| s.contains("crawl-")));
    }

    #[test]
    fn test_load_nonexistent_checkpoint() {
        let result = CrawlQueueManager::load_checkpoint("nonexistent", "/tmp/no_such_dir");
        assert!(result.is_err());
    }

    fn temp_dir() -> String {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("crawl_test_{}", ts));
        std::fs::create_dir_all(&dir).unwrap_or(());
        dir.to_string_lossy().to_string()
    }
}
