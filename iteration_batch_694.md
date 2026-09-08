# Iteration Batch 694 — Type System Research

**Date**: 2026-09-06  
**Prior**: Batch 693 (no simd-json, no KB schema versioning, no schema registry, no Avro, no fingerprinting)  
**Focus**: Type inference, gradual typing, dependent/refinement types — NEW defects and improvements for NeoTrix

---

## 1. Type Inference Findings

### 1.1 Rust Type Inference Gaps (2026)

**Source**: rustc-dev-guide, rust-analyzer 2026-08-10 release, Rust 2026 goals  
**URLs**:  
- https://rustc-dev-guide.rust-lang.org/type-inference.html  
- https://repojournal.com/showcase/rust-lang/2026-08-10/rust-analyzer-lands-parser-fixes-and-pattern-inference-overhaul  
- https://goals.rust-lang.org/2026/roadmaps.html  

**Key finding**: Rust's HM-based inference has a **coercion bug** where expected types weren't applied consistently between let statements and let expressions. Fixed in rust-analyzer 2026-08-10 by unifying code paths. The Rust 2026 project goal "Project Zero" explicitly targets **all known type system unsoundnesses**.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-1**: NeoTrix's KB query layer uses `serde_json::Value` as a dynamic "type" for node properties. This is the **gradual typing anti-pattern** — serde_json::Value is effectively `any` with zero compile-time checking. Batch 693 found no schema versioning; this defect explains *why*: the type system allows arbitrary shapes at compile time, so there's no compile-time pressure to version schemas. **Fix**: Replace `serde_json::Value` property bags with typed property enums per node kind, enforced at compile time.

### 1.2 Incremental Type Checking via Query Systems

**Source**: Salsa incremental computation framework (ruff), Rust query-based compiler  
**URLs**:  
- https://deepwiki.com/astral-sh/ruff/2.5-salsa-incremental-computation  
- https://lwn.net/Articles/997784  

**Key finding**: Ruff's `ty` type checker uses Salsa for incremental computation — queries track dependencies and only re-evaluate changed inputs. Rust's own compiler does this via `type_of()` queries with dependency graphs. Hashing intermediate results is "the main reason incremental compilation can be slower than non-incremental."

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-2**: NeoTrix's SEAL pipeline re-runs full module compilation checks from scratch each cycle. There is **no incremental type/type-check cache** — every `cargo check` starts from zero. This means the self-evolution loop pays full compile cost even for trivial changes. **Fix**: Adopt a Salsa-style query memoization layer for module health checks, keyed by file content hashes + dependency graph position.

### 1.3 FERRIUM Type Inference Invalidation

**Source**: FERRIS project research log  
**URL**: https://github.com/giodl73-repo/FERRIS/blob/main/docs/research/2026-08-08-type-inference-checking.md  

**Key finding**: FERRIUM adds "body-owner, inference, expected-type, coercion, pattern, writeback, trait-obligation, and type-result invalidation evidence" — tracking *what changed* to invalidate type inference results, with per-owner granularity. Prioritizes "read-only per-owner diagnostics and orthogonal fixtures over annotation rewrites."

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-3**: NeoTrix has no **type inference invalidation tracking** for its KB schema changes. When a node kind's property shape changes, downstream consumers (embedding pipeline, search indexer, cross-domain queries) are not notified. This is the schema evolution equivalent of missing cache invalidation. **Fix**: Add a `SchemaVersion` monotonically increasing counter per node kind; consumers check version before reading, abort with clear error on mismatch.

---

## 2. Gradual Typing Findings

### 2.1 Discriminative Typing Performance Optimization

**Source**: ACM TOPLAS 2024 (widely cited in 2026), Grift/Reticulated Python  
**URL**: https://dl.acm.org/doi/10.1145/3632931  

**Key finding**: Discriminative Typing (DT) compiles fast and slow versions of functions based on type annotation inference. Achieves 93% performance improvement across benchmarks, with 30% getting 4-25× speedups. Works by storing extra function versions and using subtyping to decide when fast version is safe.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-4**: NeoTrix's module system has no concept of "typed fast path" vs "untyped fallback path" for hot modules. NT-MIND's distillation and NT-CORE's E8 reasoning are performance-critical but use generic `serde_json::Value` intermediaries everywhere. **Fix**: Define typed fast-path structs for hot module boundaries (e.g., `DistillationInput`, `E8State`) that bypass serde_json serialization, keeping the dynamic path only for truly untyped external data.

### 2.2 Blame Tracking Fragility in AI Self-Modification

**Source**: GitHub issue #434 (hybrid-systems/aura), issue #309  
**URLs**:  
- https://github.com/cybrid-systems/aura/issues/434  
- https://github.com/cybrid-systems/aura/issues/309  

**Key finding**: "Blame tracking can break on deep structural changes (e.g. replace-subtree affecting predicate sites) if context not re-propagated. In AI multi-round self-mod (query → mutate → eval), narrowing reliability for if/match/ADT patterns is critical but not fully hardened for incremental case." Also: "IR-level coercion diagnostics are runtime-only; compile-time blame could be richer."

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-5**: NeoTrix's ConsciousnessTree performs multi-round self-modification (growth cycle modifies modules, then re-evaluates). There is **no blame tracking** for which module modification caused a health regression. When `cargo check` fails after SEAL distillation, there's no attribution to the specific change. **Fix**: Add a `ChangeBlame` struct that records: `(module_path, change_kind, timestamp, health_before, health_after)`. On regression, trace back to the specific modification.

### 2.3 Gradual Guarantee and Space-Efficient Blame

**Source**: MIT 590 W26 lecture slides, Grokipedia gradual typing article  
**URLs**:  
- https://web.eecs.umich.edu/~comar/courses/590/w26/slides/gradual-typing-slides.pdf  
- https://grokipedia.com/page/Gradual_typing  

**Key finding**: Space-efficient blame tracking for gradual types identifies two problems with casts: (1) metadata storage overhead for tracking blame labels, (2) propagation cost during execution. Typed Racket reports up to 100× overhead in partially typed configurations. The gradual guarantee states adding type annotations should never make a well-typed program ill-typed.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-6**: NeoTrix's KB has a mixed-typing problem: some modules use typed structs (`Node`, `Edge`), others use raw `serde_json::Value` for the same logical data. This violates the gradual guarantee — adding type annotations to a module that interacts with untyped modules doesn't improve safety, it just adds cast overhead. **Fix**: Enforce a "type boundary" policy: every module boundary must declare typed interfaces; `serde_json::Value` usage is restricted to I/O serialization layer only.

---

## 3. Dependent Types & Refinement Types Findings

### 3.1 Kuiper: Dependent Types for GPU (PLDI 2026)

**Source**: PLDI 2026  
**URL**: https://pldi26.sigplan.org/details/pldi-2026-papers/37/Kuiper-Correct-and-Efficient-GPU-Programming-with-Dependent-Types-and-Separation-Log  

**Key finding**: Kuiper uses F*'s dependent types + Pulse separation logic for verified GPU programming. Proves correctness of matrix multiplication with tensor cores. Extends separation logic with "located resources" for massively parallel programs. Compiles to efficient specialized CUDA with zero runtime overhead.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-7**: NeoTrix's NT-PHYSICAL module handles GPU/sensor interactions but has **no formal resource safety proofs**. Sensor acquisition → processing → release has no separation logic guarantees. A resource leak in sensor handling could cascade to the physical embodiment layer. **Fix**: Define a `ResourceToken` type using Rust's ownership system that enforces acquire-process-release at the type level (inspired by Kuiper's located resources), even without full dependent types.

### 3.2 PLEX: Normalization for Refinement Types (OOPSLA 2026)

**Source**: OOPSLA 2026  
**URL**: https://nikivazou.github.io/publications.html  

**Key finding**: PLEX solves normalization for refinement types — a key bottleneck where checking whether two refinement predicates are equivalent requires evaluating them. This enables more efficient refinement type checking by reducing redundant SMT solver calls.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-8**: NeoTrix's KB query system has no **predicate normalization**. When checking if two node property constraints are equivalent (e.g., `{x > 5}` vs `{x >= 6}` for integers), the system either does full SMT solving (expensive) or uses string equality (unsound). **Fix**: Implement a lightweight predicate normalizer for common KB query patterns, caching normalized forms to avoid redundant solver invocations.

### 3.3 Practical Range Refinement Types with Inference (Jul 2026)

**Source**: arXiv 2607.00824, Jul 2026  
**URL**: https://arxiv.org/abs/2607.00824  

**Key finding**: Aebi & Furia present practical range refinement types that work with automatic inference. Key insight: most real-world type refinements are range constraints (0 ≤ x < n, x > 0, etc.). Their system infers these automatically without annotations, with performance competitive with unrefined code.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-9**: NeoTrix's KB node IDs and indices use raw `u64` / `usize` with no range constraints. There's no compile-time guarantee that a node ID is within the valid range of the KB, or that an index doesn't overflow. This is the "unguarded array indexing" class of bug. **Fix**: Define `NodeId(u64)` newtype with range refinement `NodeId where 0 <= self.0 < KB_SIZE`, enforced at construction sites.

### 3.4 VerusBelt: Semantic Foundation for Rust Verification (PLDI 2026)

**Source**: PLDI 2026  
**URL**: https://dl.acm.org/doi/10.1145/3808325  

**Key finding**: VerusBelt provides a semantic foundation for Verus's proof-oriented extensions to Rust's type system. This bridges the gap between Rust's ownership model and formal verification, enabling "proof-oriented extensions" that are sound with respect to the language semantics.

**NEW DEFECT for NeoTrix**:  
- **DEFECT-694-10**: NeoTrix has **no semantic foundation** for its module interaction contracts. The "Dark Forest" rule (every module must compile + test + connect or be deleted) is enforced manually, not via a formal contract system. Module interfaces are ad-hoc `pub fn` signatures without pre/post-condition specifications. **Fix**: Define a `ModuleContract` trait with `pre_condition()`, `post_condition()`, and `invariant()` methods, checked at module boundaries. Start with NT-CORE and NT-MIND as pilot domains.

---

## 4. Summary: 10 New Defects Found

| ID | Domain | Severity | Description |
|----|--------|----------|-------------|
| DEFECT-694-1 | NT-MEMORY | HIGH | `serde_json::Value` as dynamic type eliminates compile-time schema enforcement |
| DEFECT-694-2 | NT-MIND | MEDIUM | No incremental type-check cache; full re-compilation every SEAL cycle |
| DEFECT-694-3 | NT-MEMORY | HIGH | No schema version invalidation tracking for downstream consumers |
| DEFECT-694-4 | NT-CORE/NT-MIND | MEDIUM | No typed fast-path for hot module boundaries; generic intermediaries everywhere |
| DEFECT-694-5 | NT-CORE | HIGH | No blame tracking for which self-modification caused health regression |
| DEFECT-694-6 | ALL | MEDIUM | Mixed typed/untyped boundaries violate gradual guarantee; cast overhead |
| DEFECT-694-7 | NT-PHYSICAL | HIGH | No formal resource safety for sensor/GPU interactions |
| DEFECT-694-8 | NT-MEMORY | MEDIUM | No predicate normalization for KB query equivalence checking |
| DEFECT-694-9 | NT-MEMORY | MEDIUM | No range refinement on node IDs/indices; unguarded access |
| DEFECT-694-10 | ALL | LOW | No semantic foundation for module interaction contracts |

## 5. Sources Cited

1. Arnab Ghosh, "Type Inference Without Annotations: How Algorithm W Works" (Feb 2026) — https://www.arnabg.me/blog/hindley-milner
2. UT Dallas CS 6371, "Hindley-Milner Type Inference" (Apr 2026) — https://personal.utdallas.edu/~hamlen/cs6371sp26/typeinference.pdf
3. Rust Compiler Dev Guide, "Type Inference" — https://rustc-dev-guide.rust-lang.org/type-inference.html
4. Rust-analyzer 2026-08-10 release — https://repojournal.com/showcase/rust-lang/2026-08-10/
5. Rust 2026 Goals, "Project Zero" — https://goals.rust-lang.org/2026/roadmaps.html
6. Salsa Incremental Computation (ruff/ty) — https://deepwiki.com/astral-sh/ruff/2.5-salsa-incremental-computation
7. LWN, "Rust's incremental compiler architecture" — https://lwn.net/Articles/997784
8. FERRIS/FERRIUM type inference research — https://github.com/giodl73-repo/FERRIS/blob/main/docs/research/2026-08-08-type-inference-checking.md
9. ACM TOPLAS, "Type-Based Gradual Typing Performance Optimization" — https://dl.acm.org/doi/10.1145/3632931
10. hybrid-systems/aura#434, Blame tracking issues — https://github.com/cybrid-systems/aura/issues/434
11. hybrid-systems/aura#309, EDA blame tracking — https://github.com/cybrid-systems/aura/issues/309
12. MIT 590 W26, Gradual Typing lecture — https://web.eecs.umich.edu/~comar/courses/590/w26/slides/gradual-typing-slides.pdf
13. Grokipedia, "Gradual Typing" — https://grokipedia.com/page/Gradual_typing
14. PLDI 2026, "Kuiper: Correct and Efficient GPU Programming with Dependent Types" — https://pldi26.sigplan.org/details/pldi-2026-papers/37/
15. OOPSLA 2026, "PLEX: Normalization for Refinement Types" — https://nikivazou.github.io/publications.html
16. arXiv 2607.00824, "Practical Range Refinement Types with Inference" — https://arxiv.org/abs/2607.00824
17. PLDI 2026, "VerusBelt: A Semantic Foundation for Verus" — https://dl.acm.org/doi/10.1145/3808325
18. Flux: Liquid Types for Rust — https://github.com/flux-rs/flux
19. ByteLedger, "Static vs Dynamic Typing 2026" — https://byteledger.vizleo.com/blog/static-vs-dynamic-typing-2026
20. DRCodes, "Gradual Typing Guide" — https://drcodes.com/posts/gradual-typing-guide-add-type-safety-to-dynamic-languages

## 6. Connection to Batch 693

Batch 693 identified: no simd-json, no KB schema versioning, no schema registry, no Avro, no fingerprinting.

Batch 694 reveals the **root cause** of several 693 findings:
- The lack of schema versioning (693) exists because `serde_json::Value` eliminates compile-time pressure to define schemas (694-1)
- The lack of schema registry (693) is symptomatic of missing schema invalidation tracking (694-3)
- The lack of fingerprinting (693) would be partially solved by PLEX-style predicate normalization (694-8)

**Evolution trajectory**: The type system research suggests NeoTrix should invest in **typed boundaries** (not full dependent types) as the highest-ROI improvement. The gradual typing literature shows that even partial type annotation at module boundaries yields disproportionate safety and performance gains.
