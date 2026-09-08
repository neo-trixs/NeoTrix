# Iteration Batch 889 — Sources 99759-99790

## Async Process Management (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99759 | tokio::process | docs.rs/tokio/process | 2026 | kill_on_drop(true) prevents orphan zombies; PidfdReaper preferred |
| 99760 | spawn_blocking | docs.rs/tokio/spawn_blocking | 2026 | Cannot be aborted once started; 10-100μs max between .await |
| 99761 | Child I/O piping | docs.rs/tokio/process/Child | 2026 | Write stdin in separate task to avoid deadlock |
| 99762 | Signal handling | tokio::signal::unix | 2026 | signal(SignalKind) returns infinite stream; cancel-safe |
| 99763 | Graceful shutdown | tokio-graceful-shutdown | 2026 | CancellationToken + TaskTracker + timeout enforcement |
| 99764 | PID 1 container | crates.io/pid1, crostini | 2026 | Signal forwarding + zombie reaping + orphan adoption |
| 99765 | Process isolation | sandbox-rs, pnut | 2026 | Namespaces + cgroups v2 + seccomp BPF + Landlock |
| 99766 | Watchdog | heartbeat-watchdog | 2026 | UDP/GPI heartbeat monitoring |

**Defect categories addressed**: D-PROC-001 to D-PROC-008, D-CANCEL-007

## Configuration Patterns (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99767 | config-rs layered | docs.rs/config | 2026 | Builder merges sources; try_parsing for env var coercion |
| 99768 | serde deserialization | serde.rs | 2026 | #[serde(default)]; deny_unknown_fields; flatten |
| 99769 | dotenvy | github.com/allan2/dotenvy | 2026 | #[dotenvy::load] before async runtime; set_var unsafe in 2024 |
| 99770 | garde validation | docs.rs/garde | 2026 | #[derive(Validate)] + range/length/email rules; nested |
| 99771 | Hot-reload | arc-swap + notify | 2026 | Lock-free reads via ArcSwap; atomic swap; debounce 500ms |
| 99772 | Secrecy | docs.rs/secrecy | 2026 | SecretBox<T>; Debug redacts; memory zeroed on drop |
| 99773 | Config testing | oneuptime.com | 2026 | Programmatic config; port:0 random; env save/restore |
| 99774 | Config docs gen | config-generator crate | 2026 | #[derive(ConfigGenerator)]; env_key mapping |

**Defect categories addressed**: D-CONFIG-001 to D-CONFIG-008

## Networking (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99775 | Tokio TCP/UDP | dasroot.net/network-programming | 2026 | ~100K connections; into_split() for independent halves |
| 99776 | rustls TLS | github.com/rustls/rustls | 2026 | LazyConfigAcceptor async SNI; pluggable CryptoProvider |
| 99777 | reqwest HTTP | rustify.rs/reqwest-vs-ureq | 2026 | Reuse single Client; set timeouts; don't log auth headers |
| 99778 | WebSocket | websocket.org/rust-guide | 2026 | ws.split() mandatory; Message::Binary for hot paths |
| 99779 | hickory-dns | github.com/hickory-dns | 2026 | TokioAsyncResolver; TTL cache; UDP→TCP fallback |
| 99780 | Circuit breaker | reliability-toolkit-rs | 2026 | Rate limiter + breaker + retry + bulkhead composition |
| 99781 | QUIC quinn | github.com/quinn-rs/quinn | 2026 | Sans-IO pattern; quinn-proto state machine without I/O |
| 99782 | Connection pool | bb8/deadpool docs | 2026 | Guard auto-return; max_lifetime behind LBs; min_idle warm |

**Defect categories addressed**: D-NET-001 to D-NET-008, D-POOL-011

## Serialization (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99783 | serde bound attrs | serde.rs/container-attrs | 2026 | #[serde(bound = "T: MyTrait")]; separate serialize/deserialize |
| 99784 | serde_json streaming | docs.rs/serde_json/StreamDeserializer | 2026 | StreamDeserializer for multiple values; byte_offset() |
| 99785 | bincode-next v3.1 | github.com/Apich-Organization/bincode | 2025 | SIMD varint; Encode/Decode native; original unmaintained |
| 99786 | rkyv zero-copy | rkyv.org | 2026 | Deserialization = pointer cast; 12ns access; no_std |
| 99787 | rmp-serde msgpack | docs.rs/rmp-serde | 2025 | serde_bytes for binary; to_vec vs to_vec_named |
| 99788 | postcard no-std | docs.rs/postcard | 2026 | Varint; flavors (COBS, CRC32); smallest wire format |
| 99789 | Value vs typed | serde-rs/json#635 | 2026 | Value: 32 bytes, 3× more memory; typed: faster, compact |
| 99790 | Error handling | serde.rs/error-handling | 2026 | ser::Error/de::Error traits; S::Error::custom for cross-format |

**Defect categories addressed**: D-SER-001 to D-SER-008

## New Defects (Batch 889)

### D-PROC-009 HIGH: No async process management (tokio::process not used)
- **Evidence**: tokio::process: kill_on_drop(true) prevents zombies; PidfdReaper preferred
- **Impact**: Zombie processes accumulate; orphan reaping fails
- **Fix**: Replace std::process::Command with tokio::process::Command; enable kill_on_drop(true)

### D-PROC-010 HIGH: No signal handling (SIGTERM/SIGINT)
- **Evidence**: tokio::signal::unix::signal returns infinite cancel-safe stream
- **Impact**: Container SIGTERM ignored; graceful shutdown impossible
- **Fix**: Add signal handler: select! { SIGTERM/SIGINT → cancel CancellationToken }

### D-CONFIG-009 HIGH: No layered configuration (defaults→file→env)
- **Evidence**: config-rs: Config::builder() merges sources; try_parsing for env coercion
- **Impact**: Hardcoded defaults; no env var overrides; no local overrides
- **Fix**: Add config-rs with layered sources: defaults → base.toml → env.toml → local.toml → env vars

### D-CONFIG-010 HIGH: No configuration validation
- **Evidence**: garde: #[derive(Validate)] with range/length/email rules; fail-fast at startup
- **Impact**: Invalid config (zero port, empty URL) causes runtime panics
- **Fix**: Add garde validation: port range(1,65535), url required, max_connections range(1,100)

### D-NET-009 HIGH: No DNS resolution (hickory-dns not used)
- **Evidence**: hickory-dns: TokioAsyncResolver; TTL cache; UDP→TCP fallback
- **Impact**: System resolver blocking in async context; no caching
- **Fix**: Add hickory-resolver with TokioAsyncResolver; configure TTL cache

### D-NET-010 HIGH: No circuit breaker for external HTTP calls
- **Evidence**: reliability-toolkit-rs: rate limiter + breaker + retry + bulkhead composition
- **Impact**: Cascading failures when external services down
- **Fix**: Add circuit breaker: 5 failures → open for 30s → half-open → closed

### D-SER-001 HIGH: rkyv zero-copy deserialization declared but never instantiated
- **Evidence**: RkyvStorage defined but never created; store_to_rkyv uses JSON not rkyv
- **Impact**: Dead code; rkyv's 12ns zero-copy access unused
- **Fix**: Wire RkyvStorage into production; replace JSON store with rkyv access()

### D-SER-002 HIGH: serde_json::Value used where typed structs would be 3× faster
- **Evidence**: Value: 32 bytes per value, 3× more memory than typed structs
- **Impact**: Memory bloat on large JSON documents; slower parsing
- **Fix**: Replace Value with typed structs where structure is known; use Value only for dynamic schemas

### D-SEC-001 HIGH: No secret management (secrecy crate not used)
- **Evidence**: secrecy: SecretBox<T>; Debug redacts; memory zeroed on drop
- **Impact**: Secrets logged in debug output; secrets linger in memory
- **Fix**: Add secrecy crate for password/token fields; wrap in SecretBox

### D-LOG-007 MED: No request ID correlation middleware
- **Evidence**: UUIDv7: 48-bit ms timestamp; attach to root span; propagate in headers
- **Impact**: Cross-service debugging impossible
- **Fix**: Add middleware: generate UUIDv7 → attach to tracing span → include in response headers
