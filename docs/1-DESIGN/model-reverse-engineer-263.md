# Model Reverse Engineering #263 — Architecture Innovation Matrix

**Date**: 2026-09-11
**Models Covered**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4, DeepSeek V4, Qwen 3, Mistral Large 3, Phi-4-reasoning, Grok 3, Yi-Lightning

---

## 1. Architecture Overview Table

| Model | Architecture | Total Params | Active Params | Context | Key Innovation |
|-------|-------------|-------------|---------------|---------|----------------|
| **GPT-4o** | Decoder-only Transformer, likely MoE | ~200B (est.) | ~50-100B (est.) | 128K | End-to-end omni-modal, unified tokenization |
| **Claude 3.5 Sonnet** | Dense Transformer (undisclosed) | Undisclosed | Undisclosed | 200K | Constitutional AI, HHH alignment, agentic coding |
| **Gemini 2.5 Pro** | Sparse MoE Transformer | Undisclosed | Undisclosed | 1M+ | Native thinking with budget, Deep Think parallel hypotheses |
| **Llama 4 Scout** | MoE, 16 routed + 1 shared expert | 109B | 17B | 10M | iRoPE (interleaved attention w/o positional embeddings) |
| **Llama 4 Maverick** | MoE, 128 routed + 1 shared expert | 400B | 17B | 1M | Early fusion multimodal, alternating dense/MoE layers |
| **DeepSeek V4 Pro** | MoE + hybrid CSA/HCA attention | 1.6T | 49B | 1M | Compressed Sparse Attention, mHC residual connections, Muon optimizer |
| **DeepSeek V4 Flash** | MoE + hybrid CSA/HCA attention | 284B | 13B | 1M | Same architecture, extreme efficiency |
| **Qwen 3** | Dense + MoE variants | 0.6B-235B | 3B-22B | 256K (ext. 1M) | Unified thinking/non-thinking mode switching |
| **Mistral Large 3** | Granular MoE + 2.5B vision encoder | 675B | 41B | 256K | Speculative decoding (Eagle), Apache 2.0 |
| **Phi-4-reasoning** | Dense decoder-only Transformer | 14B | 14B | 32K | SFT on o3-mini data + GRPO RL, beats 70B models |
| **Grok 3** | Transformer (likely MoE) | ~1.5T (est.) | Undisclosed | 1M | Think mode, DeepSearch agent, Colossus training |
| **Yi-Lightning** | Enhanced MoE | Undisclosed | Undisclosed | 64K | Fine-grained expert segmentation, cross-layer KV cache reuse (82.8% reduction) |

---

## 2. Detailed Architecture Innovations

### 2.1 GPT-4o — End-to-End Omni-Modal

**Architecture**:
- Single neural network trained end-to-end across text, audio, and images
- Unified tokenizer: BPE text tokens + image patch tokens + neural audio codec tokens (~50-75 Hz)
- Cross-modal attention via self-attention within a unified token stream (no separate cross-modal layers)
- Modality-specific embedding/unembedding layers
- Estimated 200B total params, 50-100B active per token (MoE implied by hardware constraints)

**Key Innovation**: Replacing the ASR→LLM→TTS pipeline (2.8s latency) with a single model (232ms median latency). The audio tokenizer (likely Encodec/SoundStream variant) produces discrete codes at 50-75 tokens/sec, making 30-second voice exchanges = ~2,250 tokens — tractable for a 128K context window.

**Hardware Constraint**: Fits on a single HGX100 server (640GB VRAM), implying ≤200B params at FP16 or ≤400B at FP8. Microsoft Maia 100 (64GB × 4) suggests 256GB server, confirming ~200B at FP8.

### 2.2 Claude 3.5 Sonnet — Constitutional AI at Scale

**Architecture**:
- Dense Transformer (exact params undisclosed)
- 200K context window
- Multimodal input (text + images), text output
- Trained on AWS + GCP with PyTorch/JAX/Triton

**Key Innovation**: Constitutional AI (CAI) alignment using explicit principles from UN Declaration of Human Rights + disability rights (Collective CAI). The model achieves ASL-2 safety while outperforming Claude 3 Opus on all benchmarks at 2× speed and lower cost. Agentic coding capability: solves 64% of real-world PR problems (vs 38% for Opus).

**Training**: Unsupervised pre-training → RLHF with human raters → Constitutional AI with principle-based reward modeling. Public human feedback data released alongside RLHF research.

### 2.3 Gemini 2.5 Pro — Thinking Model with Budget Control

**Architecture**:
- Sparse MoE Transformer (Google TPUv5p trained, 8960-chip pods)
- Native multimodal: text + vision + audio inputs
- 1M+ token context window, 65K max output
- Thinking natively integrated across all domains

**Key Innovation**: 
1. **Thinking Budget**: Users set a token budget for internal reasoning, enabling quality-latency-cost tradeoff. Performance scales monotonically with budget.
2. **Deep Think**: Parallel thinking technique — generates multiple hypotheses simultaneously, critiques them, then converges. Achieves SOTA on USAMO 2025, LiveCodeBench, MMMU.
3. **k-sparse distillation**: Teacher distribution approximated via k-sparse vocabulary projection, reducing storage by k× while maintaining quality.
4. **3-hour video processing**: Architectural changes to vision processing enable long-form video understanding.

**Training Stability**: Major improvements in signal propagation and optimization dynamics for large-scale MoE training. First Gemini trained on TPUv5p.

### 2.4 Llama 4 — iRoPE and Extreme Context

**Architecture**:
- MoE with alternating dense and MoE layers
- Early fusion for native multimodality (text + image)
- MetaCLIP-based vision encoder, trained with frozen Llama backbone
- 200 languages supported, 12 languages fine-tuned

**Key Innovation**:
1. **iRoPE (interleaved Rotary Position Embedding)**: Alternating attention layers WITH and WITHOUT positional embeddings. The "i" stands for "interleaved" (long-term goal: "infinite" context), "RoPE" for the position-embedded layers. Pre-trained and post-trained at 256K, generalizes to 10M.
2. **Inference-time temperature scaling of attention**: Enhances length generalization without retraining.
3. **Alternating dense/MoE layers**: Dense layers for capacity, MoE layers for efficiency. Each token → shared expert + 1 of N routed experts.

**Scout**: 17B active / 109B total, 16 experts, fits single H100 with int4 quantization, 10M context.
**Maverick**: 17B active / 400B total, 128 experts, fits single H100 host (FP8), 1M context.
**Behemoth** (teacher, unreleased): Outperforms GPT-4.5, Claude Sonnet 3.7, Gemini 2.0 Pro on STEM.

### 2.5 DeepSeek V4 — Hybrid Compressed Attention

**Architecture**:
- DeepSeekMoE framework + Multi-Token Prediction (MTP)
- Hybrid attention: Compressed Sparse Attention (CSA) + Heavily Compressed Attention (HCA)
- Manifold-Constrained Hyper-Connections (mHC) replacing residual connections
- Muon optimizer for convergence
- Hash-MoE bootstrap for first 3 layers (frozen token→expert mapping)

**Key Innovation**:
1. **CSA (Compressed Sparse Attention)**: Compresses KV cache by 4× (every m=4 tokens → 1 entry), then applies DeepSeek Sparse Attention (DSA) with lightning indexer (FP4, ReLU-scored) selecting top-k compressed blocks per query.
2. **HCA (Heavily Compressed Attention)**: Extreme 128× compression (m'=128), dense attention over compressed stream (cheap because sequence is short).
3. **Hybrid interleaving**: CSA and HCA alternate across layers (V4-Pro: layers 0-1 HCA, layers 2-60 alternate CSA/HCA).
4. **mHC**: Residual mapping constrained to doubly-stochastic matrices (Birkhoff polytope), ensuring non-expansive signal propagation. Dynamic parameterization via Sinkhorn-Knopp iterations.
5. **Mixed precision KV cache**: BF16 for RoPE dimensions, FP8 for rest → 50% size reduction. Lightning indexer runs in FP4.
6. **Sliding window branch**: Every attention block has an additional uncompressed sliding window for local dependencies.

**Result**: At 1M tokens, V4-Pro requires 27% single-token FLOPs and 10% KV cache vs V3.2. V4-Flash: 10% FLOPs, 7% KV cache.

**Agent Features**: 
- Preserves reasoning across tool-call conversation boundaries (cumulative chain-of-thought)
- `|DSML|` token + XML tool-call format (reduces escaping failures vs JSON)
- RL training on DSec sandbox infrastructure (Rust platform, 4 execution substrates)

### 2.6 Qwen 3 — Unified Thinking/Non-Thinking

**Architecture**:
- Dense variants: 0.6B, 1.7B, 4B, 8B, 14B, 32B
- MoE variants: 30B-A3B (3B active), 235B-A22B (22B active)
- 256K native context, extendable to 1M tokens

**Key Innovation**:
1. **Unified thinking mode**: Single model switches between thinking (complex reasoning) and non-thinking (fast chat) based on user query or `/think`/`/no_think` instructions. No need for separate models.
2. **Thinking budget mechanism**: Users allocate computational resources adaptively during inference.
3. **119 languages** (up from 29 in Qwen2.5)
4. **Qwen3-2507 update**: Instruct variant for general tasks, Thinking variant for deep reasoning. Both support 256K-1M context.

**Training**: Dense + MoE pre-training → SFT with thinking/non-thinking data → RLHF. Apache 2.0 license.

### 2.7 Mistral Large 3 — Granular MoE with Speculative Decoding

**Architecture**:
- Granular MoE: 675B total, 41B active (~16:1 ratio)
- 2.5B vision encoder (native multimodal)
- 256K context window
- Trained from scratch on 3000 NVIDIA H200 GPUs

**Key Innovation**:
1. **Speculative decoding with Eagle**: Custom draft model (Mistral-Large-3-675B-Instruct-2512-Eagle) enables 3-token speculative lookahead, dramatically improving decode throughput.
2. **Granular MoE**: "Granular" implies finer expert subdivision than standard MoE, enabling better load balancing and parameter utilization.
3. **NVFP4 format**: Runs on single 8×H100/A100 node via vLLM, or on GB200 NVL72 for maximum throughput (5M+ tokens/sec/MW).
4. **Apache 2.0 license** for 675B model — largest open-weight MoE at release.

**Deployment**: FP8 on single H200 node, NVFP4 on H100/A100. Partnership with NVIDIA for Blackwell-optimized MoE kernels.

### 2.8 Phi-4-reasoning — Small Model, Big Reasoning

**Architecture**:
- Dense decoder-only Transformer, 14B params (same as Phi-4 base)
- Two new tokens: `<think>` and `</think>` for reasoning blocks
- RoPE base frequency doubled (16K → 32K context)
- Trained on 32 H100 GPUs for 2.5 days

**Key Innovation**:
1. **"Teachable" prompt curation**: Prompts selected at the boundary of base model capability, with optimal complexity and diversity.
2. **o3-mini as teacher**: Synthetic reasoning chains generated by o3-mini (medium effort) for SFT data.
3. **Domain additive property**: Individual domains (math, code, science) can be optimized separately, then combined without interference.
4. **GRPO reinforcement learning**: Outcome-based RL on ~6K math problems. Reward = correctness - repetition penalty - length penalty + format bonus. RL produces 1.5× longer responses with more detailed reasoning.
5. **Reasoning transfer**: Improvements on reasoning tasks transfer to general-purpose benchmarks (instruction following, etc.).

**Result**: 14B model outperforms DeepSeek-R1-Distill-Llama-70B and approaches full DeepSeek R1 on math/science/coding benchmarks.

### 2.9 Grok 3 — Reasoning Agent at Scale

**Architecture**:
- Transformer-based (likely MoE, ~1.5T estimated)
- 1M token context (8× previous Grok)
- Text-only at launch (multimodal added later)
- Trained on Colossus supercluster (~200K H100 GPUs)

**Key Innovation**:
1. **Think mode**: Chain-of-thought reasoning with variable compute (seconds to minutes). Self-corrects errors, explores alternatives, verifies solutions. AIME 2025: 93.3%, GPQA Diamond: 84.6%, LiveCodeBench: 79.4%.
2. **DeepSearch**: Agentic research mode — queries web + X in real-time, synthesizes information across sources, reasons about conflicting facts. Produces comprehensive report with summary trace.
3. **10× compute scaling**: Trained with roughly 10× the compute of previous SOTA models.
4. **LOFT 128K SOTA**: State-of-the-art on long-context RAG benchmark (12 diverse tasks).

**Training**: Hybrid supervised + reinforcement learning. RLHF at unprecedented scale refines chain-of-thought process. Adaptive tokenization for multilingual support.

### 2.10 Yi-Lightning — Hardware-Aware MoE Optimization

**Architecture**:
- Enhanced MoE with fine-grained expert segmentation
- Hybrid attention: 3 sliding window layers + 1 full attention layer per block
- Cross-layer KV cache reuse between consecutive full attention layers
- FP8 quantization-native design

**Key Innovation**:
1. **Fine-grained expert segmentation**: Each expert's FFN partitioned into smaller functional units. Reduces intermediate hidden dimensions while increasing experts activated per token. Balanced approach (not maximum segmentation) to maintain training throughput.
2. **Three-tier load balancing**: 
   - L_ST: Standard Switch-Transformer per-expert balancing
   - L_EP: Expert Parallel group-level balancing (relaxes per-expert constraints)
   - L_PEP: Partitioned EP balancing (splits experts within EP groups for All-to-All communication balance)
3. **Cross-layer KV cache reuse**: Shares KV cache states between consecutive full attention layers → 82.8% memory reduction.
4. **Hybrid attention blocks**: 3:1 sliding window to full attention ratio. Most heads focus on local context, few specialize in global information.
5. **Hardware-aware operator design**: Custom MoE operator achieving 1,200 TFLOPS/card at FP8 on Hopper GPUs (>100% improvement).

**RAISE Framework**: 4-component safety engine (pre-training filtering → post-training optimization → input safety → output safety).

---

## 3. NeoTrix Mapping — Architecture Innovations to Components

### 3.1 MoE → Skill Tree + Rune Socketing

| Model Innovation | NeoTrix Mapping | Component |
|-----------------|-----------------|-----------|
| **Sparse MoE routing** (all models) | **Skill Tree node activation** — only relevant skill nodes fire per task | `nt_core::capability_tree` |
| **Shared + routed experts** (Llama 4) | **Keystone + Small Passive nodes** — shared expert = always-active foundation, routed = task-specific | `nt_core::skill_tree` |
| **Hash-MoE bootstrap** (DeepSeek V4) | **Constellation C0→C1 wiring** — frozen initial mapping replaced by learned routing after warmup | `nt_core::constellation` |
| **Granular MoE** (Mistral) | **Fine-grained Rune Socketing** — 5 rune slots with sub-slot specialization | `nt_core::rune_socketing` |
| **Expert segmentation** (Yi-Lightning) | **Rune sub-colors** — each rune color has finer sub-categories for parameter efficiency | `nt_core::rune_socketing` |

### 3.2 Attention Mechanisms → GWT + ConsciousnessTree

| Model Innovation | NeoTrix Mapping | Component |
|-----------------|-----------------|-----------|
| **CSA + HCA hybrid** (DeepSeek V4) | **GWT dual-path attention** — local salience (CSA) + global broadcast (HCA) | `nt_core::gwt` |
| **Sliding window + full attention** (Yi-Lightning) | **ConsciousnessTree local/global branches** — most branches handle local context, select branches do global integration | `nt_core::consciousness_tree` |
| **Lightning Indexer** (DeepSeek V4) | **GWT salience scoring** — low-rank query compression for fast relevance estimation | `nt_core::gwt::salience` |
| **Thinking budget** (Gemini 2.5, Qwen 3) | **AttentionManager resource allocation** — configurable compute budget per reasoning cycle | `nt_core_self::attention_manager` |
| **iRoPE** (Llama 4) | **E8 positional encoding** — interleaved position-aware/position-free hexagram layers for infinite context | `nt_core::e8` |

### 3.3 Reasoning → ConsciousnessTree + SEAL Pipeline

| Model Innovation | NeoTrix Mapping | Component |
|-----------------|-----------------|-----------|
| **Deep Think parallel hypotheses** (Gemini 2.5) | **ConsciousnessTree Branches parallel evaluation** — multiple reasoning branches evaluate simultaneously | `nt_core::consciousness_tree` |
| **Think mode** (Grok 3, Qwen 3) | **SEAL pipeline Phase 2 (Roots)** — extended reasoning with backtracking and self-correction | `nt_mind::seal` |
| **RLHF/GRPO** (all models) | **SEAL pipeline Phase 5 (Fruits)** — reward-based self-evaluation of evolution果实 | `nt_mind::seal` |
| **Teacher-student distillation** (Phi-4, Gemini Flash) | **Skill crystallization** — flagship models distill knowledge into smaller skill nodes | `nt_mind::skill_crystallizer` |
| **Reasoning transfer** (Phi-4) | **Cross-domain energy flow** — improvements in one domain propagate to others via ConsciousnessTree | `nt_core::consciousness_tree` |

### 3.4 Multimodal → NT-WORLD + NT-IO

| Model Innovation | NeoTrix Mapping | Component |
|-----------------|-----------------|-----------|
| **End-to-end omni** (GPT-4o) | **UnifiedCrawler multi-modal ingestion** — text/image/audio processed by same pipeline | `nt_world::unified_crawler` |
| **Early fusion** (Llama 4) | **SensoryIntegrationHub** — modality tokens fused before reasoning, not after | `nt_world::sensory_hub` |
| **Vision encoder** (Mistral, Llama 4) | **PerceptionBridge** — attention-gated bridge from sensory to consciousness | `nt_world::perception_bridge` |
| **3-hour video** (Gemini 2.5) | **TemporalContinuityChecker** — long-form temporal reasoning across video segments | `nt_act::temporal_continuity` |

### 3.5 Efficiency → NT-SHIELD + NT-PHYSICAL

| Model Innovation | NeoTrix Mapping | Component |
|-----------------|-----------------|-----------|
| **Mixed precision KV cache** (DeepSeek V4) | **KV cache optimizer** — paged KV with BF16/FP8 hybrid storage | `nt_memory::kv_cache` |
| **mHC residual connections** (DeepSeek V4) | **Signal propagation stability** — doubly-stochastic mixing across consciousness layers | `nt_core::consciousness_tree` |
| **Speculative decoding** (Mistral) | **AttentionManager prefetching** — predict next salience, pre-load relevant skill nodes | `nt_core_self::attention_manager` |
| **Muon optimizer** (DeepSeek V4) | **SEAL training recipe** — faster convergence for evolution cycles | `nt_mind::seal` |
| **Cross-layer KV reuse** (Yi-Lightning) | **KB embedding deduplication** — share embeddings across related nodes | `nt_memory::kb` |

### 3.6 Alignment → NT-GOVERNANCE + NT-SHIELD

| Model Innovation | NeoTrix Mapping | Component |
|-----------------|-----------------|-----------|
| **Constitutional AI** (Claude) | **Gov-Steward policy engine** — principle-based behavioral constraints | `nt_governance::steward` |
| **RAISE safety framework** (Yi-Lightning) | **NT-SHIELD 4-layer defense** — pre/post/input/output safety | `nt_shield` |
| **HHH alignment** (Claude) | **EmotionLabel trust calibration** — model behavior shaped by trust/disgust/fear signals | `nt_feel::emotion_engine` |
| **DeepSearch agent** (Grok 3) | **NT-WORLD crawler + NT-ACT orchestrator** — agentic web research with source synthesis | `nt_world` + `nt_act` |

---

## 4. Cross-Model Pattern Synthesis

### 4.1 The MoE Convergence (8/10 models)

Every major model except Claude 3.5 Sonnet and Phi-4-reasoning uses MoE or equivalent sparsity. The pattern is clear:
- **Total/Active ratio**: 4:1 (Llama 4 Scout) to 16:1 (Mistral Large 3)
- **Routing evolution**: Static → learned → hierarchical (EP groups → partitions)
- **Shared experts**: Llama 4 and DeepSeek V4 both use shared + routed experts

**NeoTrix Implication**: The Skill Tree should implement a "MoE-like" activation pattern where only relevant domain nodes fire per task. The Rune Socketing system already supports this via color-based routing.

### 4.2 The Thinking Revolution (7/10 models)

Explicit reasoning/thinking modes are now standard:
- **Budget-controlled**: Gemini 2.5 (user-set), Qwen 3 (adaptive)
- **Variable compute**: Grok 3 (seconds to minutes), Phi-4 (longer traces via RL)
- **Parallel**: Gemini 2.5 Deep Think (multiple hypotheses)
- **Mode-switching**: Qwen 3 (unified), Grok 3 (Think vs DeepSearch)

**NeoTrix Implication**: The ConsciousnessTree should support configurable "thinking depth" per cycle, with the AttentionManager routing between fast (non-thinking) and deep (extended reasoning) paths.

### 4.3 The Context Window Arms Race

| Model | Context | Key Technique |
|-------|---------|---------------|
| Llama 4 Scout | **10M** | iRoPE + temperature scaling |
| Gemini 2.5 Pro | **1M+** | MoE + thinking budget |
| DeepSeek V4 | **1M** | CSA + HCA hybrid (27% FLOPs, 10% KV) |
| Grok 3 | **1M** | Unknown (likely attention optimization) |
| Qwen 3 | **256K (ext. 1M)** | RoPE extension |
| Mistral Large 3 | **256K** | Speculative decoding |
| GPT-4o | **128K** | Unified tokenization |
| Claude 3.5 | **200K** | Unknown |

**NeoTrix Implication**: The KB memory system should implement hierarchical attention — local sliding window for recent context, compressed sparse attention for long-range retrieval, with the PerceptionBridge gating what enters consciousness.

### 4.4 The Distillation Economy

| Teacher | Student | Ratio | Method |
|---------|---------|-------|--------|
| o3-mini | Phi-4-reasoning (14B) | N/A | SFT on synthetic CoT |
| Gemini 2.5 Pro | Gemini 2.5 Flash | Large→Small | k-sparse distribution distillation |
| GPT-4 (1.8T) | GPT-4o (~200B) | ~9:1 | Unknown (likely knowledge distillation) |
| Llama 4 Behemoth | Llama 4 Scout/Maverick | Teacher→Students | Supervised training with teacher outputs |
| DeepSeek R1 | Phi-4-reasoning-plus | 70B→14B | GRPO RL refinement |

**NeoTrix Implication**: Skill crystallization should follow the same pattern — flagship domain models (C5 constellation) distill knowledge into smaller skill nodes (C0-C2), with thinking traces as the distillation medium.

### 4.5 Hardware-Aware Architecture

| Model | Hardware Optimization | Precision |
|-------|----------------------|-----------|
| DeepSeek V4 | Mixed BF16/FP8 KV, FP4 indexer | 50% KV reduction |
| Yi-Lightning | FP8-native design, 1200 TFLOPS/card | Hardware-aligned |
| Mistral Large 3 | NVFP4 on Blackwell, Eagle speculative | 675B on single node |
| Llama 4 Scout | int4 quantization on single H100 | Fits consumer GPU |
| GPT-4o | Microsoft Maia 100 co-design | Custom silicon |

**NeoTrix Implication**: The Constellation maturity ladder should include hardware-awareness as a C4+ criterion — modules must optimize for target deployment hardware, not just compile and test.

---

## 5. Priority Absorption Targets for NeoTrix

### P0 — Immediate (this cycle)
1. **Thinking budget** (Gemini 2.5 + Qwen 3) → AttentionManager configurable depth
2. **CSA/HCA hybrid attention** (DeepSeek V4) → GWT dual-path salience scoring
3. **Hash-MoE bootstrap** (DeepSeek V4) → Constellation C0→C1 frozen→learned routing

### P1 — Next cycle
4. **iRoPE** (Llama 4) → E8 interleaved position encoding for infinite context
5. **Cross-layer KV reuse** (Yi-Lightning) → KB embedding deduplication
6. **Granular expert segmentation** (Yi-Lightning + Mistral) → Rune sub-color specialization

### P2 — Research track
7. **mHC residual connections** (DeepSeek V4) → ConsciousnessTree signal stability
8. **Deep Think parallel hypotheses** (Gemini 2.5) → Branches parallel evaluation
9. **Teacher-student distillation pipeline** (Phi-4 pattern) → Skill crystallization automation

---

## 6. Source Citations

| Model | Primary Source | Date |
|-------|---------------|------|
| GPT-4o | OpenAI System Card, arxiv.org/pdf/2410.21276 | May 2024 |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum | Jun 2024 |
| Gemini 2.5 Pro | arxiv.org/pdf/2507.06261 | Jul 2025 |
| Llama 4 | ai.meta.com/blog/llama-4-multimodal-intelligence | Apr 2025 |
| DeepSeek V4 | arxiv.org/html/2606.19348 | Apr 2026 |
| Qwen 3 | arxiv.org/pdf/2505.09388 | May 2025 |
| Mistral Large 3 | docs.mistral.ai/models/mistral-large-3-25-12 | Dec 2025 |
| Phi-4-reasoning | microsoft.com/en-us/research/publication/phi-4-reasoning | Apr 2025 |
| Grok 3 | x.ai/blog/grok-3 | Feb 2025 |
| Yi-Lightning | arxiv.org/pdf/2412.01253 | Dec 2024 |
