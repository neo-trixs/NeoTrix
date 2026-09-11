# Model Reverse Engineering #310 — 10 Frontier Architectures

**Date**: 2026-09-11
**Scope**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3

---

## Summary Matrix

| Model | Total Params | Active Params | Architecture | Experts | Context | Key Innovation |
|-------|-------------|---------------|--------------|---------|---------|----------------|
| **GPT-4o** | ~200B (est.) | ~50-100B (est.) | Omni MoE | 16 (est.) | 128K | End-to-end multimodal training |
| **Claude 3.5 Sonnet** | Undisclosed | Undisclosed | Dense Transformer | N/A | 200K | Constitutional AI + Computer Use |
| **Gemini 2.5 Pro** | Undisclosed | Undisclosed | Sparse MoE | N/A | 1M | Thinking budget + k-sparse distillation |
| **Llama 4 Scout** | 109B | 17B | MoE (Early Fusion) | 16 routed + 1 shared | 10M | iRoPE + interleaved attention |
| **DeepSeek V4.1 Flash** | 552B | 8B (prefill) / 16B (decode) | Causal Encoder-Decoder | 384 routed + 1 shared | 1M | CSA2 + CED asymmetric activation |
| **Qwen 3-235B-A22B** | 235B | 22B | Dense/MoE | 128 total / 8 active | 128K | Unified thinking/non-thinking mode |
| **Mistral Large 3** | 675B | 41B | Granular MoE | Undisclosed | 256K | 6% activation ratio, NVFP4 deploy |
| **Phi-4 Reasoning** | 14B | 14B (dense) | Dense Transformer | N/A | 32K | Data-centric SFT + GRPO RL |
| **Yi-Lightning** | Undisclosed | Undisclosed | Enhanced MoE | Undisclosed | 128K+ | EP/PEP load balancing + cross-layer KV reuse |
| **Grok 3** | ~1.5-2.7T (est.) | Undisclosed | MoE + Think Mode | 128 (est.) | 1M | DeepSearch agent + RL reasoning |

---

## 1. GPT-4o (OpenAI)

**Source**: GPT-4o System Card (May 2024), OpenAI API docs

### Architecture
- **Type**: End-to-end autoregressive omni model
- **Total Params**: ~200B (industry estimate; not disclosed)
- **Active Params**: ~50-100B (constrained by single-server H100/640GB VRAM)
- **Experts**: ~16, top-2 routing (unofficial estimate)
- **Context**: 128K tokens, 16K max output
- **Modality**: Unified text/audio/image/video → text/audio/image

### Key Innovations
1. **Unified Multimodal Training**: Single neural network trained end-to-end across text, vision, and audio — not the staged CLIP+Whisper+TTS pipeline used in GPT-4
2. **232ms Audio Latency**: Median response latency comparable to human conversation speed (vs. 2.8s in previous staged pipeline)
3. **Unified Token Stream**: Text BPE tokens + image patch tokens + audio codec tokens all processed through the same transformer stack
4. **Modality-Specific Embedding/Unembedding**: Separate input tables per modality, shared attention core

### NeoTrix Mapping
- **GWT**: Cross-modal attention routing — modality tokens compete for salience in unified stream
- **NT-WORLD**: UnifiedCrawler 的多模态感知同构
- **NT-FEEL**: 情感表达 (text→audio) 需要跨模态一致性
- **Skill**: End-to-end multimodal training 替代 staged pipeline → 对应 NeoTrix 的 **CapabilityBridge** (演进视图 ↔ 运行视图)

---

## 2. Claude 3.5 Sonnet (Anthropic)

**Source**: Model Card Addendum (June 2024), Claude 3 Model Family Card

### Architecture
- **Type**: Dense decoder-only transformer (estimated)
- **Total Params**: Undisclosed
- **Active Params**: Same as total (dense)
- **Context**: 200K tokens
- **Modality**: Text + image input, text output

### Key Innovations
1. **Constitutional AI (CAI)**: Rule-based self-critique and revision training — the model aligns itself against a set of constitutional principles
2. **Computer Use**: Screenshot→GUI action generation (OSWorld 14.9% → 22% with more steps)
3. **Agentic Coding**: 64% → 78% on internal benchmark (iterative multi-file code editing in sandbox)
4. **ASL-2 Safety Tier**: Structured Responsible Scaling Policy with capability thresholds

### NeoTrix Mapping
- **NT-SHIELD**: Constitutional AI → **Gov-Steward** 治理合规框架
- **NT-ACT**: Computer Use → **Dev-匠** 工具调用 + **ProductionOrchestrator** 多步骤执行
- **NT-CORE**: Structured self-critique → **ConsciousnessTree** 的 6 阶段反馈循环
- **Skill**: SafeDeleter 的 archive-before-delete 同构 CAI 的 "safety first" 原则

---

## 3. Gemini 2.5 Pro (Google DeepMind)

**Source**: Gemini 2.5 Technical Report (Jul 2025), Model Card

### Architecture
- **Type**: Sparse MoE transformer with native multimodal support
- **Total Params**: Undisclosed
- **Active Params**: Undisclosed
- **Experts**: N/A (undisclosed)
- **Context**: 1M tokens (2M coming)
- **Modality**: Text + vision + audio input, text output

### Key Innovations
1. **Thinking Budget**: User-configurable compute allocation — model decides how long to reason, or constrained by budget
2. **k-Sparse Distillation**: Teacher's next-token distribution approximated by top-k sparse vectors, reducing storage while maintaining quality
3. **Training Stability Breakthrough**: Solved MoE training instabilities through improved signal propagation and optimization dynamics
4. **TPUv5p Multi-Datacenter Training**: First model trained on TPUv5p across multiple Google datacenters
5. **3-Hour Video Processing**: Native long-form video understanding (not frame sampling)

### NeoTrix Mapping
- **GWT**: Thinking budget → **AttentionManager** 的注意力分配 + **A1 Cost-Aware Routing**
- **NT-MIND**: k-sparse distillation → SEAL pipeline 的 **蒸馏阶段** 同构
- **NT-REPAIR**: Training stability breakthrough → **HeartbeatAggregator** 系统健康信号
- **Skill**: Thinking budget → **Rune Socketing** 的 Obsidian(缓存) 层 — 按需分配推理预算

---

## 4. Llama 4 Scout (Meta)

**Source**: Llama 4 Model Card (Apr 2025), Meta AI blog

### Architecture
- **Type**: Autoregressive MoE with early fusion for native multimodality
- **Total Params**: 109B (Scout), 400B (Maverick)
- **Active Params**: 17B
- **Experts**: 16 routed + 1 shared (Scout), 128 routed + 1 shared (Maverick)
- **Context**: 10M tokens (Scout), 1M (Maverick)
- **Modality**: Multilingual text + image input, text + code output

### Key Innovations
1. **iRoPE Architecture**: Interleaved attention layers **without** positional embeddings + inference-time temperature scaling of attention → "infinite" context length goal
2. **Alternating Dense/MoE Layers**: MoE and MLP layers alternate (Scout: 1:1, Maverick: 2:1 MLP:MoE)
3. **Shared Expert + Routed Expert**: Each token goes to shared expert + 1 routed expert (not top-k among all)
4. **MetaP Hyperparameter Transfer**: Automatically sets per-layer learning rates and initialization across scales
5. **Single H100 Deployment**: Scout fits on single H100 with int4 quantization

### NeoTrix Mapping
- **NT-MEMORY**: 10M context → **KVMem** 的 paged KV virtualization (A2 Context as Scarce Resource)
- **NT-CORE**: iRoPE → **E8 Hexagram** 的无位置编码层 — 适合无限上下文
- **NT-ACT**: Shared + routed expert → **CapabilityBridge** 的共享能力 + 专用能力路由
- **Skill**: MetaP → **NT-MIND** 的 SEAL pipeline 自动超参调优

---

## 5. DeepSeek V4.1 Flash (DeepSeek)

**Source**: DeepSeek V4.1 Flash Technical Report (Sep 2026), HuggingFace model card

### Architecture
- **Type**: Causal Encoder-Decoder (CED) MoE
- **Total Params**: 552B backbone
- **Active Params**: 8B (prefill), 16B (decode) — asymmetric
- **Experts**: 384 routed + 1 shared per MoE layer, 6 activated per token
- **Context**: 1M tokens
- **Modality**: Native vision (DeepSeek-ViT) + text

### Key Innovations
1. **Causal Encoder-Decoder (CED)**: 20-layer encoder + 20-layer decoder; decoder's KV cache projected from encoder's final hidden states → 8B active for input, 16B for output
2. **Compressed Sparse Attention 2 (CSA2)**: 3 static modes (Full/Reindex/Reuse) per layer, sharing KV and indexer K across layers + FP4 KV caching (890 bytes/token, 1/4 of V4-Flash)
3. **SWA Bounded Replay**: Reconstructs sliding-window attention KV by replaying only last n_win tokens → 1/8 persistent KV footprint
4. **Engram Conditional Memory**: 196B parameters, sparsely accessed via 4-gram hash lookup into residual stream
5. **DSpark Speculative Decoding**: Semi-autoregressive draft generation with confidence-scheduled verification
6. **Manifold-Constrained Hyper-Connections (mHC)**: Constrain residual mapping onto manifold for stable signal propagation
7. **Hash Routing in Initial Layers**: First few blocks use Hash routing instead of learned routing

### NeoTrix Mapping
- **NT-MEMORY**: CSA2 + FP4 KV → **KVMem** paged KV 的极致压缩; Engram → **KB** 向量存储的 n-gram 变体
- **NT-CORE**: mHC → **E8 Hexagram** 残差流的稳定性保证
- **GWT**: Asymmetric activation (8B/16B) → **A1 Cost-Aware Routing** — 输入/输出用不同计算预算
- **NT-ACT**: DSpark → **ParallelTaskManager** 投机解码的多阶段验证
- **Skill**: Hash routing → **Rune Socketing** 的 Crimson(数据) 层 — 确定性路由

---

## 6. Qwen 3 (Alibaba Cloud)

**Source**: Qwen3 Technical Report (May 2025), arXiv:2505.09388

### Architecture
- **Type**: Dense + MoE variants
- **Flagship**: 235B total, 22B active (Qwen3-235B-A22B)
- **Experts**: 128 total, 8 activated per token (no shared experts)
- **Context**: 128K tokens
- **Languages**: 119 languages (up from 29 in Qwen2.5)

### Key Innovations
1. **Unified Thinking/Non-Thinking Mode**: Single model switches between step-by-step reasoning and fast context-driven responses — eliminates separate chat vs. reasoning models
2. **Thinking Budget Mechanism**: Users allocate compute adaptively during inference
3. **No Shared Experts**: Unlike DeepSeekMoE, Qwen3-MoE excludes shared experts entirely
4. **Global-Batch Load Balancing Loss**: Encourages expert specialization across the full batch (not per-expert)
5. **Strong-to-Weak Distillation**: 5 dense + 1 MoE models distilled from 235B flagship
6. **36T Token Pre-training**: 3-stage curriculum (30T → 5T knowledge-intensive → long-context extension)

### NeoTrix Mapping
- **GWT**: Unified thinking/non-thinking → **AttentionManager** 双专精 (Dual Specialization) — 按任务类型路由
- **NT-MIND**: Strong-to-Weak Distillation → SEAL pipeline 的 **蒸馏阶段** + **Skill Crystallization**
- **NT-CORE**: Thinking budget → **A1 Cost-Aware Routing** + **A2 Context as Scarce Resource**
- **Skill**: No shared experts → **Constellation** 的独立星辰模型 (每颗星独立，不共享核心)

---

## 7. Mistral Large 3 (Mistral AI)

**Source**: Mistral 3 announcement (Dec 2025), HuggingFace, NVIDIA blog

### Architecture
- **Type**: Granular sparse MoE
- **Total Params**: 675B
- **Active Params**: 41B (~6% activation ratio)
- **Experts**: Granular (many small experts), exact count undisclosed
- **Context**: 256K tokens
- **Modality**: Text + image input, text output (native vision encoder ~2.5B)

### Key Innovations
1. **Extreme Sparsity Ratio**: 675B/41B = 16:1 ratio — only ~6% of network fires per token
2. **Granular MoE**: Many small experts instead of few large ones → finer-grained routing, better specialization
3. **Apache 2.0 + Frontier Scale**: First 675B model fully open-sourced under permissive license
4. **NVFP4 Deployment**: Full model fits on single 8×H100 node via NVFP4 quantization
5. **NVIDIA Co-design**: Blackwell attention kernels + MoE kernels + speculative decoding for 256K context

### NeoTrix Mapping
- **NT-ACT**: Granular MoE → **CapabilityRegistry** 的细粒度能力路由
- **NT-SHIELD**: Apache 2.0 + open weights → **R-P1 #![forbid(unsafe_code)]** 开放安全
- **NT-MEMORY**: 6% activation → **HeartbeatAggregator** 的按需资源分配
- **Skill**: NVFP4 deployment → **Rune Socketing** 的 Obsidian(缓存) — 量化压缩

---

## 8. Phi-4 Reasoning (Microsoft Research)

**Source**: Phi-4-reasoning Technical Report (Apr 2025), arXiv:2504.21318

### Architecture
- **Type**: Dense decoder-only transformer
- **Total Params**: 14B
- **Active Params**: 14B (dense)
- **Context**: 32K tokens (extended from 16K via RoPE frequency doubling)
- **Modality**: Text only

### Key Innovations
1. **Data-Centric Reasoning**: 1.4M curated "teachable" prompts at the edge of base model capability → o3-mini generates reasoning traces
2. **Thinking Tokens**: `<think>` / `</think>` placeholder tokens repurposed from base model vocabulary
3. **SFT + GRPO RL Pipeline**: Supervised fine-tuning on reasoning traces → short reinforcement learning on math problems with verifiable rewards
4. **Reasoning as Transferable Meta-Skill**: Improvements transfer to domains NOT targeted in training (planning, spatial, algorithmic)
5. **14B Outperforms 70B**: Beats DeepSeek-R1-Distill-Llama-70B; approaches full DeepSeek-R1

### NeoTrix Mapping
- **NT-MIND**: Data curation → **Skill Crystallization** 的高质量经验蒸馏
- **NT-CORE**: Thinking tokens → **ConsciousnessTree** 的显式推理轨迹
- **NT-MEMORY**: "Teachable" prompt selection → **experience-tree** 的快照→蒸馏→分类
- **Skill**: SFT+RL pipeline → **SEAL pipeline** 的 Phase-3(RL) + Phase-4(吸收)
- **Axiom A1**: 14B beating 70B → **Cost-Aware Routing** — 小模型+高效训练 > 大模型暴力训练

---

## 9. Yi-Lightning (01.AI)

**Source**: Yi-Lightning Technical Report (Dec 2024), arXiv:2412.01253

### Architecture
- **Type**: Enhanced MoE transformer
- **Total Params**: Undisclosed
- **Active Params**: Undisclosed
- **Context**: 128K+ tokens
- **Modality**: Text

### Key Innovations
1. **Fine-Grained Expert Segmentation**: FFN partitioned into smaller units → more experts activated per token, finer knowledge decomposition
2. **EP/PEP Load Balancing**: Expert Parallel load balancing + Partitioned EP (sub-partitions within groups) for All-to-All communication balance
3. **Cross-Layer KV Cache Reuse**: Share KV states between consecutive full-attention layers → 50% memory reduction for full attention
4. **Hybrid Attention Blocks**: 3 sliding window + 1 full attention layer → 82.8% memory reduction
5. **RAISE Safety Engine**: 4-component framework (pre-training, post-training, serving safety)
6. **Hardware-Aware FP8**: Architecture designed for FP8 quantization compatibility; MoE operator at 1,200 TFLOPS/card on Hopper

### NeoTrix Mapping
- **NT-MEMORY**: Cross-layer KV reuse → **KVMem** 的 paged KV virtualization (A2)
- **GWT**: Hybrid attention blocks → **PerceptionBridge** 的注意力门控
- **NT-SHIELD**: RAISE → **NT-SHIELD** stealth net 的多层安全框架
- **NT-ACT**: EP/PEP load balancing → **ParallelTaskManager** 的多设备负载均衡
- **Skill**: Hardware-aware design → **Rune Socketing** 的 Golden(错误恢复) — 硬件适配

---

## 10. Grok 3 (xAI)

**Source**: xAI announcement (Feb 2025), Comprehensive Analysis report

### Architecture
- **Type**: MoE transformer + reasoning modes
- **Total Params**: ~1.5-2.7T (estimated; 128 expert networks)
- **Active Params**: Undisclosed
- **Context**: 1M tokens
- **Modality**: Text + image input, text output

### Key Innovations
1. **Think Mode**: Chain-of-thought reasoning with self-correction, backtracking, and multi-approach exploration (seconds to minutes)
2. **DeepSearch Agent**: Web-scraping + X-platform data + tool-use stitched into single pipeline; real-time citations in <2s
3. **Colossus Supercluster**: ~200K H100 GPUs, ~200M GPU-hours training (10x previous SOTA)
4. **Cross-Expert Attention Gates**: Knowledge sharing between specialist MoE components without catastrophic interference
5. **Dynamic Compute Allocation ("Big Brain Mode")**: Ups GPU quota for harder tasks, lowering latency on 100+ step reasoning chains

### NeoTrix Mapping
- **GWT**: Think Mode → **ConsciousnessTree** 6-stage feedback loop 的显式推理
- **NT-ACT**: DeepSearch → **NT-WORLD** UnifiedCrawler + **NT-ACT** tool orchestration
- **NT-CORE**: Cross-expert attention gates → **E8 Hexagram** 的跨卦象信息共享
- **NT-REPAIR**: Self-correction/backtracking → **MAPE-K** 自愈循环
- **Skill**: Dynamic compute allocation → **A1 Cost-Aware Routing** — 按任务难度动态分配计算资源

---

## Cross-Model Innovation Matrix

### Pattern 1: MoE Is Universal (9/10 models)
All models except Claude 3.5 Sonnet use MoE or MoE-adjacent architectures. The trend is toward:
- **More experts, smaller per-expert** (Mistral's "granular MoE")
- **Shared + routed experts** (Llama 4, DeepSeek)
- **Asymmetric activation** (DeepSeek V4.1: 8B input / 16B output)

**NeoTrix Absorption**: CapabilityRegistry 的细粒度路由 + AttentionManager 的成本感知分配

### Pattern 2: KV Cache Compression Arms Race
- DeepSeek V4.1: CSA2 + FP4 = 890 bytes/token (1/4 of V4-Flash)
- Yi-Lightning: Cross-layer KV reuse = 50% reduction
- Llama 4 Scout: iRoPE = position-free infinite context

**NeoTrix Absorption**: KVMem paged KV virtualization (A2 Context as Scarce Resource)

### Pattern 3: Thinking/Reasoning as First-Class Feature
- Qwen 3: Unified thinking/non-thinking mode
- Phi-4: `<think>` tokens + thinking budget
- Grok 3: Think mode (seconds to minutes)
- Gemini 2.5: User-configurable thinking budget

**NeoTrix Absorption**: ConsciousnessTree 6-stage loop + AttentionManager dual specialization

### Pattern 4: Data Curation > Model Scale
- Phi-4 Reasoning: 14B beats 70B via curated data
- Qwen 3: 36T tokens with 3-stage curriculum
- Grok 3: Synthetic data + RL for reasoning traces

**NeoTrix Absorption**: experience-tree 五阶段吸收 (快照→蒸馏→分类→落盘→反馈)

### Pattern 5: Asymmetric Compute
- DeepSeek V4.1: Different compute for input vs. output
- Grok 3: Dynamic compute allocation per task difficulty
- Qwen 3: Thinking budget mechanism

**NeoTrix Absorption**: A1 Cost-Aware Routing + Rune Socketing 5色槽位

---

## Priority Absorption Targets for NeoTrix

| Priority | Innovation | Source | NT Component | Status |
|----------|-----------|--------|--------------|--------|
| P0 | CED asymmetric activation | DeepSeek V4.1 | NT-CORE attention | Research |
| P0 | Unified thinking/non-thinking | Qwen 3 | GWT routing | Research |
| P0 | Cross-layer KV reuse | Yi-Lightning | KVMem | Research |
| P1 | Granular MoE routing | Mistral Large 3 | CapabilityRegistry | Design |
| P1 | Thinking budget mechanism | Gemini 2.5 / Qwen 3 | AttentionManager | Design |
| P1 | CSA2 + FP4 KV compression | DeepSeek V4.1 | KVMem | Research |
| P2 | iRoPE infinite context | Llama 4 Scout | E8 Hexagram | Research |
| P2 | Data curation pipeline | Phi-4 Reasoning | experience-tree | Existing |
| P2 | Cross-expert attention gates | Grok 3 | E8 Hexagram | Research |
| P3 | RAISE safety framework | Yi-Lightning | NT-SHIELD | Review |
| P3 | k-sparse distillation | Gemini 2.5 | SEAL pipeline | Review |

---

*Generated by NeoTrix model-reverse-engineer pipeline. Sources: arXiv, HuggingFace, official model cards, technical reports. All parameter counts marked "est." are unofficial industry estimates.*
