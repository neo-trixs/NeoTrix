# Research Batch 210 — arXiv 2026-09-01~09-08

> Generated: 2026-09-09 | Papers: 9 | Focus: Agent Architecture, Self-Evolution, Security, Reasoning

---

## 1. AuK: Open-Source Foundational Model for Speech Generation and Editing

- **URL**: https://arxiv.org/abs/2609.08936
- **Title**: AuK Technical Report: An Open-Source Foundational Model for Speech Generation and Editing
- **Authors**: Ziyang Ma et al. (Kimi AI)

### Core Contribution
Unified speech generation+editing model via natural-language instructions + audio context. Combines multimodal LLM for semantic conditioning, jointly-trained VAE (speech/general-audio/music), and hybrid rectified-flow Transformer (MMDiT→DiT). Distilled to AuK-Flash: 4-step inference without CFG, 4.5× wall-clock speedup. 3.03B instruction-audio instances, 1.95M hours supervision.

### Absorbable Patterns
- **Multi-task instruction interface**: Single natural-language API covers 5 task families (generation, content editing, enhancement/separation, paralinguistic editing, acoustic editing)
- **Dual-stream→unified-stream generation**: MMDiT blocks (semantic+acoustic streams) followed by unified DiT blocks — progressive fusion architecture
- **Consistency initialization + task-routed Decoupled DMD**: Distillation reduces 4-step inference from full model
- **Human-feedback preference optimization + reward-based RL**: Complementary post-training for open-ended vs signal-level tasks

### NeoTrix Mapping
- NT-IO: `nt_io::consistency_adapter` / `nt_io::reference_generation` — speech generation/editing as platform capability
- NT-PHYSICAL: `nt_physical::audio_sync_library` — audio acoustics conditioning layer
- NT-ACT: `nt_act::parallel_task` — multi-task instruction routing

### Priority: P2
Reference architecture for multi-modal generation+editing unified interface. Not directly runtime-critical for NeoTrix core.

---

## 2. Optimal Rates for Agentic Networked Information Aggregation

- **URL**: https://arxiv.org/abs/2609.05318v1
- **Title**: Optimal Rates for Agentic Networked Information Aggregation
- **Authors**: Bateni, Hadizadeh, Hajiaghayi, JafariRaviz, Taherijam

### Core Contribution
Closes the gap on information aggregation in DAG-based agent networks (building on Kearns/Roth/Ryu SODA'26). Agents sit in a DAG, each sees subset of features + parent predictions, passes only predictions forward. Proves the correct error rate is Θ(M²/D) beyond depth M², matching upper and lower bounds. Also proves same optimal rate for logistic classification in logit-passing model.

### Absorbable Patterns
- **DAG information aggregation**: Theoretical framework for how distributed agents converge on full information — agents in a DAG path achieve geometric error contraction
- **M-covered path**: Every block of M consecutive agents sees all raw features — sufficient redundancy for distributed sensing
- **Error contraction proof**: For any fixed distribution, excess error contracts geometrically along the path — validates that GWT-like broadcast can asymptotically reach full-information performance

### NeoTrix Mapping
- NT-CORE: `nt_core::gwt` — theoretical foundation for GWT attention routing convergence properties
- NT-MEMORY: `nt_memory::kb` — distributed knowledge aggregation guarantees
- NT-CORE: `nt_core::e8` — multi-agent coordination topology design

### Priority: P1
Directly relevant to GWT convergence theory. Validates that agent-network information aggregation in NeoTrix's architecture has provable error bounds.

---

## 3. Trace as State: Reasoning Traces as Conditional States for Long-Context Transformers

- **URL**: https://arxiv.org/abs/2609.02702
- **Title**: Trace as State: Reasoning Traces as Conditional States for Long-Context Transformers
- **Authors**: Xu Zou, Jie Tang

### Core Contribution
Formalizes the mismatch between causal transformers and tasks requiring later-discovered state. Proposes "Trace as State": collect reasoning traces from initial pass, place them BEFORE the long-context block on a fresh pass as task-state proxy. Outperforms Trace Append (same proxy placed after context) in 26/27 model×task×metric combinations. DeepSeek V4 Pro: 29.2%→81.8% exact match on GraphWalks Parents.

### Absorbable Patterns
- **Conditional state update**: Placing derived information before the context is exponentially more efficient than placing it after — "what was discovered later guides re-reading"
- **Two-pass inference**: First pass collects traces as state; second pass with traces prepended enables causal models to access future-discovered information
- **Trace as textual state proxy**: Reasoning traces serve as compact task-state representation without architectural changes

### NeoTrix Mapping
- NT-CORE: `nt_core::e8` — self-reference in hexagram reasoning: Trace-as-State as a two-pass self-consultation pattern
- NT-MIND: `nt_mind::distiller` — trace distillation for context-efficient re-reasoning
- NT-NEXUS: `nt_nexus` — cross-session experience traces as state for long-horizon tasks

### Priority: P0
Directly applicable to NeoTrix's SEAL pipeline: collect evolution traces → re-inject as state for next cycle. Natural mapping to ConsciousnessTree's multi-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). Trace re-injection before context block could dramatically improve long-horizon reasoning in NT-CORE.

---

## 4. Procedural Graphs: Self-Evolving Execution Structures for LLM Agents

- **URL**: https://arxiv.org/abs/2609.09153
- **Title**: Procedural Graphs: Self-Evolving Execution Structures for LLM Agents
- **Authors**: Yuxing Lu, Yicheng Chen, Shanchan Wu, Sercan Ö. Arık (Google)

### Core Contribution
Procedural Graph organizes procedural knowledge as (procedure, relation, procedure) triplets — analogous to knowledge graphs but for "what-to-do" instead of "what-is." At each step, localizes active node and translates surrounding subgraph into situational guidance. Graph is self-evolving: LLM refiner contrasts failed vs successful trajectories, edits topology/attributes, keeps edits that improve held-out validation performance. Starts from minimal skeleton, builds graphs that match or surpass hand-designed ones.

### Absorbable Patterns
- **Procedural graph = knowledge graph for actions**: (procedure, relation, procedure) triplets — bridges KB knowledge representation to action orchestration
- **Self-evolving topology**: Failed trajectories → graph edits → validation-gated acceptance — structural self-improvement without human engineering
- **Subgraph-to-guidance translation**: Local subgraph context biases solver's next action without dictating it — soft guidance vs hard planning
- **Rejected edit retention**: Retaining rejected edits discourages repetition — negative knowledge accumulation

### NeoTrix Mapping
- NT-CORE: `nt_core::e8` — Procedural Graph as execution-level hexagram: procedure nodes = hexagram states, edges = transitions
- NT-MIND: `nt_mind::seal` — self-evolving topology matches SEAL pipeline's exploration→validation loop
- NT-ACT: `nt_act::orchestrator` — action orchestration via procedural graph traversal
- NT-NEXUS: `nt_nexus` — cross-session procedural knowledge persistence

### Priority: P0
Core alignment with NeoTrix's self-evolution philosophy. Procedural Graphs provide a concrete implementation pattern for: (1) E8 hexagram execution guidance, (2) SEAL pipeline's exploration→validation loop at structural level, (3) cross-session procedural memory in NT-NEXUS. Should be absorbed as a pattern into NT-MIND's skill crystallization.

---

## 5. On-Policy Reverse Distillation (OPRD): Weak-to-Strong Generalization

- **URL**: https://arxiv.org/abs/2609.08798
- **Title**: Eliciting Weak-to-Strong Generalization with On-Policy Reverse Distillation
- **Authors**: Youngrok Park et al. (Mila)

### Core Contribution
Solves weak-to-strong generalization: can stronger models learn from weaker supervisors and surpass them? OPRD evaluates teacher's policy shift relative to reference policy on student rollouts, amplifies verifier-supported policy gradient along that direction. Key insight: rescaling only verifier-supported updates preserves stationary points while accelerating learning beyond the teacher. Students remain closer to verifier-RL-only models than to weak teachers — teacher guidance accelerates rather than redirects optimization.

### Absorbable Patterns
- **Verifier-guided distillation**: Use verifier signal to filter which teacher knowledge is beneficial — prevents teacher capacity ceiling from capping student
- **On-policy reverse distillation**: Teacher evaluated on student's own rollouts (not teacher's rollouts) — prevents distribution mismatch
- **Policy shift amplification**: Scale updates along teacher-student divergence only when verifier supports it — selective knowledge transfer

### NeoTrix Mapping
- NT-MIND: `nt_mind::distiller` — distillation from weaker specialized models to stronger unified models
- NT-MIND: `nt_mind::skill_engine` — skill crystallization: distill domain expertise into core capabilities without ceiling transfer
- NT-REPAIR: `nt_repair::healer` — weaker diagnostics guiding stronger repair strategies

### Priority: P1
Relevant to NeoTrix's multi-model architecture (heterogeneous model pool). When consolidating capabilities from specialized models into unified models, OPRD's verifier-gated distillation prevents quality degradation.

---

## 6. Terminal-Universe: Turning Agent Trajectories into Scalable Terminal Environments

- **URL**: https://arxiv.org/abs/2609.04148
- **Title**: Terminal-Universe: Turning Agent Trajectories into Scalable Terminal Environments
- **Authors**: Jie Wu et al. (Alibaba)

### Core Contribution
Turns agent trajectories into reusable executable environments. Replays file operations to restore pre-modification workspace states, completion agent fills missing files/dependencies. Scales tasks along breadth (cross-workspace queries) and depth (multi-round sessions via user agent). Produced 37.3k task-sufficient environments from public trajectories. SFT on Qwen3.5-27B: +11.9 points single-round, +13.8 points multi-round on benchmarks.

### Absorbable Patterns
- **Trajectory → environment conversion**: File operation history exposes environment structure, enabling environment reconstruction from trajectories
- **Breadth×depth scaling**: Cross-workspace dependency mining + multi-round user-agent sessions — two axes for task diversity
- **Completion agent for workspace recovery**: Fill missing files/dependencies to create fully reproducible execution environments
- **Replay-based verification**: Replay recorded file operations to create verifiable task instances — execution-grounded evaluation

### NeoTrix Mapping
- NT-WORLD: `nt_world::crawler` — trajectory parsing and environment reconstruction
- NT-ACT: `nt_act::tool_registry` — tool execution history as environment specification
- NT-MEMORY: `nt_memory::kb` — trajectory storage as executable environment nodes
- NT-REPAIR: `nt_repair::healer` — self-healing: replay trajectory to diagnose failure points

### Priority: P1
Directly applicable to NeoTrix's SEAL pipeline: convert exploration trajectories into reusable test environments. Supports R-P79 (external absorption must wire to production) by providing verifiable execution contexts.

---

## 7. Assistant Bias in LLM User Simulators Using a Role Vector

- **URL**: https://arxiv.org/abs/2609.00608
- **Title**: Investigating Assistant Bias in LLM User Simulators Using a Role Vector
- **Authors**: Daeheon Jeong et al. (KAIST)

### Core Contribution
Identifies "assistant bias" in LLM user simulators — tendency to cooperate/pursue goals rather than exhibit realistic user frustration/disengagement. Extracts user role vector by contrasting user vs assistant activations on same dialogue. User direction is identifiable in activations and captures distinct characteristics from assistant traits. Steering along user direction improves simulation realism but can exaggerate behaviors and override individual profiles.

### Absorbable Patterns
- **Role vector extraction**: Contrast user vs assistant activations to find behavioral direction in representation space
- **Bias as structural**: Assistant bias is baked in during training, role-playing prompts cannot override it — must operate at activation level
- **Directional steering**: User-role activation steering improves realism but requires calibration to avoid exaggeration

### NeoTrix Mapping
- NT-FEEL: `nt_feel::emotion_engine` — role-aware emotion expression calibration
- NT-IO: `nt_io::llm_provider` — user simulation quality for testing; assistant bias awareness in multi-turn interactions
- NT-SHIELD: `nt_shield` — bias detection in agent behavior patterns

### Priority: P2
Niche but important for NeoTrix's user interaction quality. Role vector methodology could improve NT-FEEL's social emotion modeling and NT-IO's conversational quality.

---

## 8. Intentest: Structured State for Long-Horizon Automated Penetration Testing

- **URL**: https://arxiv.org/abs/2609.07344
- **Title**: Staying on the Attack Path: Structured State for Long-Horizon Automated Penetration Testing
- **Authors**: Weizhe Wang et al.

### Core Contribution
Proposes Intentest: externalizes long-horizon state from LLM context window onto persistent fact-intent DAG. Verified network states stored as immutable fact nodes, exploration directions constrained as intent edges bounded by predecessor facts. Three-layer architecture: fact-intent mapping (global state), task scheduling (degradation recovery + load balancing), intent retrieval/prediction (5-stage filtering). 88.2% overall success rate on CTF benchmark, 75% on hard tasks (+44/50 percentage points over baseline).

### Absorbable Patterns
- **Fact-intent DAG**: Externalize state from LLM context onto persistent graph — immutable facts + bounded intent edges prevent context forgetting
- **Three-layer state management**: Mapping layer (global state) → scheduling layer (execution stability) → prediction layer (tactical priors)
- **Two-phase degradation recovery**: Execution stability through graceful degradation — critical for long-horizon agent reliability
- **Intent retrieval as top-down filtering**: 5-stage filtering algorithm provides tactical priors — reduces invalid transitions by ~33-48%

### NeoTrix Mapping
- NT-SHIELD: `nt_shield::sandbox` — penetration testing agent architecture reference
- NT-CORE: `nt_core::e8` — fact-intent DAG as a structured reasoning state representation (hexagram as fact-intent node)
- NT-MEMORY: `nt_memory::kb` — persistent state externalization pattern
- NT-ACT: `nt_act::orchestrator` — long-horizon task scheduling with degradation recovery

### Priority: P1
Strong architectural pattern for long-horizon agent state management. Fact-intent DAG is a clean abstraction for externalizing state that directly applies to NT-CORE's reasoning state and NT-MEMORY's persistent knowledge.

---

## 9. NeoHorse-1: Recursive Self-Improvement via Agentic Post-Training with Routing Harness

- **URL**: https://arxiv.org/abs/2609.08183 (via HuggingFace)
- **Title**: NeoHorse-1: Towards Recursive Self-Improvement via Agentic Post-Training with Routing Harness
- **Authors**: NeoHorse Team (37 authors)

### Core Contribution
Explores recursive self-improvement (RSI) through agentic post-training. Heterogeneous model pool + intelligent routing records predicted capability demand, selected service tier, and interaction for each turn. Records converted into training examples preserving interleaved reasoning + tool calls + harness context. Structural validation + 6-dimensional semantic evaluation + subscene-level labeling gate training data. Three-stage curriculum SFT + routing-guided on-policy distillation. Evaluation→selection→update loop closes the RSI cycle. Macro-average: 58.94→64.87 (4B), 65.60→69.04 (9B).

### Absorbable Patterns
- **Routing harness for RSI**: Record (demand预测, tier选择, interaction) per turn → convert to training data → close the loop
- **6-dimensional semantic evaluation**: Quality gate for training data admission — prevents garbage-in-garbage-out in self-improvement
- **Routing-guided on-policy distillation**: Teacher supervises student under same routing progression — curriculum-aligned distillation
- **Capability-guided allocation**: Evaluation feedback → next training mixture — what the system learns to do shapes what it learns from next

### NeoTrix Mapping
- NT-MIND: `nt_mind::seal` — SEAL pipeline IS the RSI loop; NeoHorse provides concrete routing-harness implementation pattern
- NT-CORE: `nt_core::e8` — routing harness as hexagram-guided capability selection
- NT-IO: `nt_io::llm_provider` — heterogeneous model pool routing and tier selection
- NT-NEXUS: `nt_nexus` — cross-session learning feedback loop

### Priority: P0
**Core alignment with NeoTrix's SEAL pipeline.** NeoHorse-1 is essentially a concrete implementation of what SEAL aspires to be: routing signals → training data → curriculum → feedback loop. The 6-dimensional semantic evaluation and capability-guided allocation patterns should be absorbed directly into NT-MIND. This is the most directly actionable paper in this batch for NeoTrix.

---

## Summary

| # | Paper | Priority | Key Pattern | NeoTrix Domain |
|---|-------|----------|-------------|----------------|
| 1 | AuK Speech Foundation | P2 | Multi-task instruction interface | NT-IO |
| 2 | Agentic Networked Info Aggregation | P1 | DAG convergence theory for GWT | NT-CORE/GWT |
| 3 | Trace as State | P0 | Two-pass trace re-injection | NT-CORE + NT-MIND |
| 4 | Procedural Graphs | P0 | Self-evolving action knowledge graph | NT-CORE + NT-MIND |
| 5 | OPRD Weak-to-Strong | P1 | Verifier-gated distillation | NT-MIND |
| 6 | Terminal-Universe | P1 | Trajectory → executable environment | NT-WORLD + NT-ACT |
| 7 | Assistant Bias Role Vector | P2 | Activation-level role steering | NT-FEEL + NT-IO |
| 8 | Intentest Pen Testing | P1 | Fact-intent DAG state externalization | NT-SHIELD + NT-CORE |
| 9 | NeoHorse-1 RSI | P0 | Routing harness RSI loop | NT-MIND/SEAL |

### Top 3 Absorbable Patterns

1. **Routing Harness RSI Loop** (NeoHorse-1, P0) → SEAL pipeline concrete implementation: record routing decisions → convert to training data → 6D semantic gate → curriculum SFT → feedback closes loop
2. **Procedural Graphs** (P0) → Self-evolving (procedure, relation, procedure) triplets: E8 hexagram execution guidance + SEAL exploration→validation at structural level + rejected-edit retention for negative knowledge
3. **Trace as State** (P0) → Two-pass reasoning: collect traces → prepend as state → guide re-reading. Maps to ConsciousnessTree multi-stage feedback + experience-tree branch re-injection

### Failed Fetches
None — all 9 URLs successfully retrieved.
