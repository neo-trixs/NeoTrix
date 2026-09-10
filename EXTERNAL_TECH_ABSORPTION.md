# External Technical Absorption Report
## Model Architecture, Inference & Integration Patterns
**Generated**: 2026-09-10 | **Sources**: 48 papers/repos/blogs (2024-2026)

---

## 1. Transformer Architecture Patterns

### Pattern: FlashAttention-2/3 (IO-Aware Exact Attention)
- **Source**: Tri Dao et al. (ICLR 2024 / NeurIPS 2024) — arxiv:2307.08691, arxiv:2407.08608
- **Key Insight**: Reduces attention from quadratic to linear memory via tiling + recomputation. FA-2: 2x faster than FA-1, reaches 73% theoretical FLOPs/s on A100. FA-3: warp-specialization + FP8 block quantization, reaches 85% utilization on H100 (840 TFLOPs/s BF16, 1.3 PFLOPs/s FP8).
- **NeoTrix Mapping**: `nt_physical::gpu_attention_kernel` — GPU attention backend for inference/training acceleration
- **Implementation Priority**: P0

### Pattern: PagedAttention (Virtual Memory for KV Cache)
- **Source**: Kwon et al. (SOSP 2023) — vLLM, arxiv:2309.06180
- **Key Insight**: Divides KV cache into fixed-size blocks (pages), stored in non-contiguous GPU memory. Eliminates memory fragmentation, enables KV cache sharing across requests (beam search, parallel sampling). Near-zero waste in KV cache memory.
- **NeoTrix Mapping**: `nt_physical::kv_cache_manager` — Page-based KV cache allocation/sharing for inference
- **Implementation Priority**: P0

### Pattern: SwiGLU (Gated FFN Activation)
- **Source**: Shazeer (2020), used in LLaMA/Mistral/Qwen/DeepSeek
- **Key Insight**: Three-projection gated FFN: `SwiGLU(x) = W_down[SiLU(W_gate * x) ⊙ (W_up * x)]`. Replaces GELU FFN. Provides learned multiplicative interaction between gate and content paths. Standard in all modern LLMs.
- **NeoTrix Mapping**: `nt_core::ffn_backbone` — Core FFN activation pattern for any transformer module
- **Implementation Priority**: P0

### Pattern: RMSNorm (Pre-Norm Normalization)
- **Source**: Zhang & Sennrich (2019), adopted by LLaMA/Mistral/Qwen
- **Key Insight**: Removes mean centering from LayerNorm: `RMSNorm(x) = x / sqrt(mean(x²) + ε) * γ`. Faster computation, comparable performance to LayerNorm. Always used as Pre-Norm (before attention/FFN).
- **NeoTrix Mapping**: `nt_core::norm_layer` — Standard normalization for transformer blocks
- **Implementation Priority**: P0

### Pattern: RoPE (Rotary Position Embedding)
- **Source**: Su et al. (2021), with YaRN extension (Mistral 3)
- **Key Insight**: Encodes relative position via rotation matrices applied to Q/K vectors. Supports length extrapolation. GPT-J style (pairs adjacent elements). YaRN adds frequency interpolation for context lengths beyond training window.
- **NeoTrix Mapping**: `nt_core::positional_encoding` — Position encoding for attention layers
- **Implementation Priority**: P0

### Pattern: GQA (Grouped-Query Attention)
- **Source**: Ainslie et al. (2023), used in LLaMA-2 70B+, Mistral, Qwen
- **Key Insight**: Multiple query heads share single KV head (grouped). Reduces KV cache memory by factor of num_kv_groups. LLaMA-2 70B: 8 KV heads for 64 query heads. No accuracy loss vs MHA.
- **NeoTrix Mapping**: `nt_core::attention_variant` — Configurable attention mechanism (MHA/GQA/MQA)
- **Implementation Priority**: P0

### Pattern: Mixture of Experts (MoE) with Fine-Grained Routing
- **Source**: DeepSeekMoE (2024), DeepSeek-V3/V4 (2024-2025)
- **Key Insight**: Two innovations: (1) Fine-grained expert segmentation — more, smaller experts (256 routed + 1 shared). (2) Shared expert isolation — always-active experts capture common knowledge, freeing routed experts to specialize. DeepSeek-V4: Hash-MoE bootstrap for first layers (frozen tid→eid lookup), then learned routing. Auxiliary-loss-free load balancing via dynamic bias adjustment.
- **NeoTrix Mapping**: `nt_core::moe_router` — Expert routing with shared/routed separation
- **Implementation Priority**: P1

### Pattern: Auxiliary-Loss-Free Load Balancing
- **Source**: DeepSeek-V3 (2024)
- **Key Insight**: Instead of auxiliary loss (conflicts with training objective), uses dynamic bias adjustment: after each batch, update expert biases based on load. Preserves main training signal while preventing routing collapse.
- **NeoTrix Mapping**: `nt_core::moe_balance` — Load balancing without auxiliary loss
- **Implementation Priority**: P1

---

## 2. Inference Optimization

### Pattern: KV-Compress (Block-Wise KV Cache Eviction)
- **Source**: arxiv:2410.00161 (2024)
- **Key Insight**: Evicts contiguous KV blocks within PagedAttention framework. Variable compression rates per attention head. Integration with vLLM increases throughput by up to 5.18x.
- **NeoTrix Mapping**: `nt_physical::kv_compression` — Memory-efficient KV cache with eviction
- **Implementation Priority**: P1

### Pattern: LayerKV (Layer-Wise KV Cache Management)
- **Source**: arxiv:2410.00428 (2024)
- **Key Insight**: Alternates caching of KV layers between GPU and CPU. Reduces GPU KV block demand, facilitates scheduling new requests. Optimizes TTFT SLO.
- **NeoTrix Mapping**: `nt_physical::kv_offloading` — GPU↔CPU KV cache tiering
- **Implementation Priority**: P1

### Pattern: EAGLE-3 (Speculative Decoding)
- **Source**: Li et al. (ICML 2025) — arxiv:2503.01840
- **Key Insight**: Draft model predicts next features (not tokens) using target model's top-layer features. EAGLE-3: 3.0x-6.5x speedup, scaling law — more training data → proportional speedup increase. 40% throughput improvement at batch size 64 in SGLang.
- **NeoTrix Mapping**: `nt_io::speculative_decoder` — Draft-verify inference acceleration
- **Implementation Priority**: P1

### Pattern: Dynamic Batching (Memory-Aware + SLA-Constrained)
- **Source**: Pang et al. (2025) — arxiv:2503.05248
- **Key Insight**: Real-time batch size adjustment based on GPU memory utilization + latency SLAs. 8-28% throughput gain over static batching. 22% capacity improvement. Works with existing inference infrastructure.
- **NeoTrix Mapping**: `nt_io::batch_scheduler` — Dynamic request batching for LLM serving
- **Implementation Priority**: P1

### Pattern: Continuous Batching (Iteration-Level Scheduling)
- **Source**: HuggingFace TGI, vLLM, Orca
- **Key Insight**: Process one decoding step per iteration, immediately remove completed requests and add new ones. GPU never idles. Synergy with PagedAttention enables packing more sequences. Up to 20x throughput improvement.
- **NeoTrix Mapping**: `nt_io::continuous_batching` — Non-stop GPU utilization for LLM serving
- **Implementation Priority**: P0

### Pattern: Confidence-Adaptive SwiGLU (κ-SwiGLU for MoE)
- **Source**: arxiv:2606.00761 (2026)
- **Key Insight**: Adjusts expert gate sharpness based on token-level routing confidence. High-confidence tokens → sharp, selective gating; low-confidence → smooth, broad gating. +0.6 to +1.0 CORE score improvement across model depths.
- **NeoTrix Mapping**: `nt_core::adaptive_gate` — Confidence-aware expert gating for MoE
- **Implementation Priority**: P2

---

## 3. Fine-tuning & Adaptation

### Pattern: DoRA (Weight-Decomposed Low-Rank Adaptation)
- **Source**: Liu et al. (ICML 2024 Oral) — arxiv:2405.17357
- **Key Insight**: Decomposes pretrained weight into magnitude + directional components. Uses LoRA for efficient directional adaptation. Surpasses full fine-tuning with <0.3% trainable parameters. No additional inference overhead after merging.
- **NeoTrix Mapping**: `nt_mind::lora_adapter` — Parameter-efficient fine-tuning backbone
- **Implementation Priority**: P0

### Pattern: QLoRA (Quantized LoRA)
- **Source**: Dettmers et al. (NeurIPS 2023)
- **Key Insight**: Quantizes pretrained model to 4-bit (NormalFloat4), then applies LoRA adapters on frozen quantized backbone. Finetune 65B model on single 48GB GPU. Guanaco reaches 99.3% ChatGPT performance.
- **NeoTrix Mapping**: `nt_mind::qlora_adapter` — Memory-efficient fine-tuning for edge/resource-constrained
- **Implementation Priority**: P1

### Pattern: AdaLoRA (Adaptive Rank Allocation)
- **Source**: Zhang et al. (ICLR 2023)
- **Key Insight**: Adaptively adjusts parameter budget across weight matrices via SVD-based importance scoring. Allocates higher rank to more important layers. ~3.5% trainable parameters.
- **NeoTrix Mapping**: `nt_mind::adaptive_lora` — Budget-aware rank allocation
- **Implementation Priority**: P2

### Pattern: DPO (Direct Preference Optimization)
- **Source**: Rafailov et al. (NeurIPS 2023)
- **Key Insight**: Eliminates reward model entirely. Reparameterizes RLHF objective to directly optimize policy from preference pairs. Offline, stable training. Foundation for all subsequent preference methods.
- **NeoTrix Mapping**: `nt_mind::alignment_dpo` — Reference-free preference alignment
- **Implementation Priority**: P0

### Pattern: ORPO (Odds Ratio Preference Optimization)
- **Source**: Hong et al. (2024) — arxiv:2403.07691
- **Key Insight**: Unifies SFT + preference learning into single stage. No reference model needed. Uses odds ratio for preference scoring. Outperforms SFT+DPO and SFT+PPO pipelines. 1.0x length overhead (no length hacking).
- **NeoTrix Mapping**: `nt_mind::alignment_orpo` — Single-stage alignment without reference model
- **Implementation Priority**: P1

### Pattern: KTO (Kahneman-Tversky Optimization)
- **Source**: Ethayarajh et al. (2024)
- **Key Insight**: Handles unpaired binary feedback (no preference pairs needed). Based on prospect theory — loss aversion asymmetry. Only needs "thumbs up/thumbs down" signals.
- **NeoTrix Mapping**: `nt_mind::alignment_kto` — Binary feedback alignment
- **Implementation Priority**: P2

### Pattern: PPO vs DPO Selection Matrix
- **Source**: Xu et al. (ICML 2024), oxRL framework (2026)
- **Key Insight**: PPO outperforms DPO in challenging code generation and when online generation is available. DPO better for limited compute. SimPO has length hacking issues. No variant statistically beats vanilla DPO consistently (oxRL: 51 algorithms, no clear winner).
- **NeoTrix Mapping**: `nt_mind::alignment_router` — Method selection based on task/data constraints
- **Implementation Priority**: P1

---

## 4. RAG & Knowledge Integration

### Pattern: Graph RAG (GRAG)
- **Source**: Hu et al. (NAACL 2025) — arxiv:2405.16506
- **Key Insight**: Retrieves textual subgraphs (not isolated chunks). Divide-and-conquer subgraph retrieval in linear time. Dual-view generation: text view + graph view. Outperforms standard RAG on multi-hop reasoning benchmarks.
- **NeoTrix Mapping**: `nt_memory::graph_rag` — Graph-structured knowledge retrieval
- **Implementation Priority**: P1

### Pattern: KG-Guided RAG (KG²RAG)
- **Source**: Zhu et al. (NAACL 2025) — arxiv:2502.06864
- **Key Insight**: Uses knowledge graphs for fact-level relationships between chunks. KG-guided chunk expansion + KG-based chunk organization. Improves diversity and coherence of retrieved results.
- **NeoTrix Mapping**: `nt_memory::kg_guided_retrieval` — Knowledge-graph-enhanced chunk retrieval
- **Implementation Priority**: P1

### Pattern: GFM-RAG (Graph Foundation Model for RAG)
- **Source**: Luo et al. (NeurIPS 2025)
- **Key Insight**: Graph neural network reasons over graph structure to capture query-knowledge relationships. 8M parameter GFM trained on 60 KGs (14M triples, 700K documents). Zero-shot generalization to unseen datasets.
- **NeoTrix Mapping**: `nt_memory::gfm_retrieval` — Neural graph-based retrieval
- **Implementation Priority**: P2

### Pattern: Hybrid Search (BM25 + Dense Retrieval)
- **Source**: Multiple (LangChain, LlamaIndex, etc.)
- **Key Insight**: Combines lexical (BM25) and semantic (dense embedding) retrieval. Handles both keyword-precise and semantic-broad queries. Reciprocal rank fusion for score combination.
- **NeoTrix Mapping**: `nt_memory::hybrid_search` — Dual-path retrieval (lexical + semantic)
- **Implementation Priority**: P0

---

## 5. Agent Frameworks

### Pattern: LangGraph (Graph-Based State Machine)
- **Source**: LangChain (2024-2026), 37K+ GitHub stars
- **Key Insight**: Agents as nodes, transitions as edges in directed graph. State carried through graph with external persistence. Best-in-class MCP integration. Best for production RAG, conditional branching, retry loops. 4.2s execution time, 2800 tokens/task.
- **NeoTrix Mapping**: `nt_act::workflow_graph` — Production agent orchestration
- **Implementation Priority**: P1

### Pattern: CrewAI (Role-Based Multi-Agent)
- **Source**: CrewAI Inc. (2024-2026), 55K+ GitHub stars
- **Key Insight**: Agents as "crew members" with roles/goals/tools. Event-driven orchestration. Fastest prototyping (2-4 hours to working prototype). Best for sequential multi-agent pipelines. Higher token consumption (5200/task).
- **NeoTrix Mapping**: `nt_act::role_agents` — Role-based multi-agent collaboration
- **Implementation Priority**: P2

### Pattern: AutoGen (Conversation-Driven Multi-Agent)
- **Source**: Microsoft Research (2024-2025), 60K+ GitHub stars (now AG2, maintenance mode)
- **Key Insight**: Agents negotiate through multi-turn conversations. GroupChat with dynamic speaker selection. Built-in code execution sandbox. Best for open-ended research tasks. Highest token cost (8900/task).
- **NeoTrix Mapping**: `nt_act::conversation_agents` — Conversational multi-agent reasoning
- **Implementation Priority**: P2

### Pattern: Cascade Routing (Unified Routing + Cascading)
- **Source**: Dekoninck et al. (ICML 2025) — arxiv:2410.10347
- **Key Insight**: Integrates routing (pick best model per query) and cascading (try cheap→expensive sequentially). Theoretically optimal strategy. Up to 8% improvement on RouterBench, 14% on SWE-Bench. Quality estimator is the critical factor.
- **NeoTrix Mapping**: `nt_io::cascade_router` — Unified model selection strategy
- **Implementation Priority**: P0

### Pattern: Difficulty-Aware Model Routing
- **Source**: Multiple — RouteLLM, vLLM Semantic Router (2025-2026)
- **Key Insight**: Classify query complexity → route to cheapest capable model. ModernBERT classifier for intent/complexity. Reasoning queries → CoT models; simple queries → standard inference. ~90% token savings (Spotify Portal Shunt).
- **NeoTrix Mapping**: `nt_io::difficulty_router` — Query complexity-based model selection
- **Implementation Priority**: P0

### Pattern: Cross-Attention Routing
- **Source**: arxiv:2509.09782 (2025)
- **Key Insight**: Single-head cross-attention jointly models query + model embeddings. Lightweight, generalizes across domains. Exponential reward function for stable cost-quality balancing.
- **NeoTrix Mapping**: `nt_io::attention_router` — Neural model selection via cross-attention
- **Implementation Priority**: P2

---

## 6. Architecture Variations Summary

### LLaMA Family (GPT→LLaMA Evolution)
| Component | GPT-2/3 | LLaMA | Key Change |
|-----------|---------|-------|------------|
| Normalization | LayerNorm | RMSNorm | Remove mean centering |
| Position | Learnable absolute | RoPE | Relative position, extrapolation |
| Activation | GELU | SwiGLU | Gated FFN |
| Attention | MHA | GQA (LLaMA-2+) | Shared KV heads |
| Bias | All layers | None | Reduces params |

### DeepSeek-V3/V4 MoE Architecture
- **Total**: 671B params (V3), 284B-1.6T (V4)
- **Active**: 37B (V3), 13B-49B (V4)
- **Shared experts**: 1 (always active)
- **Routed experts**: 256 (V3), 256-384 (V4)
- **Active routed**: 8 (V3), 6 (V4 — finer specialization)
- **V4 novelty**: Hash-MoE bootstrap (first 3 layers), Sqrt(Softplus) affinity, clamped SwiGLU

### Mistral 3 Architecture
- YaRN-corrected RoPE (partial per-frequency interpolation)
- No QK-norm (skips per-head RMSNorm of Q/K)
- GQA: 8 KV heads for all sizes
- Pixtral vision encoder integration
- Context: 131K-262K tokens

---

## Cross-Reference: NeoTrix Module Mapping

| NeoTrix Module | Absorbed Patterns | Priority |
|----------------|-------------------|----------|
| `nt_core::ffn_backbone` | SwiGLU, GeGLU | P0 |
| `nt_core::norm_layer` | RMSNorm Pre-Norm | P0 |
| `nt_core::positional_encoding` | RoPE + YaRN | P0 |
| `nt_core::attention_variant` | MHA/GQA/MQA | P0 |
| `nt_core::moe_router` | DeepSeek fine-grained MoE | P1 |
| `nt_core::moe_balance` | Auxiliary-loss-free balancing | P1 |
| `nt_core::adaptive_gate` | κ-SwiGLU confidence gating | P2 |
| `nt_physical::gpu_attention_kernel` | FlashAttention-2/3 | P0 |
| `nt_physical::kv_cache_manager` | PagedAttention vLLM | P0 |
| `nt_physical::kv_compression` | KV-Compress block eviction | P1 |
| `nt_physical::kv_offloading` | LayerKV GPU↔CPU tiering | P1 |
| `nt_io::continuous_batching` | Iteration-level scheduling | P0 |
| `nt_io::batch_scheduler` | Dynamic batching (memory+SLA) | P1 |
| `nt_io::speculative_decoder` | EAGLE-3 draft-verify | P1 |
| `nt_io::cascade_router` | Unified routing+cascading | P0 |
| `nt_io::difficulty_router` | Query complexity routing | P0 |
| `nt_io::attention_router` | Cross-attention model selection | P2 |
| `nt_mind::lora_adapter` | DoRA weight decomposition | P0 |
| `nt_mind::qlora_adapter` | QLoRA 4-bit + LoRA | P1 |
| `nt_mind::adaptive_lora` | AdaLoRA rank allocation | P2 |
| `nt_mind::alignment_dpo` | Direct preference optimization | P0 |
| `nt_mind::alignment_orpo` | Single-stage alignment | P1 |
| `nt_mind::alignment_kto` | Binary feedback alignment | P2 |
| `nt_mind::alignment_router` | PPO/DPO/ORPO selection | P1 |
| `nt_memory::graph_rag` | GRAG subgraph retrieval | P1 |
| `nt_memory::kg_guided_retrieval` | KG²RAG chunk expansion | P1 |
| `nt_memory::hybrid_search` | BM25 + dense retrieval | P0 |
| `nt_act::workflow_graph` | LangGraph state machine | P1 |
| `nt_act::role_agents` | CrewAI role-based agents | P2 |

---

## Priority Distribution
- **P0 (12 patterns)**: FlashAttention, PagedAttention, SwiGLU, RMSNorm, RoPE, GQA, Continuous Batching, Cascade Router, Difficulty Router, DPO, DoRA, Hybrid Search
- **P1 (13 patterns)**: MoE routing, Auxiliary-loss-free balancing, KV-Compress, LayerKV, EAGLE-3, Dynamic Batching, ORPO, PPO/DPO selection, Graph RAG, KG²RAG, LangGraph, QLoRA
- **P2 (7 patterns)**: κ-SwiGLU, AdaLoRA, KTO, GFM-RAG, CrewAI, AutoGen, Cross-Attention Router
