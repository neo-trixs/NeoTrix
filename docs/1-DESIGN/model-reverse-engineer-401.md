# Model Reverse Engineering — Cycle 401

**Date**: 2026-09-12
**Focus**: Efficient inference, attention mechanisms, agent coordination, memory architectures
**Sources**: arXiv (Aug-Sep 2026), ACL 2026, EMNLP 2026

---

## 5 Selected Papers/Models

### 1. MAGMA: Multi-Graph Agentic Memory (ACL 2026)

**Paper**: https://aclanthology.org/2026.acl-long.1709.pdf
**Core Innovation**: Orthogonal graph representation for agent memory across 4 relational dimensions

**Architecture**:
- 4 independent graph views: semantic, temporal, causal, entity
- Adaptive Traversal Policy: query-intent-aware routing over graph views
- Dual-stream evolution: Synaptic Ingestion (fast) + Asynchronous Consolidation (slow)
- Query Process: Intent-Aware Router → Adaptive Topological Retrieval → Context Synthesizer

**Key Results**:
- Outperforms SOTA on LoCoMo (temporal/multi-hop reasoning)
- Outperforms SOTA on LongMemEval (long-context benchmarks)
- Reduces retrieval latency vs monolithic memory approaches

**NeoTrix Domain Mapping**:
| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** | 4-graph schema | KB node/edge: add temporal/causal/entity graph dimensions to current semantic-only schema |
| **NT-CORE** | Intent-Aware Router | GWT salience: route attention based on query intent decomposition |
| **NT-WORLD** | Dual-stream ingestion | Crawler pipeline: fast path for live data, slow path for structural consolidation |
| **NT-MIND** | Asynchronous consolidation | SEAL pipeline: background memory consolidation between evolution cycles |

**Absorption Action**:
- Extend KB schema with temporal/causal/entity graph edges (R-P42: extend existing, no new module)
- Implement intent-aware retrieval routing in nt_memory (leverages existing GWT attention)
- Study dual-stream pattern for crawl pipeline (fast ingestion vs slow enrichment)

---

### 2. MARCH: Memory-Anchor Routing across Context History

**Paper**: https://arxiv.org/pdf/2608.12435v1
**Core Innovation**: Content-routed state anchors for scaling recurrent memory beyond fixed-size state

**Architecture**:
- Periodic state checkpointing as "state anchors"
- Each anchor keyed by content-conditioned compact key
- Memory bank grows with context length (controllable history vs cost tradeoff)
- Anchor query attends all causally available state anchors
- Learned null route suppresses historical branch when current state sufficient

**Key Results**:
- Consistent improvement over strong recurrent baselines
- Outperforms on commonsense reasoning, LongBench, in-context retrieval, NIAH
- Robust extrapolation beyond training context length

**NeoTrix Domain Mapping**:
| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-MEMORY** | State anchors | experience-tree: checkpoint branch states at cycle boundaries for lazy loading |
| **NT-CORE** | Null route suppression | ConsciousnessTree: suppress historical context when current coherence sufficient |
| **NT-MIND** | Growing memory bank | SEAL evolution: expand memory capacity proportional to evolution depth |
| **NT-IO** | Content routing | GWT: route attention based on content similarity to historical anchors |

**Absorption Action**:
- Implement state-anchor pattern for experience-tree branch indexing
- Study null-route mechanism for GWT attention gating (suppress irrelevant historical context)
- Evaluate growing memory bank for KB embedding capacity planning

---

### 3. Kimi K3: Native Multimodal Agentic Model (2.8T MoE)

**Model**: https://github.com/MoonshotAI/Kimi-K3
**Core Innovation**: Kimi Delta Attention (KDA) + Attention Residuals (AttnRes) + Stable LatentMoE

**Architecture**:
- 2.8T total parameters, 16/896 experts active (2.5x scaling efficiency over K2)
- KDA: gated multi-head latent attention layers (from Kimi model line)
- AttnRes: attention residual connections for stable long-context processing
- Native vision: text + images + video in unified model
- 1M token context window

**Key Results**:
- GPQA Diamond: 93.5 (competitive with GPT-5.6 Sol at 94.1)
- BrowseComp: 91.0 (surpasses most frontier models)
- Terminal-Bench 2.1: 88.3
- MathVision: 94.3
- Sustains long engineering sessions with minimal human oversight

**NeoTrix Domain Mapping**:
| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | KDA attention | E8 reasoning: evaluate KDA-style gated attention for HyperCube attention |
| **NT-WORLD** | Native multimodality | UnifiedCrawler: extend perception to image/video understanding |
| **NT-IO** | 1M context | Provider integration: test Kimi K3 via API for long-context tasks |
| **NT-MIND** | MoE sparsity | Skill routing: study expert selection for task-specific capability dispatch |
| **NT-SHIELD** | AttnRes stability | Safety: evaluate residual connections for stable reasoning under adversarial input |

**Absorption Action**:
- Study KDA gating mechanism for potential integration with HyperCube attention
- Test Kimi K3 API for long-context NT-WORLD perception tasks
- Evaluate MoE expert selection pattern for NT-MIND skill routing

---

### 4. HeRo: History-Aware Routing for Efficient LLM Inference

**Paper**: https://arxiv.org/abs/2609.08189
**Core Innovation**: Router memory mechanism that maintains explicit routing state across model depth

**Architecture**:
- Linear attention constructs routing history: incrementally aggregates preceding routing scores + residual updates
- At each routed layer: router conditions jointly on accumulated state + current hidden representation
- Token-wise FFN routing with lightweight routers + adapters on frozen backbone
- No modification to pretrained parameters

**Key Results**:
- Llama 3.1-8B: bypasses 26.87% of parameters while achieving 100.24% of dense performance
- Retains 97.01% while bypassing 38.82% under tighter budget
- Ablation: removing routing history degrades performance, especially on multi-step reasoning + code

**NeoTrix Domain Mapping**:
| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | History-aware routing | GWT: add routing memory to attention routing decisions |
| **NT-IO** | Dynamic layer bypass | Provider routing: implement cost-aware layer skipping for inference |
| **NT-MIND** | Lightweight adapters | SEAL: train small adapters for task-specific routing without full fine-tuning |
| **NT-ACT** | Resource efficiency | Orchestration: reduce inference cost for multi-agent task execution |

**Absorption Action**:
- Study linear-attention routing history for GWT attention modulation
- Evaluate layer-bypass pattern for NT-IO cost optimization (Axiom A1: Cost-Aware Routing)
- Test lightweight adapter training for task-specific routing in NT-MIND

---

### 5. NeuralFSM: Adaptive Multi-Agent Coordination via Finite-State Execution

**Paper**: https://aclanthology.org/2026.acl-long.1543.pdf
**Core Innovation**: FSM-based coordination with Temporal Graph Network controller learning state transitions + sparse communication routing

**Architecture**:
- Reusable FSM structure: state transitions + inter-agent communication weights learned from traces
- Temporal Coordination Controller: TGN-based, task-conditioned, adaptive state transitions
- Dual-defense protection: graph regularization (training) + trust-aware message attenuation (runtime)
- Sparse routing: activates only relevant subgraph at each step

**Key Results**:
- 6.74%–19.39% improvement over baselines across 6 benchmarks
- Strong robustness: only 1.82% performance drop under adversarial attack
- Reduces token consumption via sparse routing
- Best on MATH (+21.69% over IO baseline, +12.75% over SOTA)

**NeoTrix Domain Mapping**:
| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** | FSM coordination | Orchestration: model agent workflows as finite-state machines |
| **NT-CORE** | TGN controller | ConsciousnessTree: temporal graph networks for cycle-to-cycle state transitions |
| **NT-SHIELD** | Trust-aware attenuation | Security: attenuate low-trust agent communications in multi-agent scenarios |
| **NT-MEMORY** | Interaction traces | KB: store FSM transition traces for workflow optimization |

**Absorption Action**:
- Study FSM-based workflow modeling for NT-ACT task orchestration
- Evaluate trust-attenuation pattern for NT-SHIELD multi-agent security
- Test TGN-based temporal coordination for ConsciousnessTree cycle transitions

---

## Synthesis: Cross-Paper Patterns

### Pattern 1: Multi-Dimensional Memory Representation
- **MAGMA**: 4 orthogonal graphs (semantic/temporal/causal/entity)
- **MARCH**: State anchors with content-conditioned keys
- **Convergence**: Memory is not monolithic — disentangled representations improve retrieval
- **NeoTrix Impact**: Extend KB schema beyond semantic-only edges

### Pattern 2: History-Aware Dynamic Routing
- **HeRo**: Router memory across model depth
- **NeuralFSM**: TGN-based temporal state transitions
- **Convergence**: Routing decisions benefit from accumulated history, not just current state
- **NeoTrix Impact**: Add routing memory to GWT attention mechanism

### Pattern 3: Sparse Activation for Efficiency
- **Kimi K3**: MoE 16/896 experts
- **HeRo**: Bypass 27-39% of parameters
- **NeuralFSM**: Sparse subgraph activation
- **Convergence**: Selective activation is key to scaling without proportional cost increase
- **NeoTrix Impact**: Evaluate MoE-style expert selection for skill routing

### Pattern 4: Dual-Stream Processing
- **MAGMA**: Fast ingestion + slow consolidation
- **NeuralFSM**: FSM structure + learned controller
- **Convergence**: Separating structure from dynamics enables both responsiveness and depth
- **NeoTrix Impact**: Apply dual-stream to crawl pipeline and evolution pipeline

### Pattern 5: Trust and Robustness in Multi-Agent Systems
- **NeuralFSM**: Dual-defense protection
- **Kimi K3**: AttnRes for stable reasoning
- **Convergence**: Multi-agent systems need explicit robustness mechanisms
- **NeoTrix Impact**: Extend NT-SHIELD with trust-attenuation for agent communications

---

## Priority Integration Roadmap

| Priority | Paper | Integration | Effort |
|----------|-------|-------------|--------|
| P0 | MAGMA | KB 4-graph schema extension | Medium |
| P0 | HeRo | GWT routing memory | Medium |
| P1 | MARCH | experience-tree state anchors | Low |
| P1 | NeuralFSM | NT-ACT FSM workflow modeling | Medium |
| P2 | Kimi K3 | KDA attention study + API integration | Low |
| P2 | MAGMA | Dual-stream crawl ingestion | Medium |
| P3 | HeMo | Layer-bypass cost optimization | High |
| P3 | NeuralFSM | Trust-attenuation for NT-SHIELD | Medium |

---

## Citation Index

| Paper | Venue | Date | arXiv |
|-------|-------|------|-------|
| MAGMA | ACL 2026 | 2026 | 2026.acl-long.1709 |
| MARCH | arXiv | Aug 2026 | 2608.12435 |
| Kimi K3 | MoonshotAI | Sep 2026 | GitHub |
| HeRo | arXiv | Sep 2026 | 2609.08189 |
| NeuralFSM | ACL 2026 | 2026 | 2026.acl-long.1543 |
