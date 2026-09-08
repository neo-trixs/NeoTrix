# Iteration Batch 833 Report — NeoTrix Consciousness Architecture

## Research Sources (40+)

### Send/Sync Patterns (10)
- Auto traits don't flow through async traits — Send/Sync bounds on async fn in trait methods remain tricky
- trait_variant::make(SendFoo: Send) auto-generates Send variant for static dispatch
- "Future is not Send" errors are always structural — fix is never fighting the type system
- 3 patterns: scope non-Send values before .await, extract sync work, replace with Send-safe alternatives
- tokio::task::spawn_local is the escape hatch for genuinely !Send tasks
- RefCell in keyvault.rs: !Sync violation in production security code
- 100+ bare tokio::spawn without Send verification
- No structured concurrency — fire-and-forget tasks without JoinSet/CancellationToken
- CPU work inside tokio::spawn without spawn_blocking
- Library crate calls tokio::spawn directly (runtime coupling)

### Pin/Unpin Patterns (10)
- Pin is a type-level contract, not a runtime barrier
- Drop guarantee most violated invariant — need delegate to inner function for !Unpin types
- Structural pinning has 4 rules: Unpin only if all pinned fields are, drop must not move pinned fields
- Pin<Box<T>> safe and ergonomic for !Unpin types
- pin! macro (Rust 1.68+) for stack-local pinning
- pin-project is community standard for safe projection
- PeekedStream manually implements AsyncRead/AsyncWrite without pin-project (fragile)
- ProxyStream trait requires Unpin blanket (prevents !Unpin stream extensibility)
- No pin-project dependency in crate
- Entire codebase implicitly Unpin (Pin provides zero protection)

### Ownership/Arc/Rc/Weak Patterns (10)
- Share trait (2026 RFC): distinguishes clone() (deep copy) from share() (alias creation)
- Reflexive Arc<Mutex<T>> is #1 antipattern — most code can restructure around single-owner ownership transfer
- Arc::new_cyclic (stable since 1.60): solves self-referential init without unsafe
- Lock ordering not enforced by Rust's type system — need Loom/Prusti for deadlock prevention
- EventBus Clone silently drops cloned state (handles and sync_handlers become empty)
- EvidenceApiState has 6 independent Mutex locks (deadlock risk)
- NeoTrix FFI facade holds 10 Arcs with no Weak references (cycle risk)
- No Weak usage in graph-like structures (ConsciousnessTree, VSA HyperCube)
- Missing Arc::new_cyclic for self-referential initialization
- .lock().unwrap() panic amplification

### Drop/RAII Patterns (10)
- !Forget trait (2026): types opt out of mem::forget, enabling safe scoped spawn
- AsyncDrop trait: nightly-only, requires ManuallyDrop::take + tokio::spawn workaround
- Guard convention: *Guard suffix signals RAII behavior
- AccountLease Drop ignores RwLock poisoning (in_flight counter permanently leaks)
- WriteAheadLog Drop holds Mutex across flush_state I/O (deadlock risk)
- EventBus Drop calls shutdown() which joins threads (potential hang/detach)
- ReentryGuard uses Ordering::Relaxed (insufficient for cross-thread correctness)
- No scopeguard usage despite dependency (ad-hoc guard duplication)
- ProxyKernel Drop sends shutdown signal but doesn't wait (fire-and-forget)
- No AsyncDrop guard pattern for NT-MEMORY async resources

---

## Defects Identified (40+)

### Send/Sync (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-SEND-1 | RefCell in keyvault.rs (!Sync in production security code) | Critical |
| D-SEND-2 | 100+ bare tokio::spawn without Send verification | High |
| D-SEND-3 | No structured concurrency (fire-and-forget tasks) | High |
| D-SEND-4 | CPU work inside tokio::spawn without spawn_blocking | High |
| D-SEND-5 | Async trait definitions without Send bounds on return futures | Medium |
| D-SEND-6 | Library crate calls tokio::spawn directly (runtime coupling) | Medium |
| D-SEND-7 | EventBus Clone drops handler registrations | Medium |
| D-SEND-8 | No runtime policy for !Send type usage | Low |

### Pin/Unpin (6)
| ID | Defect | Severity |
|----|--------|----------|
| D-PIN-1 | PeekedStream manual pin projection without pin-project (fragile) | Medium |
| D-PIN-2 | self_referential.rs misnamed (contains zero Pin usage) | Low |
| D-PIN-3 | ProxyStream requires Unpin blanket (prevents !Unpin extensibility) | Medium |
| D-PIN-4 | No pin-project dependency in crate | Medium |
| D-PIN-5 | Entire codebase implicitly Unpin (Pin provides zero protection) | Info |
| D-PIN-6 | Box::pin closures cannot be unpinned for inspection/debugging | Low |

### Ownership/Arc/Rc/Weak (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-ARC-1 | EventBus Clone silently drops cloned state | Critical |
| D-ARC-2 | EvidenceApiState 6 independent Mutex locks (deadlock risk) | High |
| D-ARC-3 | FFI facade 10 Arcs with no Weak references (cycle risk) | High |
| D-ARC-4 | No Weak usage in graph-like structures (ConsciousnessTree, HyperCube) | High |
| D-ARC-5 | Missing Arc::new_cyclic for self-referential initialization | Medium |
| D-ARC-6 | .lock().unwrap() panic amplification | Medium |
| D-ARC-7 | Missing RwLock for read-heavy subsystems | Low |
| D-ARC-8 | No Share trait semantics (clone vs alias distinction) | Low |

### Drop/RAII (8)
| ID | Defect | Severity |
|----|--------|----------|
| D-DROP-1 | AccountLease Drop ignores RwLock poisoning (in_flight leaks) | Critical |
| D-DROP-2 | WriteAheadLog Drop holds Mutex across flush_state I/O (deadlock) | High |
| D-DROP-3 | EventBus Drop calls shutdown() (potential hang/detach) | Medium |
| D-DROP-4 | ReentryGuard uses Ordering::Relaxed (cross-thread incorrect) | High |
| D-DROP-5 | No scopeguard usage despite dependency (ad-hoc duplication) | Low |
| D-DROP-6 | ProxyKernel Drop sends shutdown but doesn't wait | Medium |
| D-DROP-7 | No AsyncDrop guard pattern for async resources | Medium |
| D-DROP-8 | Missing Acquire/Release ordering convention | Medium |

## Key Insights (This Batch)

1. **EventBus Clone silently drops state**: handles and sync_handlers become empty on clone. Any subscriber spawned on original is orphaned in clone. This is a correctness bug.

2. **AccountLease Drop ignores RwLock poisoning**: If any thread panics while holding lock, all subsequent Drop calls silently fail, permanently leaking in_flight slots. Pool capacity permanently shrinks.

3. **WriteAheadLog Drop holds Mutex across I/O**: If flush_state blocks (disk full), Mutex stays locked. Concurrent shutdown = deadlock.

4. **ReentryGuard uses Relaxed ordering**: Provides no inter-thread synchronization. Works on x86 (strong ordering) but technically UB-adjacent on ARM.

5. **EvidenceApiState 6 independent Mutex locks**: Any handler needing 2+ fields must acquire multiple locks in arbitrary order → deadlock risk.

6. **Entire codebase implicitly Unpin**: Pin provides zero protection. No PhantomPinned markers anywhere. If any module needs address-sensitive guarantees, must add explicit `impl !Unpin`.

7. **Reflexive Arc<Mutex<T>> is #1 antipattern**: Most code can restructure around single-owner ownership transfer. Arc/Mutex correct only for genuinely concurrent shared state.

8. **!Forget trait (2026)**: Types can opt out of mem::forget, guaranteeing Drop::drop runs. Enables safe scoped spawn. NeoTrix should prepare for this.

9. **Library crate calls tokio::spawn directly**: neotrix-core is a library that should not assume specific async runtime. If downstream uses smol/async-std, these calls panic at runtime.

10. **No Weak usage in graph structures**: ConsciousnessTree has 11 branches referencing each other. VSA HyperCube is a graph. Neither uses Weak for back-references → memory leak risk.

## Cumulative Totals

| Metric | Value |
|--------|-------|
| Batches completed | 833 |
| New defects (this batch) | 30 |
| Cumulative defects | D01-D76947 |
| Research sources (this batch) | 40 |
| Cumulative research sources | 97,839+ |
