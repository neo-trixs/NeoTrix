# Model Reverse Engineering — Cycle 419

**Date:** 2026-09-12
**Focus:** Efficient inference, attention mechanisms, agent coordination, memory compression
**Sources:** arXiv, ICML'26, ACL'26, ICLR'26

---

## 5 Selected Papers

### 1. AGAO: Adaptive Goal-aware Attention Orchestration
- **Paper:** [arxiv.org/abs/2607.23678](https://arxiv.org/abs/2607.23678)
- **Venue:** arXiv 2026.07
- **Core Idea:** Extends attention from token-level to workflow-level agent coordination. Three complementary mechanisms:
  1. **Goal-aware Attention** — semantic relevance between user objectives and agent capabilities
  2. **Topology-aware Attention** — graph structural dependencies in agent execution
  3. **Resource-aware Attention** — adaptive computational budget allocation across agents

#### Pattern Analysis
| Dimension | AGAO Approach | Implication |
|-----------|--------------|-------------|
| Attention Scope | Token-level → Workflow-level | Attention is a universal coordination mechanism |
| Routing | Static graph → Dynamic per-query | Agent activation depends on goal, not topology |
| Resource Allocation | Uniform → Budget-weighted | Low-value agents get cheap models, high-value get expensive |
| Feedback Loop | None → Execution feedback updates routing | Self-correcting agent graphs |

#### NeoTrix Mapping
- **GWT Attention Routing:** AGAO is the academic formalization of what GWT does in NeoTrix. Goal-aware attention ≈ GWT salience scoring. Topology-aware attention ≈ ConsciousnessTree branch dependencies.
- **Axiom A1 (Cost-Aware Routing):** Resource-aware attention is exactly this — routing cheap models to low-impact agents, expensive models to critical paths.
- **Six-Layer Architecture:** AGAO operates at L5 (Cognition) coordinating L1 (Action) agents. Maps to NT-MIND meta-cognition controlling NT-ACT execution.

#### Actionable Insight
```
AGAO validates NeoTrix GWT design:
- Salience scoring should incorporate goal semantics, not just recency
- Agent graph should be dynamic per-task, not fixed
- Resource budget should flow through attention scores
- Execution feedback should update routing weights
```

---

### 2. ReActNet: Inference-Time Graph Engineering for Multi-Agent Workflows
- **Paper:** [arxiv.org/abs/2609.05774](https://arxiv.org/abs/2609.05774)
- **Venue:** arXiv 2026.09
- **Core Idea:** Training-free framework that compiles a query + role-specialized agents into a sequence of directed communication graphs (temporal workflow). Each graph snapshot = one reasoning stage. Each edge = natural-language instruction for message passing.

#### Pattern Analysis
| Dimension | ReActNet Approach | Implication |
|-----------|------------------|-------------|
| Topology | Fixed → Task-conditioned temporal graphs | Graph structure varies per reasoning stage |
| Edge Semantics | Weight vectors → Natural-language instructions | Edges carry meaning, not just connectivity |
| Compilation | Learned → Training-free (compile at inference) | No RL or gradient-based optimization needed |
| Separation | Monolithic → Compile-then-execute | Graph compilation separated from graph execution |

#### NeoTrix Mapping
- **SEAL Pipeline:** ReActNet's temporal graph snapshots = SEAL pipeline stages (Phase-0 through Phase-6). Each stage has a different communication topology.
- **ConsciousnessTree:** The temporal graph sequence mirrors the 6-stage feedback loop (Soil→Roots→Trunk→Branches→Fruits→Core). Each stage compiles a different agent graph.
- **NT-MIND:** "Compile then execute" = meta-cognition designing the workflow before action. Separation of planning from execution.

#### Actionable Insight
```
ReActNet validates NeoTrix SEAL design:
- Pipeline stages should have distinct communication topologies
- Edge instructions should be semantic (natural language), not numeric
- Training-free compilation at inference = zero-cost workflow adaptation
- Separating graph compilation from execution enables inspection
```

---

### 3. AgentInfer: Co-Design of Inference and Architecture for Efficient Agents
- **Paper:** [arxiv.org/abs/2512.18337](https://arxiv.org/abs/2512.18337) (v2: 2026.02)
- **Venue:** arXiv 2026.02
- **Core Idea:** Four synergistic components for end-to-end agent acceleration:
  1. **AgentCollab** — Hierarchical dual-model reasoning (large model plans, small model executes, escalation on stagnation)
  2. **AgentSched** — Cache-aware hybrid scheduler (SJF + cache-preserving modes)
  3. **AgentSAM** — Suffix-automaton speculative decoding reusing cross-session semantic memory
  4. **AgentCompress** — Semantic compression of agent memory without disrupting reasoning

#### Pattern Analysis
| Dimension | AgentInfer Approach | Implication |
|-----------|-------------------|-------------|
| Model Usage | Single model → Dual-model with escalation | Large model for hard, small model for easy |
| Scheduling | FIFO → Cache-aware hybrid | KV cache hit rate matters more than raw latency |
| Speculative Decoding | Generic → Session-memory-augmented | Past sessions accelerate future ones |
| Memory Compression | Synchronous → Asynchronous | Compression runs in background, doesn't block reasoning |

#### NeoTrix Mapping
- **Axiom A1 (Cost-Aware Routing):** AgentCollab is the implementation — large model only for genuinely hard segments. "Concentrating expensive large-model usage on genuinely hard segments."
- **NT-MEMORY:** AgentSAM uses cross-session memory for speculative decoding — past agent interactions accelerate future ones. Maps to experience-tree lazy loading.
- **NT-MIND:** AgentCompress = SEAL pipeline distillation phase. Asynchronous compression = background evolution.
- **KV Cache Optimization:** AgentSched's cache-aware scheduling = paged KV virtualization (KVMem axiom A2).

#### Actionable Insight
```
AgentInfer validates NeoTrix inference design:
- Dual-model (large/small) with escalation = AttentionManager routing
- Cross-session memory for speculative decoding = experience-tree value
- Asynchronous compression = background SEAL pipeline cycles
- 50%+ token reduction with 1.8-2.5x speedup is achievable
```

---

### 4. Codebook Agent: Amortized Topology Design for Multi-Agent Systems
- **Paper:** [arxiv.org/abs/2609.02264](https://arxiv.org/abs/2609.02264)
- **Venue:** arXiv 2026.09
- **Core Idea:** Vector-quantized autoencoder compresses successful topologies into a 16-entry codebook. Reward-weighted MLP maps query embedding to code distribution. MLP proxy reranks candidates. No iterative search, no message passing at test time.

#### Key Empirical Findings
1. Topologies collapse to ~6 distinct graphs even with 64-entry codebook capacity
2. Edge count is **negatively correlated** with token consumption (Pearson r ≈ -0.4) — sparsifying graphs makes inference MORE expensive
3. Message-passing scorer is adjacency-invariant when agents share profiles — cannot rank candidates in default configurations

#### Pattern Analysis
| Dimension | Codebook Agent Approach | Implication |
|-----------|------------------------|-------------|
| Topology Search | Iterative/autoregressive → 16-entry codebook lookup | Most topologies are reusable, not unique |
| Edge Count | Sparse = efficient → Dense = efficient | Intuition about sparsity is wrong for agents |
| Scoring | Message-passing → MLP on flattened adjacency | Simple models beat complex ones when topology is fixed |
| Latency | Search-based → 2.4ms codebook lookup | Topology selection can be near-zero cost |

#### NeoTrix Mapping
- **HyperCube:** The 16-entry codebook is a compressed HyperCube. Successful reasoning topologies are discrete states in the HyperCube space.
- **GWT:** Codebook lookup = GWT salience routing. The codebook IS the routing table. 2.4ms latency = real-time attention routing.
- **ConsciousnessTree:** Topology collapse to ~6 graphs = the 6-stage loop is sufficient. ConsciousnessTree's 6 stages may be the natural attractor.
- **Empirical Finding #2 (Dense = Efficient):** Challenges the "Dark Forest" delete-orphans instinct. In multi-agent systems, more connections reduce total token cost.

#### Actionable Insight
```
Codebook Agent challenges NeoTrix assumptions:
- 16-entry codebook covers most tasks → ConsciousnessTree may need only ~6 topologies
- Dense graphs reduce token cost → don't over-prune agent connections
- Topology selection can be a lookup, not a search → zero-cost routing
- MLP scoring beats message-passing when agents share profiles
```

---

### 5. CondenseFlow: Semantic Compression for Multi-Agent Latent Collaboration
- **Paper:** [aclanthology.org/2026.findings-acl.669](https://aclanthology.org/2026.findings-acl.669.pdf)
- **Venue:** ACL 2026 Findings
- **Core Idea:** Latent Thought Condenser (LTC) uses learnable semantic probes to compress KV caches into fixed-size representations. O(1) communication complexity regardless of context length. Compression error bounded by attention concentration.

#### Key Technical Results
- KV cache memory reduced by **99%+** vs dense transfer
- Inference latency reduced by **~20%** vs dense transfer
- Outperforms text-based methods by **1.7 percentage points** on average
- Compression dimension K=64 achieves optimal efficiency-effectiveness balance

#### Pattern Analysis
| Dimension | CondenseFlow Approach | Implication |
|-----------|----------------------|-------------|
| Communication | Full KV transfer → Fixed-size semantic anchors | Cross-agent communication scales to any context length |
| Compression | Heuristic pruning → Learnable semantic probes | End-to-end learned compression preserves reasoning quality |
| Error Bound | Unknown → Bounded by attention concentration | Theoretical guarantee on compression quality |
| Scaling | Linear with rounds → O(1) per round | Multi-agent systems can scale to unlimited rounds |

#### NeoTrix Mapping
- **VSA HyperCube:** Semantic probes = learnable HyperCube projections. Each probe is a dimension in the VSA space capturing a specific aspect of the thought stream.
- **KB Embedding:** LTC compression = KB embedding with attention-concentration-aware quality. The error bound tells us WHEN compression is safe.
- **NT-MEMORY:** Fixed-size representations = bounded memory per agent. O(1) complexity = memory budget is constant regardless of session length.
- **KVMem (Axiom A2):** Validates Context as Scarce Resource. 99% KV reduction with 20% latency improvement = paged KV virtualization is the right direction.

#### Actionable Insight
```
CondenseFlow validates NeoTrix memory design:
- Fixed-size semantic anchors = bounded memory per module
- O(1) communication = memory budget scales independently of context
- Attention concentration as error bound = know when compression is safe
- Learnable probes > heuristic pruning for cross-agent communication
```

---

## Cross-Paper Synthesis

### Pattern 1: Attention as Universal Coordination
| Paper | Attention Mechanism | Scale |
|-------|-------------------|-------|
| AGAO | Goal + Topology + Resource-aware | Workflow-level |
| ReActNet | Temporal graph snapshots | Reasoning-stage-level |
| AgentInfer | Dual-model escalation signals | Per-step |
| Codebook Agent | Codebook lookup (compressed attention) | Topology-level |
| CondenseFlow | Semantic probes on KV cache | Token-level |

**Synthesis:** Attention operates at every level of the stack — token, agent, workflow, topology. NeoTrix GWT should implement multi-scale attention: token-level (existing), agent-level (AGAO-style), topology-level (Codebook Agent-style).

### Pattern 2: Compression Enables Scale
| Paper | What's Compressed | Reduction |
|-------|------------------|-----------|
| AgentInfer | Agent memory + reasoning traces | 50%+ tokens |
| Codebook Agent | Topology to 16-entry codebook | 2.4ms selection |
| CondenseFlow | KV cache to fixed-size anchors | 99%+ memory |
| BudgetMem | Per-module budget tiers | Variable cost |

**Synthesis:** The future of agent systems is compression at every layer. BudgetMem provides the meta-framework: each module independently compresses based on its budget tier. NeoTrix should implement tiered compression across all modules.

### Pattern 3: Training-Free Methods Dominate
| Paper | Training Required | Method |
|-------|------------------|--------|
| AGAO | No | Rule-based attention scoring |
| ReActNet | No | Query-conditioned graph compilation |
| AgentInfer | No (except RL router) | Heuristic escalation + suffix automaton |
| Codebook Agent | Lightweight (codebook VQ) | Autoencoder + MLP |
| CondenseFlow | Learnable probes | Cross-attention aggregation |

**Synthesis:** Most effective multi-agent coordination methods are training-free or lightly trained. Heavy RL optimization is unnecessary for production agent systems. Aligns with NeoTrix R-P1 (zero unsafe code) — training-free = simpler, safer, more inspectable.

### Pattern 4: Dense > Sparse for Agent Graphs
| Paper | Finding |
|-------|---------|
| Codebook Agent | Edge count negatively correlated with efficiency (r ≈ -0.4) |
| AGAO | Topology-aware attention improves over sparse selection |
| CondenseFlow | Full KV transfer baseline still strong before compression |
| Hive | Colony with full shared ledger outperforms isolated workers |

**Synthesis:** The "Dark Forest" delete-orphans rule may be counterproductive for multi-agent topology. Dense connections reduce total token cost by enabling better information flow. NeoTrix should revisit pruning strategies for agent graphs.

---

## NeoTrix Integration Roadmap

### Immediate (P0)
1. **Multi-Scale GWT Attention:** Implement goal-aware + topology-aware + resource-aware scoring (AGAO pattern)
2. **Budget-Tier Modules:** Each NT-* module gets independent Low/Mid/High budget tiers (BudgetMem pattern)
3. **Fixed-Size Semantic Anchors:** Cross-agent communication via compressed representations, not full state (CondenseFlow pattern)

### Short-Term (P1)
4. **Codebook Topology Router:** 16-entry codebook for workflow topology selection at 2.4ms (Codebook Agent pattern)
5. **Dual-Model Escalation:** Large model for hard segments, small model for easy, escalation on stagnation (AgentInfer pattern)
6. **Asynchronous Memory Compression:** Background compression without blocking reasoning (AgentInfer AgentCompress)

### Medium-Term (P2)
7. **Dense Agent Graph Pruning Policy:** Revisit Dark Forest rule — dense graphs may reduce token cost (Codebook Agent finding)
8. **Training-Free Workflow Compilation:** Compile task-conditioned graphs at inference without RL (ReActNet pattern)
9. **Cross-Session Speculative Decoding:** Past sessions accelerate future ones via suffix automaton (AgentSAM pattern)

---

## References

1. AGAO: Adaptive Goal-aware Attention Orchestration — arXiv:2607.23678
2. ReActNet: Inference-Time Graph Engineering — arXiv:2609.05774
3. AgentInfer: Co-Design of Inference and Architecture — arXiv:2512.18337v2
4. Codebook Agent: Amortized Topology Design — arXiv:2609.02264
5. CondenseFlow: Semantic Compression for Multi-Agent — ACL 2026 Findings
6. QueenBee Planner: Skill-Evolving Topologies — arXiv:2606.27492
7. Agent-Radar: Attention Steering with Context Relevance — arXiv:2605.30136
8. SpecBox: Speculative Sandbox Scheduling — arXiv:2607.23933
9. VeriAttn: Communication-Efficient Verifiable Attention — arXiv:2606.16352
10. BudgetMem: Budget-Tier Agent Memory — ICML'26
11. PlugMem: Plug-and-Play Long-Term Memory — ICML'26
12. CraniMem: Cranial-Inspired Gated Memory — ICLR'26 MemAgents
