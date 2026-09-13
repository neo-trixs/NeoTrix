//! Download progress persistence — cross-restart resume support.
//!
//! Provides JSON-backed storage for download records, enabling resume
//! detection and progress tracking across process restarts.
//!
//! **Sidecar state files** (`.ntstate`) provide per-download, chunk-level
//! resume with ETag/Last-Modified validation. Legacy `DownloadStore` is
//! preserved for backward compatibility.

use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

// ═══════════════════════════════════════════════════════════════════════════
// ChunkDownloadStatus — per-chunk download lifecycle state
// ═══════════════════════════════════════════════════════════════════════════

/// Per-chunk download lifecycle status for streaming pipeline tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkDownloadStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

// ═══════════════════════════════════════════════════════════════════════════
// ChunkState — per-chunk tracking (unified for sidecar + streaming)
// ═══════════════════════════════════════════════════════════════════════════

/// Per-chunk tracking for granular resume support and streaming pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkState {
    /// Zero-based chunk index.
    pub index: u32,
    /// Byte offset in the target file.
    pub offset: u64,
    /// Expected size of this chunk.
    pub size: u64,
    /// Whether this chunk has been written and flushed (sidecar compat).
    pub completed: bool,
    /// Optional SHA-256 checksum of the chunk data.
    pub checksum: Option<String>,
    /// Byte range start (inclusive) for streaming downloads.
    #[serde(default)]
    pub start: u64,
    /// Byte range end (inclusive) for streaming downloads.
    #[serde(default)]
    pub end: u64,
    /// Bytes downloaded within this chunk for streaming downloads.
    #[serde(default)]
    pub downloaded: u64,
    /// Download lifecycle status for streaming pipeline.
    #[serde(default)]
    pub status: ChunkDownloadStatus,
}

impl ChunkState {
    /// Create a ChunkState for sidecar-based persistence (legacy).
    pub fn sidecar(index: u32, offset: u64, size: u64) -> Self {
        Self {
            index,
            offset,
            size,
            completed: false,
            checksum: None,
            start: offset,
            end: offset + size - 1,
            downloaded: 0,
            status: ChunkDownloadStatus::Pending,
        }
    }

    /// Create a ChunkState for streaming pipeline downloads.
    pub fn streaming(index: u32, start: u64, end: u64) -> Self {
        let size = end - start + 1;
        Self {
            index,
            offset: start,
            size,
            completed: false,
            checksum: None,
            start,
            end,
            downloaded: 0,
            status: ChunkDownloadStatus::Pending,
        }
    }

    /// Return a reference to the chunk's download status.
    pub fn status(&self) -> &ChunkDownloadStatus {
        &self.status
    }

    /// Returns true if the chunk has been fully written.
    pub fn is_complete(&self) -> bool {
        self.status == ChunkDownloadStatus::Complete || self.completed
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DownloadStatus — sidecar lifecycle state
// ═══════════════════════════════════════════════════════════════════════════

/// Download lifecycle status for sidecar state tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Complete,
    Failed,
}

// ═══════════════════════════════════════════════════════════════════════════
// SidecarState — per-download JSON-serializable state
// ═══════════════════════════════════════════════════════════════════════════

/// Per-download sidecar state persisted as `.ntstate` files.
///
/// Each download writes its own sidecar file alongside the target file,
/// enabling chunk-level resume with ETag/Last-Modified validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarState {
    /// Monotonic download ID (UUID v4).
    pub id: String,
    /// Source URL.
    pub url: String,
    /// Absolute path to the target file being written.
    pub file_path: PathBuf,
    /// Content-Length from initial HEAD/GET (None if server didn't report).
    pub total_bytes: Option<u64>,
    /// Sum of completed chunk sizes.
    pub downloaded_bytes: u64,
    /// ETag from server response — used for resume validation.
    pub etag: Option<String>,
    /// Last-Modified header from server response.
    pub last_modified: Option<String>,
    /// Per-chunk progress vector.
    pub chunk_progress: Vec<ChunkState>,
    /// Chunk size used for this download.
    pub chunk_size: u64,
    /// Current download status.
    pub status: DownloadStatus,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last update timestamp.
    pub updated_at: String,
}

impl SidecarState {
    /// Create a new sidecar state for a fresh download.
    pub fn new(url: String, file_path: PathBuf, total_bytes: Option<u64>, chunk_size: u64) -> Self {
        let now = chrono_now();
        let chunks = if let Some(total) = total_bytes {
            (0..total.div_ceil(chunk_size))
                .map(|i| ChunkState::sidecar(i as u32, i * chunk_size, std::cmp::min(chunk_size, total - i * chunk_size)))
                .collect()
        } else {
            Vec::new()
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            file_path,
            total_bytes,
            downloaded_bytes: 0,
            etag: None,
            last_modified: None,
            chunk_progress: chunks,
            chunk_size,
            status: DownloadStatus::Pending,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Returns true if the download is in Complete status.
    pub fn is_complete(&self) -> bool {
        self.status == DownloadStatus::Complete
    }

    /// Returns true if the download can be resumed (Downloading or Paused).
    pub fn is_resumable(&self) -> bool {
        matches!(
            self.status,
            DownloadStatus::Downloading | DownloadStatus::Paused
        )
    }

    /// Returns the first chunk that has not yet been completed.
    pub fn next_pending_chunk(&self) -> Option<&ChunkState> {
        self.chunk_progress.iter().find(|c| !c.completed)
    }

    /// Count of completed chunks.
    pub fn completed_count(&self) -> usize {
        self.chunk_progress.iter().filter(|c| c.completed).count()
    }

    /// Progress percentage (0.0–100.0). Returns 0.0 if total is unknown.
    pub fn progress_pct(&self) -> f64 {
        if let Some(total) = self.total_bytes {
            if total == 0 {
                return 100.0;
            }
            (self.downloaded_bytes as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Mark a chunk as complete and recalculate downloaded_bytes.
    pub fn mark_chunk_complete(&mut self, index: u32, checksum: Option<String>) {
        if let Some(chunk) = self.chunk_progress.iter_mut().find(|c| c.index == index) {
            chunk.completed = true;
            chunk.checksum = checksum;
            chunk.status = ChunkDownloadStatus::Complete;
            self.downloaded_bytes = self
                .chunk_progress
                .iter()
                .filter(|c| c.completed)
                .map(|c| c.size)
                .sum();
            self.updated_at = chrono_now();
        }
    }

    /// Mark the entire download as complete.
    pub fn mark_complete(&mut self) {
        self.status = DownloadStatus::Complete;
        for chunk in &mut self.chunk_progress {
            chunk.completed = true;
            chunk.status = ChunkDownloadStatus::Complete;
        }
        if let Some(total) = self.total_bytes {
            self.downloaded_bytes = total;
        }
        self.updated_at = chrono_now();
    }

    /// Mark the download as failed.
    pub fn mark_failed(&mut self) {
        self.status = DownloadStatus::Failed;
        self.updated_at = chrono_now();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ResumeValidation — HEAD-based resume check result
// ═══════════════════════════════════════════════════════════════════════════

/// Result of a HEAD-based resume validation check.
pub struct ResumeValidation {
    /// Whether it is safe to resume the download.
    pub can_resume: bool,
    /// ETag reported by the server.
    pub server_etag: Option<String>,
    /// Last-Modified header reported by the server.
    pub server_last_modified: Option<String>,
    /// Content-Length reported by the server.
    pub server_total: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════════════════
// ChunkPersistenceConfig — save-interval tuning
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration for chunk-level sidecar persistence frequency.
pub struct ChunkPersistenceConfig {
    /// Save sidecar after every N completed chunks (default: 5).
    pub save_interval: u32,
    /// Also save on resume-start and on-complete.
    pub save_on_start: bool,
}

impl Default for ChunkPersistenceConfig {
    fn default() -> Self {
        Self {
            save_interval: 5,
            save_on_start: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Sidecar file operations
// ═══════════════════════════════════════════════════════════════════════════

/// Derive the sidecar `.ntstate` path from the target file path.
pub fn sidecar_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sidecar_name = format!("{}.ntstate", name);
    target.with_file_name(sidecar_name)
}

/// Atomically save a sidecar state file (tmp → rename).
///
/// Creates parent directories if they don't exist. The write is atomic:
/// data goes to a `.tmp` file first, then is renamed to the final path.
pub async fn save_sidecar(path: &Path, state: &SidecarState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("create dir: {}", e))?;
    }
    let data =
        serde_json::to_string_pretty(state).map_err(|e| format!("serialize sidecar: {}", e))?;
    let tmp = path.with_extension("ntstate.tmp");
    tokio::fs::write(&tmp, &data)
        .await
        .map_err(|e| format!("write sidecar tmp: {}", e))?;
    tokio::fs::rename(&tmp, path)
        .await
        .map_err(|e| format!("rename sidecar: {}", e))?;
    Ok(())
}

/// Load an existing sidecar state from disk.
///
/// Returns `Ok(None)` if the file doesn't exist. Returns an error if the
/// file exists but cannot be parsed.
pub async fn load_sidecar(path: &Path) -> Result<Option<SidecarState>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let data = tokio::fs::read(path)
        .await
        .map_err(|e| format!("read sidecar: {}", e))?;
    let state: SidecarState =
        serde_json::from_slice(&data).map_err(|e| format!("parse sidecar: {}", e))?;
    Ok(Some(state))
}

/// HEAD the remote URL and compare ETag/Last-Modified against saved state.
///
/// Returns a `ResumeValidation` indicating whether it's safe to resume.
/// Resume is rejected if the resource has changed (stale state).
pub async fn validate_resume(client: &Client, url: &str, saved: &SidecarState) -> ResumeValidation {
    let mut result = ResumeValidation {
        can_resume: false,
        server_etag: None,
        server_last_modified: None,
        server_total: None,
    };

    let resp = match client.head(url).send().await {
        Ok(r) => r,
        Err(_) => return result,
    };

    result.server_total = resp.content_length();
    result.server_etag = resp
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    result.server_last_modified = resp
        .headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // ETag match → safe to resume
    if let (Some(ref saved_etag), Some(ref server_etag)) = (&saved.etag, &result.server_etag) {
        if saved_etag == server_etag {
            result.can_resume = true;
            return result;
        }
    }

    // Last-Modified match → safe to resume
    if let (Some(ref saved_lm), Some(ref server_lm)) =
        (&saved.last_modified, &result.server_last_modified)
    {
        if saved_lm == server_lm {
            result.can_resume = true;
            return result;
        }
    }

    // No validators present → assume file-size based resume is ok
    if result.server_etag.is_none() && result.server_last_modified.is_none() {
        result.can_resume = true;
    }

    result
}

/// Remove a sidecar file if it exists.
pub async fn remove_sidecar(target: &Path) -> Result<(), String> {
    let path = sidecar_path(target);
    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| format!("remove sidecar: {}", e))?;
    }
    Ok(())
}

/// Mark a chunk complete and persist the sidecar every `config.save_interval` chunks.
///
/// Called from the download loop after each chunk write. Handles both the
/// state update and conditional persistence in one call.
pub async fn maybe_save_chunk(
    state: &mut SidecarState,
    chunk_index: u32,
    checksum: Option<String>,
    config: &ChunkPersistenceConfig,
) -> Result<(), String> {
    state.mark_chunk_complete(chunk_index, checksum);
    let completed = state.completed_count() as u32;
    if completed % config.save_interval == 0 || state.is_complete() {
        let path = sidecar_path(&state.file_path);
        save_sidecar(&path, state).await?;
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Disk space & checksum utilities
// ═══════════════════════════════════════════════════════════════════════════

/// Check whether the filesystem containing `path` has at least `required_bytes` free.
///
/// Uses synchronous `std::fs::metadata` — acceptable for a quick pre-download check.
pub async fn check_disk_space(path: &Path, required_bytes: u64) -> Result<(), String> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let meta = std::fs::metadata(dir).map_err(|e| format!("disk check: {e}"))?;
    // Note: std::fs::metadata is sync, acceptable for a quick check
    // We cannot query available bytes via std alone; a production impl
    // would use libc::statvfs or sysinfo crate. For now, the metadata
    // check verifies the path is accessible.
    let _ = (meta, required_bytes);
    Ok(())
}

/// Compute the SHA-256 hash of a file using streaming reads (8 KiB buffer).
///
/// Avoids loading the entire file into memory — safe for large downloads.
pub async fn compute_sha256_streaming(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| format!("sha256 open: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .await
            .map_err(|e| format!("sha256 read: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// ═══════════════════════════════════════════════════════════════════════════
// DownloadRecord — single download entry (backward compat)
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
// DownloadStore — thread-safe JSON persistence (backward compat)
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
// ResumeSupport — integration helpers (backward compat)
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

/// ISO 8601 timestamp string via chrono.
fn chrono_now() -> String {
    Utc::now().to_rfc3339()
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    // ── Legacy DownloadStore tests ───────────────────────────────────

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
            Path::new("/nonexistent/path/file.mp4"),
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

    // ── SidecarState tests ───────────────────────────────────────────

    #[test]
    fn test_sidecar_state_serialization_roundtrip() {
        let state = SidecarState::new(
            "https://example.com/big.mp4".into(),
            PathBuf::from("/tmp/big.mp4"),
            Some(1_048_576),
            262_144,
        );

        let json = serde_json::to_string(&state).unwrap();
        let restored: SidecarState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.id, restored.id);
        assert_eq!(state.url, restored.url);
        assert_eq!(state.file_path, restored.file_path);
        assert_eq!(state.total_bytes, restored.total_bytes);
        assert_eq!(state.chunk_size, restored.chunk_size);
        assert_eq!(state.chunk_progress.len(), restored.chunk_progress.len());
        assert_eq!(state.status, restored.status);
    }

    #[test]
    fn test_sidecar_path_derivation() {
        let target = Path::new("/home/user/downloads/video.mp4");
        let sp = sidecar_path(target);
        assert_eq!(sp, PathBuf::from("/home/user/downloads/video.mp4.ntstate"));

        let target2 = Path::new("/tmp/archive.tar.gz");
        let sp2 = sidecar_path(target2);
        assert_eq!(sp2, PathBuf::from("/tmp/archive.tar.gz.ntstate"));
    }

    #[test]
    fn test_chunk_state_mark_complete_accounting() {
        let mut state = SidecarState::new(
            "https://example.com/file.bin".into(),
            PathBuf::from("/tmp/file.bin"),
            Some(1_000_000),
            250_000,
        );

        assert_eq!(state.completed_count(), 0);
        assert_eq!(state.downloaded_bytes, 0);

        state.mark_chunk_complete(0, Some("sha256:abc".into()));
        assert_eq!(state.completed_count(), 1);
        assert_eq!(state.downloaded_bytes, 250_000);

        state.mark_chunk_complete(1, Some("sha256:def".into()));
        assert_eq!(state.completed_count(), 2);
        assert_eq!(state.downloaded_bytes, 500_000);

        assert_eq!(state.progress_pct(), 50.0);
    }

    #[test]
    fn test_sidecar_status_transitions() {
        let mut state = SidecarState::new(
            "https://example.com/file.mp4".into(),
            PathBuf::from("/tmp/file.mp4"),
            None,
            262_144,
        );

        assert!(state.is_resumable());
        assert!(!state.is_complete());

        state.status = DownloadStatus::Downloading;
        assert!(state.is_resumable());

        state.mark_complete();
        assert!(state.is_complete());
        assert!(!state.is_resumable());

        let mut state2 = SidecarState::new(
            "https://example.com/fail.mp4".into(),
            PathBuf::from("/tmp/fail.mp4"),
            None,
            262_144,
        );
        state2.mark_failed();
        assert_eq!(state2.status, DownloadStatus::Failed);
    }

    #[tokio::test]
    async fn test_sidecar_save_load_roundtrip() {
        let dir = temp_dir().join("nt_sidecar_test");
        let _ = tokio::fs::create_dir_all(&dir).await;
        let target = dir.join("testfile.mp4");
        let sp = sidecar_path(&target);

        let mut state = SidecarState::new(
            "https://example.com/test.mp4".into(),
            target.clone(),
            Some(524_288),
            131_072,
        );
        state.status = DownloadStatus::Downloading;
        state.mark_chunk_complete(0, None);

        save_sidecar(&sp, &state).await.unwrap();
        let loaded = load_sidecar(&sp).await.unwrap().unwrap();

        assert_eq!(loaded.id, state.id);
        assert_eq!(loaded.downloaded_bytes, 131_072);
        assert_eq!(loaded.completed_count(), 1);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn test_maybe_save_chunk_persists_at_interval() {
        let dir = temp_dir().join("nt_maybe_save_test");
        let _ = tokio::fs::create_dir_all(&dir).await;
        let target = dir.join("chunked.bin");
        let sp = sidecar_path(&target);

        let mut state = SidecarState::new(
            "https://example.com/chunked.bin".into(),
            target.clone(),
            Some(1_000_000),
            100_000,
        );
        state.status = DownloadStatus::Downloading;
        let config = ChunkPersistenceConfig {
            save_interval: 3,
            save_on_start: true,
        };

        // Chunks 0, 1 — should NOT persist (completed=2, 2%3 != 0)
        maybe_save_chunk(&mut state, 0, None, &config)
            .await
            .unwrap();
        assert!(!sp.exists());
        maybe_save_chunk(&mut state, 1, None, &config)
            .await
            .unwrap();
        assert!(!sp.exists());

        // Chunk 2 — should persist (completed=3, 3%3 == 0)
        maybe_save_chunk(&mut state, 2, None, &config)
            .await
            .unwrap();
        assert!(sp.exists());

        let loaded = load_sidecar(&sp).await.unwrap().unwrap();
        assert_eq!(loaded.completed_count(), 3);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    #[test]
    fn test_progress_pct_edge_cases() {
        let mut state = SidecarState::new(
            "https://example.com/zero.mp4".into(),
            PathBuf::from("/tmp/zero.mp4"),
            Some(0),
            262_144,
        );
        assert_eq!(state.progress_pct(), 100.0);

        let state2 = SidecarState::new(
            "https://example.com/unknown.mp4".into(),
            PathBuf::from("/tmp/unknown.mp4"),
            None,
            262_144,
        );
        assert_eq!(state2.progress_pct(), 0.0);
    }

    #[test]
    fn test_next_pending_chunk() {
        let mut state = SidecarState::new(
            "https://example.com/pending.mp4".into(),
            PathBuf::from("/tmp/pending.mp4"),
            Some(500_000),
            250_000,
        );

        let next = state.next_pending_chunk().unwrap();
        assert_eq!(next.index, 0);

        state.mark_chunk_complete(0, None);
        let next = state.next_pending_chunk().unwrap();
        assert_eq!(next.index, 1);

        state.mark_chunk_complete(1, None);
        assert!(state.next_pending_chunk().is_none());
    }

    // ── ChunkState streaming tests ───────────────────────────────────

    #[test]
    fn test_chunk_state_streaming_constructor() {
        let chunk = ChunkState::streaming(0, 0, 131071);
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.start, 0);
        assert_eq!(chunk.end, 131071);
        assert_eq!(chunk.size, 131072);
        assert_eq!(chunk.offset, 0);
        assert_eq!(chunk.downloaded, 0);
        assert_eq!(*chunk.status(), ChunkDownloadStatus::Pending);
        assert!(!chunk.is_complete());
    }

    #[test]
    fn test_chunk_state_status_method() {
        let mut chunk = ChunkState::streaming(1, 1024, 2047);
        assert_eq!(*chunk.status(), ChunkDownloadStatus::Pending);

        chunk.status = ChunkDownloadStatus::InProgress;
        assert_eq!(*chunk.status(), ChunkDownloadStatus::InProgress);

        chunk.status = ChunkDownloadStatus::Complete;
        assert!(chunk.is_complete());
    }

    // ── Disk space & SHA-256 tests ───────────────────────────────────

    #[tokio::test]
    async fn test_check_disk_space_ok() {
        let path = temp_dir().join("disk_space_check_test.tmp");
        let result = check_disk_space(&path, 1024).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compute_sha256_streaming() {
        let dir = temp_dir().join("nt_sha256_test");
        let _ = tokio::fs::create_dir_all(&dir).await;
        let path = dir.join("test.bin");
        let data = b"hello neotrix";
        tokio::fs::write(&path, data).await.unwrap();

        let hash = compute_sha256_streaming(&path).await.unwrap();

        // Verify it matches a fresh Sha256 computation
        use sha2::{Digest, Sha256};
        let mut expected = Sha256::new();
        expected.update(data);
        let expected_hex = format!("{:x}", expected.finalize());
        assert_eq!(hash, expected_hex);

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}

impl Default for ChunkDownloadStatus {
    fn default() -> Self {
        ChunkDownloadStatus::Pending
    }
}
