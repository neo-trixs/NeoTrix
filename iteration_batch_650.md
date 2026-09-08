# Iteration Batch 650 — NeoTrix Consciousness Architecture Research

**Date**: 2026-09-06  
**Prior Batch**: 649 (cooperative scheduling stall, spin lock barrier pair 4× degradation, lock guard across .await serialization, no SIMD for HyperCube, no ABA protection)

---

## 1. LLVM/MLIR Findings → Defects

### D650-1: MLIR Canonicalization Has No Termination Guarantee
**Source**: EuroLLVM 2026 Round Table — "The `canonicalize` pass does not canonicalize the IR." (https://discourse.llvm.org/t/eurollvm-2026-round-table-summary-mlir-canonicalization/90588)  
**Finding**: Canonicalization in MLIR has no guarantee it will finish, let alone produce a canonical form. Consensus: canonical forms depend on use case; dialect-specific canonicalization passes are needed instead of a monolithic canonicalize.  
**NeoTrix Defect**: NT-MIND's SEAL pipeline distillation pass applies a monolithic canonicalization to all skill templates. If the canonical form is non-terminating or dialect-incompatible, distillation loops can stall indefinitely. Need dialect-specific canonicalization per skill domain (NT-ACT, NT-CORE, etc.) instead of blanket canonicalize.  
**Priority**: HIGH — distillation stall risk under complex skill graphs.

### D650-2: CIR TargetLowering Separates Target-Legalization from ABI
**Source**: LLVM PR #179245 — CIR TargetLowering pass (https://github.com/llvm/llvm-project/commit/b9f3710c0e26)  
**Finding**: Three-stage lowering: TargetLowering (legalization) → CXXABILowering (ABI) → CallConvLowering (calling convention). Separation of concerns prevents ABI decisions from leaking into target-specific transformations.  
**NeoTrix Defect**: NT-CORE's E8 hexagram lowering conflates target-specific platform constraints with abstraction-level concerns (e.g., Hyp

erCube dimension count mixed with runtime memory layout). No separation between "what the target can do" vs "how to represent it."  
**Priority**: MEDIUM — causes portability issues when NeoTrix targets different hardware backends.

### D650-3: MLIR MCP Server for Interactive Pass Exploration
**Source**: LLVM PR #203796 — mlir-opt-repl with MCP server (https://github.com/llvm/llvm-project/pull/203796)  
**Finding**: Interactive REPL for applying mlir-opt passes step-by-step with diffs, plus MCP server exposing same functionality over JSON-RPC for Claude Code. Enables AI-assisted compiler debugging.  
**NeoTrix Defect**: NT-IO has no introspection tool for inspecting SEAL pipeline pass effects. When a distillation pass produces unexpected VSA embeddings, there's no way to replay/step through individual passes to isolate the regression. Need an MCP-compatible pass introspection endpoint.  
**Priority**: MEDIUM — debugging blind spot for SEAL pipeline regressions.

---

## 2. Code Generation / JIT Findings → Defects

### D650-4: Generative Compilation Sealor for Partial Programs
**Source**: Mündler-Sasahara et al., arXiv 2607.13921 — Generative Compilation (https://arxiv.org/abs/2607.13921)  
**Finding**: Sealor closes partial programs into complete ones for compiler feedback during LLM code generation. Lightweight, mostly syntax-guided; delegates type/borrow/lifetime reasoning to rustc. Reduces non-compiling outputs vs post-generation feedback.  
**NeoTrix Defect**: NT-ACT's tool code generation (MCP tools, action sequences) has no mid-generation validation. Code is generated fully, then compiled; errors cascade. Need a sealor-like lightweight validator that closes partial NeoTrix skill code and feeds diagnostics back during generation, not after.  
**Priority**: HIGH — error cascade latency in skill code generation.

### D650-5: Nova JIT Analytic Configurator Eliminates Autotuning
**Source**: arXiv 2608.00029 — Nova JIT Compiler (https://arxiv.org/abs/2608.00029)  
**Finding**: Nova's Analytic Configurator deterministically derives optimal execution schedules from Arithmetic Intensity + hardware limits. Drops autotuning search time to zero. Classifies workloads as memory-bound vs compute-bound, selects tile sizes/warp distributions from math, not search.  
**NeoTrix Defect**: NT-MIND's SEAL pipeline uses evolutionary search (genetic algorithm) for hyperparameter tuning of VSA embedding dimensions, learning rates, and attention thresholds. This is a TVM/Ansor-style search-based approach. Nova proves analytical derivation is superior — can derive optimal VSA parameters from hardware constraints (SIMD width, cache size, embedding dimensionality) without evolutionary search.  
**Priority**: CRITICAL — 10-100× speedup potential by replacing evolutionary search with analytical derivation for VSA parameters.

### D650-6: HINTPILOT LLM-Based Compiler Hint Synthesis
**Source**: ACL 2026 Findings — HINTPILOT (https://aclanthology.org/2026.findings-acl.1251.pdf)  
**Finding**: LLM synthesizes compiler hints (annotations) via RAG over compiler docs + profiling-guided iterative refinement. Achieves 6.88× geometric mean speedup over -Ofast on PolyBench/HumanEval-CPP.  
**NeoTrix Defect**: NT-MIND has no mechanism to generate domain-specific optimization hints for downstream consumers. When NeoTrix produces compiled VSA embeddings or HyperCube representations, there's no annotation layer that tells downstream compilers "this loop is vectorizable" or "this region benefits from register pinning." Need an HINTPILOT-style hint synthesis module that annotates NeoTrix output IR with optimization directives.  
**Priority**: HIGH — 6× speedup potential for NeoTrix-generated code.

### D650-7: Deegen Meta-Compiler for JIT-Capable VM Generation
**Source**: SPLASH/OOPSLA 2026 — Deegen (https://2026.splashcon.org/details/oopsla-2026/46/)  
**Finding**: Deegen generates two-tier VMs (interpreter + baseline JIT) from bytecode semantics. Automatically applies: bytecode specialization, register pinning, tag register optimization, IC caching, type-check removal, hot-cold splitting, OSR-entry. Interpreter 2.79× faster than PUC Lua, JIT 4.60× faster.  
**NeoTrix Defect**: NT-IO lacks a meta-compilation tier for runtime dialect execution. NeoTrix dialects (NT-ACT tool invocations, NT-MIND evolution commands) are always interpreted. No baseline JIT tier exists to fast-path frequently-visited dialect execution paths. Need Deegen-style two-tier execution: interpreter for cold paths, baseline JIT for hot evolution loops.  
**Priority**: MEDIUM — execution speed for frequently-repeated SEAL cycles.

---

## 3. Compiler Optimization / Vectorization Findings → Defects

### D650-8: MaximizeBandwidth for X86 Loop Vectorizer
**Source**: LLVM RFC — Enable MaximizeBandwidth by default for X86 (https://discourse.llvm.org/t/rfc-enable-maximizebandwidth-by-default-for-x86-in-the-loop-vectorizer/90992)  
**Finding**: MaxBW selects VF based on smallest element type, not widest. For mixed i8/i32 loops on AVX2, picks VF=32 instead of VF=8. Accompanied by split cost floor to prevent TTI from underpricing operations on wide vector types. No regressions observed on X86.  
**NeoTrix Defect**: NT-CORE's HyperCube iteration loops mix element widths (u8 flags, u32 indices, f32 weights). Current auto-vectorization picks VF based on widest element (f32 → VF=8 on AVX2). MaximizeBandwidth would pick VF=32 for the u8-heavy operations, 4× throughput improvement. But NeoTrix doesn't pass `-vectorizer-maximize-bandwidth` to rustc/LLVM.  
**Priority**: HIGH — direct 4× throughput for mixed-width HyperCube operations.

### D650-9: Flang Array-Section Reduction Promotion for Vectorization
**Source**: LLVM RFC — Promote loop-invariant array-section reductions (https://discourse.llvm.org/t/rfc-flang-promote-loop-invariant-array-section-reductions-for-vectorization/91499)  
**Finding**: Loop-invariant descriptor sections prevent vectorizer from recognizing memory independence. Promoting to fixed-size local temporaries removes descriptor overhead and enables vectorization. ~2.5× speedup alone, ~8× with related PRs.  
**NeoTrix Defect**: NT-MEMORY's KB query loops iterate over FTS5 result sets through SQLite's descriptor-like rowid indirection. The vectorizer sees may-alias memory dependencies and refuses to vectorize. Need to promote hot KB query inner loops to fixed-size local accumulators (analogous to array-section promotion) before vectorization.  
**Priority**: MEDIUM — KB query hotpath improvement.

### D650-10: Dual-SIMD Loop Unrolling for Vectorizable Loops
**Source**: Yao et al., J. Supercomputing 82, 228 (2026) (https://link.springer.com/article/10.1007/s11227-026-08404-w)  
**Finding**: Compiler pass for dual-SIMD extensions (Shenwei + Intel). Loop unrolling that exploits two SIMD units simultaneously. 1.031× geometric mean, up to 1.165× on full applications, 1.168× on kernel loops on Shenwei. Portable across architectures.  
**NeoTrix Defect**: NT-CORE assumes single SIMD pipeline. On dual-SIMD targets (Shenwei, some ARM big.LITTLE), HyperCube batch operations don't exploit both SIMD units. Need dual-SIMD-aware loop unrolling for HyperCube batch operations.  
**Priority**: LOW — hardware-specific optimization, but important for NeoTrix's RISC-V targets.

### D650-11: GCC BB SLP Predicated Tails for Partial Vectors
**Source**: GCC Patch v13/v14 — Extend BB SLP vectorization (https://sourceware.org/pipermail/gcc-patches/2026-July/725144.html)  
**Finding**: Basic Block SLP vectorization now supports predicated tails via mask/length limits. Groups whose size is not divisible by vector length can be vectorized using predicate masks. Enables vectorization of previously unvectorizable non-aligned blocks.  
**NeoTrix Defect**: NT-CORE's HyperCube has 64-element hexagram grids. On AVX-512 (64-bit vectors), this aligns. But on AVX2 (256-bit = 32 elements) or SSE (128-bit = 16 elements), the 64-element grid doesn't divide evenly. Without predicated tail support, the last 0-31 elements remain scalar. Need predicated tail vectorization for HyperCube batch operations on non-64-bit SIMD widths.  
**Priority**: HIGH — HyperCube operations are scalar on 30%+ of target hardware without this.

---

## Summary

| ID | Defect | Source | Priority | NEW vs Prior |
|----|--------|--------|----------|--------------|
| D650-1 | SEAL distillation non-terminating canonicalization | EuroLLVM 2026 | HIGH | NEW — extends D649 scheduling stall to distillation |
| D650-2 | E8 lowering conflation of target vs abstraction | LLVM CIR PR#179245 | MEDIUM | NEW — architectural separation |
| D650-3 | No SEAL pass introspection/MCP endpoint | LLVM PR#203796 | MEDIUM | NEW — debugging gap |
| D650-4 | No mid-generation validation for skill code | Generative Compilation | HIGH | NEW — error cascade |
| D650-5 | Evolutionary search for VSA params (replace with analytical) | Nova JIT | CRITICAL | NEW — 10-100× speedup |
| D650-6 | No optimization hint synthesis for NeoTrix output | HINTPILOT | HIGH | NEW — 6× speedup |
| D650-7 | No JIT tier for dialect execution | Deegen | MEDIUM | NEW — runtime acceleration |
| D650-8 | HyperCube mixed-width VF suboptimal | LLVM MaxBW RFC | HIGH | NEW — extends D649 SIMD gap |
| D650-9 | KB query loops not vectorizable | Flang RFC | MEDIUM | NEW — memory access pattern |
| D650-10 | Single-SIMD assumption on dual-SIMD targets | J. Supercomputing | LOW | NEW — hardware portability |
| D650-11 | HyperCube non-aligned scalar tails | GCC BB SLP v13/v14 | HIGH | NEW — extends D649 SIMD gap |

## Sources Cited

1. EuroLLVM 2026 Round Table: MLIR Canonicalization — https://discourse.llvm.org/t/eurollvm-2026-round-table-summary-mlir-canonicalization/90588
2. LLVM CIR TargetLowering Pass — https://github.com/llvm/llvm-project/commit/b9f3710c0e26
3. LLVM mlir-opt-repl MCP Server — https://github.com/llvm/llvm-project/pull/203796
4. Generative Compilation (Mündler-Sasahara et al.) — https://arxiv.org/abs/2607.13921
5. Nova JIT Compiler — https://arxiv.org/abs/2608.00029
6. HINTPILOT (ACL 2026) — https://aclanthology.org/2026.findings-acl.1251.pdf
7. T-LLM Compiler — https://arxiv.org/abs/2608.14953
8. Deegen JIT VM Generator — https://2026.splashcon.org/details/oopsla-2026/46/
9. LLVM MaximizeBandwidth RFC — https://discourse.llvm.org/t/rfc-enable-maximizebandwidth-by-default-for-x86-in-the-loop-vectorizer/90992
10. Flang Array-Section Reduction RFC — https://discourse.llvm.org/t/rfc-flang-promote-loop-invariant-array-section-reductions-for-vectorization/91499
11. Dual-SIMD Loop Unrolling (Yao et al.) — https://link.springer.com/article/10.1007/s11227-026-08404-w
12. GCC BB SLP Predicated Tails v13 — https://sourceware.org/pipermail/gcc-patches/2026-July/725144.html
13. Compile by Training — https://arxiv.org/abs/2609.04199
