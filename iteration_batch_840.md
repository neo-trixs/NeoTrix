# Iteration Batch 840 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### CI/CD & Linting (12)
- No CI pipeline exists (.github/workflows/ directory does not exist)
- Near-zero Clippy configuration (only 1 lint: unwrap_used = "warn")
- No rustfmt configuration (defaults to 2015 formatting rules)
- deny.toml audit ignore list unbounded (38 advisories, no reasons/expiry)
- License allowlist overly permissive (GPL-2.0/3.0 allowed for MIT project)
- No --locked flag enforcement for cargo audit
- Edition 2021 with Rust-version 1.81 (2024 edition stable since Feb 2025)
- No cargo audit bin for release artifacts
- No SBOM generation
- rustls 0.21 dangerous_configuration (deprecated feature)
- holon unpinned git dependency (supply chain risk)
- Safety-critical lints incoming from SCRC (50-200 new lints)

### Documentation (8)
- cargo doc build is broken (30+ unresolved link errors)
- No #![deny(missing_docs)] lint
- No #![deny(rustdoc::broken_intra_doc_links)] lint
- Zero #Examples sections across 1,306 pub items
- Zero #Errors sections
- Negligible #Panics/#Safety coverage
- Near-zero doc tests (7 files out of 1,028)
- Only 22 files use intra-doc links

### Benchmarking (10)
- Criterion 0.5 pinned (3 versions behind, missing async support)
- No benchmark regression CI gate
- Disabled benchmark is dead code (Dark Forest violation)
- Tokio::runtime::Runtime::new() inside bench loops (contaminates data)
- No memory/allocation benchmarks
- No parameterized throughput benchmarks
- sample_size(30) below statistical confidence
- No async benchmarks (NT-IO/NT-ACT heavily async)
- No benchmark artifact tracking
- iai is dead (last update 2021)

### Deployment (7)
- No release pipeline (zero workflows, no Dockerfile, no flake.nix)
- No cross-compilation target matrix (single x86_64-unknown-linux-gnu)
- Release profile optimized for size, not correctness verification
- No container strategy (no Dockerfile, no distroless)
- No Nix reproducible build
- No artifact signing or supply chain integrity
- No incremental build cache strategy

---

## Defects Identified (37+)

### CI/CD (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-CI-1 | No CI pipeline exists | Critical |
| D-CI-2 | Near-zero Clippy configuration (1 lint) | High |
| D-CI-3 | No rustfmt configuration (2015 defaults) | High |
| D-CI-4 | deny.toml audit ignore list unbounded | Medium |
| D-CI-5 | License allowlist overly permissive (GPL allowed) | Medium |
| D-CI-6 | No --locked flag enforcement | High |
| D-CI-7 | Edition 2021 (2024 stable since Feb 2025) | Medium |
| D-CI-8 | No cargo audit bin for release artifacts | Medium |
| D-CI-9 | No SBOM generation | Medium |
| D-CI-10 | holon unpinned git dependency | Medium |

### Documentation (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-DOC-1 | cargo doc build broken (30+ link errors) | Critical |
| D-DOC-2 | No #![deny(missing_docs)] lint | High |
| D-DOC-3 | No #![deny(rustdoc::broken_intra_doc_links)] | High |
| D-DOC-4 | Zero #Examples sections (1,306 pub items) | High |
| D-DOC-5 | Zero #Errors sections | Medium |
| D-DOC-6 | Negligible #Panics/#Safety coverage | Medium |
| D-DOC-7 | Near-zero doc tests (7/1,028 files) | Medium |
| D-DOC-8 | Only 22 files use intra-doc links | Low |

### Benchmarking (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-BENCH-1 | Criterion 0.5 pinned (3 versions behind) | High |
| D-BENCH-2 | No benchmark regression CI gate | High |
| D-BENCH-3 | Disabled benchmark is dead code | Medium |
| D-BENCH-4 | Runtime creation inside bench loops (contaminates data) | High |
| D-BENCH-5 | No memory/allocation benchmarks | Medium |
| D-BENCH-6 | No parameterized throughput benchmarks | Medium |
| D-BENCH-7 | sample_size(30) below statistical confidence | Low |
| D-BENCH-8 | No async benchmarks | Medium |

### Deployment (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-DEPLOY-1 | No release pipeline | Critical |
| D-DEPLOY-2 | No cross-compilation target matrix | High |
| D-DEPLOY-3 | No CI profile (fast builds) | High |
| D-DEPLOY-4 | No container strategy | Medium |
| D-DEPLOY-5 | No Nix reproducible build | Medium |
| D-DEPLOY-6 | No artifact signing | Medium |
| D-DEPLOY-7 | No incremental build cache strategy | Medium |

## Key Insights (This Batch)

1. **No CI pipeline exists**: Zero .github/workflows/, zero Dockerfile, zero flake.nix. All quality gates are manual/local-only. Every production Rust project needs minimum: fmt check, clippy, cargo-deny, cargo-audit.

2. **cargo doc build is broken**: 30+ unresolved link errors. #![cfg_attr(not(test), deny(warnings))] implies deny(rustdoc::broken_intra_doc_links), so cargo doc fails entirely.

3. **Criterion 0.5 pinned (3 versions behind)**: Missing async benchmarks, improved statistics, plotters support. Must upgrade to 0.8.

4. **No release pipeline**: cargo-dist v0.32.0 generates full release.yml from dist init. NeoTrix uses none of this.

5. **Runtime creation inside bench loops**: tokio::runtime::Runtime::new().unwrap() inside iter() measures runtime creation, not the actual operation. Contaminates benchmark data.

6. **i18n is CSS/layout concern**: RTL support done via CSS logical properties (margin-inline-start), not i18n crate. ICU4X handles text directionality detection.

7. **Arabic has 6 plural forms**: Only icu_plurals handles zero/one/two/few/many/other correctly. Must use ICU4X for proper pluralization.

8. **Edition 2024 stable since Feb 2025**: unsafe_op_in_unsafe_fn now deny-by-default, static mut references become hard errors, RPIT lifetime capture rules changed.

9. **sccache with S3 backend**: 92% cache hit rate. kache for cross-worktree reuse. Must add to CI for fast builds.

10. **cargo-dist generates full release pipeline**: plan→build→host→publish→announce. Multi-platform targets. Post-announce jobs for Docker.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 840 |
| New defects (this batch) | 33 |
| Cumulative defects | D01-D77145 |
| Research sources (this batch) | 37 |
| Cumulative research sources | 98,086+ |
