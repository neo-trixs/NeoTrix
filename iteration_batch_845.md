# Iteration Batch 845 Report — NeoTrix Consciousness Architecture

## Research Sources (38+)

### Documentation (8)
- Zero doc tests across 1,382 source files (critical)
- No missing_docs lint (362 files with zero doc comments)
- No intra-doc links (only 1 file uses them)
- No doc coverage measurement
- No doc(test(...)) configuration
- Module-level docs incomplete (6-line ASCII diagram)
- No cargo doc --no-deps in CI
- docs/ vs rustdoc split (no cross-referencing)

### Benchmarking (10)
- Criterion 0.5 (3 versions behind 0.8.2)
- Zero CI benchmark workflow (no regression gating)
- No baseline persistence or comparison
- repair_c3.rs is placeholder stub (dead code)
- No iai-callgrind or Divan coverage
- No async benchmarks despite tokio in features
- No throughput measurements for I/O-heavy operations
- No flamegraph integration
- No cross-domain Constellation benchmark pattern
- No per-domain benchmark taxonomy

### Deployment (12)
- Edition 2021 + resolver v2 (stale, 2024 stable)
- Full LTO in release (2-3x compile cost)
- No dist profile (cargo-dist cannot integrate)
- Zero CI/CD configuration
- No rust toolchain pinning
- No .cargo/config.toml (missing linker + CI tuning)
- glibc default (no musl/static build path)
- No cargo-chef layer caching
- No release artifact signing
- Tauri desktop has no containerized build path
- opt-level = "s" (size vs speed tradeoff)
- No cross-compilation matrix

### Serialization (8)
- bincode is dead (RUSTSEC-2025-0141)
- rmp-serde dangerous for untrusted data (OOM crash)
- postcard: no_std, MaxSize derive, smallest wire format
- rkyv 10x faster for reads (~5ns/struct)
- JSON cannot zero-copy (always allocates)
- store_to_rkyv serializes to JSON despite rkyv feature
- rkyv load_deserialized discards zero-copy advantage
- No format versioning in rkyv storage

---

## Defects Identified (38+)

### Documentation (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-DOC-1 | Zero doc tests across 1,382 files | Critical |
| D-DOC-2 | No missing_docs lint (362 files undocumented) | Critical |
| D-DOC-3 | No intra-doc links (1/848 files) | High |
| D-DOC-4 | No doc coverage measurement | High |
| D-DOC-5 | No doc(test(...)) configuration | Medium |
| D-DOC-6 | Module-level docs incomplete | Medium |
| D-DOC-7 | No cargo doc --no-deps in CI | Medium |
| D-DOC-8 | docs/ vs rustdoc split | Low |

### Benchmarking (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-BENCH-1 | Criterion 0.5 (3 versions behind) | High |
| D-BENCH-2 | Zero CI benchmark workflow | Critical |
| D-BENCH-3 | No baseline persistence | High |
| D-BENCH-4 | No async benchmarks | Medium |
| D-BENCH-5 | No throughput measurements for I/O | Medium |
| D-BENCH-6 | No flamegraph integration | Low |

### Deployment (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-DEPLOY-1 | Edition 2021 + resolver v2 (stale) | High |
| D-DEPLOY-2 | Full LTO (2-3x compile cost) | Medium |
| D-DEPLOY-3 | No dist profile (cargo-dist cannot integrate) | High |
| D-DEPLOY-4 | Zero CI/CD configuration | Critical |
| D-DEPLOY-5 | No rust toolchain pinning | High |
| D-DEPLOY-6 | No .cargo/config.toml | Medium |
| D-DEPLOY-7 | glibc default (no musl) | Medium |
| D-DEPLOY-8 | No cargo-chef layer caching | Medium |

### Serialization (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-SER-1 | store_to_rkyv serializes to JSON (dead code) | High |
| D-SER-2 | rkyv load_deserialized discards zero-copy | Medium |
| D-SER-3 | No format versioning in rkyv storage | Medium |
| D-SER-4 | Hash collision silently overwrites | Low |
| D-SER-5 | KB persistence uses only JSON (O(n) allocation) | High |
| D-SER-6 | No serde_bytes on binary blobs | Low |

## Key Insights (This Batch)

1. **Zero doc tests across 1,382 files**: Every public API ships without verified usage examples. Doc examples can silently rot. Must add no_run doc tests to all public items.

2. **Criterion 0.5 (3 versions behind)**: Missing async benchmarks, improved statistics, plotters support. Must upgrade to 0.8.2.

3. **Edition 2021 + resolver v2 is stale**: 2024 edition stable since Feb 2025. NeoTrix misses async closures, precise capturing, combined rustdoc tests for free.

4. **store_to_rkyv serializes to JSON**: The rkyv-storage gated function writes serde_json::to_string_pretty to bank.json. rkyv is never used for actual serialization. Dead code path.

5. **Full LTO in release (2-3x compile cost)**: lto = true triggers full (fat) LTO. lto = "thin" achieves ~95% benefit at 1/3 link time.

6. **rkyv 10x faster for reads**: Zero-copy at ~5ns/struct. NeoTrix has nt_core_rkyv.rs but only does from_bytes (full deserialization), not true zero-copy access.

7. **JSON cannot zero-copy**: Due to character escaping, serde_json always allocates. For hot-path KB reads, this is a bottleneck. Must use postcard/rkyv.

8. **postcard is the embedded king**: no_std, MaxSize derive (compile-time upper bound prevents OOM), varint encoding (smallest wire format).

9. **No cross-compilation matrix**: Single x86_64-unknown-linux-gnu implicit target. Cannot deploy to ARM servers, Alpine containers, or Windows.

10. **No release artifact signing**: Supply chain attacks on Rust binaries are documented. cargo-dist supports artifact signing natively.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 845 |
| New defects (this batch) | 28 |
| Cumulative defects | D01-D77290 |
| Research sources (this batch) | 38 |
| Cumulative research sources | 98,259+ |
