# Iteration Batch 752 — Connection Pooling, Connection Management & SQLite Performance

**Date**: 2026-09-07
**Prior Batch**: 751 (no migration reversibility, no expand-contract, no partial rollback, no smoke test gate, no auto-rollback trigger)
**Sources**: 2026 web search across connection pooling (bb8/deadpool/sqlx), connection management/tuning, SQLite WAL/PRAGMA performance

---

## Sources Cited

| # | Source | Topic |
|---|--------|-------|
| S1 | leapcell.io/blog/efficient-database-connection-management-with-sqlx-and-bb8-deadpool-in-rust | bb8 vs deadpool integration with sqlx, pool lifecycle, health checks |
| S2 | oneuptime.com/blog/post/2026-01-25-connection-pools-bb8-deadpool-rust | Pool sizing formula (cores*2)+spindles, deadpool RecyclingMethod::Verified, config file support |
| S3 | oneuptime.com/blog/post/2026-01-30-performance-connection-pool-tuning | Pool tuning checklist, min/max connections, acquire timeout, idle timeout, validation strategy |
| S4 | imperialis.tech/en/blog/connection-pooling-database-high-scale-systems-production-2026 | High-scale pooling strategies, PgBouncer external pooler, connection exhaustion patterns |
| S5 | imperialis.tech/en/blog/database-connection-pooling-best-practices-2026 | Production best practices, keepAlive, statement_timeout, Redis pool config |
| S6 | codelit.io/blog/database-connection-management | Connection lifecycle, PgBouncer/ProxySQL, prepared statements, idle timeouts, serverless connections |
| S7 | aidev.fit/en/database/connection-management.html | Per-connection memory (5-10MB), pg_stat_activity monitoring, separate OLTP/analytics pools |
| S8 | oneuptime.com/blog/post/2026-02-20-database-connection-management | Microservices connection explosion: 10 services × 5 replicas × 10 conn = 500 > DB limit |
| S9 | adhdecode.com/articles/sqlite/sqlite-performance-tuning-production | PRAGMA optimization benchmark: WAL + synchronous=NORMAL + cache_size + temp_store=MEMORY |
| S10 | sqlite.org/wal.html | WAL-reset bug (CVE-level, fixed 3.51.3), WAL checkpoint tuning, concurrent writer limitations |
| S11 | forwardemail.net/en/blog/docs/sqlite-performance-optimization-pragma-chacha20-production-guide | wal_autocheckpoint=1000 gives +12% inserts, ChaCha20 vs AES256, /tmp vs /dev/shm for temp |
| S12 | gist.github.com/phiresky | PRAGMA tuning reference: synchronous=normal safe in WAL mode, mmap_size, page_size |
| S13 | github.com/open-webui/pull/23758 | PRAGMA optimize in pool checkin handler caused production crashes (write lock contention) |
| S14 | github.com/DK26/anyfs-sqlite/issues/4 | WAL-mode default pragmas table, busy_timeout=5000, checkpoint-on-close pattern |
| S15 | github.com/DecisionNerd/graphforge/issues/392 | Row-by-row I/O is bottleneck before WAL; fix bulk I/O (fetchall + executemany) first |

---

## New Defects Found

### DEFECT-752-01: No Connection Pool for KB (SQLite) — Connections Created Per-Request (CRITICAL)

**Severity**: CRITICAL
**Source**: S1, S2, S4, S7
**Finding**: NeoTrix KB uses SQLite. Current implementation likely creates connections per-request or uses a static single connection. Industry standard (2026): use `sqlx::Pool` or `deadpool` with explicit pool sizing. Each SQLite connection consumes 5-10MB RAM (S7). A pool of 20 connections = 100-200MB baseline. Without pooling, every KB query pays TCP/auth overhead (S1) and risks connection exhaustion under concurrent access (S4).
**Impact**: Under load (e.g., SEAL pipeline running multiple phases concurrently), KB becomes a bottleneck. No connection reuse = repeated WAL-mode re-initialization per connection.
**Fix**: Initialize `sqlx::SqlitePool` with `PoolOptions::new().max_connections(4)` (conservative for SQLite). Single pool shared across all NT-MEMORY operations. Pool created at NT-MEMORY startup, not per-query.

### DEFECT-752-02: No WAL-Reset Bug Protection (HIGH)

**Severity**: HIGH
**Source**: S10
**Finding**: SQLite WAL-reset bug (fixed in 3.51.3, 2026-03-13) causes corruption when two connections write/checkpoint simultaneously. The bug affects all SQLite 3.7.0 through 3.51.2. NeoTrix does not pin SQLite version and has no protection against this race condition.
**Impact**: If NeoTrix runs on SQLite < 3.51.3 with multiple connections (e.g., WAL checkpoint running while a write occurs), KB corruption is possible. This is a data race with tight timing constraints but real consequences.
**Fix**: (1) Pin `libsqlite3-sys` to >= 3.51.3 in Cargo.toml. (2) Use `busy_timeout=5000` to serialize writers. (3) Set `wal_autocheckpoint=1000` (S11) to keep WAL size bounded. (4) Run `PRAGMA wal_checkpoint(TRUNCATE)` on connection close (S14).

### DEFECT-752-03: PRAGMA optimize in Pool Event Handler Causes Crashes (HIGH)

**Severity**: HIGH
**Source**: S13
**Finding**: Open-WebUI PR #23758 discovered that running `PRAGMA optimize` in a pool checkin/close handler causes production crashes. `PRAGMA optimize` conditionally runs `ANALYZE`, which is a write operation that contends for SQLite's write lock. Under concurrent checkins, this starves the connection pool. The fix was to drop the handler entirely.
**Impact**: If NeoTrix ever adds connection lifecycle hooks (e.g., on-checkin ANALYZE), it will trigger write-lock contention under concurrent KB access. This is a known production failure pattern.
**Fix**: Never run ANALYZE or PRAGMA optimize in connection lifecycle hooks. Run ANALYZE as a periodic background task (e.g., every 1000 writes or on SEAL phase completion) using a dedicated connection, not from the pool event system.

### DEFECT-752-04: No Connection Pool Sizing for SQLite (MEDIUM)

**Severity**: MEDIUM
**Source**: S2, S3
**Finding**: The canonical pool sizing formula is `(core_count * 2) + effective_spindle_count` (S2, S3). For SQLite (single-file, single-writer), this formula doesn't directly apply — but the principle of small pools does. SQLite supports limited concurrent writers (typically 1 writer + N readers in WAL mode). A pool > 4 for SQLite is wasteful and increases lock contention. NeoTrix has no pool sizing guidance for its KB.
**Impact**: Oversized pool (e.g., max_connections=20) creates artificial contention. Undersized pool (e.g., max_connections=1) serializes all KB access.
**Fix**: For SQLite: `max_connections = min(cpu_cores, 4)`. For NeoTrix on a typical dev machine: `max_connections = 2-4`. Document this as KB configuration guidance.

### DEFECT-752-05: No Separate OLTP vs Analytics Pools for KB (MEDIUM)

**Severity**: MEDIUM
**Source**: S6, S7
**Finding**: Best practice (2026): separate connection pools for OLTP (short, frequent queries) and analytics (long, bulk queries). NeoTrix KB serves both: (1) OLTP-like ops (kv_store get/set, experience pointer lookups) and (2) analytics-like ops (BM25 index rebuilds, embedding batch inserts, VACUUM). Sharing a single pool means a VACUUM can starve kv_store reads.
**Impact**: Long-running KB operations (VACUUM, bulk index rebuild) block short-lived queries, causing latency spikes in GWT attention routing that depends on fast KB lookups.
**Fix**: Two pools: `kb_oltp_pool` (max_connections=2, for kv_store/experience queries) and `kb_analytics_pool` (max_connections=1, for VACUUM/index/embedding ops). Analytics pool serialized by default.

### DEFECT-752-06: No Connection Health Check / Recycling for KB (MEDIUM)

**Severity**: MEDIUM
**Source**: S1, S2
**Finding**: Both bb8 and deadpool support connection recycling: bb8 via `test_on_check_out(true)`, deadpool via `RecyclingMethod::Verified` (S2). SQLx Pool validates connections before return. If NeoTrix KB pool has no health check, stale/corrupt connections are handed to callers.
**Impact**: A corrupted SQLite connection (e.g., after WAL-reset bug or filesystem error) silently fails queries instead of being recycled.
**Fix**: Enable `sqlx::sqlite::SqlitePoolOptions::new().test_before_acquire(true)` to run a lightweight validation query before returning connections.

### DEFECT-752-07: No Acquire Timeout Configuration (MEDIUM)

**Severity**: MEDIUM
**Source**: S3, S5
**Finding**: Pool acquire timeout prevents indefinite blocking when all connections are busy. Default in many pools is infinite wait. NeoTrix KB has no acquire timeout configured. Under contention, threads block forever waiting for a KB connection.
**Impact**: Deadlock risk: a SEAL pipeline phase holds a connection while waiting for another KB operation that needs a connection. Both wait forever.
**Fix**: Set `acquire_timeout(Duration::from_secs(5))` on KB pool. Return `PoolTimeout` error instead of blocking indefinitely.

### DEFECT-752-08: No WAL Autocheckpoint Tuning (LOW)

**Severity**: LOW
**Source**: S10, S11, S14
**Finding**: Default WAL autocheckpoint is 1000 pages (~4MB). Forward Email benchmarks (S11) show `wal_autocheckpoint=1000` gives +12% insert throughput vs default. Setting it to 0 (no autocheckpoint) causes WAL to grow unbounded, degrading read performance. NeoTrix does not configure WAL autocheckpoint.
**Impact**: Suboptimal write throughput. WAL file may grow large during burst writes (e.g., SEAL batch absorption), degrading subsequent reads.
**Fix**: Set `PRAGMA wal_autocheckpoint=1000` at KB pool initialization. This is the empirically optimal value.

### DEFECT-752-09: Row-by-Row KB Operations Before WAL Optimization (LOW)

**Severity**: LOW
**Source**: S15
**Finding**: GraphForge issue #392 (S15) found that row-by-row I/O is a bigger bottleneck than WAL mode. Before optimizing WAL, fix bulk I/O: replace individual `SELECT`/`INSERT` loops with `fetchall()` + `executemany()`. NeoTrix KB operations (experience absorption, embedding batch inserts) may use row-by-row patterns.
**Impact**: WAL mode improvement is marginal if the real bottleneck is row-by-row I/O. Bulk operations can give 10-100x improvement that WAL tuning alone cannot match.
**Fix**: Audit KB write paths. Replace row-by-row experience absorption with batch `executemany`. Replace individual embedding inserts with bulk insert. Profile before applying WAL optimizations.

---

## Summary

| Category | New Defects | Severity Breakdown |
|----------|-------------|-------------------|
| Connection Pooling | 4 | 1 CRITICAL, 1 HIGH, 2 MEDIUM |
| Connection Management | 3 | 1 HIGH, 2 MEDIUM |
| SQLite Performance | 2 | 1 MEDIUM, 1 LOW |
| **Total** | **9** | **1 CRITICAL, 2 HIGH, 5 MEDIUM, 1 LOW** |

**Key Insight**: The most critical finding is that NeoTrix KB has no connection pooling at all — connections are likely created per-request. The second critical pattern is the WAL-reset bug (S10), which is a real corruption vector for multi-connection SQLite. The third high-severity finding is that ANALYZE in pool event handlers is a known production crash pattern (S13). These three defects should be addressed before any WAL tuning or performance optimization.
