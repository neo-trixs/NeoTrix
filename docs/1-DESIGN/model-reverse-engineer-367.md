# Model Reverse Engineering — Cycle 367

Date: 2026-09-12
Scope: Recent papers on efficient inference, attention, agent coordination

## 5 Models/Papers Analyzed

---

### 1. Declarative Attention (DA)

**Paper**: [2609.02737] Language Models Can Control Their Own Attention
**Date**: 2026-09-02 | **Venue**: arXiv

#### Core Innovation
Models declare *where* they need to attend within their chain-of-thought, partitioning generation into three modes:
- `<global>` — full context
- `<focus>` — specific region
- `<local>` — recent output only

The inference engine parses these declarations like tool calls and skips most KV cache reads.

#### Key Results
- 52% reduction in attended tokens on Gemma-4-31B with only 1.27pp accuracy drop
- 31% reduction on Qwen-3.6-27B with 2.75pp drop
- Zero-shot on off-the-shelf models — no training required

#### Reverse Engineering the Pattern
The model learns to self-partition its attention budget. This is **intrinsic sparse attention** — the model itself decides what to read, rather than an external proxy scoring relevance. The three modes map directly to a state machine:
```
global → scan all context
  ↓ (finds relevant region)
focus → narrow to specific tokens
  ↓ (context shifts)
local → only recent output
```

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** (GWT) | DA's mode-switching maps to GWT attention routing. `<global>` = broadcast, `<focus>` = targeted selection, `<local>` = suppressed. | GWT salience scoring could incorporate DA-style declaration signals |
| **NT-MEMORY** (KV) | DA's KV skip pattern ≈ KVMem's paged KV tiering. Both reduce O(N) reads to O(k). | GrowPage (paper 3) already implements on-demand KV budgeting — DA provides the declaration protocol |
| **NT-MIND** (evolution) | Self-declared attention is a form of meta-cognition — the model reasons about its own reasoning path. | ConsciousnessTree branch health could use DA-style declarations to route attention |

#### Reuse Potential: HIGH
DA is zero-shot applicable to existing models. Could be integrated as a GWT pre-filter: before full attention pass, generate DA declarations to identify focus regions. Reduces GWT's O(N) broadcast cost.

---

### 2. BeaconKV: KV Cache Compression via Beacon Queries

**Paper**: [2609.04971] BeaconKV
**Date**: 2026-09-04 | **Venue**: ICML 2026

#### Core Innovation
Large Reasoning Models generate Thought Revisiting Tokens (TRT) that re-attend to distant context (e.g., early plans). These TRT cluster into similarity groups in embedding space. BeaconKV maintains **beacon queries** — compact representatives for each cluster — to anticipate which KV pairs will be revisited without storing entire query history.

#### Key Results
- Up to 5.8× memory reduction
- Nearly preserves full cache accuracy
- 4.3× throughput improvement
- Training-free

#### Reverse Engineering the Pattern
```
Observation: Reasoning models revisit old context (TRT)
  ↓
Insight: TRT cluster in embedding space (small # of groups)
  ↓
Solution: Store beacon queries per cluster
  ↓
At inference: beacon queries predict which KV pairs will be revisited
  ↓
Result: Only store high-probability KV pairs
```

This is a **predictive compression** pattern — instead of reacting to attention patterns, you predict them from structural signals (TRT clustering).

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** (KV) | BeaconKV's cluster-based prediction ≈ NeoTrix's KB embedding clusters. Both identify recurring patterns. | Extend `kv_cache_optimizer.rs` with beacon query tracking for reasoning traces |
| **NT-CORE** (E8) | TRT clustering resembles E8 hexagram pattern recognition — recurring reasoning states form geometric clusters. | E8 state machine could predict TRT recurrence patterns |
| **NT-MIND** (SEAL) | Beacon queries as compressed experience pointers — similar to experience-tree's hub index pattern. | SEAL pipeline could use beacon-style summarization for cross-session memory |

#### Reuse Potential: HIGH
BeaconKV is training-free and directly applicable to NeoTrix's KV cache management. The beacon query concept maps cleanly to KB hub indexing.

---

### 3. HeRo: History-Aware Routing

**Paper**: [2609.08189] HeRo
**Date**: 2026-09-08 | **Venue**: arXiv

#### Core Innovation
Dynamic layer routing skips layers per token, but existing methods treat each routing decision independently. HeRo introduces **router memory** — an explicit routing state across model depth, constructed via linear attention that incrementally aggregates preceding routing scores.

#### Key Results
- On Llama 3.1-8B: bypasses 26.87% of parameters while achieving 100.24% of dense performance
- At tighter budget: retains 97.01% while bypassing 38.82%
- Ablation: removing routing history degrades most on multistep reasoning and code generation

#### Reverse Engineering the Pattern
```
Previous: router(h_t) → skip/execute (stateless, per-layer)
HeRo: router(h_t, memory_t) → skip/execute (stateful)
  where memory_t = linear_attention(memory_{t-1}, scores_{t-1})
```

The memory is a compact representation of *all previous routing decisions*. This turns routing from a sequence of independent choices into a path-dependent process. The key insight: **routing decisions are coupled across depth** — earlier decisions shape what downstream routers see.

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** (GWT) | HeRo's router memory ≈ GWT's resonance-based routing with history. GWT currently uses per-step salience; HeRo shows path-dependent routing is superior. | Extend GWT salience with accumulated routing state (linear attention over routing history) |
| **NT-MIND** (evolution) | Path-dependent routing = learning from routing trajectory. HeRo's memory is a compressed evolution history. | ConsciousnessTree cycle pointer could accumulate routing decisions across growth cycles |
| **NT-PHYSICAL** (body schema) | HeRo's "bypassing parameters" ≈ physical embodiment's energy management — skip unnecessary compute like skipping unnecessary motor commands. | Layer-skip budgeting for constrained devices |

#### Reuse Potential: MEDIUM-HIGH
HeRo's linear attention router memory is lightweight and applicable to GWT's attention routing. The path-dependent insight is fundamental — NeoTrix's current routing is per-step, not path-dependent.

---

### 4. CoSkill: Hierarchical Skill Evolution

**Paper**: [2609.04865] CoSkill
**Date**: 2026-09-04 | **Venue**: arXiv

#### Core Innovation
Recasts static meta-skill workflows as a learnable **Meta-Skill Agent** jointly trained with a **Reasoning Agent** over a hierarchical skill library. Two agents share a single backbone — the Reasoning Agent conditions actions on retrieved skills, while task performance guides the Meta-Skill Agent in refining those skills.

#### Key Results
- ALFWorld: 98.4% success (+3.5pp over prior SOTA)
- WebShop: 90.6% (+6.2pp)
- Superior early-stage sample efficiency and asymptotic performance

#### Reverse Engineering the Pattern
```
Traditional: Skill Library → Reasoning Agent picks skill → executes
CoSkill: 
  Reasoning Agent ←→ Meta-Skill Agent (shared backbone)
    ↕                        ↕
  retrieves step skills    refines skill library
    ↕                        ↕
  task performance signal  guides skill evolution
```

The key insight: skills are not passive objects to be managed, but active agents that co-evolve with the reasoning agent. This is **bidirectional skill adaptation** — the agent learns to use skills better while the skill library learns to serve the agent better.

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** (skill nodes) | CoSkill's hierarchical skill library ≈ NeoTrix's Skill Tree (3 tiers: Small/Notable/Keystone). CoSkill adds bidirectional co-evolution. | Skill Tree nodes could implement CoSkill's meta-skill agent pattern for self-improving skill nodes |
| **NT-MIND** (SEAL) | CoSkill's joint training = SEAL's self-test + distillation cycle, but bidirectional. | SEAL Phase-5 (absorption) could incorporate CoSkill-style feedback from task performance |
| **NT-CORE** (E8) | Skill hierarchy maps to E8's hexagram state transitions — different reasoning states trigger different skill compositions. | E8 state machine could model skill retrieval patterns |

#### Reuse Potential: MEDIUM
CoSkill requires RL training infrastructure. The conceptual pattern (bidirectional skill evolution) is more immediately applicable than the implementation. NT-MIND could adopt CoSkill's feedback loop design without the full RL training.

---

### 5. CEDAR: Error-Bounded Residual Routing

**Paper**: [2609.07237] CEDAR
**Date**: 2026-09-07 | **Venue**: arXiv

#### Core Innovation
Post-hoc sparse attention that preserves global coverage. Each semantic chunk contributes a cheap key-value summary to a **residual attention path**. Chunks with high estimated approximation error are expanded to exact token attention. Exact and summarized contributions are combined in a single softmax — refinement replaces, not duplicates, coarse evidence.

#### Key Results
- Recovers most quality lost by hard sparse routing
- Maintains ~3× kernel speedup at 128K context
- Output-error bound governed by within-chunk key/value dispersion

#### Reverse Engineering the Pattern
```
Hard sparse: route → attend to top-k chunks → ignore rest (routing miss = permanent error)
CEDAR: route → attend to top-k chunks exactly + summarize rest as residuals
  ↓
Error estimation per chunk (key/value dispersion)
  ↓
Expand high-error chunks to exact attention
  ↓
Single softmax: exact + residual (refinement replaces coarse)
```

The key insight: **residual summaries reduce error by 98%** vs hard dropping at equal exact-chunk budgets. The single softmax normalization ensures refinement replaces rather than duplicates coarse evidence.

#### NeoTrix Domain Mapping
| Domain | Mapping | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** (GWT) | CEDAR's residual path ≈ GWT's broadcast with fallback. High-salience items get full attention; low-salience items get summary attention. | GWT could implement CEDAR-style error-bounded routing: full attention for high-salience, summary for low-salience |
| **NT-MEMORY** (KB) | CEDAR's chunk summarization ≈ KB's BM25 index + embedding hybrid. Both provide rough retrieval + precise refinement. | KB search could use CEDAR-style residual summaries for tiered retrieval |
| **NT-MIND** (distillation) | CEDAR's error-bounded refinement ≈ SEAL's distillation step — compress, then refine based on quality signals. | SEAL Phase-2 (distillation) could use error-bounded quality thresholds |

#### Reuse Potential: HIGH
CEDAR's residual routing is a general attention optimization pattern. Directly applicable to GWT's broadcast mechanism and KB's search pipeline. The error-bounded refinement is the key innovation — instead of binary attend/skip, use a quality-aware tiered approach.

---

## Cross-Paper Synthesis

### Meta-Pattern: The Attention Hierarchy

All five papers address the same fundamental problem: **attention is O(N) but only O(k) is useful**. They propose a hierarchy:

| Level | Paper | Mechanism |
|-------|-------|-----------|
| **Declaration** | DA (paper 1) | Model declares attention zones |
| **Prediction** | BeaconKV (paper 2) | Predict future attention from TRT clusters |
| **Path-dependence** | HeRo (paper 3) | Routing decisions coupled across depth |
| **Bidirectional** | CoSkill (paper 4) | Skill use ↔ skill evolution co-adapt |
| **Residual** | CEDAR (paper 5) | Summary + refinement with error bounds |

These are **complementary, not competing** — they operate at different levels of the attention stack:
```
Declaration (DA) → what to attend to
  ↓
Prediction (BeaconKV) → when to attend
  ↓
Path-dependence (HeRo) → how routing history shapes future routing
  ↓
Bidirectional (CoSkill) → how skill use shapes skill evolution
  ↓
Residual (CEDAR) → how to combine exact + summary attention
```

### NeoTrix Integration Priority

| Priority | Pattern | Paper | Integration |
|----------|---------|-------|-------------|
| **P0** | Residual routing | CEDAR | GWT broadcast enhancement — error-bounded attention tiers |
| **P0** | Beacon queries | BeaconKV | KV cache prediction — anticipatory compression |
| **P1** | Declaration protocol | DA | GWT pre-filter — model declares focus zones |
| **P1** | Router memory | HeRo | Path-dependent GWT routing with accumulated state |
| **P2** | Bidirectional skill evolution | CoSkill | SEAL feedback loop — task performance guides skill refinement |

### Axiom Alignment

| Axiom | Paper Alignment |
|-------|-----------------|
| **A1: Cost-Aware Routing** | DA (52% token reduction), BeaconKV (5.8× memory), HeRo (38% parameter bypass) — all reduce cost proportional to task difficulty |
| **A2: Context as Scarce Resource** | CEDAR (error-bounded compression), BeaconKV (predictive compression) — both maximize information per KV slot |
| **A3: Skill as Production Template** | CoSkill (bidirectional skill evolution) — skills as active agents, not passive templates |

### Contradiction & Resolution

| Tension | Resolution |
|---------|-----------|
| DA zero-shot vs HeRo trained routers | DA is the declaration layer; HeRo is the routing layer. Both can coexist — DA declares zones, HeRo routes within zones |
| BeaconKV predictive vs CEDAR reactive | Complementary: BeaconKV predicts before attention; CEDAR handles errors during attention |
| CoSkill's bidirectional evolution vs NeoTrix's Dark Forest | CoSkill's meta-skill agent = Dark Forest with feedback — skills that don't perform get pruned, skills that do get refined |
