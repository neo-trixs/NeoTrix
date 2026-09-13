# Persistence Enhancement — Sidecar State Files + Chunk-Level Resume

## Problem

Current `persistence.rs` stores download records in a centralized `~/.neotrix/downloads.json` with flat byte-level progress tracking. On crash, resume relies only on file size matching — no validation that the remote resource hasn't changed, no chunk granularity, no per-download isolation.

## Design Goals

1. **Per-download state** — each download gets its own `.ntstate` sidecar file alongside the target file
2. **Chunk-level progress** — track which chunks completed with checksums for integrity
3. **Resume validation** — HEAD request checks ETag/Last-Modified before assuming resume is safe
4. **Crash recovery** — persist state every N chunks, not on every byte write

---

## Struct Definitions

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ─── ChunkState ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChunkState {
    /// Zero-based chunk index
    pub index: u32,
    /// Byte offset in the target file
    pub offset: u64,
    /// Expected size of this chunk
    pub size: u64,
    /// Whether this chunk has been written and flushed
    pub completed: bool,
    /// Optional SHA-256 checksum of the chunk data
    pub checksum: Option<String>,
}

// ─── DownloadState ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    /// Monotonic download ID (UUID v4)
    pub id: String,
    /// Source URL
    pub url: String,
    /// Absolute path to the target file being written
    pub file_path: PathBuf,
    /// Content-Length from initial HEAD/GET (None if server didn't report)
    pub total_bytes: Option<u64>,
    /// Sum of completed chunk sizes
    pub downloaded_bytes: u64,
    /// ETag from server response — used for resume validation
    pub etag: Option<String>,
    /// Last-Modified header from server response
    pub last_modified: Option<String>,
    /// Per-chunk progress vector
    pub chunk_progress: Vec<ChunkState>,
    /// Chunk size used for this download
    pub chunk_size: u64,
    /// Current download status
    pub status: DownloadStatus,
    /// ISO 8601 creation timestamp
    pub created_at: String,
    /// ISO 8601 last update timestamp
    pub updated_at: String,
}

impl DownloadState {
    pub fn new(url: String, file_path: PathBuf, total_bytes: Option<u64>, chunk_size: u64) -> Self {
        let now = chrono_now();
        let chunks = if let Some(total) = total_bytes {
            (0..total.div_ceil(chunk_size))
                .map(|i| ChunkState {
                    index: i as u32,
                    offset: i * chunk_size,
                    size: std::cmp::min(chunk_size, total - i * chunk_size),
                    completed: false,
                    checksum: None,
                })
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

    pub fn is_complete(&self) -> bool {
        matches!(self.status, DownloadStatus::Complete)
    }

    pub fn is_resumable(&self) -> bool {
        matches!(self.status, DownloadStatus::Downloading | DownloadStatus::Paused)
    }

    pub fn next_pending_chunk(&self) -> Option<&ChunkState> {
        self.chunk_progress.iter().find(|c| !c.completed)
    }

    pub fn completed_count(&self) -> usize {
        self.chunk_progress.iter().filter(|c| c.completed).count()
    }

    pub fn progress_pct(&self) -> f64 {
        if let Some(total) = self.total_bytes {
            if total == 0 { return 100.0; }
            (self.downloaded_bytes as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn mark_chunk_complete(&mut self, index: u32, checksum: Option<String>) {
        if let Some(chunk) = self.chunk_progress.iter_mut().find(|c| c.index == index) {
            chunk.completed = true;
            chunk.checksum = checksum;
            self.downloaded_bytes = self.chunk_progress.iter()
                .filter(|c| c.completed)
                .map(|c| c.size)
                .sum();
            self.updated_at = chrono_now();
        }
    }

    pub fn mark_complete(&mut self) {
        self.status = DownloadStatus::Complete;
        if let Some(total) = self.total_bytes {
            self.downloaded_bytes = total;
        }
        self.updated_at = chrono_now();
    }

    pub fn mark_failed(&mut self) {
        self.status = DownloadStatus::Failed;
        self.updated_at = chrono_now();
    }
}

// ─── Sidecar file operations ─────────────────────────────────────────

pub fn sidecar_path(target: &std::path::Path) -> PathBuf {
    let mut name = target.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str(".ntstate");
    target.with_file_name(name)
}

pub async fn load_state(target: &std::path::Path) -> Result<Option<DownloadState>, String> {
    let path = sidecar_path(target);
    if !path.exists() {
        return Ok(None);
    }
    let data = tokio::fs::read(&path).await
        .map_err(|e| format!("read sidecar: {}", e))?;
    let state: DownloadState = serde_json::from_slice(&data)
        .map_err(|e| format!("parse sidecar: {}", e))?;
    Ok(Some(state))
}

pub async fn save_state(state: &DownloadState) -> Result<(), String> {
    let path = sidecar_path(&state.file_path);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| format!("create dir: {}", e))?;
    }
    let data = serde_json::to_string_pretty(state)
        .map_err(|e| format!("serialize state: {}", e))?;
    // Atomic write: write to temp then rename
    let tmp = path.with_extension("ntstate.tmp");
    tokio::fs::write(&tmp, data).await
        .map_err(|e| format!("write sidecar: {}", e))?;
    tokio::fs::rename(&tmp, &path).await
        .map_err(|e| format!("rename sidecar: {}", e))?;
    Ok(())
}

pub async fn remove_state(target: &std::path::Path) -> Result<(), String> {
    let path = sidecar_path(target);
    if path.exists() {
        tokio::fs::remove_file(&path).await
            .map_err(|e| format!("remove sidecar: {}", e))?;
    }
    Ok(())
}
```

---

## Resume Validation

```rust
use reqwest::Client;

pub struct ResumeValidation {
    pub can_resume: bool,
    pub server_etag: Option<String>,
    pub server_last_modified: Option<String>,
    pub server_total: Option<u64>,
}

/// HEAD the remote URL and compare ETag/Last-Modified against saved state.
/// Returns false if the resource has changed (state is stale).
pub async fn validate_resume(
    client: &Client,
    url: &str,
    saved: &DownloadState,
) -> ResumeValidation {
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
    result.server_etag = resp.headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    result.server_last_modified = resp.headers()
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
    if let (Some(ref saved_lm), Some(ref server_lm)) = (&saved.last_modified, &result.server_last_modified) {
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
```

---

## Crash-Safe Persistence (Save Every N Chunks)

```rust
pub struct ChunkPersistenceConfig {
    /// Save sidecar after every N completed chunks (default: 5)
    pub save_interval: u32,
    /// Also save on resume-start and on-complete
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

/// Called from the download loop after each chunk write.
/// Persists sidecar every `config.save_interval` chunks.
pub async fn maybe_save_chunk(
    state: &mut DownloadState,
    chunk_index: u32,
    checksum: Option<String>,
    config: &ChunkPersistenceConfig,
) -> Result<(), String> {
    state.mark_chunk_complete(chunk_index, checksum);
    let completed = state.completed_count() as u32;
    if completed % config.save_interval == 0 || state.is_complete() {
        save_state(state).await?;
    }
    Ok(())
}
```

---

## JSON Schema Example

File: `video_movie.mp4.ntstate`

```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "url": "https://cdn.example.com/movie.mp4",
  "file_path": "/home/user/downloads/video_movie.mp4",
  "total_bytes": 1073741824,
  "downloaded_bytes": 335544320,
  "etag": "\"3a7b-abc123\"",
  "last_modified": "Sat, 12 Sep 2026 10:30:00 GMT",
  "chunk_progress": [
    { "index": 0, "offset": 0, "size": 262144, "completed": true, "checksum": "sha256:a1b2c3..." },
    { "index": 1, "offset": 262144, "size": 262144, "completed": true, "checksum": "sha256:d4e5f6..." },
    { "index": 2, "offset": 524288, "size": 262144, "completed": true, "checksum": "sha256:789abc..." },
    { "index": 3, "offset": 786432, "size": 262144, "completed": false, "checksum": null },
    { "index": 4, "offset": 1048576, "size": 262144, "completed": false, "checksum": null }
  ],
  "chunk_size": 262144,
  "status": "Downloading",
  "created_at": "2026-09-12T10:30:00Z",
  "updated_at": "2026-09-12T10:35:12Z"
}
```

---

## Integration with `streaming.rs` (Caller Patterns)

### Modified `stream_http_download`

```rust
async fn stream_http_download(
    client: &reqwest::Client,
    url: &str,
    output: &Path,
    chunk_size: usize,
    timeout: Duration,
    bytes_written: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    progress_tx: mpsc::Sender<PipelineProgress>,
    media_kind: MediaKind,
    auth: Option<&AuthConfig>,
    persistence: Option<Arc<super::persistence::DownloadStore>>,
) -> Result<(), PipelineError> {
    // ── Load or create sidecar state ────────────────────────────────
    let sidecar_config = ChunkPersistenceConfig::default();
    let mut state = match persistence::load_state(output).await {
        Ok(Some(s)) if s.is_resumable() => {
            // Validate resume against server
            let validation = persistence::validate_resume(client, url, &s).await;
            if validation.can_resume {
                // Restore state
                bytes_written.store(s.downloaded_bytes, Ordering::Relaxed);
                s
            } else {
                // Stale — start fresh
                persistence::remove_state(output).await.ok();
                persistence::DownloadState::new(
                    url.to_string(),
                    output.to_path_buf(),
                    validation.server_total,
                    chunk_size as u64,
                )
            }
        }
        _ => {
            let validation = persistence::validate_resume(
                client, url,
                &persistence::DownloadState::new(
                    url.to_string(), output.to_path_buf(), None, chunk_size as u64
                ),
            ).await;
            let mut s = persistence::DownloadState::new(
                url.to_string(), output.to_path_buf(),
                validation.server_total, chunk_size as u64,
            );
            s.etag = validation.server_etag;
            s.last_modified = validation.server_last_modified;
            s.status = persistence::DownloadStatus::Downloading;
            s
        }
    };

    // ── Build request with Range header from next pending chunk ─────
    let start_byte = if let Some(next_chunk) = state.next_pending_chunk() {
        next_chunk.offset
    } else if output.exists() {
        fs::metadata(output).await.map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let req = client
        .get(url)
        .header("Range", format!("bytes={}-", start_byte))
        .header("Accept-Encoding", "identity")
        .timeout(timeout);

    // ... (auth handling unchanged) ...

    let resp = req.send().await
        .map_err(|e| PipelineError::Network(e.to_string()))?;

    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(PipelineError::Http(resp.status().as_u16()));
    }

    let total_size = resp.content_length().map(|cl| cl + start_byte);
    state.total_bytes = total_size;

    // If server returned new total and we don't have chunk_progress yet, rebuild
    if state.chunk_progress.is_empty() && total_size.is_some() {
        state = persistence::DownloadState::new(
            url.to_string(), output.to_path_buf(),
            total_size, chunk_size as u64,
        );
        state.etag = resp.headers().get("etag")
            .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        state.last_modified = resp.headers().get("last-modified")
            .and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        state.status = persistence::DownloadStatus::Downloading;
    }

    // Save initial state
    persistence::save_state(&state).await.ok();

    let mut stream = resp.bytes_stream();
    let file = if start_byte > 0 {
        fs::OpenOptions::new().append(true).open(output).await
    } else {
        File::create(output).await
    }.map_err(|e| PipelineError::Io(e.to_string()))?;

    let mut writer = BufWriter::with_capacity(chunk_size, file);
    bytes_written.store(start_byte, Ordering::Relaxed);

    let mut current_chunk_index = state.next_pending_chunk()
        .map(|c| c.index)
        .unwrap_or(0);
    let mut chunk_bytes_received: u64 = 0;

    // ... existing speed sampling vars ...

    while let Some(chunk) = stream.next().await {
        // cancel check unchanged
        match chunk {
            Ok(data) => {
                writer.write_all(&data).await
                    .map_err(|e| PipelineError::Io(e.to_string()))?;
                let written = bytes_written.fetch_add(data.len() as u64, Ordering::Relaxed)
                    + data.len() as u64;
                chunk_bytes_received += data.len() as u64;

                // Check if current chunk completed
                if let Some(chunk_def) = state.chunk_progress.iter()
                    .find(|c| c.index == current_chunk_index)
                {
                    if chunk_bytes_received >= chunk_def.size {
                        persistence::maybe_save_chunk(
                            &mut state,
                            current_chunk_index,
                            None, // checksum optional for speed
                            &sidecar_config,
                        ).await.ok();
                        current_chunk_index += 1;
                        chunk_bytes_received = 0;
                    }
                }

                // ... existing progress reporting (uses state.downloaded_bytes) ...
            }
            Err(e) => {
                // Save state on failure so we can resume
                state.status = persistence::DownloadStatus::Paused;
                persistence::save_state(&state).await.ok();
                return Err(PipelineError::Network(e.to_string()));
            }
        }
    }

    state.mark_complete();
    persistence::save_state(&state).await.ok();
    // Optionally remove sidecar on success:
    // persistence::remove_state(output).await.ok();

    Ok(())
}
```

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Sidecar file, not central DB** | Per-download isolation — no lock contention, delete download = delete sidecar |
| **Atomic write (tmp→rename)** | Prevents corrupt sidecar on crash during write |
| **Save every N chunks, not every byte** | Reduces fsync overhead; 5-chunk interval = ~1.2 MB persistence cost at 256KB chunks |
| **HEAD validation on resume** | Detects content drift (server replaced file) before wasting bandwidth |
| **Etag/Last-Modified fallback chain** | ETag is strongest, Last-Modified next, file-size-only as last resort |
| **chunk_progress Vec rebuilt from total** | Handles unknown total gracefully — chunks added dynamically as size is discovered |
| **Backward compat** | `DownloadRecord` + `DownloadStore` preserved; sidecar is additive |

## Migration Path

1. New code writes both sidecar `.ntstate` AND legacy `downloads.json` during transition
2. On resume, prefer sidecar if present; fall back to legacy `DownloadStore::get_by_url()`
3. After one release cycle, deprecate `DownloadStore` central file

## Testing Strategy

- **Unit**: `ChunkState` serialization roundtrip, `sidecar_path()` derivation, `mark_chunk_complete()` accounting
- **Integration**: Mock server returns ETag → save → HEAD → resume validates → continues from correct offset
- **Crash sim**: Kill process mid-download → restart → validate sidecar loaded → resume works
- **Stale check**: Server changes ETag between sessions → resume rejected → fresh download starts
