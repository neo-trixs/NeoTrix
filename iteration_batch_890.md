# Iteration Batch 890 — Sources 99791-99822

## Middleware & HTTP Patterns (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99791 | Tower Service/Layer | docs.rs/tower, tower-rs/tower | 2026 | poll_ready() backpressure; Layer composition: A→B→S→B→A |
| 99792 | Axum middleware | docs.rs/axum 0.8.9 | 2026 | from_fn: async fn + extractors + Next; route_layer vs layer |
| 99793 | HTTP interceptors | tonic Interceptor, tower guides | 2026 | Tower Service preferred over Interceptor for full control |
| 99794 | Rate limiting | tower-governor 0.8.0 | 2025 | GCRA algorithm; KeyExtractor trait; into_make_service_with_connect_info |
| 99795 | Auth middleware | cmackenzie1/axum-jwt-auth | 2026 | from_fn + Extension<Claims>; JWKS auto-refresh; token from header/cookie |
| 99796 | Request ID | tower-http request-id | 2026 | SetRequestId → PropagateRequestId; x-request-id header |
| 99797 | Compression | tower-http 0.7.0 | 2026 | CompressionLayer: gzip/deflate/br/zstd based on Accept-Encoding |
| 99798 | CORS | docs.rs/tower-http/cors | 2026 | MUST be outermost layer; CorsLayer::new() restrictive, ::permissive() |

**Defect categories addressed**: D-MW-001 to D-MW-008, D-SEC-001

## Database ORM & Migrations (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99799 | SQLx migrations | docs.rs/sqlx/migrate | 2026 | migrate!() compile-time embed; advisory locks; rollback via undo() |
| 99800 | SeaORM 2.0 | sea-ql.org/blog/2026-07-27 | 2026-07-20 | #[sea_orm::model]; ActiveValue states (Set/Unchanged/NotSet); nested ActiveModel |
| 99801 | diesel-async v0.9 | crates.io/diesel-async | 2026-06-19 | Drop-in async; AsyncPgConnection; streaming queries load_stream() |
| 99802 | Indexing | dev.to/aperd/database-performance | 2026-08-28 | Selectivity <10-20%; composite order: most selective first; covering indexes |
| 99803 | SeaQuery | docs.rs/sea-query | 2026 | Safe Rust; apply_if() for dynamic WHERE; cross-DB via QueryBuilder trait |
| 99804 | Health checks | docs.rs/sqlx-core/PoolOptions | 2026 | test_before_acquire; before_acquire callback; memory-based eviction |
| 99805 | Backup/recovery | production-database.md | 2026 | pg_dump + WAL archiving + PITR quarterly; additive migration pattern |
| 99806 | Multi-DB | diesel-dualdb crate | 2026 | #[derive(MultiConnection)]; DualConnection type; UUID→BLOB mapping |

**Defect categories addressed**: D-DB-001 to D-DB-015, D-MIG-001 to D-MIG-004

## Testing & CI/CD (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99807 | tokio::test | docs.rs/tokio/test | 2026 | flavor="multi_thread"; time::pause()+advance(); assert_pending/assert_ready |
| 99808 | Criterion async | bheisler.github.io/criterion | 2026 | to_async(executor); async_tokio feature; FuturesExecutor |
| 99809 | rust-cache | Swatinem/rust-cache v2.9.2 | 2026 | Auto-keyed by lock/toml hashes; CARGO_INCREMENTAL=0; 10GB limit |
| 99810 | cargo-llvm-cov | taiki-e/cargo-llvm-cov | 2026 | LLVM source-based; --lcov output; coverage_nightly coverage(off) |
| 99811 | proptest | proptest-rs/proptest v1.11 | 2026 | 256 default cases; .proptest-regressions persistence; prop_assert! |
| 99812 | cargo-fuzz | rust-fuzz/cargo-fuzz v0.13 | 2026 | fuzz_target!; --sanitizer none for no-unsafe; cargo fuzz tmin |
| 99813 | testcontainers | testcontainers-rs v0.27 | 2026 | GenericImage; get_host_port_ipv4; OnceCell reuse; Docker CI |
| 99814 | insta snapshots | mitsuhiko/insta v1.48 | 2026 | assert_json_snapshot!; redaction with JSONPath; cargo insta review |

**Defect categories addressed**: D-TEST-001 to D-TEST-005, D-CI-001 to D-CI-004

## Safety & Security (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99815 | cargo-geiger | github.com/geiger-rs/cargo-geiger | 2026 | Counts used/unused unsafe per crate; supply chain unsafe surface |
| 99816 | Miri UB detection | POPL 2026, research.ralfj.de | 2026-01 | Stacked/Tree Borrows; 70%+ test pass; data race detection |
| 99817 | safer_ffi | github.com/getditto/safer_ffi | 2026 | #[derive_ReprC]+#[ffi_export]; zero-unsafe FFI; C header generation |
| 99818 | UniFFI | github.com/mozilla/uniffi-rs v0.32 | 2026 | Multi-language bindings; proc-macro or UDL; Kotlin/Swift/Python |
| 99819 | Pin safety | doc.rust-lang.org/Pin | 2026 | Pin prevents move; PhantomPinned opts out of Unpin; safe self-referential |
| 99820 | Send/Sync | doc.rust-lang.org/nomicon/send-and-sync | 2026 | Send=move; Sync=&T; !Send: Rc/Cell; !Sync: UnsafeCell |
| 99821 | catch_unwind | doc.rust-lang.org/std/panic | 2026 | UnwindSafe trait; AssertUnwindSafe wrapper; not UnwindSafe: &mut T |
| 99822 | Supply chain | cargo-deny + cargo-audit + cargo-vet | 2026 | 4 dimensions: advisories/licenses/bans/sources; cargo-vet human audit |

**Defect categories addressed**: D-SAFETY-001 to D-SAFETY-008, D-FFI-001, D-FFI-002

## New Defects (Batch 890)

### D-MW-001 HIGH: No Tower Service/Layer middleware composition
- **Evidence**: Tower: poll_ready() backpressure; Layer composition A→B→S→B→A
- **Impact**: No standardized middleware pipeline; custom ad-hoc implementations
- **Fix**: Implement Tower Service trait for NT-IO handlers; compose via ServiceBuilder

### D-MW-002 HIGH: No request ID middleware (UUID correlation)
- **Evidence**: tower-http: SetRequestId → PropagateRequestId; x-request-id header
- **Impact**: Cross-service debugging impossible without correlated request IDs
- **Fix**: Add tower-http SetRequestIdLayer + PropagateRequestIdLayer

### D-DB-016 HIGH: No ORM — raw SQL queries everywhere
- **Evidence**: SeaORM 2.0: nested ActiveModel; change detection; M-N relations
- **Impact**: N+1 queries; no type safety; manual FK dependency ordering
- **Fix**: Adopt SeaORM for complex queries; keep SQLx for simple queries

### D-DB-017 HIGH: No database migration strategy
- **Evidence**: SQLx migrate!() compile-time embed; advisory locks; rollback support
- **Impact**: Schema drift; no rollback; deployment race conditions
- **Fix**: Add sqlx migrate!() with advisory locks; embed migrations in binary

### D-TEST-006 HIGH: No CI pipeline (swatinem/rust-cache + cargo-deny + cargo-llvm-cov)
- **Evidence**: Swatinem: auto-keyed cache; cargo-deny: 4 policy dimensions; llvm-cov: source-based
- **Impact**: No automated quality gate; no coverage measurement; no supply chain audit
- **Fix**: Create .github/workflows/ci.yml with cache + deny + coverage

### D-SAFETY-009 HIGH: Zero unsafe auditing (cargo-geiger not run)
- **Evidence**: cargo-geiger: counts used/unused unsafe per crate; supply chain surface
- **Impact**: Unsafe code exposure unknown across dependency tree
- **Fix**: Add cargo-geiger to CI; set threshold for unsafe expressions

### D-SAFETY-010 HIGH: Zero Miri testing for UB detection
- **Evidence**: Miri: Stacked/Tree Borrows; data race detection; 70%+ test pass across 100K crates
- **Impact**: Undefined behavior undetected in unsafe code
- **Fix**: Add cargo miri test to CI; enable tree-borrows model

### D-FFI-003 HIGH: FFI functions lack SAFETY comments
- **Evidence**: Rust std-dev-guide + Linux kernel + Chromium all require // SAFETY: comments
- **Impact**: Audit trail impossible; unsound code propagates
- **Fix**: Add // SAFETY: comments to all unsafe blocks; enable clippy::undocumented_unsafe_blocks

### D-CONFIG-011 MED: No configuration validation at startup
- **Evidence**: garde: #[derive(Validate)] with range/length/email rules
- **Impact**: Invalid config causes runtime panics
- **Fix**: Add garde validation: port range(1,65535), url required

### D-LOG-008 MED: No request ID in tracing spans
- **Evidence**: UUIDv7: 48-bit ms timestamp; attach to root span; propagate in headers
- **Impact**: Trace context lost across async task boundaries
- **Fix**: Generate UUIDv7 at edge; attach to info_span!("request", %request_id)
