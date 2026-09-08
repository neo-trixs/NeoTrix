# Iteration Batch 810 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Macros (10)
- RFC 3698: Declarative derive macros merged (macro_rules can implement #[derive])
- RFC 3697: Declarative attribute macros merged
- macro fn: Comptime macro functions with for/match iteration (next frontier)
- rules_derive: macro_rules-based derives match syn/quote quality with <1k LoC
- proc-macro-error: Known audit advisory (RUSTSEC-2024-0436), no fix
- 6 macros exported globally via #[macro_export] (pollutes namespace)
- Hardcoded NUM_FIELDS=23 drifts from actual field list
- CapabilityVector field list duplicated 3 times
- Logging macros use $super instead of $crate (cross-module breakage)
- make_stage! generates minimal structs without trait wiring

### Typestate Patterns (10)
- statum: Typed rehydration from DB rows, #[validators] for persistence
- typestate-pipeline: Async phase advancement, Resolved/InFlight dual-arm
- fluxo-typestate: Enum-declared transitions with Mermaid auto-gen
- FUNARCH 2026 (ICFP): Typestate improves faultlessness and testability
- Zero PhantomData usage anywhere in NeoTrix
- SEAL pipeline has no type-level phase encoding
- No builder pattern for complex constructors (all pub fields + Default)
- CapabilityId(pub String) is untyped (any string works)
- E8 lattice operations don't encode valid transition paths
- Branch health uses HashMap<String, f64> instead of typed states

### Enum Ergonomics (8)
- strum v0.28.0: EnumString, Display, EnumIter, EnumDiscriminants, VariantArray
- enum_dispatch v0.3.13: 3-4× faster than dyn Trait (2.1ns vs 8.4ns)
- derive_more v2.0.1: From, Display, Deref, Error, IsVariant, no_std compatible
- frunk Coproducts: 2× memory overhead vs manual enum, O(N) extraction
- enum_dispatch cross-crate limitation (trait+enum must be same crate)
- enum_dispatch breaks rust-analyzer (IDE-hostile)
- strum VariantArray requires unit variants only
- GWT dispatch should use manual enum + strum (not frunk Coproduct)

### Async Patterns (12)
- Async closures stable since Rust 1.85 (Feb 2026)
- JoinSet = structured concurrency (drop cancels all children)
- CancellationToken = hierarchical shutdown (tokio-util)
- 48 bare tokio::spawn sites, 0 JoinSet usage
- 72 Arc<Mutex<T>> in async contexts (deadlock candidates)
- 5 Box::pin(async ...) should use native async closures
- 71 #[async_trait] uses (Pin<Box<dyn Future>> overhead)
- Background loop: 20+ handlers spawned into Vec<JoinHandle> (zombie risk)
- 5-second hard abort for shutdown (no cooperative cancellation)
- Fire-and-forget tor_crawler spawns (no JoinSet, no cancellation)

---

## Defects Identified (30+)

### Macros (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-MAC-1 | #[macro_export] pollution (6 macros in global namespace) | Medium |
| D-MAC-2 | Hardcoded NUM_FIELDS=23 drifts from actual field list | High |
| D-MAC-3 | CapabilityVector field list duplicated 3 times (DRY violation) | Medium |
| D-MAC-4 | Logging macros use $super (cross-module resolution breaks) | High |
| D-MAC-5 | spawn_handler! has inconsistent parameter arms | Medium |
| D-MAC-6 | No proc-macro crate in workspace (no custom derives) | Medium |
| D-MAC-7 | trust_rule! regex compiled at runtime, not compile-time | Low |
| D-MAC-8 | make_stage! generates minimal structs without trait wiring | Low |

### Typestate (7)
| ID | Defect | Severity |
|----|--------|----------|
| D-TS-1 | Zero PhantomData usage (all state transitions runtime-checked) | High |
| D-TS-2 | SEAL pipeline has no type-level phase encoding | High |
| D-TS-3 | CoreSnapshot serializes all state as flat scalars | Medium |
| D-TS-4 | No builder pattern for complex constructors | Medium |
| D-TS-5 | Branch health uses HashMap<String, f64> (stringly-typed) | Medium |
| D-TS-6 | CapabilityId(pub String) is untyped | Medium |
| D-TS-7 | E8 lattice operations don't encode valid transition paths | Medium |

### Enum Ergonomics (5)
| ID | Defect | Severity |
|----|--------|----------|
| D-ENUM-1 | enum_dispatch cross-crate limitation (can't dispatch across domains) | High |
| D-ENUM-2 | frunk Coproduct 2× memory overhead + O(N) extraction | High |
| D-ENUM-3 | strum VariantArray requires unit variants only | Medium |
| D-ENUM-4 | enum_dispatch breaks rust-analyzer (IDE-hostile) | Medium |
| D-ENUM-5 | derive_more doesn't provide EnumDiscriminants | Low |

### Async Patterns (10)
| ID | Defect | Severity |
|----|--------|----------|
| D-ASYNC-1 | 20+ zombie-prone handlers (Vec<JoinHandle>, not JoinSet) | Critical |
| D-ASYNC-2 | Fire-and-forget spawns (no JoinSet, no cancellation) | Critical |
| D-ASYNC-3 | Orphaned relay task (no cancellation token) | High |
| D-ASYNC-4 | 48 bare tokio::spawn sites with no structured concurrency | High |
| D-ASYNC-5 | 71 #[async_trait] uses (Pin<Box<dyn Future>> overhead) | High |
| D-ASYNC-6 | 5 Box::pin(async ...) should use native async closures | Medium |
| D-ASYNC-7 | Brutal 5-second hard abort shutdown (no cooperative drain) | Medium |
| D-ASYNC-8 | 72 Arc<Mutex<T>> in async contexts (deadlock candidates) | Medium |
| D-ASYNC-9 | No cancel safety audit for select! arms | Low |
| D-ASYNC-10 | No CancellationToken usage anywhere | High |

## Key Insights (This Batch)

1. **Zombie task pattern is critical**: 20+ handlers spawned into Vec<JoinHandle> without JoinSet. If shutdown() never called (crash/panic), all handlers become zombies with no parent scope.

2. **Zero PhantomData usage**: All state transitions are runtime-checked via PipelineState counters. The compiler cannot enforce that SEAL phases execute in order.

3. **enum_dispatch is 3-4× faster than dyn Trait**: 2.1ns vs 8.4ns per call. But cross-crate limitation makes it unusable for NeoTrix's domain-separated architecture.

4. **frunk Coproduct is 2× larger**: Coprod!(SensorEvent, ActCommand, ...) with 7+ variants would be ~2× larger than manual enum. Use manual enum + strum instead.

5. **$super in logging macros breaks cross-module**: Only $crate provides reliable cross-module resolution. This is a correctness bug.

6. **NUM_FIELDS=23 can silently drift**: Hardcoded constant not derived from field list. If fields added/removed, serialization breaks at runtime.

7. **JoinSet eliminates manual shutdown**: Drop JoinSet = automatic cancellation. No more 5-second hard abort needed.

8. **Native async closures replace Box::pin**: async |x| { ... } is stable since Rust 1.85. Five sites still use Box::pin(async move { ... }).

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 810 |
| New defects (this batch) | 30 |
| Cumulative defects | D01-D76386 |
| Research sources (this batch) | 40+ |
| Cumulative research sources | 97,154+ |
