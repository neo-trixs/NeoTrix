# Iteration Batch 887 — Sources 99695-99726

## Unsafe FFI & Verification (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99695 | Rust Std Safety Comments | std-dev-guide.rust-lang.org/policy/safety-comments | 2026 | // SAFETY: inline + # Safety doc sections; clippy::undocumented_unsafe_blocks |
| 99696 | Linux Kernel Rust Guidelines | docs.kernel.org/6.6/rust/coding-guidelines | 2026 | SAFETY comments mandatory, capitalized, period-terminated |
| 99697 | UniFFI proc-macro | github.com/mozilla/uniffi-rs | 2026 | #[uniffi::export] + FfiConverter; eliminates UDL duplication |
| 99698 | Kani contracts | arxiv.org/html/2607.01504v1 | 2026-07 | 16K+ harnesses in stdlib; requires/ensures contracts; 88.9% auto-spec success |
| 99699 | Miri UB detection | POPL 2026, research.ralfj.de/papers/2026-popl-miri | 2026-01 | Stacked/Tree Borrows; 70%+ test pass rate; integrated into stdlib CI |
| 99700 | Sanitizers stable | goals.rust-lang.org/2026/sanitizer-support | 2026 | ASan/LSan near stable; Tier 2 targets for MSan/TSan; #[sanitize] per-function |
| 99701 | repr(C)/repr(transparent) | doc.rust-lang.org/nomicon/other-reprs | 2026 | repr(C) C ABI; repr(transparent) single-field identity; Option<NonNull<T>> |
| 99702 | FFI panic safety | RFC 2945 c-unwind ABI | 2026 | "C" = panic→abort; "C-unwind" = panic→unwind; catch_unwind at boundaries |

**Defect categories addressed**: D-EMBED-001, D-EMBED-002, D-FORMAL-001 to D-FORMAL-006

## Database Patterns (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99703 | SQLx production | abrarqasim.com/blog/rust-sqlx-production | 2026-04-29 | Generic Executor trait; offline mode; pool defaults wrong for prod |
| 99704 | SQLite WAL write perf | emschwartz.me/sqlite-connection-pool | 2026-02-17 | Single writer + reader pool: 2,586→60,061 rows/sec (23x) |
| 99705 | FTS5 CJK trigram | zenn.dev/kanseilink/fts5-trigram-cjk | 2026-05-07 | Dual-table: unicode61 + trigram; <3 char fallback to LIKE |
| 99706 | FTS5 ICU tokenizer | github.com/cwt/fts5-icu-tokenizer | 2025 | ICU word segmentation; locale-specific transliteration |
| 99707 | SQLite pool tuning | productionhardening.org/wal-optimization | 2026 | Readers=min(cores,8); writers=1; PRAGMA parity; circuit breaker |
| 99708 | fred Redis client | github.com/aembke/fred.rs | 2026 | RESP2/3; clustered/sentinel; auto-reconnect; auto-pipelining |
| 99709 | Bitemporal TensorDB | docs.rs/tensordb | 2026 | valid_from/valid_to + commit_ts; VALID AT + AS OF queries |
| 99710 | petgraph-live | github.com/geronimo-iia/petgraph-live | 2026-04-30 | Generation-keyed cache; disk snapshot bincode+zstd; algorithms |

**Defect categories addressed**: D-DB-001 to D-DB-013, D-POOL-011, D-POOL-012

## CI/CD & Tooling (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99711 | CI pipeline 6-stage | ThreatFlux/rust-cicd-template | 2026-01-12 | check→test→coverage→security→cross→release; SHA-pinned actions |
| 99712 | cargo-deny | EmbarkStudios/cargo-deny | 2026 | licenses+bans+advisories+sources; deny.toml config |
| 99713 | cargo-audit | rustsec/rustsec v0.22.2 | 2026-06-05 | Scans Cargo.lock; Vulnerability/Unmaintained/Unsound; audit.toml |
| 99714 | cargo-tarpaulin | xd009642/tarpaulin | 2026 | --fail-under; LLVM engine; Lcov output; workspace support |
| 99715 | cargo-mutants | sourcefrog/cargo-mutants v27.1 | 2025 | --in-place; --test-tool nextest; #[mutants::skip]; sharding |
| 99716 | cargo-dist | axodotdev/cargo-dist v0.32 | 2026 | Tag-driven releases; cross-platform builds; dist init |
| 99717 | rustfmt style_edition | doc.rust-lang.org/edition-guide | 2026 | style_edition="2024" decouples from edition; always set explicitly |
| 99718 | Clippy pedantic/restriction | doc.rust-lang.org/clippy | 2026 | pedantic=opinionated; restriction=cherry-pick; msrv-aware |

**Defect categories addressed**: D-CI-001 to D-CI-003, D-DEPLOY-001 to D-DEPLOY-008

## WASM Plugin System (8 sources)

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 99719 | WIT language | github.com/WebAssembly/component-model | 2025 | Interface types + worlds; package = distribution unit |
| 99720 | wit-bindgen v0.53 | github.com/bytecodealliance/wit-bindgen | 2026-02-13 | generate! macro; wasm32-wasip2 target; Guest trait |
| 99721 | Wasmtime production | systemshardening.com/wasmtime-hardening | 2026-04-27 | Fuel+Epoch combo; static_memory_maximum_size(64MiB); disable threads |
| 99722 | Wassette (Microsoft) | opensource.microsoft.com/blog/2025/wassette | 2025-08-06 | Deny-by-default; interactive consent; OCI distribution; signing |
| 99723 | Extism v1.21 | github.com/extism/extism | 2026-03-26 | PluginBuilder; fuel_limit; host functions; XTP bindgen |
| 99724 | ResourceLimiter | docs.wasmtime.dev/ResourceLimiter | 2026 | memory_growing(); instances() default 10000; ResourceLimiterAsync |
| 99725 | Component supply chain | systemshardening.com/wasm-supply-chain | 2026 | Transitive deps invisible; runtime digest verification; wkg+OCI |
| 99726 | Shared-nothing linking | WebAssembly/component-model/Linking.md | 2026 | Separate memories; canonical ABI serialization; wac composition |

**Defect categories addressed**: D-WASM-001 to D-WASM-007, D-PLUGIN-001 to D-PLUGIN-006

## New Defects (Batch 887)

### D-FFI-001 CRITICAL: unsafe blocks without SAFETY comments
- **Evidence**: Rust std-dev-guide, Linux kernel, Chromium all require // SAFETY: comments
- **Impact**: Audit trail impossible; unsound code propagates silently
- **Fix**: Add clippy::undocumented_unsafe_blocks lint; add SAFETY comment to all 5+ unsafe blocks

### D-FFI-002 HIGH: neotrix-sysctl FFI provenance gaps
- **Evidence**: Kani found 11 bugs across Firecracker/s2n-quic/Hifitime; miri detects pointer violations
- **Impact**: Potential undefined behavior in transmute, pointer casts
- **Fix**: Add Kani proof harnesses for unsafe FFI functions; run miri on all FFI code

### D-FORMAL-007 HIGH: Zero Kani proof harnesses (reconfirmed with new evidence)
- **Evidence**: Rust stdlib has 16K+ Kani harnesses; KaPilot auto-generates 88.9% success rate
- **Impact**: Safety-critical invariants unverified
- **Fix**: Add Kani requires/ensures contracts to core algorithms; integrate into CI

### D-DB-014 HIGH: SQLite single-writer constraint violated by pool size > 1
- **Evidence**: emschwartz.me benchmark: 50-connection pool = 2,586 rows/sec; single writer = 60,061 rows/sec (23x)
- **Impact**: Write performance 23x worse than necessary
- **Fix**: Set max_connections(1) for writer pool; separate reader pool

### D-DB-015 MED: FTS5 unicode61 tokenizer fails for CJK content
- **Evidence**: zenn.dev: unicode61 treats entire CJK runs as one opaque token
- **Impact**: CJK search returns irrelevant results; trigram tokenizer needed
- **Fix**: Add dual-table strategy: unicode61 + trigram; route by hasCJK()

### D-CI-004 HIGH: No CI workflow exists (reconfirmed)
- **Evidence**: ThreatFlux template shows 6-stage pipeline as standard; cargo-deny/cargo-audit/tarpaulin all have GitHub Actions
- **Impact**: No automated quality gate; regressions merge silently
- **Fix**: Create .github/workflows/ci.yml with check→test→coverage→security→deny→audit

### D-WASM-008 HIGH: Wasmtime pinned at v42 (CVE-2026-34971, CVE-2026-34987 Critical 9.0)
- **Evidence**: systemshardening.com recommends fuel+epoch combo; current code has no WASM resource limits
- **Impact**: Critical RCE vulnerabilities in WASM runtime
- **Fix**: Upgrade wasmtime to latest stable; add ResourceLimiter with 64MiB cap; enable fuel+epoch

### D-WASM-009 MED: No capability-based deny-by-default for plugins
- **Evidence**: Wassette/salvor_wasm/mcpdef-sandbox all use empty Linker as default
- **Impact**: Plugins get ambient authority (filesystem, network) by default
- **Fix**: Instantiate plugins against empty Linker; grant capabilities explicitly via host functions

### D-DEPLOY-009 MED: No rustfmt.toml with style_edition
- **Evidence**: doc.rust-lang.org: "always set style_edition explicitly; rustfmt CLI defaults to 2015"
- **Impact**: Inconsistent formatting between cargo fmt and editor rustfmt
- **Fix**: Add rustfmt.toml with style_edition = "2024"

### D-DEPLOY-010 MED: No Clippy pedantic/restriction lints configured
- **Evidence**: Clippy docs: pedantic is opinionated lints for power users; restriction for cherry-picked safety
- **Impact**: No automated code quality enforcement beyond default lints
- **Fix**: Add #![warn(clippy::pedantic)] and cherry-pick restriction lints (unwrap_used, panic)
