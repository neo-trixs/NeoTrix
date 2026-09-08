# Agent 1: Memory Model Patterns (Batch 861)

## Sources
1. **The Rust Programming Language** - Ownership and Move Semantics (ch04-01)
2. **Rust Patterns Book** - Memory & Ownership Patterns (Pattern 1-12)
3. **Rustonomicon** - Ownership, Lifetime Elision, PhantomData, Unchecked Uninitialized Memory
4. **Rust Reference** - Lifetime Elision Rules
5. **Rust RFC 0141** - Lifetime Elision RFC
6. **std::pin Documentation** - Pin, Self-Referential Structs, PhantomPinned
7. **EliteDev** - 8 Battle-Tested Unsafe Patterns
8. **Medium/Techkoala** - 5 Unsafe Patterns Causing Silent Memory Corruption
9. **Stanford** - Unsafe Rust Syntactic Patterns
10. **Rust Compiler Dev Guide** - Borrow Checker, Non-Lexical Lifetimes, Two-Phase Borrows
11. **Rustonomicon** - Limits of Lifetimes
12. **Rust Project Goals 2026** - The Borrow Checker Within (Polonius)

## Defects

### D-MEM-001: Dispatcher Waterfall Recursive Stack Overflow Risk | neotrix-core/src/unified/core/nt_core_dispatch.rs:117 | HIGH | Source: Rustonomicon ownership patterns + Stack depth limits
The `dispatch_chain` function uses unbounded recursion for waterfall middleware. With N handlers, this creates N stack frames. A malicious or pathological configuration with 1000+ handlers causes stack overflow. The recursion pattern (`dispatch_chain` calling itself) has no depth guard. **Fix**: Convert to iterative loop with explicit stack or limit recursion depth.

### D-MEM-002: RevertibleContext Closure Lifetime Mismatch | neotrix-core/src/unified/core/nt_core_context/revertible.rs:24-31 | MEDIUM | Source: Rust RFC 0141 lifetime elision + Rustonomicon limits of lifetimes
`ClosureEffect<S>` stores `Box<dyn Fn(&mut S) + Send + Sync>` with implicit `'static` bound, but `RevertibleContext<'a, S>` has lifetime `'a` for effects. When `S` is a borrowed type (e.g., `&mut Registry`), the closure captures may not live long enough. The `'a` bound on `effects: Vec<Box<dyn RevertibleEffect<S> + 'a>>` does not propagate to the inner closure's capture lifetime, creating potential dangling pointer if closures outlive the borrow. **Fix**: Add explicit lifetime bound on closure captures or use `Arc` for long-lived state.

### D-MEM-003: PeekedStream AsyncRead Self-Referential Pin Safety | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:10-25 | MEDIUM | Source: std::pin documentation + self-referential struct patterns
`PeekedStream<T>` wraps an inner `T` and stores `peeked: Vec<u8>` with `pos: usize`. The `poll_read` implementation uses `self.get_mut()` which bypasses pin safety guarantees. While `T: Unpin` is required, the `peeked` buffer and `pos` counter create implicit self-referential state (pos references into peeked). If the struct is moved after partial peek consumption, the position becomes invalid relative to the buffer. **Fix**: Pin the inner `T` field separately or use `Pin<Box<Self>>` construction pattern.

### D-MEM-004: Static OnceLock Mutex Potential Deadlock | neotrix-core/src/unified/core/nt_core_self_review/scanners.rs:34-35 | LOW | Source: Rust unsafe patterns + thread safety analysis
Global static `CACHE: OnceLock<Mutex<HashMap<...>>>` pattern is used extensively. If a function holding the Mutex guard calls another function that tries to acquire the same Mutex, deadlock occurs. The `cached_rs_files` and `production_lines_cached` functions both use this pattern but call other functions that might trigger cache lookup. **Fix**: Use `try_lock()` with fallback or restructure to avoid nested acquisition.

### D-MEM-005: Arc Reference Cycle Risk in WisdomBridge | neotrix-core/src/unified/core/l7_capability/consciousness_bridge.rs:48-84 | MEDIUM | Source: Rust Patterns Book - Breaking Cycles with Weak
`WisdomBridge` stores `bus: RwLock<Option<Arc<NativeBusHandle>>>` and the `NativeBusHandle` contains `Arc<std::sync::Mutex<NativeBus>>`. If `NativeBus` holds a reference back to `WisdomBridge` (even indirectly through callbacks), this creates an Arc reference cycle causing memory leak. The `attach_bus` method takes `Arc<NativeBusHandle>` without `Weak` option. **Fix**: Use `Weak<NativeBusHandle>` for back-references or audit the ownership graph for cycles.

### D-MEM-006: PhantomData Missing for Type-Safe Resource Handles | neotrix-core/src/unified/core/nt_core_meta/self_model.rs:110 | LOW | Source: PhantomData documentation + lifetime branding
`ModuleInfo` struct tracks `unsafe_count: usize` but does not use `PhantomData` to enforce module identity or prevent cross-module confusion. The `has_unsafe: bool` field at line 123 is redundant with `unsafe_count > 0` check. This creates potential for confusion when comparing modules from different subsystems. **Fix**: Add `PhantomData<*const ()>` marker to make modules `!Send + !Sync` if they contain thread-unsafe state, or use typestate pattern.

### D-MEM-007: SelfCodeMonitor Clone Derivation on Active State | neotrix-core/src/unified/core/nt_core_iter/self_ref_code.rs:65-70 | MEDIUM | Source: Rust ownership patterns + Clone semantics
`SelfCodeMonitor` derives `Clone` but contains `active_mutations: HashMap<String, MutationResult>`. Cloning this struct creates independent copies of mutation state, potentially allowing concurrent modification without synchronization. The `record_result` method modifies `active_mutations` without any locking, so cloning + concurrent mutation causes data race in multi-threaded context. **Fix**: Remove `Clone` derive or wrap internal state in `Arc<RwLock<...>>`.

### D-MEM-008: SelfReferentialMonitor Unbounded History Growth | neotrix-core/src/unified/core/nt_core_self/self_referential.rs:18 | LOW | Source: Rust memory patterns + resource management
`SelfReferentialMonitor` has `plan_history: Vec<PlanRecord>` with no maximum size limit. The `auto_tune` method pushes new records without eviction. Over long-running sessions, this vector grows unbounded, causing memory pressure. The `plan_quality_trend` method only looks at last 10 records but stores all history. **Fix**: Add `max_history` field similar to `SelfCodeMonitor` and implement FIFO eviction.

### D-MEM-009: Dispatcher Handler Vec Reallocation Invalidation | neotrix-core/src/unified/core/nt_core_dispatch.rs:66 | MEDIUM | Source: Rustonomicon ownership + invalidation patterns
`Dispatcher::register` pushes handlers to a `Vec<Box<dyn Fn(...)>>` which may reallocate. If a handler holds a reference to the dispatcher's handler slice (obtained before reallocation), that reference becomes dangling. The `dispatch` method borrows `self.handlers` immutably, but `register` borrows mutably - this is safe at compile time, but if handlers store any state referencing the handler indices, those indices become stale after reallocation. **Fix**: Use stable indices (return `usize` and document invalidation) or use `Arena` allocation.

### D-MEM-010: PeekedStream AsyncWrite Lacks Pin Projection Safety | neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_traffic/mitm.rs:37-52 | MEDIUM | Source: std::pin Pin projection patterns
The `AsyncWrite` implementation for `PeekedStream<T>` uses `Pin::new(&mut self.get_mut().inner)` which is correct for `Unpin` types, but does not use `pin-project` crate for safe pin projection. If `T` is changed to not implement `Unpin` in the future, this code silently becomes unsound. The manual pin projection bypasses the compiler's pin safety checks. **Fix**: Use `pin-project` or `pin-project-lite` crate for safe pin projection, or add `where T: Unpin` bound explicitly.

## Key Insights

1. **Recursion vs Iteration**: The waterfall dispatch pattern should be converted to iterative to prevent stack overflow in pathological cases.

2. **Arc Cycle Prevention**: The codebase heavily uses `Arc<RwLock<...>>` for shared state. Without `Weak` references for back-edges, memory leaks are possible in complex object graphs.

3. **Pin Safety**: The `PeekedStream` implementation manually projects pins without `pin-project`, creating maintenance risk if inner type bounds change.

4. **Clone Semantics on Mutable State**: Deriving `Clone` on structs with mutable internal state (like `SelfCodeMonitor`) can lead to subtle data races in multi-threaded contexts.

5. **Lifetime Elision Boundaries**: The `RevertibleContext` pattern mixes lifetimes with closure captures, requiring careful analysis to ensure captures don't outlive the borrowed state.

6. **Static Global State**: Extensive use of `OnceLock<Mutex<...>>` for global caches requires careful deadlock analysis, especially when functions may call each other while holding locks.

7. **PhantomData for Type Safety**: The codebase lacks `PhantomData` usage for type-level guarantees, relying instead on runtime checks where compile-time enforcement would be safer.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| New defects (this batch) | 10 |
| D-MEM-001 through D-MEM-010 | 10 |
| Critical | 0 |
| High | 1 |
| Medium | 6 |
| Low | 3 |
