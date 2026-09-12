//! Download progress persistence — cross-restart resume support.
//!
//! Provides JSON-backed storage for download records, enabling resume
//! detection and progress tracking across process restarts.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════════════
// DownloadRecord — single download entry
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: String,
    pub url: String,
    pub output: PathBuf,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub media_kind: String,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub speed_bps: f64,
    pub error: Option<String>,
}

impl DownloadRecord {
    pub fn new(url: String, output: PathBuf, media_kind: String) -> Self {
        let now = unix_now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            output,
            total_bytes: None,
            downloaded_bytes: 0,
            media_kind,
            status: "downloading".into(),
            created_at: now,
            updated_at: now,
            speed_bps: 0.0,
            error: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.status == "complete"
    }

    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }

    pub fn is_resumable(&self) -> bool {
        self.status == "downloading" || self.status == "paused"
    }

    pub fn mark_complete(&mut self, total_bytes: u64) {
        self.status = "complete".into();
        self.downloaded_bytes = total_bytes;
        self.total_bytes = Some(total_bytes);
        self.updated_at = unix_now();
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = "failed".into();
        self.error = Some(error);
        self.updated_at = unix_now();
    }

    pub fn update_progress(&mut self, downloaded: u64, total: Option<u64>, speed_bps: f64) {
        self.downloaded_bytes = downloaded;
        self.total_bytes = total;
        self.speed_bps = speed_bps;
        self.updated_at = unix_now();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DownloadStore — thread-safe JSON persistence
// ═══════════════════════════════════════════════════════════════════════════

pub struct DownloadStore {
    records: Arc<RwLock<Vec<DownloadRecord>>>,
    file_path: PathBuf,
}

impl DownloadStore {
    pub fn new() -> Self {
        let file_path = default_store_path();
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            file_path,
        }
    }

    pub fn with_path(file_path: PathBuf) -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            file_path,
        }
    }

    pub async fn load(&self) -> Result<(), String> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let data = tokio::fs::read(&self.file_path)
            .await
            .map_err(|e| format!("read store: {}", e))?;

        if data.is_empty() {
            return Ok(());
        }

        let loaded: Vec<DownloadRecord> =
            serde_json::from_slice(&data).map_err(|e| format!("parse store: {}", e))?;

        let mut records = self.records.write().await;
        *records = loaded;
        Ok(())
    }

    pub async fn save(&self) -> Result<(), String> {
        let records = self.records.read().await;
        let data = serde_json::to_string_pretty(&*records)
            .map_err(|e| format!("serialize store: {}", e))?;

        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("create dir: {}", e))?;
        }

        tokio::fs::write(&self.file_path, data)
            .await
            .map_err(|e| format!("write store: {}", e))?;

        Ok(())
    }

    pub async fn add_record(&self, record: DownloadRecord) {
        let mut records = self.records.write().await;
        records.push(record);
    }

    pub async fn update_record<F>(&self, id: &str, update_fn: F) -> Option<DownloadRecord>
    where
        F: FnOnce(&mut DownloadRecord),
    {
        let mut records = self.records.write().await;
        if let Some(record) = records.iter_mut().find(|r| r.id == id) {
            update_fn(record);
            Some(record.clone())
        } else {
            None
        }
    }

    pub async fn get_record(&self, id: &str) -> Option<DownloadRecord> {
        let records = self.records.read().await;
        records.iter().find(|r| r.id == id).cloned()
    }

    pub async fn get_by_url(&self, url: &str) -> Option<DownloadRecord> {
        let records = self.records.read().await;
        records
            .iter()
            .rfind(|r| r.url == url && r.is_resumable())
            .cloned()
    }

    pub async fn list_records(&self) -> Vec<DownloadRecord> {
        let records = self.records.read().await;
        records.clone()
    }

    pub async fn cleanup_old(&self, days: u32) {
        let cutoff = unix_now().saturating_sub(days as u64 * 86400);
        let mut records = self.records.write().await;
        records.retain(|r| r.updated_at >= cutoff);
    }

    pub async fn record_count(&self) -> usize {
        let records = self.records.read().await;
        records.len()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ResumeSupport — integration helpers
// ═══════════════════════════════════════════════════════════════════════════

pub fn check_resume(_url: &str, output: &Path) -> Option<u64> {
    let done_marker = output.with_extension("done");

    let file_size = std::fs::metadata(output).ok().map(|m| m.len()).unwrap_or(0);
    let has_done = done_marker.exists();

    if has_done {
        if file_size > 0 {
            return Some(file_size);
        }
        return Some(0);
    }

    if file_size > 0 {
        return Some(file_size);
    }

    None
}

pub fn mark_done(output: &Path) -> Result<(), String> {
    let done_marker = output.with_extension("done");
    std::fs::write(&done_marker, "").map_err(|e| format!("write done marker: {}", e))
}

pub fn is_done(output: &Path) -> bool {
    let done_marker = output.with_extension("done");
    done_marker.exists()
}

// ═══════════════════════════════════════════════════════════════════════════
// AutoSave — periodic persistence during downloads
// ═══════════════════════════════════════════════════════════════════════════

pub struct AutoSave {
    store: Arc<DownloadStore>,
    interval: Duration,
}

impl AutoSave {
    pub fn new(store: Arc<DownloadStore>) -> Self {
        Self {
            store,
            interval: Duration::from_secs(5),
        }
    }

    pub fn with_interval(store: Arc<DownloadStore>, interval: Duration) -> Self {
        Self { store, interval }
    }

    pub async fn run(&self, mut shutdown: tokio::sync::watch::Receiver<bool>) {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(self.interval) => {
                    if let Err(e) = self.store.save().await {
                        eprintln!("[persistence] auto-save failed: {}", e);
                    }
                }
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        let _ = self.store.save().await;
                        break;
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PipelineConfig integration
// ═══════════════════════════════════════════════════════════════════════════

pub fn attach_persistence(config: &mut super::streaming::PipelineConfig) -> Arc<DownloadStore> {
    let store = Arc::new(DownloadStore::new());
    config.persistence = Some(store.clone());
    store
}

// ═══════════════════════════════════════════════════════════════════════════
// Utilities
// ═══════════════════════════════════════════════════════════════════════════

fn default_store_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".neotrix")
        .join("downloads.json")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_store_add_and_get() {
        let store = DownloadStore::with_path(temp_dir().join("nt_test_store.json"));
        let record = DownloadRecord::new(
            "https://example.com/file.mp4".into(),
            temp_dir().join("file.mp4"),
            "VideoMp4".into(),
        );
        let id = record.id.clone();

        store.add_record(record).await;
        assert!(store.get_record(&id).await.is_some());
        assert!(store
            .get_by_url("https://example.com/file.mp4")
            .await
            .is_some());

        let _ = tokio::fs::remove_file(&store.file_path).await;
    }

    #[tokio::test]
    async fn test_store_update() {
        let store = DownloadStore::with_path(temp_dir().join("nt_test_store_update.json"));
        let record = DownloadRecord::new(
            "https://example.com/file.mp4".into(),
            temp_dir().join("file.mp4"),
            "VideoMp4".into(),
        );
        let id = record.id.clone();

        store.add_record(record).await;
        let updated = store
            .update_record(&id, |r| {
                r.update_progress(1024, Some(4096), 512.0);
            })
            .await
            .unwrap();

        assert_eq!(updated.downloaded_bytes, 1024);
        assert_eq!(updated.total_bytes, Some(4096));

        let _ = tokio::fs::remove_file(&store.file_path).await;
    }

    #[tokio::test]
    async fn test_store_save_load() {
        let path = temp_dir().join("nt_test_store_persist.json");
        let store1 = DownloadStore::with_path(path.clone());
        let record = DownloadRecord::new(
            "https://example.com/file.mp4".into(),
            temp_dir().join("file.mp4"),
            "VideoMp4".into(),
        );
        let id = record.id.clone();
        store1.add_record(record).await;
        store1.save().await.unwrap();

        let store2 = DownloadStore::with_path(path.clone());
        store2.load().await.unwrap();
        assert!(store2.get_record(&id).await.is_some());

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn test_cleanup_old() {
        let store = DownloadStore::with_path(temp_dir().join("nt_test_cleanup.json"));

        let mut old = DownloadRecord::new(
            "https://example.com/old.mp4".into(),
            temp_dir().join("old.mp4"),
            "VideoMp4".into(),
        );
        old.updated_at = unix_now() - 10 * 86400;

        let fresh = DownloadRecord::new(
            "https://example.com/new.mp4".into(),
            temp_dir().join("new.mp4"),
            "VideoMp4".into(),
        );

        store.add_record(old).await;
        store.add_record(fresh).await;

        store.cleanup_old(7).await;
        assert_eq!(store.record_count().await, 1);

        let _ = tokio::fs::remove_file(&store.file_path).await;
    }

    #[test]
    fn test_check_resume_no_file() {
        let result = check_resume(
            "https://example.com/file.mp4",
            &Path::new("/nonexistent/path/file.mp4"),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_record_status_transitions() {
        let mut record = DownloadRecord::new(
            "https://example.com/file.mp4".into(),
            PathBuf::from("/tmp/file.mp4"),
            "VideoMp4".into(),
        );

        assert!(record.is_resumable());
        assert!(!record.is_complete());
        assert!(!record.is_failed());

        record.update_progress(1024, Some(4096), 512.0);
        assert!(record.is_resumable());

        record.mark_complete(4096);
        assert!(record.is_complete());
        assert!(!record.is_resumable());

        let mut record2 = DownloadRecord::new(
            "https://example.com/fail.mp4".into(),
            PathBuf::from("/tmp/fail.mp4"),
            "VideoMp4".into(),
        );
        record2.mark_failed("network error".into());
        assert!(record2.is_failed());
    }
}
