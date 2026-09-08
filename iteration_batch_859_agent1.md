# Agent 1: Rust Async Closure Patterns (Batch 859)

## Sources
- Rust RFC 3668: Async Closures (https://rust-lang.github.io/rfcs/3668-async-closures.html)
- Rust RFC 3654: Return Type Notation (https://rust-lang.github.io/rfcs/3654-return-type-notation.html)
- Rust Project Goals 2026: Prepare TAIT + RTN (https://goals.rust-lang.org/2026/rtn.html)
- Rust Project Goals 2026: Native async fn dynamic dispatch in traits (https://goals.rust-lang.org/2026/afidt-box.html)
- Rust Project Goals 2026: Async statemachine optimisation (https://rust-lang.github.io/goals/2026/async-statemachine-optimisation.html)
- Async Closures Stabilization Report (https://github.com/rust-lang/rust/pull/132706)
- Async Drop Tracking Issue (https://github.com/rust-lang/rust/issues/126482)
- Dyn async traits, part 10: Box box box (https://smallcultfollowing.com/babysteps/blog/2025/03/24/box-box-box/)
- Async Rust never left the MVP state (https://tweedegolf.nl/en/blog/237/async-rust-never-left-the-mvp-state/)
- Microsoft Rust Training: Async Traits (https://microsoft.github.io/RustTraining/async-book/ch10-async-traits.html)
- Wren Learns Rust: The Async Trait Problem (https://wrenlearnsrust.com/posts/async-traits-2026.html)
- Rust Internals: Specifying bounds on Futures (https://internals.rust-lang.org/t/specifying-bounds-on-futures-returned-by-async-functions/19740)

## Defects

### D-ASYNC-001: NeoTrix uses `async_trait` crate extensively (52+ usages) but Rust 1.75+ provides native `async fn` in traits (AFIT) with zero-cost static dispatch | Multiple files | medium | Source: Microsoft Rust Training, Wren Learns Rust

**Evidence**: 52 occurrences of `#[async_trait::async_trait]` across NeoTrix codebase (nt_core_llm.rs:31, traits.rs:6, gateway/mod.rs:160, proxy_pool.rs:7, etc.). The `async_trait` crate desugars to `Pin<Box<dyn Future>>` on every call, adding heap allocation overhead. Native AFIT is stable since Rust 1.75 and provides zero-cost static dispatch.

**Impact**: Every `async_trait`-annotated method incurs a heap allocation per call. In performance-critical paths (LLM provider calls, proxy pool health checks, sandbox execution), this adds unnecessary latency and memory pressure.

**Recommendation**: Migrate from `#[async_trait]` to native `async fn` in traits for static dispatch paths. Use `trait_variant::make(SendXxx: Send)` for Send bounds when needed. Reserve `async_trait` only for genuine `dyn Trait` dispatch scenarios (which are currently few in NeoTrix).

### D-ASYNC-002: NeoTrix uses `Box::pin(async move { ... })` for closure-based async probes, but Rust 1.85+ async closures (`async || {}`) eliminate this pattern | nt_world_osint/backend_router.rs:152-179 | low | Source: RFC 3668

**Evidence**: `backend_router.rs:152-179` uses `Box::pin(async move { ... })` closures as probe functions. The `Backend` struct at line 16-19 stores `for<'a> fn(...) -> Pin<Box<dyn Future<...> + Send + 'a>>` which requires explicit boxing.

**Impact**: Manual boxing adds verbosity and allocation overhead. With Rust 1.85+ async closures, these could be expressed as `async |target, client| { ... }` with the `AsyncFn` trait, eliminating the `Box::pin` wrapper and enabling lending borrows from captures.

**Recommendation**: When Rust edition 2024 and async closures are adopted, refactor `Backend.probe` to use `AsyncFn` bounds. For now, the current pattern is functional but verbose.

### D-ASYNC-003: NeoTrix has no mechanism to express `Send` bounds on async trait method futures, blocking safe `tokio::spawn` usage | nt_core_llm.rs:32, multiple trait definitions | high | Source: RTN RFC 3654, Rust Project Goals 2026

**Evidence**: The `LlmProvider` trait at `nt_core_llm.rs:32` uses `#[async_trait]` which implicitly adds `Send` to futures. However, native AFIT traits like `CapabilityNode`, `SkillRunner`, `EngineProvider` (nt_core_traits.rs) use `async_trait` without explicit Send bounds. The Send bound problem means generic code cannot require `T::method(..): Send` for spawning on multi-threaded executors.

**Impact**: If NeoTrix migrates to native AFIT, it loses the ability to require Send futures at trait boundaries. This would break `tokio::spawn` calls that depend on trait method futures being Send. Currently masked by `async_trait` always boxing to `Pin<Box<dyn Future + Send>>`.

**Recommendation**: Prepare for RTN (Return Type Notation) stabilization. When migrating to native AFIT, use `trait_variant::make(SendXxx: Send)` to generate Send variants, or adopt RTN bounds like `T::method(..): Send + 'static` at call sites.

### D-ASYNC-004: NeoTrix's `Pin<Box<dyn Future<...> + Send + 'a>>` pattern in `BackendRouter` creates lifetime-dependent future types that prevent async closure migration | nt_world_osint/backend_router.rs:16-19 | medium | Source: RFC 3668, async closures stabilization

**Evidence**: `Backend.probe` signature at line 16-19: `for<'a> fn(&'a OsintTarget, &'a Client) -> Pin<Box<dyn Future<Output = Result<Box<dyn BackendResult>, BackendError>> + Send + 'a>>`. The higher-ranked lifetime `for<'a>` with `+ 'a` creates a lending future that borrows from inputs.

**Impact**: This pattern requires explicit lifetime management and boxing. Async closures in Rust 1.85+ natively support lending (borrowing from captures), but the `for<'a>` HRTB pattern with `Pin<Box<dyn Future + Send + 'a>>` is a workaround for the pre-1.85 limitation.

**Recommendation**: When adopting async closures, the `Backend` struct could use `AsyncFn(&OsintTarget, &Client) -> impl Future<Output = Result<Box<dyn BackendResult>, BackendError>>` bounds, eliminating the manual boxing and lifetime annotations.

### D-ASYNC-005: NeoTrix lacks async drop support for resources requiring async cleanup (network connections, file handles in async contexts) | Global | medium | Source: Async Drop Tracking Issue #126482

**Evidence**: NeoTrix manages network resources (proxy connections in `nt_shield_proxy_kernel`, stealth browser connections, file handles in `nt_file_ability`). The `AsyncDrop` trait (nightly only, issue #126482) is not available on stable Rust, and NeoTrix has no workaround pattern for async resource cleanup.

**Impact**: Resources requiring async cleanup (e.g., closing network connections gracefully, flushing async file buffers) must rely on sync `Drop` which cannot `.await`. This can lead to resource leaks or incomplete cleanup in async contexts.

**Recommendation**: Design a manual async cleanup pattern (e.g., `async fn close(&self) -> Result<()>` called before drop) until `AsyncDrop` stabilizes. Document the pattern in CONTEXT.md. Monitor Rust 1.85+ async drop progress.

### D-ASYNC-006: NeoTrix's async state machine futures may have suboptimal size due to deep future nesting in coordinator/executor patterns | nt_core_parallel/coordinator.rs:60, executor.rs:28 | medium | Source: Async statemachine optimisation goal, Tweede Golf analysis

**Evidence**: `coordinator.rs:60` (`execute_tasks`) and `executor.rs:28` (`execute`) spawn tasks via `tokio::spawn(async move { ... })`. The coordinator creates nested futures (coordinator → agent tasks → LLM calls). Each nesting level adds a state machine variant, increasing future size. The Rust compiler currently does not optimize away single-await state machines or collapse identical states.

**Impact**: Deep future nesting in NeoTrix's parallel execution paths (coordinator spawning agent tasks that call LLM providers) can lead to larger-than-necessary state machines, increasing stack usage and reducing cache efficiency.

**Recommendation**: Apply the "last await inlining" optimization pattern manually: extract the final await into a separate function to reduce state machine depth. Monitor the Rust async statemachine optimisation goal (2026) for compiler-level fixes.

### D-ASYNC-007: NeoTrix uses `#[async_trait]` on traits that could benefit from method-scope dyn compatibility (Phase 1 of AFIDT) | nt_core_llm.rs:31, nt_core_gate/mod.rs:706, multiple | low | Source: Native async fn dynamic dispatch in traits goal

**Evidence**: Traits like `LlmProvider` (nt_core_llm.rs:31), `AsyncPanelJudge` (nt_core_gate/mod.rs:706), and others use `#[async_trait]` which forces all methods through `Pin<Box<dyn Future>>`. The AFIDT goal (2026-2027) introduces method-scope dyn compatibility where only incompatible methods are excluded from vtable, not the whole trait.

**Impact**: Current `async_trait` desugaring forces boxing even for methods that don't need dynamic dispatch. Method-scope dyn compatibility would allow traits with async methods to be `dyn`-compatible while keeping static dispatch for non-async methods.

**Recommendation**: When method-scope dyn compatibility stabilizes, evaluate which traits genuinely need `dyn` dispatch and which can use native AFIT. Reserve `async_trait` for traits that actually require dynamic dispatch.

## Key Insights

1. **Migration Path**: NeoTrix has 52+ `#[async_trait]` usages that should be systematically migrated to native AFIT (Rust 1.75+) for zero-cost static dispatch. The `async_trait` crate is now a legacy pattern for most use cases.

2. **Send Bounds Gap**: The Send bound problem (inability to express `T::method(..): Send` with native AFIT) is the primary blocker for migration. RTN (Return Type Notation) will solve this but is blocked on next-gen trait solver work (expected late 2026).

3. **Async Closures Opportunity**: Rust 1.85+ async closures (`async || {}`) with `AsyncFn` traits could simplify callback patterns like `BackendRouter.probe`, eliminating `Box::pin(async move { ... })` wrappers.

4. **Async Drop Gap**: NeoTrix manages network/file resources that would benefit from `AsyncDrop`, but this feature is nightly-only with incomplete `dyn` support. Manual async cleanup patterns are needed.

5. **State Machine Bloat**: Deep future nesting in coordinator/executor patterns creates large state machines. Manual optimization (extracting last awaits) and monitoring compiler improvements can help.

6. **Dyn Dispatch Future**: The `.box` notation for async trait dyn dispatch (2026-2027) will eventually replace `async_trait` for genuine dynamic dispatch needs, providing explicit allocation control.

## Cumulative Totals
| Metric | Value |
|--------|-------|
| New defects (this batch) | 7 |
| Sources analyzed | 12 |
| Rust features identified | 6 (AFIT, RTN, Async Closures, Async Drop, AFIDT, Async State Machine Opt) |
| NeoTrix files with async patterns | 52+ (async_trait), 70+ (async move), 6 (Pin<Box<dyn Future>>) |
