# Agent 2: Hyper Connection Pooling (Batch 859)

## Sources
- https://github.com/hyperium/hyper/pull/2991 — TCP keepalive interval/retries configuration
- https://github.com/hyperium/hyper/issues/2375 — HTTP/2 multiplexing with low-level client API
- https://github.com/hyperium/hyper/issues/2063 — Hyper's pool opens only one TCP connection for HTTP/2 (singleton problem)
- https://github.com/hyperium/hyper/issues/2420 — Idle connections never cleaned up (file descriptor leak)
- https://github.com/hyperium/hyper/issues/3785 — Pool grows unbounded across hosts (no global limit)
- https://github.com/hyperium/hyper/issues/3685 — Idle connections accumulate forever (memory leak)
- https://seanmonstar.com/blog/hyper-util-composable-pools/ — Composable pool layers (cache/singleton/negotiate/map)
- https://github.com/seanmonstar/reqwest/pull/2434 — Pool timer not passed, idle cleanup silently broken
- https://github.com/seanmonstar/reqwest/issues/3033 — No max connection lifetime (DNS stale after k8s scale-up)
- https://github.com/seanmonstar/reqwest/issues/2424 — No global pool size limit
- https://arcmutex.com/content/reqwest-connection-pooling-performance — Pool configuration best practices
- https://github.com/hyperium/hyper/discussions/2628 — pool_max_idle_per_host(0) disables pooling entirely
- Hyper source: `hyper_util::client::legacy::pool.rs` — PoolInner with idle HashMap, connecting HashSet, waiters
- Crawlex source: `crawlex/impersonate/pool.rs` — Single-slot H2 pool pattern with DashMap

## Defects

### D-POOL-001: NT-WORLD fetchers bypass central HTTP factory, each creates isolated per-fetcher reqwest::Client
**Files**: `nt_world_edgar.rs:288`, `nt_world_adsb.rs:101`, `nt_world_opencorporates.rs:95`, `nt_world_polymarket.rs:85`, `nt_world_search.rs:177`, `nt_world_ofac.rs:108`, `nt_world_gdacs.rs:115`, `nt_world_usgs.rs:255`, `nt_world_urlhaus.rs:124`, `nt_world_bgpview.rs:94`, `nt_world_aoi.rs:162`, `nt_world_ucdp.rs:164`
**Severity**: HIGH
**Source**: Codebase analysis + hyper pool internals (hyper issue #2063)
**Description**: Each NT-WORLD fetcher lazily initializes its own `reqwest::blocking::Client` via `OnceLock`, bypassing the central `nt_io_http_factory`. With 12+ fetchers, this creates 12+ independent connection pools (each with up to 32 idle connections per host = 384+ potential idle sockets per host). The hyper pool key is `(scheme, host, port)` — each isolated pool duplicates this namespace. Worse, when the same host is fetched by EdgarFetcher + PolymarketFetcher, they cannot share idle connections. This defeats connection reuse, wastes TLS session tickets, and multiplies idle-timeout cleanup overhead. The central `nt_io_http_factory::global_client()` exists but is not wired to these fetchers.

### D-POOL-002: StealthHttpClient proxy pool reuses `reqwest::Client` from a 9-second TTL map, but reqwest Client wraps Arc<hyper::Client> — pool lifecycle mismatch
**File**: `nt_shield_stealth_net/http_client/pool.rs:79-106`
**Severity**: MEDIUM
**Source**: Hyper pool internals (pool.rs Pooled<T,K> Drop impl)
**Description**: `get_or_create_client` retains entries for only 9 seconds (`Duration::from_secs(9)`), then drops old clients. However, each `reqwest::Client` holds an `Arc` over hyper's `ClientInner` which contains the `Pool<T,K>` with its own `IdleTask` background reaper. Dropping the reqwest::Client clones the pool ref but the underlying hyper pool's `IdleTask` only runs if `pool_timer` is provided. When a client is created without explicitly configuring `pool_timer`, idle connections may never be reaped (hyper-util issue #2434). The 9-second client TTL is arbitrary — shorter than the `pool_idle_timeout` (90s from config), meaning the reqwest::Client wrapper is dropped while the underlying hyper pool may still reference idle connections. This creates a brief window of orphaned connections that neither the wrapper TTL nor the pool idle timeout covers.

### D-POOL-003: `fetch_safe_http_inner` and `download_to_file_inner` create ephemeral per-request reqwest::blocking::Client — zero connection reuse for SSRF-pinned fetches
**File**: `nt_memory_kb/nt_http.rs:191-214` and `nt_memory_kb/nt_http.rs:448-466`
**Severity**: HIGH
**Source**: Hyper pool internals (pool.rs — connection racing, idle reuse)
**Description**: Every call to `fetch_safe_http` and `download_to_file` builds a fresh `reqwest::blocking::Client` with `.resolve(&host, addr)` to pin DNS. Since `resolve()` is per-client, the shared `shared_blocking_client()` singleton cannot be used. Each new client creates its own hyper PoolInner. This means: (1) TCP handshake + TLS negotiation on every fetch — no connection reuse across sequential requests to the same host; (2) TLS session tickets are wasted; (3) Each client's `IdleTask` is spawned but has no timer (blocking client path doesn't set `pool_timer`), so idle connections from these ephemeral clients may leak until the client is dropped. The same pattern appears in `resolve_redirects_safely` at line 134. For batch crawl operations hitting the same hosts repeatedly, this is a significant performance and resource penalty.

### D-POOL-004: `nt_io_http_factory` and `nt_http` define conflicting pool constants — split-brain configuration
**Files**: `nt_io_http_factory.rs:46-50` vs `nt_memory_kb/nt_http.rs:17-22`
**Severity**: MEDIUM
**Source**: Hyper pool config (hyper client::Builder)
**Description**: `nt_io_http_factory` defines `POOL_MAX_IDLE_PER_HOST=32`, `POOL_IDLE_TIMEOUT_SECS=90`, `TCP_KEEPALIVE_SECS=15`, `CONNECT_TIMEOUT_SECS=10`, `REQUEST_TIMEOUT_SECS=60`. `nt_http` defines `TIMEOUT=30s`, `CONNECT_TIMEOUT=15s` with no pool constants at all (blocking client has no `pool_max_idle_per_host`, `pool_idle_timeout`, or `tcp_keepalive`). The `shared_blocking_client()` at nt_http.rs:44 is the primary SSRF-safe fetch path but uses reqwest defaults (60s idle timeout, unlimited idle per host, no keepalive). This means the SSRF-safe path has different pool characteristics than the factory path — 3x shorter connect timeout but no keepalive probes, and unlimited idle connections per host (vs 32 in factory). When both are active in the same process, the system has two different pool behaviors for the same hosts.

### D-POOL-005: No global pool size cap across all hosts — unbounded memory growth in long-running processes
**File**: `nt_io_http_factory.rs:46` (`POOL_MAX_IDLE_PER_HOST=32`)
**Severity**: MEDIUM
**Source**: Hyper issue #3785 (unbounded pool growth)
**Description**: The hyper-util pool uses `idle: HashMap<Key, Vec<Idle<T>>>` — one key per (scheme, host, port). `pool_max_idle_per_host=32` limits per-host but not global. NeoTrix crawls many different hosts via NT-WORLD (sec.gov, adsb.lol, opencorporates.com, polymarket.com, usgs.gov, bgpview.io, urlhaus.abuse.ch, etc.). Each unique host can hold 32 idle connections. In a long-running crawl session visiting 100 unique hosts, that's 3200 idle connections, each holding a TCP socket + TLS session state. The hyper-util pool has no LRU eviction across hosts (hyper issue #3785). This leads to file descriptor exhaustion on systems with default ulimit (1024). The fix requires either an LRU cache for the pool's `idle` map or a global max-connections semaphore.

### D-POOL-006: `nt_http.rs` blocking client has no `pool_idle_timeout` — idle connections accumulate until process death
**File**: `nt_memory_kb/nt_http.rs:45-58`
**Severity**: MEDIUM
**Source**: Hyper issue #2420 (idle connections never cleaned up)
**Description**: `shared_blocking_client()` uses `reqwest::blocking::Client::builder()` without calling `.pool_idle_timeout()`. Reqwest defaults to 90 seconds for pool_idle_timeout, but critically the blocking client path may not spawn the `IdleTask` reaper (hyper-util issue #2434 shows the timer must be explicitly passed). If the blocking client's pool_timer is not set by default, the idle timeout check only runs when a new connection is checked out from the pool, not proactively. This means if a host is accessed once and never again, its idle connections persist indefinitely in memory. Combined with D-POOL-005 (unbounded per-host), this is a slow memory leak in any long-running NeoTrix process that makes HTTP requests to many transient hosts.

### D-POOL-007: `nt_world_browse/session.rs` creates a bare client with no pool config, no connect_timeout, and danger_accept_invalid_certs=true
**File**: `nt_world_browse/session.rs:130-133`
**Severity**: MEDIUM
**Source**: Hyper pool + TLS config
**Description**: `DumpDomFetcher::fetch_http` creates `reqwest::blocking::Client::builder().danger_accept_invalid_certs(true).build()` with zero pool tuning, zero timeout, and no connect_timeout. This means: (1) No idle timeout — connections persist forever if the process runs; (2) No keepalive — stale connections are never detected; (3) No connect_timeout — a hanging TLS handshake blocks the thread indefinitely; (4) No pool limit — every `fetch_http` call to a different host adds to the pool with no eviction. This is the worst client configuration in the codebase — it bypasses both the http_factory and nt_http patterns entirely. Combined with `danger_accept_invalid_certs(true)`, this is also a TLS security gap (MITM vulnerability for the browse path).

### D-POOL-008: HTTP/2 singleton pool problem — NeoTrix crawlers may serialize on a single H2 connection without knowing it
**File**: `nt_io_http_factory.rs:56` (TlsVariant::ModernH2 enables h2)
**Severity**: LOW
**Source**: Hyper issue #2063 (HTTP/2 multiplexing limits)
**Description**: When `TlsVariant::ModernH2` is used (the default for `global_client()`), reqwest negotiates HTTP/2 via ALPN. Hyper's pool uses a singleton pattern for HTTP/2 connections per host — only one TCP connection is opened, and all requests are multiplexed over it. This is efficient for low-latency scenarios but becomes a bottleneck when: (1) The single connection hits TCP head-of-line blocking under packet loss; (2) The server's `SETTINGS_MAX_CONCURRENT_STREAMS` limits simultaneous streams (common default: 100); (3) High-variance latency requests (some <1ms, some >60s) block the entire H2 connection. NT-WORLD crawls with high latency variance (geo APIs, EDGAR, OSINT) may silently serialize. NeoTrix has no mechanism to detect H2 congestion and fall back to multiple HTTP/1.1 connections.

### D-POOL-009: `tor_client()` in nt_io_http_factory creates client without pool_max_idle_per_host or pool_idle_timeout
**File**: `nt_io_http_factory.rs:166-174`
**Severity**: LOW
**Source**: Hyper pool internals
**Description**: `tor_client()` builds a reqwest::Client for SOCKS5 Tor proxy with `.https_only(false)` but no pool configuration (no `pool_max_idle_per_host`, no `pool_idle_timeout`, no `tcp_keepalive`). Through a Tor proxy, every connection already has high latency. Without pool limits, idle Tor connections accumulate. Without keepalive, Tor circuits that die silently are never detected until the next request fails. The default pool settings (60s idle, unlimited per host) are particularly wasteful for Tor since each idle connection holds a Tor circuit open, consuming Tor bandwidth and guard node capacity.

## Key Insights

1. **Connection pooling is fragmented**: NeoTrix has at least 3 independent HTTP client factory patterns (`nt_io_http_factory`, `nt_http::shared_blocking_client`, per-fetcher `OnceLock<Client>`), each with different pool settings, none sharing idle connections. The design intent of a "single HTTP client factory" is stated in `nt_http.rs:1` but not enforced — the NT-WORLD fetchers were added independently and bypass it.

2. **SSRF-safe paths create ephemeral clients by design**: The `resolve_safe_origin` → `.resolve(&host, addr)` pattern in `nt_http.rs` forces per-request client creation because `resolve()` is per-client in reqwest. This is architecturally correct (DNS pinning prevents rebinding) but destroys connection reuse. A better pattern would be a connection pool keyed by `(host, pinned_addr)` that survives across requests.

3. **No global pool size limit is the highest-risk defect**: With 12+ fetchers each maintaining independent pools and `pool_max_idle_per_host=32`, the theoretical maximum idle connections is `12 fetchers × 32/host × N hosts`. In practice, the most dangerous path is `nt_http.rs` which has NO pool limits at all. On a long crawl session visiting hundreds of hosts, this can exceed file descriptor limits.

4. **The `danger_accept_invalid_certs(true)` default is a TLS security gap**: 16 call sites disable certificate verification. While some are in the stealth/anti-detection path (intentional), the global_client and browse session use it unconditionally. This disables certificate pinning and enables MITM attacks on all NeoTrix HTTP traffic.

5. **Pool timer / idle reaper may not work for blocking clients**: The hyper-util issue #2434 shows that `pool_timer` must be explicitly passed for idle cleanup to work. Reqwest's blocking client may not set this by default, meaning idle timeout is only checked opportunistically at checkout time, not proactively.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 9 |
| HIGH severity | 2 |
| MEDIUM severity | 5 |
| LOW severity | 2 |
| Sources consulted | 13 |
