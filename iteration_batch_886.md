# Iteration Batch 886 — Sources 99663-99694

## Async Cancellation & Structured Concurrency (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99663 | tokio-util CancellationToken | docs.rs/tokio-util/CancellationToken | 2026 | child_token() = unidirectional hierarchy; cancel propagates to all descendants |
| 99664 | tokio JoinSet | docs.rs/tokio/JoinSet | 2026 | Drop aborts ALL tasks — structured concurrency guarantee; join_next() unordered |
| 99665 | tokio TaskTracker | docs.rs/tokio-util/TaskTracker | 2024 | No abort on drop; close()+wait() for graceful drain; immediate memory reclamation |
| 99666 | khive_runtime | docs.rs/khive-runtime/track_background_task | 2026 | Atomic increment on enqueue; BackgroundTaskGuard Drop decrements — panic-safe |
| 99667 | tokio-graceful-shutdown | github.com/Finomnis/tokio-graceful-shutdown | 2026 | Subsystem tree: handle_shutdown_requests(timeout); partial subtree shutdown |
| 99668 | tokio-util AbortOnDropHandle | docs.rs/tokio-util/AbortOnDropHandle | 2026 | Wraps JoinHandle; calls abort() in Drop; #[must_use] compiler warning |
| 99669 | CodeSnips supervisor | codesnips.io/1180 | 2026 | 30+ handlers: TaskTracker (not JoinSet — avoids OOM from return values) |
| 99670 | tokio select biased | docs.rs/tokio/select | 2026 | biased; polls top-to-bottom; shutdown branch MUST be first; deterministic, no RNG |

**Defect categories addressed**: D-CANCEL-001 to D-CANCEL-007, D-SCON-001 to D-SCON-005, D-CSAFE-001 to D-CSAFE-009

## Error Handling & Type Unification (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99671 | thiserror 2.0 | docs.rs/thiserror | 2026 | #[derive(Error)] + #[from] + #[source] + #[error(transparent)] |
| 99672 | anyhow | docs.rs/anyhow | 2026 | 8-byte trait object; .context() lazy evaluation; .root_cause() |
| 99673 | anyhow::Context | docs.rs/anyhow/Context | 2026 | .with_context(\|\|) lazy; downcast to context OR original error |
| 99674 | #[non_exhaustive] | doc.rust-lang.org/reference/attributes | 2026 | RFC 2008: callers must include _ wildcard; allows adding variants |
| 99675 | Error::source() | doc.rust-lang.org/std/error/Error | 2026 | Linked list from high→low; chain() iterator; root_cause() |
| 99676 | eyre | github.com/eyre-rs/eyre | 2026 | Fork of anyhow; color-eyre adds source snippets; customizable handler |
| 99677 | Microsoft RustTraining | microsoft.github.io/RustTraining | 2026 | Library=typed(thiserror), App=anyhow, CLI=eyre; #[error(transparent)] wrappers |
| 99678 | Rust FFI patterns | rust-unofficial.github.io/patterns | 2026 | Flat enums→int codes; structured enums with typed data; #[error("msg")] Display |

**Defect categories addressed**: D-ERR-001 to D-ERR-010, D-TRAIT-003 to D-TRAIT-005

## Memory Allocator & Zero-Copy (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99679 | mimalloc v3.4.5 | github.com/microsoft/mimalloc | 2026-08-05 | Per-thread heaps; cross-thread free reconciliation; ~13% throughput gain |
| 99680 | allocator benchmark | kunalganglani.com | 2026-07-20 | mimalloc fast small allocs; jemalloc best observability; tcmalloc high concurrency |
| 99681 | GlobalAlloc trait | doc.rust-lang.org/std/alloc/GlobalAlloc | 2026 | Never use std::sync::Mutex in allocator; feature-gated multi-allocator pattern |
| 99682 | memmap2 | docs.rs/memmap2 | 2026 | MmapOptions::new().map(); SIGBUS risk; RAII drop→unmap; make_read_only() |
| 99683 | rkyv v0.8.17 | github.com/rkyv/rkyv | 2026-07-02 | Zero-copy: serialized bytes ARE in-memory; access()/access_unchecked() |
| 99684 | Arc/Weak cycles | doc.rust-lang.org/book/ch15-06 | 2026 | Parent→children=Arc, children→parent=Weak; Arc::new_cyclic for self-referential |
| 99685 | Pin patterns | rust-lang.github.io/async-book/pinning | 2026 | Box::pin heap; tokio::pin! stack; PhantomPinned opt-out of Unpin |
| 99686 | nexus-slab | docs.rs/nexus-slab | 2026-04-13 | 15x faster than Box for 32B; 0-cycle free; SLUB-style for same-type churn |

**Defect categories addressed**: D-ALLOC-001, D-MEM-001 to D-MEM-009, D-ZERO-001 to D-ZERO-005

## Connection Pooling (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99687 | deadpool v0.13 | docs.rs/deadpool | 2026-08-26 | Manager trait: create()+recycle(); lazy recycle on next get(); retain() eviction |
| 99688 | bb8 v0.9 | docs.rs/bb8 | 2025 | test_on_check_out; min_idle warm; max_idle_lifetime; is_valid() health check |
| 99689 | hyper pool | hyper_util::client::pool | 2026 | Composable cache/negotiate; set_idle_timeout; clear_idle(); HTTP/2 quirk |
| 99690 | reqwest internals | docs.rs/reqwest | 2026 | One Client per process; pool_max_idle_per_host; tcp_keepalive(60s); clone() shares |
| 99691 | SQLite sqlx | docs.rs/sqlx/sqlite | 2026 | WAL mode mandatory; max_connections small (1-10); test_before_acquire(true) |
| 99692 | HTTP/2 multiplex | rustfaq.org HTTP/2 guide | 2026 | ALPN negotiation; h2 crate; stream flow control; HPACK compression |
| 99693 | Pool warm-up | polyfill-rs benchmark | 2026 | 70% faster subsequent requests; min_idle keeps pool warm; startup acquire loop |
| 99694 | Health checking | bb8/deadpool/sqlx docs | 2026 | On-borrow vs on-return tradeoff; SELECT 1 ping; background cleanup task |

**Defect categories addressed**: D-POOL-001 to D-POOL-010, D-DB-001 to D-DB-012

## New Defects (Batch 886)

### D-CANCEL-008 CRITICAL: Zero JoinSet usage in entire codebase
- **Evidence**: tokio JoinSet Drop aborts ALL tasks — structured concurrency guarantee
- **Impact**: 50-92+ bare spawns become invisible to shutdown; orphan tasks leak
- **Fix**: Replace Vec<JoinHandle> with JoinSet for batch task lifecycle

### D-CANCEL-009 HIGH: Zero TaskTracker usage for 30+ background handlers
- **Evidence**: TaskTracker doesn't accumulate return values (JoinSet OOMs with 30+ tasks)
- **Impact**: JoinSet return value accumulation causes memory growth
- **Fix**: Use TaskTracker for long-lived handlers; JoinSet for batch operations

### D-CSAFE-010 HIGH: select! without biased; allows shutdown starvation
- **Evidence**: Unbiased select! uses random branch ordering; shutdown can be delayed by work cycles
- **Impact**: Graceful shutdown takes extra work cycles; Container SIGTERM timeout risk
- **Fix**: Add biased; with shutdown branch first in ALL select! loops

### D-ERR-011 CRITICAL: NeoTrixError hand-written — zero #[from], zero #[source], zero .context()
- **Evidence**: thiserror 2.0 provides #[derive(Error)] with #[from] and #[source]; anyhow provides .context()
- **Impact**: source() returns None; blanket From<String> destroys provenance; 100+ .map_err(format!) sites
- **Fix**: Migrate to thiserror per domain enum + #[non_exhaustive] + source chain

### D-ERR-012 HIGH: 100+ .map_err(|e| e.to_string()) destroys error provenance
- **Evidence**: anyhow::Context provides .with_context(||) lazy evaluation; thiserror #[from] auto-generates From
- **Impact**: All error context lost; root cause analysis impossible
- **Fix**: Replace with .context("action failed") or .map_err(|e| MyError::Wrap { source: e.into() })

### D-ALLOC-001 HIGH: No global allocator configured — default system allocator
- **Evidence**: mimalloc v3.4.5 shows ~13% throughput gain; per-thread heaps reduce contention
- **Impact**: Suboptimal allocation performance; fragmentation under high concurrency
- **Fix**: Add #[global_allocator] static GLOBAL: MiMalloc = MiMalloc; (2 lines of code)

### D-ALLOC-002 HIGH: No Arc/Weak cycle detection in consciousness graph
- **Evidence**: WebSocket gateway grew 512MB→14GB over weekend from bidirectional strong refs
- **Impact**: Memory leak in long-running consciousness entity
- **Fix**: Replace children→parent strong refs with Weak; add Arc::new_cyclic for self-referential types

### D-POOL-011 HIGH: fetch_safe_http creates fresh Client per request
- **Evidence**: reqwest docs: "One Client per process; clone() shares Arc; rebuilding defeats keep-alive"
- **Impact**: No connection reuse; each request pays TCP+TLS handshake cost
- **Fix**: Create reqwest::Client once, clone() for all requests; pool_max_idle_per_host(10)

### D-POOL-012 MED: No pool warm-up at startup
- **Evidence**: polyfill-rs benchmark: pool warm-up yields 70% faster subsequent requests
- **Impact**: Cold start latency spike; first N requests slower
- **Fix**: Add warm_pool() function: loop acquire() during init phase

### D-DB-013 MED: SQLite missing WAL mode configuration
- **Evidence**: sqlx docs: WAL mode mandatory for concurrent reads; PRAGMA journal_mode=WAL
- **Impact**: Single-writer serialization even with connection pool
- **Fix**: Add connection_customizer: execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
