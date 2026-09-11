# Model Architecture Reverse Engineering — 10 Models

> **Date**: 2026-09-11
> **Purpose**: Extract architectural innovations from 10 frontier LLMs and map them to NeoTrix subsystems
> **Sources**: Technical reports, system cards, arXiv papers, official blogs

---

## Table of Contents

1. [GPT-4o](#1-gpt-4o)
2. [Claude 3.5 Sonnet](#2-claude-35-sonnet)
3. [Gemini 2.5 Pro](#3-gemini-25-pro)
4. [Llama 4 Scout](#4-llama-4-scout)
5. [DeepSeek V4 Flash](#5-deepseek-v4-flash)
6. [Qwen 3](#6-qwen-3)
7. [Mistral Large 3](#7-mistral-large-3)
8. [Phi-4 Reasoning](#8-phi-4-reasoning)
9. [Yi-Lightning](#9-yi-lightning)
10. [Grok 3](#10-grok-3)

---

## 1. GPT-4o

**Source**: OpenAI System Card (arXiv:2410.21276), Hello GPT-4o blog, GPT-ImgEval benchmark

### Architecture Summary

- **Type**: End-to-end omnimodal autoregressive transformer
- **Parameters**: Undisclosed (speculated < GPT-4 Turbo via MoE or efficient arch)
- **Context**: 128K tokens
- **Modalities**: Text + Image + Audio (native, unified network)
- **Training**: Joint end-to-end across text, vision, and audio — NOT staged CLIP pipeline

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Unified Multimodal Tokenization** | Single transformer stack consumes text tokens (BPE), image patch tokens (ViT-style), and audio tokens (neural codec like Encodec/SoundStream at 50-75 Hz). Cross-modal attention happens via self-attention within the unified stream. |
| **End-to-End Multimodal Training** | Collapsed 3-stage pipeline (ASR→LLM→TTS) into single network. Median voice latency dropped from 2.8s to 232ms (12x reduction). |
| **Modality-Specific Embedding/Unembedding** | Input embedding tables are modality-specific (text, image patch, audio codec codebook); output heads produce the right modality's tokens depending on context. |
| **Autoregressive Image Generation** | Uses AR + diffusion-based head for image decoding (not VAR-like). Continuous visual tokenizer, not discrete VQ. |
| **Inference Cost Efficiency** | Cheaper per-token than GPT-4 Turbo — likely MoE + efficient kernels + speculative decoding. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Unified multimodal tokenization | **NT-WORLD** (SensoryIntegrationHub) | PerceptionBridge attention-gated fusion; unify text/image/audio into HyperCube VSA vectors |
| End-to-end pipeline collapse | **NT-ACT** (orchestration) | SEAL pipeline stage fusion; eliminate inter-stage latency in capability network |
| Autoregressive + diffusion hybrid | **NT-MIND** (evolution) | Hybrid reasoning: AR for sequential logic + diffusion for generative exploration |
| Cost-aware modality routing | **NT-CORE** (GWT salience) | A1 axiom: route cheap modalities to lightweight experts, expensive to heavy thinkers |

---

## 2. Claude 3.5 Sonnet

**Source**: Anthropic Model Card Addendum, Claude 3 Model Family PDF

### Architecture Summary

- **Type**: Dense transformer (likely >200B, undisclosed)
- **Parameters**: Undisclosed (Sonnet tier = mid-tier)
- **Context**: 200K tokens
- **Modalities**: Text + Image input, Text output
- **Training**: Unsupervised learning + Constitutional AI (RLHF)

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Constitutional AI (CAI)** | Training with explicit principles rather than just human labels. Self-critique and revision during training. |
| **Computer Use (GUI Agent)** | Native ability to interpret screenshots and generate mouse/keyboard actions. OSWorld SOTA at 22% with 50 steps. |
| **Agentic Coding Loop** | 64% → 78% on internal agentic coding eval. Model writes, runs, and iteratively self-corrects code in sandboxed environments. |
| **Artifacts Workspace** | Dynamic content generation alongside conversation — real-time editing and building on AI outputs. |
| **Responsible Scaling Policy (RSP)** | ASL-2 safety levels with quantitative "thresholds of concern" for CBRN, cyber, and autonomy risks. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Constitutional AI | **NT-GOVERNANCE** (Gov-衡) | Policy-driven alignment; constitution as governance primitives in KB |
| Computer Use / GUI Agent | **NT-ACT** (Dev-匠) + **NT-IO** | Tool-use scaffolding; screen→action loop as MCP tool chain |
| Agentic coding loop | **NT-ACT** (Dev-匠) | SEAL pipeline: write→test→reflect→rewrite as iterative evolution |
| ASL safety levels | **NT-SHIELD** (Rev-明) | RiskAssessor graduated safety tiers; threshold-gated deployment |
| Artifacts workspace | **NT-IO** (Edu-灯) | Live workspace for skill execution; incremental artifact building |

---

## 3. Gemini 2.5 Pro

**Source**: Gemini 2.5 Technical Report (arXiv:2507.06261), Google DeepMind blog

### Architecture Summary

- **Type**: Sparse Mixture-of-Experts (MoE) transformer
- **Parameters**: Undisclosed (Flash = smaller, Pro = larger)
- **Context**: 1M tokens (2M planned)
- **Modalities**: Native text + vision + audio (multimodal from pretraining)
- **Training**: TPUv5p across multiple datacenters, 8960-chip pods

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Native Multimodal MoE** | Sparse MoE activates subset of parameters per token. Decouples total model capacity from serving cost per token. Native text+vision+audio from pretraining. |
| **Thinking Model (Test-Time Compute)** | Model decides how long to think before answering. Thinking budget mechanism lets users trade performance vs cost. Budget scales accuracy. |
| **Deep Think (Parallel Hypothesis)** | Generates multiple hypotheses in parallel, critiques them, then arrives at final answer. Blends parallel thinking techniques during generation. |
| **1M+ Token Context** | Processes 3-hour video, entire codebases. Architectural changes to vision processing for long-form video understanding. |
| **k-Sparse Distillation** | Smaller models use distillation with k-sparse approximation of teacher's next-token distribution. Reduces storage by factor of k while maintaining quality. |
| **Training Infrastructure** | Slice-granularity elasticity (auto-continue with fewer chips on failure, 97% throughput during recovery). Split-phase SDC detection via lightweight deterministic replay. |
| **Agentic Workflows** | Deep Research agent with task prioritization, dead-end detection. HLE benchmark: 7.95% → 32.4% with higher compute. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Sparse MoE routing | **NT-CORE** (GWT) | Expert activation as salience-gated attention routing; only fire relevant specialists |
| Thinking budget | **NT-MIND** (SEAL) | Adaptive compute allocation per task; budget = SEAL cycle depth |
| Parallel hypothesis (Deep Think) | **NT-CORE** (E8 + HyperCube) | Parallel hypothesis generation = multiple hexagram states; critique = resonance collapse |
| k-Sparse distillation | **NT-MIND** | Strong-to-weak knowledge transfer; distillation as skill crystallization |
| Training elasticity | **NT-REPAIR** (Repair-医) | Self-healing: auto-recover from chip failures with graceful degradation |
| Deep Research agent | **NT-ACT** + **NT-WORLD** | Multi-step agentic exploration with dead-end detection = SEAL exploration phase |

---

## 4. Llama 4 Scout

**Source**: Meta AI Blog, Model Card (GitHub)

### Architecture Summary

- **Type**: Sparse MoE with early fusion multimodality
- **Parameters**: 17B active / 109B total (16 experts)
- **Context**: 10M tokens (industry-leading)
- **Modalities**: Multilingual text + image input → text + code output
- **Training**: ~40T tokens, 200 languages

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **iRoPE Architecture** | Interleaved attention layers WITHOUT positional embeddings + RoPE in other layers. Temperature scaling at inference for length generalization. Enables "infinite" context. |
| **10M Token Context** | Pre-trained and post-trained at 256K, generalizes to 10M. Needle-in-haystack retrieval across 10M code tokens. |
| **Early Fusion Multimodality** | Text and vision tokens fused in unified backbone from pretraining start (not CLIP-staged). Vision encoder based on MetaCLIP, trained with frozen Llama. |
| **MetaP Hyperparameter Transfer** | Per-layer learning rates and initialization scales transfer well across batch size, model width, depth, and training tokens. Reliable hyperparameter setting. |
| **Alternating Dense + MoE Layers** | MoE layers use 128 routed experts + 1 shared expert. Each token sent to shared expert + 1 routed expert. |
| **Mid-Training Recipe** | Continued training with specialized datasets for long context extension, improving quality while extending context. |
| **Single H100 Deployment** | 17B active params fit on single H100 with int4 quantization. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| iRoPE (position-free attention) | **NT-CORE** (HyperCube) | Position-agnostic embeddings → VSA binding invariant to sequence position |
| 10M context | **NT-MEMORY** (KB) | Ultra-long context = KVMem-style paged KV; hot/cold tiering for context management |
| Early fusion | **NT-WORLD** (SensoryIntegrationHub) | Native multimodal fusion from PerceptionBridge; no staged encoder |
| MetaP transfer | **NT-MIND** | Hyperparameter meta-learning; transferable initialization as skill template |
| Shared + routed experts | **NT-CORE** (GWT) | Shared expert = global workspace broadcast; routed = specialist attention |

---

## 5. DeepSeek V4 Flash

**Source**: DeepSeek-V4 paper (arXiv:2606.19348), HuggingFace model card

### Architecture Summary

- **Type**: Sparse MoE (DeepSeekMoE paradigm)
- **Parameters**: 284B total / 13B activated (Flash); 1.6T total / 49B activated (Pro)
- **Context**: 1M tokens
- **Modalities**: Text
- **Training**: 32T+ tokens, Muon optimizer

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Hybrid CSA + HCA Attention** | Compressed Sparse Attention (compress KV by m=4, then sparse top-k) + Heavily Compressed Attention (m'=128, dense). Interleaved hybrid makes 1M context practical with 27% FLOPs and 10% KV cache vs V3. |
| **Manifold-Constrained Hyper-Connections (mHC)** | Replaces residual connections. Parallel residual streams constrained to a manifold via doubly-stochastic Sinkhorn-Knopp projection. Non-expansive signal propagation across deep stacks. |
| **Hash-MoE Bootstrap** | First few MoE layers use static token-id → expert-id hash function (frozen routing). Learned gate still weights selected experts. Replaces dense FFN in initial layers. |
| **Sqrt(Softplus) Affinity** | Changed routing activation from Sigmoid to Sqrt(Softplus(·)) for smoother expert affinity scoring. |
| **Auxiliary-Loss-Free Load Balancing** | No auxiliary loss for load balancing; uses bias correction buffer. Sequence-wise balance loss prevents extreme imbalance. |
| **Multi-Token Prediction (MTP)** | Predicts multiple future tokens simultaneously (inherited from V3). |
| **Muon Optimizer** | Faster convergence and improved training stability vs Adam. |
| **FP4 Mixed Precision** | MoE experts at FP4, other params at FP8. NVFP4 for Blackwell GPUs. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Hybrid CSA + HCA | **NT-MEMORY** + **NT-CORE** | Adaptive attention: local sliding window + compressed global = GWT local/global routing |
| mHC (Hyper-Connections) | **NT-CORE** (ConsciousnessTree) | Manifold-constrained signal flow = consciousness layers with non-expansive propagation |
| Hash-MoE bootstrap | **NT-MIND** | Static routing for cold-start; learned routing for adaptation = skill tree initialization |
| Sqrt(Softplus) affinity | **NT-CORE** (GWT salience) | Smoother expert selection = continuous salience scoring (not binary gate) |
| Auxiliary-loss-free balancing | **NT-ACT** | Load balancing without penalty = self-organizing task routing |
| FP4 mixed precision | **NT-PHYSICAL** | Hardware-aware quantization; energy-efficient inference |

---

## 6. Qwen 3

**Source**: Qwen3 Technical Report (arXiv:2505.09388), GitHub

### Architecture Summary

- **Type**: Dense + MoE variants
- **Parameters**: 0.6B → 235B (flagship MoE: 235B total / 22B activated)
- **Context**: 128K (up to 1M in 2507 update)
- **Modalities**: Text (+ Qwen-VL for multimodal)
- **Training**: 36T tokens, 119 languages

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Thinking Mode Fusion** | Single model integrates thinking (multi-step reasoning) and non-thinking (fast response) modes. Dynamic switching via /think and /no_think flags in chat template. |
| **Thinking Budget Mechanism** | User controls reasoning depth via token budget. Emergent intermediate handling — model generates based on incomplete thinking when budget exceeded. Not explicitly trained, emerges from mode fusion. |
| **Strong-to-Weak Distillation** | Flagship models (235B, 32B) distilled into smaller models (0.6B-14B). Preserves dual-mode capabilities. |
| **128 Expert MoE** | 128 total experts, 8 activated per token. NO shared experts (unlike Llama 4). Global-batch load balancing loss for specialization. |
| **QK-Norm** | Removed QKV-bias from Qwen2, introduced QK-Norm for stable training. |
| **36T Token Pretraining** | 119 languages (up from 29 in Qwen2.5). 10x more multilingual tokens. |
| **GRPO + SFT Pipeline** | Reasoning RL (GRPO) → Thinking Mode Fusion (continual SFT) → Strong-to-Weak Distillation. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Thinking/non-thinking fusion | **NT-MIND** (SEAL) | Adaptive SEAL cycle depth: quick评估 vs deep reasoning based on task complexity |
| Thinking budget | **NT-CORE** (GWT salience) | Token budget = attention allocation; adaptive compute per salience score |
| Strong-to-weak distillation | **NT-MIND** | Knowledge distillation as skill crystallization; flagship→small skill transfer |
| 128 experts (no shared) | **NT-CORE** (GWT) | Pure specialist routing without global broadcast; contrast with Llama 4's shared+specialist |
| Global-batch load balancing | **NT-ACT** | Task distribution across specialists; prevents expert collapse = domain overfitting |
| Emergent intermediate handling | **NT-REPAIR** | Graceful degradation when budget exceeded; auto-adjust reasoning depth |

---

## 7. Mistral Large 3

**Source**: Mistral AI blog, NVIDIA technical blog, Mistral Technical Documentation

### Architecture Summary

- **Type**: Granular Sparse MoE
- **Parameters**: 675B total / 41B active
- **Context**: 256K tokens
- **Modalities**: Text + Image (native, fused ~2.5B vision encoder)
- **Training**: 3000 NVIDIA H200 GPUs, from scratch

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Granular MoE** | ~16:1 ratio (675B/41B). Expert sub-networks with routing mechanism. Inference cost on par with 40-50B dense model while storing 675B knowledge. |
| **Native Vision Encoder** | ~2.5B parameter vision encoder fused directly into model (not adapter). Enables OCR, document Q&A, layout-aware comprehension. |
| **NVFP4 Quantization** | FP8-scale factors + fine-grained block scaling for MoE weights only. Other components at original precision. Minimal accuracy loss. |
| **Wide Expert Parallelism** | NVIDIA TensorRT-LLM Wide-EP: optimized MoE GroupGEMM kernels, expert distribution and load balancing. Exploits NVL72 coherent memory. |
| **Prefill/Decode Disaggregation** | NVIDIA Dynamo: rate-matching and disaggregating prefill/decode phases for long-context efficiency. |
| **Multi-Token Prediction** | Speculative decoding with EAGLE-3 for throughput improvement. |
| **Ministral Dense Suite** | 3B/8B/14B dense models with Base/Instruct/Reasoning variants (9 models total). Edge-to-cloud continuum. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Granular MoE | **NT-CORE** (GWT) | Extreme sparsity routing; 97% params idle per token = selective attention |
| Native vision fusion | **NT-WORLD** (SensoryIntegrationHub) | PerceptionBridge native multimodal; no adapter staging |
| NVFP4 quantization | **NT-PHYSICAL** | Hardware-aware precision; energy-efficient inference kernel |
| Wide Expert Parallelism | **NT-ACT** | Distributed specialist execution; load balancing across GPU fabric |
| Prefill/decode disaggregation | **NT-IO** | Pipeline separation for latency optimization; streaming architecture |
| Edge model suite | **NT-PHYSICAL** | Body schema: small sensors (3B) → medium actuators (14B) → large brain (675B) |

---

## 8. Phi-4 Reasoning

**Source**: Phi-4-reasoning Technical Report (arXiv:2504.21318), Microsoft Research

### Architecture Summary

- **Type**: Dense decoder-only Transformer
- **Parameters**: 14B (same as Phi-4 base)
- **Context**: 32K tokens (doubled from 16K via RoPE frequency scaling)
- **Modalities**: Text
- **Training**: 16B tokens, 32 H100 GPUs, 2.5 days

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Data-Centric Reasoning** | 1.4M "teachable" prompts filtered at the boundary of base model capability. Quality > quantity. |
| **Synthetic Teacher (o3-mini)** | High-quality reasoning traces generated by o3-mini as supervision. Medium effort = token-efficient; high effort = stronger but longer. |
| **Thinking Tokens** | Repurposed two placeholder tokens as `<think>` and `</think>` to mark reasoning blocks. |
| **RoPE Frequency Doubling** | Doubled RoPE base frequency to extend context from 16K to 32K for reasoning chains. |
| **GRPO Reinforcement Learning** | Group Relative Policy Optimization on 6K math problems. Rule-based reward (no neural reward model). Result: 1.5x longer responses, higher accuracy. |
| **Domain Additivity** | Optimizing individual domains (math, code, logic) independently, then combining yields additive improvements. |
| **Reasoning as Transferable Skill** | Improvements transfer to domains not targeted in training (algorithmic, planning, spatial). 14B outperforms 70B distilled models. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Teachable prompt selection | **NT-MIND** (SEAL) | Task selection at capability boundary = optimal learning signal; VoI-guided experiment design |
| Synthetic teacher distillation | **NT-MIND** | Strong-to-weak transfer; teacher model as Knowledge crystallizer |
| Thinking tokens | **NT-CORE** (ConsciousnessTree) | Explicit reasoning blocks = consciousness cycle markers; SoR→Roots→Trunk |
| Domain additivity | **NT-MEMORY** | Skill composition: independent domain optimization → unified KB fusion |
| Reasoning as transferable skill | **NT-CORE** | Meta-skill transfer across domains = VSA analogical reasoning |
| GRPO with rule-based reward | **NT-GOVERNANCE** | Deterministic reward = constitution-enforced alignment; no reward hacking |

---

## 9. Yi-Lightning

**Source**: Yi-Lightning Technical Report (arXiv:2412.01253)

### Architecture Summary

- **Type**: Enhanced Mixture-of-Experts
- **Parameters**: Undisclosed (MoE with fine-grained experts)
- **Context**: 64K tokens
- **Modalities**: Text
- **Training**: Multi-stage pre-training + SFT + RLHF on XCloud infrastructure

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Fine-Grained Expert Segmentation** | Each expert's FFN partitioned into smaller functional units. Reduces hidden dimensions, increases activated experts per token. Balanced approach to maintain training throughput. |
| **EP + PEP Load Balancing** | Expert Parallel load balancing (group-level) + Partitioned EP load balancing (sub-group). Addresses token dispatching imbalance during All-to-All communication. |
| **Hybrid Attention Blocks** | 3 sliding window attention layers + 1 full attention layer. Captures local patterns + global dependencies efficiently. |
| **Cross-Layer KV Cache Reuse** | Shares KV cache between consecutive full attention layers. 82.8% memory reduction for long sequences. |
| **FP8 Hardware-Aware Design** | Architecture aligned with GPU specs. MoE operator achieves 1,200 TFLOPS/card at FP8 on Hopper. |
| **95% GPU Utilization** | Multi-module, multi-process async scheduling. Decouples task execution, minimizes inter-module latency. |
| **RAISE Safety Engine** | 4-component framework: pre-training safety → post-training optimization → input safety → output safety. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Fine-grained expert segmentation | **NT-CORE** (HyperCube) | VSA component binding: smaller functional units = finer-grained concept decomposition |
| EP + PEP load balancing | **NT-ACT** | Hierarchical task distribution: group-level → partition-level = domain→skill routing |
| Hybrid attention | **NT-CORE** (GWT) | Sliding window = local workspace; full attention = global broadcast. GWT attention modulation. |
| Cross-layer KV reuse | **NT-MEMORY** | KV cache sharing = experience caching across consciousness cycles |
| 95% GPU utilization | **NT-PHYSICAL** | Maximal hardware utilization = body schema optimization |
| RAISE 4-stage safety | **NT-SHIELD** (Rev-明) | Multi-layer defense: pre→post→input→output = Shield's layered protection |

---

## 10. Grok 3

**Source**: xAI blog, Perplexity AI analysis, AI/TLDR, lowtouch.ai

### Architecture Summary

- **Type**: Transformer-based LLM (likely MoE, undisclosed)
- **Parameters**: Undisclosed (speculated 2.7T based on Grok 2 → 3 scaling)
- **Context**: 1M tokens (8x Grok 2)
- **Modalities**: Text (Grok 3); multimodal arrived later with Grok 4
- **Training**: Colossus supercluster, ~200K H100 GPUs, 12.8T tokens

### Key Innovations

| Innovation | Description |
|-----------|-------------|
| **Think Mode (Test-Time Reasoning)** | Explicit reasoning with visible chain-of-thought. Self-corrects errors, explores alternatives. Can spend seconds to minutes reasoning. |
| **DeepSearch Agent** | Agentic web/X research agent. Queries real-time data, synthesizes across sources, produces comprehensive reports. |
| **Colossus Scale** | 200K H100 GPUs. 200M GPU-hours. 10-15x more compute than Grok 2. Completed in 122+92 days. |
| **Synthetic Data Training** | Trained on synthetic datasets simulating real-world scenarios. Reduces web-scraping reliance. Self-correction framework reduced factual errors by 37%. |
| **Cross Expert Attention Gates** | Allows knowledge sharing between specialized expert components without catastrophic interference. |
| **1M Token Context** | 8x larger than Grok 2. State-of-the-art on LOFT (128K RAG benchmark). |
| **Energy Efficiency** | 70W per query (vs 100W for Grok 2). 30% power reduction through optimized neural pathways. |
| **Staggered Curriculum Learning** | 9 training phases progressing from linguistic patterns to complex reasoning. |

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Mapping |
|-----------|-------------------|---------|
| Think mode (visible CoT) | **NT-CORE** (ConsciousnessTree) | Transparent reasoning = consciousness trace; SoR→Roots→Trunk→Branches visible |
| DeepSearch agent | **NT-WORLD** + **NT-ACT** | Multi-source exploration agent = SEAL exploration phase with real-time data |
| Cross expert attention gates | **NT-CORE** (GWT) | Knowledge sharing between specialists without interference = resonance-based routing |
| Synthetic data + self-correction | **NT-MIND** | Synthetic experience generation + self-audit = SEAL self-test phase |
| Staggered curriculum | **NT-MIND** (SEAL) | Progressive training phases = Constellation maturity C0→C5 |
| Energy efficiency | **NT-PHYSICAL** | Power management = body schema thermal optimization |
| 1M context + RAG | **NT-MEMORY** | Ultra-long context retrieval = KB hub with hot/cold tiering |

---

## Cross-Cutting Patterns

### Pattern 1: MoE is Universal

All 10 models either use MoE or have MoE variants. The trend is clear: **sparse activation is the default architecture for 2025-2026**.

| Model | MoE? | Active/Total | Ratio |
|-------|------|-------------|-------|
| GPT-4o | Likely | Undisclosed | — |
| Claude 3.5 Sonnet | No (dense) | — | — |
| Gemini 2.5 Pro | Yes | Undisclosed | — |
| Llama 4 Scout | Yes | 17B/109B | 6.4:1 |
| DeepSeek V4 Flash | Yes | 13B/284B | 21.8:1 |
| Qwen 3 | Yes | 22B/235B | 10.7:1 |
| Mistral Large 3 | Yes | 41B/675B | 16.5:1 |
| Phi-4 Reasoning | No (dense) | — | — |
| Yi-Lightning | Yes | Undisclosed | — |
| Grok 3 | Likely | Undisclosed | — |

**NeoTrix Implication**: GWT salience routing IS MoE at the architecture level. Every specialist module is an "expert"; salience score = routing weight.

### Pattern 2: Thinking Budgets / Test-Time Compute

Models are converging on **user-controllable reasoning depth**:

- **Gemini 2.5 Pro**: Thinking budget tokens
- **Qwen 3**: Thinking/non-thinking mode + budget
- **Grok 3**: Think mode (seconds to minutes)
- **DeepSeek V4**: Think High / Think Max modes
- **Phi-4 Reasoning**: Implicit via CoT length

**NeoTrix Implication**: SEAL pipeline depth should be adaptive. A1 axiom (Cost-Aware Routing) + A2 (Context as Scarce Resource) = dynamic compute allocation per task.

### Pattern 3: Long Context Arms Race

| Model | Context |
|-------|---------|
| GPT-4o | 128K |
| Claude 3.5 Sonnet | 200K |
| Gemini 2.5 Pro | 1M (2M planned) |
| Llama 4 Scout | **10M** |
| DeepSeek V4 | 1M |
| Qwen 3 | 128K (1M in update) |
| Mistral Large 3 | 256K |
| Phi-4 Reasoning | 32K |
| Yi-Lightning | 64K |
| Grok 3 | 1M |

**NeoTrix Implication**: NT-MEMORY needs KVMem-style paged KV virtualization. Adaptive compaction (<256K) vs paged KV (>256K).

### Pattern 4: Synthetic Data + Self-Correction

- **Grok 3**: Synthetic training data, self-correction framework (37% error reduction)
- **Phi-4 Reasoning**: 1.4M synthetic reasoning traces from o3-mini
- **Qwen 3**: Strong-to-weak distillation from flagship
- **DeepSeek V4**: On-policy distillation after domain expert cultivation

**NeoTrix Implication**: SEAL pipeline should generate synthetic experience → self-audit → distill → crystallize. The experience-tree absorption protocol.

### Pattern 5: Hardware-Aware Architecture

- **Yi-Lightning**: FP8-native design, 1200 TFLOPS/card
- **Mistral Large 3**: NVFP4 for Blackwell, Wide Expert Parallelism
- **DeepSeek V4**: FP4+FP8 mixed precision
- **Llama 4 Scout**: Single H100 deployment with int4

**NeoTrix Implication**: NT-PHYSICAL body schema must be hardware-aware. Quantization strategy = energy management in the embodiment layer.

---

## Summary: Top 10 NeoTrix-Architecture Mappings

| # | Model Innovation | NeoTrix Mapping | Priority |
|---|-----------------|-----------------|----------|
| 1 | MoE sparse routing | GWT salience routing | P0 (already exists) |
| 2 | Thinking budgets | SEAL adaptive depth | P0 |
| 3 | 10M context (Llama 4 iRoPE) | NT-MEMORY paged KV | P1 |
| 4 | mHC hyper-connections (DeepSeek) | ConsciousnessTree signal flow | P1 |
| 5 | Hybrid CSA+HCA attention (DeepSeek) | GWT local/global attention | P1 |
| 6 | Strong-to-weak distillation (Qwen/Phi) | Skill crystallization | P1 |
| 7 | End-to-end multimodal (GPT-4o) | PerceptionBridge fusion | P2 |
| 8 | Constitutional AI / RAISE | NT-GOVERNANCE + NT-SHIELD | P2 |
| 9 | Computer Use / GUI Agent | NT-ACT tool scaffolding | P2 |
| 10 | Hardware-aware quantization | NT-PHYSICAL body schema | P2 |

---

*Generated from 10 model architectures, ~50 search results, cross-referenced with NeoTrix CONTEXT.md and AGENTS.md.*
