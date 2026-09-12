# Model Reverse Engineering — Cycle 403

**Date**: 2026-09-12
**Focus**: Recent papers on efficient inference, attention, agent coordination, memory systems

---

## Paper 1: Declarative Attention (DA) — Self-Declared Sparse Attention
**arXiv**: 2609.02737 (Sep 2, 2026)
**Source**: Gemma-4-31B, Qwen-3.6-27B zero-shot

### Pattern
Model declares WHERE it needs to attend during chain-of-thought, partitioning generation into three modes:
- `<global>` — full context
- `<focus>` — specific region
- `<local>` — recent output only

Inference engine parses declarations like tool calls and skips KV cache reads.

### Results
- 52.0% / 31.1% reduction in total attended tokens (Gemma-4-31B / Qwen-3.6-27B)
- Modest accuracy drops (1.27pp / 2.75pp) that shrink with model scale

### NeoTrix Mapping
| Domain | Integration |
|--------|-------------|
| **NT-CORE (GWT)** | Declarative Attention is a form of intrinsic salience routing — the model itself tells GWT what's relevant, replacing proxy scoring |
| **NT-MEMORY** | KV cache skip is memory optimization — maps to `kv_cache_optimizer.rs` extension with attention-skip hints |
| **NT-PHYSICAL** | Resource conservation — fewer GPU cycles on irrelevant context |

### Actionable Insight
NeoTrix GWT could adopt a "declaration protocol" where attention-routed modules emit `<focus>` tags declaring their active regions, enabling downstream modules to skip irrelevant KV reads. This inverts the current approach (external scoring → routing) to (self-declared intent → routing).

---

## Paper 2: CEDAR — Error-Bounded Residual Attention Routing
**arXiv**: 2609.07237 (Sep 7, 2026)

### Pattern
Coarse-to-fine error-aware dynamic attention routing:
1. Each semantic chunk contributes a cheap key-value summary (residual path)
2. Chunks with high estimated approximation error are expanded to exact token attention
3. Exact + summarized combined in single softmax — refinement REPLACES coarse evidence
4. Variable refinement budget allocated by output-error bound

### Results
- 98% reconstruction error reduction vs hard dropping
- ~3x kernel speedup at 128K context
- Quality recovered vs hard sparse routing

### NeoTrix Mapping
| Domain | Integration |
|--------|-------------|
| **NT-CORE (HyperCube)** | Residual summaries as compressed VSA representations — low-rank embedding of semantic chunks |
| **NT-MEMORY** | Error-bounded memory compression — apply same "refine what matters" pattern to KB embedding storage |
| **NT-MIND** | Variable refinement budget = adaptive compute allocation, maps to SEAL pipeline resource awareness |

### Actionable Insight
CEDAR's "error-bound" concept could be applied to NeoTrix's experience-tree absorption: instead of uniform compression, estimate which experience chunks have high approximation error and preserve them in full, while summarizing the rest.

---

## Paper 3: ReActNet — Inference-Time Graph Engineering for Multi-Agent Workflows
**arXiv**: 2609.05774 (Sep 4, 2026)

### Pattern
- Synthesizes task-conditioned TEMPORAL workflow graphs at inference time
- Each graph snapshot = one reasoning stage
- Each edge carries NL instruction specifying message source→target
- Graph compilation SEPARATED from graph execution
- No RL or gradient-based topology optimization needed

### Results
- Consistently improves over fixed-topology and learned-topology baselines
- Competitive inference cost
- Works across knowledge reasoning, math, code, GAIA tasks

### NeoTrix Mapping
| Domain | Integration |
|--------|-------------|
| **NT-CORE (E8)** | Temporal workflow graphs map to E8 hexagram sequences — each hexagram is a reasoning stage snapshot |
| **NT-MIND (SEAL)** | Graph compilation = SEAL exploration phase; execution = SEAL distillation phase |
| **NT-ACT** | Dynamic orchestration — replace static capability routing with task-conditioned temporal graphs |

### Actionable Insight
NeoTrix's E8 reasoning engine could generate temporal workflow graphs where each hexagram stage specifies which specialist modules communicate and what messages flow. This makes multi-agent coordination explicit and inspectable, like ReActNet's edge-level NL instructions.

---

## Paper 4: CondenseFlow — Semantic Compression for Multi-Agent Latent Communication
**ACL 2026 Findings**

### Pattern
- Latent Thought Condenser (LTC): learnable semantic probes compress KV caches into FIXED-SIZE representations
- O(1) communication complexity regardless of context length
- Cross-attention aggregation discovers valuable patterns through end-to-end learning
- Compression error bounded by attention concentration

### Results
- 99%+ memory reduction vs dense transfer
- ~20% inference latency reduction
- 1.7pp accuracy improvement over text-based methods
- Stable across 7 benchmarks, 6 models

### NeoTrix Mapping
| Domain | Integration |
|--------|-------------|
| **NT-MEMORY** | Fixed-size compressed memory representations for cross-session knowledge transfer |
| **NT-CORE (HyperCube)** | LTC probes as VSA dimensionality reduction — compress high-dim embeddings to fixed-width symbolic vectors |
| **NT-MIND** | Semantic compression for skill crystallization — compress experience into compact, transferable skill representations |

### Actionable Insight
NeoTrix's inter-domain communication (NT-CORE↔NT-MIND↔NT-MEMORY) could use fixed-size compressed representations instead of full KV or text transfers. LTC's "attention concentration as compressibility metric" maps directly to HyperCube embedding density.

---

## Paper 5: Dual-Layer Agentic Memory (Fast Write / Slow Consolidation)
**arXiv**: 2608.22215 (Aug 23, 2026)

### Pattern
Inspired by Complementary Learning Systems (CLS) neuroscience:
1. **Fast Write Routing**: Cost-aware epistemic routing classifies incoming info as non-write/write-new/write-update via small→large model cascade
2. **Slow Consolidation**: Periodic parametric consolidation (SFT) internalizes high-value external memories into model parameters
3. 1.7B→8B cascade: prunes 68% redundant memory, escalates <50% of inputs, retains 98% QA EM

### Results
- 68% redundant memory pruned
- <50% inputs escalated to larger model
- 98% downstream QA accuracy retained
- Adaptive router suppresses redundant writes as model's epistemic boundaries evolve

### NeoTrix Mapping
| Domain | Integration |
|--------|-------------|
| **NT-MEMORY** | Write-phase routing = KB ingestion pipeline optimization. Slow consolidation = experience-tree absorption. |
| **NT-MIND (SEAL)** | Consolidation is SEAL Phase 4 (落盘) — internalize high-value experiences into system parameters |
| **NT-CORE (ConsciousnessTree)** | Epistemic boundary tracking maps to module maturity awareness — what does the system already know? |

### Actionable Insight
NeoTrix's experience-tree could adopt dual-layer routing: (1) Fast path classifies new experience as skip/insert/update via cheap model cascade, (2) Slow path periodically consolidates high-value experiences into system state via SEAL distillation. This prevents memory bloat while preserving critical knowledge.

---

## Cross-Paper Synthesis

### Unified Pattern: Self-Declared, Error-Bounded, Compressed Coordination

| Paper | Core Mechanism | NeoTrix Integration Point |
|-------|---------------|---------------------------|
| DA | Model declares attention regions | GWT declaration protocol |
| CEDAR | Error-bounded residual routing | Experience compression with quality bounds |
| ReActNet | Temporal workflow graphs | E8 hexagram sequencing for agent coordination |
| CondenseFlow | Fixed-size semantic compression | Cross-domain compressed communication |
| Dual-Layer Memory | Fast write + slow consolidation | Experience-tree dual-phase absorption |

### Meta-Pattern: Memory as Active Control, Not Passive Storage

All five papers converge on treating memory/computation as ACTIVE CONTROL signals rather than passive storage:
- DA: Attention regions as control declarations
- CEDAR: Error bounds as quality control gates
- ReActNet: Graph compilation as coordination control
- CondenseFlow: Compression as communication control
- Dual-Layer: Routing as epistemic boundary control

**NeoTrix Implication**: The existing architecture already treats memory as active (KB pipeline, experience-tree). The papers validate this direction and add three specific mechanisms:
1. **Declaration protocols** (DA) — modules declare their resource needs
2. **Error bounds** (CEDAR) — quality-aware compression budgets
3. **Temporal graph compilation** (ReActNet) — dynamic coordination topology

### Priority Actions

| Priority | Action | Paper Source |
|----------|--------|-------------|
| P0 | Implement GWT declaration protocol — modules emit `<focus>` tags | DA |
| P1 | Add error-bounded compression to experience-tree absorption | CEDAR + Dual-Layer |
| P1 | Design temporal workflow graph generation for E8 reasoning | ReActNet |
| P2 | Implement fixed-size compressed cross-domain communication | CondenseFlow |
| P2 | Add fast-write/slow-consolidation dual phase to KB ingestion | Dual-Layer Memory |
