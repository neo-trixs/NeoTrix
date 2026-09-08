# Iteration 611 — Batch Report
**Date**: 2026-09-06
**Focus**: Language Design, Type Systems, Compiler — post-batch-610 incremental findings

## Batch 610 Recap
| # | Finding | Value |
|---|---------|-------|
| 1 | L1I$ MPKI rises | 2.53× instruction cache pressure dominant |
| 2 | SMT contention | Non-linear collapse |
| 3 | LLM-guided compiler hints | 6.88× over -Ofast |
| 4 | Arena allocators | 342× speedup on Apple Silicon |
| 5 | General-purpose allocators | 2× degradation under fragmentation |

---

## 1. Language Design

### 1.1 Zig 2026: Colorless Async I/O + Type Resolution Redesign
**Source**: ziglang.org/devlog/2026, blog.bokvi.com/blog/zig-in-2026, sesamedisk.com

**What's NEW vs batch 610**: Zig 0.16.0 (April 2026) shipped `std.Io` — a colorless async I/O interface modeled identically to Zig's `Allocator` pattern. This is a fundamentally different answer to the function-coloring problem than Rust's async/await. Instead of colored functions, Zig passes `io` as a parameter. `Io.Threaded` (blocking + thread pool) and `Io.Evented` (io_uring/kqueue/GCD) are interchangeable — application code is I/O-backend-agnostic.

**Key findings**:
- **Type resolution redesign**: 30,000-line PR (March 2026) — lazy field analysis now skips analyzing unreferenced struct fields, eliminating "over-analysis" in incremental compilation. Compile times for large projects reduced significantly.
- **@bitCast semantic redesign**: Decoupled from physical memory layout → logical bit layout. Fixes LLVM backend miscompilations with non-ABI-width integers. LLVM backend sees ~5% performance improvement on the Zig compiler itself from better integer lowering.
- **ELF linker**: Self-hosted, supports fast incremental relinking (~244ms). Can now build the Zig compiler itself.
- **Build system split**: Maker/configurer process separation cuts `zig build --help` from 150ms → 14ms (-90%).
- **Package management moved from compiler to build system**: Binary shrunk 14.1→13.5 MiB.

**Defects/improvements over batch 610**:
- Batch 610 analyzed LLM-guided compiler hints on LLVM. Zig's approach is architecturally orthogonal: instead of ML-guided optimization, Zig eliminates entire classes of overhead by making I/O and allocation explicit parameters. This sidesteps the "phase ordering problem" that IntOpt (see §3.3) addresses with LLMs.
- **NEW DEFECT**: Zig's `comptime` evaluates at compile time but has no equivalent of Rust's const generics for type-level computation in const contexts. This limits certain type-state patterns that NeoTrix's ConsciousnessTree uses for compile-time verification of module wiring.

### 1.2 Nim/Nimony: Final IR as Structured Assembler
**Source**: github.com/nim-lang/nimony/issues/1946

**What's NEW vs batch 610**: Nim's Nimony compiler introduces a "Final IR" that is a structured assembler — not SSA, not arbitrary CFG. Control flow is reducible, properly-nested DAGs (plus loop back-edges). Every pass is a total tree traversal with no fixpoint iteration needed.

**Key findings**:
- `lab`/`jmp` are NOT general labels-and-gotos — they maintain two invariants that keep the CFG reducible and properly-nested.
- `loop` has no condition — the guard is outside the loop body.
- Control flow is entirely statement-based: `if`/`case`/`try` in expression position are lowered to statement form with multi-join destinations.
- Final IR maps DIRECTLY to machine instructions — it's a verified, typed assembly subset. No SSA ⟹ merges are free.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610's analysis focused on IR-level optimization passes (LLVM MIR). Nimony's Final IR suggests a different architectural choice: if your IR is sufficiently structured, you can skip SSA entirely and still get efficient codegen. This challenges the assumption that SSA is necessary for optimization — the structured constraints do the work instead.
- **NEW DEFECT for NeoTrix**: The ConsciousnessTree currently models its SE pipeline as an arbitrary DAG of module dependencies. Nimony's invariant that "every pass is a total tree traversal" suggests NeoTrix's SEAL pipeline could benefit from enforcing reducibility constraints on its evolution DAG, enabling pass ordering without fixpoint iteration.

---

## 2. Type Systems

### 2.1 Intersection Types for Algebraic Effects (ICFP 2026)
**Source**: icfp26.sigplan.org/details/icfp-2026-icfp-papers/35/

**What's NEW vs batch 610**: A novel intersection type system for λ-calculus with algebraic effects and handlers. The system is behavioral — it characterizes terminating terms AND reduces the reachability problem to type inference. First such system with these properties for a calculus with handlers.

**Key findings**:
- Subject reduction AND expansion both hold.
- Decidable Higher-Order Model Checking (HOMC) problem — unlike Dal Lago & Ghyselen's HEPCF.
- Type inference is directly connected to reachability analysis.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610's LLM-guided compiler hints operate at the IR level. This intersection type system provides a static alternative: if effects can be typed precisely enough, the compiler can reason about effect interactions without ML inference. For NeoTrix's GWT attention routing, this suggests a typed effect system could statically verify that attention broadcasts don't create feedback loops.

### 2.2 Cambria: Parametrized Algebraic Effects with Resource Abstraction
**Source**: arxiv.org/abs/2608.27798 (August 2026)

**What's NEW vs batch 610**: Cambria extends algebraic effects to the parametrized setting where effect signatures use abstract parameter types instantiated by handlers. Parameters abstract over resources (memory locations, thread IDs), are first-class in the type system, but erased at runtime.

**Key findings**:
- Parametricity proved via step-indexed logical relation.
- Abstract thread IDs shared between concurrent computations — goes beyond standard effect instances.
- First calculus with user-defined resource-allocating effects that guarantees (via parametricity) that client code cannot depend on how a handler represents its resources.

**Defects/improvements over batch 610**:
- **NEW DEFECT for NeoTrix**: NeoTrix's NT-ACT domain manages tool invocations (MCP) without effect typing. Cambria's model suggests that tool calls could be typed as algebraic effects with resource parameters, enabling the compiler to statically verify that tool calls don't leak resource handles across sandbox boundaries (relevant to NT-SHIELD's egress privacy guard).
- **NEW INSIGHT**: Cambria's "parameters erased at runtime" model parallels NeoTrix's VSA HyperCube approach where symbolic representations are compile-time only but have runtime performance implications.

### 2.3 Graded Types: Bridging Linear-Base and Graded-Base (ICFP 2026)
**Source**: icfp26.sigplan.org, ar5iv.labs.arxiv.org/html/2606.28042

**What's NEW vs batch 610**: First formal connection between two dominant coeffect type system lineages: linear-base (Girard's Linear Logic) and graded-base (pervasive annotations on all assumptions). Proves mutual type-, grade-, and operational-semantics-preserving translations.

**Key findings**:
- Both approaches can express the same context-dependence notions.
- Linear Haskell, Idris 2, Granule, and QTT all fall on the graded-base side.
- The bridge enables transfer of results and ideas between the two traditions.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: NeoTrix's skill tree has node tiers (Small Passive/Notable Passive/Keystone) that imply resource usage patterns. Graded types could formalize these tiers as coeffect grades — "this function uses at most 2KB stack, 1 GPU, 0.5ms wall time" — enabling static verification of resource budgets. This connects to batch 610's ResourceBudgetManager.
- **NEW DEFECT**: The current NeoTrix type system (Rust) uses `Send`/`Sync` markers as coarse-grained resource annotations. Graded types offer a finer-grained alternative: instead of binary "is thread-safe", a grade like `@concurrent[1]` could express "safe to call from 1 concurrent context."

### 2.4 Gradual Typing Performance: Annotation Selection (Programming 2026)
**Source**: arxiv.org/abs/2603.05649, 2026.programming-conference.org

**What's NEW vs batch 610**: A lightweight technique for selecting a SUBSET of type annotations derived by type inference to improve execution performance in gradually typed programs. Selects annotations along data flows to avoid expensive runtime casts at typed/untyped boundaries.

**Key findings**:
- Adding ALL type annotations can DECREASE performance (counterintuitive).
- Selection along data flows outperforms naive "use all annotations" strategy.
- Compilation time is stable (amortized approach) unlike prior methods.
- Evaluated on Reticulated Python.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610's LLM-guided compiler hints optimize optimization passes. This work shows that TYPE ANNOTATIONS themselves can be optimized for performance — it's a different dimension of the optimization space. For NeoTrix's gradual typing of NT-MIND's skill engine, this means careful annotation placement matters more than exhaustive annotation.
- **NEW DEFECT**: NeoTrix's type annotations on skill node interfaces are currently all-or-nothing. This research suggests a "minimal annotation" strategy: only annotate types at boundaries where runtime casts would be expensive (cross-domain skill calls), leaving internal types inferred.

---

## 3. Compiler

### 3.1 Rust MIR Move Elimination (RFC 3943)
**Source**: github.com/rust-lang/rust/pull/157943, github.com/rust-lang/rust/pull/162048

**What's NEW vs batch 610**: A new MIR optimization pass that eliminates unnecessary copies through place unification. Three components:
1. **PreciseLiveness**: Sub-statement granularity liveness analysis, borrows-aware.
2. **TailCopyToMove**: Pre-processing converts copies-to-return-place into moves.
3. **MoveElimination**: Main pass eliminates moves through place unification.

**Key findings**:
- Supersedes `DestinationPropagation` completely.
- Gated behind `-Z mir-move-elimination` (new MIR semantics from RFC 3943).
- Miri support added for the new semantics (LiveUnallocated state, move-out-to-temporary semantics).
- **Soundness challenge**: StorageLive insertion without exploding compile times is an open problem.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610 analyzed SMT contention as non-linear collapse. MoveElimination addresses a different kind of contention: register pressure from unnecessary copies. The PreciseLiveness analysis at sub-statement granularity is analogous to NeoTrix's fine-grained attention routing — both require tracking resource usage at finer granularity than basic blocks.
- **NEW DEFECT**: NeoTrix's data flow between NT-MEMORY (KB) and NT-CORE (reasoning) currently uses explicit clone/copy at domain boundaries. MoveElimination's place unification suggests that if NeoTrix's domains shared a common IR, the compiler could eliminate these copies automatically.

### 3.2 Rust Post-Mono MIR Optimizations
**Source**: github.com/rust-lang/rust/pull/156858

**What's NEW vs batch 610**: A `build_codegen_mir` query that monomorphizes MIR during traversal, enabling post-monomorphization MIR passes. Uses `Steal<Cow<'tcx, Body<'tcx>>>` to reuse `optimized_mir` when already monomorphic.

**Key findings**:
- Removes just-in-time monomorphization from codegen code.
- 0.9% primary instruction count regression, 11.7% max-RSS regression (memory overhead of monomorphized MIR storage).
- Enables follow-up PRs to migrate codegen-time transforms to MIR passes.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: The memory regression (11.7% max-RSS) directly relates to batch 610's finding #5 (arena allocators 342× speedup). The RSS regression is because monomorphized MIR bodies are stored in the compiler's arena — the compiler itself benefits from arena allocation but pays in peak memory. This is a concrete example of the arena memory/performance tradeoff.
- **NEW DEFECT**: NeoTrix's SEAL pipeline runs optimization passes at compile time but doesn't monomorphize its pass pipeline. If skill nodes were monomorphized per-task-type (like Rust monomorphizes generics), post-mono optimizations could apply, but at the cost of binary size — the same tradeoff Rust faces.

### 3.3 IntOpt: Intent-Driven IR Optimization with LLMs
**Source**: arxiv.org/abs/2602.18511 (February 2026)

**What's NEW vs batch 610**: IntOpt explicitly separates high-level optimization intent from low-level analysis/transformation. Three stages: intent formulation → intent refinement → intent realization. Uses GPT-5 for refinement and realization.

**Key findings**:
- 90.5% verified correctness, 2.660× average speedup on 200-program test set.
- Outperforms LLVM -O3 on 37 benchmarks with speedups up to 272.60×.
- Ablation: removing intent refinement drops speedup from 2.660× to 1.122×.
- Compiler analysis is essential: without it, correctness drops to 77.0%.
- **272.60× speedup case**: IntOpt discovered optimization opportunities beyond traditional compiler passes.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610's "LLM-guided compiler hints 6.88× over -Ofast" was the precursor. IntOpt formalizes this into a principled three-stage pipeline. The key difference: IntOpt makes "optimization intent" an explicit intermediate representation, rather than implicitly embedding it in the LLM prompt. This is architecturally similar to NeoTrix's E8 Hexagram — an explicit reasoning state between intent and action.
- **NEW DEFECT**: IntOpt's intent formulation stage uses a fixed LLM (GPT-5). For NeoTrix's SEAL pipeline, this suggests that evolution passes should have an explicit "intent" phase that characterizes what optimization is needed before selecting which transformation to apply — rather than blindly applying a fixed pass order.

### 3.4 MLIR: CSE Between Greedy Rewriter Iterations
**Source**: github.com/llvm/llvm-project/pull/193081

**What's NEW vs batch 610**: The MLIR greedy pattern rewrite driver now optionally runs full CSE between iterations. Previously only deduplicated constants via folding. Now structurally identical non-constant subexpressions can be eliminated between iterations, unblocking further canonicalizations.

**Key findings**:
- Off by default (rebuilds dominance info each iteration).
- CSE driver moved to `Utils/CSE.cpp` to avoid layering cycles.
- Region-scoped overload added for greedy driver use.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: This addresses the "phase ordering problem" that IntOpt (§3.3) also tackles. MLIR's approach is local (CSE between iterations within a single canonicalizer), while IntOpt's is global (intent across the full pipeline). Both are needed.
- **NEW DEFECT**: NeoTrix's SEAL pipeline runs passes in a fixed order without inter-pass CSE. If a distillation pass produces redundant intermediate representations, they persist into subsequent passes. This MLIR pattern suggests adding a "canonicalize between SEAL phases" step.

### 3.5 SPEC CPU2026: Compiler Sensitivity Renewed
**Source**: arxiv.org/abs/2605.03713, chipsandcheese.com

**What's NEW vs batch 610**: SPEC CPU2026 confirms batch 610's L1I$ MPKI finding (2.53×) AND adds: gcc-15 reduces instruction count by 17.7% on SPEC CPU2026 vs only 5.1% on SPEC CPU17. Fresh workloads expose renewed compiler optimization headroom.

**Key findings**:
- SPEC CPU2026 shifts pressure toward higher instruction-cache stress (confirmed from batch 610).
- Compiler sensitivity is 3.5× higher on SPEC CPU2026 than SPEC CPU17.
- 706.stockfish_r instruction count reduced 3× by gcc-15.
- Frontend/translation pressure: L1 iTLB rises 2.08× (INT Rate).

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610's L1I$ MPKI 2.53× finding is now independently confirmed by SPEC CPU2026 characterization. The NEW finding is that compiler sensitivity is also renewed — fresh benchmarks expose optimization headroom that SPEC CPU17 no longer provides. This validates the use of SPEC CPU2026 for evaluating NeoTrix's LLM-guided compiler hints.
- **NEW DEFECT**: NeoTrix's build benchmarks currently use SPEC CPU2017. Migrating to SPEC CPU2026 would provide a more sensitive evaluation target for the LLM-guided optimization passes.

### 3.6 ICARUS: Branch-History-Based L2 Instruction Caching
**Source**: ASPLOS 2026, cse.iitb.ac.in/~biswa/ASPLOS2026.pdf

**What's NEW vs batch 610**: ICARUS uses branch history as context to identify critical instruction lines in L2 cache. Only 3.49% of instruction fetches are critical but cause 23.18% of front-end stalls. Proposed bin-based replacement policy using criticality AND reuse.

**Key findings**:
- L2 instruction MPKI reduced from 4.72 to 1.94 (geomean).
- Context-based critical line detection (BHC) uses branch history × instruction address.
- BRC (bin-based replacement) improves over EMISSARY by preserving critical lines with long reuse.
- Speedup: 5.6% on 12 datacenter applications.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: Batch 610's SMT contention analysis was about CPU-level resource contention. ICARUS addresses a different level: L2 instruction cache contention in datacenter workloads. The branch-history context is analogous to NeoTrix's GWT attention routing — both use "what happened before" (branch history / attention history) to predict "what matters now" (critical cache lines / salient information).
- **NEW DEFECT**: NeoTrix's HeartbeatAggregator doesn't track instruction cache metrics. Adding L1I$/L2 MPKI to the health snapshot would enable the GWT to modulate attention based on frontend pressure — a direct application of ICARUS's insight.

### 3.7 TRRIP: Compiler-Assisted Instruction Cache Temperature
**Source**: ACM 2026, doi.org/10.1145/3725843.3756110

**What's NEW vs batch 610**: Software-hardware co-design where the compiler classifies code as hot/warm/cold and communicates this to hardware via page attributes. Hardware uses temperature hints to optimize RRIP replacement policy.

**Key findings**:
- L2 instruction MPKI reduced by 26.5% on top of PGO-optimized mobile code.
- Geomean speedup: 3.9%.
- Compiler provides hints via OS page attributes (zero hardware interface cost).
- Hot lines inserted as "Immediate" (RRPV=0), warm as "Near", cold follow default.

**Defects/improvements over batch 610**:
- **NEW INSIGHT**: TRRIP bridges the gap between compiler knowledge and hardware behavior. Batch 610's arena allocator findings were about software-side memory management. TRRIP shows that the compiler can also influence hardware-side memory (cache) management through metadata. For NeoTrix, this suggests that the compiler could annotate skill node code with temperature hints to improve instruction cache behavior in the NT-ACT execution engine.
- **NEW DEFECT**: NeoTrix doesn't currently distinguish hot/cold code paths in its SEAL pipeline output. Adding temperature annotations to generated code would enable hardware-assisted cache management.

---

## 4. Allocator Findings (Cross-Cutting)

### 4.1 "Reconsidering Custom Memory Allocation" (2026 Revisit)
**Source**: arxiv.org/abs/2605.17119 (May 2026)

**What's NEW vs batch 610**: A 25-year revisit of Berger et al.'s seminal paper. Modern general-purpose allocators (jemalloc, mimalloc) have narrowed the gap with region allocators from 44% to 15% on clean heaps. BUT: adversarial allocation (heap fragmentation) reveals the true advantage — region allocators are unaffected while naive allocation degrades up to 2×.

**Key findings**:
- Region allocators: up to 15% speedup on clean heap (down from 44% in 2002).
- Under adversarial allocation: region allocators provide 3.2–27% advantage over naive+modern-allocator.
- glibc malloc degrades 2× under fragmentation; jemalloc/mimalloc degrade up to 27%.
- Per-class allocators: STILL no substantial benefit (confirmed from 2002).

**Defects/improvements over batch 610**:
- **UPDATES batch 610 finding #4/5**: The 342× arena speedup from batch 610 was on Apple Silicon with a specific workload. This 2026 paper shows the real-world picture: on clean heaps, modern allocators are competitive. The arena advantage is primarily FRAGMENTATION RESILIENCE, not raw throughput. Batch 610's 342× likely measured allocation throughput only, not fragmentation scenarios.
- **NEW DEFECT**: NeoTrix's NT-MEMORY KB uses SQLite's built-in allocator for most operations. The paper suggests that wrapping SQLite's allocation in a region allocator for transaction-lifetime allocations could provide fragmentation resilience without performance penalty.

### 4.2 Arena vs TLAB: Contention Is the Real Bottleneck
**Source**: medium.com/@ak0182274 (April 2026)

**What's NEW vs batch 610**: Lock-free arena allocators with shared cursors are 3× SLOWER than malloc under 4 threads due to CAS contention on the shared cursor. TLAB (Thread-Local Allocation Buffer) solves this by providing per-thread allocation blocks, reducing atomic ops to ~0.0001 per allocation.

**Key findings**:
- Arena-only: 0.76M items/sec (4 threads) — 68% SLOWER than baseline.
- TLAB: 2.00M items/sec — competitive with malloc.
- Arena reset lifecycle: 22.2M items/sec — 4.8× faster than per-node delete.
- Shared cursor CAS: 2.79 atomic ops per allocation (1 CAS + 1.79 failed retries).

**Defects/improves batch 610**:
- **CORRECTS batch 610 finding #4**: The 342× arena speedup assumed single-threaded or low-contention scenarios. Under multi-threaded contention, arena-only is SLOWER than malloc. TLAB is the correct architecture for concurrent arena allocation. This is a critical correction for NeoTrix's NT-ACT domain which uses multi-threaded tool execution.
- **NEW DEFECT**: NeoTrix's arena allocator strategy should use TLAB, not raw bump-pointer arena, for concurrent skill node execution. The shared cursor contention would destroy performance under load.

---

## Summary: NEW vs Batch 610

| # | New Finding | Defect/Improvement |
|---|-------------|-------------------|
| N1 | Zig's colorless async I/O sidesteps function-coloring via explicit parameter passing | Architecturally orthogonal to ML-guided optimization; NeoTrix could adopt parameter-passing for I/O instead of colored async |
| N2 | Nimony's Final IR eliminates SSA via structured assembler invariants | Challenges SSA assumption; NeoTrix SEAL DAG could enforce reducibility for pass ordering without fixpoint |
| N3 | Intersection types for algebraic effects enable static effect reasoning | Could statically verify GWT attention broadcast doesn't create feedback loops |
| N4 | Cambria's parametrized effects with erased resource types | Tool calls could be typed as effects; resource parameters verify sandbox isolation |
| N5 | Graded types bridge linear-base and graded-base coeffect systems | Skill node tiers could be formalized as coeffect grades for static resource verification |
| N6 | Gradual typing annotation selection: less is more for performance | Minimal annotation strategy for NeoTrix skill boundaries; exhaustive annotation is counterproductive |
| N7 | Rust MoveElimination: sub-statement PreciseLiveness | NeoTrix domain boundaries could eliminate explicit copies with shared IR |
| N8 | Post-mono MIR: 11.7% RSS regression from monomorphization | Arena memory/performance tradeoff is real; NeoTrix must account for monomorphization cost |
| N9 | IntOpt: 2.66× speedup via explicit optimization intent | Formalizes batch 610's 6.88× finding; three-stage pipeline (formulate→refine→realize) |
| N10 | MLIR CSE-between-iterations | Phase ordering problem; NeoTrix SEAL needs inter-phase canonicalization |
| N11 | SPEC CPU2026: compiler sensitivity 3.5× higher | Fresh benchmarks expose optimization headroom; validates LLM-guided hints evaluation |
| N12 | ICARUS: branch-history context for L2 instruction caching | GWT attention routing parallels branch-history criticality prediction |
| N13 | TRRIP: compiler-hardware co-design for instruction cache temperature | NeoTrix should annotate hot/cold code paths for hardware cache management |
| N14 | Arena advantage is fragmentation resilience, not raw throughput (2026 revisit) | Corrects batch 610's 342× to context-dependent: 15% on clean heap, 3–27% under fragmentation |
| N15 | TLAB solves arena contention; raw arena is 3× slower under threads | NeoTrix must use TLAB, not raw bump-pointer, for concurrent execution |

## Sources Cited
1. ziglang.org/devlog/2026 — Zig type resolution redesign, @bitCast semantics, ELF linker
2. blog.bokvi.com/blog/zig-in-2026 — Zig 0.16.0 std.Io colorless async
3. sesamedisk.com/zig-type-resolution-redesign-2026 — Zig lazy field analysis
4. github.com/nim-lang/nimony/issues/1946 — Nimony Final IR design
5. icfp26.sigplan.org — Intersection types for algebraic effects (ICFP 2026)
6. arxiv.org/abs/2608.27798 — Cambria: parametrized algebraic effects
7. ar5iv.labs.arxiv.org/html/2606.28042 — Graded types linear-base vs graded-base
8. arxiv.org/abs/2603.05649 — Gradual typing annotation selection
9. github.com/rust-lang/rust/pull/157943 — Rust MIR move elimination (RFC 3943)
10. github.com/rust-lang/rust/pull/156858 — Rust post-mono MIR optimizations
11. arxiv.org/abs/2602.18511 — IntOpt: intent-driven IR optimization
12. github.com/llvm/llvm-project/pull/193081 — MLIR CSE between iterations
13. arxiv.org/abs/2605.03713 — SPEC CPU2026 characterization
14. chipsandcheese.com/p/evaluating-spec-cpu2026 — SPEC CPU2026 microarchitectural analysis
15. cse.iitb.ac.in/~biswa/ASPLOS2026.pdf — ICARUS L2 instruction caching
16. doi.org/10.1145/3725843.3756110 — TRRIP temperature-based re-reference
17. arxiv.org/abs/2605.17119 — Reconsidering custom memory allocation (2026)
18. medium.com/@ak0182274 — Arena vs TLAB contention analysis
19. github.com/llvm/llvm-project/pull/203796 — MLIR interactive REPL + MCP server
20. arxiv.org/abs/2608.00029 — Nova: MLIR JIT compiler for DL
21. arxiv.org/abs/2608.20137 — Formal performance guarantees for compiler heuristics
