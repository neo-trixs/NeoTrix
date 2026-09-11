# Model Reverse Engineering — Cycle 323

**Date**: 2026-09-11
**Focus**: Layer sparsity for efficient inference, latent action spaces, amortized multi-agent topology, attention steering, multi-role single-agent synthesis

---

## 5 Models/Papers for Reverse Engineering

### 1. Don't Drop Dropout: Optimizing Layer Sparsity for Efficient LLM Training and Inference
- **Paper**: https://arxiv.org/abs/2609.05275 (ICML 2026)
- **Key Innovation**: Layer dropout should be used in SOTA LLM training — not dropped. Optimal layer distribution, time schedule, and optimizer hyperparameters enable: (1) same training FLOPs with lower loss, (2) up to 25% training FLOP savings, (3) significant post-training optimizations — early exit, intermediate-layer skipping, self-speculative decoding yielding up to 1.5x inference speedup with negligible accuracy loss. Establishes best practices for layer sparsity scaling.
- **Architecture Pattern**: Layer dropout as dual-purpose technique — reduces training cost AND enables inference acceleration. The insight: training with dropout teaches the model to function with missing layers, which then enables self-speculative decoding at inference time. One technique serves two optimization targets.
- **NeoTrix Domain Mapping**:
  - **Axiom A1 (Cost-Aware)**: 25% training FLOP savings + 1.5x inference speedup = double cost reduction. The layer sparsity pattern should inform our model selection — prefer models trained with dropout that support self-speculative decoding
  - **KVMem (NT-MEMORY)**: Self-speculative decoding with layer skipping reduces KV cache pressure. Layers that can be skipped during draft phases reduce memory bandwidth. The "learned sparsity" pattern validates our approach of training-aware optimization
  - **NT-IO (Provider Routing)**: Models with layer sparsity support faster inference — provider selection should weight inference efficiency. Early exit capability = variable-depth inference for cost scaling
  - **Dual Specialization**: Full model verification (Weapon Set I) + sparse draft layers (Weapon Set II) = the self-speculative pattern at the hardware level

#### Fusion Opportunities
- **Action**: Weight provider selection by self-speculative decoding capability — models with dropout-trained sparsity get preference for cost-sensitive routing
- **Action**: Study layer sparsity profiles to determine optimal early-exit points for different task types in our SEAL pipeline

---

### 2. Latent Action Reparameterization (LAR) for Efficient Agent Inference
- **Paper**: https://arxiv.org/abs/2605.18597 (May 2026)
- **Key Innovation**: Learns a compact latent action space where each latent action corresponds to a multi-step semantic behavior. By reparameterizing agent actions into latent units, LAR reduces the effective decision horizon while preserving expressiveness. Unlike hand-crafted macros or hierarchical controllers, latent actions are learned from trajectories and integrated directly into the model. Both planning and execution operate over abstract action representations. Substantial reductions in action tokens and wall-clock time while maintaining task success rates.
- **Architecture Pattern**: Action representation learning as a bottleneck breaker. The key insight: the action space itself is a critical factor for inference efficiency, not just model architecture or hardware. Latent actions compress the decision horizon — what takes 10 low-level text actions becomes 1 latent action. Learned from data, not hand-crafted.
- **NeoTrix Domain Mapping**:
  - **NT-ACT (Tool Execution)**: LAR directly maps to our PTC (Programmatic Tool Calling) approach — instead of stringing together individual tool calls, compound actions are latent units. The "10 text actions → 1 latent action" compression is exactly what we need for NT-ACT efficiency
  - **GWT (NT-CORE)**: Latent actions as GWT attention primitives — each latent action = a broadcast of coordinated tool calls. Planning over latent space = GWT salience planning over abstract capabilities, not individual tools
  - **NT-MIND (SEAL)**: LAR-learned latent actions = crystallized skill nodes. The trajectory-to-latent-action learning = our experience-tree distillation (execution traces → skill nodes). The "integrated directly into the model" pattern validates our approach of baking skills into the agent
  - **Axiom A2 (Context as Scarce)**: Reduced action horizon = fewer tokens for action sequences = more context budget for reasoning. LAR is a context-efficiency technique at the action representation level

#### Fusion Opportunities
- **Action**: Study LAR's trajectory-to-latent-action learning for our SEAL pipeline skill crystallization — learn compound actions from execution traces rather than hand-crafting skill macros
- **Action**: Map latent action space to our skill node graph — each latent action = a skill node with multi-step behavior

---

### 3. Codebook Agent: Amortized Topology Design for LLM Multi-Agent Systems
- **Paper**: https://arxiv.org/abs/2609.02264 (Sep 2026)
- **Key Innovation**: Three key empirical findings challenge current multi-agent topology design: (1) Topologies collapse to ~6 distinct graphs even with 64-entry codebooks — the topology space is much smaller than assumed; (2) Edge count is NEGATIVELY correlated with token consumption (Pearson r ≈ -0.4) — sparsifying graphs makes inference MORE expensive, not less; (3) Message-passing scorers are adjacency-invariant when agents share profiles — they can't rank candidates in the default configuration. Solution: VQ-autoencoder compresses successful topologies into a 16-entry codebook; reward-weighted MLP maps query to code distribution; MLP proxy reranks candidates in single batched forward pass. No iterative search, no message passing at test time. 84.6 avg accuracy (vs 83.0 SOTA), 2.4ms topology emission, 21.9-33.2% fewer LLM tokens.
- **Architecture Pattern**: Amortized topology design replaces per-query search. The codebook approach learns a small set of effective topologies offline, then selects at test time via a lightweight router. The counter-intuitive finding (fewer edges = more tokens) suggests coordination overhead matters more than communication sparsity.
- **NeoTrix Domain Mapping**:
  - **ConsciousnessTree**: Codebook Agent's topology codebook = our ConsciousnessTree branch activation patterns. Instead of computing branch activation per-cycle, learn a small codebook of effective activation patterns and select via lightweight router. The ~6 collapse finding validates that we don't need infinite branch combinations — a small set covers most tasks
  - **GWT (NT-CORE)**: The reward-weighted MLP mapping query→topology = GWT salience scoring. The finding that sparsifying increases cost maps to GWT — overly aggressive attention sparsification wastes tokens on coordination overhead. The "no message passing at test time" pattern = GWT broadcast should be direct, not relayed through intermediaries
  - **NT-ACT**: The topology codebook = our multi-agent coordination templates. Pre-learned coordination patterns selected by query type. The 21.9-33.2% token savings directly reduces NT-ACT execution cost
  - **Axiom A1 (Cost-Aware)**: Codebook Agent's 2.4ms topology emission is near-zero overhead — the cost of topology selection is negligible compared to the 21-33% token savings

#### Fusion Opportunities
- **Action**: Build a ConsciousnessTree topology codebook — learn 6-16 effective branch activation patterns from KB execution traces, select via lightweight router instead of computing per-cycle
- **Action**: Study the sparsity paradox (fewer edges = more tokens) to validate our GWT broadcast strategy — direct broadcast may be cheaper than multi-hop coordination

---

### 4. Agent-Radar: Attention Steering with Context Relevance for Multi-Agent Communication
- **Paper**: https://arxiv.org/abs/2605.30136 (May 2026)
- **Key Innovation**: Training-free context management for multi-agent systems. Dynamically steers each agent's attention toward relevant context with temporal and spatial decay mechanism. As conversations lengthen, relevant information is diluted by irrelevant context — Agent-Radar combats this with decay-weighted relevance scoring. Gains of up to 7.64 absolute points across 5 benchmarks. Robust as number of agents and interaction rounds increases. Core components are generalizable across settings.
- **Architecture Pattern**: Attention steering via relevance-weighted decay. Two decay mechanisms: temporal (recent context weighted higher) + spatial (agent-proximate context weighted higher). Training-free — works with off-the-shelf models. Scalable — maintains effectiveness as system grows.
- **NeoTrix Domain Mapping**:
  - **GWT (NT-CORE)**: Agent-Radar IS a GWT implementation for multi-agent attention. Temporal decay = attention recency bias. Spatial decay = attention proximity bias (modules closer in architecture graph get higher salience). The training-free nature validates our approach of using attention mechanisms without retraining
  - **ConsciousnessTree**: Agent-Radar's decay mechanism = our cycle aging — newer cycles weighted higher, distant cycles decayed. The "robust as agents increase" finding validates our branch scaling approach
  - **NT-MEMORY (KB)**: Temporal decay = KB experience freshness weighting. Spatial decay = KB namespace proximity weighting (experiences in same domain namespace weighted higher). The 7.64 point improvement shows that relevance-weighted retrieval significantly outperforms flat retrieval
  - **Axiom A2 (Context as Scarce)**: Agent-Radar's core insight — irrelevant context dilutes performance — directly validates our context compression strategies. The decay mechanism keeps context relevant, not just compact

#### Fusion Opportunities
- **Action**: Implement temporal + spatial decay in KB experience retrieval — weight experiences by recency AND domain proximity. This is a retrieval-time attention mechanism for our knowledge base
- **Action**: Apply Agent-Radar's decay model to our ConsciousnessTree branch health scoring — recent health signals weighted higher, distant signals decayed

---

### 5. Mixture of Roles (MoRe): Multi-Agent Synergy in a Single Agent via Steering Vectors
- **Paper**: https://arxiv.org/abs/2608.27338 (Aug 2026)
- **Key Innovation**: Adaptively composes multiple specializations into a single steering vector for single-turn inference. Learns a diversified codebook of steering vectors, each encoding a latent role. Query-aware router dynamically fuses the codebook into a composite steering vector encompassing multiple roles. By steering the backbone LLM with the composed vector, enables multi-perspective specialization in single-agent, single-turn inference. Three-stage SFT curriculum + GRPO post-training while backbone remains frozen. Outperforms single-agent baselines by 2.2% avg, achieves MAS-level performance with 20x token cost reduction.
- **Architecture Pattern**: Codebook of role-specific steering vectors + query-aware composition. The frozen backbone + learned steering approach means multiple specializations coexist in a single model. Single-turn inference eliminates multi-turn coordination overhead. The 20x token reduction is dramatic — multi-agent quality without multi-agent cost.
- **NeoTrix Domain Mapping**:
  - **Dual Specialization**: MoRe is the theoretical foundation for our Dual Specialization pattern. Instead of switching between Weapon Set I and Weapon Set II, MoRe COMPOSES both into a single steering vector. This suggests our dual specialization could be implemented as role codebook + query-aware routing, not binary switching
  - **GWT (NT-CORE)**: MoRe's query-aware router = GWT salience scoring. The composition of multiple roles = broadcasting to multiple NT-* domains simultaneously. The single-turn inference = one GWT cycle producing a composite response
  - **NT-MIND (SEAL)**: The diversified codebook = our skill node library. Each steering vector = a skill node encoding a role specialization. Query-aware composition = task-dependent skill combination. The frozen backbone + learned steering = our approach of training minimal new code while preserving existing knowledge
  - **Axiom A1 (Cost-Aware)**: 20x token reduction while maintaining quality is the ultimate cost optimization. Single-turn inference eliminates the multi-turn overhead that plagues multi-agent systems

#### Fusion Opportunities
- **Action**: Study MoRe's steering vector composition for our Dual Specialization — instead of binary weapon set switching, learn a codebook of role-specific attention patterns and compose them per-query
- **Action**: Map MoRe's three-stage SFT curriculum to our SEAL pipeline — stages could train role-specific steering vectors, not just skill nodes

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Amortized Design Replaces Per-Query Search (Codebook Agent, MoRe)
Both papers replace expensive per-query optimization with learned codebooks. Codebook Agent: topology codebook replaces per-query topology search. MoRe: role codebook replaces per-query role assignment. Both achieve better quality at fraction of the cost.

**NeoTrix Mapping**: Our ConsciousnessTree should learn a codebook of effective branch activation patterns instead of computing activation per-cycle. Our Dual Specialization should use a role codebook instead of binary switching. The key insight: the space of effective configurations is much smaller than the theoretical space — learn the effective subset.

### Pattern 2: Action/Role Representation as Bottleneck (LAR, MoRe)
Both papers identify representation quality as the key to efficiency. LAR: compact latent action space compresses decision horizon. MoRe: diversified role codebook compresses multi-agent coordination. Both learn representations from data, not hand-craft them.

**NeoTrix Mapping**: Our skill nodes should be learned from execution trajectories (LAR pattern), not hand-designed. Our domain specialization should be represented as steering vectors (MoRe pattern), not hard-coded configurations. Representation learning is the highest-leverage optimization.

### Pattern 3: Sparsity Paradox in Coordination (Codebook Agent, Don't Drop Dropout)
Codebook Agent: fewer edges = more tokens (negative correlation). Don't Drop Dropout: layer sparsity enables efficiency through learned redundancy. Both show that naively reducing compute/communication often backfires.

**NeoTrix Mapping**: GWT attention sparsification must be intelligent, not naive. Reducing broadcast scope may increase coordination overhead. The correct approach: learn which sparsity patterns work (codebook), not apply uniform sparsity.

### Pattern 4: Temporal-Spatial Attention Decay (Agent-Radar, Don't Drop Dropout)
Agent-Radar: temporal + spatial decay for relevance scoring. Don't Drop Dropout: temporal scheduling of layer dropout. Both use time-dependent mechanisms to allocate attention/compute.

**NeoTrix Mapping**: Our ConsciousnessTree should use temporal decay for cycle relevance and spatial decay for branch proximity. Our experience-tree should weight recent experiences higher and same-domain experiences higher. Decay is not just memory management — it's attention allocation.

### Pattern 5: Frozen Backbone + Learned Steering (MoRe, LAR)
MoRe: frozen LLM + learned steering vectors. LAR: frozen model + learned latent action space. Both preserve existing knowledge while adding new capabilities through lightweight learned components.

**NeoTrix Mapping**: Our SEAL pipeline should follow this pattern — freeze the existing knowledge base and learn lightweight steering/routing components on top. This is the most efficient evolution strategy: don't modify what works, add what's needed.

---

## Implementation Candidates

| Priority | Paper | Action | NeoTrix Component |
|----------|-------|--------|-------------------|
| P0 | Codebook Agent | Build ConsciousnessTree topology codebook (6-16 patterns) | nt_core (ConsciousnessTree) |
| P0 | Agent-Radar | Implement temporal+spatial decay in KB experience retrieval | nt_memory (KB retrieval) |
| P1 | MoRe | Study role codebook for Dual Specialization (compose, don't switch) | nt_core_self (AttentionManager) |
| P1 | Don't Drop Dropout | Weight provider selection by self-speculative decoding capability | nt_io (provider routing) |
| P2 | LAR | Learn compound actions from execution traces for skill crystallization | nt_mind (SEAL pipeline) |
| P2 | Codebook Agent | Study sparsity paradox to validate GWT broadcast strategy | nt_core (GWT) |
