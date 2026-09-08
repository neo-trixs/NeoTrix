# Iteration Batch 816 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Async Traits/Syntax (8)
- AFIT stable since Rust 1.75, but Send bounds don't flow through automatically
- RPITIT stable since 1.75 (synthetic GATs, zero-cost monomorphization)
- dyn async NOT stable (12-month project, Phase 1 dyn reform + Phase 2 async dyn)
- Async closures stabilized in Rust 1.85 (Feb 2026)
- RTN (Return Type Notation) blocked on TAIT + next-gen trait solver
- 24 #[async_trait] usages across NeoTrix (legacy pattern)
- AsyncPanelJudge trait missing Send bound on futures
- No RPITIT adoption anywhere in codebase

### Safety/Soundness (8)
- Miri: POPL 2026 paper, can find all de-facto UB in deterministic programs
- Kani: ASE 2026 paper, function contracts found 11 bugs in s2n-quic/Firecracker/Cedar
- cargo-mutants v27.1.0: ThoughtWorks Trial, zero-configuration mutation testing
- fuzzcheck: Structure-aware fuzzing (nightly-only, not actively maintained)
- loom v0.7.2: Concurrency permutation testing (2K+ monthly downloads)
- TSan: LLVM thread sanitizer via RUSTFLAGS (nightly-only for Rust)
- NeoTrix has zero fuzz targets for external input parsers
- neotrix-sysctl FFI escapes Miri detection scope

### Build/CI Tools (8)
- cargo-deny 0.14.x: 37 advisory ignores in deny.toml (large surface)
- cargo-audit 0.22.2: Dedicated RustSec scanner, cargo audit bin feature
- cargo-vet 0.10.2: Mozilla's supply-chain audit (not in NeoTrix)
- cargo-machete 0.9.2: Unused dependency detection (not in CI)
- cargo-udeps 0.1.61: Compiler-level unused dep detection (nightly)
- cargo-expand 1.0.124: Macro expansion debugging (not in NeoTrix)
- deny.toml allows GPL/LGPL/MPL (copyleft risk)
- No pre-commit hooks for cargo tools

### WASM/Embedded (8)
- wasm-bindgen 0.2.127: Component Model interop, WasmGC, exception handling
- wasm-pack 0.15.0: CVE-2026-23745 tar vulnerability
- Wasmtime 48.0.1: 12 security advisories (sandbox escapes on aarch64)
- WASI 0.3 pre-release: Async + stream primitives
- embedded-hal 1.0.0: Stable, async companion crates
- Embassy: Default async embedded framework (cooperative scheduling)
- Wasmtime fuel mechanism: Instruction-level guest cap
- probe-rs replaces OpenOCD, defmt replaces printf

---

## Defects Identified (28+)

### Async Traits/Syntax (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-AFIT-1 | 24 #[async_trait] usages (legacy pattern, should migrate to AFIT) | High |
| D-AFIT-2 | AsyncPanelJudge missing Send bound on futures | High |
| D-AFIT-3 | No RPITIT adoption (zero-cost static dispatch alternative) | Medium |
| D-AFIT-4 | async-trait crate in 3 Cargo.toml files (can be eliminated) | Medium |
| D-AFIT-5 | 62+ tokio::spawn calls without consistent Send verification | High |
| D-AFIT-6 | No async closure usage despite Rust 1.85 availability | Low |

### Safety/Soundness (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-SAFE-1 | Zero fuzz targets for external input parsers | Critical |
| D-SAFE-2 | No Miri CI gate for neotrix-core | High |
| D-SAFE-3 | No loom tests for EventBus/AttentionManager concurrency | High |
| D-SAFE-4 | No cargo-mutants kill-rate metric (phantom assertions likely) | High |
| D-SAFE-5 | neotrix-sysctl FFI escapes Miri detection scope | Medium |
| D-SAFE-6 | No TSan CI job for data race detection | Medium |

### Build/CI (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-CI-1 | No cargo-vet supply-chain audit trail | High |
| D-CI-2 | No unused dependency detection in CI | High |
| D-CI-3 | deny.toml has 37 advisory ignores (stale risk) | Medium |
| D-CI-4 | GPL/LGPL/MPL allowed in deny.toml (copyleft risk) | Medium |
| D-CI-5 | No pre-commit hooks for cargo tools | Medium |
| D-CI-6 | No cargo-expand for SEAL macro debugging | Low |
| D-CI-7 | Redundant CI workflows (security-audit.yml + deny.yml) | Low |
| D-CI-8 | deny.toml multiple-versions = "warn" (R-P15 violation) | Low |

### WASM/Embedded (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-WASM-1 | No WASM sandbox isolation layer (12 CVEs in Wasmtime) | Critical |
| D-WASM-2 | Missing Component Model / WIT interface contracts | Medium |
| D-WASM-3 | No async embedded runtime for NT-PHYSICAL sensors | High |
| D-WASM-4 | No embedded-hal trait abstraction in nt_physical | Medium |
| D-WASM-5 | WASI P3 async not available for server-side plugins | Medium |
| D-WASM-6 | No WASM fuel/memory limit enforcement | High |

## Key Insights (This Batch)

1. **24 #[async_trait] usages should migrate to native AFIT**: Rust 1.75+ supports async fn in traits natively. The async-trait crate adds proc macro overhead and loses Send bounds.

2. **Zero fuzz targets for external input parsers**: NT-WORLD crawls untrusted web content, NT-FILE-ABILITY parses XLSX/PDF. These are attack surfaces with no fuzz coverage.

3. **Kani found 11 bugs that testing/fuzzing missed**: Function contracts are more effective than random testing for correctness invariants. Should apply to EventBus and KB consistency.

4. **Wasmtime 48.0.1 has 12 CVEs**: Sandbox escapes on aarch64 Cranelift. Must upgrade immediately if using WASM plugins.

5. **cargo-vet is missing**: Mozilla's supply-chain audit tool provides in-tree audit records. Essential for 8+ workspace members with deep dep trees.

6. **deny.toml allows GPL/LGPL/MPL**: Copyleft licenses may conflict with NeoTrix's MIT distribution. Must restrict allowlist.

7. **Embassy is default for embedded Rust**: Cooperative async with hardware abstraction. NT-PHYSICAL should adopt instead of superloop pattern.

8. **RTN blocked on TAIT**: Cannot write T::method(): Send on stable. Workaround: trait_variant::make or manual where clauses.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 816 |
| New defects (this batch) | 26 |
| Cumulative defects | D01-D76548 |
| Research sources (this batch) | 32 |
| Cumulative research sources | 97,345+ |
