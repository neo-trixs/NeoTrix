# Model Reverse Engineering — Cycle 330

**Date**: 2026-09-11
**Focus**: Efficient inference, speculative execution, agent coordination, context compression, self-evolving topologies
**Previous cycles**: 318–329 (excluded)

---

## Paper 1: SpecBox — Speculative Sandbox Scheduling for Efficient LLM Agent Serving

**URL**: https://arxiv.org/abs/2607.23933
**Authors**: Zhang, Wo, Wang, Sun, Zhang, Yuan, Li, Hu, Zomaya, Yang
**Date**: 2026-07-27

### Core Idea
LLM agents invoke isolated sandboxes via MCP, creating cold-start vs. resource-waste tension. SpecBox resolves this with three speculative mechanisms:
1. **Intent-Aware Prewarming**: Scans streaming token output mid-generation to predict which sandbox the agent intends to call before the tool-call JSON is complete. Keyword Router (token-by-token matching, 323ms latency, 95% coverage) + Semantic Router (TF-IDF over tool descriptions, 2.1ms latency, Micro-F1 0.970). Union policy reduces average waiting latency to 124.45ms.
2. **Stochastic Sandbox Prefetching**: Sandbox Dependency Graph (SDG) modeled as first-order Markov process from historical trajectories. Budgeted prefetch under cost threshold (λ=5s) and probability threshold (τ=0.6). Extends prewarming across sequential agent steps.
3. **Reuse-Aware Data Transmission**: Semantic result cache (cosine similarity ≥ 0.8) prunes redundant invocations. Out-of-band shared-memory transport for large artifacts — zero-copy transfer achieves 5.97ms vs 1,873ms for JSON-RPC at 1GB payload (313x reduction).

### Key Results
- P99 latency: 2.9x reduction vs on-demand baseline (88.7s vs 257.2s at QPS=20)
- Peak memory: 45.9% reduction vs permanently reserved sandboxes
- Prewarming hit rate: 97.9% — fewer than 3% of tool calls suffer cold start
- 97.9% prewarming hit rate across 200 multi-turn agent trajectories

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-ACT** | Intent-aware prewarming → NT-ACT tool execution pipeline. SpecBox's streaming intent detection applies to our tool call generation — predict which tool the agent needs before the call is complete |
| **NT-SHIELD** | Semantic result cache → NT-ACT tool caching with deduplication. Reuse-aware execution prevents redundant tool invocations |
| **NT-PHYSICAL** | Shared-memory transport → NT-PHYSICAL zero-copy data transfer for large artifacts between modules |
| **NT-IO** | MCP integration patterns → SpecBox's sandbox dependency graph for MCP server lifecycle management |

### NeoTrix Application
**Speculative tool execution**: Our NT-ACT CapabilityRouter could adopt SpecBox's intent-aware prewarming pattern. As the LLM generates a tool call, scan the streaming tokens to predict which capability node will be needed. Begin pre-loading the capability's context (KB namespace, skill definitions, tool schemas) before the call is complete. This overlaps capability preparation with LLM generation.

**Sandbox dependency graph for MCP**: For NT-IO MCP server management, build a Markov transition model of tool call sequences. When tool A completes, speculatively warm tool B's sandbox based on historical transition probabilities. Budget-constrained: only prefetch expensive sandboxes (λ threshold) with high transition probability (τ threshold).

---

## Paper 2: QueenBee Planner — Skill-Evolving Communication Topologies

**URL**: https://doi.org/10.48550/arxiv.2606.27492
**Authors**: Tian, Yao, Cui
**Date**: 2026-06-25

### Core Idea
Treats inter-agent communication topology as a retrievable and self-improving design skill. An outer LLM planner learns to generate temporal communication DAGs — who sends to whom, in which round, who merges, who emits. Execution traces distilled into evidence-backed design rules with three actions: Preserve, Modify, Avoid.

Anti-self-devolution safeguards:
- **Held-out acceptance gates**: Prevent lucky runs from becoming policy
- **Variance-aware credit**: Distinguish skill quality from noise
- **Motif-level attribution**: Track which structural patterns contribute to success
- **Transfer trust**: Validate patterns generalize across tasks
- **Insight falsification**: Actively test whether learned rules are true
- **Structural deduplication**: Prevent redundant rules from accumulating

### Key Results
- Count-Frequency: RMSE reduced from 12.53 (strongest fixed topology) to 7.87
- Simultaneously reduces messages, model calls, and token cost
- Self-evolved graphs outperform both fixed topologies and cold generation

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MIND (SEAL)** | Skill-evolving topology → SEAL pipeline stage transitions evolve as design skills. The Preserve/Modify/Avoid action taxonomy maps to SEAL feedback categories |
| **NT-ACT** | Communication DAGs → NT-ACT agent coordination graphs. Dynamic topology generation for multi-agent workflows |
| **NT-GOVERNANCE** | Anti-self-devolution safeguards → quality gates preventing self-deception. Held-out acceptance gates align with our ConvergeCheck |
| **NT-CORE (E8)** | Temporal DAG generation → E8 hexagram exploration of communication structures |

### NeoTrix Application
**Self-evolving SEAL topology**: QueenBee's approach of distilling execution traces into Preserve/Modify/Avoid design rules directly applies to our SEAL pipeline. Currently, SEAL stages are fixed (Soil→Roots→Trunk→Branches→Fruits→Core). QueenBee suggests the stage transitions themselves should evolve based on execution outcomes. The anti-self-devolution safeguards are critical — they prevent our ConsciousnessTree from mistaking lucky runs for genuine capability.

**Communication topology as skill**: The "topology as retrievable skill" pattern means multi-agent coordination structures should be stored, versioned, and retrieved — not hardcoded. Our DomainBridge could retrieve optimal coordination topologies from KB based on task type.

---

## Paper 3: MetaInfer — LLM-as-Compiler Inference Engine Generator

**URL**: https://arxiv.org/abs/2607.12875
**Authors**: Miao, Wang, Mi
**Date**: 2026-07-14

### Core Idea
Users specify only runtime constraints (model family, hardware, quantization, parallelization). An LLM-driven multi-agent collaboration system, coupled with a Contract Knowledge Base (CKB), automatically generates a compact customized inference framework. The key insight: eliminate complex manual abstractions by letting the LLM reason about optimization constraints.

Three components:
1. **Contract Knowledge Base (CKB)**: Structured knowledge about model families, hardware capabilities, quantization schemes, optimization kernels
2. **Multi-Agent Generation**: LLM agents collaborate to decompose constraints, select optimizations, and generate code
3. **Continuous Knowledge Feedback Loop**: Generation constraints → validation feedback → knowledge consolidation → improved future generation

### Key Results
- Runnable customized inference frameworks generated from explicit knowledge constraints
- Zero-reference constraint satisfaction on CKB-covered targets
- Continuous closed-loop improvement: generation → validation → knowledge update

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-MIND** | LLM-as-Compiler → SEAL pipeline as self-optimizing compiler. The constraint→generation→validation→feedback loop is exactly SEAL's evolution loop |
| **NT-IO** | CKB (Contract Knowledge Base) → our KB schema for model/hardware constraints. Structured knowledge about what optimizations work on what hardware |
| **NT-PHYSICAL** | Hardware-aware inference generation → model adaptation for constrained hardware |
| **NT-CORE** | Multi-agent generation decomposition → E8 reasoning for constraint satisfaction |

### NeoTrix Application
**SEAL as LLM-compiler**: MetaInfer's architecture maps directly to our SEAL pipeline. SEAL currently runs fixed stages (explore→distill→self-test→absorb). MetaInfer shows SEAL could become a "compiler" that takes task constraints and generates optimized execution plans. The CKB pattern suggests our KB should store structured knowledge about what configurations work for what tasks — not just experiences but constraint-satisfaction rules.

**Knowledge feedback loop**: MetaInfer's continuous feedback (generation → validation → knowledge update) is our experience-tree absorption but at the infrastructure level. Every SEAL cycle should update the CKB with what worked and what didn't, enabling future cycles to start from accumulated constraint knowledge.

---

## Paper 4: Parason — Parallel Reasoning with Trial Parallelism

**URL**: https://arxiv.org/pdf/2608.24658
**Authors**: Not fully specified in search results
**Date**: 2026-08-25

### Core Idea
Two forms of parallel reasoning in LLM inference:
1. **Subtask Parallelism**: Decompose problem into independent required parts, execute in parallel, merge results
2. **Trial Parallelism**: Explore competing uncertain solution attempts in parallel, keep useful results

Key empirical finding: Trial Parallelism accounts for **65.5%** of parallelizable reasoning steps in DeepSeek-V4 on Humanity's Last Exam (73.8% for DeepSeek-R1). Hard reasoning ≠ decomposition; it = trying and refining uncertain solution paths.

Mechanism:
- **Context-Free Grammar (CFG)**: Converts sequential traces to structured parallel trajectories. Tags specify whether branches are necessary subtasks or speculative trials. Engine-parseable for tool-call execution.
- **PA-GRPO (Parallelism-Aware GRPO)**: Training reward balances accuracy, latency, and parallelism ratios. Teaches model to use both parallelism types effectively.

### Key Results
- ~1.7x average acceleration on AIME24/AIME25 while maintaining competitive accuracy
- Trial Parallelism is majority on hard problems (65-74% of parallelizable steps)
- CFG structure enables real execution, not just theoretical parallelism

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (E8)** | Trial Parallelism → E8 hexagram exploration of multiple reasoning branches simultaneously. Each hexagram trial is a speculative attempt |
| **NT-ACT** | CFG-based parallel dispatch → parallel tool execution with subtask/trial classification |
| **NT-MIND (SEAL)** | PA-GRPO training signal → SEAL reward should balance accuracy, speed, and exploration diversity |
| **Axiom A1 (Cost-Aware)** | Parallelism ratio optimization → cost-aware allocation between subtask and trial parallelism |

### NeoTrix Application
**E8 with trial branches**: Parason's finding that Trial Parallelism is dominant maps directly to E8 hexagram exploration. Our E8 engine currently explores hexagrams sequentially. Parason suggests exploring multiple hexagram branches in parallel, classifying each as subtask (deterministic decomposition) or trial (speculative exploration). The CFG structure could formalize our hexagram reasoning paths as parseable, executable parallel programs.

**SEAL with parallel exploration**: SEAL stages could execute trial branches in parallel — exploring multiple distillation strategies simultaneously, then selecting the best. PA-GRPO's reward balance (accuracy + latency + exploration diversity) provides a principled training signal for SEAL evolution.

---

## Paper 5: RAM — Read As Human: Context Compression via Parallelizable Close Reading and Skimming

**URL**: https://aclanthology.org/2026.acl-long.1309.pdf
**Authors**: Tang, Liu, Zhang, Lv, Zhao, Lu, Liu, Chen, Yuan, Zheng, Su, Zheng
**Date**: 2026 (ACL 2026)

### Core Idea
Mimics human reading behavior for context compression:
- **Close reading** on high-relevance segments (full retention, natural language)
- **Skimming** on low-relevance segments (query-guided compression into compact summary vectors)

Architecture:
1. **Query-Aware Parallel Encoding**: All segments processed in parallel with query using shared encoder (no quadratic cost, no iterative compression)
2. **Adaptive Compression**: Learnable selection mechanism decides per-segment: close reading (retain verbatim) or skimming (compress to vector)
3. **Hybrid Representation**: Explicit close-reading segments + implicit skimming vectors concatenated for decoder
4. **Contrastive Learning**: Trained on positive/negative query-segment pairs to optimize the close-reading vs skimming decision boundary

### Key Results
- Up to 12x end-to-end speedup on long inputs (avg 16K, max 32K tokens)
- Outperforms baselines on QA and summarization benchmarks across two backends
- Preserves key information while maintaining natural language interpretability
- Parallel encoding avoids quadratic attention cost

### NeoTrix Domain Mapping

| Domain | Integration |
|--------|------------|
| **NT-CORE (GWT)** | Close reading vs skimming → GWT attention routing: high-salience signals get full attention, low-salience signals get summarized. Adaptive, query-aware |
| **NT-MEMORY** | Hybrid representation → KB query results: relevant entries returned as full text, less relevant as compressed embeddings |
| **NT-WORLD** | Parallel segment processing → parallel document chunk processing in crawl pipeline |
| **Axiom A2 (Context as Scarce)** | Context compression → selective context retention under budget constraints |

### NeoTrix Application
**GWT with adaptive attention depth**: RAM's close-reading vs skimming decision directly applies to GWT attention routing. Currently GWT broadcasts salient info uniformly. RAM suggests an adaptive approach: high-salience signals get full attention (close reading), low-salience signals get compressed summaries (skimming). The contrastive learning objective provides a training signal for the attention-depth decision.

**KB query result hybridization**: When querying KB experiences, return high-relevance results as full text (close reading) and low-relevance results as compressed embeddings (skimming). This maintains context budget while preserving coverage. The parallel encoding approach avoids the quadratic cost of sequential context processing.

---

## Cross-Paper Patterns

| Pattern | Papers | NeoTrix Mapping |
|---------|--------|-----------------|
| **Speculative execution** | SpecBox, Parason | Overlap preparation with execution — predict tool needs, explore reasoning branches in parallel |
| **Self-evolving structures** | QueenBee, MetaInfer | SEAL pipeline and communication topologies should evolve from execution traces, not be fixed |
| **Adaptive attention depth** | RAM, SpecBox | Not all context needs equal attention — close reading for high-relevance, skimming for low |
| **Knowledge as constraint** | MetaInfer, QueenBee | Structured knowledge about what works (CKB, design rules) enables faster future decisions |
| **Parallel exploration** | Parason, RAM | Hard problems require parallel trial exploration, not just sequential decomposition |
| **Anti-self-deception** | QueenBee | Self-evolving systems need safeguards against mistaking luck for capability |

---

## Implementation Candidates

| Priority | Paper | Action | NeoTrix Component |
|----------|-------|--------|-------------------|
| P0 | SpecBox | Add intent-aware prewarming to NT-ACT tool execution | nt_act::tool_prewarming |
| P0 | RAM | Add adaptive attention depth (close-read vs skim) to GWT | nt_core_gwt::adaptive_attention |
| P1 | Parason | Add trial parallelism to E8 hexagram exploration | nt_core_e8::parallel_trials |
| P1 | QueenBee | Add Preserve/Modify/Avoid rule distillation to SEAL | seal::design_rules |
| P2 | MetaInfer | Build CKB (Contract Knowledge Base) for SEAL constraints | seal::constraint_kb |
| P2 | QueenBee | Add anti-self-devolution safeguards to ConsciousnessTree | nt_meta::safeguard_delusion |
