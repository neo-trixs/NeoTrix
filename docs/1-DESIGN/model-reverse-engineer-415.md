# Model Reverse Engineering — Cycle 415

> Date: 2026-09-12
> Focus: Efficient inference, attention mechanisms, agent coordination

## 5 New Papers/Models

---

### 1. FFD: Faster Than Flash Decoding
**Paper**: arXiv:2609.00097 (ICML 2026)
**Authors**: Multiple (hardware-algorithm co-design)

**Core Innovation**: Fuses selector and computer into a single kernel. Replaces external metadata indices with content-aware scanning via low-bit quantization. Top-delta strategy dynamically filters blocks for distribution-adaptive sparsity without global synchronization.

**Key Results**: 11.6x kernel-level speedup, scales to 256K context, 2.37x end-to-end throughput improvement. Training-free, plug-and-play.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Top-delta dynamic filtering → GWT salience could use similar distribution-adaptive attention routing. Instead of fixed salience thresholds, use content-aware block filtering.
- **NT-PHYSICAL**: Hardware-algorithm co-design pattern → NT-PHYSICAL's sensor/motor pipelines should co-optimize with hardware constraints.
- **Pattern**: Fuse selector+compute into single kernel. Eliminate metadata index overhead.

---

### 2. DeAR: Decentralized Agentic Reasoning
**Paper**: arXiv:2608.17282
**Authors**: Multiple (peer-to-peer agent collaboration)

**Core Innovation**: Replaces centralized orchestrator with autonomous peer-to-peer collaboration. Three mechanisms: (1) Decentralized Capability Grounding — agents self-identify from model cards, not rigid roles. (2) Thought Map Navigation — dynamic collaboration propensity matrix. (3) Topology Update — progressive backtracking on dead ends.

**Key Results**: Outperforms AutoGen, AgentVerse, DyLAN across 9 benchmarks. No centralized routing bottleneck.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Collaboration Propensity Matrix → GWT's salience weighting could be decentralized. Each domain module computes its own salience contribution rather than central GWT computing all.
- **NT-MIND**: Thought Map Navigation → SEAL pipeline could use progressive backtracking instead of fixed stage progression.
- **Pattern**: Decentralized capability grounding (agents declare what they can do, not what they're assigned). Dynamic topology updates on failure.

---

### 3. AgensFlow: Coordination-Policy Substrate
**Paper**: arXiv:2605.27466
**Authors**: Nicole Koenigstein

**Core Innovation**: Treats multi-agent coordination as online policy-learning under partial observability. skip:X makes skill omission a first-class topology decision. Warm-started policy graphs reduce exploration cost. Cross-judge reward auditability.

**Key Results**: Learned routing beats fixed pipeline on coordination-heavy tasks. Warm-start reduces ~21% token usage. skip:X isolates topology compression as meaningful.

**NeoTrix Mapping**:
- **NT-ACT**: skip:X → NT-ACT's tool routing should learn which tools to SKIP, not just which to invoke. First-class omission.
- **NT-MIND**: Warm-start policy graphs → SEAL pipeline should transfer learned coordination priors across domains.
- **Pattern**: Coordination as learnable policy, not static wiring. Omission as first-class action. Cross-judge auditability.

---

### 4. Declarative Attention (DA)
**Paper**: arXiv:2609.02737
**Authors**: Multiple (intrinsic attention control)

**Core Innovation**: Model declares WHERE it needs to attend via chain-of-thought. Three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output). Engine parses declarations like tool calls and skips KV cache reads.

**Key Results**: 52% reduction in attended tokens (Gemma-4-31B), 31% (Qwen-3.6-27B). Modest accuracy drops that shrink with model scale. Zero-shot, no training required.

**NeoTrix Mapping**:
- **NT-CORE (GWT)**: Declarative attention → GWT broadcast could include attention declarations. Modules declare what context they need, GWT selectively routes only relevant KV pairs.
- **NT-MEMORY**: KV cache skip → NT-MEMORY's KB queries could use similar declarative filtering. Modules declare query scope, memory system skips irrelevant indexes.
- **Pattern**: Model self-reports attention needs. Engine optimizes based on declarations. Three-tier attention: global/focus/local.

---

### 5. CoSkill: Hierarchical Skill Evolution
**Paper**: arX Reasoning + Meta-Skill Agents for Hierarchical Skill Evolution
**Authors**: Multiple (joint RL framework)

**Core Innovation**: Recasts static meta-skill workflow as learnable Meta-Skill Agent. Jointly trains Reasoning Agent + Meta-Skill Agent over hierarchical skill library. End-to-end co-adaptation: reasoning guides skill refinement, skills guide reasoning actions.

**Key Results**: 98.4% success on ALFWorld, 90.6% on WebShop (+3.5pp and +6.2pp over baselines). Superior early-stage sample efficiency and asymptotic performance.

**NeoTrix Mapping**:
- **NT-MIND**: Meta-Skill Agent → SEAL pipeline's skill crystallization could be modeled as a learnable agent that evolves skills alongside reasoning.
- **NT-ACT**: Hierarchical skill library → NT-ACT's tool registry should support hierarchical skill composition (task skills → step skills).
- **Pattern**: Skills as active agents, not passive objects. Co-evolution of reasoning and skill refinement. Hierarchical skill libraries.

---

## Cross-Paper Synthesis

### Convergence Pattern 1: Decentralization
DeAR, AgensFlow, and AgentFugue all push toward decentralized coordination. The centralized orchestrator is becoming a bottleneck. **Action for NeoTrix**: Evaluate whether NT-CORE's GWT can operate in decentralized mode where domains self-coordinate.

### Convergence Pattern 2: Attention as First-Class Resource
FFD, Declarative Attention, and CSAttention (from earlier search) all treat attention computation as a稀缺 resource to be allocated. **Action for NeoTrix**: GWT salience should incorporate attention cost as a weighting factor (aligns with Axiom A1: Cost-Aware Routing).

### Convergence Pattern 3: Skills as Evolving Agents
CoSkill and AgensFlow both treat skills as active, learnable entities rather than static configs. **Action for NeoTrix**: NT-MIND's skill crystallization should model skills as agents that co-evolve with the reasoning system.

### Convergence Pattern 4: Omission as Action
AgensFlow's skip:X and DeAR's progressive backtracking both treat "not doing something" as a deliberate, learnable decision. **Action for NeoTrix**: NT-ACT's tool routing should include explicit skip/omit actions in its action space.

## Priority Absorption Targets

| Priority | Paper | Pattern | NeoTrix Integration Point |
|----------|-------|---------|---------------------------|
| P0 | DeAR | Decentralized capability grounding | GWT salience decentralization |
| P0 | Declarative Attention | Self-reported attention needs | GWT broadcast optimization |
| P1 | CoSkill | Skills as co-evolving agents | SEAL skill crystallization |
| P1 | AgensFlow | skip:X omission as action | NT-ACT tool routing |
| P2 | FFD | Fuse selector+compute kernel | NT-PHYSICAL hardware co-design |
