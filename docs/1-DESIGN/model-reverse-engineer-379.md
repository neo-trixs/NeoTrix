# Model Reverse Engineering — Cycle 379

**Date:** 2026-09-12
**Focus:** Recent papers on efficient inference, attention mechanisms, agent coordination, and multi-agent memory
**Sources:** arXiv (Apr-Sep 2026), ACL 2026, AAAI-26, ICLR 2026

---

## 1. Flux Attention — Context-Aware Layer-Level Hybrid Attention

**Source:** arXiv:2604.07394, Apr 2026

### Key Claims
- 2.8x prefill speedup, 2.0x decode speedup over baseline
- Only 12 hours training on 8x A800 GPUs (parameter-efficient)
- Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA)
- Outperforms static hybrid methods across long-context + math reasoning benchmarks

### Architecture Insights
- **Layer Router**: Lightweight router injected into frozen pretrained LLMs. Each layer independently routed to FA or SA based on input context.
- **Hard routing**: Binary decision per layer via argmax — no soft mixing overhead
- **Sparsity constraint**: Dynamic penalty prevents router from degenerating to all-FA
- **Contiguous memory access**: Layer-level routing preserves hardware-friendly memory patterns (unlike head-level sparsity which creates load imbalance)
- **Key insight**: Static allocation fails because different tasks have variable retrieval demands. Layer-level granularity is the sweet spot — fine enough for adaptation, coarse enough for hardware efficiency.

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | GWT layer-level attention routing | GWT could route attention at layer granularity — each module declares FA/SA needs per task |
| NT-IO | Cost-aware routing (Axiom A1) | Layer Router is a cost-aware decision: FA = expensive/high-fidelity, SA = cheap/low-fidelity. Direct map to model routing |
| NT-MEMORY | KV cache optimization | Layer-level sparsity reduces KV cache I/O. Aligns with paged KV virtualization (Axiom A2) |

### Absorption
- **Layer Router pattern**: Implement as GWT modifier — each specialist module declares attention mode per layer, not per-query
- **Sparsity constraint**: Add to SEAL pipeline — prevent evolution from degenerating to "all resources" allocation
- **Hardware-aware routing**: Layer-level granularity = the right abstraction level for NeoTrix's module routing

---

## 2. SAVeR — Self-Audited Verified Reasoning for Faithful Agent Beliefs

**Source:** ACL 2026 (acl-long.1440)

### Key Claims
- Enforces verification over internal belief states before action commitment
- Adversarial auditing localizes violations, constraint-guided minimal interventions repair
- Persona-based diverse candidate beliefs for selection under faithfulness-relevant structure space
- Addresses systematic behavioral drift in long-horizon agentic systems

### Architecture Insights
- **Problem**: Coherent reasoning can still violate logical/evidential constraints. Unsupported beliefs get repeatedly stored and propagated across decision steps → behavioral drift.
- **Three-phase pipeline**:
  1. **Persona-based belief generation**: Generate diverse candidate beliefs from different "persona" perspectives
  2. **Adversarial auditing**: Test each candidate against logical/evidential constraints
  3. **Minimal intervention repair**: Fix violations with smallest possible changes under verifiable acceptance criteria
- **Key insight**: Consensus ≠ faithfulness. Multiple agents agreeing doesn't mean the belief is logically sound. Need explicit verification before storage.

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MIND | Belief verification before experience absorption | Before writing to KB, verify belief consistency. Maps to SEAL distillation phase |
| NT-CORE | Anti-drift mechanism | Behavioral drift detection = meta-cognitive health monitoring. ConsciousnessTree branch for belief integrity |
| NT-SHIELD | Pre-commit validation | Faithfulness audit = inbound belief guard. Complements Egress Privacy Guard (outbound) |
| NT-MEMORY | Conflict detection + repair | Judge role in SAVeR maps to KB consistency checking. Refresher maps to experience-tree update |

### Absorption
- **Verify-before-commit**: Add faithfulness audit to experience-tree's distillation phase — verify beliefs before KB write
- **Persona-based diversity**: Generate multiple candidate interpretations of an experience before selecting the most faithful one
- **Minimal intervention repair**: When KB inconsistencies detected, repair with minimal changes rather than full rewrites

---

## 3. GLIDE — Guided Layerwise Hybrid Attention via Linear Recurrence

**Source:** arXiv:2607.24788, Jun 2026

### Key Claims
- Reduces KV cache I/O by non-uniformly compressing softmax footprint across layers
- Early layers: high sensitivity to softmax removal → keep softmax. Deep layers: redundancy → replace with linear recurrence
- Variable-sized softmax window per layer, not uniform
- Preserves expressive power where most vital while reducing aggregate KV cache I/O

### Architecture Insights
- **Layer-wise heterogeneity**: Different layers have fundamentally different sensitivity to attention mechanism changes
- **Hybrid per layer**: Each layer independently balances linear recurrence + variable softmax window
- **Non-uniform compression**: Unlike uniform hybrid approaches, GLIDE compresses aggressively where safe (deep layers) and conservatively where needed (early layers)
- **Storage-for-computation tradeoff**: Front-loads computation into offline prefill, amortized across queries

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | Layer-aware resource allocation | Different NT-* domains have different sensitivity to attention changes. Allocate resources non-uniformly |
| NT-MEMORY | Prefill-amortized knowledge loading | Precompute knowledge representations offline, amortize across queries. KB embedding precomputation |
| NT-IO | Adaptive inference budgets | Each module gets different compute budget based on sensitivity. Cost-Aware Routing (Axiom A1) refinement |

### Absorption
- **Non-uniform sensitivity**: Profile each NT-* domain's sensitivity to context changes. Allocate GWT attention budget accordingly
- **Prefill amortization**: Precompute expensive knowledge structures (embeddings, graphs) offline, serve from cache at runtime
- **Variable windows**: Each domain gets variable context window based on task demands, not fixed allocation

---

## 4. AMA — Adaptive Memory via Multi-Agent Collaboration

**Source:** arXiv:2601.20352, ACL 2026 Findings

### Key Claims
- 80% token reduction vs full-context methods while outperforming SOTA baselines
- Four specialized agents: Constructor, Retriever, Judge, Refresher
- Multi-granularity memory aligned with task complexity
- Iterative retrieval triggered by insufficient evidence; conflict detection triggers memory refresh

### Architecture Insights
- **Four-role decomposition**:
  1. **Constructor**: Builds multi-granularity memories from conversation history
  2. **Retriever**: Routes queries to appropriate memory granularity (coarse ↔ fine)
  3. **Judge**: Verifies relevance + consistency of retrieved content. Triggers iterative retrieval or Refresher
  4. **Refresher**: Enforces memory consistency — targeted updates or removal of outdated entries
- **Adaptive granularity**: Retrieval granularity dynamically matched to task complexity — simple tasks use coarse memory, complex tasks use fine-grained
- **Conflict-aware**: Judge detects logical conflicts → triggers Refresher for consistency maintenance
- **Five sentence patterns**: S-V, S-V-O, S-V-C, S-V-O-O, S-V-O-C as memory building blocks

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Multi-granularity memory architecture | KB supports coarse/fine retrieval. Constructor = experience-tree distillation, Retriever = KB query routing |
| NT-CORE | Judge role → meta-cognitive verification | ConsciousnessTree acts as Judge — verifies memory consistency before allowing propagation |
| NT-MIND | Refresher → experience lifecycle management | experience-tree's absorption already has update/delete. Formalize as Refresher role |
| NT-ACT | Task-complexity routing | Route to appropriate memory granularity based on task type. Aligns with GWT salience |

### Absorption
- **Four-role model**: Map to NeoTrix: Constructor=experience-tree, Retriever=KB query, Judge=ConsciousnessTree, Refresher=experience-tree update
- **Adaptive granularity**: KB query interface should support coarse (summary) and fine (full) retrieval modes
- **Conflict detection**: Add logical consistency checking to KB write path — detect contradictions before commit
- **Token efficiency**: 80% reduction via granularity matching — only retrieve what task complexity demands

---

## 5. EvoRoute — Experience-Driven Self-Routing for LLM Agent Systems

**Source:** ACL 2026 (acl-long.1771)

### Key Claims
- Breaks the Agent System Trilemma (accuracy vs efficiency vs resource use)
- Pareto-optimal model selection at each step, balancing accuracy + efficiency + cost
- Experience-driven: continuously refines routing strategy from environment feedback
- Fine-grained model selection per step, not per-task static assignment

### Architecture Insights
- **Agent Role Matching**: Identifies all historical steps performed by the same agent role — captures functionally equivalent precedents
- **Multi-faceted retrieval**: Combines role-based similarity + task difficulty + resource availability for routing decisions
- **Pareto-optimal selection**: Not just "best model" but best tradeoff across accuracy, latency, cost
- **Self-evolving**: Routing strategy improves from execution feedback — experience accumulation drives better decisions over time
- **Two paradigms**: (1) Static model-to-role mapping, (2) Dynamic per-step selection. EvoRoute transcends both.

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-IO | Dynamic provider routing | EvoRoute's per-step model selection = NT-IO's Ordered Backend Router with experience-driven refinement |
| NT-MIND | Experience-driven evolution | Routing strategy evolves from execution history. Maps to SEAL pipeline's self-improvement loop |
| NT-CORE | GWT cost-aware salience | Pareto-optimal selection = GWT attention with Axiom A1 cost weighting. Not just relevance but cost+latency+accuracy |
| NT-ACT | Role-based task routing | Agent Role Matching maps to NT-ACT's specialist routing — match task to domain by functional equivalence |

### Absorption
- **Experience-driven routing**: NT-IO provider selection should learn from past execution — track success/failure/cost per provider per task type
- **Pareto selection**: GWT salience should be multi-objective: relevance × cost × latency × reliability
- **Role matching**: Map task types to NT-* domains by functional similarity to historical precedents, not static rules
- **Trilemma breaking**: Accept that no single routing strategy is optimal — use Pareto frontier to offer tradeoff options

---

## Cross-Paper Synthesis

### Emerging Meta-Pattern: Layered Verification Before Commitment

All five papers converge on a pattern: **verify before you commit**.

| Paper | Verification Point | What's Verified |
|-------|-------------------|-----------------|
| Flux Attention | Layer router decision | Attention mode is appropriate for context |
| SAVeR | Belief before action | Logical/evidential faithfulness |
| GLIDE | Compression budget | Which layers can tolerate compression |
| AMA | Memory before retrieval | Relevance + consistency of stored memories |
| EvoRoute | Model before execution | Pareto optimality of provider selection |

**NeoTrix Implication**: Every write path (KB, experience, routing decision, provider selection) should have a verification gate. The existing Egress Privacy Guard is one instance — expand to all commitment points.

### Emerging Meta-Pattern: Non-Uniform Resource Allocation

| Paper | What's Non-Uniform | Granularity |
|-------|--------------------|-------------|
| Flux Attention | FA vs SA per layer | Per-layer |
| GLIDE | Compression depth | Per-layer |
| AMA | Memory retrieval granularity | Per-query |
| EvoRoute | Model selection | Per-step |
| SAVeR | Verification intensity | Per-belief |

**NeoTrix Implication**: Fixed resource allocation is wasteful. Every subsystem should allocate dynamically based on sensitivity/complexity/cost. Axiom A1 (Cost-Aware Routing) should permeate all layers.
