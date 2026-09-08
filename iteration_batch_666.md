# Iteration Batch 666 — Cache Architecture Defects & Improvements

**Date**: 2026-09-06
**Triggered by**: Batch 665 findings (TCP HOL, no 0-RTT, no BBR, no CDN, QUIC blind spot)
**Research domain**: Caching patterns, eviction algorithms, cache invalidation strategies (2026 state-of-art)

---

## Sources

1. Redis 8 / 8.6 / 8.8 release notes & caching guides (redis.io, anhtu.dev, upstash.com, ilirivezaj.com, minervadb.com)
2. "Which Eviction Policy Should an LLM Cache Use?" — arxiv 2608.20280 (CLEVER benchmark)
3. "Demystifying and Improving Lazy Promotion in Cache Eviction" — VLDB 2026, arxiv 2608.29993
4. "Comparative Evaluation of Cache Replacement Algorithms for Time-Bounded Stale-Tolerant Data Workloads" — IJSRT 2026
5. "Cache Eviction Algorithms: LRU vs LFU vs S3-FIFO" — theinfinity.dev 2026-08
6. "LRU Cache Design: Eviction Strategies and Trade-offs" — sujeet.pro 2026-02
7. Redis blog: Cache Consistency Strategies 2026
8. "Cache Invalidation: The Hard Problem, Honestly" — bubble.ro 2026-06
9. "Complete Guide to Cache Invalidation" — stackpractices.com 2026-07
10. "Consistency and Invalidation for Hybrid Edge Caches" — caching.website 2026-01
11. "Distributed Caching in 2026" — amtocsoft.blogspot.com 2026-04
12. "Cache invalidation is the dual-write problem" — dev.to 2026-07
13. Linux MGLRU kernel docs (Multi-Gen LRU)
14. S3-FIFO (Yang et al., SOSP '23)

---

## NEW Defects Found

### D666-01: NT-MEMORY uses naive LRU — catastrophically vulnerable to scan pollution

**Evidence**: Every tested benchmark shows LRU degrades to 0% hit rate under sequential/cyclic scan (S3-FIFO benchmark: LRU 0% vs S3-FIFO 88% at 90% cache size; arxiv 2608.20280 confirms LRU trails LFU by up to 8.67pp on LLM semantic caches). Linux kernel abandoned vanilla LRU for MGLRU. Redis approximates it with random sampling (90% accuracy, 10% overhead). NT-MEMORY's KB cache uses no scan resistance at all.

**Impact**: Any batch scan, analytics query, or crawl sweep through KB nodes evicts the entire hot working set. Single crawl of 10K nodes destroys all cached embeddings.

**Fix**: Adopt 2Q or S3-FIFO as default eviction policy. S3-FIFO is 70 lines of Python, lock-free on read path, and outperforms LRU by 7.6-10.9pp on mixed workloads. For KB specifically, the probation queue (10% capacity) filters one-shot scan entries before they pollute the main cache.

### D666-02: No probabilistic early expiration (XFetch) — cache stampede risk on KB node expiry

**Evidence**: Redis 8.x consensus (redis.io, upstash.com, ilirivezaj.com) is to layer XFetch + single-flight lock. Without XFetch, a hot key expiry at 10K req/s with 300ms recompute leaks ~3000 queries to SQLite simultaneously. Redis 8.4's `SET ... IFEQ` / `DELEX` make CAS native, eliminating Lua scripts. NT-MEMORY has zero stampede protection.

**Impact**: KB node expiry (e.g., embedding cache for frequently-queried concepts) creates thundering herd on SQLite, which is single-writer and will serialize all requests.

**Fix**: Implement XFetch probabilistic early expiration with `beta` tuned to KB recompute cost. Layer single-flight lock using Redis 8.4+ native CAS commands. Add TTL jitter (`TTL + random(-30s, 30s)`) as baseline defense.

### D666-03: No LRM (Least Recently Modified) eviction — stale embeddings never evicted

**Evidence**: Redis 8.6 introduced `volatile-lrm` and `allkeys-lrm` policies. LRM evicts based on write time, not read time. For AI/semantic caches, embeddings are read constantly but never modified — under LRU they never get evicted, consuming memory indefinitely. NT-MEMORY stores VSA embeddings that are write-once-read-many.

**Impact**: Embedding cache fills with cold write-once entries that are never evicted because they're still "recently used" (read). Effective cache capacity shrinks over time.

**Fix**: Use `allkeys-lrm` for embedding caches or implement LRM at the application layer with a "last modified" timestamp per KB node.

### D666-04: No null/negative result caching — cache penetration attacks on non-existent nodes

**Evidence**: Redis caching guide (ilirivezaj.com) and Upstash guide both identify this as critical: "Repeated lookups for a key that doesn't exist in the database become a penetration attack vector — every request hits the DB." Cache negative results with short TTL (30-60s). Redis 8 built-in Bloom Filter provides O(1) membership check. NT-MEMORY has no negative cache.

**Impact**: Queries for non-existent KB nodes (common during exploration) bypass cache entirely, hitting SQLite every time. At scale, this is a denial-of-service vector.

**Fix**: Cache null/miss results with 30-60s TTL. Consider Redis 8 Bloom Filter for probabilistic negative lookup (zero false negatives, tunable false positives).

### D666-05: No hot-key detection or mitigation — single-node bottlenecks invisible

**Evidence**: Redis 8.6 introduced `HOTKEYS` command with CPU/network consumption tracking per slot. Redis 8.2 added `CLUSTER SLOT-STATS`. MinervaDB documents that whale tenants concentrate on single shards with no visibility. NT-MEMORY has no mechanism to detect which KB nodes are hot.

**Impact**: Frequently-accessed KB nodes (e.g., core concept embeddings) bottleneck a single SQLite page cache slot, degrading all concurrent queries.

**Fix**: Implement access frequency tracking per KB node. Use `HOTKEYS`-style sampling or lightweight counter. Evict or replicate hot nodes.

### D666-06: Cache invalidation is a dual-write problem — no transactional outbox for cross-tier consistency

**Evidence**: dev.to 2026-07 explicitly frames cache invalidation as the dual-write problem: "A database and a cache cannot commit atomically." Rails 8 and Laravel 11 default to DB-backed cache but don't push invalidations to in-memory tiers. NT-MEMORY writes KB and evicts cache in separate operations with no atomicity guarantee.

**Impact**: KB update + cache eviction can race: cache is evicted but KB write fails, leaving cache empty (cold start penalty); or KB write succeeds but cache eviction is lost, leaving stale data indefinitely.

**Fix**: Use transactional outbox pattern: write invalidation event inside the KB transaction, process asynchronously via CDC or polling. Implement delayed double-delete to handle concurrent repopulation races.

### D666-07: No stale-while-revalidate — cache misses block on slow recompute

**Evidence**: AmtocSoft 2026 documents `stale-while-revalidate` as "the single most impactful directive for perceived latency — a user never waits for a cache refresh." bubble.ro 2026 confirms stale-while-revalidate eliminates user-visible latency spikes but requires background recomputation. NT-MEMORY serves nothing on miss — full synchronous recompute.

**Impact**: Every cache miss for KB nodes blocks the calling thread during SQLite query + embedding computation. Under load, this creates cascading latency.

**Fix**: Serve stale cached value immediately, trigger async background recompute. Return fresh value on next request. Requires versioning cache entries with generation counter.

### D666-08: No versioned cache keys — schema changes require full cache flush

**Evidence**: stackpractices.com 2026 and amtocsoft.blogspot.com 2026 both document versioned keys: include version in cache key (`product:42:v7`), increment on schema change, old keys expire naturally. NT-MEMORY uses flat node IDs as cache keys with no versioning.

**Impact**: Any change to embedding format, quantization, or KB schema requires manual full cache flush or silent serving of incompatible data.

**Fix**: Prefix cache keys with schema version hash. On format change, increment version; old entries expire via TTL naturally.

### D666-09: No multi-tier cache coherence — L1 (in-process) and L2 (KB) drift independently

**Evidence**: amtocsoft.blogspot.com documents that when L2 (Redis) is invalidated, L1 (in-process maps) across N instances still hold old values. Options: short L1 TTL (10-30s), explicit L1 invalidation via pub/sub, or accept divergence. For auth/permission data, skip L1 entirely. NT-MEMORY's in-process cache and KB have no coherence mechanism.

**Impact**: Multi-agent sessions see different KB node values depending on which process cached first. No pub/sub invalidation channel exists.

**Fix**: Implement keyspace notifications for KB writes. L1 entries get 10-30s TTL for non-critical data, zero TTL for consistency-critical data.

### D666-10: ARC patent expired Feb 2024 — adaptive replacement now free to implement

**Evidence**: arxiv 2608.20280 confirms ARC = LFU within 0.041pp across all LLM cache benchmarks. Megiddo & Modha (USENIX FAST 2003) patent expired February 2024. ARC self-tunes recency/frequency balance via ghost lists. IJSRT 2026 shows ARC highest mean hit ratio across all stale-tolerant workloads tested.

**Impact**: NT-MEMORY has been using static LRU with no adaptation. ARC is now patent-free and outperforms LRU by 1-3pp consistently with zero tuning parameters.

**Fix**: Implement ARC with semantic ghost lists (adapted for KB node embeddings). Ghost list hit adapts T1/T2 split automatically.

### D666-11: No CDN/edge cache for KB public assets — zero geographic distribution

**Evidence**: Batch 665 identified zero CDN. amtocsoft confirms CDN eliminates origin hits for cacheable HTTP responses with <5ms edge latency. Surrogate keys (cache tags) enable single-call purge across all edge nodes. CDN Origin Shield coalesces 250 edge misses into 1 origin request. NT-MEMORY serves all KB reads from origin.

**Impact**: Every KB query hits origin server regardless of user geography. No edge caching for public concept definitions, documentation, or static assets.

**Fix**: Implement CDN with Cache-Control headers (`s-maxage`, `stale-while-revalidate`, `stale-if-error`). Use surrogate keys for tag-based purge on KB writes.

### D666-12: No read-your-writes bypass — users see stale data after their own writes

**Evidence**: Redis 2026 consistency blog and AmtocSoft both document "read-your-writes bypass": after a write, mark session as "recently wrote" with 5-10s TTL, force subsequent reads to origin. Uber CacheFront implements this as write-through validation. NT-MEMORY has no mechanism.

**Impact**: User updates a KB node, immediately queries it, sees old value. Confidence-breaking.

**Fix**: On write, set session-scoped flag. For next 5-10s, bypass cache for that node's reads.

---

## Improvements Identified

### I666-01: Redis 8.8 MGET prefetch — 68% throughput gain available

Redis 8.8 added batched memory prefetch on MGET/MSET paths. Pipelined bulk string reads gain up to 68% throughput. NT-MEMORY's batch KB lookups should adopt pipelined MGET with io-threads enabled.

### I666-02: Redis 8.8 Array data structure — new primitive for ordered KB node collections

Redis 8.8 introduces `array` — an index-addressable, sparse-friendly, compute-aware container. Could replace sorted-set-based ordered collections for KB node ordering with lower overhead.

### I666-03: Redis 8.8 window counter rate limiter — built-in throttling

Redis 8.8 native window counter rate limiter replaces Lua scripts. Useful for throttling KB query rate from agents without custom implementation.

### I666-04: Proactive cache refresh — 2026 trend for hot paths

Instead of reactive cache-aside (miss → DB → populate), proactively refresh cache before expiry. User never experiences miss or stale data. Applicable to frequently-accessed KB concept embeddings.

### I666-05: Per-field hash expiration — collapse class of workarounds

Redis 8 `HEXPIRE`/`HGETEX`/`HGETDEL` let individual hash fields expire independently. Previously required one key per field or full hash rebuild. Applicable to KB node metadata stored as Redis hashes.

### I666-06: Delay-LRU and FIFO-reinsertion — 20-60% fewer promotions

VLDB 2026 paper (arxiv 2608.29993) shows Delay-LRU and FIFO-reinsertion reduce promotions by 20-60% while achieving similar or lower miss ratio. Directly applicable to NT-MEMORY's LRU implementation if LRU is retained.

### I666-07: S3-FIFO probation queue — 7.6-10.9pp hit rate improvement over LRU

SOSP '23 S3-FIFO uses 3 FIFO queues with no linked-list surgery on hits. Small queue (10% capacity) filters one-hit wonders. Outperforms LRU by 7.6-10.9pp on mixed workloads. Within 1.4pp of offline optimum (Belady's MIN).

### I666-08: Bloom Filter for cache penetration defense — zero false negatives

Redis 8 built-in Bloom Filter provides O(1) membership check for negative lookups. Prevents cache penetration attacks without storing null values.

---

## Summary

| Category | Count |
|----------|-------|
| New Defects | 12 |
| Improvements | 8 |
| Sources | 14 |

**Critical path**: D666-01 (scan pollution) + D666-02 (stampede) + D666-06 (dual-write) are the highest-impact defects. S3-FIFO adoption (I666-07) + XFetch (D666-02 fix) + transactional outbox (D666-06 fix) form the minimum viable fix set.
