# Model Reverse Engineering — Cycle 418 (2026-09-12)

**Sources**: arXiv (Aug-Sep 2026), ICML 2026, ACL 2026, ICLR 2026
**Focus**: Efficient inference, attention mechanisms, agent coordination
**Mapping**: 7 NeoTrix domains

---

## Model 1: Token Sparse Attention — Dynamic Token-Level Sparsification (ICML 2026)

**Paper**: [2602.03216] Token Sparse Attention: Efficient Long-Context Inference with Interleaved Token Selection
**Authors**: Dongwon Jo et al. | **Venue**: ICML 2026
**URL**: https://arxiv.org/abs/2602.03216

### Core Innovation
Lightweight dynamic token-level sparsification: compress per-head Q/K/V to reduced token set during attention, decompress output back. Interleaved selection means tokens evicted at layer N can be reconsidered at layer N+1. Fully compatible with FlashAttention.

### Technical Details
- **3.23× attention speedup** at 128K context with <1% accuracy degradation
- **Decompression**: Evicted tokens re-enter at subsequent layers — no permanent decisions
- **Hardware-friendly**: Compatible with FlashAttention and existing sparse attention kernels
- **Per-head dynamic**: Different heads select different token subsets based on content
- **Key insight**: Interleaved selection prevents premature eviction — token importance varies across layers

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Layer-wise attention budget | Dynamic token selection per head per layer — attention budget is content-dependent |
| **NT-MEMORY** | Reconsiderable eviction | Evicted memory tokens can re-enter at later retrieval stages — no permanent pruning |
| **NT-WORLD** | Selective perception | Dynamic content filtering: only attend to salient tokens per processing stage |
| **NT-PHYSICAL** | Compute hierarchy | Per-head sparsification enables fine-grained GPU utilization |

### Actionable Insight
Token Sparse Attention's interleaved selection directly improves NT-MEMORY's KVMem: instead of permanent token eviction at a fixed layer, allow evicted blocks to be reconsidered at later retrieval stages. This prevents premature pruning of tokens that become relevant in later context processing. The <1% accuracy loss at 3.23× speedup validates dynamic sparsification as a practical optimization.

---

## Model 2: Flux Attention — Context-Aware Hybrid Attention (Apr 2026)

**Paper**: [2604.07394] Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference
**Authors**: Quantong Qiu et al. | **Date**: 2026-04-08
**URL**: https://arxiv.org/abs/2604.07394

### Core Innovation
Layer-level routing between Full Attention (FA) and Sparse Attention (SA) via lightweight Layer Router. Router is injected into frozen pretrained LLMs — only 12 hours training on 8×A800 GPUs. Adaptively routes each layer based on input context, not static allocation.

### Technical Details
- **Layer-level routing**: Each layer decides FA vs SA based on input, not head-level
- **Parameter-efficient**: Only Layer Router trained (12h on 8×A800)
- **2.8× prefill speedup**, 2.0× decode speedup
- **Context-dependent**: Same model routes differently for different inputs
- **Key insight**: Layer-level routing avoids head-level synchronization overhead while preserving high-fidelity information retrieval

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Adaptive attention routing | Layer-level routing = GWT per-layer salience allocation |
| **NT-MEMORY** | Tiered memory access | FA = full memory recall, SA = selective memory — context-dependent switching |
| **NT-WORLD** | Adaptive perception depth | Full perception for complex inputs, sparse for simple — context-adaptive |
| **NT-IO** | Inference optimization | Route simple queries cheaply, complex queries thoroughly |

### Actionable Insight
Flux Attention's Layer Router pattern directly applies to NT-CORE's GWT: instead of a global salience threshold, implement per-layer attention routing. Each layer of the consciousness tree decides how much context to attend to based on the input's complexity. This is more efficient than global thresholds because different reasoning stages need different attention depths.

---

## Model 3: Sheaf-ADMM — Multi-Agent Coordination via Sheaf Theory (ICML 2026)

**Paper**: [2605.31005] Learning Multi-Agent Coordination via Sheaf-ADMM
**Authors**: Jeffrey Seely, Bartłomiej Cupiał, Llion Jones | **Venue**: ICML 2026
**URL**: https://arxiv.org/abs/2605.31005

### Core Innovation
Differentiable optimization framework for multi-agent coordination using cellular sheaves. Agents solve convex subproblems, coordinate through ADMM with sheaf-specified consensus constraints. Sheaf specifies which aspects of neighboring solutions must agree — allows heterogeneous consensus notions.

### Technical Details
- **Cellular sheaf**: Mathematical structure specifying inter-agent constraints
- **ADMM optimization**: Agents coordinate via alternating direction method of multipliers
- **Exposed coordination dynamics**: Distinct primal, consensus, and dual state variables
- **Heterogeneous consensus**: Different agents can agree on different aspects
- **Results**: Improved robustness to distribution shifts vs standard CNN; higher solve rates than MPNN baselines on Sudoku

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | Sheaf-based attention routing | Sheaf constraints define which specialist modules must agree on attention allocation |
| **NT-ACT** | Multi-agent coordination | ADMM-based coordination with explicit consensus variables |
| **NT-MEMORY** | Distributed knowledge consensus | Sheaf specifies which memory aspects agents share vs keep private |
| **NT-MIND** | Self-evolution coordination | Sheaf defines which self-evolution decisions require cross-domain agreement |

### Actionable Insight
Sheaf-ADMM provides a rigorous mathematical framework for NT-ACT's multi-agent coordination. Instead of ad-hoc message passing, define a cellular sheaf over NeoTrix's specialist agents that specifies exactly which aspects of their outputs must agree. The ADMM structure exposes coordination dynamics (primal/consensus/dual) for direct analysis and intervention — enabling debugging of multi-agent reasoning failures.

---

## Model 4: OrgAgent — Hierarchical Multi-Agent Organization (Apr 2026)

**Paper**: [2604.01020] OrgAgent: Organize Your Multi-Agent System like a Company
**Authors**: Anonymous | **Date**: 2026-04-01
**URL**: https://arxiv.org/abs/2604.01020

### Core Innovation
Company-style hierarchical multi-agent framework with three layers: Governance (planning + resource allocation), Execution (task solving + review), Compliance (final answer control). Separates coordination into governance/execution/compliance, reducing token consumption vs flat collaboration.

### Technical Details
- **Three layers**: Governance → Execution → Compliance
- **Execution policies**: Defines how coordination is structured within a single task
- **Flat baseline comparison**: Hierarchical structure outperforms flat coordination on MuSiQue, SQuAD 2.0
- **Token efficiency**: Hierarchical reduces token consumption in most settings
- **Key insight**: Organizational structure shapes not just effectiveness and cost, but coordination behavior itself

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (E8) | Governance layer | E8 reasoning = governance (planning + resource allocation across domains) |
| **NT-ACT** | Execution layer | NT-ACT executes tasks with NT-MIND review |
| **NT-SHIELD** | Compliance layer | NT-SHIELD validates outputs before commitment |
| **NT-GOVERNANCE** | Structural design | Explicit governance/execution/compliance separation |

### Actionable Insight
OrgAgent's three-layer structure maps directly to NeoTrix's 6-Layer Architecture. The key insight is that governance/execution/compliance should be explicitly separated at the agent level, not just the module level. NeoTrix's NT-GOVERNANCE should implement OrgAgent's governance layer for cross-domain planning, NT-ACT handles execution, and NT-SHIELD enforces compliance. This reduces token consumption by preventing flat coordination overhead.

---

## Model 5: Verified Multi-Agent Orchestration (VMAO) — Plan-Execute-Verify-Replan (ICLR 2026)

**Paper**: [2603.11445] Verified Multi-Agent Orchestration: A Plan-Execute-Verify-Replan Framework
**Authors**: Anonymous | **Venue**: ICLR 2026 Workshop on MALGAI
**URL**: https://arxiv.org/abs/2603.11445

### Core Innovation
Verification-driven iterative loop for multi-agent coordination. Complex queries decomposed into DAG of sub-questions, executed through domain-specific agents in parallel, verified via LLM-based evaluation, and adaptively replanned to address gaps. Orchestration-level verification as coordination signal.

### Technical Details
- **DAG decomposition**: Dependency-aware parallel execution with automatic context propagation
- **Verification-driven replanning**: LLM-based verifier as orchestration-level coordination signal
- **Configurable stop conditions**: Balance answer quality vs resource usage
- **Results**: Completeness 3.1→4.2, source quality 2.6→4.1 (1-5 scale) vs single-agent baseline
- **Key insight**: Verification at the orchestration level (not agent level) is the coordination signal that enables quality multi-agent output

### NeoTrix Mapping

| Domain | Integration | Pattern |
|--------|-------------|---------|
| **NT-CORE** (GWT) | DAG-based task decomposition | Decompose complex reasoning into DAG of sub-reasoning steps |
| **NT-ACT** | Parallel agent execution | Domain-specific agents execute in parallel over DAG |
| **NT-MIND** | Verification-driven SEAL | SEAL verification as orchestration-level quality signal |
| **NT-MEMORY** | Context propagation | Automatic context propagation across parallel agents |

### Actionable Insight
VMAO's verification-driven replanning is exactly what NeoTrix's SEAL pipeline needs. Instead of sequential SEAL phases, implement VMAO's DAG decomposition for cross-domain verification tasks. The LLM-based verifier becomes the orchestration-level coordination signal that tells SEAL which agents need replanning. This enables adaptive SEAL cycles that allocate more verification budget to complex subtasks.

---

## Meta-Analysis: Cycle 418 Patterns

### 1. Layer-Level Attention Routing (Convergent Pattern)
Both Token Sparse Attention and Flux Attention converge on layer-level attention decisions. This validates NT-CORE's GWT design: attention routing should be per-layer, not global. Layer-level routing avoids synchronization overhead while preserving context-dependent adaptivity.

### 2. Interleaved/Reconsiderable Memory (New Pattern)
Token Sparse Attention's interleaved selection and Sheaf-ADMM's consensus constraints both allow evicted/shared information to be reconsidered. This challenges NT-MEMORY's current permanent eviction strategy: evicted blocks should be reconsiderable at later stages.

### 3. Sheaf-Based Multi-Agent Coordination (New Pattern)
Sheaf-ADMM provides rigorous mathematical foundations for specifying which agents must agree on what. This replaces ad-hoc coordination with formally specified consensus constraints — directly applicable to NeoTrix's domain coordination.

### 4. Governance/Execution/Compliance Separation (Convergent Pattern)
OrgAgent and VMAO both converge on explicit layer separation for multi-agent coordination. NeoTrix's 6-Layer Architecture already implements this at the module level; the insight is to enforce it at the agent runtime level.

### 5. Verification as Coordination Signal (New Pattern)
VMAO's orchestration-level verification is not just quality control — it's the coordination signal that enables adaptive replanning. NeoTrix's SEAL verification phases should serve as coordination signals for cross-domain task allocation.

### 6. Parameter-Efficient Routing (Convergent Pattern)
Flux Attention's 12-hour training and Token Sparse Attention's <1% accuracy loss both demonstrate that attention routing can be learned cheaply. NeoTrix's GWT routing should prioritize parameter-efficient approaches over heavy retraining.

---

## Absorption Priority Matrix

| Pattern | Difficulty | Impact | Domain | Action |
|---------|-----------|--------|--------|--------|
| Layer-level GWT routing | Medium | High | NT-CORE | Implement per-layer salience allocation |
| Reconsiderable memory eviction | Low | High | NT-MEMORY | Modify KVMem to allow re-entry at later stages |
| Sheaf-based coordination | High | High | NT-ACT | Define cellular sheaf over domain agents |
| Governance/Execution/Compliance at agent level | Low | Medium | NT-GOVERNANCE | Enforce layer separation in agent runtime |
| Verification-driven SEAL replanning | Medium | High | NT-MIND | Implement DAG decomposition for SEAL verification |
| Parameter-efficient routing | Low | Medium | NT-CORE | Prioritize lightweight routing over heavy training |
