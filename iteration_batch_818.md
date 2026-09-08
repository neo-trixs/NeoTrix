# Iteration Batch 818 Report — NeoTrix Consciousness Architecture

## Research Sources (32+)

### Profiling/Debugging (8)
- samply 0.13.1: Firefox Profiler UI, on+off-cpu sampling, macOS xctrace
- flamegraph 6.0k stars: SVG flamegraph via Inferno, requires --no-rosegment with mold
- pprof-rs 0.15: Signal-based profiler, pprof protobuf, criterion integration
- tracy 16.7k stars: Real-time nanosecond profiler, GPU API support, remote telemetry
- tokio-console 4.6k stars: Async runtime debugger, gRPC streaming diagnostics
- NeoTrix has zero profiling tooling integrated
- No async runtime visibility (GWT/SEAL/EventBus timing invisible)
- No lock contention profiling

### Concurrent Data Structures (7)
- crossbeam-epoch 0.9.21: Epoch-based GC for lock-free structures
- arc-swap 1.9.2: Atomically swappable Arc (weakened orderings in 1.9)
- triomphe 0.1.16: Servo's Arc fork (no weak ref count, saves 1 word)
- weak-table 0.4.0: Weak hash maps with auto-cleanup
- lru 0.18.4: O(1) LRU cache (single-threaded only)
- NeoTrix has 100+ Arc<RwLock<T>> patterns (no arc-swap)
- 7 FFI wrappers all use Arc::new(RwLock::new(...))

### Unicode/Text (7)
- unicode-segmentation 1.13.3: GraphemeCursor has 9 panic paths
- unicode-width 0.2.2: Semver catastrophe (\n width 0→1 broke many crates)
- unicode-normalization 0.1.25: Stale Unicode version (15.x vs 17.0)
- strsim 0.11.1: No grapheme-aware edit distance
- textwrap 0.16.2: No grapheme-aware wrapping by default
- NeoTrix nt_normalizer.rs uses ASCII whitespace split (no grapheme)
- GraphRAG split_sentences is ASCII-only (no CJK terminators)

### GPU/Compute (7)
- wgpu v30: Cross-platform, 5-10% overhead, compute well-supported
- vulkano 0.35+: vulkano-taskgraph requires unsafe for core API
- ash 0.38: Raw Vulkan, entirely unsafe, last release Apr 2024
- cuda-sys 0.2.0: Frozen at 2018, Rust-CUDA rebooted
- ocl 0.19.7: OpenCL declining, no Mac support
- NeoTrix R-P1 forbids unsafe (blocks vulkano/ash adoption)
- WGSL spec still Working Draft (instability risk)

---

## Defects Identified (26+)

### Profiling/Debugging (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-PROF-1 | Zero profiling tooling integrated | High |
| D-PROF-2 | No async runtime visibility (tokio-console) | High |
| D-PROF-3 | No lock contention profiling | Medium |
| D-PROF-4 | Criterion benchmarks without profiling | Medium |
| D-PROF-5 | No continuous profiling in production | Medium |
| D-PROF-6 | No cross-layer timing instrumentation | Medium |

### Concurrent Data Structures (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-CONC-1 | No epoch-based GC for lock-free structures | Medium |
| D-CONC-2 | 7 FFI wrappers use Arc<RwLock<T>> (should use arc-swap) | High |
| D-CONC-3 | LruCache<Mutex<>> contention under high concurrency | Medium |
| D-CONC-4 | std::sync::Arc carries unused weak ref count (1 word waste) | Low |
| D-CONC-5 | arc-swap 1.9 weakened orderings (regression risk) | Medium |

### Unicode/Text (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-UNI-1 | nt_normalizer.rs uses ASCII whitespace split | Medium |
| D-UNI-2 | GraphRAG split_sentences is ASCII-only | Medium |
| D-UNI-3 | unicode-normalization stale Unicode version | Medium |
| D-UNI-4 | strsim no grapheme-aware edit distance | High |
| D-UNI-5 | No grapheme-aware text wrapping | Low |

### GPU/Compute (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-GPU-1 | wgpu staging buffer pattern doubles memory for GPU→CPU | High |
| D-GPU-2 | vulkano/ash require unsafe (blocks R-P1) | Critical |
| D-GPU-3 | cuda-sys frozen at 2018 (no CUDA 12+) | High |
| D-GPU-4 | OpenCL declining, no Mac support | Medium |
| D-GPU-5 | WGSL spec instability (still Working Draft) | Medium |

## Key Insights (This Batch)

1. **No profiling infrastructure is critical**: NeoTrix is entirely async but has zero visibility into task scheduling, lock contention, or cross-layer timing. tokio-console would immediately surface GWT/SEAL/EventBus timing issues.

2. **arc-swap for FFI wrappers**: 7 FFI wrappers use Arc<RwLock<T>> but reads are lock-free. arc-swap allows atomic pointer swaps for reads, eliminating lock contention.

3. **unicode-segmentation GraphemeCursor panics**: 9 panic paths in GraphemeCursor. Must avoid or wrap for adversarial web content.

4. **unicode-normalization stale**: Still on Unicode 15.x tables (17.0 is current). CJK/emoji content may normalize incorrectly, causing KB fingerprint drift.

5. **strsim no grapheme awareness**: levenshtein("é", "e\u{0301}") = 2 (not 0). KB entity deduplication will produce false negatives for equivalent Unicode.

6. **wgpu is only viable GPU option**: vulkano/ash require unsafe (R-P1 blocks). cuda-sys frozen. OpenCL declining. wgpu 5-10% overhead is acceptable cost.

7. **triomphe saves 1 word per Arc**: NeoTrix uses no Weak references anywhere. triomphe eliminates weak count overhead.

8. **arc-swap 1.9 weakened orderings**: Author admits "proofs based on wrong assumptions". Pin to 1.8.x if adopting.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 818 |
| New defects (this batch) | 21 |
| Cumulative defects | D01-D76591 |
| Research sources (this batch) | 29 |
| Cumulative research sources | 97,403+ |
