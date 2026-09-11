# Model Reverse Engineering — Cycle 322

**Date**: 2026-09-11
**Focus**: Intrinsic attention control, state space evolution, finite-state agent coordination, multi-principal protocols, LLM-guided MARL

---

## 5 Models/Papers for Reverse Engineering

### 1. Declarative Attention: Language Models Can Control Their Own Attention
- **Paper**: https://arxiv.org/abs/2609.02737 (Sep 2026)
- **Key Innovation**: Models declare WHERE they need to attend within their chain-of-thought, partitioning generation into three modes: `<global>` (full context), `<focus>` (specific region), `<local>` (recent output only). The inference engine parses these declarations like tool calls and skips most KV cache reads. Zero-shot on off-the-shelf models (Gemma-4-31B, Qwen-3.6-27B): 52.0% / 31.1% reduction in total attended tokens during decoding, with modest accuracy drops (1.27pp / 2.75pp) that shrink with model scale.
- **Architecture Pattern**: Intrinsic attention control via chain-of-thought declarations. The model itself decides which regions are relevant at each generation step. No external proxy scorer needed — the model already knows where to look. Three-mode partitioning enables hardware to skip entire KV cache regions.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: Declarative Attention is an intrinsic GWT — the model broadcasts its own attention needs. The three modes map to GWT salience tiers: `<global>` = full broadcast, `<focus>` = domain-specific attention, `<local>` = working memory only. This validates our GWT-first architecture: attention routing should be endogenous, not exogenous
  - **KVMem (NT-MEMORY)**: The three modes directly map to our paged KV strategy: `<global>` = full KV cache, `<focus>` = selected pages, `<local>` = recent pages only. The inference engine that "parses declarations like tool calls" is a hardware-level attention router
  - **Axiom A2 (Context as Scarce)**: Declarative Attention is the ultimate context-scarce mechanism — the model tells the hardware which context matters, enabling 52% token reduction with minimal accuracy loss
  - **Dual Specialization**: The three modes map to weapon set activation: `<global>` = Weapon Set I (full analysis), `<focus>` = targeted investigation, `<local>` = quick execution

#### Fusion Opportunities
- **Action**: Investigate declarative attention patterns for NT-IO provider routing — can the model declare its own cost-tier needs?
- **Action**: Map the three-mode partitioning to our GWT salience scoring — implement salience tiers as global/focus/local broadcast modes

---

### 2. Mamba-3: Improved Sequence Modeling using State Space Principles
- **Paper**: https://arxiv.org/abs/2603.15569 (Mar 2026)
- **Key Innovation**: Three core improvements to Mamba: (1) Exponential-trapezoidal discretization for more expressive recurrence; (2) Complex-valued state update rule enabling richer state tracking (equivalent to data-dependent rotary embedding, efficiently computable); (3) Multi-input multi-output (MIMO) formulation that increases FLOP efficiency during decoding without increasing state size or wall-clock latency. Mamba-3 (MIMO) at 1.5B scale: +2.2 over Transformers, +1.9 over Mamba-2, +1.8 over Gated DeltaNet. Achieves Mamba-2 perplexity with HALF the state size (64 vs 128).
- **Architecture Pattern**: Inference-first design — every innovation targets decoding efficiency, not just training quality. Complex-valued state enables state tracking that previous linear models couldn't achieve. MIMO increases FLOPs during memory-bound state update (more computation during the bottleneck) without compromising speed. The key insight: hardware utilization matters more than theoretical complexity.
- **NeoTrix Domain Mapping**:
  - **NT-CORE (E8)**: Mamba-3's complex-valued state is analogous to our E8 hexagram reasoning — each state carries both magnitude (real) and phase (imaginary) information, enabling richer pattern matching than real-only representations
  - **NT-MEMORY**: The state size reduction (128→64 with same perplexity) validates our KB compression strategy — achieve same recall with smaller memory footprint through better state representation
  - **KVMem**: MIMO's "more computation during memory-bound update" maps to our paged KV optimization — when loading from NVMe (memory-bound), do more computation to amortize the I/O cost
  - **Axiom A1 (Cost-Aware)**: Mamba-3's inference-first design is the ultimate cost-aware architecture — every design decision optimizes for deployment cost, not just benchmark accuracy

#### Fusion Opportunities
- **Action**: Study complex-valued state representation for HyperCube knowledge encoding — phase information could encode relational semantics
- **Action**: Apply MIMO principle to our KV cache optimization — when loading cold pages, do more computation per page load to reduce total I/O

---

### 3. NeuralFSM: Adaptive Multi-Agent Coordination via Learning Finite-State Execution Policy
- **Paper**: ACL 2026 (Long Paper)
- **Code**: https://github.com/DisseverYOLO/NeuralFSM
- **Key Innovation**: Formulates multi-agent problem solving as a finite-state execution process. Learns both state transition distribution and inter-agent communication weights from interaction traces using Temporal Coordination Controller (Temporal Graph Networks). Reusable FSM backbone + learned execution policy. Dual-defense protection layer: training-time graph regularization + runtime trust-aware message attenuation. Average improvement of 6.74% over SOTA MAS baselines, 19.39% over single-agent IO. Strong robustness: only 1.82% performance drop under adversarial attack.
- **Architecture Pattern**: FSM as coordination backbone — defines the search space; learned controller selects transitions and routing within that space. Temporal Graph Networks enable task-conditioned coordination without manual protocol design. Trust-aware message attenuation provides robustness without architectural changes. Sparse routing reduces token cost.
- **NeoTrix Domain Mapping**:
  - **ConsciousnessTree**: NeuralFSM's reusable FSM backbone = our 6-stage growth loop (Soil→Roots→Trunk→Branches→Fruits→Core). The learned temporal controller = our stage transition policy (learned from execution traces, not hardcoded). Task-conditioned transitions = our task-type-dependent branch activation
  - **GWT (NT-CORE)**: Trust-aware message attenuation = GWT salience with trust weighting — messages from low-trust sources get attenuated in broadcast. The FSM state = current attention mode (which branches are active). Sparse routing = cost-aware attention allocation
  - **NT-REPAIR**: Dual-defense protection layer maps to our self-healing defense — training-time regularization prevents learning bad patterns, runtime attenuation handles adversarial inputs. The 1.82% performance drop under attack validates robustness requirements for NT-SHIELD
  - **NT-ACT**: The FSM backbone defines agent execution modes — different NT-* domains activate as FSM states based on task requirements

#### Fusion Opportunities
- **Action**: Map NeuralFSM's FSM states to our ConsciousnessTree branches — each branch activation = FSM state transition
- **Action**: Implement trust-aware message attenuation in our EventBus — messages from degraded modules get attenuated

---

### 4. MPAC: Multi-Principal Agent Coordination Protocol
- **Paper**: https://arxiv.org/abs/2604.09744 (Apr 2026)
- **Key Innovation**: Application-layer protocol for multi-principal agent coordination — when agents from DIFFERENT owners must coordinate over shared state. Five logical layers: Session, Intent, Operation, Conflict, Governance. 21 message types, 3 state machines, Lamport-clock causal ordering, optimistic concurrency control on shared state. Makes intent declaration a precondition for action. Represents conflicts as first-class structured objects (not silent side-effects). 95% reduction in coordination overhead (68.65s → 3.02s), 4.8× wall-clock speedup. Three security profiles: open, authenticated, verified.
- **Architecture Pattern**: Intent-before-action as protocol invariant — no agent can modify shared state without declaring intent first. Structured conflict representation enables machine-readable dispute resolution. Governance layer enables pluggable human-in-the-loop arbitration. Lamport-clock ordering ensures causal consistency without centralized coordination.
- **NeoTrix Domain Mapping**:
  - **NT-SHIELD**: MPAC's governance layer maps directly to our NT-GOVERNANCE domain — intent declaration as precondition = our policy enforcement. Structured conflict representation = our audit dimension system (D1-D50). The three security profiles map to our trust tiers (Trusted/Contracted/Untrusted)
  - **GWT (NT-CORE)**: Intent declaration = GWT salience pre-scoring — before an agent broadcasts, it must declare what it intends to do. The governance layer evaluates intent against session policy before allowing broadcast. This is "gated broadcast" — not all intents pass the salience filter
  - **NT-IO**: MPAC fills the multi-principal gap in our external agent interfaces — when NeoTrix agents coordinate with agents from other systems, MPAC provides the coordination semantics. The 21 message types could inform our EventBus message taxonomy
  - **Axiom A2 (Context as Scarce)**: MPAC's intent-before-action reduces unnecessary state modifications — agents only modify what they've declared intent to modify, reducing wasted computation

#### Fusion Opportunities
- **Action**: Implement MPAC's intent declaration as a precondition for NT-ACT tool calls — agents must declare intent before executing tools
- **Action**: Map MPAC's governance layer to our NT-GOVERNANCE domain — pluggable policy enforcement for multi-agent coordination

---

### 5. LLM-GNCF: LLM-Guided Graph Neural Coordination for Cooperative MARL
- **Paper**: Springer Nature, Complex & Intelligent Systems, Vol 12, Art 225 (Jun 2026)
- **Key Innovation**: Integrates LLM semantic reasoning with Graph Neural Networks for cooperative multi-agent reinforcement learning. LLM dynamically constructs Team-Adaptive Coordination Graph (TACG) based on real-time strategic semantics. TACG validated by Tactical Critic model. LLM-empowered latent reward shaping via Chain of Aggregation (CoA) mechanism — aggregates multi-frame information for fine-grained, context-aware semantic feedback while managing computational overhead. Two-stage training: LLM-guided pre-training (LLM acts as high-level commander with evaluation/correction) → RL fine-tuning (agents optimize without LLM). Competitive performance on StarCraft II micromanagement tasks.
- **Architecture Pattern**: LLM as strategic coordinator, GNN as execution substrate. The LLM provides high-level semantic guidance (what to do), the GNN handles low-level coordination (how to do it). Chain of Aggregation reduces LLM inference cost by only querying at key temporal checkpoints. Two-stage training separates guidance (LLM) from execution (RL), preventing LLM hallucination from propagating into learned policies.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: LLM as strategic coordinator = GWT salience engine. TACG as dynamic coordination graph = our ConsciousnessTree branch dependencies (dynamically constructed per task). The Tactical Critic = our ConsciousnessTree health scoring (validates branch health before full activation)
  - **NT-MIND (SEAL)**: Two-stage training (LLM guidance → RL optimization) = our SEAL pipeline stages (exploration → distillation → crystallization). The LLM provides initial guidance, then the system learns from execution traces
  - **NT-ACT**: TACG's dynamic agent grouping = our domain activation pattern — different NT-* domains form temporary coordination groups based on task requirements. The Chain of Aggregation reduces cost by batching LLM queries at checkpoints
  - **Axiom A1 (Cost-Aware)**: CoA mechanism queries LLM only at key checkpoints, not every step — this is cost-aware reasoning depth. The two-stage separation (LLM for guidance, RL for execution) optimizes cost by using expensive models only where they add value

#### Fusion Opportunities
- **Action**: Map TACG's dynamic coordination graph to our ConsciousnessTree — branches dynamically form coordination groups based on task semantics
- **Action**: Implement Chain of Aggregation pattern in our SEAL pipeline — batch LLM queries at stage boundaries, not every step

---

## Cross-Cutting Patterns

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Intrinsic attention control** | Declarative Attention | GWT endogenous salience, KVMem paged KV |
| **Inference-first design** | Mamba-3 (MIMO, complex-valued state) | Axiom A1 cost-aware architecture |
| **FSM as coordination backbone** | NeuralFSM | ConsciousnessTree 6-stage loop, GWT attention modes |
| **Intent-before-action protocol** | MPAC | NT-SHIELD governance, GWT gated broadcast |
| **LLM as strategic coordinator** | LLM-GNCF | GWT salience, SEAL two-stage training |
| **Trust-weighted communication** | NeuralFSM, MPAC | NT-SHIELD adversarial defense, EventBus trust |
| **Cost-aware reasoning depth** | LLM-GNCF (CoA), Declarative Attention | Axiom A1, Axiom A2 |
| **Dynamic coordination graphs** | LLM-GNCF (TACG), NeuralFSM | ConsciousnessTree branch activation |

---

## Priority Fusion Actions

1. **Declarative Attention → GWT** — Investigate intrinsic attention declarations as endogenous salience signals for GWT broadcast
2. **Mamba-3 Complex State → HyperCube** — Study complex-valued state representation for relational encoding in VSA HyperCube
3. **NeuralFSM → ConsciousnessTree** — Map FSM backbone to 6-stage growth loop with learned transition policies
4. **MPAC Intent Declaration → NT-ACT** — Implement intent-before-action precondition for tool calls
5. **LLM-GNCF TACG → GWT** — Dynamic coordination graph construction per task, validated by health scoring
