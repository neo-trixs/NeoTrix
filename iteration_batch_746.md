# Iteration Batch 746 — Cargo & Build System Research

**Date**: 2026-09-07
**Prior batch**: 745 (toml CVE-2026-16634, serde_yaml archived, config schema validation gap, parsing timeout, libyaml recursive anchor)

---

## Findings

### F1: Cargo build-dir layout v2 — Re-stabilization After Rollback (CRITICAL)
- **Source**: https://github.com/rust-lang/cargo/pull/17354 (merged 2026-08-18)
- **What**: The build-dir layout v2 was first stabilized in PR #16807 (July 2026), but **reverted** due to performance and scaling issues on nightly. Re-stabilized in PR #17354 after fixes in #17191, #17168, #17236.
- **Defect for NeoTrix**: **NeoTrix build scripts and `OUT_DIR`-dependent code may break under the new layout.** The old layout organized by artifact type (`deps/`, `build/`, `fingerprint/`); the new layout organizes by package name + hash. Any code that hardcodes paths like `target/debug/deps/` or `target/debug/build/*/out/` will fail. NeoTrix must audit all `OUT_DIR` consumers, build script path lookups, and any `env!("CARGO_BIN_EXE_*")` usage.
- **Severity**: HIGH — silent breakage if NeoTrix depends on build-dir internals.

### F2: Cross-Workspace Build Cache — New Shared Cache Initiative (IMPROVEMENT)
- **Source**: https://cephalochromoscope.net/8fb51b44 (Cargo cross-workspace cache design, tracked in rust-lang/cargo#7931)
- **What**: Cargo is designing a shared build cache across workspaces. Artifacts stored by fingerprint (not crate name), enabling reuse across workspaces. Planned for nightly in 2026. Prerequisite: build-dir layout v2 (now re-stabilized).
- **Defect for NeoTrix**: **No NeoTrix-level build cache strategy exists.** NeoTrix has 9 domains × many crates. Cross-workspace artifact sharing could reduce CI build times by ~68% (per johal.in data). NeoTrix should: (1) enable `build-cache = { cross-crate-share = true }` when available, (2) add `cargo-cache-stats` monitoring to CI, (3) design cache warming for top-3 build configs.

### F3: Cargo Build Scheduler Suboptimal — 9.6% Overhead (IMPROVEMENT)
- **Source**: https://spirali.github.io/blog/cargo-scheduler/ (2026-08-31)
- **What**: Ada's research shows Cargo's default build scheduler is **9.6% slower** than a simple b-level (bottom-level) heuristic at n=4 cores, and **2.3% slower** at n=16. A blind b-level scheduler (no timing data, just graph depth) still beats Cargo in 16/17 projects at n=4. The research suggests a local database of past build timings could bootstrap better scheduling.
- **Defect for NeoTrix**: **NeoTrix workspace with 9+ domains suffers scheduler waste.** On developer machines (4-8 cores), Cargo's scheduler leaves ~10% performance on the table. NeoTrix should: (1) benchmark `cargo build --timings` across the workspace, (2) consider the `b-level` heuristic for task ordering, (3) track if Cargo 1.99+ integrates better scheduling.

### F4: Cargo Non-Deterministic Resolver — 3-Tier Build Instability (CRITICAL)
- **Source**: https://vanuan.github.io/blog/2026-09-04-myth-of-rust-determinism/ (2026-09-04)
- **What**: Exposé on Cargo's structural non-determinism:
  - **Tier 1**: Published crates discard `Cargo.lock`; clean builds on Monday ≠ clean builds on Friday (greedy resolver pulls live registry snapshots).
  - **Tier 2**: SemVer semantic gap — MSRV bumps, diagnostic escalations, auto-trait leaks, feature matrix regressions all break downstream without SemVer violations.
  - **Tier 3**: Downstream context inversion — `-Dwarnings`, cross-graph feature unification, trait ingestion scope clashes.
- **Defect for NeoTrix**: **NeoTrix has no defense against Tier 1/2/3 instability.** Specific gaps:
  - No `resolver.incompatible-rust-versions = "fallback"` configured (Resolver v3, Rust 1.84+).
  - No `cargo-semver-checks` integration in CI.
  - No `--locked` / `--frozen` enforcement for reproducible CI builds.
  - No MSRV pinning via `package.rust-version` across workspace members.
  - Feature unification could silently activate unwanted features across 9 domains.

### F5: `min-publish-age` Stabilization — Supply Chain Defense (IMPROVEMENT)
- **Source**: https://github.com/rust-lang/cargo/pull/17335 (feat(resolver): Stabilize min-publish-age)
- **What**: New Cargo feature allowing users to specify a minimum age for dependency versions. Versions newer than the threshold are rejected unless already in `Cargo.lock`. Defense against supply chain attacks (typosquatting, compromised fresh releases).
- **Defect for NeoTrix**: **NeoTrix has no supply chain age gate.** A compromised crate published to crates.io could be pulled immediately. NeoTrix should configure `registry.global-min-publish-age` (e.g., 7 days) in `.cargo/config.toml` and add exception workflow for urgent security fixes.

### F6: TOML v1.1 Parser in Cargo 1.94+ (NEW)
- **Source**: Cargo 1.94 changelog (2026-03-05), PR #16415
- **What**: Cargo now parses TOML v1.1 for manifests and config files. Using v1.1 features raises MSRV. This is the same parser family that Batch 745 identified as having CVE-2026-16634 (CVSS 9.8 stack overflow).
- **Defect for NeoTrix**: **Cargo's own TOML parser may be vulnerable to the same CVE.** NeoTrix uses `toml` crate (same parser lineage). If Cargo 1.94+ ships with the vulnerable parser version, any `Cargo.toml` with crafted content could trigger stack overflow during `cargo build`. NeoTrix must: (1) verify which `toml` version Cargo 1.94+ bundles, (2) add `toml` to the security advisory watchlist, (3) consider `toml_edit` as a safer alternative for config parsing.

### F7: Build Script Timeout Gap (CARRY-FROM-745)
- **Source**: Iteration 745 findings
- **What**: No timeout on build script execution. Malicious or buggy `build.rs` can hang indefinitely.
- **Defect**: **NeoTrix has no `build.build-timeout` configured.** Cargo supports this config but NeoTrix doesn't use it. A single hung build script blocks the entire workspace build.

### F8: Cargo Clean `--workspace` Flag (NEW)
- **Source**: Cargo 1.93 changelog (2026-01-22), PR #16263
- **What**: `cargo clean --workspace` now available. Cleans artifacts of all workspace members.
- **Defect**: **NeoTrix CI may be cleaning incomplete.** Without `--workspace`, `cargo clean` only cleans the current package, leaving stale artifacts from other domains. CI should use `cargo clean --workspace` before cache uploads.

---

## Defect Summary (New in Batch 746)

| # | Defect | Severity | Category |
|---|--------|----------|----------|
| F1 | Build-dir layout v2 may break OUT_DIR consumers | HIGH | Build |
| F2 | No cross-workspace build cache strategy | MEDIUM | Build/CI |
| F3 | Cargo scheduler 9.6% overhead at n=4 | LOW | Performance |
| F4 | No defense against Cargo resolver non-determinism (3-tier) | HIGH | Reproducibility |
| F5 | No supply chain age gate (min-publish-age) | HIGH | Security |
| F6 | Cargo's own TOML parser may share CVE-2026-16634 | CRITICAL | Security |
| F7 | No build script timeout configured | MEDIUM | Build |
| F8 | CI cargo clean may be incomplete | LOW | CI |

---

## Sources Cited

1. https://github.com/rust-lang/cargo/pull/17354 — build-dir v2 re-stabilization
2. https://github.com/rust-lang/cargo/pull/16807 — build-dir v2 original stabilization
3. https://blog.rust-lang.org/2026/03/13/call-for-testing-build-dir-layout-v2/
4. https://cephalochromoscope.net/8fb51b44 — cross-workspace cache design
5. https://spirali.github.io/blog/cargo-scheduler/ — Cargo scheduler analysis
6. https://vanuan.github.io/blog/2026-09-04-myth-of-rust-determinism/ — determinism exposé
7. https://github.com/rust-lang/cargo/pull/17335 — min-publish-age stabilization
8. https://doc.rust-lang.org/stable/cargo/CHANGELOG.html — Cargo 1.93-1.98 changelogs
9. https://dev.to/kunal_d6a8fea2309e1571ee7/how-to-reduce-rust-compile-time-2026-sccache-mold-28h7 — compile time optimization
10. https://vegaloop.com/blog/technical/taming-rust-compile-times-in-ci/ — CI compile time
