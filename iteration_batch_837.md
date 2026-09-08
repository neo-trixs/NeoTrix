# Iteration Batch 837 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Serialization (10)
- bincode is dead (RUSTSEC-2025-0141): v3.0.0 is compile_error!; postcard recommended replacement
- postcard: no_std, stable wire format, MaxSize derive, smallest wire size
- bincode-next v3.1.1: wire-compatible with bincode 2.x, zero-copy via relative pointers
- rkyv 0.8.17: total zero-copy (archived form = in-memory layout), unsafe internally
- serde-zap: fastest serde binary, tagged varint, zero-copy, no_std, fuzzed
- rmp-serde: no MaxSize compile-time bound (OOM crash from corrupted data)
- Zero-copy has two tiers: partial (serde Cow) vs total (rkyv pointer cast)
- serde zero-copy treacherous with human-readable formats (runtime failure)
- Apache arrow-rs migrated bincode→postcard Jan 2026
- zerovec: zero-copy Vec/HashMap via ZeroVec/ZeroMap

### Concurrency (10)
- DashMap cross-shard deadlock when holding Ref guard + insert on different key
- DashMap Eq/is_empty race false positives (dangerous for synchronization)
- Tokio Mutex deadlock: futures paused without polling cause permanent deadlock
- std::sync::Mutex in async hot path → Tokio runtime starvation (423ms heartbeat gap)
- parking_lot 0.12.5: 1 byte Mutex, 1.5-5x faster, no poisoning
- Arc<Mutex<T>> as default antipattern; exhaust single-owner design first
- Relaxed ordering insufficient for cross-thread visibility of non-atomic data
- EventBus Clone creates isolated copies (silently drops sync_handlers)
- MemoryBudget peak RSS race (TOCTOU): lost peak updates
- usage_accumulator Relaxed ordering may lose cross-thread visibility

### Process Management (10)
- Kernel containment is gold standard: process-group, cgroup v2, Job Object
- SIGTERM→SIGKILL escalation: graceful first, forced after timeout
- Process-group isolation: setsid() + killpg() for whole-tree kill
- CancellationToken + TaskTracker: cooperative shutdown with tracked completion
- Orphan processes: without kernel containment, grandchildren outlive parent
- TorClient::stop() uses child.kill() without SIGTERM escalation
- TorClient uses blocking std::process::Command in async context
- No process-group isolation for spawned children
- Sandbox::exec_local has no timeout (blocks forever)
- No Drop implementation for TorClient (process leak on panic)

### Caching (10)
- S3-FIFO: 72% of cache objects are one-hit wonders, lowest miss ratio on 10/14 datasets
- quick-cache: 6× throughput vs LRU at 16 threads, S3-FIFO eviction
- moka: TinyLFU admission + LRU eviction, built-in TTL/TTI, eviction listeners
- SLRU: probation→protected promotion gives scan resistance
- Cache-aside is safest default; write-through doubles write cost; write-behind risks data loss
- Lease tokens cut stampede load 13× (Facebook memcache)
- Jittered TTLs eliminate synchronized expiration
- Single-flight locks (Redis SETNX) prevent stampede
- Hand-rolled LRU with O(n) eviction (VecDeque retain)
- 8+ independent cache implementations with no unified policy

---

## Defects Identified (40+)

### Serialization (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-SER-1 | bincode dependency unmaintained (RUSTSEC-2025-0141) | High |
| D-SER-2 | rmp-serde has no MaxSize compile-time bound (OOM crash) | High |
| D-SER-3 | rmp-serde byte array overhead ~50% | Medium |
| D-SER-4 | No zero-copy deserialization in NT-MEMORY | Medium |
| D-SER-5 | No compile-time serialized size bounds | Medium |
| D-SER-6 | No format versioning in serialized data | Medium |
| D-SER-7 | Mixed serialization formats (complexity) | Low |

### Concurrency (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-CON-1 | EventBus Clone creates isolated copies (silently drops sync_handlers) | Critical |
| D-CON-2 | MemoryBudget peak RSS race (TOCTOU) | High |
| D-CON-3 | Pervasive std::sync::Mutex in async contexts (starvation) | High |
| D-CON-4 | Overly conservative SeqCst ordering where weaker suffices | Medium |
| D-CON-5 | parking_lot underutilized (only 1 file uses it) | Medium |
| D-CON-6 | usage_accumulator Relaxed ordering may lose cross-thread visibility | Medium |

### Process Management (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-PROC-1 | TorClient::stop() uses child.kill() without SIGTERM escalation | High |
| D-PROC-2 | TorClient uses blocking std::process::Command in async context | High |
| D-PROC-3 | No process-group isolation for spawned children | High |
| D-PROC-4 | Sandbox::exec_local has no timeout (blocks forever) | High |
| D-PROC-5 | No Drop implementation for TorClient (process leak on panic) | High |
| D-PROC-6 | SandboxPool::acquire() not actually pool-aware | Medium |
| D-PROC-7 | No signal handling for graceful shutdown | Critical |
| D-PROC-8 | std::process::exit(0) bypasses all destructors | Medium |
| D-PROC-9 | No cgroup resource limits on sandbox children | Medium |
| D-PROC-10 | TorClient uses static LAST_ROTATION (shared across instances) | Low |

### Caching (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CACHE-1 | Hand-rolled LRU with O(n) eviction (VecDeque retain) | Critical |
| D-CACHE-2 | No concurrent cache (all require &mut self) | High |
| D-CACHE-3 | No S3-FIFO or scan-resistant eviction | Medium |
| D-CACHE-4 | No cache stampede/thundering herd protection | High |
| D-CACHE-5 | No jittered TTLs (synchronized expiration) | Medium |
| D-CACHE-6 | No eviction listener (no hooks for cleanup) | Low |
| D-CACHE-7 | No write-through/write-behind pattern | Medium |
| D-CACHE-8 | Cache fragmentation (8+ independent implementations) | Structural |
| D-CACHE-9 | Moka get() clones value on every hit | Low |
| D-CACHE-10 | GraphCache has no eviction (unbounded memory) | Medium |

## Key Insights (This Batch)

1. **EventBus Clone creates isolated copies**: Cloned EventBus silently drops all sync_handlers and thread handles. Events emitted from clone execute handlers that were never registered. Silent data loss bug.

2. **bincode is dead**: RUSTSEC-2025-0141 marks bincode as unmaintained. v3.0.0 is compile_error! (deliberate sabotage). postcard is the recommended replacement (Apache arrow-rs migrated Jan 2026).

3. **Kernel containment is gold standard**: Process-group (setsid + killpg), cgroup v2, Job Object (Windows). Kill-on-drop is a kernel operation over the whole tree, not best-effort signal to one PID.

4. **S3-FIFO handles scan resistance**: 72% of cache objects are one-hit wonders. LRU is vulnerable to scan pollution. S3-FIFO's small-main segment split handles this natively.

5. **Lease tokens cut stampede load 13×**: Facebook memcache showed lease mechanisms reduce peak DB load from 17,000 to 1,300 QPS during stampedes.

6. **std::sync::Mutex in async hot path causes starvation**: Valkey GLIDE #5450 case study: 423ms heartbeat gap, 60 stuck operations. Must use tokio::sync::Mutex or parking_lot.

7. **rmp-serde OOM crash from corrupted data**: No MaxSize compile-time bound allows corrupted bytes to be interpreted as multi-exabyte length prefixes.

8. **Hand-rolled LRU with O(n) eviction**: VecDeque retain scans entire vector on every get(). ~1000× slower than O(1) intrusive doubly-linked list at 1000 entries.

9. **TorClient has no Drop implementation**: If TorClient is dropped during panic unwinding, Tor process is never killed. Must implement Drop or use kill_on_drop(true).

10. **No jittered TTLs**: Items inserted during burst all expire simultaneously, creating synchronized stampedes. Jitter ±10% eliminates this.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 837 |
| New defects (this batch) | 33 |
| Cumulative defects | D01-D77048 |
| Research sources (this batch) | 40 |
| Cumulative research sources | 97,967+ |
