# Model Download Patterns — Cross-Framework Analysis

> Research date: 2026-09-13
> Purpose: Extract download strategies from major LLM frameworks for NeoTrix Rust-native implementation (R-P48: no external binaries)

## Comparison Matrix

| Dimension | Ollama | llama.cpp | HuggingFace Hub | vllm | text-generation-webui |
|-----------|--------|-----------|-----------------|------|----------------------|
| **Language** | Go | C++ (httplib) | Python (httpx) | Python (hf_hub) | Python (requests) |
| **Protocol** | HTTP Range (parallel chunks) | HTTP Range (single file) | HTTP streaming + Range | Delegates to HF Hub | HTTP Range (single file) |
| **Parallel chunks** | Yes (N parts per blob) | No (per-file async only) | No per-file; thread_map per-file | No (delegates) | Yes (thread_map, 4 threads) |
| **Resume** | `.tmp` + Range header | `.downloadInProgress` + Range | `.incomplete` sidecar + Range | Delegates to HF Hub | `.tmp` check + Range |
| **Integrity** | SHA-256 (full blob) | ETag file sidecar | ETag-based cache key | Delegates to HF Hub | SHA-256 (post-download) |
| **Cache** | Content-addressed blob store | Flat cache dir + ETag | Git-aware blob/snapshot/ref | HF cache (delegates) | Flat `user_data/models/` |
| **Stall detection** | 30s per-part timeout | None built-in | 5 retries with 1s backoff | Delegates | Exponential backoff |

---

## 1. Ollama (Go)

### Source
- `server/download.go` — main download logic
- `x/transfer/download.go` — newer transfer layer

### Download Protocol
- **Parallel chunked download**: Splits each blob into N parts (default `numDownloadParts`), each downloaded concurrently via `errgroup`.
- **HTTP Range headers**: Each chunk sends `Range: bytes=<start>-<end>`.
- **Direct URL resolution**: Follows redirects to find CDN URL, then reuses that URL for all chunks. Prevents unnecessary redirects per-chunk.

```go
// Key pattern: split blob into parts, download concurrently
req.Header.Set("Range", fmt.Sprintf("bytes=%d-%d", part.StartsAt(), part.StopsAt()-1))
```

### Resume Mechanism
- **Part files**: Each chunk writes to `<name>-partial-<N>` on disk.
- **Progress tracking**: Each part tracks `Completed` (atomic int64) separately.
- **On restart**: Reads existing part files, skips completed parts (`part.Completed.Load() == part.Size`).
- **Large blob resume** (newer `transfer.go`): For blobs ≥64MB, preserves `.tmp` file on failure. On retry, re-hashes existing data and sends `Range: bytes=<existing_size>-`.
- **Graceful fallback**: If server returns HTTP 200 (not 206), falls back to full download.

```go
// Resume: check existing partial file
if existingSize > 0 {
    req.Header.Set("Range", fmt.Sprintf("bytes=%d-", existingSize))
}
```

### Parallel Strategy
- `errgroup.SetLimit(numDownloadParts)` — limits concurrency.
- Per-part stall detection: 30s without progress → retry that part.
- Newer `transfer.go`: configurable `BodyConcurrency` via semaphore.

### Integrity Verification
- **SHA-256**: Full blob hash verified after download.
- **Chunk-level verification**: Newer `chunksums` API fetches per-chunk digests for inline verification (no separate verification pass).
- **Write atomicity**: Download to `Name-partial`, then `os.Rename` to final name.

### Cache Management
- Content-addressed: `blobs/sha256/<hex>` directory structure.
- Duplicate detection by digest: `blobDownloadManager.LoadOrStore(digest)`.
- Blobs checked before download: `os.Stat(fp)` returns → skip.

### Error Handling
- **Exponential backoff**: `math.Pow(2, try)` seconds between retries.
- **Max retries**: `maxRetries` per part.
- **Stall detection**: 30s no-data → `errPartStalled` → retry.
- **Cancellation-aware**: `context.Canceled` and `ENOSPC` return immediately.
- **Configurable timeout**: `OLLAMA_DOWNLOAD_TIMEOUT` env var.

### Adaptation Notes for NeoTrix
- The Go `errgroup` maps to Rust `tokio::task::JoinSet` or `futures::stream::FuturesUnordered`.
- Part files on disk for crash recovery → use temp files with atomic rename.
- SHA-256 streaming verification via `sha2` crate.

---

## 2. llama.cpp (C++)

### Source
- `common/download.cpp` — HTTP download via cpp-httplib
- `common/download.h` — interface definitions

### Download Protocol
- **Single HTTP GET with optional Range**: Uses cpp-httplib (`httplib::Client`).
- **HEAD request first**: Gets `Content-Length`, `ETag`, `Accept-Ranges` header.
- **Conditional download**: Only downloads if ETag differs from cached.

```cpp
auto head = cli.Head(parts.path);
// Check Accept-Ranges
bool supports_ranges = head->has_header("Accept-Ranges") 
    && head->get_header_value("Accept-Ranges") != "none";
```

### Resume Mechanism
- **Temp file**: Downloads to `<path>.downloadInProgress`.
- **Resume on retry**: Checks if temp file exists + server supports ranges → sends `Range: bytes=<existing_size>-`.
- **If server doesn't support ranges**: Deletes temp file and restarts.

```cpp
size_t existing_size = 0;
if (std::filesystem::exists(path_temporary)) {
    if (supports_ranges) {
        existing_size = std::filesystem::file_size(path_temporary);
    } else if (remove(path_temporary.c_str()) != 0) {
        break;
    }
}
```

### Parallel Strategy
- **Per-file async**: `std::async(std::launch::async)` for downloading multiple files (model + mmproj + mtp) in parallel.
- **No intra-file parallelism**: Single connection per file.
- **Retry**: 3 attempts, exponential backoff (2s, 4s, 8s).

### Integrity Verification
- **ETag sidecar**: Stores `<path>.etag` and `<path>.lastModified` alongside model file.
- **No checksum verification**: Relies on ETag + file size match.
- **Cache validation**: HEAD → compare ETag with stored → skip if same.

### Cache Management
- Flat cache directory (`~/.cache/llama.cpp/` or configurable via `LLAMA_CACHE`).
- ETag-based versioning: re-download only when ETag changes.
- `--skip-download` flag for offline mode (returns cached file or errors).

### Error Handling
- **Offline mode**: `opts.offline` → use cached file only, return 304.
- **HEAD failure fallback**: If HEAD fails but file exists → assume cached (304).
- **Cancel support**: `opts.callback->is_cancelled()` check before retry.

### Adaptation Notes for NeoTrix
- ETag-based caching is lightweight and effective — good for NeoTrix's KB model metadata.
- No intra-file parallelism is a missed opportunity; NeoTrix should adopt Ollama's chunk-based approach.
- Sidecar files (`.etag`) for cache validation is a clean pattern.

---

## 3. HuggingFace Hub (Python)

### Source
- `src/huggingface_hub/file_download.py` — core `hf_hub_download()` and `http_get()`
- `src/huggingface_hub/_snapshot_download.py` — `snapshot_download()`

### Download Protocol
- **HTTP streaming**: `response.iter_bytes(chunk_size=DOWNLOAD_CHUNK_SIZE)` (5MB default).
- **Range headers for resume**: `Range: bytes=<resume_size>-`.
- **HEAD for metadata**: Gets `ETag`, `Content-Length`, handles redirects.
- **Xet storage** (newer): Content-addressable chunk-based deduplication via `hf_xet` (Rust `xet-core`).

```python
# Resume via Range header
if resume_size > 0:
    headers["Range"] = _adjust_range_header(headers.get("Range"), resume_size)
```

### Resume Mechanism
- **Always-on resume**: `resume_download` parameter deprecated — downloads always resume when possible.
- **`.incomplete` temp file**: Downloaded to `<cache_path>.incomplete`, renamed on completion.
- **Server 200 fallback**: If server returns 200 (not 206) when Range sent → truncates temp file and restarts.
- **Resume metadata sidecar** (PR #4143, newer): `.metadata` file with ETag/size/URL to detect stale resumes.
- **5 retries**: Recursive `http_get` calls with 1s sleep between retries.

```python
# If server ignores Range header
if resume_size > 0 and response.status_code == 200:
    temp_file.seek(0)
    temp_file.truncate()
    resume_size = 0
```

### Parallel Strategy
- **snapshot_download()**: Downloads multiple files via `thread_map(max_workers=8)`.
- **No intra-file parallelism** by default.
- **hf_transfer** (Rust extension): Parallel chunk download for large files, enables ~1GB/s.
- **hf_xet** (newer): Content-addressable chunks, download only needed ranges.

### Integrity Verification
- **ETag-based cache key**: Filename = `hash(url + etag)`.
- **Size validation**: `expected_size != temp_file.tell()` → `OSError`.
- **No content hash verification**: Relies on ETag + size.
- **Cache structure**: `blobs/<sha256>` (content-addressed) with symlinks in `snapshots/<commit>/`.

### Cache Management
- **Git-aware cache**: `refs/<branch> → commit_hash`, `snapshots/<commit>/` symlinks to blobs.
- **Content-addressed blobs**: Same content shared across revisions.
- **Version pinning**: Pass commit hash to `revision` for deterministic downloads.
- **Cache location**: `~/.cache/huggingface/hub/` (configurable via `HF_HOME`).

### Error Handling
- **Offline fallback**: `local_files_only=True` → use cached files only.
- **Connection error fallback**: Network down → try cached version.
- **Stall retry**: 5 retries on `ConnectError`/`TimeoutException` with 1s sleep.
- **Hub downtime**: Falls back to local cache silently.

### Adaptation Notes for NeoTrix
- ETag-based versioning is the gold standard — adopt this.
- Content-addressed blob storage prevents duplicate downloads.
- `thread_map` parallelism across files is simple and effective.
- The 200-vs-206 fallback logic is critical for CDN compatibility.

---

## 4. vllm (Python)

### Source
- `vllm/model_executor/model_loader/weight_utils.py` — `download_weights_from_hf()`
- `vllm/transformers_utils/repo_utils.py` — `get_model_path()`

### Download Protocol
- **Delegates entirely to HuggingFace Hub**: Uses `snapshot_download()` and `hf_hub_download()`.
- **No custom download logic**: Relies on HF Hub's implementation.

```python
hf_folder = snapshot_download(
    model_name_or_path,
    allow_patterns=allow_pattern,
    cache_dir=cache_dir,
    tqdm_class=DisabledTqdm,
    revision=revision,
    local_files_only=local_only,
)
```

### Resume Mechanism
- Inherits HF Hub's resume behavior (`.incomplete` files + Range headers).

### Parallel Strategy
- **File-level parallelism**: `snapshot_download()` uses `thread_map(max_workers=8)`.
- **Smart pattern reduction**: Tries to narrow `allow_patterns` to a single pattern to minimize download calls.
- **Weight index optimization**: Downloads `model.safetensors.index.json` first to determine exact shard files needed.

```python
# Download index file first, then only needed shards
index_path = hf_hub_download(repo_id=model_name_or_path, filename=SAFE_WEIGHTS_INDEX_NAME)
weight_map = json.load(index_path)["weight_map"]
allow_patterns = [list(set(weight_map.values()))]
```

### Integrity Verification
- Delegates to HF Hub (ETag-based).

### Cache Management
- Uses HF Hub cache (`~/.cache/huggingface/hub/`).
- `--download-dir` flag for custom cache location.
- `HF_HUB_CACHE` env var support.
- File locking via `get_lock()` to prevent concurrent downloads of same model.

### Error Handling
- **Offline fallback**: `HF_HUB_OFFLINE=1` → use cached files.
- **Connection fallback**: Catches `ConnectionError`/`Timeout` → try local cache.
- **File lock**: Prevents multiple processes downloading same model simultaneously.

### Adaptation Notes for NeoTrix
- The weight index optimization (download index first, then only needed shards) is valuable.
- File locking pattern prevents duplicate downloads across processes.
- Pattern-based filtering is useful for downloading only needed model variants.

---

## 5. text-generation-webui (Python)

### Source
- `download-model.py` — standalone download script

### Download Protocol
- **HTTP Range headers**: Checks existing file size, sends `Range: bytes=<size>-`.
- **HEAD for metadata**: Gets `x-linked-size` or `content-length` for total size.
- **Streamed download**: `r.iter_content(1024 * 1024)` (1MB chunks).

```python
headers = {'Range': f'bytes={current_file_size_on_disk}-'}
mode = 'ab'  # append mode for resume
```

### Resume Mechanism
- **File size check**: If `current_file_size_on_disk >= total_size` → skip download.
- **Range header**: Sends Range request for remaining bytes.
- **Append mode**: Opens file in `'ab'` mode for resume.
- **No temp file**: Downloads directly to final path.

### Parallel Strategy
- **thread_map**: Downloads multiple files in parallel (`max_workers=threads`, default 4).
- **Per-file retry**: Each file has independent retry logic.
- **Progress bar slots**: Shared array for coordinating tqdm progress bar positions.

```python
thread_map(
    lambda url: self.get_single_file(url, output_folder, start_from_scratch=start_from_scratch),
    file_list,
    max_workers=threads,
)
```

### Integrity Verification
- **SHA-256 post-download**: Reads `lfs oid` from HF API response, verifies after download.
- **Separate check command**: `--check` flag validates checksums of previously downloaded files.
- **No inline verification**: Downloads first, verifies separately.

```python
# From HF API: sha256 = [[filename, lfs_oid], ...]
for i in range(len(sha256)):
    file_hash = hashlib.sha256(open(fpath, 'rb').read()).hexdigest()
    if file_hash != sha256[i][1]:
        print(f'Checksum failed: {sha256[i][0]}')
```

### Cache Management
- **Flat model directory**: `user_data/models/<org>_<model>/`.
- **No versioning**: Just overwrite on re-download.
- **GGUF single file**: Stored directly in `user_data/models/`.
- **Metadata file**: `huggingface-metadata.txt` with URL, branch, date, SHA-256 hashes.

### Error Handling
- **Exponential backoff**: `2 ** attempt` seconds between retries.
- **Max retries**: Configurable (default 7).
- **Retry on**: `RequestException`, `ConnectionError`, `Timeout`.
- **No offline mode**: Always requires network.

### Adaptation Notes for NeoTrix
- The metadata file pattern (`huggingface-metadata.txt`) is useful for audit trails.
- SHA-256 verification from HF API is a clean approach.
- Progress bar coordination pattern is good for multi-file downloads.

---

## Synthesis: Best Patterns for NeoTrix (Rust-Native)

### Core Architecture (R-P48 compliant)

```rust
// Proposed: nt_world::model_download module

pub struct ModelDownloader {
    client: reqwest::Client,
    cache: ContentAddressedCache,
    max_concurrent_chunks: usize,  // default 4-8
    max_retries: usize,            // default 5
    stall_timeout: Duration,       // default 30s
}

pub struct DownloadPlan {
    pub blobs: Vec<BlobDescriptor>,
    pub total_size: u64,
}

pub struct BlobDescriptor {
    pub digest: String,        // sha256:<hex>
    pub size: u64,
    pub url: String,
    pub etag: Option<String>,  // for cache validation
}

pub struct DownloadState {
    pub completed_parts: Vec<PartState>,
    pub total_completed: u64,
}

pub struct PartState {
    pub index: usize,
    pub offset: u64,
    pub size: u64,
    pub completed: u64,       // bytes downloaded
    pub temp_path: PathBuf,   // .partial-<index>
}
```

### Key Design Decisions

| Decision | Pattern | Source |
|----------|---------|--------|
| **Chunk-based parallel download** | Split large blobs into N parts, download concurrently | Ollama |
| **ETag-based cache validation** | Store `<path>.etag` sidecar, skip download if matches | llama.cpp, HF Hub |
| **Content-addressed storage** | `blobs/sha256/<hex>` prevents duplicates | Ollama, HF Hub |
| **Always-on resume** | `.partial` temp files + Range headers | HF Hub, text-gen-webui |
| **Atomic file writes** | Download to `.partial`, rename on completion | Ollama, HF Hub |
| **Size + hash verification** | Final SHA-256 after download | Ollama, text-gen-webui |
| **Stall detection** | Per-part 30s timeout with retry | Ollama |
| **Weight index optimization** | Download index first, then only needed shards | vllm |
| **File locking** | Prevent duplicate downloads across processes | vllm |
| **Offline fallback** | Use cached files when network unavailable | HF Hub, llama.cpp, vllm |

### Recommended Rust Crates

| Crate | Purpose | Replaces |
|-------|---------|----------|
| `reqwest` (with `stream` feature) | HTTP client with streaming | Python `requests`/`httpx` |
| `tokio` | Async runtime for parallel downloads | Go `errgroup` |
| `sha2` | SHA-256 streaming verification | `hashlib` |
| `tokio::fs` | Async file I/O | Go `os.OpenFile` |
| `futures` | Stream combinators, `FuturesUnordered` | `thread_map` |
| `serde`/`serde_json` | State file serialization | Manual file I/O |
| `tracing` | Structured logging | `slog` (Go) |

### Download Flow

```
1. Resolve model identifier → (repo, revision, files)
2. HEAD request → get ETag, Content-Length, Accept-Ranges
3. Check cache → compare ETag with stored → skip if match
4. Plan download → split into chunks (min 64MB per chunk for resume efficiency)
5. Check existing partial files → resume from last position
6. Download concurrently → tokio::JoinSet with semaphore limit
7. Per-chunk stall detection → 30s no-data → retry
8. Per-chunk SHA-256 verification (if chunksums available)
9. Atomic rename → .partial → final path
10. Full blob SHA-256 verification
11. Update cache metadata → ETag, size, timestamp
```

### State File Format

```json
{
  "digest": "sha256:abcdef1234567890...",
  "etag": "\"abc123\"",
  "total_size": 4294967296,
  "chunk_size": 67108864,
  "chunks": [
    {
      "index": 0,
      "offset": 0,
      "size": 67108864,
      "completed": 67108864,
      "etag": "\"chunk0_abc\""
    },
    {
      "index": 1,
      "offset": 67108864,
      "size": 67108864,
      "completed": 33554432,
      "etag": null
    }
  ]
}
```

### Error Handling Matrix

| Error | Action | Source |
|-------|--------|--------|
| Connection reset | Retry with backoff (2^n seconds) | All frameworks |
| HTTP 408/429/502/503 | Retry with backoff | All frameworks |
| HTTP 200 when Range sent | Restart from byte 0 | HF Hub |
| HTTP 416 Range Not Satisfiable | Restart download | HF Hub |
| ENOSPC (disk full) | Abort immediately | Ollama |
| Stall (30s no data) | Retry that chunk | Ollama |
| ETag mismatch | Re-download entirely | llama.cpp, HF Hub |
| Offline/network down | Fall back to cached files | HF Hub, llama.cpp, vllm |

---

## References

| Source | File | Key Lines |
|--------|------|-----------|
| Ollama | `server/download.go` | `Prepare()`, `run()`, `downloadChunk()` |
| Ollama | `x/transfer/download.go` | `download()`, `downloadOnce()`, `save()` |
| llama.cpp | `common/download.cpp` | `common_download_file_single_online()`, `common_download_model()` |
| HF Hub | `file_download.py` | `http_get()`, `hf_hub_download()` |
| HF Hub | `_snapshot_download.py` | `snapshot_download()` |
| vllm | `weight_utils.py` | `download_weights_from_hf()` |
| text-gen-webui | `download-model.py` | `ModelDownloader.get_single_file()` |
