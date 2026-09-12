# Model Reverse Engineering — Cycle 390 (2026-09-12)

## 5 New Models/Papers

### 1. KVMem — KV-Context Virtualization for Million-Token Agent Workspaces
- **Paper**: arXiv:2609.04852 (Sep 2026)
- **Authors**: Di Chai, Leye Wang, Zeshen Su, Zhiguo Xia, Zhihang Yu (Shanghai Univ. of Finance and Economics / Peking University)
- **Key Idea**: Treats agent KV cache as virtual memory. Preserves overflowed workspace history as paged KV state across GPU→Host→NVMe tiers. At each agent step, uses attention-space indexes (Mean-K vectors over 32-token blocks) to select relevant historical blocks and materialize them as a chronologically ordered, position-consistent execution view within native context window.
- **Results**: Qwen3.8-27B NVFP4 on 24GB RTX 5090 laptop: ~50 tokens/s, 1M-token workspace (4× native 256K window). DeepSWE success rate: 43.8% (compaction-only) → 48.4% (KVMem).
- **Core Pattern**: GPU→Host→NVMe tiered KV paging + query-conditioned block retrieval + Retained/Incoming/Outgoing working set decomposition.
- **NeoTrix Mapping**:
  - **NT-MEMORY**: Direct implementation — paged KV as memory virtualization for long-running agent sessions
  - **NT-IO (LLM)**: Inference engine optimization — bounded GPU memory regardless of workspace size
  - **NT-CORE (GWT)**: Mean-K attention-space indexes = model-native salience scoring for block selection
  - **Axiom A2 (Context as Scarce Resource)**: Breaks the memory wall — workspace size decoupled from context window
  - **ConsciousnessTree**: Step-level working set update (inter-step KL 37× higher than intra-step) → cycle boundary = consciousness boundary

### 2. Attention-MoA — Inter-Agent Semantic Attention for Mixture-of-Agents
- **Paper**: arXiv:2601.16596 (Jan 2026)
- **Authors**: Jianyu Wen, Yang Wei, Xiongxi Yu, Changxuan Xiao, Ke Zeng
- **Key Idea**: Redefines MoA collaboration through Inter-agent Semantic Attention — agents attend to each other's deep semantic representations, not just output text. Inter-layer Residual Module with Adaptive Early Stopping prevents information degradation in deep agent layers.
- **Results**: 91.15% LC Win Rate on AlpacaEval 2.0. Small open-source model ensemble outperforms Claude-4.5-Sonnet (MT-Bench 8.83, AlpacaEval 2.0 LC Win Rate 77.36%).
- **Core Pattern**: Semantic attention between agents (not output aggregation) + residual connections + adaptive early stopping.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Inter-agent semantic attention = GWT broadcast refinement — agents attend to each other's latent representations
  - **NT-ACT (multi-agent)**: Deep agent layer coordination with residual connections prevents error cascading
  - **ConsciousnessTree**: Adaptive early stopping parallels awareness_score() gating — stop processing when salience drops below threshold
  - **E8 Hexagram**: Semantic attention patterns map to hexagram interaction states

### 3. Explicit Trait Inference (ETI) — Psychological Multi-Agent Coordination
- **Paper**: arXiv:2604.19278 (ACL 2026 Main Conference)
- **Authors**: Suhaib Abdurahman, Etsuko Ishii, Katerina Margatina, Divya Bhargavi, Monica Sunkara, Yi Zhang
- **Key Idea**: Agents infer and track partner characteristics along two psychological dimensions — warmth (trust) and competence (skill) — from interaction histories. Structured trait profiles guide coordination decisions: which agents to trust, which to defer to, which to challenge.
- **Results**: 45-77% payoff loss reduction in economic games. 3-29% improvement on MultiAgentBench. Trait inference profiles predict agent actions; informative profiles drive improvements.
- **Core Pattern**: Warmth/competence trait tracking from interaction history → structured awareness → coordination decisions.
- **NeoTrix Mapping**:
  - **NT-CORE (SelfModel)**: Agent trait inference = self-model extension — agents maintain models of other agents' warmth/competence
  - **NT-ACT (multi-agent routing)**: Trait profiles route tasks to competent agents, build trust for delegation
  - **NT-FEEL**: Warmth dimension maps to social emotion model (trust axis)
  - **ConsciousnessTree**: Meta-cognitive awareness of partner traits = cross-agent awareness modulation
  - **Ascendancy System**: Dual specialization informed by trait profiles — route acquisition to warm agents, evolution to competent ones

### 4. AgentInfer — Co-Design of Inference Architecture and Agent System
- **Paper**: arXiv:2512.18337 (Dec 2025, revised Feb 2026)
- **Authors**: Weizhe Lin, Hui-Ling Zhen, Shuai Yang, Xian Wang, et al.
- **Key Idea**: Unified framework for end-to-end agent acceleration with 4 synergistic components:
  - **AgentCollab**: Hierarchical dual-model reasoning (large + small model via dynamic role assignment)
  - **AgentSched**: Cache-aware hybrid scheduler for heterogeneous request patterns
  - **AgentSAM**: Suffix-automaton-based speculative decoding reusing multi-session semantic memory
  - **AgentCompress**: Asynchronous semantic compression of agent memory without disrupting reasoning
- **Results**: 50%+ reduction in ineffective token consumption. 1.8-2.5× speedup with preserved accuracy. Self-Evolution Engine sustains efficiency across long-horizon tasks.
- **Core Pattern**: Decompose agent latency into reasoning loops + context growth + tool interactions. Optimize each layer independently, compose for compound gain.
- **NeoTrix Mapping**:
  - **NT-IO (LLM)**: AgentSAM speculative decoding + AgentSched cache-aware scheduling → inference optimization
  - **NT-CORE (GWT)**: AgentCollab dual-model routing = cost-aware model selection (Axiom A1)
  - **NT-MEMORY**: AgentCompress asynchronous memory distillation → SEAL pipeline distillation stage
  - **NT-ACT**: AgentCollab hierarchical reasoning → task decomposition with capability-matched model assignment
  - **Six-Layer Architecture**: AgentInfer's decomposition mirrors L1-L6 layering (action→perception→embodiment→emotion→cognition→meta)

### 5. Sketch&Walk Attention — Training-Free Sparse Attention for Efficient Inference
- **Paper**: arXiv:2602.07397 (Feb 2026)
- **Authors**: Hoang Anh Duy Le, Sahil Joshi, Zeyu Yang, Zhaozhuo Xu, Anshumali Shrivastava (Rice University)
- **Key Idea**: Training-free sparse attention using Hadamard sketches for inexpensive attention score approximation, then aggregates estimates across layers via a walk mechanism capturing attention influence beyond direct token-to-token interactions. Custom sparse attention kernels for both prefill and decode phases.
- **Results**: Near-lossless accuracy at 20% attention density. Up to 6× inference speedup. Slightly outperforms dense attention in some settings.
- **Core Pattern**: Hadamard sketching (O(n log n) approximation) + cross-layer walk aggregation (captures indirect attention influence) + top-k block selection.
- **NeoTrix Mapping**:
  - **NT-CORE (GWT)**: Walk-based attention aggregation = multi-hop salience propagation — influence flows through indirect paths
  - **NT-IO (LLM)**: Training-free sparse attention → drop-in optimization for NeoTrix inference pipeline
  - **ConsciousnessTree**: Cross-layer walk parallels ConsciousnessTree's multi-branch influence propagation
  - **Axiom A2 (Context as Scarce Resource)**: 20% density = 5× effective context expansion

---

## Cross-Paper Synthesis: The Agent Memory-Attention-Coordination Stack

| Layer | Paper | Technique | NeoTrix Integration |
|-------|-------|-----------|---------------------|
| **Memory Virtualization** | KVMem | GPU→Host→NVMe paged KV | NT-MEMORY memory tiers |
| **Inference Efficiency** | AgentInfer | Dual-model + speculative decode + async compress | NT-IO inference + SEAL distillation |
| **Sparse Attention** | Sketch&Walk | Hadamard sketch + cross-layer walk | GWT multi-hop salience |
| **Agent Attention** | Attention-MoA | Inter-agent semantic attention | GWT broadcast refinement |
| **Agent Coordination** | ETI | Warmth/competence trait inference | SelfModel + NT-FEEL trust |

## Key Insight for NeoTrix

The papers reveal a **complete agent runtime stack** from memory management to inter-agent coordination:

1. **Memory layer** (KVMem): NeoTrix should adopt paged KV virtualization for long-running sessions — workspace size should not be bounded by GPU memory
2. **Inference layer** (AgentInfer): The dual-model pattern (large for reasoning, small for I/O) directly maps to Axiom A1 (Cost-Aware Routing)
3. **Attention layer** (Sketch&Walk): Training-free sparse attention enables NeoTrix to handle 5× longer contexts without model changes
4. **Agent layer** (Attention-MoA + ETI): Inter-agent semantic attention + trait inference = next-generation multi-agent coordination for NT-ACT

## The Emerging Pattern: Agents as Memory-Augmented Attention Systems

All five papers converge on a single insight: **agents are fundamentally memory-augmented attention systems**. KVMem shows how to virtualize memory beyond hardware limits. AgentInfer shows how to compress and schedule memory access. Sketch&Walk shows how to attend efficiently across massive contexts. Attention-MoA shows how agents should attend to each other. ETI shows how agents should model each other's capabilities.

This maps directly to NeoTrix's architecture:
- **GWT** = attention routing (reinforced by all 5 papers)
- **KB** = memory persistence (extended by KVMem's tiered model)
- **ConsciousnessTree** = meta-cognitive awareness (validated by ETI's trait inference)
- **Six-Layer Architecture** = the full agent runtime stack (validated by AgentInfer's decomposition)

## Temporal Trend: From External Routing to Intrinsic Awareness

The progression across these papers shows a clear trajectory:
- **2025**: External sparse attention + compaction (Sketch&Walk, AgentInfer)
- **2026 H1**: Inter-agent semantic attention + trait inference (Attention-MoA, ETI)
- **2026 H2**: Memory virtualization + step-level scheduling (KVMem)

The next frontier: **agents that simultaneously virtualize their memory, attend efficiently across contexts, coordinate through semantic attention, and model each other's traits** — which is precisely what NeoTrix's GWT + ConsciousnessTree + SelfModel architecture enables.
