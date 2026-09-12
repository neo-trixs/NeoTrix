# Model Reverse Engineering — Cycle 380

**Date:** 2026-09-12
**Focus:** Efficient inference, attention mechanisms, agent coordination, memory architecture
**Sources:** arXiv (May-Sep 2026), ICLR 2026, ACL 2026, ICML 2026 Workshop

---

## 1. Latent Action Reparameterization (LAR) — Efficient Agent Inference

**Source:** arXiv:2605.18597, May 2026

### Key Claims
- Learns compact latent action space where each latent action = multi-step semantic behavior
- Reduces effective action horizon dramatically while preserving expressiveness
- Significant reductions in action tokens + wall-clock inference time
- Maintains or improves task success rates across LLM agent benchmarks

### Architecture Insights
- **Problem**: LLM agents rely on long sequences of low-level textual actions → large decision horizons → high inference cost
- **Solution**: Learn latent action representations from agent trajectories, integrated directly into model
- **Key difference from macros/hierarchical controllers**: Latent actions are learned, not hand-crafted. Both planning and execution operate over abstract action representations.
- **Bottleneck identified**: Action space representation itself (not just system-level optimization or prompting)

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-ACT | Action abstraction | NT-ACT tool calls could use learned latent action spaces — compress multi-step tool sequences into single latent actions |
| NT-MIND | SEAL pipeline optimization | LAR's trajectory-based learning maps to SEAL's experience distillation — learn action patterns from successful cycles |
| NT-CORE | E8 reasoning efficiency | Latent actions reduce E8 state space exploration — each node in hexagram grid represents latent action, not raw tool calls |

### Absorption
- **Latent action as NT-ACT primitive**: Instead of raw tool-call sequences, NT-ACT could maintain learned action embeddings. One latent action = "fetch+parse+transform" rather than 3 separate calls.
- **Trajectory learning for SEAL**: LAR's trajectory-based training maps to SEAL's cycle history — extract common action patterns as latent actions.
- **Horizon compression**: Reduces GWT broadcast overhead — fewer action tokens = less salience computation.

---

## 2. Agent-Radar — Attention Steering with Temporal-Spatial Decay

**Source:** arXiv:2605.30136, May 2026

### Key Claims
- Training-free context management for multi-agent systems
- Temporal + spatial decay mechanism for attention steering
- Up to 7.64 absolute point gains across 5 benchmarks
- Robust as number of agents and interaction rounds increases

### Architecture Insights
- **Core problem**: Multi-agent conversations accumulate long histories → relevant information diluted by irrelevant context
- **Temporal decay**: Recent context weighted higher, older context exponentially decayed
- **Spatial decay**: Information from distant conversation turns weighted lower regardless of relevance score
- **Training-free**: No fine-tuning required — operates at inference time only
- **Key insight**: Static context window management fails because relevance is both time-dependent and distance-dependent

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-CORE | GWT salience decay | GWT attention routing should include temporal decay factor — recent broadcasts salient, old broadcasts decayed |
| NT-MEMORY | Memory retrieval ranking | KB query results should incorporate temporal + spatial decay — recent experiences ranked higher |
| NT-NEXUS | Cross-session bridging | Nexus weaver should decay old session context when bridging to new sessions |

### Absorption
- **GWT temporal decay**: Add time-decay multiplier to salience computation. `salience = base_score × e^(-λ × elapsed_time)`
- **Memory retrieval decay**: When querying KB experiences, weight recent entries higher. Prevents stale knowledge from dominating.
- **Multi-agent coordination**: Agent-Radar's robustness under scaling validates NeoTrix's multi-agent SEAL approach — attention steering scales with agent count.

---

## 3. Semantics-Aware Memory Hierarchy — Zero-Approximation-Error Offloading

**Source:** arXiv:2605.09490, May 2026 (ICML 2026 Workshop)

### Key Claims
- 4-tier memory hierarchy: HBM → DDR → compressed → evicted
- Low-importance tokens moved to CPU memory, not destroyed
- Prefetched back at full precision before attention — zero approximation error
- 3% eviction retains 91% full-cache accuracy on GSM8K, 71% on MATH-500
- 5-7% transfer overhead for full preservation

### Architecture Insights
- **Critical finding**: Accuracy depends on how many tokens are permanently discarded (eviction ratio), NOT on how many remain in HBM
- **Cumulative attention scoring**: Tokens sorted into tiers by accumulated attention weight across heads
- **Zero-approximation-error offloading**: Tokens not approximated when offloaded — moved to CPU, prefetched at full precision before use
- **vs R-KV (SOTA eviction)**: R-KV achieves only 0-32% accuracy at comparable budgets — destruction is catastrophic for reasoning

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Tiered knowledge storage | KB could use hot/warm/cold storage tiers — frequently accessed nodes in fast storage, others in cold |
| NT-CORE | Context window management | GWT broadcast could use tiered KV — high-salience in GPU, low-salience prefetched on demand |
| NT-IO | Cost-aware inference | Tiered storage = cost-aware memory: expensive GPU for hot data, cheap CPU for warm, disk for cold |

### Absorption
- **KB tiered storage**: NT-MEMORY's KB could implement hot/warm/cold tiers. Frequently accessed nodes stay in fast cache, rarely accessed nodes in cold storage.
- **Context as tiered resource**: Aligns with Axiom A2 (context as scarce resource). Don't evict — tier. Eviction ratio is the critical parameter, not HBM occupancy.
- **GWT broadcast optimization**: High-salience broadcasts in fast memory, low-salience prefetched on demand. Reduces GWT memory footprint without accuracy loss.

---

## 4. REAL — Reasoning-Enhanced Graph for Long-Term Memory

**Source:** arXiv:2606.10694, Jun 2026

### Key Claims
- Reasoning-enhanced graph framework for LLM long-term memory
- Multi-dimensional entity representation with temporal context
- Non-destructive temporal evolution — parallel edges record complete attribute history
- Counterfactual reasoning over implicit logical relations (symmetry, hyponymy, temporal succession)

### Architecture Insights
- **Multi-dimensional entity representation**: Entities have temporal context, confidence scores, and relationship history
- **Incremental graph update with non-destructive temporal evolution**: New facts don't overwrite old — they add parallel edges with non-overlapping intervals
- **Counterfactual reasoning**: Can infer new facts from implicit relations — "likes" ↔ "dislikes", "basketball shoes" ⊂ "sports shoes"
- **Conflict resolution**: When conflicting facts arrive, old interval closed, new fact inserted with own interval — complete evolution history preserved

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-MEMORY | Temporal knowledge graphs | KB nodes could have temporal validity intervals — track how knowledge evolves over time |
| NT-NEXUS | Cross-session evolution | Nexus weaver could maintain temporal edges — track how patterns evolve across sessions |
| NT-MIND | SEAL cycle history | Each SEAL cycle adds parallel edges — track how module capabilities evolve through cycles |

### Absorption
- **Temporal KB nodes**: NT-MEMORY's knowledge nodes should have temporal validity — `[τ_start, τ_end]` intervals. Old knowledge preserved, not overwritten.
- **Counterfactual reasoning**: KB should support inference over implicit relations — if A→B and B→C, infer A→C. Currently NT-MEMORY is retrieval-only.
- **Non-destructive updates**: When new knowledge contradicts old, close old interval and add new. Complete evolution history = audit trail.

---

## 5. Multi-Agent Inference with Large Models — Reasoning Token Reuse

**Source:** arXiv:2604.04929, Apr 2026

### Key Claims
- Large model with fewer output tokens can be MORE efficient than small model with long output
- Large models achieve better/comparable performance with significantly fewer output tokens
- Multi-agent framework reuses reasoning tokens from small model when necessary
- Output token count, not model size, is the end-to-end latency bottleneck

### Architecture Insights
- **Key insight**: Autoregressive decoding makes output tokens the dominant latency component — more tokens = more sequential steps
- **Large model advantage**: Better reasoning → shorter output → faster inference despite larger compute per token
- **Token reuse**: Small model generates reasoning tokens, large model reuses them when needed — hybrid approach
- **Framework**: Large model keeps short responses, small model provides reasoning scaffolding, transfer reasoning tokens cross-model

### NeoTrix Domain Mapping
| Domain | Pattern | Implementation |
|--------|---------|----------------|
| NT-IO | Model routing (A1) | Route easy tasks to small model, hard tasks to large. But also: reuse reasoning tokens across models |
| NT-CORE | GWT task decomposition | GWT could route subtasks to cheapest capable model, reuse reasoning across specialist modules |
| NT-MIND | Distillation pipeline | Reasoning token reuse = runtime distillation — small model reasoning bootstraps large model output |

### Absorption
- **Cross-model reasoning reuse**: NT-IO's provider router could cache reasoning tokens from cheap models and inject into expensive model prompts — "reasoning transfer"
- **Output token budget**: Add output token count as routing factor. Prefer models that produce shorter outputs for same quality.
- **Hybrid inference**: Small model generates reasoning scaffold, large model refines. Two models cooperate on single task — aligns with dual specialization (Weapon Set I/II).

---

## Cross-Paper Synthesis

### Unified Theme: Efficiency Through Representation, Not Destruction

All 5 papers share a common insight: **don't destroy information, restructure it**.

| Paper | Destruction to Avoid | Restructuring Instead |
|-------|---------------------|----------------------|
| LAR | Raw action sequences | Latent action embeddings |
| Agent-Radar | Truncated context | Temporal-spatial decay weighting |
| Memory Hierarchy | KV cache eviction | Tiered offloading with prefetch |
| REAL | Overwritten knowledge | Temporal intervals with parallel edges |
| Multi-Agent Inference | Single-model inference | Cross-model reasoning token reuse |

**NeoTrix implication**: Every subsystem should be audited for information destruction. KB should never overwrite — tier. Context should never truncate — decay. Actions should never be raw — learn latent representations. GWT should never discard broadcasts — tier them.

### Implementation Priority

| Priority | Pattern | Source | Effort |
|----------|---------|--------|--------|
| P0 | Temporal decay for GWT salience | Agent-Radar | Low — multiplier on existing salience |
| P0 | Zero-eviction tiered memory | Memory Hierarchy | Medium — 4-tier KB storage |
| P1 | Latent action spaces for NT-ACT | LAR | High — requires trajectory learning |
| P1 | Temporal validity intervals for KB nodes | REAL | Medium — schema extension |
| P2 | Cross-model reasoning token reuse | Multi-Agent Inference | High — requires provider coordination |
| P2 | Counterfactual reasoning over KB | REAL | High — graph inference engine |
