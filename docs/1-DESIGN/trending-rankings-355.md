# Trending Rankings — Cycle 355

**Date**: 2026-09-12  
**Focus**: AI agents, LLM tools, reasoning frameworks, memory/attention/routing patterns  
**Sources**: GitHub Trending, ProductHunt, arXiv, OssInsight, HuggingFace Papers

---

## Top 10 New Projects (Not in Cycles 318-354)

### 1. RAGEN — RL Framework for Training Reasoning LLM Agents
- **GitHub**: [mll-lab-nu/RAGEN](https://github.com/mll-lab-nu/RAGEN) — ★ 2.8K+, Apache-2.0
- **Paper**: arXiv:2504.20073 + RAGEN-2 (arXiv:2604.06268)
- **What**: Modular RL system for training LLM agents with multi-turn reinforcement learning. Implements StarPO (State-Thinking-Actions-Reward Policy Optimization) — a trajectory-level agent RL framework. Diagnoses three failure modes: Echo Trap (reward variance cliffs), gradient collapse, and reasoning degradation. RAGEN-2 introduces template collapse (input-agnostic reasoning patterns invisible to entropy metrics) and proposes SNR-Aware Filtering for high-signal prompt selection.
- **Key Pattern**: Trajectory-level RL for agents — not just single-turn reward optimization but entire interaction sequences. The "template collapse" failure mode (reasoning that looks diverse but is input-agnostic) is a critical insight for any self-evolving agent system. Mutual Information as a diagnostic metric over entropy for reasoning quality.
- **NeoTrix Relevance**: The trajectory-level RL framework maps directly to SEAL pipeline's self-evolution — NeoTrix should track MI (not just entropy) when evaluating reasoning quality during evolution cycles. The "template collapse" failure mode is exactly what ConsciousnessTree's cross-domain health monitoring should detect: agents that appear healthy (diverse outputs) but are actually producing input-agnostic responses. StarPO's State-Thinking-Action-Reward decomposition mirrors NeoTrix's GWT attention routing (salience → decision → action → feedback). The SNR-Aware Filtering pattern could improve experience-tree's distillation phase — filter low-signal experiences before crystallization.
- **Stars**: 2.8K+ | **License**: Apache-2.0 | **URL**: github.com/mll-lab-nu/RAGEN

### 2. HugAgentOS — Self-Evolving AgentOS for Ontology-Grounded Reasoning
- **GitHub**: [ZJU-REAL/HugAgentOS](https://github.com/ZJU-REAL/HugAgentOS) — ★ 768, Apache-2.0
- **What**: Enterprise-grade AgentOS that treats domain ontology as a control plane for agent reasoning, decisions, and actions. Combines agentic chat, private KB RAG, sub-agents, MCP tools, Agent Skills, sandboxed execution, long-term memory, automation, and data canvas. Three-layer personal memory (L1 profile in relational store + Milvus vector + Neo4j graph). Audited self-evolution: memory, skills, and orchestration each settled with user approval before activation.
- **Key Pattern**: Ontology-as-control-plane — domain vocabulary constrains agent reasoning, not just context injection. Audited self-evolution with human-in-the-loop approval before any evolved behavior takes effect. Three-layer memory (relational → vector → graph) as architectural standard.
- **NeoTrix Relevance**: The ontology-as-control-plane pattern validates NT-CORE's shared language (CONTEXT.md) — NeoTrix's ubiquitous language is doing exactly this: constraining agent reasoning through formal domain definitions. The audited self-evolution pattern (user approves before evolved behavior activates) is a governance mechanism NeoTrix should adopt for SEAL pipeline — evolution outputs should be reviewed before production wiring. Three-layer memory validates NT-MEMORY's multi-backend architecture. The "settle memory and skills out of real work" principle is experience-tree's core philosophy.
- **Stars**: 768 | **License**: Apache-2.0 | **URL**: github.com/ZJU-REAL/HugAgentOS

### 3. InftyThink — Breaking Length Limits of Long-Context Reasoning
- **GitHub**: [ZJU-REAL/InftyThink](https://github.com/ZJU-REAL/InftyThink) — ★ 57, MIT
- **Paper**: arXiv:2503.06692 (ICLR 2026) + InftyThink+ (ICML 2026)
- **What**: Paradigm that transforms monolithic reasoning into iterative process with intermediate summarization. Divides complex reasoning into multiple interrelated short reasoning segments, each within computationally efficient context length. Creates sawtooth memory pattern. InftyThink+ adds RL training for adaptive summarization. Reduces O(L²) to O(n·ℓ²) complexity. 3-13% improvement on MATH500, AIME24, GPQA_diamond with Qwen2.5-Math-7B. Enables 8K-context models to perform long-context reasoning.
- **Key Pattern**: Iterative summarization as a primitive — not a hack but a first-class reasoning paradigm. Sawtooth memory pattern: reason→summarize→reason→summarize, each segment bounded. The key insight: you don't need bigger context windows if you restructure the reasoning process itself. Human cognitive working memory as architectural inspiration.
- **NeoTrix Relevance**: The sawtooth memory pattern maps directly to experience-tree's hub-and-spoke architecture: hub index (summary) → branch (detailed reasoning) → hub (re-summarize). InftyThink validates NeoTrix's lazy branch loading — don't load all context, load summaries on-demand and drill into details only when needed. The RL-trained adaptive summarization (InftyThink+) could improve SEAL pipeline's distillation phase — learn when to summarize vs when to preserve detail. The "8K-context models doing long-context reasoning" result validates Axiom A2 (Context as Scarce Resource) — clever process design beats raw context scaling.
- **Stars**: 57 | **License**: MIT | **URL**: github.com/ZJU-REAL/InftyThink

### 4. Agent libOS — Runtime Substrate for Capability-Controlled Self-Evolving Agents
- **Paper**: arXiv:2606.03895 (2026)
- **What**: Agent-native library OS substrate that separates three planes: action plane (what the agent can do), authority plane (what the agent is authorized to affect), and evidence plane (durable intent, outcomes, audit). Core invariant: "A self-evolving agent may change what it can ask for, but it cannot thereby change what it is authorized to affect or where its information may flow." Solves the problem that self-evolution becomes authority escalation when action visibility equals authorization.
- **Key Pattern**: Evolve affordances without implicit authority — the three-plane separation (action/authority/evidence) prevents self-evolution from becoming a security hole. Capability systems + information flow control for agents. The "exact release" mechanism: one-shot authorization that expires, not persistent permissions.
- **NeoTrix Relevance**: The three-plane separation maps directly to NT-SHIELD architecture: action plane (tool capabilities), authority plane (egress guard trust tiers), evidence plane (audit log). The core invariant is exactly NT-SHIELD's security model — evolving agent capabilities must not expand implicit authority. The "exact release" pattern (one-shot authorization) could improve NT-SHIELD's secret scrubbing — authorize per-request, not per-session. This paper provides the formal security foundation for NeoTrix's self-evolution that AGENTS.md's "指针守恒" rule is trying to enforce informally. The evidence plane maps to EventBus event_log (R-P84).
- **Stars**: N/A (paper) | **License**: arXiv perpetual non-exclusive | **URL**: arxiv.org/abs/2606.03895

### 5. Ring-Linear-2.0 — Hybrid Linear+Softmax Attention for Long-Context
- **Paper**: arXiv:2510.19338 (2025, models released 2026)
- **Models**: Ring-mini-linear-2.0 (16B params, 957M activations), Ring-flash-linear-2.0 (104B params, 6.1B activations)
- **What**: Hybrid architecture integrating linear attention (constant KV cache, O(n) compute) with softmax attention (quadratic but high expressivity). Layer groups of M linear layers + 1 softmax layer. Linear attention uses Lightning Attention with fixed decay. MoE architecture for sparsity. 1/10th inference cost of 32B dense model, 50% reduction vs original Ring series. Optimal ratio: M=7 (7 linear per 1 softmax) at high FLOP budgets.
- **Key Pattern**: Hybrid attention as architecture principle — not linear OR softmax but strategic interleaving. The ratio M (linear:softmax layers) is a tunable efficiency-expressivity knob. Constant KV cache from linear layers + selective expressivity from softmax layers. Scaling law curves show hybrid consistently outperforms pure softmax.
- **NeoTrix Relevance**: The hybrid attention pattern maps to GWT's attention routing — not all attention needs the same computational cost. NeoTrix could implement "attention tiers": lightweight linear attention for routine monitoring (NT-MEMORY indexing, heartbeat), full softmax attention for complex reasoning (SEAL distillation, architecture decisions). The M=7 ratio insight is actionable: ~87% of processing should be cheap/linear, ~13% expensive/expressive. The MoE sparsity pattern validates Rune Socketing — not all modules activate for all tasks. The constant KV cache from linear attention validates KVMem's paged KV approach (Axiom A2).
- **Stars**: N/A (paper) | **License**: Research | **URL**: huggingface.co/inclusionAI

### 6. Qualixar OS — Universal OS for AI Agent Orchestration
- **Paper**: arXiv:2604.06392 (April 2026)
- **What**: First application-layer operating system for universal AI agent orchestration. 10 LLM providers, 8+ agent frameworks, 7 communication transports. 12 multi-agent execution topologies (sequential, parallel, hierarchical, DAG, grid, debate, tournament, etc.). Goodhart detection via cross-model entropy monitoring. Self-evolution trilemma navigation. Behavioral contracts with design-by-contract invariants.
- **Key Pattern**: Topology-as-configuration — 12 execution patterns as first-class primitives, not hard-coded workflows. Goodhart detection for judge integrity (cross-model entropy). The self-evolution trilemma: no alignment method can simultaneously achieve adaptation, safety, and performance — must navigate tradeoffs.
- **NeoTrix Relevance**: The 12 topologies map to NT-ACT orchestration patterns — NeoTrix should expose topology as a configuration primitive (sequential SEAL phases, parallel capability execution, debate-style reasoning). Goodhart detection maps to NT-META's cross-module audit — detecting when metrics diverge from actual quality. The self-evolution trilemma is the formal statement of what ConsciousnessTree tries to balance empirically. The behavioral contracts pattern validates AGENTS.md's axioms — formal invariants that constrain agent behavior, not just conventions.
- **Stars**: N/A (paper) | **License**: Research | **URL**: arxiv.org/abs/2604.06392

### 7. CoAgent — Concurrency Control for Multi-Agent Systems
- **Paper**: arXiv:2606.15376 (June 2026)
- **What**: Addresses the multi-agent concurrency problem — when multiple agents operate over shared state simultaneously. Classical concurrency control (locks, OCC) fails because agent transactions span minutes of inference, read sets are broad/opaque, and writes take effect immediately. Proposes LLM-native concurrency control: the LLM itself inspects its own read/write history and identifies only premised actions for re-execution on conflict, replacing OCC's all-or-nothing restart. Prefix KV caching optimization for efficiency.
- **Key Pattern**: LLM-native concurrency control — instead of database-style locks/OCC, let the agent reason about its own conflict history. The "agent transaction spans minutes" insight: classical CC assumptions break down for LLM agents. Selective re-execution (not full rollback) based on causal dependency analysis.
- **NeoTrix Relevance**: The multi-agent concurrency problem directly applies to NeoTrix's multi-domain execution (NT-CORE, NT-MIND, NT-WORLD operating in parallel). CoAgent's insight that "agent transactions span minutes" validates NT-NEXUS's need for cross-session state management. The LLM-native CC pattern (agent reasons about its own conflicts) maps to ConsciousnessTree's self-audit — the system should reason about its own state conflicts. The selective re-execution pattern could improve EventBus event handling — instead of replaying all events on conflict, identify which downstream effects are premised on the conflicting value.
- **Stars**: N/A (paper) | **License**: arXiv perpetual non-exclusive | **URL**: arxiv.org/abs/2606.15376

### 8. HeteroPanacea — Disaggregated Serving for Agentic Inference
- **Paper**: arXiv:2608.03741 (August 2026)
- **What**: Simulation framework for disaggregated LLM serving — separates prefill, decode, attention, and FFN onto different hardware. Agentic inference (multi-turn tool-calling) creates heterogeneous workloads where prefill and decode have different compute/memory-bandwidth needs. Simulates 75% throughput improvement over traditional serving. 4-way PDAF (Prefill-Decode-Attention-FFN) disaggregation is most consistent across models. Studies model architecture vs. disaggregation benefit relationships.
- **Key Pattern**: Disaggregation by compute phase — not all inference phases need the same hardware. Agentic workloads (multi-turn, tool-calling) are fundamentally different from batch inference and need specialized serving. The "what hardware for each component" question is the new frontier.
- **NeoTrix Relevance**: The disaggregated serving pattern maps to NeoTrix's tiered routing (Axiom A1: Cost-Aware Routing). Different phases of agent processing (perception, reasoning, action, memory consolidation) could run on different compute tiers. The insight that "agentic workloads are fundamentally different" validates NeoTrix's separation of concern across 7 domains — each domain has different computational characteristics. The PDAF disaggregation is a hardware-level version of Rune Socketing: different compute resources for different function types. The 75% throughput improvement suggests NeoTrix should optimize for phase-specific execution, not generic processing.
- **Stars**: N/A (paper) | **License**: Research | **URL**: arxiv.org/abs/2608.03741

### 9. PackInfer — Compute- and I/O-Efficient Attention for Batched Inference
- **Paper**: arXiv:2602.06072 (February 2026)
- **What**: Kernel-level attention framework for heterogeneous batched inference. Addresses the mismatch between per-request optimization (FlashAttention) and production batching with heterogeneous sequence lengths. Orchestrates batched requests into load-balanced execution groups. I/O-aware grouping co-locates shared-prefix requests and reorganizes KV caches into group-contiguous layouts. 13-20% latency reduction, 20% throughput improvement over FlashAttention.
- **Key Pattern**: Batch-aware attention optimization — not all requests in a batch are equal. Shared-prefix co-location reduces redundant data movement. Group-contiguous KV layouts reduce memory fragmentation. The insight: production batching with heterogeneous sequences is a different optimization problem than single-request attention.
- **NeoTrix Relevance**: The batch-aware optimization pattern maps to NT-MEMORY's KB operations — batch queries against the knowledge base should be optimized for shared-prefix co-location (e.g., multiple queries about the same domain share context). The group-contiguous KV layout insight is actionable for experience-tree's hub-and-spoke storage — group experiences by domain/branch for contiguous access patterns. The 20% throughput improvement from I/O-awareness validates focusing on data movement, not just computation. This is a systems-level optimization NeoTrix should apply to KB query batching.
- **Stars**: N/A (paper) | **License**: Research | **URL**: arxiv.org/abs/2602.06072

### 10. Attention-MoA — Inter-Agent Semantic Attention for Mixture-of-Agents
- **Paper**: arXiv:2601.16596 (January 2026)
- **What**: Enhances Mixture-of-Agents (MoA) framework through Inter-agent Semantic Attention — agents actively attend to each other's outputs, not just aggregate them. Inter-layer Residual Module with Adaptive Early Stopping prevents information degradation in deep layers. Ensemble of small open-source models outperforms Claude-4.5-Sonnet and GPT-4.1 on MT-Bench (8.83) and AlpacaEval 2.0 (77.36% LC Win Rate). 91.15% LC Win Rate on AlpacaEval 2.0 overall.
- **Key Pattern**: Semantic attention between agents (not just aggregation) — agents actively reason about each other's outputs. Adaptive early stopping prevents over-computation. Small model ensembles outperforming large proprietary models through structured collaboration. The key insight: MoA variants with dynamic routing and residual connections fail without deep semantic interaction between agents.
- **NeoTrix Relevance**: The inter-agent semantic attention pattern maps to NT-CORE's GWT broadcast — instead of broadcasting raw salient information, agents should semantically attend to each other's outputs. The adaptive early stopping pattern could improve SEAL pipeline — stop evolution cycles when additional iterations show diminishing returns. The small-model-beats-large-model result validates NeoTrix's domain-specialized architecture — 7 specialized domains (NT-CORE through NT-FEEL) collaborating should outperform a single monolithic model. The residual connection pattern maps to NT-NEXUS cross-session memory — preserve and propagate important information across layers/iterations.
- **Stars**: N/A (paper) | **License**: Research | **URL**: arxiv.org/abs/2601.16596

---

## Meta-Observations

### Cross-Project Patterns
1. **Self-Evolution with Guardrails**: RAGEN (MI-based quality), HugAgentOS (audited evolution), Agent libOS (capability control), Qualixar (trilemma navigation) — all converge on "evolve but constrain"
2. **Process Restructuring over Hardware Scaling**: InftyThink (iterative summarization), Ring-Linear-2.0 (hybrid attention), PackInfer (batch-aware), HeteroPanacea (disaggregation) — clever process design beats raw compute
3. **Agent Concurrency as First-Class Problem**: CoAgent, Qualixar, RAGEN — multi-agent coordination is now a systems problem, not just an application problem
4. **Small Models Collaborating**: Attention-MoA proves structured collaboration of small models beats large proprietary models — validates domain-specialized architecture

### NeoTrix Absorption Priority
| Priority | Project | Pattern to Absorb | Target Domain |
|----------|---------|-------------------|---------------|
| P0 | Agent libOS | Three-plane security (action/authority/evidence) | NT-SHIELD |
| P0 | RAGEN-2 | Template collapse detection via MI | NT-META |
| P1 | InftyThink | Sawtooth memory / iterative summarization | NT-MEMORY |
| P1 | CoAgent | LLM-native concurrency control | NT-NEXUS |
| P2 | Ring-Linear-2.0 | Attention tiers (linear:softmax ratio) | NT-CORE/GWT |
| P2 | Qualixar | Topology-as-configuration | NT-ACT |
| P3 | Attention-MoA | Inter-agent semantic attention | NT-CORE |
| P3 | PackInfer | Batch-aware KB query optimization | NT-MEMORY |
