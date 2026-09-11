# Model Reverse Engineering — Cycle 319

**Date**: 2026-09-11
**Focus**: Efficient inference, novel attention, multi-agent coordination, memory systems, reasoning

---

## 5 Models/Papers for Reverse Engineering

### 1. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
- **Paper**: https://arxiv.org/abs/2609.07237 (Sep 2026)
- **Key Innovation**: Coarse-to-fine attention routing that preserves global coverage while achieving sparse attention speedups. Each semantic chunk contributes a cheap key-value summary to a residual attention path. Chunks with high estimated approximation error are expanded to exact token attention. Output-error bound governed by within-chunk key/value dispersion controls refinement budget.
- **Architecture Pattern**: Two-phase (summary → selective refinement) with mathematical error guarantees. Residual summaries reduce reconstruction error by 98%+ vs hard dropping. ~3x kernel speedup at 128K context.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (E8)**: Error-bounded refinement maps to E8 hexagram confidence propagation — each reasoning step has bounded error contribution
  - **NT-MEMORY**: Chunk summary + selective expansion parallels KB embedding (summary) + VSA retrieval (exact)
  - **GWT**: The residual attention path is an analog to salience broadcast — cheap summary always available, expensive detail only when needed

### 2. GLIDE: Guided Layerwise Hybrid Attention
- **Paper**: https://arxiv.org/abs/2607.24788 (Jun 2026)
- **Key Innovation**: Layer-wise heterogeneity insight: early layers are sensitive to softmax removal, deeper layers tolerate linear recurrence replacement. Non-uniformly compresses softmax footprint across model — different layers get different attention strategies based on their sensitivity profile.
- **Architecture Pattern**: Each transformer layer independently balances linear recurrence with variable-sized softmax window. Layer-specific adaptation rather than global uniform strategy.
- **NeoTrix Domain Mapping**:
  - **6-Layer Architecture**: Direct mapping — different architecture layers (L1-L6) should have different attention/computation strategies based on their role
  - **NT-CORE (GWT)**: Layer-wise routing = domain-wise attention allocation — cheap domains (NT-IO) get sparse attention, expensive domains (NT-CORE) get full attention
  - **Dual Specialization**: GLIDE's layer sensitivity profiling informs which weapon set (CORE+WORLD vs CORE+MIND) to activate per task type

### 3. MemMA: Memory Cycle Multi-Agent Coordination
- **Paper**: https://arxiv.org/abs/2603.18718 (Mar 2026)
- **Code**: https://github.com/ventr1c/memma
- **Key Innovation**: Two-path memory coordination. Forward path: Meta-Thinker (strategic reasoning) guides Memory Manager (construction) and Query Reasoner (iterative retrieval). Backward path: in-situ self-evolution — synthesize probe QA pairs, verify memory, convert failures into repair actions before memory commit.
- **Architecture Pattern**: Planner-Worker separation (Meta-Thinker ≠ Memory Manager). Backward path provides dense supervision for memory construction by generating synthetic verification probes.
- **NeoTrix Domain Mapping**:
  - **NT-MEMORY**: Direct implementation of KB quality gate — probe QA pairs verify experience entries before commit
  - **NT-MIND (SEAL)**: Backward path = SEAL Phase-0 converge_check — verify before absorb
  - **ConsciousnessTree**: Meta-Thinker maps to NT-META coordinator, Memory Manager to NT-MEMORY executor
  - **experience-tree**: The in-situ verification pattern should be added to our five-stage absorption protocol

### 4. Gated-Memory Routing
- **Paper**: https://arxiv.org/abs/2609.00237 (Aug 2026)
- **Code**: https://github.com/rajibrhasan/gated-memory-routing
- **Key Innovation**: Joint training of memory management + agent routing + halting policy. Memory Write Gate commits only non-redundant steps. Retrieval Gate surfaces compact step-relevant subsets. Adaptive Halting Controller stops when gated state is sufficient. End-to-end reward trades quality vs cost.
- **Architecture Pattern**: Gated execution memory as the central control signal — determines who acts next, what they read, and when to stop. Memory is not passive storage but active routing control.
- **NeoTrix Domain Mapping**:
  - **GWT**: The Retrieval Gate = salience filtering — only relevant context enters broadcast. Halting Controller = attention modulation
  - **NT-MEMORY**: Write Gate = experience-tree deduplication. Retrieval Gate = KB query optimization
  - **Axiom A1 (Cost-Aware)**: Adaptive halting = cost-aware routing — stop reasoning when evidence is sufficient, don't waste tokens
  - **Axiom A2 (Context as Scarce)**: Gated memory keeps context compact — exactly our KVMem compaction strategy

### 5. Flux Attention
- **Paper**: https://arxiv.org/abs/2604.07394 (Apr 2026)
- **Key Innovation**: Lightweight Layer Router evaluates semantic context and adaptively routes each transformer layer to Full Attention or Sparse Attention. Only router trained (12 hours, 8x A800). Layer-level routing preserves contiguous GPU memory access. Task-aware — retrieval-intensive tasks get more full attention, context-holistic tasks get more sparse.
- **Architecture Pattern**: Router (lightweight, trained) + Backbone (frozen, large). Router makes per-layer decisions based on input context. Decoupled prefill/decode routing decisions.
- **NeoTrix Domain Mapping**:
  - **Dual Specialization**: Layer Router = AttentionManager — routes between Weapon Set I (CORE+WORLD) and Weapon Set II (CORE+MIND) based on task context
  - **GWT**: The router is a GWT-like salience mechanism at the layer level — decides which layers get full compute
  - **SEAL Pipeline**: Router training (frozen backbone + lightweight fine-tuning) parallels our skill crystallization — train minimal new code, preserve existing knowledge
  - **Axiom A1**: Task-aware routing directly implements Cost-Aware Routing — retrieval tasks get expensive full attention, simple tasks get cheap sparse

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Two-Phase Coarse-to-Fine (CEDAR, Flux Attention)
Both papers use a cheap coarse phase followed by selective expensive refinement. This maps to our GWT architecture: cheap salience scoring → selective deep attention. Could improve our PerceptionBridge by adding error-bounded refinement.

### Pattern 2: Verify-Before-Commit (MemMA, Gated-Memory)
Both papers verify memory quality before committing to storage. Our experience-tree absorption protocol should adopt this: generate probe queries against new experience entries, verify they can be retrieved and used correctly, only then commit to KB.

### Pattern 3: Joint Training of Routing+Memory+Halting (Gated-Memory)
The insight that routing, memory management, and halting should be jointly trained rather than independently optimized is powerful. Our ConsciousnessTree could benefit from joint optimization of GWT salience, KB write gating, and reasoning depth.

### Pattern 4: Layer-Adaptive Computation (GLIDE, Flux Attention)
Different layers should get different computation budgets. Our 6-layer architecture (L1-L6) should have different attention strategies per layer — L1 Action gets sparse/fast, L5 Cognition gets full/expensive.

### Pattern 5: Model-Declared Attention (Declarative Attention)
The model itself declares where it needs to attend, eliminating external proxy scoring. Our ConsciousnessTree could adopt this: let the reasoning engine itself declare salient context regions rather than relying on external heuristics.

---

## Implementation Candidates

| Priority | Paper | Action | NeoTrix Component |
|----------|-------|--------|-------------------|
| P0 | MemMA | Add probe-qa verification to experience-tree backward path | experience-tree SKILL |
| P0 | Gated-Memory | Implement Write Gate for KB experience entries | nt_memory |
| P1 | Declarative Attention | Add focus/global/local modes to GWT salience | nt_core (GWT) |
| P1 | Flux Attention | Implement task-aware layer routing in AttentionManager | nt_core_self |
| P2 | CEDAR | Add error-bounded chunk summaries to KB embeddings | nt_memory |
| P2 | GLIDE | Profile layer sensitivity for 6-layer architecture | nt_meta |
