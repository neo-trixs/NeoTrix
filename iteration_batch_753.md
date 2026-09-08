# Iteration Batch 753 — Global Allocator Architecture Audit

**Date**: 2026-09-07
**Research Loop**: 753/10000+
**Focus**: Memory allocator architecture, global allocator selection, allocation lifecycle patterns
**Sources**: 12 web sources (2026-dated), Rust stdlib docs, arxiv paper

---

## Batch 752 Carryover (Unresolved)

| # | Defect | Status |
|---|--------|--------|
| 1 | No connection pool for KB | UNRESOLVED |
| 2 | No WAL-reset bug protection | UNRESOLVED |
| 3 | PRAGMA optimize causes production crashes | UNRESOLVED |
| 4 | No connection health check | UNRESOLVED |
| 5 | Row-by-row operations before WAL | UNRESOLVED |

---

## NEW Defects Found in Batch 753

### DEFECT-753-1: CRITICAL — No Global Allocator Declared

NeoTrix uses Rust's **unspecified default allocator** (platform-dependent, usually glibc ptmalloc2 on Linux). No `#[global_allocator]` is declared anywhere in the crate graph.

**Evidence**: `GlobalAlloc` trait is stable since Rust 1.28. Every production Rust service in 2026 declares an explicit allocator. The default ptmalloc2 was designed for 4-core machines and exhibits severe fragmentation under 64+ thread concurrent workloads (NeoTrix has multi-domain concurrent processing via EventBus).

**Impact**: 
- P99 latency degradation under concurrent KB operations (cross-thread frees hit ptmalloc2's single arena lock)
- RSS growth over long uptimes (ptmalloc2 poor at returning pages)
- KB connection pool operations (Batch 752 #1) would amplify this since each connection's allocations contend on the same arena

**Source**: Ganglani 2026 (kunalganglani.com), mimalloc Deep Dive 2026 (braindetox.kr), Microsoft Research blog 2026

**Fix Required**: Add `mimalloc = "0.1"` to Cargo.toml, declare `#[global_allocator]` in neotrix-core main entrypoint.

---

### DEFECT-753-2: CRITICAL — No Arena/Region Allocation for KB Operations

KB operations have **predictable lifetimes** — query params, result sets, embeddings — but use generic `Vec<Box<str>>` heap allocation. Region/arena allocators provide up to **15% speedup** even over mimalloc (arxiv 2605.17119, 2026).

**Evidence**: The paper "Reconsidering Custom Memory Allocation" (Berger et al. revisited, 2026) shows:
- Region allocators still outperform even mimalloc by up to 15% on clean heap
- Under **adversarial fragmentation**, naive allocation slows by **2x** while regions are **unaffected**
- NeoTrix KB operations create long-lived embeddings + short-lived query intermediates — classic region pattern

**Impact**: KB search/embed operations allocate and free per-query, creating fragmentation that degrades over time. Under concurrent access this compounds.

**Fix Required**: Implement `bumpalo` or `typed-arena` for KB query lifetimes. Allocate query params + result marshalling in arena, bulk-free after query completes.

**Source**: van Kempen & Berger, arxiv 2605.17119v1 (2026-05-16)

---

### DEFECT-753-3: HIGH — No Allocator Observability Integration

jemalloc provides `MALLOC_CONF` with `malloc_stats_print()` and `mallctl` for production introspection. mimalloc provides `mi_stats_print()`. NeoTrix has **zero allocator-level metrics** feeding into GWT or HeartbeatAggregator.

**Evidence**: 
- jemalloc's `dirty_decay_ms` and `muzzy_decay_ms` controls RSS vs tail latency tradeoff
- mimalloc's secure mode adds 3-5% overhead for heap hardening (free-list pointer encryption, guard pages)
- Without allocator stats, the HeartbeatAggregator cannot detect memory pressure vs CPU pressure — it conflates them

**Impact**: SystemHealthSnapshot cannot distinguish "allocator fragmentation growing" from "too many concurrent tasks". Self-healing (NT-REPAIR) cannot trigger appropriate remediation.

**Source**: Ganglani 2026, Microsoft Research blog 2026-05-13

---

### DEFECT-753-4: HIGH — tcmalloc Thread Cache Risk for KB Connection Pool

If tcmalloc is chosen (it shouldn't be for NeoTrix, but worth documenting), thread-local caches **retain memory** that can blow up container limits and trigger OOM — the "latency optimization that kills you."

**Evidence**: Google tcmalloc documentation, Ganglani 2026. tcmalloc thread caches can retain memory proportional to `num_threads × cache_size`. For NeoTrix's 7-domain concurrent architecture, this could mean 7× retention of KB result sets.

**Impact**: RSS surprise under high concurrency → container OOM → production crash. This is the exact failure mode that Batch 752 #3 (PRAGMA optimize crashes) would trigger if tcmalloc were used — the crash would be misattributed to SQLite rather than allocator retention.

**Source**: Ganglani 2026, StratCraft NexusFIX benchmarks 2026-03-12

---

### DEFECT-753-5: MEDIUM — Secure Mode Not Evaluated for Egress Privacy Guard

mimalloc's secure mode (`-DMI_SECURE=ON`) provides free-list pointer encryption, guard pages, and double-free detection at 3-5% overhead. NeoTrix's Egress Privacy Guard handles outbound LLM requests with sensitive data — heap hardening would protect against heap spraying attacks on the guard itself.

**Evidence**: mimalloc secure mode is "the cheapest production-grade heap hardening in open-source allocator space" (braindetox.kr deep dive 2026).

**Impact**: Without heap hardening, a compromised dependency could spray the heap to corrupt Egress Privacy Guard's trust tier decisions, potentially downgrading `Untrusted`→`Trusted`.

**Source**: braindetox.kr mimalloc deep dive 2026-05-25

---

### DEFECT-753-6: MEDIUM — `allocator_api` Still Unstable After 6+ Years

The `Allocator` trait (per-collection allocator) is still nightly-only. `allocator_api2` crate exists as a bridge (25M+ downloads). NeoTrix cannot use per-collection allocation on stable Rust.

**Evidence**: cetra3.github.io "State of Allocators in 2026" — consensus not reached, trait split debates ongoing. `Vec<T, A>` and `Box<T, A>` generic parameters exist on nightly.

**Impact**: Cannot assign different allocators to different subsystems (e.g., arena for KB queries, mimalloc for general, jemalloc for long-lived caches). Forced into single global allocator choice.

**Source**: cetra3.github.io (2026-03-11), Rust RFC 1974

---

### DEFECT-753-7: LOW — No NUMA-Aware Allocation for Multi-Core KB

mimalloc v3.x is improving NUMA awareness. jemalloc has been strong here for years. NeoTrix's KB operations on multi-socket machines (server deployments) do not consider NUMA node locality for connection pool placement.

**Evidence**: Microsoft Research 2026 — mimalloc v3.x NUMA story improving. jemalloc has `arena_t` per-NUMA-node. NeoTrix KB connections are created without NUMA affinity.

**Impact**: Cross NUMA-node KB access adds 2-3x memory latency. Under high-throughput KB operations this degrades P99.

**Source**: Microsoft Research blog 2026-05-13

---

## Summary Table

| # | Defect | Severity | Category | Fix Complexity |
|---|--------|----------|----------|----------------|
| 753-1 | No global allocator declared | CRITICAL | Performance | Low (1 line + Cargo.toml) |
| 753-2 | No arena/region for KB ops | CRITICAL | Performance | Medium (arena lifecycle) |
| 753-3 | No allocator observability | HIGH | Observability | Medium (stats bridge) |
| 753-4 | tcmalloc thread cache risk | HIGH | Reliability | Low (don't use tcmalloc) |
| 753-5 | Secure mode not evaluated | MEDIUM | Security | Low (build flag) |
| 753-6 | allocator_api unstable | MEDIUM | Architecture | N/A (upstream blocker) |
| 753-7 | No NUMA-aware allocation | LOW | Performance | High (NUMA topology) |

---

## Key Research Findings (New)

1. **mimalloc v3.3.2** (2026-04-29) is the current stable — Rust wrapper has 100K+ daily downloads
2. **jemalloc 5.3.1** — Meta just assumed control, first release in 4 years (2026-04)
3. **Region allocators still beat GP allocators by up to 15%** — the gap narrowed from 44% (2002) but is real
4. **Allocator choice moves P99 by double-digit percentages** — multiple independent sources confirm
5. **mimalloc secure mode** — 3-5% overhead for production-grade heap hardening (free-list encryption + guard pages)
6. **Per-class allocators provide NO benefit** over modern GP allocators — region allocators are the only custom strategy worth implementing
7. **Adversarial fragmentation** causes naive allocation to slow by 2x — region allocators unaffected. This directly applies to NeoTrix's long-running KB operations

---

## Recommended Priority Order

1. **Add mimalloc as global allocator** — 5 minutes of work, 5-30% P99 improvement expected
2. **Implement arena allocation for KB query lifetimes** — region pattern for predictable lifetimes
3. **Bridge allocator stats into HeartbeatAggregator** — memory pressure visibility
4. **Evaluate mimalloc secure mode** for Egress Privacy Guard protection
5. **Document NUMA topology** for server deployments (deferred)
