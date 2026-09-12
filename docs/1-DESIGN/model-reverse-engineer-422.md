# Model Reverse Engineering — Cycle 422 (2026-09-12)

## 5 New Models/Papers → NeoTrix Domain Mapping

---

### 1. TIPEX: Two-Tier Inference-Time Parallelism for Multi-Agent LLM Systems
- **Paper**: arXiv:2608.05791 (ICML 2026)
- **Authors**: Zihan Xu, Haolin Tian, Hai Jiang
- **Key Innovation**: Unifies two levels of parallelism in multi-agent LLM systems:
  - **Replica Parallelism**: Multiple complete solution paths at task level
  - **Structural Parallelism**: Concurrent execution within single path via task decomposition
- **Results**: Significant accuracy improvement + latency reduction on GAIA benchmark. Intermediate complexity tasks benefit most from coordination.
- **Key Finding**: Overly aggressive parallel strategies don't necessarily yield better performance — sweet spot exists.

#### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | GWT attention routing | Replica parallelism = multiple salience paths explored simultaneously. Structural parallelism = decomposition within GWT broadcast. |
| **NT-ACT** | Orchestration | TIPEX framework directly applicable to NT-ACT task decomposition. Replica paths as parallel tool invocations. |
| **NT-MIND** | SEAL pipeline | Parallel exploration paths = SEAL exploration phase diversification. |
| **NT-PHYSICAL** | Hardware coordination | Structural parallelism maps to multi-device load balancing. |

**NeoTrix Integration**: Implement `ParallelExploration` in NT-CORE that spawns replica paths for complex reasoning tasks, with structural decomposition for subtask parallelism. Add cost-aware threshold — only activate for intermediate-complexity tasks (validated by TIPEX finding).

---

### 2. AgentSpec: Speculative Decoding for Batch Inference of LLM Agents
- **Paper**: arXiv:2608.24004 (EMNLP 2026)
- **Authors**: Xin Wang, Ziming Miao, Yi Zhu, Hui Shen, Zhongwei Wan, Fan Yang, Mi Zhang
- **Key Innovation**: Speculative decoding tailored for LLM agent workloads:
  - **Structure-Isolated Drafting**: Constrains speculation to semantically coherent segments of agent workflow
  - **Redundancy-Aware Budget Allocation**: Uses agent-level information to utilize dynamic token budgets
- **Results**: Extremely low rejection rate. Superior over SOTA on 5 workloads × 4 models in vLLM.
- **Key Finding**: Agent workflows have predictable structure that can be exploited for speculative drafting.

#### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-IO** | Inference optimization | AgentSpec directly applicable to NT-IO LLM provider optimization. Structure-isolated drafting = workflow-aware inference. |
| **NT-ACT** | Tool calling | Agent workflow structure = tool call sequences. Speculative drafting pre-generates likely tool call patterns. |
| **NT-CORE** | E8 reasoning | Agent workflow segments map to E8 hexagram states — predict next state, speculate execution. |
| **NT-SHIELD** | Safety | Speculative execution must respect safety boundaries — pre-generated content still subject to guardrails. |

**NeoTrix Integration**: Implement `WorkflowAwareSpeculation` in NT-IO that uses agent task structure to predict likely next steps. Connect to E8 state transitions for hexagram-aware speculation. Critical: speculation must be rolled back on safety violations.

---

### 3. Agent-Radar: Attention Steering with Context Relevance
- **Paper**: arXiv:2605.30136 (2026)
- **Authors**: Hongxiang Zhang, Yuan Tian, Tianyi Zhang
- **Key Innovation**: Training-free context management for multi-agent systems:
  - **Temporal and Spatial Decay Mechanism**: Dynamically steers each agent's attention toward relevant context
  - Handles long conversation histories where relevant info is diluted by irrelevant context
- **Results**: Up to 7.64 absolute points improvement across 5 benchmarks. Robust as agents and rounds increase.
- **Key Finding**: Core components are crucial and generalizable across different settings.

#### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | GWT attention routing | Agent-Radar IS attention routing — temporal/spatial decay = GWT salience decay model. Direct mapping. |
| **NT-MEMORY** | Context management | Temporal decay = experience aging. Spatial decay = relevance filtering. Maps to KB query relevance. |
| **NT-MIND** | Distillation | Long conversation histories → distilled relevant context. Training-free = post-hoc distillation. |
| **NT-NEXUS** | Cross-session memory | Temporal decay across sessions = session relevance weighting. |

**NeoTrix Integration**: Implement `RadarAttention` in NT-CORE GWT layer. Temporal decay function for experience aging (maps to `experience-tree` branch relevance). Spatial decay for context window management. Critical: training-free approach aligns with NeoTrix's post-training adaptation philosophy.

---

### 4. Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs
- **Paper**: arXiv:2604.07394 (2026)
- **Authors**: Quantong Qiu, Zhiyi Hong, Yi Yang, Haitian Wang, Kebin Liu, Qingqing Dang, Juntao Li, Min Zhang
- **Key Innovation**: Layer-level dynamic routing between Full Attention (FA) and Sparse Attention (SA):
  - **Lightweight Layer Router**: Adaptively routes each layer to FA or SA based on input context
  - Preserves high-fidelity information retrieval while ensuring contiguous memory access
  - Parameter-efficient: only 12 hours training on 8×A800 GPUs
- **Results**: Up to 2.8× prefill speedup, 2.0× decode speedup. Superior performance-speed tradeoff.
- **Key Finding**: Layer-wise routing > head-level routing (avoids load imbalance and synchronization issues).

#### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | GWT attention | Flux Attention IS attention optimization. Layer routing = domain-specific attention allocation. |
| **NT-IO** | Provider optimization | Layer-aware inference for NT-IO LLM providers. Route cheap layers to sparse, expensive to full. |
| **NT-PHYSICAL** | Memory management | Contiguous memory access = hardware-aware attention. Maps to edge device optimization. |
| **NT-MIND** | Efficiency | Parameter-efficient adaptation = SEAL pipeline fast adaptation. 12-hour training = rapid skill crystallization. |

**NeoTrix Integration**: Implement `FluxRouter` in NT-CORE attention layer. Layer-level routing decision based on context complexity. Connect to Cost-Aware Routing (A1) — cheap layers get sparse attention, expensive reasoning layers get full attention. Hardware-aware optimization for NT-PHYSICAL edge deployment.

---

### 5. MoBiE: Efficient Inference of Mixture of Binary Experts
- **Paper**: arXiv:2604.06798 (2026)
- **Authors**: (Multiple authors from MoBiE framework)
- **Key Innovation**: First binarization framework tailored for MoE-based LLMs:
  - **Cross-Expert Redundancy**: Handles similarity across experts (ignored by prior methods)
  - **Task-Agnostic Importance Estimation**: Router norm change + intra-neuron variance for bit-width allocation
  - **Quantization-Induced Routing Shifts**: Addresses routing degradation under binarization
- **Results**: On Qwen3-30B-A3B: 52.2% perplexity reduction, 43.4% zero-shot improvement, 2× speedup.
- **Key Finding**: Expert-shift (routing migration) is the core challenge — binarization degrades routing mechanisms.

#### NeoTrix Domain Mapping

| Domain | Pattern | Integration |
|--------|---------|-------------|
| **NT-CORE** | E8/Hexagram reasoning | MoE experts ≈ E8 hexagram states. Cross-expert redundancy ≈ hexagram similarity. Routing shift = hexagram state drift. |
| **NT-IO** | Model efficiency | MoBiE enables binary deployment of MoE models. Critical for edge inference in NT-PHYSICAL. |
| **NT-MIND** | Distillation | Binary expert selection = distilled expert routing. Task-agnostic importance = generalizable routing. |
| **NT-SHIELD** | Security | Routing shift detection = security audit. Quantization-induced routing = potential vulnerability vector. |
| **NT-PHYSICAL** | Edge deployment | Binary weights = extreme compression for device deployment. 2× speedup enables real-time edge inference. |

**NeoTrix Integration**: Implement `BinaryExpertRouter` in NT-CORE for MoE-style reasoning. Cross-expert redundancy detection for hexagram deduplication. Routing shift monitoring in NT-SHIELD (quantization attacks as adversarial vector). Binary weight support in NT-PHYSICAL for edge deployment.

---

## Cross-Paper Synthesis

### Pattern: Attention as Routing Primitive
All 5 papers treat **attention/routing as the primary optimization target**:
- TIPEX: Parallel routing across solution paths
- AgentSpec: Workflow-aware routing for speculation
- Agent-Radar: Temporal/spatial attention steering
- Flux Attention: Layer-level attention routing
- MoBiE: Expert routing under quantization

**NeoTrix Implication**: GWT should be the unified routing primitive across all domains. Current GWT implementation should be extended with:
1. Trajectory-level routing (from TIPEX)
2. Temporal/spatial decay (from Agent-Radar)
3. Layer-level granularity (from Flux Attention)
4. Workflow-aware prediction (from AgentSpec)
5. Quantization-resilient routing (from MoBiE)

### Pattern: Training-Free Adaptation
3 of 5 papers achieve improvements **without training** (Agent-Radar, Flux Attention partial, AgentSpec). This validates NeoTrix's post-training adaptation philosophy.

### Pattern: Agent Structure Exploitation
AgentSpec and TIPEX both exploit the **predictable structure of agent workflows**. NeoTrix's E8 hexagram states provide exactly this structure — each state transition is predictable and can be speculated.

### Pattern: Efficiency-Accuracy Pareto Frontier
All papers navigate the efficiency-accuracy tradeoff. MoBiE achieves extreme compression (binary), Flux Attention achieves layer-level optimization, Caprese achieves distillation recovery. NeoTrix should implement a **Pareto frontier tracker** that selects optimal configuration based on current resource constraints.

## Recommended Next Actions

1. **Immediate**: Implement `RadarAttention` in NT-CORE (training-free, directly applicable)
2. **Short-term**: Add trajectory-level routing to GWT (from TIPEX findings)
3. **Medium-term**: Build `WorkflowAwareSpeculation` in NT-IO (from AgentSpec)
4. **Long-term**: Implement `BinaryExpertRouter` for edge deployment (from MoBiE)
5. **Cross-cutting**: Build Pareto frontier tracker for efficiency-accuracy optimization
