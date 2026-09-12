# Model Reverse Engineering — Cycle 385

**Date**: 2026-09-12
**Focus**: Efficient inference, attention mechanisms, agent coordination

---

## Model 1: GLIDE — Guided Layerwise Hybrid Attention

**Paper**: arXiv:2607.24788 (Jun 2026)
**Authors**: Vimal William, Ravi Tandon, Jyotikrishna Dass
**Category**: Efficient inference / KV cache optimization

### Core Idea
Layer-wise heterogeneity in transformers: early layers are sensitive to softmax removal, deeper layers tolerate aggressive replacement by linear alternatives. GLIDE introduces layer-wise adaptive hybrid attention — each layer balances efficient linear recurrence with variable-sized softmax window.

### Key Mechanism
- **Non-uniform compression**: Unlike uniform hybrid approaches, GLIDE compresses softmax footprint differently per layer
- **Layer-wise adaptation**: Each layer independently selects its attention mix based on sensitivity analysis
- **KV cache I/O reduction**: Reduces aggregate KV cache I/O while preserving expressive power where vital

### Results
- Superior performance-efficiency tradeoffs vs uniform hybrid approaches
- Reduced end-to-end latency for long-context generation without quality loss

### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Layer-wise attention = consciousness layer hierarchy | Map L1-L6 layers to GLIDE's layer sensitivity model — each layer has different attention requirements |
| **NT-MEMORY** | KV cache compression | KVMem paged KV + GLIDE hybrid attention for >256K sessions |
| **NT-MIND** | Adaptive resource allocation | SEAL pipeline stages map to layer-wise resource adaptation |

### Absorption Vector
**Priority P1** — Layer-wise hybrid attention directly applicable to NeoTrix's 6-layer architecture. Each consciousness layer (L1-L6) should have different attention allocation strategies, not uniform GWT broadcast.

---

## Model 2: Flux Attention — Context-Aware Hybrid Attention

**Paper**: arXiv:2604.07394 (Apr 2026)
**Authors**: Quantong Qiu et al.
**Category**: Efficient inference / adaptive attention routing

### Core Idea
Context-aware framework that dynamically optimizes attention computation at the layer level. Lightweight Layer Router integrated into frozen pretrained LLMs adaptively routes each layer to Full Attention or Sparse Attention based on input context.

### Key Mechanism
- **Layer Router**: Lightweight module that decides FA vs SA per layer per input
- **Parameter-efficient**: Only 12 hours training on 8×A800 GPUs
- **Contiguous memory access**: Preserves hardware acceleration despite sparsity
- **Task-adaptive**: Routes differently for retrieval vs reasoning tasks

### Results
- Up to 2.8× speedup in prefill, 2.0× in decode
- Superior trade-off vs static allocation methods

### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Layer Router = attention salience router | Flux Router pattern for GWT: route information to specialist modules based on context, not fixed topology |
| **NT-WORLD** | Task-adaptive perception | Different attention strategies for different input types (code vs text vs structure) |
| **NT-ACT** | Dynamic resource routing | Route agent tasks to cheapest capable model based on context (Cost-Aware Routing axiom) |

### Absorption Vector
**Priority P1** — Layer Router pattern directly maps to GWT attention routing. NeoTrix should implement context-aware module routing instead of static broadcast. The "12 hours training on 8 GPUs" efficiency makes this practical for adaptation.

---

## Model 3: MATSIR — Multi-Agent Enhanced Monte Carlo Tree Search

**Paper**: ACL Findings 2026 (Li et al.)
**Category**: Agent coordination / inductive reasoning

### Core Idea
Plug-and-play test-time framework integrating Multi-Agent coordination with Monte Carlo Tree Search (MCTS) for inductive reasoning. Dual-reward mechanism provides explicit refinement signals — promotes logically coherent and semantically enriched hypotheses, not mere rephrasing.

### Key Mechanism
- **MAS + MCTS fusion**: Multi-agent hypotheses explored via tree search
- **Dual-reward**: Logical coherence + semantic enrichment (two independent signals)
- **Error correction**: Tree structure prevents error accumulation unlike iterative self-refinement
- **Plug-and-play**: No architectural changes to base model

### Results
- Superior to iterative self-refinement (which suffers from superficial rewording)
- Dual-reward prevents "rephrasing trap" in hypothesis generation

### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE (E8)** | E8 Hexagram + MCTS = structured reasoning exploration | MATSIR's tree search maps to E8 hexagram state space exploration with multi-agent hypothesis validation |
| **NT-MIND** | Dual-reward distillation | SEAL distillation should use dual reward: logical coherence + semantic novelty (not just "does it compile") |
| **NT-REPAIR** | Error correction via tree structure | Self-healing should explore repair hypotheses via tree search, not single-path retry |

### Absorption Vector
**Priority P1** — MATSIR's dual-reward mechanism solves the "rephrasing trap" in SEAL distillation. E8 hexagram exploration + multi-agent tree search is a natural fit. The plug-and-play nature means no architectural changes needed.

---

## Model 4: SILO-BENCH — Distributed Coordination Evaluation

**Paper**: ACL 2026 (Zhang et al.)
**Category**: Multi-agent systems / evaluation

### Core Idea
Scalable environment for evaluating distributed coordination in multi-agent LLM systems. Discovers a fundamental **Communication-Reasoning Gap**: agents communicate actively yet fail to translate interaction into effective distributed computation.

### Key Mechanism
- **Scalable evaluation**: Tests 50+ agents on Level-III complexity tasks
- **Communication-Reasoning Gap**: More communication ≠ better reasoning
- **Information silos**: Agents cannot escape silos through coordination alone
- **Zero success threshold**: Performance collapses beyond 50 agents at high complexity

### Results
- Level-III tasks: zero success beyond 50 agents
- Current LLMs fundamentally limited in distributed reasoning coordination
- Communication overhead becomes counterproductive past threshold

### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Broadcast threshold | GWT must respect the Communication-Reasoning Gap — broadcast more ≠ better. Need salience filtering to prevent information overload |
| **NT-ACT** | Agent count limits | Multi-agent orchestration should cap active agents per task based on complexity (Dual Specialization: 2 weapon sets, not 50) |
| **NT-MEMORY** | Information silo detection | KB should detect when agents are in silos and force cross-pollination |

### Absorption Vector
**Priority P2** — SILO-BENCH's Communication-Reasoning Gap is a critical constraint for NeoTrix's multi-domain architecture. The 50-agent collapse threshold validates NeoTrix's approach of 7 domains + specialized skills (not 50 independent agents). GWT salience filtering should enforce this.

---

## Model 5: AgensFlow — Coordination-Policy Substrate

**Paper**: arXiv:2605.27466 (May 2026)
**Author**: Nicole Koenigstein
**Category**: Multi-agent coordination / policy learning

### Core Idea
Treats multi-agent coordination as an online policy-learning problem under partial observability. Makes coordination decisions observable and learnable from repeated trajectories, rather than treating skill/role/model/topology choices as fixed pipeline design.

### Key Mechanism
- **Four-way interaction surface**: Task signatures × Skills × Models × Topology
- **Online policy learning**: Coordination improves from reward signals over repeated runs
- **Partial observability**: True state includes user intent, latent difficulty, retrieval quality, model failure modes, intermediate reasoning quality — none directly observable
- **Reasoning as optional**: Extended reasoning can increase latency/failure paths; AgensFlow learns when to use it

### Results
- Coordination policy learned online from repeated reward signals
- Makes topology choice (parallel/swarm/hierarchical) a learnable parameter
- Demonstrates that static pipelines provide limited view of design space

### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE (GWT)** | Learnable attention routing | GWT should learn routing policies from execution traces, not hardcoded topology |
| **NT-MIND** | SEAL + policy learning | SEAL pipeline should learn which coordination topology works for which task class |
| **NT-ACT** | Dynamic topology | Agent topology (parallel/sequential/hierarchical) should be a runtime choice, not architecture decision |
| **NT-GOVERNANCE** | Observable coordination | All coordination decisions should be auditable (AgensFlow's observability = governance traceability) |

### Absorption Vector
**Priority P1** — AgensFlow's "coordination as learnable policy" is the missing piece for NeoTrix's self-evolution. SEAL should not just distill knowledge but also learn coordination patterns. The four-way interaction surface (task×skill×model×topology) maps directly to NeoTrix's domain routing.

---

## Cross-Model Synthesis

### Emerging Pattern: Adaptive Layer-wise Intelligence

All five models converge on a single insight: **uniform treatment of layers/agents/stages is suboptimal**.

| Model | Uniform → Adaptive |
|-------|-------------------|
| GLIDE | Uniform attention → layer-wise hybrid |
| Flux | Static FA/SA ratio → context-aware routing |
| MATSIR | Iterative self-refinement → tree-structured multi-agent search |
| SILO-BENCH | More communication → communication threshold |
| AgensFlow | Fixed topology → learnable coordination policy |

### NeoTrix Integration Matrix

| Layer | Current | After Absorption |
|-------|---------|------------------|
| L1 Action | Static tool routing | Flux-style context-aware task routing |
| L2 Perception | Uniform sensory processing | GLIDE-style layer-sensitive attention |
| L3 Embodiment | Fixed body schema | AgensFlow-style learnable topology |
| L4 Emotion | Static emotion labels | MATSIR-style dual-reward (coherence + novelty) |
| L5 Cognition | GWT broadcast | SILO-BENCH-aware broadcast with salience threshold |
| L6 Meta | Periodic SEAL cycles | AgensFlow online policy learning from traces |

### Priority Absorption Order

1. **P1: Flux Attention Layer Router** → GWT refinement (highest impact, most practical)
2. **P1: MATSIR Dual-Reward** → SEAL distillation quality (solves rephrasing trap)
3. **P1: AgensFlow Policy Learning** → Self-evolution coordination (paradigm shift)
4. **P2: GLIDE Layer-wise Hybrid** → KV cache optimization (complementary to KVMem)
5. **P2: SILO-BENCH Threshold** → Architecture constraint (validates existing 7-domain design)

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| GLIDE needs training vs Flux trains in 12hrs | Use Flux's lightweight router approach (parameter-efficient); GLIDE's insight for architectural design |
| MATSIR adds agents vs SILO-BENCH caps agents | MATSIR uses tree-structured agents (few, structured), not swarm (many, unstructured). Complementary. |
| AgensFlow online learning needs repeated trajectories | Bootstrap with SEAL historical cycles; new policies start with prior knowledge |
| More communication hurts (SILO-BENCH) but agents need coordination (AgensFlow) | AgensFlow learns *when* to communicate, not *more* communication. Quality > quantity. |
