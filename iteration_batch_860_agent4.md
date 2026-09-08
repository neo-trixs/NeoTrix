# Agent 4: Async Caching Patterns (Batch 860)

## Sources
1. https://docs.rs/moka/latest/moka/future/struct.Cache.html — Moka v0.12 async cache: TTL/TTI, TinyLFU, eviction listener, per-entry expiration, hierarchical timer wheels
2. https://docs.rs/crate/moka/latest — Moka feature matrix: moka vs quick-cache vs mini-moka comparison
3. https://docs.rs/cachet/latest/cachet/struct.Cache.html — Cachet stampede protection: concurrent request coalescing, leader-panic handling, TOCTOU window documentation
4. https://docs.rs/crate/fusioncache-rs/latest — FusionCache: single-flight stampede protection via broadcast channel + mutex map
5. https://redis.io/docs/latest/develop/use-cases/cache-aside/rust/ — Redis cache-aside with Lua-backed single-flight lock, lock TTL tuning, token-based release
6. https://autumn-web.app/docs/cache-stampede — Autumn framework: in-process single-flight registry, distributed fill lock, stale-while-revalidate, jittered TTL
7. https://github.com/arthurprs/quick-cache — Quick-cache: S3-FIFO/CLOCK-Pro, atomic `get_or_insert_async`, item pinning, lifecycle hooks
8. https://docs.rs/quick_cache/latest/quick_cache/ — Quick-cache docs: scan-resistant eviction, lock-per-shard, closure-based entry API

## Defects

**D-CACHE-001: No async/concurrent cache — all caches are bare `HashMap` behind `Mutex`** | `nt_core_cache.rs:52-58` | HIGH | Moka docs: `future::Cache` provides lock-free concurrent access with internal sharding. NeoTrix `SemanticCache` is a plain `HashMap<String, CacheEntry>` wrapped in `Mutex<SemanticCache>` at gateway level (`gateway/mod.rs:72`). Every `get`/`set` acquires the mutex, serializing all LLM provider calls through a single lock. Under concurrent agent dispatch this becomes a global bottleneck. | Source: moka `future::Cache` lock-free design; cachet `CacheTier` async API

**D-CACHE-002: No cache stampede protection on any cache** | `nt_core_cache.rs:92-100` | CRITICAL | When a hot semantic cache key expires, every concurrent LLM request for the same prompt will observe a miss simultaneously and recompute cosine similarity against all embedding entries. Moka's `get_with`/`try_get_with` coalesce concurrent misses; cachet's `stampede_protection()` merges waiters. NeoTrix has zero coalescing — N concurrent requests = N full embedding scans + N provider calls. | Source: cachet stampede_protection docs; fusioncache-rs single-flight; Autumn `get_or_compute`

**D-CACHE-003: No TTL/TTI on `ResponseCache` or `McpResultCache`** | `response_cache.rs:9-20`, `nt_core_mcp.rs:1271-1277` | HIGH | `ResponseCache` stores LLM responses with no expiration — stale provider responses persist until LRU eviction. `McpResultCache` records `created_at` but never checks it. Moka provides cache-level TTL/TTI + per-entry variable expiration via hierarchical timer wheels. For LLM caching, stale responses from outdated model versions or changed contexts are dangerous. | Source: moka TTL/TTI policies; Redis cache-aside TTL

**D-CACHE-004: O(n) eviction scan in `TileCache::evict_if_needed`** | `nt_memory_spatial/cache.rs:65-78` | MEDIUM | Eviction iterates all entries to find `min_by_key(cached_at)` — O(n) per eviction. At 10K default capacity this is acceptable but pathological under burst inserts. Moka uses sharded LRU queues (O(1) eviction). Quick-cache uses CLOCK-Pro (amortized O(1)). For a spatial tile cache with 500MB budget, eviction should be O(1) amortized. | Source: quick-cache CLOCK-Pro O(1) eviction; moka sharded LRU

**D-CACHE-005: `McpResultCache` uses `VecDeque::retain` O(n) on every `get`** | `nt_core_mcp.rs:1290` | MEDIUM | Every cache hit calls `self.order.retain(|k| k != key)` which scans the entire VecDeque. For MCP tool calls that may execute hundreds of times per session, this degrades to O(n) per access. Moka's access-time tracking is lock-free and O(1) amortized via internal weight-based admission. | Source: moka internal LRU queue O(1) amortized; quick-cache atomic get_or_insert

**D-CACHE-006: `StepRouteCache` FIFO eviction discards recently-used entries** | `nt_core_e8/nt_core_synthesis.rs:498-549` | MEDIUM | StepRouteCache uses FIFO (push to tail, evict from head) but the `get` method does move-to-tail for recency refresh (line 544-546). However, the move is O(n) via `remove(p)` on a Vec. This is a Vec-based LRU with O(n) per access. Under SEAL loop execution with 8+ phases, route cache lookups per step become quadratic. Quick-cache's S3-FIFO or moka's TinyLFU would provide O(1) amortized with better hit ratios. | Source: quick-cache S3-FIFO scan-resistant; moka TinyLFU admission

**D-CACHE-007: No eviction listener for cache consistency** | `nt_core_cache.rs:70-80`, `response_cache.rs:78-97` | MEDIUM | Moka provides eviction listeners (callbacks on removal with reason: size/expire/invalidate/explicit). NeoTrix caches silently drop entries. When SemanticCache evicts an embedding entry, no signal propagates to invalidate the corresponding exact entry or notify downstream consumers (e.g., GWT attention routing). This can cause stale semantic matches to surface after the exact entry is gone. | Source: moka eviction listener; cachet lifecycle hooks

**D-CACHE-008: `ResponseCache` pin mechanism can starve eviction** | `response_cache.rs:85-96` | LOW | If `MAX_PINNED` (32) entries are all pinned and cache capacity is 256, only 224 entries are evictable. If hot expert responses cluster in the 32 pinned set, cold entries cannot push them out. Moka's pinning uses weight-based admission (pinned items still consume weight budget). Quick-cache's pinning allows weight-zero items. NeoTrix pin is a simple HashSet — pinned entries consume full weight. | Source: quick-cache item pinning with weight zero; moka weight-aware admission

**D-CACHE-009: No size-aware eviction (bytes-based)** | `nt_core_cache.rs:54`, `response_cache.rs:10` | LOW | Both `SemanticCache` and `ResponseCache` evict by count only, not by memory size. LLM responses vary from 10 bytes to 100KB+. A cache with 256 entries of 100KB responses consumes 25MB — unbounded from memory perspective. Moka supports `weighted_size` for size-aware eviction. TileCache correctly tracks `total_bytes` but other caches do not. | Source: moka weighted_size eviction; TileCache `max_bytes` (correct pattern)

**D-CACHE-010: No stale-while-revalidate support** | `response_cache.rs:65-74` | LOW | When a cached LLM response expires (if TTL existed), the caller must wait for a full re-fetch. Autumn's `stale_while_revalidate` serves the last-known-good value while refreshing in background. For NeoTrix consciousness tasks where latency matters, serving stale cached routing decisions (StepRouteCache) during re-computation would improve perceived responsiveness. | Source: Autumn stale-while-revalidate; fusioncache-rs background execution

**D-CACHE-011: Semantic cache hash collision risk with `DefaultHasher`** | `nt_core_cache.rs:162-169` | LOW | `hash_embedding` uses `DefaultHasher` which is SipHash-1-3 (not collision-resistant). With thousands of embedding entries, u64 hash collisions silently overwrite entries. Quick-cache handles collisions via key comparison in shards. Moka uses lock-free hash table with proper bucket handling. NeoTrix silently replaces on collision (`HashMap::insert`). | Source: quick-cache shard-level collision handling

## Key Insights

1. **All 6 NeoTrix caches are single-threaded `HashMap`** — no cache uses `moka::future::Cache`, `quick-cache`, or any concurrent data structure. Under the `Mutex<SemanticCache>` at gateway level, all LLM requests serialize through a single lock.

2. **Zero cache stampede protection** — the most critical gap. When a hot semantic key expires under concurrent agent dispatch (e.g., 4 parallel consciousness tasks), every task independently recomputes cosine similarity. Moka's `get_with` and cachet's `stampede_protection()` solve this with single-flight coalescing.

3. **No TTL/TTI on 2 of 5 caches** — `ResponseCache` and `McpResultCache` never expire entries. Stale LLM responses and tool results persist until LRU eviction. This is dangerous for model-version-sensitive caching.

4. **All eviction algorithms are O(n)** — `TileCache::evict_if_needed` scans all entries, `McpResultCache::get` calls `VecDeque::retain`, `StepRouteCache::get` calls `Vec::remove`. Moka/quick-cache provide O(1) amortized via sharded structures.

5. **No cross-cache invalidation** — SemanticCache exact tier and embedding tier are independent. Evicting an embedding doesn't invalidate the exact entry. No eviction listeners exist to propagate invalidation to ResponseCache or StepRouteCache.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 11 |
| CRITICAL | 1 |
| HIGH | 2 |
| MEDIUM | 4 |
| LOW | 4 |
| Sources analyzed | 8 |
