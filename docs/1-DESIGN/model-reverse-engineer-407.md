# Model Reverse Engineering — Cycle 407

**Date**: 2026-09-12
**Focus**: Efficient inference, attention mechanisms, agent coordination
**Sources**: arXiv, ICLR 2026, ACL 2026, ACL Anthology

---

## Paper 1: Token Sparse Attention (ICML 2026)

**Paper**: "Token Sparse Attention: Efficient Long-Context Inference with Interleaved Token Selection"
**Authors**: Dongwon Jo et al. | **Venue**: ICML 2026 | **arXiv**: 2602.03216

### Core Mechanism
Dynamic token-level sparsification that compresses per-head Q/K/V to reduced token sets during attention, then decompresses output back to original sequence. Token information can be reconsidered in subsequent layers (unlike permanent eviction methods).

### Key Innovation
- **Interleaved Selection**: Tokens are dynamically selected per-head per-layer, not permanently evicted
- **Compress-Decompress Cycle**: Q/K/V compressed → attention computed → output decompressed back to full sequence
- **Flash Attention Compatible**: Works with dense attention implementations including Flash Attention
- **3.23x attention speedup** at 128K context with <1% accuracy degradation

### Pattern Extracted
**Dynamic Token Budget Allocation**: Each attention head maintains a dynamic token budget that adjusts based on context relevance. Tokens are "reconsidered" each layer rather than permanently evicted.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT Attention** | SelectiveState salience scoring | Dynamic token importance per broadcast cycle |
| **KVMem** | KV cache optimizer | Interleaved token selection for paged KV management |
| **ConsciousnessTree** | Layer-wise attention budget | Each "branch" (domain) gets dynamic token budget |
| **HyperCube** | VSA embedding selection | Dynamic relevance scoring for concept vectors |

**Actionable**: Implement dynamic token budget per GWT broadcast cycle — salience scoring determines which memories/tools get attention bandwidth, with reconsideration each cycle.

---

## Paper 2: Flux Attention (April 2026)

**Paper**: "Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference"
**Authors**: Quantong Qiu et al. | **arXiv**: 2604.07394

### Core Mechanism
Context-aware hybrid attention that dynamically routes each layer to Full Attention (FA) or Sparse Attention (SA) based on input context. Lightweight Layer Router added to frozen pretrained LLMs.

### Key Innovation
- **Layer-Wise Routing**: Each layer independently routes to FA or SA based on input context
- **Preserves Contiguous Memory Access**: Unlike head-level sparsity, layer-level maintains hardware-friendly memory patterns
- **Parameter-Efficient**: Only 12 hours training on 8x A800 GPUs
- **2.8x prefill speedup**, 2.0x decode speedup

### Pattern Extracted
**Context-Adaptive Compute Allocation**: A lightweight router inspects input context and allocates compute budget (full vs sparse) per layer. Routes are input-dependent, not static.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **GWT** | Salience-based routing | Router decides full-threshold vs sparse processing per task |
| **Ordered Backend Router** | Provider selection | Context-dependent routing to fast/capable/expensive providers |
| **SEAL Pipeline** | Phase allocation | Dynamic phase budget based on task complexity |
| **Cost-Aware Routing (A1)** | Model selection | Route to FA-equivalent (expensive) or SA-equivalent (cheap) models |

**Actionable**: Implement Flux-style layer router for GWT — lightweight classifier routes tasks to "full processing" (expensive model) or "sparse processing" (cheap model) based on task complexity signal.

---

## Paper 3: SparDA — Sparse Decoupled Attention (June 2026)

**Paper**: "SparDA: Sparse Decoupled Attention for Efficient Long-Context LLM Inference"
**Authors**: Yaosheng Fu et al. (Song Han group) | **arXiv**: 2606.04511

### Core Mechanism
Fourth per-layer projection ("Forecast") alongside Q/K/V. Forecast predicts which KV blocks the next layer needs, enabling lookahead selection that overlaps CPU-to-GPU prefetch with current-layer execution.

### Key Innovation
- **Decoupled Forecast Head**: Forecast is separate from attention query, reducing selection overhead
- **One Forecast Head per GQA Group**: Reduces multi-head selector overhead
- **<0.5% parameter overhead**: Only trains Forecast projections
- **1.7x decode speedup**, **5.3x higher throughput** via larger batch sizes

### Pattern Extracted
**Predictive Prefetch via Forecast Projection**: A lightweight "forecast" module predicts future resource needs (KV blocks), enabling overlapping of prefetch with computation. Decoupled from main computation path.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **ConsciousnessTree** | Cycle prediction | Forecast which domains will need attention next cycle |
| **Experience-Tree** | Branch prefetch | Predict which KB branches will be needed, prefetch from disk |
| **HeartbeatAggregator** | Health prediction | Forecast system health trajectory for proactive repair |
| **NT-NEXUS** | Cross-session prediction | Forecast which past sessions will be relevant to current task |

**Actionable**: Implement "Forecast Projection" for NT-NEXUS — lightweight module predicts which cross-session memories will be needed, prefetches from KB before task execution requires them.

---

## Paper 4: Internet of Agentic AI (IoAI) (June 2026)

**Paper**: "The Internet of Agentic AI: Communication, Coordination, and Collective Intelligence at Scale"
**Authors**: Quanyan Zhu | **arXiv**: 2606.12835

### Core Mechanism
Vision of an open ecosystem where heterogeneous agents discover one another, negotiate responsibilities, exchange context, invoke tools, and execute workflows across cloud/edge/device/organizational environments.

### Key Research Challenges Identified
1. **Controlled Emergence** — steering agent collectives without micromanagement
2. **Semantic Interoperability** — agents with different architectures communicating meaningfully
3. **Secure Identity** — agent identity and trust in open ecosystems
4. **Incentive-Compatible Coordination** — agents cooperating without exploitation
5. **Resource-Aware Orchestration** — compute/memory/bandwidth constraints across agents
6. **Governance** — policy enforcement at ecosystem scale

### Pattern Extracted
**Agent Ecosystem Topology**: Agents form a network with discovery, negotiation, context exchange, and trust layers. Not just "multi-agent" but "agent internet" with protocol-level coordination.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **7 Domains** | Agent ecosystem | Each NT-* domain as autonomous agent in IoAI ecosystem |
| **EventBus** | Communication protocol | Inter-domain message passing with semantic routing |
| **NT-SHIELD** | Trust architecture | Agent identity verification + capability attestation |
| **NT-GOVERNANCE** | Policy enforcement | Constitution-level rules for agent interaction |
| **GWT** | Resource-aware orchestration | Attention routing as resource allocation across agent network |

**Actionable**: Model NT-* domains as IoAI agents with discovery/negotiation/trust protocols. EventBus becomes the communication substrate. NT-SHIELD provides identity + trust layer.

---

## Paper 5: Emergent Coordination in Multi-Agent LMs (ICLR 2026)

**Paper**: "Emergent Coordination in Multi-Agent Language Models"
**Authors**: Riedl et al. | **Venue**: ICLR 2026 | **arXiv**: 2510.05174

### Core Mechanism
Information-theoretic framework to test whether multi-agent LLM systems show higher-order structure (synergy, complementarity) beyond mere aggregation of individual agents.

### Key Findings
1. **Control condition**: Agents show temporal synergy but little coordinated alignment
2. **Persona assignment**: Introduces stable identity-linked differentiation
3. **Theory-of-Mind prompt**: "Think about what other agents might do" produces goal-directed complementarity
4. **Paralysis under coordination ambiguity**: Larger reasoning models (Qwen3) can fail when coordination signals are ambiguous
5. **Steerable emergence**: Prompt design can shift systems from "mere aggregates" to "higher-order collectives"

### Pattern Extracted
**Prompt-Steered Collective Intelligence**: Multi-agent systems can be steered from aggregation to genuine collective intelligence through three interventions: (1) identity assignment, (2) Theory-of-Mind prompting, (3) structured coordination signals.

### NeoTrix Mapping

| Component | Integration Point | Pattern |
|-----------|-------------------|---------|
| **ConsciousnessTree** | Branch identity | Each branch (domain) gets distinct identity + ToM prompt |
| **GWT** | Coordination signals | Salience broadcasting creates structured coordination |
| **NT-CORE (E8)** | Collective reasoning | Hexagram states as coordination signal topology |
| **SEAL Pipeline** | Evolution steering | Persona-driven evolution: each domain has distinct "personality" |
| **Dual Specialization** | Weapon Set identity | Two specialization modes = two "personas" for attention routing |

**Actionable**: Implement Theory-of-Mind prompting in GWT — each specialist module receives not just "what to do" but "what other modules might do." Identity-linked differentiation prevents homogenization of specialist responses.

---

## Cross-Paper Synthesis

### Meta-Pattern: Dynamic Resource Allocation

| Paper | Resource | Allocation Mechanism |
|-------|----------|---------------------|
| Token Sparse Attention | Tokens per head | Dynamic per-layer budget |
| Flux Attention | Compute per layer | Context-adaptive router |
| SparDA | KV blocks | Forecast-driven prefetch |
| IoAI | Agent bandwidth | Resource-aware orchestration |
| Emergent Coordination | Attention/information | Prompt-steered complementarity |

**NeoTrix Unified Pattern**: Every layer of NeoTrix should have a **dynamic resource allocator** — from token-level (GWT attention) to module-level (SEAL pipeline) to ecosystem-level (NT-* domain coordination).

### Meta-Pattern: Forecast-Prefetch-Execute

All five papers share a temporal pattern:
1. **Forecast** what will be needed (SparDA Forecast head, IoAI discovery, Emergent Coordination ToM)
2. **Prefetch** resources before they're required (SparDA CPU→GPU overlap, Experience-Tree lazy loading)
3. **Execute** with dynamic allocation (Token Sparse Attention budget, Flux Attention routing)

**NeoTrix Integration**: NT-NEXUS should implement Forecast→Prefetch→Execute for cross-session memory. ConsciousnessTree should forecast next-cycle domain needs.

---

## Implementation Priorities

| Priority | Paper | NeoTrix Component | Effort |
|----------|-------|-------------------|--------|
| P0 | Flux Attention | GWT context-adaptive router | Medium |
| P0 | SparDA | NT-NEXUS forecast projection | Medium |
| P1 | Token Sparse Attention | GWT dynamic token budget | High |
| P1 | Emergent Coordination | ToM prompting for NT-* domains | Low |
| P2 | IoAI | Agent ecosystem topology | High |
