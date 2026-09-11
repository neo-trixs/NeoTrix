# Model Reverse Engineering — Cycle 355

**Date**: 2026-09-12  
**Focus**: Recent papers on efficient inference, attention, agent coordination  
**Sources**: arXiv, ACL 2026, ICLR 2026, ICML 2026, HuggingFace Papers

---

## 5 New Models/Papers Reverse-Engineered

### 1. RAGEN-2: Template Collapse in Agentic RL
- **Paper**: arXiv:2604.06268 (April 2026)
- **Authors**: Zihan Wang et al. (Northwestern, UIUC, Stanford, Microsoft, Oxford, Imperial)
- **Venue**: Under review (follow-up to RAGEN, arXiv:2504.20073 cited 276 times)
- **Core Contribution**: Identifies "template collapse" — a failure mode in multi-turn agent RL where models produce reasoning that appears diverse (stable entropy) but is actually input-agnostic (low Mutual Information). Introduces SNR-Aware Filtering to select high-signal prompts per iteration.

#### Key Technical Insights

**The Failure Mode Nobody Was Measuring**
- Entropy tracks diversity within the same input — high entropy means "the model says different things when prompted with the same input"
- But entropy CANNOT distinguish between "model adapts reasoning to different inputs" vs "model produces diverse but input-agnostic templates"
- Mutual Information (cross-input distinguishability) is the missing metric — it measures whether reasoning actually responds to different inputs
- Template collapse: model learns to satisfy regularization constraints (diverse, fluent) while ignoring input-specific requirements

**SNR Mechanism Explanation**
- Low reward variance → weak task gradients → regularization terms dominate → cross-input reasoning differences erased
- Signal-to-Noise Ratio in reward signals determines whether the model can distinguish "what matters" from "what's noise"
- SNR-Aware Filtering: use reward variance as a lightweight proxy to select high-signal prompts per iteration

**Results Across 8 Environments**
- Planning (Sokoban), math reasoning (MetaMathQA, Countdown), web navigation (WebShop, SearchQA), code execution
- MI correlates with final performance much more strongly than entropy
- SNR-Aware Filtering consistently improves both input dependence and task performance

#### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-META** | Template collapse detection | Monitor MI (not just entropy) in ConsciousnessTree's reasoning outputs. If MI drops while entropy stays stable, flag potential collapse. |
| **NT-MIND** | SEAL pipeline quality gates | Before crystallizing a skill, check that the reasoning pattern is input-dependent (high MI), not a template that works for any input. |
| **NT-CORE** | GWT attention quality | GWT salience routing should measure whether broadcasts are input-specific or generic templates. MI-based salience scoring. |
| **NT-SHIELD** | Security anomaly detection | Template collapse in security reasoning = dangerous. Agent produces "safe-looking" responses regardless of actual threat context. |
| **NT-MEMORY** | Experience distillation quality | When distilling experiences (experience-tree), verify that distilled knowledge is input-specific, not generic templates. |

#### Actionable Absorption
- Add MI tracking to ConsciousnessTree's health metrics (alongside entropy)
- Implement SNR-based prompt filtering in SEAL pipeline's training data selection
- Template collapse detection as a new audit dimension (D51: Reasoning Quality)

---

### 2. InftyThink+ — Infinite-Horizon Reasoning via Iterative Summarization + RL
- **Paper**: arXiv:2503.06692 (ICLR 2026) + arXiv:2602.06960 (ICML 2026)
- **Authors**: Yuchen Yan et al. (Zhejiang University, Meituan, Peking University)
- **Core Contribution**: Transforms monolithic long-context reasoning into iterative process with intermediate summarization. InftyThink+ adds RL training for adaptive summarization behavior.

#### Key Technical Insights

**Sawtooth Memory Pattern**
- Traditional: single continuous reasoning trace → O(L²) complexity → hard-bounded by context window
- InftyThink: reason → summarize → reason → summarize → ... → each segment bounded by η tokens
- Creates sawtooth memory pattern: each segment is O(η²), total is O(n·η²) where n = number of segments
- Enables unbounded reasoning depth with bounded computational cost

**Key Parameter η (Eta)**
- Controls maximum token length per reasoning iteration
- Too small: loses important context between iterations
- Too large: approaches vanilla long-context complexity
- Optimal η is task-dependent and can be learned (InftyThink+)

**InftyThink+ RL Training**
- Model learns WHEN to summarize and WHAT to preserve
- Adaptive behavior outperforms fixed η heuristics
- 25% training speedup over vanilla long-context RL (225s vs 300s per step)
- Consistent accuracy gains across MATH500, AIME24, GPQA_diamond

**The Critical Insight**
- You don't need bigger context windows if you restructure the reasoning process
- 8K-context models can perform long-context reasoning through iterative summarization
- Human working memory inspiration: we don't hold entire problems in mind, we summarize and proceed

#### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-MEMORY** | Hub-and-spoke architecture validation | InftyThink's sawtooth pattern IS experience-tree's hub index → branch → hub architecture. Formalize this as a memory management pattern. |
| **NT-MIND** | SEAL pipeline distillation | Use InftyThink-style iterative summarization in SEAL's distillation phase. Don't try to distill everything at once — summarize between stages. |
| **NT-CORE** | ConsciousnessTree cycle design | Each ConsciousnessTree growth cycle (Soil→Roots→Trunk→Branches→Fruits→Core) should include intermediate summarization, not continuous reasoning. |
| **NT-NEXUS** | Cross-session memory management | When bridging sessions (nexus-weaver), use iterative summarization to compress session history without losing critical context. |
| **NT-IO** | Context window optimization | For agents with limited context windows, implement InftyThink-style iterative reasoning instead of trying to fit everything in context. |

#### Actionable Absorption
- Formalize sawtooth memory pattern as a NeoTrix architecture primitive
- Implement adaptive summarization in experience-tree's distillation phase
- Add η parameter to ConsciousnessTree cycle configuration
- Document the "restructure process, don't scale context" insight as an axiom

---

### 3. Agent libOS — Capability-Controlled Self-Evolution Substrate
- **Paper**: arXiv:2606.03895 (June 2026)
- **Core Contribution**: Separates three planes (action/authority/evidence) to prevent self-evolution from becoming authority escalation. The core invariant: "A self-evolving agent may change what it can ask for, but it cannot thereby change what it is authorized to affect."

#### Key Technical Insights

**The Self-Evolution Security Problem**
- Systems now adapt prompts, memory, tools, workflows, and agent code within and across task episodes
- If exposing an action also grants the authority to perform it, self-evolution becomes permission escalation
- Host isolation alone doesn't solve it — can't attribute effects to responsible processes or link outcomes to capabilities

**Three-Plane Architecture**
1. **Action Plane**: What the agent can ask for (tools, skills, prompts, images, checkpoints, child processes)
2. **Authority Plane**: What the agent is authorized to affect (typed capabilities, task authority ceilings, budgets, data labels, sink registry)
3. **Evidence Plane**: Durable intent, classified outcomes, events, audit records, causal links — explains decisions but never grants authority

**Conjunctive Admission**
- Every action requires ALL of: typed Capability check + Task Authority ceiling + policy/Human approval + hierarchical budget + trusted data label + Host-owned Sink registry + exact one-shot release
- "Exact release" = authorization is one-shot, not persistent — each action needs fresh authorization
- Self-evolution can change the Action Plane (what the agent asks for) but NOT the Authority Plane (what it's authorized to do)

**Key Design Goals (7)**
1. Process identity for long-running agents
2. Capability-scoped action surfaces
3. Information flow control
4. Durable evidence without authority escalation
5. Hierarchical budgets
6. Reversible self-evolution where possible
7. Auditable self-evolution where not reversible

#### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-SHIELD** | Egress guard formalization | Agent libOS provides the formal security model for NT-SHIELD's trust tiers. "Trusted/Contracted/Untrusted" maps to authority plane levels. |
| **NT-GOVERNANCE** | Policy enforcement substrate | The three-plane architecture IS the governance substrate. Authority plane = governance rules, evidence plane = audit trail. |
| **NT-ACT** | Tool call validation | Every tool call goes through conjunctive admission: capability check + authority check + budget check + data label check. |
| **NT-META** | Evolution safety | Self-evolution is safe ONLY if authority plane is immutable. NT-META should enforce this invariant during SEAL cycles. |
| **NT-NEXUS** | Cross-session authority | One-shot release means authority doesn't persist across sessions. Cross-session memory must re-authorize capabilities. |

#### Actionable Absorption
- Formalize NT-SHIELD's trust tiers using Agent libOS's authority plane model
- Implement conjunctive admission for all tool calls (Capability + Authority + Budget + DataLabel + Sink)
- Add "exact release" pattern — tool authorization expires after single use, must be re-authorized
- Evidence plane maps to EventBus event_log (R-P84) — immutable audit trail
- The "指针守恒" rule in AGENTS.md is an informal version of Agent libOS's core invariant

---

### 4. Ring-Linear-2.0 — Hybrid Linear+Softmax Attention Architecture
- **Paper**: arXiv:2510.19338 (October 2025, models released 2026)
- **Authors**: Ling Team (Inclusion AI)
- **Models**: Ring-mini-linear-2.0 (16B params, 957M activations), Ring-flash-linear-2.0 (104B params, 6.1B activations)
- **Core Contribution**: Hybrid architecture interleaving linear attention (constant KV cache, O(n)) with softmax attention (quadratic but expressive). Optimal ratio M=7 (7 linear : 1 softmax) at high FLOP budgets.

#### Key Technical Insights

**Hybrid Attention Architecture**
- Layer groups of M+1 layers: M linear attention blocks + 1 GQA (Grouped Query Attention) block
- Linear attention uses Lightning Attention with fixed decay: O = Q(K^T V), constant KV cache
- Softmax attention provides retrieval/recall capability that linear attention lacks
- When M=0, reduces to pure softmax; when M→∞, approaches pure linear

**Scaling Law Discovery**
- Hybrid linear architecture consistently outperforms pure softmax on scaling law curves
- Larger M (more linear per softmax) performs better at higher FLOP budgets
- M=7 is optimal for production-scale models
- The key: linear attention is "good enough" for most processing, softmax is needed only for retrieval/recall

**Efficiency Results**
- 1/10th inference cost of 32B dense model
- 50% cost reduction vs original Ring series
- 50% training efficiency improvement via LingHe FP8 operator library
- Constant KV cache from linear layers + selective expressivity from softmax layers

**MoE Integration**
- Highly sparse Mixture-of-Experts architecture
- Only ~6% of parameters activated per token (957M of 16B, 6.1B of 104B)
- Combines architectural efficiency (hybrid attention) with computational efficiency (sparsity)

#### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-CORE** | GWT attention tiers | Implement tiered attention: lightweight "linear" attention for routine monitoring, full "softmax" attention for complex reasoning. M=7 ratio as default. |
| **NT-MEMORY** | KB query optimization | Most KB queries are "linear" (simple lookups), few need "softmax" (complex reasoning). Route queries to appropriate attention tier. |
| **NT-WORLD** | Perception processing | Most sensory input is routine (linear attention), only novel/salient events need full processing (softmax). |
| **NT-ACT** | Tool execution routing | Most tool calls are simple (linear), few require complex reasoning (softmax). Route accordingly. |
| **NT-MIND** | SEAL pipeline efficiency | Most SEAL phases are routine (linear processing), only distillation/crystallization need full expressivity. |

#### Actionable Absorption
- Implement attention tiers in GWT routing: routine (linear) vs complex (softmax)
- Use M=7 ratio as default: 87% of processing uses lightweight attention, 13% uses full expressivity
- Apply MoE sparsity to NeoTrix module activation — not all domains active for all tasks
- Constant KV cache insight validates KVMem's paged KV approach (Axiom A2)
- The "most processing is simple" insight validates cost-aware routing (Axiom A1)

---

### 5. CoAgent — LLM-Native Concurrency Control for Multi-Agent Systems
- **Paper**: arXiv:2606.15376 (June 2026)
- **Core Contribution**: Addresses multi-agent concurrency where classical database-style concurrency control (locks, OCC) fails. Proposes LLM-native concurrency control where the agent reasons about its own conflict history to selectively re-execute only premised actions.

#### Key Technical Insights

**Why Classical CC Fails for Agents**
- Agent transactions span SECONDS TO MINUTES of LLM inference (not milliseconds like DB transactions)
- Read sets are broad and opaque (not statically inferable from code)
- Live state admits neither fork nor buffer (writes take effect immediately)
- Classical locks block for the entire inference duration — unacceptable
- OCC (Optimistic Concurrency Control) abort-and-retry discards minutes of work on every conflict

**Four Coordination Protocols Compared**
1. **Serial**: Run agents one at a time — correctness floor, highest latency
2. **Naive parallel**: Ignore all dependencies — fastest, incorrect
3. **Classical CC** (locks/OCC): Too expensive for minute-scale transactions
4. **LLM-native CC**: Agent reasons about its own read/write history, identifies premised actions, re-executes only those

**LLM-Native Concurrency Control**
- On conflict, agent inspects its OWN read/write history
- Identifies which actions were premised on the now-stale value
- Re-executes ONLY those actions (not full rollback)
- Leverages prefix KV caching for efficiency
- The model is smart enough to reason about its own causal dependencies

**Key Insight: Agent Transactions Are Different**
- "A single agent transaction spans minutes of inference"
- "Read sets are broad and opaque rather than statically inferable"
- "Writes take effect the moment they execute"
- These three properties make classical CC cost-prohibitive

#### NeoTrix Domain Mapping

| Domain | Mapping | Implementation |
|--------|---------|----------------|
| **NT-NEXUS** | Cross-session state management | When multiple sessions access shared KB state, use LLM-native CC — let the agent reason about which cross-session dependencies are affected by changes. |
| **NT-ACT** | Parallel task coordination | When NT-ACT runs parallel tasks (BatchProductionManager/ProductionOrchestrator), use selective re-execution instead of full rollback on conflicts. |
| **NT-CORE** | GWT broadcast conflicts | When multiple GWT broadcasts conflict, use causal dependency analysis — identify which downstream effects are premised on stale salience. |
| **NT-MEMORY** | KB write conflicts | Multiple agents writing to KB simultaneously — use LLM-native CC to identify and re-execute only affected queries/indexes. |
| **NT-META** | ConsciousnessTree cycle conflicts | When multiple growth cycles overlap, use selective re-execution — only redo phases that depend on the conflicting state. |

#### Actionable Absorption
- Implement LLM-native CC for NT-NEXUS cross-session coordination
- Use selective re-execution (not full rollback) for EventBus event conflicts
- Apply causal dependency analysis to GWT broadcast conflicts
- The "agent transactions span minutes" insight should be documented as a design constraint
- Prefix KV caching optimization for agent reasoning across conflict boundaries

---

## Cross-Paper Synthesis

### Unified Insight: The Agent Systems Stack

These 5 papers collectively define the "agent systems stack" that NeoTrix needs:

```
┌─────────────────────────────────────────────────┐
│ L5: Reasoning Quality (RAGEN-2)                 │
│   Template collapse detection, MI-based metrics │
├─────────────────────────────────────────────────┤
│ L4: Reasoning Process (InftyThink+)             │
│   Iterative summarization, sawtooth memory      │
├─────────────────────────────────────────────────┤
│ L3: Security Invariant (Agent libOS)            │
│   Three-plane separation, capability control    │
├─────────────────────────────────────────────────┤
│ L2: Attention Efficiency (Ring-Linear-2.0)      │
│   Hybrid linear/softmax, tiered processing      │
├─────────────────────────────────────────────────┤
│ L1: Concurrency Control (CoAgent)               │
│   LLM-native CC, selective re-execution         │
└─────────────────────────────────────────────────┘
```

### Three Axioms Derived

1. **Process > Scale**: InftyThink + Ring-Linear-2.0 both show that clever process design beats raw scaling. Restructure reasoning, don't just expand context windows.

2. **Evolve > Static, but Constrain > Evolve**: RAGEN-2 + Agent libOS both address self-evolution. Evolution is necessary (RAGEN shows static agents degrade), but must be constrained (Agent libOS shows unconstrained evolution is a security hole).

3. **Agents Are Different from Databases**: CoAgent proves classical systems techniques (CC, transactions) don't transfer directly to LLM agents. Agent-specific solutions (LLM-native CC, capability systems) are needed.

### NeoTrix Absorption Roadmap

| Phase | Paper | Pattern | Target | Complexity |
|-------|-------|---------|--------|------------|
| Phase 1 | Agent libOS | Three-plane security | NT-SHIELD | High |
| Phase 1 | RAGEN-2 | MI-based quality tracking | NT-META | Medium |
| Phase 2 | InftyThink+ | Sawtooth memory | NT-MEMORY | Medium |
| Phase 2 | CoAgent | LLM-native CC | NT-NEXUS | High |
| Phase 3 | Ring-Linear-2.0 | Attention tiers | NT-CORE | Low |
