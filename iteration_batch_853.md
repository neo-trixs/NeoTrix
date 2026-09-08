# Iteration Batch 853 Report — NeoTrix Consciousness Architecture

## Research Sources (38+)

### Security (10)
- cargo-audit + cargo-deny + cargo-geiger = 2026 baseline
- build.rs is real attack surface (arbitrary code execution)
- cargo-vet catches pre-trust attacks (arrayref 0.3.10 advisory-blind)
- CISA 2026 SBOM minimum elements updated (author identity, digital signature, PURL)
- Sigstore keyless signing via Fulcio (OIDC-bound ephemeral certificates)
- Trivy compromised March 2026; Syft recommended
- 38 blanket advisory ignores without reachability analysis
- No cargo-geiger integration (unsafe code blind)
- No cargo-vet trust network
- No cargo-auditable embedding in binaries

### Documentation (8)
- Rustdoc 33% faster in 2026 (impl filtering, param_env reduction)
- doc_cfg stabilizing (feature-gated availability badges)
- missing_docs lint now emits with --test
- Zero missing_docs lint in NeoTrix
- Near-zero doc tests (36 code fences, ~2 runnable)
- No broken intra-doc link lint
- No [package.metadata.docs.rs] in any Cargo.toml
- Crate-level doc comments sparse

### Benchmarking (8)
- Criterion 0.5: statistical rigor, wall-clock + stats
- Iai-Callgrind: instruction-count precision, deterministic
- Divan 0.1: attribute simplicity, native async fn
- CodSpeed: automated CI flamegraph + regression detection
- flamegraph-rs: cargo flamegraph --bench
- dhat: allocation profiling (heap flamegraphs)
- PGO + LTO: 5-10% additional perf
- NeoTrix has single framework (Criterion 0.5), no allocation profiling, no CI tracking

### Deployment (12)
- cargo-dist (dist) v0.32.0: push tag → CI builds all platforms
- cross-rs/cross: Docker-based, zero-setup, 50+ targets
- cargo-zigbuild: Zig as cross-linker, glibc version pinning
- musl static binaries: FROM scratch Docker images
- cargo-chef: Docker layer caching
- Nix: oxalica/rust-overlay, fenix, crane for reproducible builds
- No release automation exists in NeoTrix
- No musl target
- No Docker strategy
- No Nix reproducibility
- No cargo-auditable
- No SBOM/signing

---

## Defects Identified (32+)

### Security (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-SEC-1 | 38 blanket advisory ignores (no reachability) | High |
| D-SEC-2 | No cargo-geiger integration | Medium |
| D-SEC-3 | No cargo-vet trust network | High |
| D-SEC-4 | No cargo-auditable embedding | Medium |
| D-SEC-5 | No artifact signing pipeline | High |
| D-SEC-6 | SBOM lacks CISA 2026 minimum elements | Medium |
| D-SEC-7 | cargo-deny workflow uses cargo install | Low |
| D-SEC-8 | wildcards = "allow" in bans | Medium |

### Documentation (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-DOC-1 | Zero missing_docs lint | Critical |
| D-DOC-2 | Near-zero doc tests (~2 runnable) | Critical |
| D-DOC-3 | No broken intra-doc link lint | High |
| D-DOC-4 | No [package.metadata.docs.rs] | High |
| D-DOC-5 | Crate-level doc comments sparse | Medium |
| D-DOC-6 | No doc_cfg usage | Low |

### Benchmarking (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-BENCH-1 | Single framework (Criterion 0.5 only) | Medium |
| D-BENCH-2 | No allocation profiling (dhat) | Medium |
| D-BENCH-3 | No CI benchmark tracking | High |
| D-BENCH-4 | Disabled benchmarks dead code | Medium |
| D-BENCH-5 | No async benchmarks | Medium |
| D-BENCH-6 | No flamegraph CI step | Low |

### Deployment (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-DEPLOY-1 | No release automation | High |
| D-DEPLOY-2 | No musl target | High |
| D-DEPLOY-3 | No Docker strategy | Medium |
| D-DEPLOY-4 | No Nix reproducibility | Medium |
| D-DEPLOY-5 | No cargo-auditable | Medium |
| D-DEPLOY-6 | No SBOM/signing | Medium |
| D-DEPLOY-7 | No cross-compilation matrix | High |
| D-DEPLOY-8 | No CPU-optimized containers | Low |

## Key Insights (This Batch)

1. **cargo-vet catches what advisory scanners miss**: arrayref 0.3.10 attack had no advisory on day one. cargo-audit was blind. cargo-vet's human-audited trust layer is essential.

2. **build.rs is the real attack surface**: Executes arbitrary code with full filesystem/network access during cargo build. cargo-audit and cargo-deny cannot detect build-time exploits.

3. **missing_docs lint is zero-effort enforcement**: Add #![warn(missing_docs)] to lib.rs. Start with warn, graduate to deny once coverage is sufficient.

4. **doc tests = documentation + test in one**: Use no_run for IO-touching examples. Use # hidden lines for setup. Binary crates skip doc tests.

5. **Divan is ideal for async benchmarks**: Native async fn support, minimal boilerplate. Criterion requires block_on workaround.

6. **dhat for allocation profiling**: Heap flamegraphs reveal hidden allocation costs in Vec::with_capacity patterns.

7. **cargo-dist (dist) v0.32.0**: Push a git tag → CI builds all platforms → publishes to GitHub Releases. Cross-compilation built-in via cargo-zigbuild/cargo-xwin.

8. **musl static binaries**: FROM scratch Docker images (~5-10MB). Essential for container deployment.

9. **Nix is 100% bit-for-bit reproducible**: Docker is ~68%. Nix store paths are content-addressed.

10. **cargo-chef for Docker layer caching**: Separates dependency compilation from source compilation. Essential for CI build speed.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 853 |
| New defects (this batch) | 28 |
| Cumulative defects | D01-D77501 |
| Research sources (this batch) | 38 |
| Cumulative research sources | 98,527+ |
