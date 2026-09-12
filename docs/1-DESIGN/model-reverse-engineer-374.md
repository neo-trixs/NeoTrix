# Model Reverse Engineering — Cycle 374

**Date**: 2026-09-12  
**Focus**: Recent papers on efficient inference, attention mechanisms, agent coordination  
**Sources**: arXiv, ACL Anthology, AAAI-26, EuroSys '26, ICLR 2026

---

## Paper 1: Flux Attention — Context-Aware Hybrid Attention

**Paper**: "Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference"  
**arXiv**: 2604.07394 (Apr 2026)  
**Authors**: Quantong Qiu et al.

### Core Innovation
Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA). A lightweight Layer Router is inserted into frozen pretrained LLMs, adaptively routing each layer to FA or SA based on input context.

### Key Mechanisms
1. **Layer-wise Routing**: Each transformer layer independently decides FA vs SA based on input context
2. **Lightweight Router**: Only 12 hours training on 8×A800 GPUs (parameter-efficient)
3. **Contiguous Memory Access**: Preserves hardware-friendly memory patterns
4. **Prefill + Decode Speedup**: 2.8× prefill, 2.0× decode

### Why It Matters
- Solves the static allocation problem (existing hybrid methods use fixed ratios)
- Layer-level granularity avoids head-level load imbalance
- Practical speedups, not just theoretical FLOP reduction

### NeoTrix Mapping
- **GWT**: Layer Router ≈ GWT salience filter. Each layer's attention mechanism is selected based on "how important is full context for this layer's computation?"
- **NT-CORE (E8)**: Hexagram state could encode layer routing decisions — FA/SA as yijing lines
- **Axiom A1 (Cost-Aware Routing)**: Directly applicable — route layers to cheap (SA) or expensive (FA) based on context demands

### Absorption Target
Implement Layer Router as GWT attention modulation component. Each domain module's "attention depth" could be dynamically adjusted based on task salience.

---

## Paper 2: Meta-Attention — Bayesian Per-Token Routing

**Paper**: "Meta-Attention: Bayesian Per-Token Routing for Efficient Transformer Inference"  
**arXiv**: 2605.28384 (May 2026)  
**Authors**: Alan Ferrari (Knowledge Lab AG)

### Core Innovation
Bayesian Meta-Controller that treats per-token attention mechanism selection as posterior inference under a compute-aware Dirichlet prior. Replaces ad-hoc regularization with principled ELBO objective.

### Key Mechanisms
1. **Dirichlet Prior**: Encodes compute cost preference at initialization
2. **Amortized Variational Posterior**: q(α|x_t; φ) trained with ELBO
3. **Soft-to-Hard Routing Transition**: Calibrated uncertainty governs routing sharpness
4. **2.4× Projected Efficiency**: Under hard routing while maintaining lower routing entropy (43.3% vs 55.8%)

### Why It Matters
- Principled alternative to deterministic soft-routing and prior-free learned-routing
- Prevents routing collapse to expensive full attention
- Connects to NeurIPS 2025 Best Paper (Gated Attention)

### NeoTrix Mapping
- **GWT**: Bayesian routing ≈ GWT resonance-based attention. The Dirichlet prior encodes "which attention mechanism is most cost-effective for this token's context?"
- **VSA HyperCube**: Posterior inference over attention mechanisms = symbolic routing over dimension space
- **NT-MIND**: ELBO objective could guide SEAL pipeline stage selection

### Absorption Target
Bayesian routing framework for GWT. Instead of hard salience thresholds, use variational inference to select attention depth per domain module per cycle.

---

## Paper 3: OrgAgent — Hierarchical Multi-Agent Organization

**Paper**: "OrgAgent: Organize Your Multi-Agent System like a Company"  
**arXiv**: 2604.01020 (Apr 2026)  
**Authors**: Yiru Wang et al. (Google DeepMind, Tsinghua)

### Core Innovation
Company-style hierarchical multi-agent framework with three layers: Governance (planning/resource allocation), Execution (task solving/review), Compliance (final answer control).

### Key Mechanisms
1. **Three-Layer Hierarchy**: Governance → Execution → Compliance
2. **Controlled Information Flow**: Layer-specific visibility
3. **Stable Skill Assignment**: Agents specialized per layer
4. **102.73% Performance Improvement**: Over flat multi-agent on SQuAD 2.0
5. **74.52% Token Reduction**: Hierarchical vs flat coordination

### Why It Matters
- Organizational structure shapes effectiveness, cost, AND coordination behavior
- Hierarchy helps most when tasks benefit from stable skill assignment + controlled info flow + layered verification
- Directly applicable to multi-domain systems like NeoTrix

### NeoTrix Mapping
- **ConsciousnessTree**: The 11 branches already form a natural hierarchy. OrgAgent validates this architecture.
- **6-Layer Architecture**: L6 Meta-Cognition = Governance, L5 Cognition = Execution, L1-L4 = Compliance (verification layers)
- **NT-GOVERNANCE**: Direct mapping — governance layer for policy enforcement
- **Dual Specialization**: Weapon Set routing ≈ skill assignment per layer

### Absorption Target
Formalize NeoTrix's implicit hierarchy using OrgAgent's three-layer model. Explicit governance/execution/compliance separation for cross-domain coordination.

---

## Paper 4: MPAC — Multi-Principal Agent Coordination Protocol

**Paper**: "MPAC: A Multi-Principal Agent Coordination Protocol for Interoperable Multi-Agent Collaboration"  
**arXiv**: 2604.09744 (Apr 2026)  
**Authors**: Kaiyang Qian et al.

### Core Innovation
Application-layer protocol for coordination when independent principals' agents must collaborate over shared state. Fills gap between MCP (tool invocation) and A2A (single-principal delegation).

### Key Mechanisms
1. **Five-Layer Coordination**: Session, Intent, Operation, Conflict, Governance
2. **21 Message Types**: Structured coordination semantics
3. **Three State Machines**: With normative transition tables
4. **Lamport-Clock Causal Watermarking**: Consistency without central authority
5. **Optimistic Concurrency Control**: On shared state
6. **95% Coordination Overhead Reduction**: 4.8× wall-clock speedup vs serialized human-mediated baseline

### Why It Matters
- MCP assumes single principal — breaks when engineers' coding agents edit same repo
- MPAC makes intent declaration a precondition for action
- Conflicts as first-class structured objects (not ad-hoc chat)
- Human-in-the-loop arbitration through pluggable governance

### NeoTrix Mapping
- **NT-ACT**: Multi-domain coordination when NT-WORLD, NT-MEMORY, NT-SHIELD agents need shared state
- **NT-GOVERNANCE**: Pluggable governance layer for arbitration
- **EventBus**: MPAC's conflict resolution ≈ EventBus conflict detection + resolution
- **KB**: Shared state management across domains

### Absorption Target
MPAC protocol for NeoTrix cross-domain coordination. Intent declaration before action, structured conflict objects, pluggable governance per domain pair.

---

## Paper 5: AgensFlow — Coordination-Policy Substrate

**Paper**: "AgensFlow: A Coordination-Policy Substrate for Multi-Agent Systems"  
**arXiv**: 2605.27466 (May 2026)

### Core Innovation
Treats multi-agent coordination as an online policy-learning problem under partial observability. Learns which reasoning constraint, model binding, and coordination topology to select per task class.

### Key Mechanisms
1. **Partially Observable MDP**: System doesn't observe user intent, latent difficulty, evidence quality directly
2. **Online Policy Learning**: Over skill, model-role bindings, topology choices
3. **Inspectable Routing**: Auditable coordination decisions
4. **RULER-Style Evaluation**: Relative evaluation of agent outputs
5. **Static Pipelines → Learned Policies**: Improvement over hardcoded coordination

### Why It Matters
- Coordination choices interact with task regime and operational constraints
- Static pipelines provide only limited design space view
- Learned, auditable routing improves coordination-heavy workflows
- "Which reasoning constraint should be selected" ≠ "more explicit reasoning is always correct"

### NeoTrix Mapping
- **GWT**: AgensFlow's policy learning ≈ GWT salience learning. Both learn routing decisions under partial observability.
- **SEAL Pipeline**: Policy over SEAL stages (exploration → distillation → self-test → absorption)
- **ConsciousnessTree**: Learned coordination between branches, not hardcoded
- **NT-MIND**: Self-evolution as policy optimization over skill selection

### Absorption Target
AgensFlow-style policy learning for GWT. Instead of hardcoded salience thresholds, learn routing policies under partial observability. Inspectable routing decisions for debugging.

---

## Cross-Paper Synthesis

### Pattern: Dynamic Attention Allocation
All five papers address the same fundamental problem: **not all computation needs the same attention depth**.
- Flux Attention: layer-level FA/SA routing
- Meta-Attention: per-token Bayesian routing
- OrgAgent: layer-level governance/execution/compliance
- MPAC: intent-level coordination routing
- AgensFlow: task-class-level policy routing

### NeoTrix Integration Priority

| Priority | Paper | Integration Target | Effort |
|----------|-------|-------------------|--------|
| P0 | Flux Attention | GWT attention modulation | Low (Layer Router pattern) |
| P0 | OrgAgent | ConsciousnessTree hierarchy formalization | Low (architecture validation) |
| P1 | Meta-Attention | Bayesian GWT routing | Medium (variational inference) |
| P1 | MPAC | Cross-domain coordination protocol | Medium (protocol design) |
| P2 | AgensFlow | Learned GWT policies | High (online learning) |

### Shared Insight
The field is converging on **hierarchical, dynamic, inspectable routing** as the answer to scaling multi-agent systems. Static pipelines and hardcoded coordination are giving way to learned policies with auditability. This validates NeoTrix's GWT + ConsciousnessTree architecture and suggests the next evolution: making routing itself learnable and inspectable.

---

## Sources

- arXiv:2604.07394 — Flux Attention (Apr 2026)
- arXiv:2605.28384 — Meta-Attention (May 2026)
- arXiv:2604.01020 — OrgAgent (Apr 2026)
- arXiv:2604.09744 — MPAC (Apr 2026)
- arXiv:2605.27466 — AgensFlow (May 2026)
- ACL 2026: SILO-BENCH (distributed coordination evaluation)
- AAAI-26: Adaptive Theory of Mind for multi-agent coordination
- EuroSys '26: SAS (Sparse Attention Synthesizer)
