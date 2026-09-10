# Model Reverse Engineering — 2026-09-11 (Batch 217)

> 10 frontier models analyzed for architecture innovations, training methods, inference optimizations, mappable patterns, and NeoTrix integration points.

---

## 1. Claude 4 (Anthropic) — Opus 4 / Sonnet 4 / 4.6 / 4.7 / 4.8

### Architecture Innovation (vs 上代)
- **Hybrid reasoning model**: Two modes — near-instant responses + extended thinking for deeper reasoning
- **Thinking summaries**: Smaller model condenses lengthy thought processes (triggered ~5% of time)
- **Adaptive thinking** (4.6+): Model self-selects when deeper reasoning is needed, 4 effort levels (low/medium/high/max)
- **1M token context** (4.6 beta): First Opus-class with 1M context
- **128K output tokens** (4.6+)
- **Context compaction** (4.6+): Auto-summarizes older context when approaching threshold
- **Multi-model routing**: Fast model for most questions + deeper reasoning model + real-time router

### Training Method
- Constitutional AI (based on UN Universal Declaration of Human Rights)
- Human feedback + selected character trait training
- Synthetic data from other models for post-training
- ASL-3 safety standard for Opus 4 (highest level deployed)
- Large-scale pretraining + substantial post-training/fine-tuning

### Inference Optimization
- Context compaction for long-running tasks
- Parallel tool execution
- Effort-level control (API parameter)
- Developer Mode for full CoT access
- US-only inference option (1.1× pricing)

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Adaptive effort routing | Model self-selects reasoning depth | GWT salience + effort routing |
| Context compaction | Auto-summarize old context | KVMem paged KV (compaction path) |
| Multi-model orchestration | Fast + reasoning + router | NT-CORE E8 hexagram routing |
| Thinking summarization | Small model condenses CoT | SEAL distillation pipeline |

### NeoTrix Integration
- **NT-CORE GWT**: Adaptive thinking maps directly to GWT salience modulation — low-salience tasks get fast path, high-salience get extended reasoning
- **NT-MIND SEAL**: Thinking summaries = distillation artifact reuse
- **NT-MEMORY KB**: Context compaction = KB-backed session memory with automatic pruning
- **NT-SHIELD**: ASL-3 safety framework maps to NT-SHIELD risk grading (D1-D12)

---

## 2. GPT-5 (OpenAI)

### Architecture Innovation (vs 上代)
- **Unified multi-model system**: gpt-5-main (fast) + gpt-5-thinking (reasoning) + real-time router
- **Router continuously trained** on user signals (model switches, preference rates, correctness)
- **Safe-completions**: Safety training centered on output safety rather than intent classification
- **Minimal reasoning parameter**: Control granularity for reasoning depth
- **Verbosity parameter**: Control output length
- **Speculative decoding** (inferred): EAGLE/Medusa-style based on throughput patterns
- **MoE backbone** (inferred): Multiple inference paths suggest MoE architecture

### Training Method
- Reinforcement learning for reasoning models (gpt-5-thinking)
- RL teaches to "think before answering" — long internal chain of thought
- Safe-completions approach integrated into training
- Multi-modal training: text + image + audio + video
- 1M token context window

### Inference Optimization
- Real-time router selects model based on conversation type/complexity/tool needs
- Parallel test time compute (gpt-5-thinking-pro)
- Prompt caching (paged-attention prefix-cache)
- Tiered models: standard, mini, nano for cost optimization
- GPT-5.5 leak: 4× active parameter count at equivalent cost via new MoE router

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Continuous router training | Router improves from user signals | NT-MIND SEAL self-evolution |
| Safe-completions | Output-safety-focused training | NT-SHIELD egress guard |
| Multi-tier model routing | Standard/mini/nano selection | GWT cost-aware routing (A1) |
| Speculative decoding | Draft model for token prediction | NT-ACT speculative execution |

### NeoTrix Integration
- **NT-CORE GWT**: Router maps to GWT attention routing with cost weight (Axiom A1)
- **NT-MIND**: Continuous router training = SEAL pipeline feedback loop
- **NT-ACT**: Speculative decoding pattern for tool-call optimization
- **NT-SHIELD**: Safe-completions aligns with egress privacy guard philosophy

---

## 3. Gemini 2.5 Pro (Google)

### Architecture Innovation (vs 上代)
- **Sparse MoE Transformer**: Activates subset of parameters per token, decoupling capacity from cost
- **Native multimodal**: Text, vision, audio inputs natively (not pipeline)
- **1M+ token context**: Process entire codebases, 3-hour videos
- **Thinking model**: Native thinking across all domains (not separate model)
- **Deep Think**: Parallel hypothesis generation + critique (Olympiad math SOTA)
- **Controllable thinking budget**: User sets token budget for internal computation
- **TPUv5p training**: First model family trained on TPUv5p
- **Distillation for smaller models**: k-sparse distribution approximation

### Training Method
- TPUv5p architecture, 8960-chip pods across multiple datacenters
- Synchronous data-parallel training
- Slice-granularity elasticity (auto-continues with fewer slices during failures, ~97% throughput)
- SDC (Silent Data Corruption) mitigation
- Distillation from large to small models (k-sparse distribution)
- Multi-stage training with evolving data mixtures

### Inference Optimization
- Controllable thinking budget (quality vs cost tradeoff)
- Flex inference, batch API, priority inference
- Context caching (implicit + explicit)
- 3-hour video processing
- Up to 8.6× prefill throughput improvement (Qwen comparison data)

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Controllable thinking budget | User sets reasoning depth | GWT effort levels + cost routing |
| Native multimodal fusion | Unified text/vision/audio | NT-WORLD sensory integration |
| Distillation pipeline | Large→small model transfer | SEAL distillation stage |
| Slice elasticity | Auto-recover from hardware failure | NT-REPAIR self-healing |
| Deep Think | Parallel hypothesis + critique | NT-CORE E8 multi-hexagram reasoning |

### NeoTrix Integration
- **NT-CORE E8**: Deep Think = multiple hexagram hypotheses with critique
- **NT-WORLD**: Native multimodal = SensoryIntegrationHub input
- **NT-MIND SEAL**: Distillation = crystallization stage
- **NT-REPAIR**: Slice elasticity = MAPE-K self-healing pattern
- **NT-PHYSICAL**: GPU cluster management maps to hardware awareness

---

## 4. DeepSeek V4 (DeepSeek)

### Architecture Innovation (vs 上代)
- **Hybrid attention**: CSA (Compressed Sparse Attention) + HCA (Heavily Compressed Attention) interleaved
- **Manifold-Constrained Hyper-Connections (mHC)**: Upgrades residual connections with doubly-stochastic projection via Sinkhorn-Knopp
- **Hash-MoE bootstrap**: First few layers use static token→expert hash table, then switch to learned routing
- **Sqrt(Softplus(·))** affinity function (replaces Sigmoid)
- **Multi-Token Prediction (MTP)** retained from V3
- **FP4 quantization-aware training** for MoE expert weights
- **1.6T total params / 49B activated** (Pro) — **284B total / 13B activated** (Flash)
- **1M token context** at 27% FLOPs and 10% KV cache vs V3

### Training Method
- 32T+ diverse tokens pretraining
- Muon optimizer (faster convergence, better stability)
- Hybrid ZeRO strategy for Muon
- Two-stage contextual parallelism for compressed attention
- TileLang DSL for kernel development
- Deterministic kernel libraries for bitwise reproducibility

### Inference Optimization
- CSA: compress KV every m tokens + sparse attention (top-k blocks)
- HCA: extreme compression (m' >> m) + dense attention on compressed
- Heterogeneous KV cache with on-disk storage for shared-prefix reuse
- Fused MoE kernel overlapping compute/comm/memory
- 10× KV cache reduction at 1M context

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Hybrid attention (CSA+HCA) | Two-tier compression for long context | KVMem paged KV + compaction |
| mHC residual streams | Doubly-stochastic residual mixing | NT-CORE HyperCube connections |
| Hash-MoE bootstrap | Static routing for initial layers | Capability tree initialization |
| Muon optimizer | Orthogonalization-based optimization | NT-MIND training optimization |
| FP4 quantization-aware | Low-precision training | NT-ACT resource budget optimization |

### NeoTrix Integration
- **NT-MEMORY KB**: CSA/HCA = hierarchical attention for KB search (near/far context)
- **NT-CORE HyperCube**: mHC = manifold-constrained knowledge graph connections
- **NT-MIND**: Muon optimizer = SEAL training efficiency improvement
- **NT-ACT**: Hash-MoE bootstrap = capability tree static→dynamic routing
- **NT-PHYSICAL**: FP4 quantization = resource-aware deployment

---

## 5. Llama 4 Behemoth (Meta)

### Architecture Innovation (vs 上代)
- **First MoE in Llama family**: Alternating dense + MoE layers
- **Behemoth**: 288B active / ~2T total params, 16 experts
- **Maverick**: 17B active / 400B total, 128 routed experts + 1 shared
- **Scout**: 17B active / 109B total, 16 experts, 10M context
- **Native multimodal** via early fusion
- **Codistillation**: Student (Maverick) trained simultaneously with teacher (Behemoth)
- **Novel distillation loss**: Dynamically weights soft + hard targets through training

### Training Method
- 30T+ tokens (text + image + video)
- FP8 precision training (390 TFLOPs/GPU on 32K GPUs)
- Lightweight SFT → large-scale RL → lightweight DPO
- 95% SFT data pruning (vs 50% for smaller models)
- Fully asynchronous online RL framework (10× efficiency improvement)
- MoE parallelization optimization for speed
- Curriculum learning: pass@k analysis for increasing prompt hardness

### Inference Optimization
- MoE: Only activated params computed per token
- Maverick fits single H100 host
- Scout fits single H100 (Int4 quantization)
- Alternating dense/MoE layers for efficiency

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Codistillation | Teacher-student simultaneous training | SEAL teacher-student loop |
| Dynamic distillation loss | Adaptive soft/hard target weighting | NT-MIND crystallization |
| Async online RL | Fully asynchronous training framework | NT-ACT parallel task management |
| Curriculum hardness | Progressive difficulty training | SEAL progressive complexity |
| MoE layer alternation | Dense+MoE interleaving | GWT attention/diffusion alternation |

### NeoTrix Integration
- **NT-MIND SEAL**: Codistillation = teacher node in SEAL pipeline
- **NT-ACT**: Async RL framework = production_orchestrator pattern
- **NT-CORE GWT**: Dense/MoE alternation = attention/diffusion cycle
- **NT-MEMORY**: 10M context (Scout) = long-horizon KB sessions

---

## 6. Qwen 3.8 (Alibaba)

### Architecture Innovation (vs 上代)
- **GDN + QSA Hybrid Attention**: Gated DeltaNet (compress history) + Qwen Sparse Attention (micro-block selection)
- **Gated Residual (GR)**: 4-branch residual stream with elementwise dynamic gate
- **N-gram Embedding**: 51B additional params, offloaded to host memory, async prefetch
- **Ultra-sparse MoE**: Large expert pool, small routed experts per token + shared expert
- **125B total / 6B activated** (Flash-Next) — **2.4T total / 95B activated** (Max)
- **256K native context**, extensible to 1M via YaRN
- **Muon optimizer** as primary (vs AdamW for embeddings/router)
- **250K vocabulary** (vs 150K) for 10-60% encoding efficiency

### Training Method
- Muon optimizer with orthogonalized accuracy for 2D linear maps
- AdamW for embeddings, router, low-rank GR params
- Fused parameter splitting for independent linear transformations
- Scaling law refitted for new architecture
- Zero-centered RMSNorm with weight decay
- Attention output gating for stability
- Normalized MoE router initialization

### Inference Optimization
- QSA: 7.6× prefill / 4.9× decode speedup at 1M tokens
- 8.6× prefill throughput vs Qwen3.7-Plus at 1M context (90% prefix cache hit)
- N-gram embedding offloaded to host memory (async prefetch)
- FP8 storage for residual state
- 1/9 training FLOPs vs predecessor

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| GDN + QSA hybrid | Compress + sparse selection | KVMem hot/cold tiering |
| Gated Residual branches | 4-branch parallel + dynamic gate | NT-CORE multi-hexagram branching |
| N-gram embedding offload | Host-memory async prefetch | KB embedding hot/cold storage |
| Muon optimizer | Orthogonalized 2D optimization | NT-MIND training efficiency |
| Ultra-sparse MoE | Large pool, few active | GWT sparse attention routing |

### NeoTrix Integration
- **NT-MEMORY**: N-gram offload = KB embedding tiered storage (hot GPU / warm host / cold disk)
- **NT-CORE HyperCube**: Gated Residual = multi-branch knowledge representation
- **NT-MIND**: Muon optimizer = SEAL training optimization
- **NT-WORLD**: QSA micro-block selection = NT-WORLD content chunking strategy
- **NT-ACT**: Ultra-sparse MoE = resource-aware tool routing

---

## 7. Mistral Large 3 (Mistral AI)

### Architecture Innovation (vs 上代)
- **Granular MoE**: 675B total / 41B active parameters (first MoE since Mixtral)
- **2.5B Vision Encoder**: Separate vision component
- **256K context window**
- **FP8 native training** on 3000 H200s
- **Eagle speculative decoding**: Custom draft model for inference speedup
- **Apache 2.0 license** (fully open-weight)
- **NVFP4 quantized checkpoint**: Runs on single 8×A100/H100 node

### Training Method
- Trained from scratch on 3000 NVIDIA H200 GPUs
- NVIDIA co-design: Blackwell attention + MoE kernels
- Prefill/decode disaggregated serving
- Speculative decoding with Eagle draft model
- SFT + preference tuning for instruction following

### Inference Optimization
- Eagle speculative decoding (3 speculative tokens)
- NVFP4 quantization for single-node deployment
- vLLM serving with auto-tool-choice
- Prefill/decode disaggregation
- Custom MoE kernels from NVIDIA

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Eagle speculative decoding | Draft model for token prediction | NT-ACT speculative execution |
| Granular MoE | Fine-grained expert routing | GWT fine-grained attention |
| Disaggregated serving | Separate prefill/decode | NT-IO pipeline separation |
| NVFP4 quantization | Aggressive quantization | Resource budget optimization |
| Open-weight Apache 2.0 | Community deployment | NT-ACT platform_gateway |

### NeoTrix Integration
- **NT-ACT**: Eagle speculative decoding = speculative tool execution
- **NT-IO**: Disaggregated serving = platform_gateway prefill/decode separation
- **NT-PHYSICAL**: NVFP4 = resource-aware deployment (edge/server)
- **NT-SHIELD**: Open-weight = trust tier classification for model providers

---

## 8. Grok 3 (xAI)

### Architecture Innovation (vs 上代)
- **10× compute over Grok 2**: 200K H100 GPU Colossus cluster
- **RL at pretraining scale**: Not just post-training — RL used during pretraining for reasoning
- **Think mode**: Extended chain-of-thought with backtracking, error correction, alternative exploration
- **1M token context** (8× increase)
- **600B parameters, 16 experts, 2 active** (estimated)
- **GQA**: 96 layers, 12288 hidden, 96 query heads, 16 KV heads
- **Real-time X/Twitter data** as training differentiator
- **DeepSearch agent**: Real-time internet + X data synthesis

### Training Method
- 12.8T tokens (web + X/Twitter data)
- Colossus cluster: 200K H100 GPUs, assembled in 122 days
- Large-scale RL for chain-of-thought refinement
- RL teaches backtracking, error simplification, self-correction
- Multi-approach consideration during generation
- Grok 4: RL at ~pretraining scale, 100× compute over Grok 2

### Inference Optimization
- Think mode: seconds to minutes reasoning
- DeepSearch: real-time web + X data retrieval
- Cost-efficient Grok 3 Mini ($0.30/$0.50 per M tokens)
- Vision understanding (MMMU competitive)

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| RL at pretraining scale | RL during foundation training | NT-MIND continuous evolution |
| Think mode with backtracking | CoT with error correction | NT-CORE E8 backtrack reasoning |
| Real-time data training | Live social data in training | NT-WORLD crawl pipeline |
| DeepSearch agent | Real-time synthesis agent | NT-WORLD UnifiedCrawler |

### NeoTrix Integration
- **NT-CORE E8**: Think mode backtracking = hexagram state exploration with rollback
- **NT-WORLD**: DeepSearch = UnifiedCrawler with real-time synthesis
- **NT-MIND**: RL at scale = continuous self-evolution (not just post-training)
- **NT-ACT**: Agent tools = production_orchestrator native tool integration

---

## 9. Cohere Command R+ (Cohere)

### Architecture Innovation (vs 上代)
- **RAG-optimized architecture**: Trained specifically for retrieval augmented generation
- **Multi-step tool use**: Action → Observation → Reflection loop
- **Grounded generation**: Citation-aware output with grounding spans
- **23 languages** trained, 10 evaluated
- **128K context** (Command R+ 08-2024)
- **GQA**: Grouped Query Attention for inference speed
- **Optimized transformer** (not MoE — dense architecture)
- **50% higher throughput, 25% lower latency** (08-2024 refresh)

### Training Method
- SFT + preference fine-tuning for grounded generation
- Specific prompt template training for RAG workflows
- Multi-step tool use training (Action → Observation → Reflection)
- Safety modes (granular control)
- Data through Feb 2023

### Inference Optimization
- GQA for faster inference
- 50% throughput improvement over previous version
- 25% latency reduction
- `fast` citation mode (fewer tokens, less accurate)
- `accurate` citation mode (full reasoning)

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| RAG-native architecture | Built for retrieval workflows | NT-MEMORY KB-native search |
| Multi-step tool loop | Action→Observation→Reflection | NT-ACT orchestration loop |
| Grounded generation | Citation-aware output | NT-SHIELD provenance tracking |
| Fast vs accurate modes | Quality/cost tradeoff | GWT effort routing |

### NeoTrix Integration
- **NT-MEMORY**: RAG-native = KB search + grounding built into model
- **NT-ACT**: Multi-step tool = production_orchestrator Action→Observation→Reflection
- **NT-SHIELD**: Grounded generation = cite-ledger provenance tracking
- **NT-IO**: Safety modes = trust-tier model selection

---

## 10. Yi-Lightning (零一万物)

### Architecture Innovation (vs 上代)
- **Enhanced MoE**: Fine-grained expert segmentation + balanced routing
- **Hybrid Attention**: 3 sliding window layers + 1 full attention layer (alternating)
- **Cross-Layer KV Cache Sharing (CLA)**: Share KV states between consecutive full attention layers
- **Dynamic Top-P routing**: Adaptive expert count based on task difficulty
- **KV cache 2-4× reduction** + some layers linear complexity
- **FP8 quantization compatibility** designed into architecture
- **100B parameters** (MoE)

### Training Method
- Multi-stage training: early diversity → late knowledge richness
- Different batch sizes and LR schedules per stage
- EP load balancing (relaxed from per-expert to EP-group)
- Partitioned EP load balancing (PEP) for finer control
- RLHF with deliberate multi-stage strategy
- Synthetic data construction
- RAISE safety framework (4-component)

### Inference Optimization
- Dynamic Top-P: adaptively adjusts activated experts per task
- CLA: KV cache memory halved for full attention
- Hybrid attention: 82.8% memory reduction on long sequences
- 1200 TFLOPS/card at FP8 on Hopper
- First token latency 2× improvement, generation speed 40% faster

### Mappable Patterns
| Pattern | Definition | NeoTrix Mapping |
|---------|-----------|-----------------|
| Dynamic Top-P routing | Adaptive expert activation | GWT adaptive attention budget |
| Cross-Layer KV sharing | Shared KV across layers | KB embedding deduplication |
| Multi-stage training | Phase-specific optimization | SEAL multi-stage pipeline |
| FP8 architecture design | Hardware-aware architecture | NT-PHYSICAL hardware alignment |
| Hybrid attention layers | Sliding+full alternating | GWT local/global attention |

### NeoTrix Integration
- **NT-CORE GWT**: Dynamic Top-P = adaptive attention routing based on task difficulty
- **NT-MEMORY**: CLA KV sharing = KB embedding cross-reference deduplication
- **NT-MIND SEAL**: Multi-stage training = SEAL pipeline phase transitions
- **NT-PHYSICAL**: FP8 design = hardware-aware architecture (Hopper optimization)
- **NT-WORLD**: Hybrid attention = local sliding window + global full attention for content

---

## Cross-Model Synthesis: 2026 Architecture Trends

### Universal Patterns (8+ models share)

| Pattern | Models | NeoTrix Equivalent |
|---------|--------|-------------------|
| **Sparse MoE** | Gemini, DeepSeek, Llama4, Qwen, Mistral, Grok, Yi | GWT attention routing |
| **Hybrid attention** (local+global) | DeepSeek, Qwen, Yi, Gemini | GWT attention hierarchy |
| **Extended/adaptive thinking** | Claude4, GPT-5, Gemini, Grok3 | NT-CORE E8 reasoning |
| **Multi-tier model routing** | Claude4, GPT-5, Gemini | GWT cost-aware routing |
| **Context compaction/long context** | Claude4 (1M), Gemini (1M), DeepSeek (1M), Llama4 (10M), Qwen (1M) | KVMem paged KV |

### Emerging Patterns (3-7 models)

| Pattern | Models | NeoTrix Equivalent |
|---------|--------|-------------------|
| **Muon optimizer** | DeepSeek, Qwen | NT-MIND training optimization |
| **Speculative decoding** | Mistral (Eagle), GPT-5 (inferred) | NT-ACT speculative execution |
| **Residual stream widening** | DeepSeek (mHC), Qwen (GR) | NT-CORE HyperCube |
| **Hash/bootstrap routing** | DeepSeek (Hash-MoE) | Capability tree init |
| **N-gram offload** | Qwen | KB embedding tiered storage |
| **FP4/FP8 quantization** | DeepSeek, Llama4, Mistral | Resource budget optimization |

### Novel Patterns (1-2 models, high signal)

| Pattern | Models | NeoTrix Equivalent |
|---------|--------|-------------------|
| **RL at pretraining scale** | Grok3, Llama4 | NT-MIND continuous evolution |
| **Codistillation** | Llama4 | SEAL teacher-student |
| **Thinking summarization** | Claude4 | SEAL distillation |
| **Safe-completions** | GPT-5 | NT-SHIELD output safety |
| **RAG-native architecture** | Cohere | NT-MEMORY KB-native |
| **Real-time data training** | Grok3 | NT-WORLD live crawl |

---

## NeoTrix Absorption Priority

### P0 — Immediate Integration (architecture-level patterns)
1. **Hybrid Attention (CSA/HCA/QSA)** → NT-MEMORY KB search: implement tiered attention for KB queries
2. **Adaptive Effort Routing** → NT-CORE GWT: 4-level effort control (low/medium/high/max)
3. **Muon Optimizer** → NT-MIND SEAL: faster convergence for self-evolution training
4. **Speculative Execution** → NT-ACT: draft-based tool-call prediction

### P1 — Near-term Absorption (training patterns)
5. **Codistillation Loop** → SEAL teacher-student nodes
6. **RL at Pretraining Scale** → NT-MIND continuous self-evolution
7. **Multi-Stage Training** → SEAL phase-specific optimization
8. **Context Compaction** → KVMem compaction path

### P2 — Strategic Absorption (infrastructure patterns)
9. **Hash-MoE Bootstrap** → Capability tree static→dynamic transition
10. **N-gram Host Offload** → KB embedding hot/warm/cold tiering
11. **Slice Elasticity** → NT-REPAIR self-healing
12. **Async Online RL** → NT-ACT parallel task management

---

## Source Audit

| Model | Primary Source | Date | Confidence |
|-------|---------------|------|------------|
| Claude 4 | Anthropic system cards, anthropic.com | 2025-05 → 2026-02 | HIGH |
| GPT-5 | OpenAI system card, arxiv, leaks | 2025-08 → 2026-05 | MEDIUM (MoE/speculative inferred) |
| Gemini 2.5 Pro | Google DeepMind technical report, API docs | 2025-06 | HIGH |
| DeepSeek V4 | arXiv 2606.19348, HuggingFace docs | 2026-04 | HIGH |
| Llama 4 Behemoth | Meta AI blog, MODEL_CARD.md | 2025-04 | HIGH |
| Qwen 3.8 | Alibaba Cloud blog, arXiv 2608.30320 | 2026-08 | HIGH |
| Mistral Large 3 | Mistral AI docs, HuggingFace README | 2025-12 | HIGH |
| Grok 3 | xAI news, InferenceBench, reviews | 2025-02 | MEDIUM (architecture undisclosed) |
| Cohere Command R+ | Cohere docs, HuggingFace | 2024-08 | HIGH |
| Yi-Lightning | arXiv 2412.01253, InfoQ | 2024-12 | HIGH |
