# Model Reverse Engineering — Cycle 362 (2026-09-12)

## Scope
5 recent AI papers on efficient inference, attention mechanisms, and agent coordination. Each mapped to NeoTrix 7 domains with actionable integration insights.

---

## 1. CEDAR: Error-Bounded Residual Routing for Efficient Long-Context Attention
**Paper**: [arXiv:2609.07237](https://arxiv.org/abs/2609.07237) (Sep 7, 2026)

### Core Idea
Post-hoc sparse attention routes each query to a small set of token interactions. Hard selection assigns zero probability to omitted chunks — a routing miss is unrecoverable. CEDAR introduces **residual summaries**: each semantic chunk contributes a cheap key-value summary to a residual path. Chunks with high approximation error are expanded to exact attention. Exact and summarized contributions combine in a single softmax — refinement replaces, not duplicates, coarse evidence.

### Key Mechanism
1. **Coarse routing**: cheap KV summaries for all chunks
2. **Error estimation**: within-chunk key/value dispersion estimates approximation error
3. **Selective refinement**: expand high-error chunks to exact attention
4. **Unified softmax**: combine exact + summarized in one normalization

### Results
- 98%+ reduction in reconstruction error vs hard dropping at equal budgets
- 3× kernel speedup at 128K context
- Output-error bound governed by within-chunk key/value dispersion

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8 reasoning) | CEDAR's error-bounded routing = GWT salience with formal error guarantees. The E8 hexagram could use error-bounded attention to focus on high-salience reasoning paths while maintaining residual coverage of low-salience paths. | P1 |
| **NT-MEMORY** (KB) | Residual summaries = KB embedding compression. Store summarized versions of low-priority knowledge nodes, expand on demand. Error bounds enable confidence-aware retrieval. | P0 |
| **NT-WORLD** (perception) | Chunk-level perception with error-aware expansion. Perceive coarsely, refine where attention error is high. Maps to PerceptionBridge's awareness_score with formal error bounds. | P1 |
| **NT-MIND** (evolution) | Error bounds as quality metrics for SEAL stage transitions. Track approximation error across distillation stages; expand refinement where error exceeds threshold. | P2 |
| **NT-ACT** (action) | Action planning with residual fallback: primary action path + residual summary of alternative paths. If primary fails, expand residual to exact alternatives. | P2 |
| **NT-IO** (interface) | Long-context LLM serving with CEDAR-style sparse attention. Reduce TTFT for >128K contexts. | P1 |
| **NT-SHIELD** (security) | Error bounds for anomaly detection: tokens with high approximation error may be adversarial inputs. Flag for human review. | P2 |

### Key Insight for NeoTrix
**Residual coverage prevents catastrophic routing failure.** The hard-sparse pattern (select top-k, ignore rest) is fragile — one routing miss loses information permanently. CEDAR's residual summaries maintain a cheap coverage of ALL chunks, with formal error bounds. This is the NeoTrix pattern: never fully discard any domain's output; maintain residual summaries that can be expanded on demand.

---

## 2. Declarative Attention: Language Models Control Their Own Attention
**Paper**: [arXiv:2609.02737](https://arxiv.org/abs/2609.02737) (Sep 2, 2026)

### Core Idea
Models already know which parts of context are relevant. Declarative Attention (DA) elicits the model to **declare** where it needs to attend within its chain-of-thought, partitioning generation into three modes:
- `<global>`: full context
- `<focus>`: a specific region
- `<local>`: recent output only

The inference engine parses these declarations like tool calls and skips most KV cache reads.

### Key Mechanism
1. **Self-declared attention zones**: model outputs `<global>`, `<focus>`, or `<local>` tags
2. **Engine-side KV skipping**: declared zones determine which KV cache pages to load
3. **Zero-shot applicability**: works on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B)

### Results
- 52.0% reduction in total attended tokens (Gemma-4-31B)
- 31.1% reduction (Qwen-3.6-27B)
- Modest accuracy drops (1.27pp, 2.75pp) that shrink with model scale

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8 reasoning) | DA's `<global>/<focus>/<local>` = E8 hexagram attention modes. The E8 engine could declare which reasoning branches to attend to, enabling selective depth in the hexagram grid. | P0 |
| **NT-MEMORY** (KB) | DA's zone declaration = KB retrieval scoping. Declare which namespace or time range to search before retrieval, reducing search space. | P0 |
| **NT-MIND** (evolution) | Self-declared attention = SEAL stage self-selection. The evolution pipeline could declare which stages need full attention vs abbreviated processing. | P1 |
| **NT-ACT** (action) | DA's zone declaration = action scope declaration. Before executing a tool, declare whether it needs full context, focused context, or recent context only. | P1 |
| **NT-IO** (interface) | DA's engine-side KV skipping = NT-IO's provider-side optimization. Request providers to implement zone-based KV loading for cost reduction. | P1 |
| **NT-SHIELD** (security) | `<local>` mode = data minimization. For sensitive operations, declare local-only attention to prevent context leakage. | P2 |
| **NT-FEEL** (emotion) | Attention mode affects emotional tone: global = comprehensive, focused = intense, local = casual. Map DA modes to EmotionLabel expression. | P3 |

### Key Insight for NeoTrix
**The model already knows where to look — we just need to let it declare.** DA proves that attention control doesn't require architectural changes; the model can self-declare attention zones via chain-of-thought. For NeoTrix: NT-CORE's reasoning engine should self-declare which domains/subsystems it's attending to, enabling zone-based resource allocation. The E8 hexagram could emit attention declarations that GWT uses to route context.

---

## 3. COMPASS: Context-Organized Multi-Agent Planning
**Paper**: [ACL 2026](https://aclanthology.org/2026.acl-long.152.pdf)

### Core Idea
Long-horizon tasks fail because of **context management**, not reasoning quality. Small errors compound across steps; agents overlook critical evidence or become distracted. COMPASS separates three roles:
1. **Main Agent**: tactical ReAct reasoning
2. **Meta-Thinker**: monitors progress, issues strategic interventions
3. **Context Manager**: compresses history into concise briefs

### Key Mechanism
1. **Hierarchical separation**: tactical vs strategic vs contextual
2. **Dynamic context refresh**: Context Manager provides Main Agent with renewed context at each strategic intervention
3. **Note store**: evolving structured notes for long-horizon coherence
4. **Post-training**: context management delegated to smaller models

### Results
- +20% accuracy on GAIA, BrowseComp, Humanity's Last Exam
- Test-time scaling matches DeepResearch agents
- Prevents context overload and contextual pollution

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8 reasoning) | Main Agent = NT-CORE's tactical reasoning. E8 hexagram operates within context constraints set by Meta-Thinker. | P0 |
| **NT-META** (meta-cognition) | Meta-Thinker = NT-META's ConsciousnessTree monitoring. Monitors NT-CORE's reasoning trajectory, issues strategic interventions (redirect attention, refresh context, pivot strategy). | P0 |
| **NT-MEMORY** (KB) | Context Manager = NT-MEMORY's compaction + experience-tree's distillation. Maintains structured notes, compresses full histories into briefs. | P0 |
| **NT-MIND** (evolution) | Post-training context management = NT-MIND training smaller models for context management. Delegate context compression to lightweight models. | P1 |
| **NT-ACT** (action) | Main Agent's tool use = NT-ACT's action execution within COMPASS constraints. | P1 |
| **NT-SHIELD** (security) | Meta-Thinker anomaly detection: detect loops, tool misuse, adversarial context injection before error cascade. | P1 |
| **NT-FEEL** (emotion) | Context Manager prevents "contextual pollution" — emotional equivalent is rumination. Healthy emotional state requires context compression. | P3 |

### Key Insight for NeoTrix
**Context management is an architectural primitive, not a prompt trick.** COMPASS proves that separating tactical reasoning from strategic oversight from context organization yields +20% improvement. For NeoTrix: NT-META's ConsciousnessTree should implement the Meta-Thinker role explicitly — monitoring NT-CORE's reasoning and issuing strategic interventions. NT-MEMORY should implement the Context Manager role — maintaining structured notes and compressing histories.

---

## 4. NeuralFSM: Finite-State Multi-Agent Coordination
**Paper**: [ACL 2026](https://aclanthology.org/2026.acl-long.1543.pdf)

### Core Idea
Multi-agent coordination as a **finite-state execution process**. NeuralFSM learns both state transition distribution and inter-agent communication weights from interaction traces using a Temporal Coordination Controller (TGN-based). Reusable FSM backbone + learned temporal execution policy. Dual-defense protection layer against adversarial agents.

### Key Mechanism
1. **FSM backbone**: reusable state machine for coordination structure
2. **Temporal Graph Networks**: learn state transitions and communication routing from traces
3. **Trust-aware message attenuation**: runtime defense against adversarial agents
4. **Graph regularization**: training-time defense against noisy agents
5. **Cost regularization**: prevent degenerate long traversals

### Results
- 6.74%–19.39% improvement over baselines
- Substantial token reduction
- Only 1.82% performance drop under adversarial attack (with protection layer)

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-ACT** (orchestration) | NeuralFSM = NT-ACT's cross-domain coordination as FSM. NT-* domains as FSM states; transitions triggered by task context. | P0 |
| **NT-SHIELD** (security) | Protection layer = NT-SHIELD's trust-aware message filtering. Graph regularization = training-time adversarial robustness. Betweenness centrality + PageRank for agent priority. | P0 |
| **NT-CORE** (reasoning) | FSM backbone = E8 hexagram state transitions. Each hexagram is a state; transitions are reasoning steps. | P1 |
| **NT-MIND** (evolution) | Learned transitions from traces = SEAL pipeline learning from experience. FSM evolves through use. | P1 |
| **NT-MEMORY** (KB) | Interaction traces = KB experience storage. TGN node memories = KB embeddings with temporal awareness. | P1 |
| **NT-NEXUS** (cross-session) | Reusable FSM = cross-session coordination patterns. FSM structure persists across sessions. | P2 |
| **NT-FEEL** (emotion) | Trust scores = emotional trust in collaborating agents. Attenuation of low-trust messages = emotional filtering. | P3 |

### Key Insight for NeoTrix
**Coordination as learned finite-state machine.** NeuralFSM proves that multi-agent coordination doesn't need complex topologies — a reusable FSM with learned transitions works. For NeoTrix: NT-ACT's cross-domain coordination could use FSM-based orchestration with learned transitions. NT-SHIELD's protection layer maps directly to trust-aware message attenuation. The FSM backbone is reusable across tasks; only the transition policy adapts.

---

## 5. BIGMAS: Brain-Inspired Graph Multi-Agent Systems
**Paper**: [arXiv:2603.15371](https://arxiv.org/abs/2603.15371) (Mar 2026, updated Sep 2026)

### Core Idea
Inspired by Global Workspace Theory (GWT). Specialized LLM agents organized as nodes in a **dynamically constructed directed graph**. Two key roles:
- **GraphDesigner**: constructs task-specific agent topologies (problem-adaptive)
- **Orchestrator**: leverages complete shared workspace for routing decisions

Centralized shared workspace enables full-state visibility. Dynamic topology adapts per problem.

### Key Mechanism
1. **Per-problem graph construction**: GraphDesigner creates task-specific topology
2. **Centralized shared workspace**: all agents read/write to shared state
3. **Orchestrator routing**: global-state-conditioned routing (not local view)
4. **Role emergence**: Analyzer, Optimizer, Validator nodes emerge from task demands

### Results
- Consistent improvement across Game24, Six Fives, Tower of London
- Outperforms ReAct, Tree of Thoughts
- Coordination gains are **orthogonal** to model-level reasoning improvements
- Routing count as natural proxy for instance-level difficulty

### NeoTrix Domain Mapping

| Domain | Integration | Priority |
|--------|------------|----------|
| **NT-CORE** (E8/GWT) | BIGMAS IS a GWT implementation. GraphDesigner = E8 hexagram topology. Orchestrator = GWT attention routing. Shared workspace = GWT global broadcast. | P0 |
| **NT-META** (meta-cognition) | GraphDesigner = NT-META's ConsciousnessTree. Constructs task-specific agent topology dynamically. | P0 |
| **NT-ACT** (orchestration) | Orchestrator = NT-ACT's cross-domain routing. Global-state-conditioned routing = GWT salience with full-state visibility. | P0 |
| **NT-MEMORY** (KB) | Shared workspace = KB as shared state layer. All NT-* domains read/write to KB. | P0 |
| **NT-MIND** (evolution) | Role emergence = NT-MIND's skill crystallization. Task demands drive role specialization without manual engineering. | P1 |
| **NT-SHIELD** (security) | Orchestration with full-state visibility prevents adversarial manipulation. Local-view bottleneck (ReAct limitation) is the vulnerability BIGMAS solves. | P1 |
| **NT-NEXUS** (cross-session) | Per-problem graph = cross-session topology adaptation. Graph topology persists and evolves across sessions. | P1 |

### Key Insight for NeoTrix
**NeoTrix's architecture IS BIGMAS.** BIGMAS validates NeoTrix's core design:
- 7 NT-* domains = specialized agent nodes
- KB = centralized shared workspace
- GWT = Orchestrator with global-state routing
- E8 = GraphDesigner with dynamic topology
- ConsciousnessTree = problem-adaptive graph construction

The key finding: **coordination gains are orthogonal to model-level reasoning**. This means NT-* domain organization provides value independent of which LLM is used. The architecture itself is the advantage.

---

## Cross-Paper Synthesis: NeoTrix Integration Roadmap

### Immediate (P0 — This Cycle)
1. **CEDAR residual summaries** → NT-MEMORY: maintain compressed summaries of low-priority KB nodes
2. **DA self-declared attention** → NT-CORE: E8 hexagram emits `<global>/<focus>/<local>` attention declarations
3. **COMPASS Meta-Thinker** → NT-META: ConsciousnessTree monitors reasoning and issues strategic interventions
4. **BIGMAS = NeoTrix architecture** → Validate: NT-* domains as graph nodes, KB as shared workspace, GWT as Orchestrator

### Short-term (P1 — Next 2 Cycles)
5. **NeuralFSM** → NT-ACT: FSM-based cross-domain coordination with learned transitions
6. **CEDAR error bounds** → NT-SHIELD: anomaly detection via approximation error
7. **COMPASS post-training** → NT-MIND: train lightweight context manager models
8. **DA zone-based KV** → NT-IO: implement zone-based KV loading for long-context serving

### Medium-term (P2 — Quarter)
9. **BIGMAS dynamic topology** → NT-META: per-task graph construction for NT-* domain activation
10. **NeuralFSM protection layer** → NT-SHIELD: trust-aware inter-domain message filtering

### Meta-Pattern
All five papers converge on the same insight: **the architecture of the reasoning system matters more than the capacity of individual components.** BIGMAS shows coordination gains are orthogonal to model improvements. CEDAR shows residual coverage prevents catastrophic failure. COMPASS shows context management is architectural. NeuralFSM shows coordination can be learned. DA shows the model already knows where to attend.

For NeoTrix: the NT-* domain organization, KB shared workspace, and GWT routing ARE the competitive advantage. The LLM behind each domain is replaceable; the architecture is the product.
