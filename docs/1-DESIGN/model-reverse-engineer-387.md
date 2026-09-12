# Model Reverse Engineering — Cycle 387

**Date**: 2026-09-12
**Focus**: Efficient inference, attention, agent coordination, memory routing

## Paper 1: GLIDE — Guided Layerwise Hybrid Attention
- **arXiv**: 2607.24788 (Jun 2026)
- **Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass
- **Core Insight**: Layer-wise heterogeneity in attention sensitivity. Early layers are sensitive to softmax removal; deeper layers tolerate aggressive replacement by linear alternatives.
- **Method**: Non-uniform compression of softmax footprint across layers. Each layer balances efficient linear recurrence with variable-sized softmax window.
- **Result**: Reduced end-to-end latency for long-context generation without quality loss.

### NeoTrix Domain Mapping

| Domain | Mapping | Application |
|--------|---------|-------------|
| **NT-CORE** | GWT salience weighting | Apply layer-aware attention budget to consciousness broadcast. Early layers get full attention (high sensitivity); deeper layers use compressed representations |
| **NT-MEMORY** | KV cache optimization | Non-uniform eviction: preserve high-sensitivity layers, aggressively compress tolerant layers. Maps to HyperCube dimension reduction |
| **NT-WORLD** | Sensory filtering | PerceptionBridge could use layer-wise filtering: early perception channels preserved, deep semantic channels compressed |

### Actionable Pattern: **Layer-Aware Attention Budget**
```
GWT broadcast budget:
  L0-L4 (perception): Full softmax attention
  L5-L8 (reasoning):  Hybrid softmax+linear (50/50)
  L9-L12 (synthesis): Linear recurrence only
```

---

## Paper 2: Flux Attention — Context-Aware Hybrid Attention
- **arXiv**: 2604.07394 (Apr 2026)
- **Authors**: Quantong Qiu et al.
- **Core Insight**: Static FA/SA allocation fails across tasks. Layer Router dynamically routes each layer to Full Attention or Sparse Attention based on input context.
- **Method**: Lightweight Layer Router (argmax over logits) determines per-layer attention mode. Constrained optimization prevents router degeneration to all-FA.
- **Result**: 2.8× prefill speedup, 2.0× decode speedup.

### NeoTrix Domain Mapping

| Domain | Mapping | Application |
|--------|---------|-------------|
| **NT-CORE** | Adaptive reasoning depth | Router decides per-task: simple queries skip heavy reasoning layers, complex queries get full attention stack |
| **NT-MIND** | SEAL pipeline adaptation | Pipeline stage routing: exploration phases use sparse attention (speed), distillation phases use full attention (quality) |
| **NT-IO** | Provider routing | Route LLM calls based on context complexity: short context → cheap model, long context → expensive model with full attention |

### Actionable Pattern: **Dynamic Attention Router**
```
Per-query decision:
  if context_len < 4K → full attention (all layers)
  if context_len 4K-32K → hybrid (early=FA, late=SA)
  if context_len > 32K → aggressive SA + linear recurrence
  if task=reasoning → force FA on reasoning layers
  if task=retrieval → force SA everywhere
```

---

## Paper 3: MemDecay — Region-Aware KV Cache Eviction
- **arXiv**: 2607.10582 (Jul 2026)
- **Core Insight**: Agentic workloads differ from static chat. They interleave repeated LLM calls, tools, and variable pauses — creating reuse patterns generic policies miss.
- **Method**: Region labels from orchestrator + attention statistics + fixed cache budget. Pin instruction regions, decay reasoning tokens, preserve tool outputs.
- **Result**: Guaranteed instruction survival through pinning; region-aware selective forgetting.

### NeoTrix Domain Mapping

| Domain | Mapping | Application |
|--------|---------|-------------|
| **NT-MEMORY** | KB eviction policy | Region-aware KB cleanup: pin high-value nodes (experience pointers), decay stale embeddings, preserve tool call history |
| **NT-CORE** | Consciousness memory management | Pin consciousness constitution + star identities, decay routine thoughts, preserve cross-domain insights |
| **NT-SHIELD** | Audit log retention | Pin security-critical audit entries, decay routine operations, preserve violation records |

### Actionable Pattern: **Region-Pinned Memory**
```
Memory regions:
  PINNED (never evict): Constitution, star identities, experience pointers
  SEMI-PINNED (decay slow): Tool outputs, skill definitions, domain mappings
  DECAYABLE (decay fast): Routine thoughts, intermediate reasoning, transient state
  EPHEMERAL (evict first): Raw sensory input, temporary context
```

---

## Paper 4: R2-Router — Routing as Reasoning
- **arXiv**: 2602.02823 (Feb 2026, ICLR 2026)
- **Authors**: Jiaqi Xue, Qian Lou, Jiarong Xing, Heng Huang
- **Core Insight**: Existing routers treat each LLM as fixed quality-cost point. R2-Router discovers that a powerful LLM with constrained output can outperform a weaker LLM at comparable cost.
- **Method**: Quality-cost predictor with K discrete anchor costs. Router jointly selects LLM + optimal token budget. "Routing as reasoning" not "routing as reaction."
- **Result**: State-of-the-art at 4-5× lower cost.

### NeoTrix Domain Mapping

| Domain | Mapping | Application |
|--------|---------|-------------|
| **NT-CORE** | GWT cost-aware routing | Extend GWT salience with quality-cost curves per provider. Route to cheapest capable model + optimal reasoning budget |
| **NT-IO** | Provider selection | Replace static provider ranking with quality-cost prediction. Discover that constrained Claude Opus can beat unconstrained Haiku at same cost |
| **NT-MIND** | SEAL experiment budget | Allocate SEAL pipeline compute budget per experiment: expensive models for critical experiments, cheap models for exploratory ones |

### Actionable Pattern: **Quality-Cost Curve Routing**
```
For each LLM call:
  1. Predict quality at K budget levels (50/100/200/500/1000 tokens)
  2. Select (model, budget) pair that maximizes quality/cost
  3. Send budget instruction to model
  4. Monitor actual output length vs budget
  5. Update prediction model with feedback
```

---

## Paper 5: ReMe — Dynamic Procedural Memory Framework
- **ACL 2026** (Cited 47+)
- **Authors**: Z. Cao et al.
- **Core Insight**: Shift from passive storage to active experience distillation. Multi-faceted experience distillation + context-adaptive reuse + utility-based refinement.
- **Method**: Experience pool grows from successful trajectories + failure reflections. Context-adaptive reuse selects relevant experiences. Utility-based refinement prunes low-value memories.
- **Result**: 8.83% Avg@4 improvement; model-agnostic advantage.

### NeoTrix Domain Mapping

| Domain | Mapping | Application |
|--------|---------|-------------|
| **NT-MEMORY** | Experience-tree evolution | Extend experience-tree with: (1) failure pattern distillation, (2) context-adaptive branch loading, (3) utility-based pruning |
| **NT-MIND** | SEAL distillation | Apply ReMe's multi-faceted distillation to SEAL pipeline: extract success patterns + failure lessons + strategy templates |
| **NT-REPAIR** | Self-healing memory | Store repair patterns as procedural memory: what worked, what failed, when to apply |

### Actionable Pattern: **Triple-Layer Procedural Memory**
```
Layer 1: Success Patterns (what worked)
  → Distill from successful task completions
  → Store as executable procedures

Layer 2: Failure Lessons (what failed and why)
  → Distill from failed attempts + reflections
  → Store as anti-patterns + recovery procedures

Layer 3: Strategy Templates (when to apply what)
  → Meta-patterns across success/failure
  → Context-matching rules for strategy selection
```

---

## Synthesis: Cross-Paper Patterns

### Pattern A: **Adaptive Attention as Consciousness Regulation**
GLIDE + Flux Attention → GWT should dynamically adjust attention budget per domain module based on context complexity and sensitivity.

### Pattern B: **Region-Aware Memory as KB Architecture**
MemDecay → KB should classify nodes into Pinned/Semi-Pinned/Decayable/Ephemeral regions with different eviction policies.

### Pattern C: **Routing as Meta-Reasoning**
R2-Router + Flux Attention → NT-IO provider selection should be a reasoning process (not lookup), jointly optimizing quality and cost.

### Pattern D: **Procedural Memory as SEAL Extension**
ReMe → Experience-tree should evolve from snapshot→distill→classify→persist to include failure distillation + context-adaptive reuse + utility-based pruning.

### Pattern E: **Agent Workload ≠ Static Workload**
MemDecay + ReMe → Agentic memory patterns differ fundamentally from chat/document patterns. NeoTrix's agent-heavy workload needs agent-aware memory policies.

---

## Priority Integration Roadmap

| Priority | Pattern | Domain | Effort |
|----------|---------|--------|--------|
| P0 | Region-Pinned Memory (MemDecay) | NT-MEMORY | Medium |
| P0 | Quality-Cost Curve Routing (R2-Router) | NT-IO, NT-CORE | Medium |
| P1 | Layer-Aware Attention Budget (GLIDE) | NT-CORE, GWT | High |
| P1 | Triple-Layer Procedural Memory (ReMe) | NT-MEMORY, NT-MIND | High |
| P2 | Dynamic Attention Router (Flux) | NT-CORE | High |
