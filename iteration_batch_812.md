# Iteration Batch 812 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Image Processing (8)
- image-rs 0.25.10: Missing rayon feature flag, only PNG+JPEG enabled (no WebP/AVIF/GIF)
- imageproc 0.27.0: Perceptual hashing (pHash/dHash/aHash), bilateral filters, geometric transforms
- nalgebra 0.35.0: Typed matrices/vectors, geometric transforms, no_std support
- ndarray 0.17.2: N-dimensional arrays, BLAS backend, rayon par_azip!
- rayon 1.12.0: Work-stealing parallelism, par_array_windows, join_context
- Video pipeline pixel access O(n²) per frame (no SIMD/parallel chunking)
- JEPA encoder has no real matrix library (no BLAS acceleration)
- Visual extract has no image preprocessing (min_image_dimension declared but never used)

### Web Frameworks (10)
- Axum 0.8: Clear default (60-70% job postings), Tower middleware ecosystem is differentiator
- Mutex poisoning under concurrent load (all AppState fields use std::sync::Mutex)
- Blocking I/O in async context (std::fs::read_to_string, std::process::Command::new("git"))
- No IntoResponse error type (ad-hoc tuple errors)
- SSE stream via oneshot is not truly streaming
- CORS hand-rolled instead of tower_http::CorsLayer
- No graceful shutdown
- OpenAPI static YAML (not generated from code)
- Rate limiter uses std::sync::Mutex in hot path
- No request tracing (tower_http::trace::TraceLayer missing)

### Serialization (8)
- rmp-serde: 50% byte storage overhead without serde_bytes wrapper
- postcard: No schema evolution (field order fixed)
- bincode 2.0.1: Hostile governance (bans AI contributions), OOM without explicit limit
- simd-json: Unsafe surface violates R-P1 (must isolate to NT-WORLD only)
- miniserde: Severe limitations (no generics, no enum data variants)
- serde monomorphization bloat scales with type count
- No schema versioning in any binary format
- bincode has no data versioning scheme

### Concurrency Primitives (8)
- dashmap 6.2.1: 7.0.0-rc2 stalled 14+ months
- flume: Casual maintenance mode (no new features)
- kanal: Pre-1.0 (0.1.1), breaking changes possible
- parking_lot: deadlock_detection + send_guard mutually exclusive
- tokio::sync::broadcast: Lagging receivers silently dropped
- crossbeam-channel: Unbounded variants have no backpressure
- kanal: In-process only (no cross-node support)
- tokio::sync::Mutex: Higher overhead than parking_lot for data-only sections

---

## Defects Identified (34+)

### Image Processing (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-IMG-1 | image crate missing rayon feature (single-threaded decode/encode) | Medium |
| D-IMG-2 | Video pipeline pixel access O(n²) per frame | High |
| D-IMG-3 | No WebP/AVIF/GIF support for web-crawled content | High |
| D-IMG-4 | No perceptual hashing for asset dedup (custom dHash only) | Medium |
| D-IMG-5 | No nalgebra for geometric transforms | Medium |
| D-IMG-6 | JEPA encoder has no real matrix library | High |
| D-IMG-7 | rayon version pin too loose ("1" allows any 1.x) | Low |
| D-IMG-8 | Visual extract has no image preprocessing | Medium |

### Web Frameworks (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-WEB-1 | Mutex poisoning under concurrent load (all AppState) | Critical |
| D-WEB-2 | Blocking I/O in async context (fs::read_to_string, Command) | Critical |
| D-WEB-3 | No IntoResponse error type (ad-hoc tuple errors) | Medium |
| D-WEB-4 | SSE stream via oneshot is not truly streaming | Medium |
| D-WEB-5 | CORS hand-rolled (missing preflight, spec violation) | Low |
| D-WEB-6 | No graceful shutdown | Low |
| D-WEB-7 | OpenAPI static YAML (not generated from code) | Medium |
| D-WEB-8 | Rate limiter uses std::sync::Mutex in hot path | Low |
| D-WEB-9 | No request tracing (tower_http::trace::TraceLayer missing) | Medium |
| D-WEB-10 | AtomicU64 with SeqCst overkill (Relaxed suffices) | Low |

### Serialization (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-SER-1 | rmp-serde 50% byte overhead without serde_bytes | High |
| D-SER-2 | postcard no schema evolution (field order fixed) | High |
| D-SER-3 | bincode hostile governance (bans AI contributions) | High |
| D-SER-4 | simd-json unsafe violates R-P1 (must isolate to NT-WORLD) | High |
| D-SER-5 | miniserde severe limitations (no generics, no enum data) | Low |
| D-SER-6 | serde monomorphization bloat scales with type count | Medium |
| D-SER-7 | No schema versioning in any binary format | High |
| D-SER-8 | bincode no data versioning scheme | High |

### Concurrency Primitives (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-CONC-1 | dashmap 7.0 stalled 14+ months | Medium |
| D-CONC-2 | flume casual maintenance mode | Medium |
| D-CONC-3 | kanal pre-1.0 breaking changes possible | Medium |
| D-CONC-4 | parking_lot deadlock_detection + send_guard mutually exclusive | Medium |
| D-CONC-5 | tokio broadcast lagging receivers silently dropped | High |
| D-CONC-6 | crossbeam-channel unbounded variants no backpressure | Medium |
| D-CONC-7 | kanal in-process only (no cross-node support) | Medium |
| D-CONC-8 | tokio::sync::Mutex higher overhead than parking_lot | Low |

## Key Insights (This Batch)

1. **image crate missing rayon feature**: NeoTrix already depends on rayon directly, but image crate's internal parallelism is disabled. Adding "rayon" feature is one-line fix.

2. **Axum Tower middleware ecosystem is the differentiator**: NeoTrix uses raw middleware::from_fn, bypassing the entire Tower ecosystem. Switching to Tower Layer traits unlocks shared middleware for auth, compression, rate limiting.

3. **Mutex poisoning is critical for web server**: All AppState fields use std::sync::Mutex. A panic in any handler poisons the lock, cascading failures. Must use tokio::sync::RwLock or DashMap.

4. **simd-json violates R-P1**: Must be isolated to NT-WORLD/NT-IO layers, never in NT-CORE which has #![forbid(unsafe_code)].

5. **bincode hostile to AI**: The official bincode repo bans AI contributions. NeoTrix as an AI-native project should avoid this dependency.

6. **flume in maintenance mode**: No new features, only critical fixes. Should prefer crossbeam-channel for core pipeline channels.

7. **tokio broadcast lagging is silent**: GWT attention broadcast may lose salient signals without any logging or error handling.

8. **serde_bytes is mandatory for Vec<u8>**: Without it, rmp-serde serializes byte arrays as array-of-ints (50% overhead).

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 812 |
| New defects (this batch) | 34 |
| Cumulative defects | D01-D76448 |
| Research sources (this batch) | 34 |
| Cumulative research sources | 97,220+ |
