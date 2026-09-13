# Download Architecture Research — Rust + Go Download Managers

**Date**: 2026-09-13
**Purpose**: Extract architecture patterns from 8 open-source download managers for NeoTrix NT-WORLD / NT-ACT download capability integration.

---

## 1. Individual Analyses

### 1.1 pget (Manas-Trivedi/pget) — Rust

**Repo**: `Manas-Trivedi/pget` | **License**: N/A | **Status**: Active (30 commits)

| Dimension | Detail |
|-----------|--------|
| **State file format** | Custom binary/text sidecar: `<filename>.pget`. Stores: `file_size`, `piece_size` (4 MiB), completed piece indices (set of `usize`), and resume validator (ETag or `Last-Modified`). |
| **Parallel strategy** | **Piece-based scheduling with shared queue**. File divided into fixed 4 MiB pieces. N workers (default 4) pull from a shared piece queue. Fast workers naturally grab more pieces — no long-tail stall. |
| **Resume validation** | Stores ETag or `Last-Modified` in sidecar. On restart: loads sidecar → validates `file_size` + `piece_size` match → compares ETag/Last-Modified against current remote. If mismatch → discards state, restarts from scratch. |
| **Error handling** | Exponential backoff retries on: probe requests, single-stream transfers, and ranged piece requests. Request + connect timeouts prevent indefinite hangs. Ctrl+C handler persists state on exit. |
| **Progress reporting** | Continuous in-place terminal renderer: bytes transferred, effective throughput (MiB/s), ETA. Optional `--verbose` mode with per-worker progress bars. Cursor hidden during transfer. |

**Key Design Insight**: The 4 MiB fixed piece size decouples worker count from file size. Piece queue + shared consumer eliminates load-balancing complexity. Sidecar state is validated against remote before resume — prevents stale resumes.

---

### 1.2 parallel_downloader (velumuruganr/parallel_downloader) — Rust

**Repo**: `velumuruganr/parallel_downloader` | **License**: MIT | **Crate**: `parallel_downloader v0.3.0`

| Dimension | Detail |
|-----------|--------|
| **State file format** | JSON sidecar: `*.state.json`. Per-chunk completion status, file metadata, expected SHA-256. Atomic write pattern. |
| **Parallel strategy** | **Equal-division chunking**. File divided into N equal chunks based on thread count. Each thread downloads one chunk via HTTP Range. Chunks are pre-assigned (no work-stealing). |
| **Resume validation** | On restart: reads `.state.json` → validates file size via HEAD → skips completed chunks → downloads remaining. Optional SHA-256 integrity check on completion. |
| **Error handling** | Automatic retry on network timeouts per chunk. Token-bucket rate limiting (optional) for bandwidth cap. Batch mode via `-i list.txt`. |
| **Progress reporting** | Per-chunk progress bars (indicate-style). Global progress aggregation. CLI flags: `--threads`, `--rate-limit`, `--verify-sha256`. |

**Key Design Insight**: Composable library + CLI separation. The `pd` CLI is thin glue over the library core. Token-bucket rate limiting shared across threads via atomic tokens. SHA-256 verification is opt-in per download.

---

### 1.3 Shard (nathsujal/Shard) — Rust

**Repo**: `nathsujal/Shard` | **License**: MIT | **Status**: Early development (7 commits)

| Dimension | Detail |
|-----------|--------|
| **State file format** | Planned: persistent metadata for interrupted downloads. Implementation early-stage — not yet fully specified. |
| **Parallel strategy** | **Parallel chunked downloads** with HTTP range requests. Files split into shards (chunks), downloaded concurrently, merged. Async workers via Tokio. |
| **Resume validation** | Planned: HTTP Range support for resume. Exact validation mechanism not yet implemented. |
| **Error handling** | Planned: retry and recovery mechanisms. Architecture designed for fault tolerance under unstable networks. |
| **Progress reporting** | Planned: real-time progress tracking, speed + ETA monitoring. |

**Key Design Insight**: Architectural philosophy is strong — "treat downloading as a systems engineering problem." Direct-to-disk streaming, modular architecture, extensible protocol support. Early stage but clean async-first design.

---

### 1.4 bolt (dasunNimantha/bolt) — Rust + iced GUI

**Repo**: `dasunNimantha/bolt` | **License**: MIT | **Status**: Active (71 commits, GUI app)

| Dimension | Detail |
|-----------|--------|
| **State file format** | Persistent segment state in app data directory. Segment state survives app restarts. Settings persisted as JSON (`settings.rs`). Download history tracked persistently (up to 500 entries). |
| **Parallel strategy** | **8-segment auto-scale** by file size: 1 segment < 5 MB, 2 < 20 MB, 4 < 50 MB, 6 < 200 MB, 8 for larger. Each segment = separate HTTP Range connection. Configurable up to 8 segments. |
| **Resume validation** | Segment state persisted to disk. On restart: loads segment state → resumes incomplete segments. Auto-resume on network reconnect via periodic connectivity check (`generate_204`). |
| **Error handling** | Exponential backoff per segment. Failed segments retry independently. Network drop detection → auto-pause all → auto-resume when `generate_204` succeeds. Concurrent download queue (1-10 simultaneous downloads). |
| **Progress reporting** | Native iced GUI: per-segment progress bars, global speed, ETA. System tray tooltip with speed + active count. `--background` flag for headless. |

**Key Design Insight**: The auto-scale heuristic (file size → segment count) is a simple but effective pattern. Network connectivity monitoring with `generate_204` (Chrome's connectivity check) for auto-resume is production-grade. Browser extension integration via Native Messaging Host (NMH) + TCP IPC.

---

### 1.5 concurrent-download-manager (Neelav-0211/concurrent-download-manager) — Rust

**Repo**: `Neelav-0211/concurrent-download-manager` | **License**: MIT

| Dimension | Detail |
|-----------|--------|
| **State file format** | No persistent state file. Downloads are in-memory only. Resume supported via HTTP Range headers (file size check, not chunk tracking). |
| **Parallel strategy** | **Worker thread pool**. Configurable `num_workers` threads. Each thread downloads a separate file (not chunks of one file). Library-level abstraction. |
| **Resume validation** | HTTP Range header resume: checks existing file size, requests `Range: bytes=size-`. No checksum validation during resume. SHA-256 verification available post-download. |
| **Error handling** | `DownloadError` enum with typed variants. Simple retry on failure. No exponential backoff documented. |
| **Progress reporting** | `indicatif` progress bars (real-time). `show_progress` boolean flag. Static methods: `calculate_checksum()`, `verify_checksum()`. |

**Key Design Insight**: Minimal library-first design. `Downloader::new(num_workers, show_progress)` → `submit_download(url, path, checksum)` → `wait_all()`. Simple, composable API for embedding. Focuses on multi-file concurrent downloads rather than single-file parallel chunks.

---

### 1.6 multhreadown (littlepenguin66/multhreadown) — Rust

**Repo**: `littlepenguin66/multhreadown` | **License**: MIT | **Crate**: `multhreadown v0.1.1`

| Dimension | Detail |
|-----------|--------|
| **State file format** | **CacheManager** with structured `DownloadCache` entries: `{ url, file_path, total_size, downloaded_chunks: Vec<(offset, length)> }`. Persisted to configurable cache directory. |
| **Parallel strategy** | **Multi-threaded chunked download**. Configurable thread count + chunk size. Each thread handles a range of the file. Equal-division with range-based assignment. |
| **Resume validation** | `CacheManager::load_download_state(url)` → restores chunk offset/length pairs. MD5 hashing for cache keying. Re-requests remaining ranges on resume. |
| **Error handling** | Configurable retry attempts + retry delay. Connect timeout. User-agent spoofing. Event handler trait for custom error responses. |
| **Progress reporting** | **Event-driven architecture** via `DownloadEventHandler` trait. Events: `DownloadStarted`, `DownloadCompleted`, `ProgressUpdated(url, DownloadStats)`, `DownloadFailed`, `DownloadPaused`, `DownloadResumed`. `GlobalProgress` aggregation. `indicatif` integration. |

**Key Design Insight**: The `DownloadEventHandler` trait pattern is the most extensible progress reporting among all surveyed tools. `CacheManager` with `DownloadCache` struct provides structured state persistence. The `Config` builder pattern (`with_threads()`, `with_chunk_size()`, `with_rate_limit()`) is clean.

---

### 1.7 CycleZero/downloader — Go

**Repo**: `CycleZero/downloader` | **License**: MIT

| Dimension | Detail |
|-----------|--------|
| **State file format** | **`.dlstate` JSON file**. Records: completed chunk IDs (bitmap/set), URL, file size, chunk size, ETag. Atomic write (temp file + rename) on every chunk completion. State file deleted on success. |
| **Parallel strategy** | **Dynamic chunk-based worker pool** (work-stealing). File split into many small chunks → fixed pool of N goroutines pulls from shared `taskCh`. Fast workers automatically grab more work. No long-tail bandwidth drop-off. |
| **Resume validation** | On restart: (1) HEAD/ranged GET probe for file size + ETag, (2) load `.dlstate`, (3) validate: URL + file size + chunk size + ETag (if both sides have one), (4) skip completed chunks, (5) download remaining. |
| **Error handling** | Chunk-level retry with exponential backoff (`RetryBackoffBase`, default 1s). Independent per-chunk — one failing chunk doesn't block others. First error cancels context → drains results. Graceful cancellation via `context.WithCancel`. |
| **Progress reporting** | Real-time stderr progress: percentage, speed, ETA, per-chunk completion count. |

**Key Design Insight**: **Direct `pwrite` (no merge)** is the killer feature. Each worker writes directly to the final file at its chunk offset via `WriteAt`. No temp files, no merge step, 1x disk usage. The dynamic worker pool (work-stealing via channel) eliminates the long-tail problem inherent in static chunk assignment. Atomic state writes (temp + rename) ensure crash safety.

---

### 1.8 crawlingo (Vamshavardhan50/crawlingo) — Rust

**Repo**: `Vamshavardhan50/crawlingo` | **License**: MIT | **Crate**: `crawlingo v0.1.2`

| Dimension | Detail |
|-----------|--------|
| **State file format** | No dedicated state file. Resume is file-size-based: if output file exists, send `Range: bytes=<size>-`. Single-stream, not chunked. |
| **Parallel strategy** | **Single-stream streaming**. Not a parallel downloader. `Downloader` writes response body in `chunk_size` (default 64 KiB) increments to a `Write` target. Shares transport with session's `FetchManager`. |
| **Resume validation** | HTTP 206 status indicates successful resume. If server returns 200 (no Range support), existing file is truncated and re-downloaded from start. No ETag/checksum validation. |
| **Error handling** | `retries: 2` hardcoded. Transport errors preserve partial file for future retry. Rate limiting shared via `FetchManager`. |
| **Progress reporting** | `DownloadResult` struct: `url`, `status`, `bytes_written`, `content_type`, `suggested_filename`, `resumed` (bool). No real-time progress callbacks. |

**Key Design Insight**: `extract_filename()` parses `Content-Disposition` (both `filename=` and RFC 5987 `filename*=UTF-8''`), falling back to URL path segment. `sniff_content_type()` strips charset parameters. Single-stream design is appropriate for its use case as a crawling engine's download module. Shares rate limiting, retry, caching, and auth with the parent session.

---

## 2. Comparison Matrix

| Dimension | pget | parallel_downloader | Shard | bolt | concurrent-dl-mgr | multhreadown | CycleZero | crawlingo |
|-----------|------|--------------------|-------|------|--------------------|--------------|-----------|-----------|
| **Language** | Rust | Rust | Rust | Rust | Rust | Rust | Go | Rust |
| **State format** | Custom sidecar `.pget` | JSON `.state.json` | Planned | App data JSON | None | `DownloadCache` struct | JSON `.dlstate` | File-size heuristic |
| **State persistence** | On interrupt + completion | On chunk completion | Planned | On segment change | None | On chunk completion | Atomic per-chunk | None (file exists) |
| **Parallel model** | Piece queue (work-stealing) | Equal-division | Chunked (planned) | Auto-scale segments | Worker threads | Equal-division chunks | Work-stealing pool | Single-stream |
| **Chunk size** | Fixed 4 MiB | Equal (file/N) | Configurable | File-size adaptive | N/A (per-file) | Configurable | Auto or configurable | 64 KiB streaming |
| **Resume validation** | ETag/Last-Modified + size | File size only | Planned | Segment state | File size | Chunk offset/length | URL + size + chunk size + ETag | HTTP 206 status |
| **Integrity check** | SHA-256/MD5/SHA-512 | SHA-256 | Planned | None | SHA-256 | MD5 (cache key) | SHA-256 (optional) | None |
| **Retry strategy** | Exponential backoff | Timeout retry | Planned | Per-segment retry | Simple retry | Configurable attempts + delay | Exponential backoff per chunk | Hardcoded 2 retries |
| **Rate limiting** | None (planned) | Token-bucket | Planned | Global KB/s cap | None | Bytes/sec limit | None | Shared via FetchManager |
| **Progress mechanism** | Terminal renderer | Per-chunk bars | Planned | GUI + tray | indicatif bars | Event trait + indicatif | stderr progress | DownloadResult struct |
| **Disk strategy** | Preallocate + random write | Sequential per chunk | Merge (planned) | Segment write | Sequential | Sequential | **Direct pwrite (no merge)** | Sequential streaming |
| **GUI** | CLI | CLI | CLI (planned) | Native iced | Library | CLI | Library | Library |
| **Production ready** | Yes | Yes | Early dev | Yes | Yes (library) | Yes | Yes | Yes (as crawl module) |

---

## 3. Pattern Taxonomy

### 3.1 State Persistence Patterns

| Pattern | Projects | Pros | Cons |
|---------|----------|------|------|
| **Sidecar file** (`.pget`, `.dlstate`) | pget, CycleZero | Clean separation, atomic writes possible | File management overhead |
| **JSON state file** (`.state.json`) | parallel_downloader | Human-readable, easy debug | Slower writes for large chunk sets |
| **Structured cache** (`DownloadCache`) | multhreadown | Rich metadata, typed | More complex serialization |
| **File-size heuristic** | crawlingo | Zero state overhead | No multi-chunk resume |
| **No persistence** | concurrent-download-manager | Simple | No resume across restarts |

### 3.2 Parallel Distribution Patterns

| Pattern | Projects | Pros | Cons |
|---------|----------|------|------|
| **Work-stealing queue** | pget, CycleZero | Natural load balancing, no long-tail | Implementation complexity |
| **Equal-division** | parallel_downloader, multhreadown | Simple, predictable | Long-tail if speeds vary |
| **Auto-scale by file size** | bolt | Adaptive, no config needed | Heuristic may not fit all servers |
| **Worker thread pool** | concurrent-download-manager | Simple embedding | Per-file, not per-chunk |

### 3.3 Resume Validation Patterns

| Pattern | Projects | Safety |
|---------|----------|--------|
| **ETag + size + chunk size** | CycleZero, pget | High — detects server-side changes |
| **File size only** | parallel_downloader, concurrent-dl-mgr | Medium — assumes file hasn't changed |
| **HTTP 206 status** | crawlingo | Low — no content validation |
| **Chunk offset/length** | multhreadown | Medium — assumes no corruption |

### 3.4 Error Recovery Patterns

| Pattern | Projects | Coverage |
|---------|----------|----------|
| **Exponential backoff per chunk** | pget, CycleZero | Best — independent recovery |
| **Configurable retry + delay** | multhreadown | Good — tunable |
| **Global rate-limit + timeout** | parallel_downloader, crawlingo | Adequate for simple cases |
| **Network monitoring + auto-resume** | bolt | Best UX — invisible recovery |

---

## 4. NeoTrix Integration Recommendations

### 4.1 Core Download Engine (`nt_world_download`)

**Adopt from CycleZero/downloader**:
- **Direct `pwrite` pattern**: Eliminate merge step. Each chunk worker writes directly to final file at offset. 1x disk usage.
- **Dynamic work-stealing worker pool**: Shared channel of chunk tasks. Fast workers grab more. Eliminates long-tail.
- **Atomic state writes**: Temp file + rename per chunk completion. Crash loses at most 1 in-flight chunk.

**Adopt from pget**:
- **Fixed piece size (4 MiB)** with shared queue: Simple, predictable, decouples worker count from file size.
- **Sidecar state format**: Custom binary format (not JSON) for faster serialization of large piece sets. Store: `file_size`, `piece_size`, `completed_pieces: BitVec`, `etag`, `last_modified`.

### 4.2 State Management (`nt_memory` integration)

**Unified state schema**:
```rust
struct DownloadState {
    url: String,
    file_size: u64,
    piece_size: usize,
    completed_pieces: BitVec,       // Bitmap — O(1) lookup per piece
    etag: Option<String>,
    last_modified: Option<String>,
    content_type: String,
    suggested_filename: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

- **BitVec for piece tracking**: CycleZero's bitmap approach — O(1) piece completion check, compact storage.
- **Atomic persistence**: Write to temp → rename (atomic on POSIX). State file: `<output>.ntstate`.
- **Validation on resume**: Verify URL + file_size + piece_size + ETag/Last-Modified. Mismatch → discard state → restart.

### 4.3 Error Recovery (`nt_shield` integration)

**Tiered retry**:
1. **Per-piece retry** (3 attempts, exponential backoff: 1s, 2s, 4s)
2. **Per-connection retry** (2 attempts on connection drop)
3. **Network monitoring** (adopt bolt's `generate_204` pattern — auto-pause on disconnect, auto-resume on reconnect)

**Rate limiting**: Token-bucket (adopt parallel_downloader's approach) shared across all workers. Configurable via `Config::with_rate_limit(bytes_per_sec)`.

### 4.4 Progress Reporting (`nt_io` integration)

**Event-driven architecture** (adopt multhreadown's pattern):
```rust
trait DownloadEventHandler: Send + Sync {
    fn on_started(&self, url: &str, total_size: u64);
    fn on_progress(&self, url: &str, stats: DownloadStats);
    fn on_completed(&self, url: &str, result: DownloadResult);
    fn on_failed(&self, url: &str, error: DownloadError);
    fn on_paused(&self, url: &str);
    fn on_resumed(&self, url: &str);
}
```

- **`DownloadStats`**: `bytes_downloaded`, `total_bytes`, `progress` (f64), `download_speed` (bytes/sec), `eta` (Duration).
- **Multiple consumers**: `Arc<dyn DownloadEventHandler>` — CLI progress bar, EventBus integration, KB telemetry.

### 4.5 Content Detection (adopt crawlingo patterns)

- **`Content-Disposition` parsing**: Both `filename=` and RFC 5987 `filename*=UTF-8''`.
- **MIME sniffing**: Strip charset parameters from `Content-Type`.
- **Range support probe**: `Range: bytes=0-0` probe → check for `206 Partial Content`.

### 4.6 Segmentation Heuristic (adopt bolt's auto-scale)

```rust
fn auto_segment_count(file_size: u64) -> usize {
    match file_size {
        0..=5_000_000 => 1,
        5_000_001..=20_000_000 => 2,
        20_000_001..=50_000_000 => 4,
        50_000_001..=200_000_000 => 6,
        _ => 8,
    }
}
```

### 4.7 API Design (library-first, adopt parallel_downloader + multhreadown patterns)

```rust
// Builder pattern
let dl = DownloadBuilder::new("https://example.com/big.zip")
    .output("./downloads/big.zip")
    .threads(8)
    .chunk_size(4 * 1024 * 1024)  // 4 MiB
    .rate_limit(1024 * 1024)       // 1 MiB/s
    .checksum(Sha256("abcd1234..."))
    .event_handler(my_handler)
    .build()?;

// Execute
dl.download().await?;

// Resume is automatic — checks for .ntstate sidecar
```

---

## 5. Cross-Cutting Observations

| Observation | Evidence | NeoTrix Action |
|-------------|----------|----------------|
| **JSON is too slow for large chunk sets** | CycleZero, parallel_downloader use JSON but CycleZero notes atomic write overhead | Use binary BitVec for piece tracking |
| **Work-stealing always beats equal-division** | pget, CycleZero show no long-tail; parallel_downloader, multhreadown can stall on slow chunks | Default to work-stealing queue |
| **ETag validation prevents stale resumes** | pget, CycleZero validate ETag; others don't and risk corrupted resumes | Always validate ETag + file size |
| **Direct pwrite eliminates merge complexity** | CycleZero: 1x disk usage, no temp files, no merge step | Adopt as primary disk strategy |
| **Event-based progress is most composable** | multhreadown's `DownloadEventHandler` trait allows CLI/GUI/KB consumers | Adopt trait-based pattern |
| **Network monitoring is UX-critical** | bolt's `generate_204` auto-resume is invisible to user | Implement connectivity watcher |
| **Library + CLI separation scales** | parallel_downloader, multhreadown both separate concerns | Design as library with CLI thin wrapper |

---

## 6. Reference Projects

| Project | Language | Stars | Key Innovation |
|---------|----------|-------|----------------|
| [Manas-Trivedi/pget](https://github.com/Manas-Trivedi/pget) | Rust | 0 | Piece-based scheduling, sidecar state |
| [velumuruganr/parallel_downloader](https://github.com/velumuruganr/parallel_downloader) | Rust | - | Composable API, token-bucket rate limiting |
| [nathsujal/Shard](https://github.com/nathsujal/Shard) | Rust | 0 | Async-first systems design (early) |
| [dasunNimantha/bolt](https://github.com/dasunNimantha/bolt) | Rust | 5 | Auto-scale segments, network monitoring, GUI |
| [Neelav-0211/concurrent-download-manager](https://github.com/Neelav-0211/concurrent-download-manager) | Rust | 0 | Minimal library API |
| [littlepenguin66/multhreadown](https://github.com/littlepenguin66/multhreadown) | Rust | 2 | Event-driven progress, CacheManager |
| [CycleZero/downloader](https://github.com/CycleZero/downloader) | Go | 0 | Direct pwrite, work-stealing, atomic state |
| [Vamshavardhan50/crawlingo](https://crates.io/crates/crawlingo) | Rust | - | Content-Disposition parsing, streaming |

---

## 7. Additional Reference: QDM (PBhadoo/QDM) — Tauri/Rust

Discovered during research. 112 stars. Tauri 2 + Rust backend. Key patterns:

- **Smart probing**: HEAD request before download to detect file size, filename, resumability.
- **Per-segment state persisted to disk** — similar to bolt's approach.
- **HLS/DASH streaming engine** — custom Rust streaming parser.
- **Exponential moving average** for speed calculation — smoother than simple averaging.
- **Chrome extension (MV3)** for download interception — similar to bolt's NMH approach.
- **Up to 32 segments** (configurable 1-32) — more aggressive than bolt's 8.

Reinforces: segment auto-scale, state persistence, and browser integration are table-stakes for production download managers.
