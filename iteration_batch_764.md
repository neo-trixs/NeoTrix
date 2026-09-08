# Iteration Batch 764 — Rust Edition 2026 / MSRV / Migration Research

**Date**: 2026-09-07
**Prior Batch**: 763 (OpenAPI 3.1 gap, SelfTest filter, doc lint, doctest, interactive portal)

---

## Research Queries & Sources

### Q1: Rust edition 2026 / edition 2024 / MSRV 2026
- [Inside Rust Blog — Program Management Update, April 2026](https://blog.rust-lang.org/inside-rust/2026/05/13/program-management-update--april-2026/)
- [Rust Edition Guide — What are editions?](https://doc.rust-lang.org/edition-guide/editions/)
- [Medium — Rust Editions vs Versions (2026-05-27)](https://medium.com/rustaceans/rust-editions-vs-versions-042a5f30b864)
- [State of Rust 2026 — Dev Newsletter](https://devnewsletter.com/p/state-of-rust-2026/)
- [Inside Rust Blog — Program Management Update, February 2026](https://blog.rust-lang.org/inside-rust/2026/03/27/program-management-update-2026-02/)
- [Rust Users Forum — MSRV as non-breaking change since 2024 Edition](https://users.rust-lang.org/t/is-changing-the-msrv-a-non-breaking-change-since-the-rust-2024-edition/138416)
- [RELEASES.md — Rust GitHub](https://github.com/rust-lang/rust/blob/HEAD/RELEASES.md)
- [MCP #157574 — Add flag to pass MSRV/`package.rust-version` for use by lints](https://github.com/rust-lang/rust/issues/157574)
- [Rust in 2026 — Medium](https://medium.com/@blogs-world/rust-in-2026-what-actually-changed-whats-trending-and-what-to-build-next-d70e38a4ad97)

### Q2: Rust compatibility / stable / MSRV 2026
- [Rust Release Notes — doc.rust-lang.org](https://doc.rust-lang.org/stable/releases.html)
- [Announcing Rust 1.98.0 — Blog](https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/)
- [Announcing Rust 1.97.0 — Blog](https://blog.rust-lang.org/2026/07/09/Rust-1.97.0/)
- [Announcing Rust 1.96.0 — Blog](https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/)
- [endoflife.date/rust](https://endoflife.date/rust)
- [Rust — The Cargo Book (rust-version)](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html)
- [releases.rs](https://releases.rs/)
- [pre-RFC: LTS for the Rust Toolchain — Rust Internals](https://internals.rust-lang.org/t/pre-rfc-lts-for-the-rust-toolchain/24476)

### Q3: Rust edition migration 2026 / edition 2024 migration
- [Rust Edition Guide — Rust 2024](https://doc.rust-lang.org/stable/edition-guide/rust-2024/)
- [Rust Edition Guide — Transitioning an existing project](https://doc.rust-lang.org/stable/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)
- [Rust Edition Guide — Advanced migrations](https://doc.rust-lang.org/stable/edition-guide/editions/advanced-migrations.html)
- [Rust Edition Guide — Changes to the prelude](https://doc.rust-lang.org/stable/edition-guide/rust-2024/prelude.html)
- [Rust FAQ — What Changed in Edition 2024?](https://www.rustfaq.org/en/what-changed-in-rust-edition-2024/)
- [Rustify — Rust 2024 Edition: What's New](https://rustify.rs/articles/rust-2024-edition-whats-new)
- [Reintech — Rust 2024 Edition: Complete Migration Guide](https://reintech.io/blog/rust-2024-edition-migration-guide)

---

## Key Facts Established

### Rust 2024 Edition (Stable since 1.85.0, Feb 2025)
- **Current stable Rust**: 1.98.1 (2026-09-03). Beta: 1.99.0. Nightly: 1.100.0.
- **Edition cycle**: Every ~3 years. 2024 released with 1.85.0. Next edition ~2027.
- **Key 2024 features**: async closures (`async || {}`), RPIT lifetime capture (all in scope by default), `unsafe_op_in_unsafe_fn` warns by default, Cargo Resolver v3, `gen` keyword reserved, `Future`/`IntoFuture` in prelude, v0 symbol mangling default, TOML v1.1 support (1.94.0+).

### MSRV Landscape (2026)
- **Rust for Linux MSRV**: Driven by Debian Stable releases. Forky (est. Summer 2027) will ship ~Rust 1.104.0, enabling new features.
- **MSRV-aware resolver**: Stabilized in Rust 1.85.0 (with edition 2024). `cargo add` auto-selects dependency versions compatible with your `rust-version`.
- **MSRV as non-breaking change**: Confirmed by RustCrypto/getrandom maintainers — MSRV bumps are NOT semver-breaking since edition 2024 + MSRV-aware resolver.
- **MCP #157574**: `-Z lint-rust-version` (consolidated with `--hint-msrv` MCP #772) — lint flag passing MSRV to compiler lints, in "needs-to-bake" stage.

### LTS Toolchain (pre-RFC)
- **Concept**: Rust Project may designate an annual LTS (second minor release each year, e.g., 1.103). LTS gets security patches for ~60 weeks.
- **Guidance**: Almost all projects should use **stable**, not LTS. LTS only for strictly immutable build processes (reactive security maintenance only).
- **Impact**: LTS does NOT imply MSRV policy; ecosystem crates keep targeting stable.

### Edition Migration Tooling Gaps
- **`cargo fix --edition` limitations**: Cannot update doctests, build-time generated code, or complex macros. Partial migration with `--broken-code` option exists.
- **Semantic difference detection (critical)**: Rust for Linux raised that same code can behave differently across editions (drop order, `if let` temporary scope). Current migration tooling has NO guarantee of catching all semantic differences. Proposed solution: compile under both editions, compare MIR output. **No bandwidth to build this.**
- **Backport hazard**: Semi-automatic backports from newer-edition code to older-edition branches can introduce silent semantic changes (e.g., drop order UB). No automated tool exists.

---

## NEW Defects & Improvements for NeoTrix

### DEFECT 1: No Edition Compatibility Matrix in CI
**Severity**: MEDIUM
**Finding**: NeoTrix CI does not verify compilation across multiple Rust editions. Edition 2024 changes drop order, RPIT lifetime capture, and prelude (Future/IntoFuture added). Mixed-edition workspaces are legal but can hide semantic differences.
**Impact**: Silent behavioral differences between crates compiled under different editions, especially around async drop order and RPIT lifetime capture.
**Action**: Add CI matrix testing `edition = "2021"` and `edition = "2024"` for workspace crates. Detect semantic divergence early.

### DEFECT 2: MSRV Policy Not Codified
**Severity**: MEDIUM
**Finding**: NeoTrix `Cargo.toml` files lack `rust-version` field. The MSRV-aware resolver (stable since 1.85.0) is not leveraged. Without `rust-version`, `cargo add` cannot auto-select MSRV-compatible dependency versions.
**Impact**: Dependency updates may silently break older toolchains. Users deploying NeoTrix on constrained environments (embedded, containers) have no documented minimum version.
**Action**: Set `rust-version = "1.85"` (edition 2024 baseline) in all workspace `Cargo.toml` files. Document MSRV policy.

### DEFECT 3: `cargo fix --edition` Cannot Migrate Doctests
**Severity**: HIGH (from batch 763 context)
**Finding**: Confirmed via official Rust Edition Guide — `cargo fix` explicitly cannot update documentation tests. NeoTrix has doctests (batch 763 found zero doctest validation). Post-migration doctests may use edition 2021 syntax (e.g., missing `dyn`, RPIT lifetime differences).
**Impact**: Doctests silently pass or fail depending on the edition the reader compiles them under. Documentation examples may be edition-dependent.
**Action**: Add `cargo test --doc` as mandatory CI step. Add edition annotation (`edition2024`) to all doctest code blocks in doc comments.

### DEFECT 4: No MIR Semantic-Divergence Detection
**Severity**: HIGH
**Finding**: The Rust for Linux team identified a critical gap: no tool exists to detect when the same source code produces different semantics under different editions (drop order changes from `if let` temporary scope in 2024). Proposed MIR comparison tool has no implementation bandwidth.
**Impact**: If NeoTrix ever backports patches across edition boundaries (or supports users on mixed editions), silent UB from drop-order changes is possible. Relevant for NT-SHIELD safety-critical paths.
**Action**: Investigate `cargo rustc --edition-diff` or implement a CI check that compiles key crates under both editions and diffs MIR (using `-Zdump-mir`). Long-term: advocate for upstream tooling.

### DEFECT 5: v0 Symbol Mangling Default Breaks Tooling
**Severity**: LOW-MEDIUM
**Finding**: Rust 1.97.0 enables v0 symbol mangling by default. Legacy scheme only available on nightly and planned for removal. Tools like debuggers/profilers using old demangling will break.
**Impact**: NeoTrix profiling workflows (NT-MIND evolution benchmarks, NT-ACT performance tracing) may fail to demangle symbols if using older profiling tool versions.
**Action**: Audit profiling toolchain versions. Ensure `pprof`, `flamegraph`, `perf`, `cargo-flamegraph` use demanglers that support v0. Document in build docs.

### DEFECT 6: TOML v1.1 Raises Dev MSRV
**Severity**: LOW
**Finding**: Cargo 1.94.0+ parses TOML v1.1 manifests. Using TOML v1.1 features in `Cargo.toml` raises development MSRV but published manifests remain compatible with older parsers.
**Impact**: If NeoTrix adopts TOML v1.1 features (e.g., inlines, dotted keys in new positions), developers on older toolchains cannot build from source even if the published crate is fine.
**Action**: Avoid TOML v1.1-only syntax in `Cargo.toml` until MSRV policy explicitly raises to 1.94+. Add lint check.

### IMPROVEMENT 1: Adopt `--hint-msrv` Once Stabilized
**Severity**: LOW (future)
**Finding**: MCP #157574 (`-Z lint-rust-version`) / MCP #772 (`--hint-msrv`) — both in "needs-to-bake" stage. Will allow lints to use `package.rust-version` to warn about using APIs newer than declared MSRV.
**Action**: Track stabilization. Once stable, add to CI as a lint gate. Prevents accidentally using APIs that exceed declared MSRV.

### IMPROVEMENT 2: Leverage `derive` Macro Path (1.96+)
**Severity**: LOW
**Finding**: `{core,std}::derive` stabilized in 1.96.0 (MSRV 1.96). Allows `use core::derive::Macro` instead of `use derive_more::...` or attribute-only derives.
**Action**: Evaluate for new derive usage. Not urgent but reduces external proc-macro dependencies.

### IMPROVEMENT 3: Edition-Gated Lint Groups for Migration
**Severity**: LOW
**Finding**: `cargo fix --edition` uses lint groups (`rust-2024-compatibility`). Individual lints like `rust_2024_prelude_collisions` can be enabled incrementally. Workspace migration can be crate-by-crate.
**Action**: When migrating to edition 2024, use incremental approach: enable individual lints first, fix per-crate, then flip edition field. Document migration playbook.

---

## Summary

| Category | Count | Key Items |
|----------|-------|-----------|
| New Defects | 6 | Edition compat matrix, MSRV policy, doctest migration, MIR divergence, v0 mangling, TOML v1.1 |
| New Improvements | 3 | `--hint-msrv`, `derive` path, edition lint groups |
| Sources Cited | 20+ | Official Rust blog, edition guide, RELEASES.md, GitHub issues, community discussions |

**Critical Insight**: The edition migration tooling gap (DEFECT 4) is an **upstream problem** with no current solution. NeoTrix should adopt a defensive posture: document edition boundaries, test across editions in CI, and avoid backporting across edition boundaries in NT-SHIELD safety paths.
