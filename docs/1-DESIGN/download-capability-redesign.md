# Download Capability Redesign

> **R-P42**: Strengthen existing nodes, no new parallel modules
> **R-P48**: Zero third-party binary dependency (reqwest + std only)
> **R-P79**: Wire to production in same session
> **R-P55**: No stubs, all signatures must match production callers

## Current State

| File | Lines | Role |
|------|-------|------|
| `streaming.rs` | 1071 | Single-stream sequential download (reqwest bytes_stream) |
| `persistence.rs` | 431 | JSON-backed `DownloadStore` with `DownloadRecord` |
| `download_progress.rs` | 169 | Terminal progress bar (`DownloadProgress`) |
| `router.rs` | 194 | URL→TransportType routing |

**Current limitations**:
1. Single sequential stream — no parallelism
2. Resume only via file size (`check_resume`) — no sidecar state
3. No stall detection — hangs forever on slow connections
4. No retry logic — single failure = terminal error
5. No integrity verification — corrupted downloads pass silently
6. Progress is channel-per-pipeline, no broadcast pattern

## Architecture Overview

All 7 enhancements are **in-place modifications** to existing structs/functions. No new files or modules.

```
streaming.rs (modified)
├── ParallelDownloader (new struct, ~180 lines)
├── SidecarState (new struct, ~80 lines, in persistence.rs)
├── StallDetector (new struct, ~40 lines)
├── RetryPolicy (new struct, ~30 lines)
├── IntegrityVerifier (new fn, ~40 lines)
└── stream_http_download (modified: parallel+stall+retry+verify)

persistence.rs (modified)
├── SidecarState (new struct, ~80 lines)
└── DownloadRecord (extended: 3 new fields)

download_progress.rs (modified)
└── DownloadProgress (extended: broadcast channel support)

router.rs (modified)
└── MediaRoute (extended: supports_range flag)
```

---

## Enhancement 1: Parallel Chunk Download

**Target**: `streaming.rs` — replace single-stream `stream_http_download` with work-stealing parallel download.

### 1.1 New types in `streaming.rs`

```rust
/// Per-chunk state tracked by the coordinator
#[derive(Debug, Clone)]
struct ChunkState {
    index: usize,
    start: u64,
    end: u64,
    downloaded: u64, // bytes downloaded for this chunk so far
    status: ChunkStatus,
}

#[derive(Debug, Clone, PartialEq)]
enum ChunkStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

/// Coordinator for parallel chunk downloads — work-stealing pattern.
/// Each chunk is an independent reqwest Range request. When a worker
/// finishes its chunk, it steals the next Pending chunk from the queue.
struct ParallelDownloader {
    client: reqwest::Client,
    url: String,
    output: PathBuf,
    total_size: u64,
    chunk_size: usize,
    concurrency: usize,
    chunks: Arc<Mutex<Vec<ChunkState>>>,
    bytes_written: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    stall_timeout: Duration,
    max_retries: u32,
    auth: Option<AuthConfig>,
}
```

### 1.2 Key method signatures

```rust
impl ParallelDownloader {
    /// Probe server for Range support and total size.
    /// Returns (total_size, supports_range).
    async fn probe(
        client: &reqwest::Client,
        url: &str,
        auth: Option<&AuthConfig>,
    ) -> Result<(u64, bool), PipelineError>;

    /// Split total_size into chunk_size-aligned ranges.
    fn plan_chunks(total_size: u64, chunk_size: usize) -> Vec<ChunkState>;

    /// Download a single chunk with stall detection + retry.
    /// Writes directly to output file at the correct offset.
    async fn download_chunk(
        &self,
        chunk_idx: usize,
        file: &File,
    ) -> Result<(), PipelineError>;

    /// Work-stealing loop: assign chunks to workers.
    /// Returns when all chunks are Complete or cancel is set.
    async fn run(
        &self,
        progress_tx: mpsc::Sender<PipelineProgress>,
        media_kind: MediaKind,
    ) -> Result<(), PipelineError>;

    /// Steal next Pending chunk index. Returns None if all done.
    fn steal_next(&self) -> Option<usize>;
}
```

### 1.3 Chunk strategy

```
Total: 100MB, Chunk: 4MB, Concurrency: 4

Chunk 0: [0, 4MB)     ← Worker 1
Chunk 1: [4MB, 8MB)    ← Worker 2
Chunk 2: [8MB, 12MB)   ← Worker 3
Chunk 3: [12MB, 16MB)  ← Worker 4
...last chunk may be smaller
```

Work-stealing: when Worker 1 finishes Chunk 0, it calls `steal_next()` and gets Chunk 4 (if exists). This ensures even if one chunk is slow, other workers keep busy.

### 1.4 File write strategy

Each chunk writes to a pre-allocated region of the output file:
```rust
// Pre-allocate file to total_size
file.set_len(total_size).await?;

// Each chunk writes at its offset — no lock needed on the file
file.seek(SeekFrom::Start(chunk.start + chunk.downloaded)).await?;
file.write_all(&data).await?;
```

### 1.5 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `streaming.rs` (new) | ~180 | `ChunkState`, `ChunkStatus`, `ParallelDownloader` struct + impl |
| `streaming.rs` (modify `stream_http_download`) | ~20 | Dispatch: if `supports_range && total_size > chunk_size * 2` → `ParallelDownloader::run()`, else fall back to existing single-stream |

**Callers unchanged**: `media_cmds.rs:146` calls `pipeline.run(progress_tx)` which internally dispatches. `offline_download.rs:41` creates `StreamingPipeline::new(PipelineConfig {...})` — no API change.

---

## Enhancement 2: Sidecar State File for Resume

**Target**: `persistence.rs` — new `SidecarState` struct alongside existing `DownloadStore`.

### 2.1 New struct in `persistence.rs`

```rust
/// Sidecar state for parallel download resume.
/// Written as JSON to `<output>.nt_state.json`.
/// Lightweight: only chunk-level progress, no byte-level tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarState {
    pub url: String,
    pub output: PathBuf,
    pub total_size: u64,
    pub chunk_size: usize,
    pub expected_sha256: Option<String>,
    pub chunks: Vec<ChunkResume>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResume {
    pub index: usize,
    pub start: u64,
    pub end: u64,
    pub status: String, // "pending" | "complete" | "failed"
}

impl SidecarState {
    /// Path: output.with_extension("nt_state.json")
    pub fn path_for(output: &Path) -> PathBuf;

    /// Load from disk. Returns None if file doesn't exist or is corrupt.
    pub async fn load(output: &Path) -> Option<Self>;

    /// Save to disk (atomic write via rename).
    pub async fn save(&self) -> Result<(), String>;

    /// Check if a specific chunk is already complete.
    pub fn is_chunk_complete(&self, index: usize) -> bool;

    /// Count completed chunks for progress reporting.
    pub fn completed_chunks(&self) -> usize;

    /// Mark chunk complete and persist.
    pub async fn mark_chunk_complete(&mut self, index: usize) -> Result<(), String>;

    /// Convert to ParallelDownloader's ChunkState vec.
    pub fn to_chunk_states(&self) -> Vec<ChunkState>;
}
```

### 2.2 Resume flow

```
stream_http_download
  ├── load SidecarState from <output>.nt_state.json
  ├── if state exists && total_size matches:
  │     ├── restore ParallelDownloader with completed chunks skipped
  │     └── log "Resuming from X/Y chunks"
  ├── else:
  │     ├── create fresh SidecarState
  │     └── log "Starting new download"
  ├── run parallel download
  └── on completion: delete sidecar file
```

### 2.3 Extend `DownloadRecord` with 3 fields

```rust
pub struct DownloadRecord {
    // ... existing fields ...
    pub total_chunks: Option<usize>,      // NEW: parallel chunk count
    pub completed_chunks: Option<usize>,  // NEW: chunks done
    pub expected_sha256: Option<String>,  // NEW: for integrity check
}
```

### 2.4 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `persistence.rs` (new) | ~80 | `SidecarState`, `ChunkResume` + impl |
| `persistence.rs` (modify) | ~5 | Add 3 fields to `DownloadRecord` |

---

## Enhancement 3: Stall Detection

**Target**: `streaming.rs` — `StallDetector` used inside `download_chunk`.

### 3.1 New struct in `streaming.rs`

```rust
/// Monitors download speed per-chunk. If no bytes received
/// for `timeout` (default 30s), returns Err(StallDetected).
/// Pattern: Ollama's download stall detection.
struct StallDetector {
    timeout: Duration,
    last_progress: Instant,
    last_bytes: u64,
}

impl StallDetector {
    fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            last_progress: Instant::now(),
            last_bytes: 0,
        }
    }

    /// Call after each chunk of bytes received.
    fn record_progress(&mut self, bytes_received: u64) {
        self.last_bytes += bytes_received;
        self.last_progress = Instant::now();
    }

    /// Check if download has stalled. Call periodically (e.g., every 1s).
    fn is_stalled(&self) -> bool {
        self.last_progress.elapsed() > self.timeout
    }

    /// Reset after a successful retry reconnect.
    fn reset(&mut self) {
        self.last_progress = Instant::now();
    }
}
```

### 3.2 Integration point

Inside `ParallelDownloader::download_chunk`, wrap the chunk download:

```rust
async fn download_chunk(&self, chunk_idx: usize, file: &File) -> Result<(), PipelineError> {
    let mut stall = StallDetector::new(self.stall_timeout);
    let mut attempt = 0u32;

    loop {
        // ... build Range request for chunk ...
        let resp = client.get(&self.url)
            .header("Range", format!("bytes={}-{}", chunk.start, chunk.end))
            .send()
            .await;

        match resp {
            Ok(r) => {
                let mut stream = r.bytes_stream();
                while let Some(chunk_data) = stream.next().await {
                    if self.cancel.load(Ordering::Relaxed) {
                        return Ok(());
                    }
                    match chunk_data {
                        Ok(data) => {
                            stall.record_progress(data.len() as u64);
                            // write to file at offset...
                        }
                        Err(e) => break, // retry
                    }
                }
                if stall.is_stalled() {
                    attempt += 1;
                    if attempt >= self.max_retries {
                        return Err(PipelineError::Network("stall timeout".into()));
                    }
                    stall.reset();
                    continue; // retry
                }
                return Ok(());
            }
            Err(_) => {
                attempt += 1;
                if attempt >= self.max_retries {
                    return Err(PipelineError::Network("connection failed".into()));
                }
                // n² backoff
                let backoff = Duration::from_millis(
                    (attempt as u64) * (attempt as u64) * 1000
                );
                tokio::time::sleep(backoff).await;
            }
        }
    }
}
```

### 3.3 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `streaming.rs` (new) | ~40 | `StallDetector` struct + impl |

---

## Enhancement 4: n² Backoff Retry with Jitter

**Target**: `streaming.rs` — `RetryPolicy` used inside `download_chunk`.

### 4.1 New struct in `streaming.rs`

```rust
/// Exponential backoff with jitter. Pattern: Ollama download retry.
/// delay = attempt² * base_delay + random_jitter(0..base_delay)
struct RetryPolicy {
    max_retries: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
}

impl RetryPolicy {
    fn default_download() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 1000,
            max_delay_ms: 30_000,
        }
    }

    /// Compute delay for attempt number (1-indexed).
    /// Returns None if retries exhausted.
    fn delay_for(&self, attempt: u32) -> Option<Duration> {
        if attempt > self.max_retries {
            return None;
        }
        let base = (attempt as u64) * (attempt as u64) * self.base_delay_ms;
        let capped = base.min(self.max_delay_ms);
        // Add jitter: 0..base_delay_ms
        let jitter = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64 % self.base_delay_ms;
        Some(Duration::from_millis(capped + jitter))
    }
}
```

### 4.2 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `streaming.rs` (new) | ~30 | `RetryPolicy` struct + impl |

---

## Enhancement 5: SHA-256 Integrity Verification

**Target**: `streaming.rs` — post-download hash verification. `sha2` crate already in `Cargo.toml` (line 88).

### 5.1 New function in `streaming.rs`

```rust
use sha2::{Sha256, Digest};

/// Verify file integrity against expected SHA-256 hash.
/// Returns Ok(()) if match, Err(actual_hash) if mismatch.
/// Skipped if expected_hash is None.
async fn verify_integrity(
    path: &Path,
    expected: Option<&str>,
) -> Result<(), String> {
    let expected = match expected {
        Some(h) => h,
        None => return Ok(()),
    };

    let data = tokio::fs::read(path).await
        .map_err(|e| format!("read for verify: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let actual = format!("{:x}", hasher.finalize());

    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!("SHA-256 mismatch: expected {}, got {}", expected, actual))
    }
}

/// Compute SHA-256 of a file (for storing in sidecar state).
pub async fn compute_sha256(path: &Path) -> Result<String, String> {
    let data = tokio::fs::read(path).await
        .map_err(|e| format!("read for hash: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}
```

### 5.2 Integration point

In `stream_http_download`, after all chunks complete:

```rust
// After ParallelDownloader::run() or single-stream completion:
if let Err(e) = verify_integrity(&output, config.expected_sha256.as_deref()).await {
    let _ = progress_tx.send(PipelineProgress {
        url: url.to_string(),
        status: PipelineStatus::Failed(format!("integrity check failed: {}", e)),
        media_kind,
        output: output.to_path_buf(),
        elapsed: started.elapsed(),
    }).await;
    return Err(PipelineError::Io(e));
}
```

### 5.3 Extend `PipelineConfig`

```rust
pub struct PipelineConfig {
    // ... existing fields ...
    pub expected_sha256: Option<String>,  // NEW
    pub concurrency: usize,               // NEW: parallel workers (default 4)
}
```

### 5.4 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `streaming.rs` (new) | ~40 | `verify_integrity`, `compute_sha256` |
| `streaming.rs` (modify) | ~2 | Add 2 fields to `PipelineConfig` |

---

## Enhancement 6: Channel-Based Progress Broadcast

**Target**: `download_progress.rs` — extend `DownloadProgress` to accept broadcast channel. Pattern: gosh-dl's `tokio::sync::broadcast`.

### 6.1 Extend `DownloadProgress` in `download_progress.rs`

```rust
use tokio::sync::broadcast;

/// Progress event broadcast to multiple consumers.
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed_bps: f64,
    pub active_chunks: usize,
    pub elapsed: Duration,
}

pub struct DownloadProgress {
    // ... existing fields ...
    broadcast_tx: Option<broadcast::Sender<ProgressEvent>>,  // NEW
}

impl DownloadProgress {
    /// Attach a broadcast channel for multi-consumer progress.
    pub fn with_broadcast(mut self, tx: broadcast::Sender<ProgressEvent>) -> Self {
        self.broadcast_tx = Some(tx);
        self
    }

    /// Emit progress to all broadcast subscribers.
    fn emit_broadcast(&self, event: ProgressEvent) {
        if let Some(ref tx) = self.broadcast_tx {
            let _ = tx.send(event); // ignore if no receivers
        }
    }
}
```

### 6.2 In `streaming.rs`, create broadcast channel

```rust
// At the start of stream_http_download or ParallelDownloader::run:
let (broadcast_tx, _) = broadcast::channel::<ProgressEvent>(64);

// Pass broadcast_tx to DownloadProgress
let progress = DownloadProgress::new(total_size, ProgressConfig::default())
    .with_broadcast(broadcast_tx.clone());

// Each chunk worker can subscribe:
let mut rx = broadcast_tx.subscribe();
```

### 6.3 Caller integration

`media_cmds.rs:143` already creates a `mpsc::channel(64)`. The broadcast channel is internal — the mpsc sender continues to work unchanged. The broadcast is for potential future consumers (e.g., Tauri progress widget, KB telemetry).

### 6.4 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `download_progress.rs` (modify) | ~20 | `ProgressEvent`, `broadcast_tx` field, `with_broadcast`, `emit_broadcast` |
| `streaming.rs` (modify) | ~5 | Create broadcast channel, wire to `DownloadProgress` |

---

## Enhancement 7: Graceful Degradation (Range Check)

**Target**: `router.rs` — add `supports_range` to `MediaRoute`. `streaming.rs` — fallback logic.

### 7.1 Extend `MediaRoute` in `router.rs`

```rust
pub struct MediaRoute {
    pub scheme: UrlScheme,
    pub transport: TransportType,
    pub is_huggingface: bool,
    pub supports_range: bool,      // NEW: detected via HEAD request
    pub total_size: Option<u64>,   // NEW: from HEAD Content-Length
    pub filename: Option<String>,
    pub output: PathBuf,
}
```

### 7.2 New function in `router.rs`

```rust
/// Probe server for Range support via HEAD request.
/// Returns (supports_range, total_size).
/// Pattern: gosh-dl/pget Range probe.
pub async fn probe_range_support(
    client: &reqwest::Client,
    url: &str,
) -> (bool, Option<u64>) {
    let resp = match client.head(url).send().await {
        Ok(r) => r,
        Err(_) => return (false, None),
    };

    let total = resp.content_length();
    let accepts_range = resp.headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("bytes"))
        .unwrap_or(false);

    (accepts_range, total)
}
```

### 7.3 Modify `route_url` signature

```rust
pub async fn route_url_async(
    url: &str,
    output_dir: &PathBuf,
    prefer_streaming: bool,
    client: &reqwest::Client,
) -> MediaRoute {
    let mut route = route_url(url, output_dir, prefer_streaming);
    let (supports, total) = probe_range_support(client, url).await;
    route.supports_range = supports;
    route.total_size = total;
    route
}

// Keep existing sync route_url for backward compat
pub fn route_url(url: &str, output_dir: &PathBuf, prefer_streaming: bool) -> MediaRoute {
    // ... existing impl, with supports_range: false, total_size: None as defaults
}
```

### 7.4 Fallback in `streaming.rs`

```rust
// In StreamingPipeline::run(), after route is determined:
if route.transport == TransportType::HttpRange && route.supports_range.unwrap_or(false) {
    // Use parallel download
    // ...
} else if route.transport == TransportType::HttpRange {
    // Server doesn't support Range — fall back to single-stream
    // Existing stream_http_download handles this naturally
    // (it already checks for 206 status)
}
```

### 7.5 Approximate additions

| Location | Lines | Content |
|----------|-------|---------|
| `router.rs` (modify) | ~25 | `probe_range_support`, `route_url_async`, extend `MediaRoute` |
| `router.rs` (modify `route_url`) | ~2 | Add default fields |
| `streaming.rs` (modify) | ~8 | Range check + fallback dispatch |

---

## Production Wiring (R-P79)

All enhancements wire to production callers in the same session:

### `media_cmds.rs` (lines 128-141)

```rust
// BEFORE:
let config = PipelineConfig {
    url: url.clone(),
    output_dir: out_dir.clone(),
    // ...
    persistence: None,
    // ...
};

// AFTER: add new fields (backward-compatible defaults)
let config = PipelineConfig {
    url: url.clone(),
    output_dir: out_dir.clone(),
    // ... existing fields unchanged ...
    expected_sha256: None,   // NEW: user can pass --sha256 <hash>
    concurrency: 4,          // NEW: default 4 parallel workers
};
```

### `offline_download.rs` (lines 41-46)

```rust
// BEFORE:
let pipeline = StreamingPipeline::new(PipelineConfig {
    url: url.to_string(),
    output_dir: self.output_dir.clone(),
    prefer_streaming: false,
    ..Default::default()
});

// AFTER: no change needed — Default::default() provides new fields
// expected_sha256: None, concurrency: 4 already via Default
```

### CLI extension (optional, `media_cmds.rs` arg parser)

```rust
// Add --sha256 and --concurrency flags:
"--sha256" => {
    expected_sha256 = args.get(i + 1).cloned();
    i += 2;
}
"--concurrency" | "-j" => {
    concurrency = args.get(i + 1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    i += 2;
}
```

---

## File Change Summary

| File | Before | After (est.) | Delta | Change Type |
|------|--------|--------------|-------|-------------|
| `streaming.rs` | 1071 | ~1350 | +280 | Add: `ParallelDownloader`, `StallDetector`, `RetryPolicy`, `verify_integrity`, `compute_sha256`. Modify: `stream_http_download`, `PipelineConfig` |
| `persistence.rs` | 431 | ~520 | +89 | Add: `SidecarState`, `ChunkResume`. Modify: `DownloadRecord` (3 fields) |
| `download_progress.rs` | 169 | ~200 | +31 | Add: `ProgressEvent`, `broadcast_tx` field, `with_broadcast` |
| `router.rs` | 194 | ~230 | +36 | Add: `probe_range_support`, `route_url_async`. Modify: `MediaRoute` (2 fields), `route_url` |
| `media_cmds.rs` | 325 | ~340 | +15 | Modify: add `--sha256`, `--concurrency` flags |
| `offline_download.rs` | 172 | 172 | 0 | No change needed (Default covers new fields) |
| **mod.rs** | 32 | 32 | 0 | No change — all types already re-exported |

**Total delta**: ~+450 lines across 5 files. Zero new files.

## Data Flow

```
media_cmds.rs / offline_download.rs
  │
  ▼
StreamingPipeline::run()
  │
  ├── router::route_url_async()  ← Enhancement 7: Range probe
  │     └── HEAD request → supports_range, total_size
  │
  ├── ParallelDownloader::probe()  ← Enhancement 1: Check Range support
  │     └── If !supports_range → fallback to single-stream
  │
  ├── SidecarState::load()  ← Enhancement 2: Resume check
  │     └── <output>.nt_state.json → chunk states
  │
  ├── ParallelDownloader::run()
  │     ├── spawn N worker tasks (work-stealing)
  │     │     └── download_chunk()
  │     │           ├── StallDetector  ← Enhancement 3: 30s stall
  │     │           ├── RetryPolicy    ← Enhancement 4: n² backoff
  │     │           └── write to file at chunk offset
  │     ├── SidecarState::save() every 5s  ← Enhancement 2
  │     └── broadcast ProgressEvent  ← Enhancement 6
  │
  ├── verify_integrity()  ← Enhancement 5: SHA-256
  │
  └── PipelineStatus::Complete
```

## Dependency Check

| Dependency | In Cargo.toml? | Used for |
|------------|----------------|----------|
| `reqwest` | Yes (line 100) | HTTP requests, Range headers |
| `tokio` | Yes (line 79) | Async runtime, file I/O, channels |
| `sha2` | Yes (line 88) | SHA-256 hashing |
| `serde` / `serde_json` | Yes (lines 77-78) | Sidecar state serialization |
| `futures` | Yes (line 134) | `StreamExt` for byte stream |
| `uuid` | Yes (line 82) | Sidecar state IDs |
| `dirs` | Yes (line 84) | Store path resolution |

**Zero new dependencies.** All from `reqwest` + `std` + existing crates. Compliant with R-P48.
