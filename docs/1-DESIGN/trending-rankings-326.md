# Trending Rankings — Cycle 326

**Date**: 2026-09-11
**Sources**: GitHub Trending, ProductHunt, arXiv, OSSInsight

## New Projects (not in cycles 318-325)

### 1. MPAC — Multi-Principal Agent Coordination Protocol
- **URL**: github.com/mpac-protocol/mpac
- **Stars**: ~500 (emerging)
- **What**: Application-layer protocol for multi-agent coordination across independent principals. Explicit intent declaration, structured conflict resolution, Lamport-clock causal watermarking. 21 message types, 3 state machines.
- **Key Pattern**: Coordination over shared state without single-owner assumption. 95% reduction in coordination overhead vs human-mediated baseline.
- **NeoTrix Mapping**: NT-ACT orchestration, NT-GOVERNANCE policy layer, NT-SHIELD trust boundaries

### 2. OASIS — Million-Agent Social Simulation
- **URL**: github.com/camel-ai/oasis (1.1K stars)
- **What**: Scalable social media simulator with up to 1M LLM agents. Dynamic environments, 21+ action spaces, interest-based and hot-score recommendation systems.
- **Key Pattern**: Emergent social dynamics at scale — group polarization, herd effects, information spreading. Scale-dependent behaviors only visible at >10K agents.
- **NeoTrix Mapping**: NT-WORLD perception modeling, NT-FEEL social emotion, NT-MEMORY collective knowledge patterns

### 3. MagiCrew — Enterprise AI Agent Platform
- **URL**: github.com/dtyq/magic (5K stars)
- **What**: Open-source enterprise AI agent platform. Multi-agent collaboration with orchestrator dispatching specialists. Three-tier budget control, human approval gates, sandbox isolation. IM integration + workflow engine.
- **Key Pattern**: Enterprise-grade agent deployment: budget guardrails, approval gates, cost visualization per department/user/task.
- **NeoTrix Mapping**: NT-ACT production deployment, NT-SHIELD approval gates, NT-IO multi-channel interface

### 4. RAGEN-2 — Reasoning Collapse in Agentic RL
- **URL**: github.com/mll-lab-nu/RAGEN (ICML 2026 Oral)
- **What**: Identifies "template collapse" — reasoning looks diverse but is input-agnostic. Uses mutual information (MI) proxy to detect. SNR-aware filtering to prevent collapse during RL training.
- **Key Pattern**: MI-based diagnostic (not entropy) for reasoning quality. Reward-variance-aware prompt filtering prevents generic template generation.
- **NeoTrix Mapping**: NT-CORE reasoning diagnostics, NT-MIND evolution quality monitoring, NT-META meta-cognitive health checks

### 5. Airtop Agent Builder — Compiled Agent Automation
- **URL**: airtop.ai/agent-builder
- **What**: Natural language → compiled automation code. Up to 100x more efficient than LLM-per-step agents. Self-healing repair drafts. Cloud-hosted browser with password vault.
- **Key Pattern**: Compile-once-run-many agent pattern. Eliminates per-step LLM inference cost. Self-healing with draft/repair cycle.
- **NeoTrix Mapping**: NT-ACT tool compilation, NT-PHYSICAL execution engine, NT-REPAIR self-healing

### 6. Flux Attention — Context-Aware Hybrid Attention
- **URL**: arxiv.org/abs/2604.07394
- **What**: Layer-level routing between Full Attention and Sparse Attention based on input context. Lightweight Layer Router in frozen pretrained LLMs. 2.8x prefill speedup, 2.0x decode speedup.
- **Key Pattern**: Context-adaptive attention allocation. Layer-wise routing preserves contiguous memory access for hardware efficiency.
- **NeoTrix Mapping**: NT-CORE E8 reasoning attention, GWT salience routing, NT-PHYSICAL hardware-aware optimization

### 7. veScale-FSDP — Distributed Training at Scale
- **URL**: github.com/volcengine/veScale (1K stars)
- **What**: ByteDance's production FSDP system. RaggedShard for arbitrary sharding granularity + Distributed Buffer for zero-copy access. 7.6K LoC Python, plug-and-play.
- **Key Pattern**: Structure-aware sharding with block-level granularity. Zero-copy via global buffer slices. Production-deployed at ByteDance.
- **NeoTrix Mapping**: NT-MEMORY distributed storage, NT-PHYSICAL compute orchestration, NT-ACT parallel execution

### 8. OrgAgent — Company-Style Multi-Agent Hierarchy
- **URL**: arxiv.org/abs/2604.01020
- **What**: Three-layer hierarchy: governance (planning), execution (task solving), compliance (answer control). 102.73% improvement over flat multi-agent with 74.52% token reduction.
- **Key Pattern**: Hierarchical decomposition improves both effectiveness and cost. Stable skill assignment + controlled information flow + layered verification.
- **NeoTrix Mapping**: NT-GOVERNANCE hierarchy, NT-ACT task decomposition, NT-META layered verification

### 9. AgensFlow — Coordination-Policy Substrate
- **URL**: arxiv.org/abs/2605.27466
- **What**: Online policy learning for multi-agent coordination under partial observability. Inspectable policy graph over skills, models, topology choices. Reward-signal auditability as first-class.
- **Key Pattern**: Coordination as online learning problem. Policy graph makes routing decisions inspectable and auditable.
- **NeoTrix Mapping**: NT-MIND evolution policy, NT-META auditability, GWT attention routing

### 10. Kilo Code — Agentic Engineering Platform
- **URL**: github.com/Kilo-Org/kilocode (trending Sep 2026)
- **What**: All-in-one agentic engineering platform. #1 on OpenRouter, 1.5M+ coders, 25T+ tokens processed. JetBrains integration. Open source.
- **Key Pattern**: Cross-IDE agentic platform with massive scale adoption. Single agent operating across VS Code, JetBrains, terminal.
- **NeoTrix Mapping**: NT-IO multi-interface agent, NT-ACT cross-tool orchestration

## Trending Signals

| Signal | Evidence |
|--------|----------|
| **Agent coordination > single-agent** | MPAC, OrgAgent, AgensFlow all focus on multi-agent coordination |
| **Compiled > prompted agents** | Airtop 100x efficiency gain via compile-once pattern |
| **Enterprise governance critical** | MagiCrew budget gates, MPAC trust boundaries |
| **Reasoning quality > reasoning quantity** | RAGEN-2 MI diagnostics, template collapse detection |
| **Scale reveals emergent behavior** | OASIS 1M agents show polarization only at scale |
| **Hierarchical > flat organizations** | OrgAgent 102% improvement with 74% token reduction |
| **Hardware-aware attention** | Flux Attention layer-level routing for GPU efficiency |
