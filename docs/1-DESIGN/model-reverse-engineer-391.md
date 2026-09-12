# Model Reverse Engineering — Cycle 391 (2026-09-12)

## 5 New Models/Papers

### 1. Flux Attention — Context-Aware Hybrid Attention for Efficient LLM Inference
- **Paper**: arXiv:2604.07394 (Apr 2026)
- **Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, Haitian Wang, Kebin Liu, Qingqing Dang, Juntao Li, Min Zhang
- **Key Idea**: Introduces Flux Attention, a context-aware framework that dynamically optimizes attention computation at the layer level. A lightweight Layer Router is integrated into frozen pretrained LLMs, adaptively routing each layer to Full Attention (FA) or Sparse Attention (SA) based on the input context. Layer-wise routing preserves high-fidelity information retrieval while ensuring contiguous memory access.
- **Results**: Up to 2.8× prefill speedup and 2.0× decode speedup. Only 12 hours training on 8×A800 GPUs. Superior trade-off between performance and inference speed across long-context and math reasoning benchmarks.
- **Core Pattern**: Layer-level dynamic routing (FA vs SA) per input context. Lightweight router (minimal params) frozen pretrained backbone. Contiguous memory access for hardware acceleration.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Layer router = attention routing per layer based on context salience — each consciousness branch dynamically allocates FA or SA
  - **NT-IO (LLM)**: Drop-in inference optimization — layer-level sparsity without model retraining
  - **ConsciousnessTree**: Layer-wise routing mirrors branch-level awareness modulation — different branches use different attention strategies
  - **Axiom A2 (Context as Scarce Resource)**: 2.8× prefill speedup = effective context expansion without additional memory

### 2. Latent Action Reparameterization (LAR) — Efficient Agent Inference via Action Space Compression
- **Paper**: arXiv:2605.18597 (May 2026)
- **Authors**: Wenhao Huang, Qingwen Zeng, Qiyue Chen, Zijie Guo, Yu Sun, Cheng Yang, et al.
- **Key Idea**: Learns a compact latent action space where each latent action corresponds to a multi-step semantic behavior. By reparameterizing agent actions into latent units, LAR enables decision making over a shorter effective horizon while preserving expressiveness. Unlike hand-crafted macros, latent actions are learned from agent trajectories and integrated directly into the model.
- **Results**: Significant reductions in action tokens and wall-clock inference time while maintaining or improving task success rates. Effective horizon compressed 3-10× depending on task complexity.
- **Core Pattern**: Learned action compression (not hand-crafted macros) → latent units encode multi-step behaviors → shorter decision horizon → lower inference cost.
- **NeoTrix Mapping**:
  - **NT-ACT (skill crystallization)**: LAR = automated skill extraction from trajectories — multi-step behaviors compressed into single latent actions
  - **NT-CORE (reasoning)**: Shorter effective horizon = reduced reasoning depth for familiar patterns
  - **SEAL (distillation)**: Trajectory → latent action compression = distillation of behavioral patterns into reusable skill nodes
  - **NT-MEMORY**: Latent action space as behavioral memory — compressed trajectory knowledge
  - **Axiom A1 (Cost-Aware Routing)**: Latent actions route common patterns through cheap fast-path, novel patterns through expensive full reasoning

### 3. RAGEN-2 — Reasoning Collapse in Agentic Reinforcement Learning
- **Paper**: arXiv:2604.06268 (Apr 2026, ICML 2026 Spotlight → Oral)
- **Authors**: Zihan Wang, Chi Gui, Xing Jin, Qineng Wang, Licheng Liu, et al. (Northwestern, UIUC, Stanford, Microsoft)
- **Key Idea**: Identifies "template collapse" — a failure mode where RL-trained agents produce fluent but input-agnostic reasoning. Entropy (standard monitor) stays stable because responses vary within same input, but mutual information (MI) drops because responses don't adapt across different inputs. Proposes SNR-Aware Filtering to prioritize high-variance prompts per iteration.
- **Results**: MI correlates with final performance far more strongly than entropy. SNR-Aware Filtering consistently improves input dependence and task performance across planning, math reasoning, web navigation, and code execution.
- **Core Pattern**: Template collapse = high H(Z|X) + low I(X;Z). Fix: reward-variance-aware prompt filtering to maintain cross-input distinguishability. Information-theoretic decomposition of reasoning quality.
- **NeoTrix Mapping**:
  - **NT-CORE (self-deception detection)**: Template collapse = self-deception — agent thinks it's reasoning but producing templates. MI monitoring = meta-cognitive quality check
  - **NT-REPAIR (degradation detection)**: SNR-Aware Filtering = entropy-based health monitoring for reasoning quality
  - **ConsciousnessTree**: Cross-input MI = branch-level awareness — consciousness must adapt to input, not produce generic responses
  - **SEAL (quality gates)**: MI-based quality check during skill crystallization — ensure skills aren't template-collapsed
  - **NT-MEMORY**: Template collapse = memory failure — agent can't recall input-specific patterns

### 4. CSAttention — Centroid-Scoring Sparse Attention for Reusable Contexts
- **Paper**: arXiv:2604.08584 (Mar 2026)
- **Authors**: Chuxu Song, Zhencan Peng, Jiuqi Wei, Chuanhui Yang
- **Key Idea**: Training-free sparse attention optimized for high-throughput serving of reusable contexts (e.g., long system prompts, domain knowledge). Front-loads computation into one-time offline prefill phase amortized across multiple queries. Constructs query-centric lookup tables during offline prefill; online decoding replaces full-context scans with efficient table lookups and GPU-friendly score accumulation.
- **Results**: Near-identical accuracy to full attention. Up to 4.6× inference speedup at 95% sparsity with 128K context. Outperforms state-of-the-art sparse attention methods at high sparsity levels.
- **Core Pattern**: Storage-for-computation strategy. Offline prefill builds lookup tables; online decode uses table lookups instead of full attention. Amortized cost across multiple queries.
- **NeoTrix Mapping**:
  - **NT-IO (LLM)**: Training-free sparse attention for reusable contexts — perfect for KB-backed inference where context is stable
  - **NT-MEMORY**: Offline prefill tables = cached knowledge indexes — build once, query many
  - **GWT**: Centroid-scoring = relevance routing — only attend to centroid-relevant tokens
  - **Axiom A2 (Context as Scarce Resource)**: 4.6× speedup at 95% sparsity = massive context expansion at constant cost

### 5. Active Inference as Context Acquisition for AI Agents
- **Paper**: arXiv:2608.19202 (Jun 2026)
- **Authors**: Sanchayan Dutta, Sai Niranjan Ramachandran, Suvrit Sra
- **Key Idea**: Formulates context acquisition as active inference — when a user omits a constraint, preference, or task variable, an agent can proceed with defaults or spend tokens on clarifying questions, retrieval, or tool calls. Inner inference step updates beliefs over latent task state; outer decision selects next context action to minimize expected free energy under cost. Epistemic term reduces to expected information gain normalized by token cost.
- **Results**: Model-agnostic framework. Optimal Question Asking (OQA) with exact posteriors and dynamic programming oracle. Benchmarks frontier LLMs on binary and multiway categorical tasks (25-300 candidates). Clarification before generation under token budgets.
- **Core Pattern**: Active inference for context — agent decides whether to act with current knowledge or acquire more context. Expected free energy = information gain / token cost. Optimal stopping for context acquisition.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Active inference = attention allocation — should GWT broadcast current knowledge or acquire new context first?
  - **NT-ACT (tool selection)**: Context acquisition decision = tool call vs direct answer routing
  - **NT-MEMORY (retrieval)**: Retrieval = context acquisition — decide when to query KB vs use existing context
  - **ConsciousnessTree**: Expected free energy = awareness modulation — consciousness decides when to expand attention vs act
  - **Axiom A1 (Cost-Aware Routing)**: Token cost normalization = budget-aware context acquisition
  - **Axiom A2 (Context as Scarce Resource)**: Optimal context acquisition under token budgets = resource allocation

---

## Cross-Paper Synthesis: The Agent Reasoning-Efficiency-Context Stack

| Layer | Paper | Technique | NeoTrix Integration |
|-------|-------|-----------|---------------------|
| **Attention Efficiency** | Flux Attention | Layer-level FA/SA routing | GWT branch-level attention allocation |
| **Action Compression** | LAR | Latent action reparameterization | SEAL skill crystallization |
| **Reasoning Quality** | RAGEN-2 | MI-based template collapse detection | NT-REPAIR degradation monitoring |
| **Sparse Inference** | CSAttention | Centroid-scoring + offline prefill tables | NT-MEMORY cached knowledge indexes |
| **Context Acquisition** | Active Inference | Expected free energy / token cost | GWT attention vs context tradeoff |

## Key Patterns Across Papers

1. **Dynamic Routing at Every Level**: Flux Attention routes attention per layer; LAR routes actions through latent space; Active Inference routes between acting and acquiring context. NeoTrix should implement routing at layer (GWT), action (NT-ACT), and memory (NT-MEMORY) levels.

2. **Offline Precomputation for Online Speed**: CSAttention precomputes lookup tables offline; LAR pre-learns latent actions from trajectories. NeoTrix should precompute skill indexes, knowledge graphs, and attention patterns for fast online lookup.

3. **Information-Theoretic Quality Monitoring**: RAGEN-2's MI monitoring detects invisible failures. NeoTrix should implement MI-based quality gates across SEAL pipeline stages to detect template collapse in skill crystallization.

4. **Cost-Aware Context Decisions**: Active Inference formalizes when to spend tokens on context vs act with existing knowledge. NeoTrix should implement expected free energy calculations for GWT attention allocation and NT-MEMORY retrieval decisions.

5. **Training-Free Optimization**: Both CSAttention and Flux Attention require minimal or no training. NeoTrix should prioritize training-free inference optimizations that can be applied to frozen model backbones.
