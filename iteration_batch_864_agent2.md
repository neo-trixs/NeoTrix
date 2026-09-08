# Agent 2: Connection Pool Management (Batch 864)

## Sources
1. https://users.rust-lang.org/t/connection-pool-in-hyper-client/86248 — hyper connection pool behavior explained
2. https://reintech.io/blog/reqwest-tutorial-http-client-best-practices-rust — reqwest client reuse, pool tuning, retry patterns
3. https://docs.rs/deadpool/latest/deadpool — deadpool managed/unmanaged pool API
4. https://rustify.rs/articles/rust-reqwest-vs-ureq-vs-hyper-2026 — client comparison, connection pooling differences
5. https://rust.codeguides.io/databases/connection-pooling/ — pool sizing, lifecycle knobs, gotchas
6. https://rust.codeguides.io/http-networking/best-practices/ — one shared client per process, timeout policy
7. https://rs4ts.dev/17-database/08-connection-pooling/ — pool guard drop semantics, exhaustion, max_lifetime
8. https://github.com/hyperium/hyper/issues/3728 — TCP backlog 128 vs Go 4096, connection refused under high conn count
9. https://app.studyraid.com/en/read/11242/350323/using-connection-pooling — load balancing interference with long-lived connections
10. https://www.rustfaq.org/en/how-to-implement-connection-pooling-for-network-clients/ — per-authority semaphore isolation, connection lifecycle

## Defects

D-POOL-001: **Per-call `reqwest::Client::builder().build()` in `fetch_safe_http_inner` creates a fresh connection pool per HTTP request, defeating connection reuse.** The `nt_http.rs:212-214` pin_client is built anew on every `fetch_safe_http` / `fetch_safe_http_async` call. Each new `reqwest::Client` opens its own internal pool (backed by hyper), so sequential requests to the same host incur repeated TCP+TLS handshakes. The `resolve` pin is per-client, forcing this — but a shared pool with DNS-pinned connectors would eliminate the overhead. | `nt_http.rs:191-214` | high | Sources 1,2,6,8

D-POOL-002: **`nt_io_provider::factory.rs:probe_ollama` and `probe_llamacpp` each build throwaway `reqwest::Client` instances that are immediately dropped.** These probe functions at lines 1042-1053 and 1058-1069 construct `Client::builder().timeout(2s).build()` on every invocation. Since `Client` holds a connection pool internally, each probe discards its pool after a single HEAD/GET — wasteful if probes run periodically (e.g., health-check loops). Should reuse `global_client()` or pass a shared client. | `nt_io_provider/factory.rs:1042-1069` | medium | Sources 1,2,6

D-POOL-003: **`model_registry.rs` creates `reqwest::Client::new()` per provider struct (`OllamaProvider::new` at line 32, similar patterns for other providers at lines 101, 167).** Each provider instantiation constructs its own isolated connection pool. If multiple providers are instantiated (e.g., in a Vec of providers), every provider gets its own pool, multiplying idle connections and preventing cross-provider TCP reuse for the same host. The global `reqwest::Client` in `nt_io_http_factory.rs` exists but is not used here. | `nt_native_app/model_registry.rs:32,101,167` | medium | Sources 1,2,6,10

D-POOL-004: **`StealthHttpClient::get_or_create_client` uses a hardcoded 9-second TTL (`Duration::from_secs(9)`) for client entries, which is far shorter than typical idle timeouts (90s in `nt_io_http_factory.rs:47`).** At `pool.rs:81`, client entries older than 9 seconds are discarded and rebuilt. This means the stealth net creates a new `reqwest::Client` (with its internal connection pool) every ~9 seconds per proxy URL, causing constant pool churn. The 9s TTL is not aligned with the global `POOL_IDLE_TIMEOUT_SECS = 90` constant, creating inconsistent connection lifecycle behavior between the stealth path and the global path. | `nt_shield_stealth_net/http_client/pool.rs:81,99` | high | Sources 2,5,6,9

D-POOL-005: **`nt_io_http_factory::build_async_client_with_tls` has no `max_lifetime` or connection recycling, and `danger_accept_invalid_certs(true)` is set unconditionally for `ModernH2` and `LegacyHttp11` variants.** The global client at `nt_io_http_factory.rs:93-98` configures `pool_max_idle_per_host` and `pool_idle_timeout` but never sets `tcp_nodelay` or `http2_adaptive_window`. Without `tcp_nodelay(true)`, small HTTP/2 frames may be buffered (Nagle's algorithm), adding latency to interactive requests. The `danger_accept_invalid_certs(true)` default (line 102) means all outbound traffic from the global client skips TLS verification — acceptable for stealth crawling but dangerous if the global client is used for LLM API calls (openai/anthropic/gemini providers all receive a client). | `nt_io_http_factory.rs:93-118` | high | Sources 2,5,6

D-POOL-006: **No `deadpool` or `bb8` integration despite NeoTrix having a SQLite KB (`nt_memory_kb`).** The codebase uses raw `reqwest::Client` for HTTP but has no structured connection pool for database access. The KB subsystem (`nt_memory_kb/nt_http.rs`) is the "single HTTP client configuration source" per its doc comment, yet there is no corresponding pool abstraction for SQLite connections. If NeoTrix adds PostgreSQL or MySQL backends (as hinted by the KB architecture), it will need `deadpool`/`bb8` integration — currently absent. | Global architecture gap | medium | Sources 3,5,7

D-POOL-007: **`reqwest::Client` instances built by `build_async_client_with_proxy` (line 178-201) are not pooled across callers — each invocation returns a new `Client` with its own connection pool.** The proxy-path functions `tor_client`, `build_async_client_with_proxy`, `build_blocking_client_with_proxy` all construct fresh `Client` instances. Callers in `nt_shield_stealth_net`, `proxy_pool.rs:718`, `network_diagnostics/protocol.rs:340`, and `sandbox/remote.rs:19,39` each get isolated pools. There is no proxy-aware connection pool that reuses connections across proxy rotations. | `nt_io_http_factory.rs:166-229` | medium | Sources 1,2,9,10

D-POOL-008: **`resolve_redirects_safely` in `nt_http.rs:126-176` creates a fresh `reqwest::blocking::Client` per redirect hop.** Each redirect iteration at line 134 builds `Client::builder()...build()`, meaning a 5-hop redirect chain creates 5 separate connection pools. This is especially wasteful when most redirects go to the same host. A single client with `resolve` pinning should handle all hops. | `nt_http.rs:134-141` | medium | Sources 1,2,6

D-POOL-009: **No pool metrics or observability.** The global `reqwest::Client` in `nt_io_http_factory.rs` and the stealth pool in `pool.rs` expose no connection pool statistics (idle count, reuse rate, eviction count). The `reqwest::Client` API does not expose pool stats directly, but NeoTrix could instrument at the hyper connector level or use `deadpool` which provides `Pool::status()`. Without metrics, pool exhaustion (D-POOL-001) and churn (D-POOL-004) are invisible until they cause latency spikes. | Global architecture gap | medium | Sources 5,6

D-POOL-010: **`pool_max_idle_per_host` is set to 32 (`POOL_MAX_IDLE_PER_HOST = 32`) which may be excessive for NeoTrix's outbound patterns.** NeoTrix primarily calls a small number of LLM API endpoints (OpenAI, Anthropic, Gemini, Ollama). With 32 idle connections per host and a 90-second idle timeout, the global client could hold up to 32 × N sockets open. Combined with the stealth net's per-proxy clients (D-POOL-004), total socket count could grow unbounded. The Rust SME Cookbook recommends starting near `(cores * 2)` for database pools; HTTP pools should similarly be bounded by actual concurrency needs, not a generous default. | `nt_io_http_factory.rs:46` | low | Sources 5,7,8

## Key Insights

1. **The core architectural flaw is "build a new Client per operation" in the SSRF-safe paths.** The `resolve` pin in reqwest is per-Client, which forces NeoTrix to create a fresh `Client` (and therefore a fresh connection pool) for every `fetch_safe_http` call. This is the correct security trade-off (DNS rebinding prevention), but the performance cost is severe: every request pays TCP+TLS handshake. A solution would be a pool of DNS-pinned clients keyed by (host, resolved_addr), or switching to hyper's lower-level connector API where DNS pinning can be set per-connection without per-Client overhead.

2. **Two parallel HTTP client systems exist with conflicting lifecycle semantics.** `nt_io_http_factory.rs` provides the global clients (90s idle timeout, 32 max idle), while `nt_shield_stealth_net/http_client/pool.rs` maintains its own per-proxy client cache (9s TTL, separate pool). These two systems have different timeout constants, different TLS behaviors, and no shared metrics. This is a Dark Forest violation — two modules doing the same thing differently.

3. **The absence of `deadpool`/`bb8` for database pooling is a latent risk.** NeoTrix's KB is currently SQLite (in-process, no network pool needed), but the architecture hints at future PostgreSQL/Redis usage. The HTTP client patterns suggest the team may repeat the "build per operation" anti-pattern for database connections when that migration happens.

4. **Connection pool observability is zero.** There are no metrics for pool utilization, connection reuse rates, or eviction counts. The `reqwest::Client` doesn't expose these, but `deadpool` does. For a system that routes through proxy pools, Tor circuits, and multiple LLM providers, visibility into connection health is critical for the GWT attention system to make informed routing decisions.

5. **The TCP backlog issue (hyper #3728) is relevant to NeoTrix's high-connection-count scenarios.** When the stealth net or proxy pool opens many simultaneous connections, Rust's default `TcpListener` backlog of 128 may cause "connection refused" errors under load — exactly the scenario described in the hyper issue. NeoTrix should consider using `socket2` to set a larger backlog when creating TCP listeners.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
