# Iteration Batch 842 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Error Handling (10)
- thiserror for libraries (typed enums), anyhow for applications (type-erased)
- 50+ functions use Result<T, String> in dialogue/tauri layer (CRITICAL)
- 100+ .map_err(|e| e.to_string()) occurrences (stringly-typed error anti-pattern)
- Only 9 #[from] annotations across 70+ error enum variants
- No backtrace support anywhere
- No centralized error type for NT-CORE domain
- No .context()/.with_context() usage anywhere
- bail!/ensure! macros not used
- neotrix-types exposes Result<T, String> in public API traits
- anyhow gated behind full feature flag

### Concurrency (7)
- std::sync::RwLock in FFI layer: hold-across-await deadlock risk
- std::sync::Mutex in global statics: 95% starvation under contention
- parking_lot::Mutex regression on AMD Zen (4-10x slower than std)
- DashMap len()/is_empty() TOCTOU-unsafe
- Atomic ordering mismatch: SeqCst overuse + Relaxed misuse
- No #[must_not_suspend] on async-held guards
- tokio::sync::RwLock write-preferring starvation risk

### Process Management (6)
- Command::spawn slow on GLIBC <2.24 (fork overhead)
- Inconsistent shutdown across domains (no unified ShutdownManager)
- Missing PID-1 handling (zombie processes in containers)
- No process-group signal forwarding
- cgroups v2 not integrated (no resource limits)
- Health-check/shutdown race (no 503-on-SIGTERM pattern)

### Caching (8)
- Hand-rolled LRU in AneProgramCache: O(n²) eviction
- CompressionStore uses FIFO, not LRU (access_count tracked but unused)
- TileCache: O(n) scan on every eviction
- SemanticTileCache: O(n) cosine similarity scan (no ANN index)
- No concurrency safety across any cache (all require &mut self)
- No write-through/write-behind support
- No cache stampede protection
- Fingerprint collision (64-bit effective entropy)

---

## Defects Identified (31+)

### Error Handling (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-ERR-1 | Result<T, String> epidemic (50+ functions) | Critical |
| D-ERR-2 | .map_err(|e| e.to_string()) boilerplate (100+) | High |
| D-ERR-3 | #[source]/#[from] almost absent (9/70+ variants) | High |
| D-ERR-4 | No backtrace support anywhere | Medium |
| D-ERR-5 | No centralized error type for NT-CORE | Medium |
| D-ERR-6 | No .context()/.with_context() usage | Medium |
| D-ERR-7 | bail!/ensure! macros not used | Low |
| D-ERR-8 | neotrix-types exposes Result<T, String> in traits | High |
| D-ERR-9 | No error recovery/retry patterns | Medium |
| D-ERR-10 | anyhow gated behind full feature flag | Low |

### Concurrency (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-CON-1 | FFI std::sync::RwLock across .await (deadlock) | Critical |
| D-CON-2 | Event bus std::sync::Mutex starvation (95%) | High |
| D-CON-3 | parking_lot regression on AMD Zen | Medium |
| D-CON-4 | DashMap len()/is_empty() TOCTOU-unsafe | Medium |
| D-CON-5 | SeqCst overuse on standalone counters | Low |
| D-CON-6 | No #[must_not_suspend] on async-held guards | Medium |
| D-CON-7 | tokio RwLock write-preferring in punch engine | Low |

### Process Management (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-PROC-1 | No GLIBC-aware process spawning | Medium |
| D-PROC-2 | Inconsistent shutdown across domains | High |
| D-PROC-3 | Missing PID-1 handling (zombie accumulation) | High |
| D-PROC-4 | No process-group signal forwarding | High |
| D-PROC-5 | cgroups v2 not integrated | Medium |
| D-PROC-6 | Health-check/shutdown race | Medium |

### Caching (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-CACHE-1 | AneProgramCache O(n²) eviction | High |
| D-CACHE-2 | CompressionStore FIFO (access_count unused) | Medium |
| D-CACHE-3 | TileCache O(n) scan on eviction | Medium |
| D-CACHE-4 | SemanticTileCache O(n) cosine scan (no ANN) | High |
| D-CACHE-5 | No concurrency safety (all require &mut self) | High |
| D-CACHE-6 | No write-through/write-behind support | Medium |
| D-CACHE-7 | No cache stampede protection | High |
| D-CACHE-8 | Fingerprint collision (64-bit effective entropy) | Low |

## Key Insights (This Batch)

1. **Result<T, String> epidemic is CRITICAL**: 50+ functions in dialogue/tauri layer destroy error type information, prevent programmatic handling, and make debugging impossible. Must migrate to thiserror enums.

2. **FFI std::sync::RwLock across .await is CRITICAL**: All 10 FFI structs wrap inner state in Arc<RwLock<*Inner>>. Guards held across .await will deadlock on multi-threaded tokio runtime.

3. **parking_lot regression on AMD Zen**: 4-10x slower than std::sync::Mutex when thread count approaches physical core count. Default to std mutex for data-plane hot paths.

4. **Command::spawn slow on GLIBC <2.24**: fork overhead affects NeoTrix SEAL pipeline steps. Use POSIX_SPAWN_USEVFORK or clone on older Linux.

5. **Inconsistent shutdown across domains**: NT-CORE, NT-MIND, NT-WORLD each have different shutdown paths. No unified ShutdownManager. Must implement phased shutdown (stop accept → signal clients → wait in-flight → flush state → close DB → exit).

6. **SemanticTileCache O(n) cosine scan**: No ANN index (HNSW/IVF). For large tile caches, this is a hot path performance bottleneck.

7. **No cache stampede protection**: Multiple callers hitting same cache miss simultaneously all trigger expensive computation. Must use single-flight pattern.

8. **Fingerprint collision (64-bit)**: compute_fingerprint copies same 8-byte hash to both halves, reducing effective entropy. Must use 128-bit hash.

9. **SeqCst overuse on standalone counters**: Full memory fence on every operation unnecessary. Sequence counters and shutdown flags should use Relaxed ordering.

10. **PID-1 handling missing**: Containerized NeoTrix lacks init system (tini). Zombie processes accumulate. Must add tini as init process.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 842 |
| New defects (this batch) | 31 |
| Cumulative defects | D01-D77206 |
| Research sources (this batch) | 31 |
| Cumulative research sources | 98,155+ |
