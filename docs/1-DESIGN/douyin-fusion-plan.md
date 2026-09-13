# Douyin-Downloader → NeoTrix nt_media Fusion Plan

**Source**: `jiji262/douyin-downloader` (Python)
**Target**: `nt_media` module (Rust, L1 Action layer)
**R-P42 Compliant**: Every pattern fuses into existing NeoTrix types — no parallel adapters.

---

## 1. Rate Limiter → TokenBucket (streaming.rs)

### Source Pattern (rate_limiter.py)
```python
class RateLimiter:
    def __init__(self, max_per_second: float = 2):
        self.min_interval = 1.0 / max_per_second
        self.last_request = 0.0
        self._lock = asyncio.Lock()

    async def acquire(self):
        async with self._lock:
            current = time.time()
            time_since_last = current - self.last_request
            if time_since_last < self.min_interval:
                await asyncio.sleep(self.min_interval - time_since_last)
            await asyncio.sleep(random.uniform(0, 0.5))  # jitter
            self.last_request = time.time()
```

**Core logic**: Interval-based rate limiting with jitter injection inside the lock. The jitter prevents thundering-herd on concurrent callers.

**Error handling**: None (always succeeds, just sleeps).

**Integration**: Called before every API request in `VideoDownloader.download()`.

### NeoTrix Fusion
**Target**: `nt_media::streaming::TokenBucket` (streaming.rs:781-823)

NeoTrix already has a `TokenBucket` with refill-rate semantics. The gap: **jitter** is absent. Douyin's approach adds `random.uniform(0, 0.5)` after the interval wait, which prevents synchronized bursts from multiple parallel download workers.

**Concrete change**:
```rust
// In streaming.rs TokenBucket::consume()
async fn consume(&mut self, amount: u64) -> u64 {
    self.refill();
    // ... existing wait logic ...
    // ADD: post-wait jitter to desynchronize parallel callers
    let jitter_ms = fastrand::u64(0..500);
    tokio::time::sleep(Duration::from_millis(jitter_ms)).await;
    // ... rest unchanged ...
}
```

**Also applies to**: `nt_act_rate_limiter.rs:TokenBucket` (same pattern, same fix).

---

## 2. Retry Handler → RetryPolicy Enhancement

### Source Pattern (retry_handler.py)
```python
class RetryHandler:
    def __init__(self, max_retries: int = 3):
        self.max_retries = max_retries
        self.retry_delays = [1, 2, 5]  # fixed delays, not exponential

    async def execute_with_retry(self, func, *args, **kwargs):
        for attempt in range(self.max_retries + 1):
            try:
                return await func(*args, **kwargs)
            except Exception as e:
                last_error = e
                if attempt < self.max_retries:
                    delay = self.retry_delays[min(attempt, len(self.retry_delays) - 1)]
                    await asyncio.sleep(delay)
        raise last_error
```

**Core logic**: Fixed-delay retry table `[1, 2, 5]` seconds. Caps at 4 total attempts (3 retries). Logs each failure. Re-raises last error after exhaustion.

**Error handling**: Catches all exceptions, logs warnings per attempt, raises after exhaustion.

**Integration**: Wraps every `file_manager.download_file()` call. Also wraps the video-with-fallback round-robin.

### NeoTrix Fusion
**Target**: `nt_media::streaming::RetryPolicy` (streaming.rs:728-775)

NeoTrix has `RetryPolicy` with n² backoff + jitter. Gap: **no configurable delay table** — the current impl computes `attempt² * base_delay` dynamically. Douyin's fixed `[1, 2, 5]` table is simpler and more predictable for network downloads.

**Concrete change**: Add a `delay_table` option to `RetryPolicy`:
```rust
pub struct RetryPolicy {
    max_retries: u32,
    base_delay: Duration,
    delay_table: Option<Vec<Duration>>,  // ADD: fixed delays override n²
}

impl RetryPolicy {
    pub fn delay(&self, attempt: u32) -> Option<Duration> {
        if attempt > self.max_retries { return None; }
        if let Some(ref table) = self.delay_table {
            let idx = (attempt.saturating_sub(1)) as usize;
            let delay = table.get(idx).copied().unwrap_or(*table.last().unwrap());
            return Some(delay);
        }
        // existing n² logic
    }
}
```

**Also applies to**: The `download_video_with_fallback` round-robin pattern (streaming.rs:~line 1440+ in `ParallelDownloader::download_chunk`) — this maps directly to douyin's `_download_video_with_fallback` which tries candidates in order with per-candidate retry disabled.

---

## 3. File Manager → persistence.rs Content-Length Validation

### Source Pattern (file_manager.py)
```python
async def _persist_stream(self, chunk_iter, save_path, expected_size, ...):
    # Write to .tmp, then atomic rename
    tmp_path = final_path.with_suffix(final_path.suffix + ".tmp")
    written = await self._stream_to_tmp(chunk_iter, tmp_path, expected_size, on_progress)

    if expected_size is not None and written != expected_size:
        logger.warning("Size mismatch: expected %d, got %d", expected_size, written)
        tmp_path.unlink(missing_ok=True)
        return False

    os.replace(str(tmp_path), str(final_path))  # atomic rename
```

**Core logic**:
1. Stream to `.tmp` file
2. Compare `written_bytes` against `Content-Length` (or `Content-Range` total)
3. On mismatch: delete `.tmp`, return false
4. On success: atomic rename `.tmp` → final path

**Error handling**: `BaseException` catch (including `CancelledError`) cleans up `.tmp`. `SlowDownloadError` triggers early abort. Size mismatch is a soft failure (returns false, not exception).

**Integration**: Called by both aiohttp and httpx download paths. Also handles `Content-Range` parsing for 206 responses.

### NeoTrix Fusion
**Target**: `nt_media::persistence.rs` (lines 340-460) + `streaming.rs` HttpStream path

NeoTrix already has:
- `validate_resume()` with ETag/Last-Modified checking (persistence.rs:347)
- `check_disk_space()` (persistence.rs:435)
- `compute_sha256_streaming()` (persistence.rs:449)
- `SidecarState` with chunk-level tracking

**Gaps to fill**:
1. **Atomic write pattern** — NeoTrix writes directly to the output file in the streaming path (streaming.rs:1506-1510). No `.tmp` intermediate. For non-chunked (sequential) downloads, this means a crash leaves a partial file with no way to distinguish it from a complete one.

2. **Content-Length post-validation** — NeoTrix reads `resp.content_length()` (streaming.rs:1453) to set `total_size` for progress tracking, but never validates that `bytes_written == total_size` at completion. A truncated download reports success.

**Concrete changes**:

```rust
// In streaming.rs HttpStream path, after download loop completes:
let final_bytes = bytes_written.load(Ordering::Relaxed);
if let Some(expected) = total_size {
    if final_bytes != expected {
        return Err(PipelineError::Io(format!(
            "Content-Length mismatch: expected {}, got {}",
            expected, final_bytes
        )));
    }
}
```

For non-chunked downloads, add atomic write:
```rust
// Before writing to output, use .tmp intermediate
let tmp_path = output.with_extension(format!("{}.dl_tmp", output.extension().unwrap_or_default()));
// ... write to tmp_path ...
// On success:
tokio::fs::rename(&tmp_path, output).await?;
// On failure:
tokio::fs::remove_file(&tmp_path).await.ok();
```

**Also applies to**: The `_complete_content_range_size` parser from file_manager.py (parses `Content-Range: bytes 0-12345/12346` headers). NeoTrix doesn't currently parse this — it only uses `Content-Length` directly.

---

## 4. Video Downloader → Download Integrity Check Pattern

### Source Pattern (video_downloader.py + downloader_base.py)

**A. `_download_video_with_fallback`** — Candidate-based degradation:
```python
async def _download_video_with_fallback(self, candidates, save_path, session):
    async def _attempt_round():
        for url, headers in candidates:
            if await self._download_with_retry(url, save_path, session,
                    headers=headers, optional=True, retry=False):
                return True
        raise RuntimeError("All candidates failed")

    return await self._run_within_item_deadline(
        self.retry_handler.execute_with_retry(_attempt_round), save_path)
```

**B. `_discard_if_encrypted`** — Post-download DRM check:
```python
def _discard_if_encrypted(self, video_path, aweme_id):
    scheme = detect_mp4_encryption(video_path)
    if scheme:
        video_path.unlink(missing_ok=True)
        return False
    return True
```

**C. `_run_within_item_deadline`** — Global timeout guard:
```python
async def _run_within_item_deadline(self, awaitable, save_path):
    try:
        return await asyncio.wait_for(awaitable, timeout=900)  # 15 min per item
    except asyncio.TimeoutError:
        return False
```

**D. `_should_download`** — Deduplication via local index + DB:
```python
async def _should_download(self, aweme_id):
    if self._is_locally_downloaded(aweme_id):
        return False
    if self.database and await self.database.is_downloaded(aweme_id):
        return False
    return True
```

### NeoTrix Fusion
**Target**: `nt_media::streaming.rs` ParallelDownloader + pipeline orchestration

NeoTrix already has:
- `StallDetector` (streaming.rs:~700) — speed-based stall detection
- `ParallelDownloader` with work-stealing (streaming.rs:829)
- `PipelineError` enum with `Http`, `Network`, `Io` variants

**Gaps to fill**:

1. **Candidate-based URL fallback** — NeoTrix's `StreamingPipeline` takes a single URL. No multi-mirror fallback. Douyin's `_build_video_url_candidates` builds a priority list (direct CDN > play endpoint > watermarked) and `_download_video_with_fallback` tries each. This maps to NeoTrix as:
   ```rust
   // Add to PipelineConfig:
   pub mirror_urls: Vec<String>,  // ordered fallback URLs
   pub item_deadline: Duration,   // per-item timeout (default 900s from douyin)
   ```

2. **Post-download integrity validation** — After download completes, validate the file is usable (not encrypted, not truncated, not an HTML error page). NeoTrix has SHA-256 verification but no content-type/format validation.
   ```rust
   // New function in streaming.rs or persistence.rs:
   pub fn validate_download_integrity(path: &Path, expected_kind: MediaKind) -> Result<(), PipelineError> {
       // 1. Check file size > 0
       // 2. Check magic bytes match expected format (mp4: ftyp, jpg: ff d8, etc.)
       // 3. Optional: detect DRM/encryption markers
   }
   ```

3. **Per-item deadline** — NeoTrix has stall timeout per chunk but no global per-download timeout. A single large file could block the pipeline indefinitely.
   ```rust
   // In StreamingPipeline::run(), wrap with deadline:
   let result = tokio::time::timeout(config.item_deadline, download_task).await;
   ```

---

## 5. Integration Map (R-P42 Compliant)

| Douyin Pattern | NeoTrix Target | File | Action |
|---|---|---|---|
| Rate limiter jitter | `TokenBucket::consume()` | streaming.rs:807 | Add `fastrand::u64(0..500)` ms jitter |
| Fixed delay table | `RetryPolicy::delay()` | streaming.rs:743 | Add `delay_table: Option<Vec<Duration>>` |
| Round-robin fallback | `PipelineConfig.mirror_urls` | streaming.rs:122 | Add `mirror_urls: Vec<String>` field |
| `.tmp` atomic write | HttpStream non-chunked path | streaming.rs:1505 | Add tmp→rename pattern |
| Content-Length validation | Post-download check | streaming.rs:1540 | Compare `bytes_written == total_size` |
| Per-item deadline | `PipelineConfig.item_deadline` | streaming.rs:122 | Add `item_deadline: Duration` field |
| `Content-Range` parser | `validate_resume()` | persistence.rs:347 | Parse `bytes start-end/total` |
| Download integrity check | New `validate_download_integrity()` | persistence.rs | Magic byte + size validation |
| Slow speed abort | `StallDetector` (already exists) | streaming.rs:700 | Already fused — no action |
| SHA-256 verification | `compute_sha256_streaming()` | persistence.rs:449 | Already fused — no action |

### What NOT to absorb (R-P42 exclusion)

- **`FileManager` directory hierarchy** — Douyin-specific (`author_name/sec_uid/mode/` layout). NeoTrix's `output_dir` config handles this generically.
- **`_sanitize_sec_uid_token`** — Douyin-specific path sanitization. Not applicable.
- **`_AUTHOR_DIR_STYLES`** — Domain-specific directory strategies. Not generic.
- **`detect_mp4_encryption`** — Douyin DRM detection. NeoTrix can add generic DRM detection separately if needed.
- **`TranscriptManager`** — Douyin transcript extraction. Not a download pattern.

---

## Implementation Order

1. **P0**: Content-Length post-validation (streaming.rs) — 1 function, closes data corruption risk
2. **P0**: Jitter in TokenBucket (streaming.rs) — 3 lines, prevents thundering herd
3. **P1**: `delay_table` in RetryPolicy (streaming.rs) — small refactor, enables fixed-delay mode
4. **P1**: `mirror_urls` + fallback loop (streaming.rs) — adds resilience for CDN failures
5. **P2**: `.tmp` atomic write for non-chunked path (streaming.rs) — crash safety
6. **P2**: `validate_download_integrity()` (persistence.rs) — post-download format check
7. **P2**: `item_deadline` timeout wrapper (streaming.rs) — prevents infinite block
8. **P3**: `Content-Range` parser (persistence.rs) — resume accuracy improvement
