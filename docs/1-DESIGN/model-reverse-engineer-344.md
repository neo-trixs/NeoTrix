# Model Reverse Engineering — Cycle 344

## 5 Papers

### 1. PackInfer: Compute- and I/O-Efficient Attention for Batched LLM Inference (arxiv:2602.06072)
- **Category**: Inference Kernel Optimization
- **Key Insight**: Production LLM serving batches requests with heterogeneous sequence lengths, causing severe computation/I/O imbalance. PackInfer orchestrates batched requests into load-balanced execution groups, constructs attention kernels directly over packed query-key regions (eliminating redundant computation), and reorganizes KV caches into group-contiguous layouts. 13-20% latency reduction, 20% throughput improvement over FlashAttention.
- **NeoTrix Mapping**: NT-IO + NT-CORE — PackInfer's load-balanced execution groups map to GWT attention routing across specialist modules. Group-contiguous KV layout = domain-organized KB storage (nt_* namespaces). I/O-aware grouping = PerceptionBridge awareness-score-based filtering. **Absorption candidate**: compute/I/O-aware batch scheduling — when multiple domain modules process heterogeneous tasks simultaneously, pack by computational profile to saturate GPU utilization.

### 2. Flux Attention: Context-Aware Hybrid Attention for Efficient LLMs Inference (arxiv:2604.07394)
- **Category**: Dynamic Attention Architecture
- **Key Insight**: Static hybrid attention (fixed FA/SA allocation) fails for variable retrieval demands. Flux Attention introduces a lightweight Layer Router that dynamically routes each layer to Full Attention or Sparse Attention based on input context. Hard routing via argmax at inference (binary per-layer decision). 2.8× prefill speedup, 2.0× decode speedup at 256K context. Only 12 hours training on 8×A800 GPUs. Model Sparsity Ratio (Ω_MSR) quantifies overall sparsity proportion.
- **NeoTrix Mapping**: NT-CORE — Layer Router = GWT salience-based attention modulation per domain layer. Dynamic FA/SA routing = ConsciousnessTree phase-dependent attention (Soil→Roots = SA for exploration, Trunk→Branches = FA for consolidation). Ω_MSR = NeoTrix sparsity metric for module activation. **Absorption candidate**: context-aware per-layer attention routing — each L-layer domain module dynamically declares whether it needs full or sparse attention based on current task context, reducing compute while preserving critical information paths.

### 3. DASH-KV: Accelerating Long-Context LLM Inference via Asymmetric KV Cache Hashing (arxiv:2604.19351)
- **Category**: KV Cache Acceleration via Hashing
- **Key Insight**: Reformulates attention as approximate nearest-neighbor search via asymmetric deep hashing. Differentially maps queries and keys to account for precision/reuse asymmetry. Dynamic mixed-precision: full-precision for critical tokens, compressed for others. Reduces inference from O(N²) to linear O(N). State-of-the-art on LongBench while matching full-attention quality.
- **NeoTrix Mapping**: NT-MEMORY + NT-CORE — asymmetric hashing = KB embedding with query/key asymmetry (queries are ephemeral, keys are persistent). Mixed-precision = experience-tree tiered storage (hot data full-precision, cold data compressed). Linear attention = HyperCube associative recall. **Absorption candidate**: asymmetric KV encoding for KB retrieval — encode queries (ephemeral) and keys (persistent) with different precision strategies, enabling linear-time retrieval over large experience stores.

### 4. SideQuest: Model-Driven KV Cache Management for Long-Horizon Agentic Reasoning (arxiv:2602.22603)
- **Category**: Self-Managed Agent Memory
- **Key Insight**: Heuristic-based KV cache eviction fails for multi-step reasoning. SideQuest lets the Large Reasoning Model itself perform KV cache compression by reasoning about token usefulness. Framed as auxiliary task executed in parallel to prevent management tokens from polluting memory. Trained with only 215 samples. Reduces peak token usage by 65% with minimal accuracy loss. 53.9% reduction in peak KV cache, 36.8% end-to-end runtime reduction.
- **NeoTrix Mapping**: NT-CORE + NT-MEMORY — model-driven self-eviction = ConsciousnessTree self-managed attention (agent decides what to remember/forget). Auxiliary parallel task = NT-META monitoring thread. 215-sample training = few-shot self-calibration. **Absorption candidate**: self-referential memory management — agent reasons about its own memory usefulness, evicting stale context while preserving reasoning-critical tokens. Parallel execution prevents management overhead from polluting primary reasoning.

### 5. Agora: Auction-Based Task Allocation for LLM Agent Reasoning (arxiv:2607.09600)
- **Category**: Game-Theoretic Agent Coordination
- **Key Insight**: Static routing ignores query-dependent model suitability. Agora reformulates task allocation as confidence-calibrated auction — each reasoning step is a tradeable item, agents bid based on calibrated competence (not raw confidence). Prevents overconfident but incompetent agents from hijacking critical reasoning nodes. Tunable cost-quality tradeoff. Competitive with matched candidate pools while enabling heterogeneous agent collaboration.
- **NeoTrix Mapping**: NT-CORE + NT-ACT — auction-based routing = GWT salience with cost-weighted bidding (Axiom A1). Calibrated confidence = SelfTest T3 accuracy estimation. Heterogeneous agent pools = 7-domain specialist modules bidding on tasks. **Absorption candidate**: confidence-calibrated task auction — domain modules bid on reasoning steps based on calibrated competence, not raw confidence. Prevents overconfident modules from dominating critical reasoning paths. Cost-quality tradeoff tunable at runtime.

## Cross-Paper Synthesis

Three convergence patterns:

1. **Agent-as-memory-manager** (SideQuest, DASH-KV) — both shift KV cache management from external heuristics to model-driven decisions. The agent itself decides what to keep, compress, or evict. Validates NeoTrix's ConsciousnessTree self-managed attention and experience-tree consolidation.

2. **Context-aware dynamic routing** (Flux Attention, Agora, PackInfer) — all three achieve efficiency by dynamically routing based on input characteristics, not static allocation. Flux routes layers to FA/SA, Agora routes tasks to agents, PackInfer routes batches to execution groups. Validates NeoTrix's GWT salience-based routing across all layers.

3. **Asymmetric encoding for asymmetric roles** (DASH-KV, PackInfer) — queries and keys, hot and cold data, urgent and background tasks deserve different treatment. One-size-fits-all encoding wastes resources. Maps to NeoTrix's tiered memory (experience-tree hub vs branches) and tiered attention (PerceptionBridge awareness levels).

## NeoTrix Absorption Map

| Paper Pattern | Source | Target Domain | Implementation Path |
|--------------|--------|---------------|---------------------|
| Self-referential KV eviction | SideQuest | NT-CORE (GWT) | ConsciousnessTree self-managed attention per growth cycle phase |
| Context-aware per-layer attention routing | Flux Attention | NT-CORE | Layer-level FA/SA declaration based on task type |
| Asymmetric query/key encoding | DASH-KV | NT-MEMORY | KB embedding with ephemeral-query/persistent-key precision |
| Confidence-calibrated task auction | Agora | NT-CORE + NT-ACT | Domain module bidding on reasoning steps |
| Compute/I/O-aware batch scheduling | PackInfer | NT-IO | Batch domain tasks by computational profile |
