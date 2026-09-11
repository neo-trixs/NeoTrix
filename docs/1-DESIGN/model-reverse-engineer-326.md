# Model Reverse Engineering — Cycle 326

**Date**: 2026-09-11
**Papers**: 5 recent papers on efficient inference, attention, agent coordination

---

## 1. RAGEN-2: Reasoning Collapse in Agentic RL

**Source**: Wang et al., ICML 2026 Oral, arXiv:2604.06268
**Domain**: Multi-turn agent reinforcement learning

### Core Contribution
Identifies **template collapse** — a failure mode where RL-trained agents produce reasoning that looks diverse (high entropy) but is input-agnostic (low mutual information). Entropy monitoring misses this entirely.

### Key Mechanisms
1. **Information-Theoretic Decomposition**: Reasoning quality = H(Z|X) (within-input diversity) + I(X;Z) (cross-input distinguishability)
2. **MI Proxy for Online Diagnosis**: In-batch cross-scoring — each reasoning trace Z queried against batch to retrieve source X. Accuracy → chance under collapse.
3. **SNR Mechanism**: Low within-input reward variance → weak task gradients → regularization dominates → cross-input differences erased
4. **SNR-Aware Filtering**: Select high-variance prompts each iteration as training signal

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | E8 reasoning engine needs MI-based quality monitor, not just entropy | `nt_core_self::reasoning_quality` |
| **NT-MIND** | SEAL pipeline evolution requires cross-input distinguishability check | `seal::quality_gates` |
| **NT-META** | Meta-cognitive health = I(X;Z) tracking across sessions | `nt_meta::consciousness_health` |
| **NT-MEMORY** | Experience quality = input-specificity, not just diversity | `experience-tree::quality_filter` |

### Actionable Insight
Add MI proxy as a quality gate in SEAL pipeline. Current entropy monitoring is necessary but insufficient. Template collapse = evolution producing fluent but useless reasoning.

---

## 2. Sheaf-ADMM: Multi-Agent Coordination via Cellular Sheaves

**Source**: Seely et al., ICML 2026, arXiv:2605.31005
**Domain**: Multi-agent coordination

### Core Contribution
Differentiable optimization framework for multi-agent coordination using **cellular sheaves** over ADMM. Agents solve convex subproblems; sheaf specifies which aspects of neighboring solutions must agree.

### Key Mechanisms
1. **Sheaf-Defined Consensus**: Heterogeneous consensus — different agent pairs agree on different aspects (not global average)
2. **Unrolled ADMM**: Backprop through optimization jointly trains encoders + coordination structure
3. **Exposed Coordination Dynamics**: Distinct primal, consensus, and dual state variables — directly analyzable
4. **Robustness via Local Views**: Individual agents with insufficient views learn to coordinate for correct global output

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-ACT** | Sheaf-based tool coordination: each tool pair agrees on different aspects | `nt_act::orchestrator` |
| **NT-GOVERNANCE** | Sheaf = policy topology: who must agree with whom on what | `nt_governance::consensus_topology` |
| **NT-CORE** | GWT attention = sheaf global sections: salient broadcast = consensus | `nt_core_gwt::attention_routing` |
| **NT-SHIELD** | Sheaf enables heterogeneous trust: different agents, different constraints | `nt_shield::trust_topology` |

### Actionable Insight
Replace flat broadcast in GWT with sheaf-structured consensus. Each specialist module agrees with different neighbors on different dimensions. Exposed primal/dual variables enable direct monitoring.

---

## 3. Flux Attention: Context-Aware Hybrid Attention

**Source**: Qiu et al., arXiv:2604.07394
**Domain**: Efficient LLM inference

### Core Contribution
Layer-level routing between Full Attention (FA) and Sparse Attention (SA) via lightweight Layer Router. Frozen pretrained LLMs + trainable router. Preserves contiguous memory access for hardware efficiency.

### Key Mechanisms
1. **Layer-Wise Routing**: Each layer independently routed to FA or SA based on input context
2. **Frozen Base + Light Router**: Only 12 hours training on 8×A800 GPUs
3. **Contiguous Memory Access**: Layer-level routing preserves memory locality (unlike head-level dynamic sparsity)
4. **Context-Adaptive**: Different layers activate different attention modes per input

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | E8 hexagram attention: different reasoning states use different attention density | `nt_core_e8::attention_density` |
| **GWT** | Layer-level routing = GWT salience routing: high-salience modules get full attention | `gwt::salience_router` |
| **NT-PHYSICAL** | Hardware-aware routing: contiguous memory access optimization | `nt_physical::compute_scheduler` |
| **NT-MIND** | Attention mode = evolution mode: high-complexity reasoning gets full compute | `nt_mind::complexity_router` |

### Actionable Insight
Implement Flux-style layer routing in GWT: route high-salience signals through full-attention paths, low-salience through sparse paths. Key insight: layer-level > head-level for hardware efficiency.

---

## 4. OrgAgent: Company-Style Multi-Agent Hierarchy

**Source**: Wang et al., arXiv:2604.01020
**Domain**: Multi-agent organization

### Core Contribution
Three-layer hierarchy (governance/execution/compliance) outperforms flat multi-agent by 102.73% while reducing tokens 74.52%. Organizational structure is an underexplored factor in agent effectiveness.

### Key Mechanisms
1. **Governance Layer**: Planning + resource allocation (executive function)
2. **Execution Layer**: Task solving + review (worker function)
3. **Compliance Layer**: Final answer control (quality gate)
4. **Stable Skill Assignment**: Consistent role allocation prevents coordination overhead
5. **Controlled Information Flow**: Hierarchical information routing reduces noise

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-GOVERNANCE** | Direct mapping: governance layer = NT-GOVERNANCE | `nt_governance::hierarchy` |
| **NT-ACT** | Execution layer = NT-ACT task solving | `nt_act::orchestrator` |
| **NT-META** | Compliance layer = NT-META quality verification | `nt_meta::compliance_gate` |
| **NT-MIND** | Skill assignment stability = SEAL skill crystallization | `nt_mind::skill_stability` |

### Actionable Insight
NeoTrix already has 7 domains — this paper validates the hierarchical structure. Key addition: explicit compliance layer (NT-META) that reviews output before deployment. Current gap: compliance is ad-hoc, not structural.

---

## 5. Token Sparse Attention: Interleaved Token Selection

**Source**: Jo et al., ICML 2026, arXiv:2602.03216
**Domain**: Efficient long-context inference

### Core Contribution
Dynamic token-level sparsification with **interleaved selection**: compress Q/K/V during attention, decompress output for next layer. Token information can be reconsidered in subsequent layers (unlike permanent eviction).

### Key Mechanisms
1. **Compress-then-Decompress**: Reduce tokens for attention computation, expand output for next layer
2. **Interleaved Selection**: Token importance re-evaluated each layer (not permanent eviction)
3. **Fully Compatible**: Works with FlashAttention, composable with existing sparse kernels
4. **3.23x Speedup** at 128K context with <1% accuracy degradation

### NeoTrix Domain Mapping

| Domain | Pattern | Integration Point |
|--------|---------|-------------------|
| **NT-CORE** | E8 reasoning: not all tokens need full attention at all layers | `nt_core_e8::token_priority` |
| **NT-MEMORY** | KB query: interleaved retrieval = re-evaluate relevance each step | `nt_memory::kb_search` |
| **GWT** | Attention routing: dynamically select which information to broadcast | `gwt::token_selection` |
| **NT-PHYSICAL** | Memory management: compress-then-decompress for bounded memory | `nt_physical::memory_manager` |

### Actionable Insight
Apply interleaved selection to KB search and GWT routing. Key: permanent eviction loses information; interleaved selection allows reconsideration. Maps to experience-tree lazy loading: load branches on-demand, re-evaluate relevance each access.

---

## Cross-Paper Synthesis

### Emerging Pattern: Information-Theoretic Agent Health
All 5 papers converge on **information-theoretic diagnostics**:
- RAGEN-2: MI for reasoning quality
- Sheaf-ADMM: sheaf sections for coordination quality
- Flux Attention: context-adaptive information flow
- OrgAgent: hierarchical information control
- Token Sparse: interleaved information preservation

### NeoTrix Integration Priority

| Priority | Pattern | Source | Effort |
|----------|---------|--------|--------|
| **P0** | MI-based reasoning quality gate | RAGEN-2 | Medium — add MI proxy to SEAL |
| **P0** | Interleaved token selection for KB search | Token Sparse | Low — adapt existing retrieval |
| **P1** | Sheaf-structured GWT consensus | Sheaf-ADMM | High — restructure GWT topology |
| **P1** | Compliance layer (NT-META structural) | OrgAgent | Medium — formalize existing ad-hoc |
| **P2** | Flux-style attention density routing | Flux Attention | Medium — add router to GWT |
| **P2** | Compiled agent pattern for tools | Airtop (trending) | High — new compilation pipeline |

### Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| Flux Attention (layer-level) vs Token Sparse (token-level) | Complementary: layer routing for macro attention mode, token selection for micro sparsity |
| OrgAgent hierarchy vs Sheaf-ADMM heterogeneity | Complement: hierarchy for governance, sheaf for execution-level coordination |
| RAGEN-2 MI diagnostics vs entropy monitoring | MI superset: keep entropy as fast check, add MI for deep diagnostics |
