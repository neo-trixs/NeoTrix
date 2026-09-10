# NeoTrix Research: 10000 Iterations AI Architecture Intelligence

> Generated: 2026-09-10 | Batches: 8/14 executed | Findings: 50+ techniques analyzed

---

## BATCH 1: Transformer Architecture Innovations

### FlashAttention-3
- **Source**: arxiv.org/html/2407.08608v2, spheron.network/blog/flashattention-2-vs-flashattention-3-h100-h200-guide
- **Innovation**: Hopper GPU-optimized attention achieving 75% utilization (740 TFLOPS FP16, 1.2 PFLOPS FP8). Uses warp-specialization, pingpong GEMM-softmax scheduling, and incoherent processing for FP8 accuracy. 1.5-2.0x faster than FA2. At 128K context on H100: 2.44x faster than FA2. Cost drops from $0.089 to $0.051/M tokens with FA3+FP8.
- **Code**: `flash_attn_func(q, k, v, causal=True, window_size=(-1,-1), alibi_slopes=None)`
- **NT-Domain**: NT-CORE (attention kernel optimization)
- **Priority**: P0
- **Effort**: 40h (integrate FA3 for KV cache optimization)

### Ring Attention
- **Source**: arxiv.org/abs/2310.01889, ICLR 2024
- **Innovation**: Distributes long sequences across devices with ring-topology KV block transfer, overlapping communication with blockwise attention computation. Enables 100M+ token context on 512 TPUs. Zero communication overhead when compute > transfer.
- **Code**: Ring of N devices, each sends KV block to next while computing local attention
- **NT-Domain**: NT-CORE (distributed attention), NT-MEMORY (long context)
- **Priority**: P1
- **Effort**: 80h (distributed attention for NeoTrix inference)

### Sliding Window Attention (Mistral)
- **Source**: arxiv.org/abs/2310.06825, Mistral 7B
- **Innovation**: Each token attends to W past tokens only. After k layers, effective span = k×W (e.g., 4 layers × 4096 window = 131K tokens). Rolling buffer cache: fixed W size, overwrites old entries. 2x speedup with FlashAttention.
- **Code**: `flash_attn_func(..., window_size=(W, W))`
- **NT-Domain**: NT-CORE (attention pattern), NT-IO (inference optimization)
- **Priority**: P1
- **Effort**: 20h (configurable sliding window for NeoTrix models)

### Mamba / State Space Models
- **Source**: arxiv.org/abs/2312.00752, arxiv.org/abs/2406.07887 (8B benchmark)
- **Innovation**: Selective SSM with input-dependent parameters (Δ, B, C). Linear-time sequence modeling, 5x higher throughput than Transformers. **Key 8B benchmark**: Mamba-2-Hybrid (43% Mamba-2, 7% attention, 50% MLP) exceeds 8B Transformer on ALL 12 standard tasks (+2.65 pts avg) and is 8x faster at long context. Pure SSM lags on copying/in-context learning tasks.
- **Code**: SSM recurrence: h_t = A_t * h_{t-1} + B_t * x_t; y_t = C_t * h_t
- **NT-Domain**: NT-CORE (alternative architecture), NT-IO (efficient inference)
- **Priority**: P0
- **Effort**: 120h (SSM integration for long-context tasks)

### Mamba-2 SSD Framework
- **Source**: arxiv.org/abs/2405.21060, axiomlogica.com
- **Innovation**: Structured State Space Duality proves Mamba-2 and masked attention are two contraction orders over the same semiseparable matrix. Core layer 2-8x faster than Mamba-1 fused scan. Hybrid SSD+attention is the paper's own recommended path. Production: Transformers still safer default due to ecosystem maturity.
- **NT-Domain**: NT-CORE (SSD architecture)
- **Priority**: P0
- **Effort**: 80h (SSD kernel integration)

### ALiBi / RoPE Position Encoding
- **Source**: arxiv.org/abs/2310.13017, mlmentorship.com
- **Innovation**: ALiBi adds head-specific linear bias to attention scores (no learned params). Position interpolation scales slopes by L/L' to extend context 2x without training. RoPE uses rotation matrices for relative position. NTK-aware scaling preserves high-frequency precision.
- **NT-Domain**: NT-CORE (position encoding)
- **Priority**: P1
- **Effort**: 16h (position encoding library)

---

## BATCH 2: Mixture of Experts

### DeepSeekMoE Architecture
- **Source**: arxiv.org/abs/2401.06066, deepseekai.guide
- **Innovation**: Fine-grained expert segmentation (e.g., 160 routed + 2 shared experts in V2). Shared experts process every token for common knowledge; routed experts specialize. Auxiliary-loss-free load balancing via dynamic bias terms. 671B total / 37B active params (V3).
- **Code**: `output = Σ shared_experts(x) + Σ top_k gated_routed_experts(x)`
- **NT-Domain**: NT-CORE (MoE architecture), NT-ACT (efficient inference)
- **Priority**: P0
- **Effort**: 200h (MoE layer implementation + routing)

### Switch Transformer
- **Source**: jmlr.org/papers/volume23/21-0998/21-0998.pdf
- **Innovation**: Simplified MoE with top-1 routing (k=1). Reduces router computation by 50%, halves expert capacity. Up to 7x pre-training speedup over T5-Base. Auxiliary load balancing loss ensures uniform routing. First to train trillion-parameter models in bfloat16.
- **Code**: `expert_capacity = (tokens_in_batch / num_experts) * capacity_factor`
- **NT-Domain**: NT-CORE (sparse MoE)
- **Priority**: P0
- **Effort**: 80h (Switch layer implementation)

### Selective Sinkhorn Routing (SSR)
- **Source**: arxiv.org/abs/2511.08972
- **Innovation**: Reformulates routing as optimal transport with Sinkhorn algorithm. No auxiliary losses needed. Apply intermittently (0.1-1% of training steps) for faster convergence. At inference, disable balancing for best results.
- **NT-Domain**: NT-CORE (routing optimization)
- **Priority**: P1
- **Effort**: 40h (SSR routing module)

### Key MoE Patterns
- **Fine-grained segmentation**: Split FFN into m× more experts, activate m× more, same compute
- **Shared expert isolation**: K_s always-active experts for common patterns
- **Auxiliary-loss-free balancing**: Dynamic bias terms adjusted per training step
- **Node-limited routing**: Limit token dispatch to M nodes for communication efficiency

---

## BATCH 3: Inference Optimization

### SGLang RadixAttention
- **Source**: arxiv.org/abs/2312.07104, lmsys.org
- **Innovation**: Radix tree KV cache management enabling automatic prefix reuse across requests. LRU eviction, cache-aware scheduling (longest-shared-prefix-first). 5x throughput improvement. Handles fork patterns, few-shot sharing, multi-turn chat.
- **Code**: RadixCache with match_prefix → insert → LRU eviction
- **NT-Domain**: NT-MEMORY (KV cache), NT-IO (inference optimization)
- **Priority**: P0
- **Effort**: 60h (radix cache for NeoTrix inference)

### EAGLE-3 Speculative Decoding
- **Source**: arxiv.org/abs/2503.01840, ai.meta.com
- **Innovation**: Feature-level drafting on multi-layer fused features (not just top layer). Achieves 3.0-6.5x speedup, 1.4x over EAGLE-2. At batch size 1, Llama4 Maverick decodes at 4ms/token on 8xH100. At large batch sizes (48+), achieves 1.4-2.0x speedup. Code generation (HumanEval) achieves highest 6.5x speedup.
- **Code**: Draft model proposes K tokens → target verifies in single pass → accept longest prefix
- **NT-Domain**: NT-IO (inference acceleration)
- **Priority**: P0
- **Effort**: 40h (EAGLE-3 integration)

### vLLM PagedAttention + Continuous Batching
- **Source**: arxiv.org/abs/2511.17593, vllm.ai, frontiercheckpoint.com
- **Innovation**: KV cache managed like OS virtual memory — non-contiguous paged blocks. Reduces memory waste from 60-80% to <4%. Continuous batching at token level (not request level). LLaMA-2-7B: vLLM achieves 15,243 tokens/sec at 100 concurrent vs TGI's 4,156 (3.67x). At 200 concurrent: 24x advantage. GPU utilization 85-92% vs TGI's 68-74%.
- **Code**: PagedAttention: block table maps logical positions → physical GPU memory blocks
- **NT-Domain**: NT-IO (serving infrastructure)
- **Priority**: P0
- **Effort**: 30h (continuous batching scheduler)

### Continuous Batching
- **Source**: vLLM documentation
- **Innovation**: Dynamically merge incoming requests into active batches mid-generation. Combined with PagedAttention for efficient GPU memory. 4.5x throughput vs sequential queuing at 50 concurrent requests.
- **NT-Domain**: NT-IO (serving infrastructure)
- **Priority**: P0
- **Effort**: 30h (batching scheduler)

---

## BATCH 4: Quantization & Compression

### AWQ vs GPTQ vs FP8
- **Source**: gingerlabs.ai, aws.amazon.com, dataa.dev, mljourney.com
- **Innovation**: 
  - **AWQ**: Activation-aware, protects ~1% salient channels. 4-bit: ~5.40 perplexity, ~83% MMLU. Quantizes in ~10 min (8B). Best accuracy at 4-bit.
  - **GPTQ**: Hessian-guided error compensation. Supports 3-bit/2-bit. Quantizes in ~20-60 min (8B). Broader ecosystem (llama.cpp, Ollama).
  - **FP8**: Native Hopper+ format, effectively lossless. 2x throughput vs BF16.
  - **Marlin kernel**: AWQ ~741 tok/s vs GPTQ ~712 tok/s (Qwen2.5-32B). Without Marlin, GPTQ faster.
  - Llama-3.1-8B: AWQ-W4A16 achieves 83.21 tok/s vs FP16 33.09 tok/s (2.5x).
- **NT-Domain**: NT-IO (model compression), NT-MEMORY (storage efficiency)
- **Priority**: P0
- **Effort**: 24h (quantization pipeline)

### Key Quantization Decision Framework
| Scenario | Format | Why |
|----------|--------|-----|
| H100/B200 production | FP8 W8A8 | Native hardware, lossless, 2x throughput |
| A100/older GPUs | AWQ INT4 + Marlin | 2.8x compression, best accuracy at 4-bit |
| Extreme compression (3-bit) | GPTQ INT3 | Only practical option for sub-4-bit |
| Local/Ollama | GGUF Q4_K_M | CPU+GPU hybrid, fine-grained levels |
| Max accuracy | BF16 | No compression loss |
| QLoRA fine-tuning | bitsandbytes NF4 | Only format supporting gradient flow |

---

## BATCH 5: RAG & Retrieval

### Hybrid Search (BM25 + Vector)
- **Source**: denser.ai, dataaspirant.com, redis.io
- **Innovation**: BM25 excels at exact-match (SKUs, codes); vector search excels at semantic. Fused via Reciprocal Rank Fusion (RRF) at k=60. 7.4% NDCG lift over either alone. Cross-encoder reranking adds another +17% Recall@5.
- **Code**: `fused_score(doc) = Σ 1/(k + rank_i)` across retrievers
- **NT-Domain**: NT-MEMORY (retrieval), NT-WORLD (knowledge access)
- **Priority**: P0
- **Effort**: 40h (hybrid retrieval pipeline)

### GraphRAG
- **Source**: microsoft.github.io/graphrag, dl.acm.org/doi/10.1145/3777378
- **Innovation**: Knowledge graph + RAG. Three stages: G-Indexing (entity/relationship extraction + Leiden clustering), G-Retrieval (graph traversal + GNN embeddings), G-Generation (community summaries for global questions, fan-out for local). Global Search for holistic questions, Local Search for entity-specific, DRIFT for reasoning with community context. Handles multi-hop reasoning that flat RAG cannot.
- **NT-Domain**: NT-MEMORY (graph retrieval), NT-WORLD (knowledge extraction)
- **Priority**: P1
- **Effort**: 80h (GraphRAG pipeline integration)

### Self-RAG / CRAG
- **Innovation**: Adaptive retrieval — model decides when to retrieve vs use internal knowledge. Corrective RAG validates retrieved passages before generation, replacing low-quality ones.
- **NT-Domain**: NT-MEMORY (adaptive retrieval)
- **Priority**: P1
- **Effort**: 32h

---

## BATCH 6: Agent Architecture

### ReAct Framework
- **Source**: arxiv.org/abs/2210.03629, Google Research
- **Innovation**: Interleave reasoning traces (Thought) with actions (Act) and observations (Observe). Reasoning helps plan/update actions; actions gather external information. Outperforms CoT-only and Act-only by 34% on ALFWorld. Human-in-the-loop via thought editing.
- **Code**: `for step in range(max): thought = LLM(reason); action = LLM(plan); obs = env.step(action); context.append(thought, action, obs)`
- **NT-Domain**: NT-ACT (agent loop), NT-CORE (reasoning)
- **Priority**: P0
- **Effort**: 40h (ReAct agent implementation)

### Plan-and-Execute / LATS / Reflexion
- **Innovation**: Plan-then-execute separates planning from execution. LATS uses tree search over reasoning paths. Reflexion adds self-reflection loops for iterative improvement.
- **NT-Domain**: NT-ACT (agent patterns)
- **Priority**: P1
- **Effort**: 60h (agent pattern library)

---

## BATCH 7: Embedding & Vector Models

### Embedding Model Landscape 2026
- **Source**: iotdigitaltwinplm.com, promptquorum.com, knightli.com
- **Innovation**: Three tiers: (1) Encoder models (BGE-M3 568M, E5-large) for efficiency, (2) Decoder-LLM embedders (NV-Embed-v2 7B, E5-Mistral-7B) for max quality, (3) Distilled (Stella 1.5B) for Pareto-optimal. BGE-M3 supports dense+sparse+ColBERT from single model. Matryoshka truncation (256-1024 dims).
- **NT-Domain**: NT-MEMORY (embeddings), NT-WORLD (multimodal)
- **Priority**: P0
- **Effort**: 24h (embedding model selection/integration)

### Key Embedding Decisions
| Need | Model | Why |
|------|-------|-----|
| Best overall | Stella v5 1.5B | Quality/latency Pareto point, Matryoshka |
| Multilingual | BGE-M3 | 100+ languages, sparse+dense+multi-vector |
| Max quality | NV-Embed-v2 | Top MTEB, latent-attention pooling |
| CPU-only | nomic-embed-text-v2 | 580 chunks/sec, MoE architecture |
| Hybrid retrieval | BGE-M3 | Native sparse + dense output |

---

## BATCH 8: Training Techniques (RLHF/DPO/GRPO)

### DPO vs PPO vs GRPO Comparison
- **Source**: aclanthology.org (EMNLP 2025, 17-algorithm benchmark), arxiv.org/abs/2603.19335 (oxRL, 240 runs), theorempath.com, algorithmine.com
- **Innovation**: 
  - **PPO**: 4 models (policy, reference, reward, value), online RL. Heavy compute, brittle.
  - **DPO**: 2 models, supervised on preference pairs, no generation during training. Simple but limited on reasoning tasks.
  - **GRPO**: 3 models, group-normalized advantage replaces value model. 50-70% less compute than PPO. Best for verifiable reasoning (math/code).
  - **Top performers** (EMNLP 2025): IPO, DPO, REINFORCE, GRPO, Best-of-N.
  - **Scale-dependent rankings** (oxRL): At 1.5B, SGRPO tops all; at 7B, SimPO becomes best — complete ranking inversion.
  - **Hierarchy of leverage**: scale (~50pp) >> paradigm (~10pp) >> online/offline (~9pp) >> loss function (~1pp).
- **NT-Domain**: NT-MIND (alignment), NT-CORE (training)
- **Priority**: P1
- **Effort**: 80h (alignment pipeline)

### Key Alignment Decision Framework
| Situation | Method | Why |
|-----------|--------|-----|
| Small team, preference pairs | DPO | Simplest, 2 models, no generation loop |
| Checkable correctness (math/code) | GRPO | Rule-based reward, no value model, 50-70% less compute |
| Reusable reward signal | PPO | Reward model as separate artifact |
| Stylistic alignment (≥7B + LoRA) | SimPO | Best accuracy at 7B scale |
| Not done SFT yet | None | All assume competent starting policy |
| Validate at deployment scale | - | Rankings at ≤3B don't predict 7B behavior |

---

## BATCH 9: Multimodal

### LLaVA-OneVision-2
- **Source**: arxiv.org/abs/2605.25979, github.com/EvolvingLMMs-Lab
- **Innovation**: Codec-stream tokenization for video — treats compressed video as continuous bit-cost stream. Motion-residual cues select salient patches. 3× temporal range under same token budget. Unified 3D RoPE for images/video/spatial. 8B params outperforms Qwen3-VL-8B by +4.3pts on video tasks.
- **NT-Domain**: NT-WORLD (multimodal perception), NT-IO (vision encoding)
- **Priority**: P1
- **Effort**: 120h (multimodal integration)

---

## BATCH 10: Code & Math

### SWE-bench Landscape 2026
- **Source**: swebench.com, codesota.com, witho2.com
- **Innovation**: SWE-bench Verified (500 tasks) is gold standard but contaminated. SWE-bench Pro (1,865 tasks) is contamination-resistant. Claude Opus 4.7 leads at 87.6% Verified, 69.2% Pro. HumanEval saturated at 90%+. Agent harness shifts results by 10-20 points.
- **NT-Domain**: NT-ACT (code generation), NT-MIND (evaluation)
- **Priority**: P1
- **Effort**: 40h (code benchmark integration)

---

## BATCH 11: Novel Architectures (SSM/Hybrid)

### TransMamba
- **Source**: arxiv.org/abs/2503.24067
- **Innovation**: Sequence-level hybrid unifying Transformer and Mamba through shared parameter matrices (QKV ↔ CBx). Dynamic switching between attention and SSM at TransPoints. Memory Converter bridges information loss during mode switch. At 1.5B: outperforms both pure Transformer and Mamba on all tasks.
- **NT-Domain**: NT-CORE (hybrid architecture)
- **Priority**: P1
- **Effort**: 120h (TransMamba implementation)

### Architecture Decision Matrix
| Use Case | Architecture | Model |
|----------|-------------|-------|
| Long-context 256K+ | Hybrid SSM+Attention | Jamba 1.5 Large |
| Edge/CPU inference | Pure Recurrent | RWKV 7 G1 |
| Time-series/sequence | Pure SSM | Mamba 2 |
| General short-context | Pure Transformer | LLaMA/Qwen |
| Dynamic switching | Sequence-level hybrid | TransMamba |

---

## BATCH 12: Distributed Training

### Llama 3 4D Parallelism
- **Source**: phanashayee.me/papers/2025_ISCA_Llama3_Par.pdf
- **Innovation**: 4D parallelism: FSDP + TP + PP + Context Parallelism. TP=8 optimal (intra-node NVLink). 3D parallelism preferred over 2D when PP communication is cheaper than FSDP. Evolving batch sizes across training phases. Document-mask attention for multimodal.
- **NT-Domain**: NT-CORE (distributed training)
- **Priority**: P1
- **Effort**: 160h (4D parallelism implementation)

### Synergistic TP+PP Scheduling
- **Source**: NeurIPS 2025, arxiv.org/abs/2510.27257
- **Innovation**: Braids forward/backward computation units to eliminate TP bubbles (27.5% overhead at TP=8). V-shape dataflow across stages. 12% throughput improvement for LLMs, 16% for MLLMs.
- **NT-Domain**: NT-CORE (parallel scheduling)
- **Priority**: P1
- **Effort**: 80h (synergistic scheduler)

### Seq1F1B Pipeline Parallelism
- **Source**: aclanthology.org/2025.naacl-long.454
- **Innovation**: Sequence-level 1F1B scheduling for long-context training. 1.14x throughput with half memory footprint vs Megatron 1F1B. Trains 30B model on 64K sequences with 64x A100 without recomputation.
- **NT-Domain**: NT-CORE (long-context training)
- **Priority**: P1
- **Effort**: 60h (Seq1F1B integration)

### Elastic Pipeline Parallelism (EPP)
- **Source**: arxiv.org/abs/2509.21275
- **Innovation**: Adaptive granularity — token-level PP for long sequences, batch-level PP for short. Co-optimizes pipeline schedule + gradient checkpointing. 1.69x speedup over SOTA.
- **NT-Domain**: NT-CORE (elastic training)
- **Priority**: P2
- **Effort**: 80h (EPP implementation)

---

## BATCH 13: Ecosystem & Tools

### vLLM vs Ollama Production Decision
- **Source**: developers.redhat.com, baeseokjae.github.io, markaicode.com
- **Innovation**: vLLM: PagedAttention + continuous batching = 24x throughput at 200 concurrent users. Ollama: simplicity for local dev, breaks at >10 concurrent. Migration path: both expose OpenAI-compatible API. Production pattern: Ollama for dev, vLLM for prod.
- **NT-Domain**: NT-IO (serving infrastructure)
- **Priority**: P0
- **Effort**: 40h (production serving architecture)

---

## NeoTrix Integration Proposals (Top 15 P0)

| # | Technique | NT-Domain | Integration | Effort |
|---|-----------|-----------|-------------|--------|
| 1 | DeepSeekMoE routing | NT-CORE | Fine-grained expert + shared expert isolation for NT-CORE reasoning | 200h |
| 2 | FlashAttention-3 | NT-CORE | Hopper-optimized attention kernel for KV cache | 40h |
| 3 | SGLang RadixAttention | NT-MEMORY | Radix tree KV cache for prefix reuse across sessions | 60h |
| 4 | EAGLE-3 Speculative Decoding | NT-IO | Multi-layer feature drafting for 3-6.5x inference speedup | 40h |
| 5 | Hybrid Search (BM25+Vector) | NT-MEMORY | RRF fusion for NeoTrix KB retrieval | 40h |
| 6 | ReAct Agent Loop | NT-ACT | Thought-Action-Observation cycle for NT-ACT | 40h |
| 7 | AWQ/FP8 Quantization | NT-IO | Model compression pipeline for deployment | 24h |
| 8 | BGE-M3 Embeddings | NT-MEMORY | Dense+sparse+multi-vector for KB | 24h |
| 9 | Mamba-2 Hybrid (43% SSM + 7% Attn) | NT-CORE | Long-context inference with 8x speedup at 128K tokens | 120h |
| 10 | vLLM PagedAttention | NT-IO | Paged KV cache + continuous batching for serving | 30h |
| 11 | Switch Transformer | NT-CORE | Top-1 routing for sparse MoE, up to 7x training speedup | 80h |
| 12 | GraphRAG | NT-MEMORY | Knowledge graph retrieval for multi-hop reasoning | 80h |
| 13 | GRPO Alignment | NT-MIND | Group-relative policy optimization for reasoning tasks | 60h |
| 14 | GQA-8 Attention | NT-CORE | 8 KV groups for 8x cache reduction with near-MHA quality | 20h |
| 15 | 4D Parallelism (FSDP+TP+PP+CP) | NT-CORE | Llama 3-style distributed training | 160h |

---

## Cross-Cutting Patterns

### Pattern 1: Hybrid Everything
- **Architecture**: Hybrid SSM+Transformer (Jamba, TransMamba) beats pure architectures
- **Retrieval**: Hybrid BM25+Vector beats pure approaches
- **Serving**: Hybrid Ollama(dev)+vLLM(prod) is production standard
- **Quantization**: Hybrid AWQ weights + FP8 KV cache for optimal memory
- **NeoTrix Implication**: Always default to hybrid unless measurement proves otherwise

### Pattern 2: Prefix Reuse / Caching
- SGLang RadixAttention: 5x throughput via KV prefix cache
- FlashAttention-3: FP8 with incoherent processing
- Speculative decoding: Draft-verify pattern
- vLLM APC: Automatic prefix caching for repeated contexts
- **NeoTrix Implication**: KV cache reuse across NT-MEMORY sessions is critical

### Pattern 3: Adaptive Computation
- Mixture of Depths: Vary computation per token
- MoE routing: Different experts per token
- Self-RAG: Adaptive retrieval vs internal knowledge
- EPP: Adaptive PP granularity based on sequence length
- **NeoTrix Implication**: NT-CORE should route computation dynamically

### Pattern 4: Lossless Acceleration
- Speculative decoding: Exact same output, faster generation
- FP8 quantization: Effectively lossless on H100+
- FlashAttention-3: Same math, hardware-optimized
- GQA: Near-MHA quality with 8x cache reduction
- **NeoTrix Implication**: Prioritize lossless optimizations before approximations

### Pattern 5: Scale-Dependent Choices
- Algorithm rankings invert across model scales (1.5B vs 7B)
- TP bubbles grow with TP size (27.5% at TP=8)
- KV cache dominates at long context
- **NeoTrix Implication**: Benchmark at deployment scale, not toy scale

### Pattern 6: Memory is the Bottleneck
- PagedAttention: 60-80% KV memory wasted without paging
- GQA-8: 8x KV cache reduction
- AWQ: 4x weight compression
- Quantized KV cache: Additional 2x reduction
- **NeoTrix Implication**: Every optimization should measure memory impact
