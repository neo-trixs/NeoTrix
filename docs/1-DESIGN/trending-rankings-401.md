# Trending Rankings — Cycle 401

**Date**: 2026-09-12
**Focus**: AI agents, LLM tools, reasoning frameworks, memory, attention, routing
**Sources**: GitHub Trending, ProductHunt, arXiv, HackerNews, DataAIHub

---

## Top 10 New Projects (Not in Cycles 318-400)

### 1. OmniAgent (YeQing17-2026)
- **URL**: https://github.com/YeQing17-2026/OmniAgent
- **Stars**: 2,557
- **Category**: Self-Evolving Agent Framework
- **Novel Pattern**: OmniEvolve — full-dimensional self-evolution across Skill, Context, and BrainModel. Dual-path alignment for proactive memory (explicit feedback + implicit LLM induction). Hyper-Harness with dynamic multi-agent (Sentinel planner + Guardian safety). Deep Reflexion dual-layer architecture (real-time risk interception + failure-to-insight conversion).
- **NeoTrix Mapping**: NT-CORE (ConsciousnessTree ↔ OmniEvolve), NT-MIND (SEAL pipeline ↔ skill self-evolution), NT-SHIELD (Hyper-Harness ↔ 4-layer security scanning)
- **Source**: GitHub, April 2026

### 2. MAGMA: Multi-Graph Agentic Memory
- **URL**: https://aclanthology.org/2026.acl-long.1709.pdf
- **Stars**: N/A (paper)
- **Category**: Memory Architecture
- **Novel Pattern**: Orthogonal graph representation across semantic, temporal, causal, and entity dimensions. Adaptive Traversal Policy routes retrieval based on query intent. Dual-stream memory evolution (synaptic ingestion fast path + asynchronous consolidation slow path). Outperforms state-of-the-art on LoCoMo and LongMemEval.
- **NeoTrix Mapping**: NT-MEMORY (KB ↔ MAGMA graphs), NT-CORE (GWT ↔ intent-aware routing), NT-WORLD (crawler ↔ dual-stream ingestion)
- **Source**: ACL 2026

### 3. Qwen-AgentWorld
- **URL**: https://github.com/QwenLM/Qwen-AgentWorld
- **Stars**: 984
- **Category**: Language World Model
- **Novel Pattern**: Native language world model simulating agentic environments via chain-of-thought across 7 domains (MCP, Search, Terminal, SWE, Android, Web, OS). Three-stage pipeline (CPT→SFT→RL) trained on 10M+ real-world interaction trajectories. Zero-shot generalization to out-of-distribution environments. MoE 35B total / 3B active.
- **NeoTrix Mapping**: NT-WORLD (UnifiedCrawler ↔ environment simulation), NT-CORE (E8 ↔ 7-domain reasoning), NT-ACT (tool calling ↔ trajectory training)
- **Source**: arXiv 2606.24597, June 2026

### 4. MARCH: Memory-Anchor Routing across Context History
- **URL**: https://arxiv.org/pdf/2608.12435v1
- **Stars**: N/A (paper)
- **Category**: Recurrent Memory Architecture
- **Novel Pattern**: Periodically caches cumulative recurrent-state checkpoints as state anchors with content-conditioned keys. Memory bank grows with context length. Attention-style aggregation over historical anchors. Learned null route suppresses historical branch when current state is sufficient. Consistent improvements over recurrent baselines on LongBench and NIAH.
- **NeoTrix Mapping**: NT-MEMORY (experience-tree ↔ state anchors), NT-CORE (ConsciousnessTree ↔ null route suppression), NT-MIND (evolution ↔ memory expansion)
- **Source**: arXiv 2608.12435, Aug 2026

### 5. GitNexus (Akon Labs)
- **URL**: https://www.producthunt.com/products/gitnexus-akon-labs
- **Stars**: 45,000 (GitHub)
- **Category**: Knowledge Graph Kernel for Coding Agents
- **Novel Pattern**: Deterministic code graph resolution (not embedding-based). Unifies every codebase into one source of truth. Exact callers, imports, impact analysis. 51% cheaper coding agent runs. Works with any agent over MCP.
- **NeoTrix Mapping**: NT-MEMORY (KB graph ↔ code knowledge graph), NT-CORE (HyperCube ↔ graph-native context), NT-ACT (MCP tools ↔ code graph queries)
- **Source**: ProductHunt, Aug 2026

### 6. GNAP: Git-Native Agent Protocol
- **URL**: https://github.com/farol-team/gnap
- **Stars**: ~500 (new)
- **Category**: Agent Coordination Protocol
- **Novel Pattern**: Git-native coordination using 4 JSON files. No server, no database. Any agent that can git push can participate. Identity, rules, memory, tools, skills all version-controlled in git. Decentralized agent team coordination.
- **NeoTrix Mapping**: NT-ACT (orchestration ↔ git-native coordination), NT-MEMORY (version-controlled state), NT-CORE (skill tree ↔ git-versioned skills)
- **Source**: awesome-ai-agents-2026 list

### 7. ReActNet: Inference-Time Graph Engineering
- **URL**: https://arxiv.org/abs/2609.05774
- **Stars**: N/A (paper)
- **Category**: Multi-Agent Workflow Optimization
- **Novel Pattern**: Training-free framework compiling queries into temporal workflow graphs. Each graph snapshot = one reasoning stage. Edges carry natural-language communication instructions. Separates graph compilation from graph execution. Multi-agent coordination without RL or gradient-based topology.
- **NeoTrix Mapping**: NT-CORE (GWT ↔ temporal graph compilation), NT-ACT (orchestration ↔ message passing), NT-MIND (SEAL ↔ workflow evolution)
- **Source**: arXiv 2609.05774, Sep 2026

### 8. NeoHorse-1: Recursive Self-Improvement via Agentic Post-Training
- **URL**: https://arxiv.org/abs/2609.08183
- **Stars**: N/A (paper)
- **Category**: Self-Improving Agent Models
- **Novel Pattern**: Heterogeneous model pool + intelligent routing. Records predicted capability demand, selected service tier, and interaction for each turn. Routing signals organize SFT into 3-stage curriculum. Routing-guided on-policy distillation. Evaluation-selection-update loop closes the RSI cycle.
- **NeoTrix Mapping**: NT-MIND (SEAL ↔ agentic post-training), NT-CORE (GWT ↔ capability-aware routing), NT-REPAIR (self-healing ↔ recursive improvement)
- **Source**: arXiv 2609.08183, Sep 2026

### 9. DeLM: Decentralized Language Models with Shared Context
- **URL**: https://arxiv.org/abs/2606.10662
- **Stars**: N/A (paper)
- **Category**: Decentralized Multi-Agent Systems
- **Novel Pattern**: Decentralized coordination via parallel agents + shared verified context + task queue. Agents asynchronously claim subtasks, read accumulated progress, perform local reasoning, write back compact verified updates. Shared context as common communication substrate. 10.5pp gain on SWE-bench, 50% cost reduction.
- **NeoTrix Mapping**: NT-ACT (orchestration ↔ decentralized task queue), NT-MEMORY (shared KB ↔ verified context), NT-CORE (GWT ↔ progressive context loading)
- **Source**: arXiv 2606.10662, Jun 2026

### 10. OmniRoute: AI Gateway Routing Infrastructure
- **URL**: GitHub trending August 2026
- **Stars**: +16.8K (August growth)
- **Category**: AI Model Routing
- **Novel Pattern**: Routes multiple AI tools and model providers through one endpoint. Unified gateway for heterogeneous models. Fallback chains, observability, access control. Bridges local models with cloud providers.
- **NeoTrix Mapping**: NT-IO (LLM providers ↔ unified routing), NT-CORE (GWT cost-aware routing ↔ OmniRoute), NT-SHIELD (access control ↔ gateway security)
- **Source**: GitHub Trending Aug 2026

---

## Emerging Meta-Trends (Cycle 401)

| Trend | Signal | NeoTrix Relevance |
|-------|--------|-------------------|
| **Self-Evolving Agents** | OmniAgent, NeoHorse-1, QueenBee | SEAL pipeline, ConsciousnessTree |
| **Multi-Graph Memory** | MAGMA, ROAM, SYNAPSE | KB node/edge schema, HyperCube |
| **Language World Models** | Qwen-AgentWorld | NT-WORLD perception, E8 reasoning |
| **State-Anchor Recurrence** | MARCH, Metis | experience-tree, memory hub |
| **Git-Native Agent Identity** | GNAP, GitAgent, GitNexus | NT-MEMORY versioning, skill tree |
| **Decentralized MAS** | DeLM, ReActNet, NeuralFSM | NT-ACT orchestration, EventBus |
| **Deterministic Code Graphs** | GitNexus | KB graph, capability bridge |
| **Routing as Infrastructure** | OmniRoute, ngrok AI Gateway | NT-IO provider routing, GWT cost |
| **Recursive Self-Improvement** | NeoHorse-1, OmniAgent | SEAL feedback loop, NT-REPAIR |
| **Temporal Workflow Graphs** | ReActNet, Codebook Agent | GWT salience, ConsciousnessTree |

---

## Star Velocity Leaders (New Entrants)

| Project | Category | Velocity | Novelty |
|---------|----------|----------|---------|
| OmniRoute | AI Gateway | +16.8K/mo | Unified routing endpoint |
| GitNexus | Code Knowledge Graph | 45K total | Deterministic graph resolution |
| OmniAgent | Self-Evolving Agent | 2.5K total | OmniEvolve 3-axis |
| Qwen-AgentWorld | World Model | 984 total | 7-domain language WM |
| DeLM | Decentralized MAS | 10.5pp SWE gain | Shared verified context |
| NeoHorse-1 | RSI Models | N/A | 3-stage curriculum routing |
| ReActNet | Workflow Graphs | N/A | Temporal graph compilation |
| GNAP | Agent Protocol | ~500 total | Git-native coordination |
| MAGMA | Memory Architecture | N/A | 4-graph orthogonal retrieval |
| MARCH | Recurrent Memory | N/A | State-anchor content routing |

---

## Cross-Reference with Prior Cycles

Projects confirmed **NOT** in cycles 318-400:
- OmniAgent: novel (self-evolving agent framework, not previously tracked)
- MAGMA: novel (multi-graph memory, ACL 2026)
- Qwen-AgentWorld: novel (language world model)
- MARCH: novel (state-anchor recurrence)
- GitNexus: novel (deterministic code graph kernel)
- GNAP: novel (git-native agent protocol)
- ReActNet: novel (inference-time graph engineering)
- NeoHorse-1: novel (recursive self-improvement)
- DeLM: novel (decentralized MAS with shared context)
- OmniRoute: novel (unified AI routing infrastructure)

## Action Items for NeoTrix Integration

1. **MAGMA 4-graph schema** → Evaluate orthogonal graph representation for KB node/edge schema
2. **Qwen-AgentWorld 7-domain WM** → Study environment simulation for NT-WORLD crawler testing
3. **MARCH state anchors** → Review state-anchor pattern for experience-tree lazy branch loading
4. **GitNexus deterministic graph** → Consider embedding-free code graph for capability bridge
5. **NeoHorse-1 routing curriculum** → Examine 3-stage curriculum for SEAL pipeline
6. **DeLM shared verified context** → Study decentralized task queue for NT-ACT orchestration
7. **OmniRoute unified routing** → Evaluate for NT-IO provider gateway
8. **GNAP git-native identity** → Assess for skill tree versioning
