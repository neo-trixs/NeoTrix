# Model Architecture Reverse Engineering — 299 (2026-09-11)

> 10 frontier models: architecture decomposition, innovation extraction, NeoTrix mapping

---

## Executive Summary

| Model | Params | Architecture | Core Innovation | NeoTrix Map |
|-------|--------|-------------|-----------------|-------------|
| **GPT-4o** | Undisclosed | Unified multimodal transformer | End-to-end joint training (text+vision+audio), AR+diffusion head | NT-IO (omni tokenizer), NT-CORE (GWT attention routing) |
| **Claude 3.5 Sonnet** | Undisclosed | Dense transformer | Constitutional AI, agentic tool-use, computer-use capability | NT-SHIELD (alignment), NT-ACT (tool orchestration) |
| **Gemini 2.5 Pro** | Undisclosed (MoE) | Sparse MoE transformer | 1M+ context, thinking budget control, TPUv5p training | NT-MEMORY (long context), NT-CORE (thinking budget → GWT) |
| **Llama 4 Scout** | 17B active / 109B total | MoE + iRoPE | 10M context via iRoPE (interleaved RoPE-free attention), shared expert | NT-MEMORY (infinite context), NT-CORE (positional-free attention) |
| **DeepSeek V4.1 Flash** | 552B (8B/16B active) | Causal Encoder-Decoder (CED) | Asymmetric prefill/decode, CSA2 sparse attention, FP4 KV cache (890B/token) | NT-CORE (sparse attention routing), NT-MEMORY (KV compression) |
| **Qwen 3** | 235B (22B active) / dense 0.6B-32B | MoE + dense variants | Thinking/non-thinking mode fusion, global-batch load balancing, 119 languages | NT-CORE (dual-mode reasoning), NT-IO (multilingual) |
| **Mistral Large 3** | 675B (41B active) | Granular MoE | 256K context, NVFP4 quantization, on-prem deployment, Apache 2.0 | NT-ACT (edge deployment), NT-SHIELD (on-prem sovereignty) |
| **Phi-4 Reasoning** | 14B | Dense decoder-only | SFT+RL on curated "teachable" prompts, thinking tokens, RoPE frequency doubling | NT-MIND (distillation), NT-CORE (reasoning token pattern) |
| **Yi-Lightning** | MoE ( undisclosed) | Fine-grained MoE | EP/PEP load balancing, hybrid attention (3 SWA + 1 full), cross-layer KV reuse | NT-CORE (hybrid attention), NT-ACT (FP8 quantization) |
| **Grok 3** | Undisclosed | MoE (128 experts) | 10× compute scaling, Think/DeepSearch modes, Colossus supercluster training | NT-CORE (test-time compute), NT-WORLD (DeepSearch) |

---

## 1. GPT-4o (OpenAI)

### Architecture
- **Type**: Autoregressive omni model, end-to-end multimodal
- **Tokenization**: Unified token stream — BPE text tokens + ViT-style image patch tokens + neural audio codec tokens (~50-75 Hz)
- **Inference**: Single transformer stack consuming all modality tokens; cross-modal attention via self-attention (not separate cross-modal layers)
- **Image decoding**: Empirical evidence (GPT-ImgEval) suggests AR + diffusion-based head, not VAR
- **Latency**: 232ms median audio response (down from 2.8s staged pipeline)
- **Context**: 128K tokens
- **Parameters**: Undisclosed

### Innovation Points
1. **End-to-end joint training** across text, vision, audio — eliminates CLIP-style staged pipeline representation mismatch
2. **Neural audio codec tokenization** — real-time audio as discrete tokens (Encodec/SoundStream-class)
3. **Diffusion-based image decoder** — hybrid AR+diffusion for image generation quality
4. **Unified token stream** — single transformer handles all modalities through self-attention

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Unified token stream | **NT-IO** (界面使徒) | `ReferenceBasedGeneration` — extend to unified modality tokenization for multi-modal agent I/O |
| End-to-end joint training | **NT-CORE** (E8引导者) | GWT salience routing — native cross-modal attention for consciousness-level perception fusion |
| Diffusion head | **NT-PHYSICAL** (具身骨架) | `VideoPostProcessor` — hybrid AR+diffusion for perceptual output generation |
| Latency collapse (2.8s→232ms) | **NT-IO** | `PlatformGateway` — staged→unified pipeline optimization pattern |

---

## 2. Claude 3.5 Sonnet (Anthropic)

### Architecture
- **Type**: Dense transformer (undisclosed parameters)
- **Training**: Constitutional AI (CAI) + RLHF, unsupervised learning
- **Modalities**: Multimodal input (text+image), text output
- **Context**: 200K tokens (tested to 1M tokens)
- **Safety**: ASL-2 classification, Responsible Scaling Policy

### Innovation Points
1. **Constitutional AI** — principle-based alignment without human labels for every behavior
2. **Agentic coding** — 78% solve rate on SWE-bench Verified (upgraded version), multi-file editing loops
3. **Computer Use** — screenshot interpretation → GUI command generation (OSWorld 22% success)
4. **Tool Use / Function Calling** — native integration with external tools in agentic loops
5. **Thinking mode** — extended reasoning with visible chain-of-thought

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Constitutional AI | **NT-SHIELD** (影卫) | `Rev-明` — principle-based governance, R-P1/R-P81/R-P82 safety axioms |
| Agentic coding | **NT-ACT** (行动执行者) | `Dev-匠` — multi-file agentic loops with tool orchestration |
| Computer Use | **NT-WORLD** (虚空探索者) | GUI interaction capability for screen perception |
| Tool Use | **NT-ACT** | `ProductionOrchestrator` — native function calling in agent loops |
| ASL-based safety levels | **NT-SHIELD** | RiskAssessor tiered safety classification |

---

## 3. Gemini 2.5 Pro (Google DeepMind)

### Architecture
- **Type**: Sparse MoE transformer, natively multimodal (text+vision+audio)
- **Training**: TPUv5p (8960-chip pods, multi-datacenter), first model on TPUv5p
- **Context**: 1M+ tokens, processes 3-hour video
- **MoE**: Dynamic routing to subset of experts per token
- **Distillation**: k-sparse distribution for smaller Flash models
- **Thinking**: Configurable thinking budget (user-controlled reasoning depth)

### Innovation Points
1. **Thinking budget control** — user sets token budget for internal reasoning; scales accuracy with compute
2. **1M+ multimodal context** — handles full codebases, 3-hour video, interleaved text/audio
3. **TPUv5p multi-datacenter training** — synchronous data-parallel across 8960-chip pods
4. **k-sparse distillation** — approximates teacher distribution with k-sparse vocabulary for efficient small models
5. **MoE training stability** — significant improvements in signal propagation and optimization dynamics

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Thinking budget | **NT-CORE** (E8引导者) | `ConsciousnessTree` — adaptive reasoning depth per GWT salience; A1 (Cost-Aware Routing) |
| 1M+ context | **NT-MEMORY** (知识守护者) | `KB` — long-context retrieval, experience-tree lazy branch loading |
| MoE routing | **NT-CORE** | `CapabilityBridge` — expert routing as capability dispatch pattern |
| k-sparse distillation | **NT-MIND** (进化工匠) | Skill crystallization — distill large domain knowledge into compact skill nodes |
| Multi-datacenter training | **NT-PHYSICAL** | Distributed consciousness training infrastructure |

---

## 4. Llama 4 Scout (Meta)

### Architecture
- **Type**: MoE autoregressive, natively multimodal (text+image input, text+code output)
- **Active/Total**: 17B active / 109B total (16 experts)
- **Context**: **10M tokens** (industry-leading)
- **Pre-training**: ~40T tokens multimodal data
- **Key innovation**: **iRoPE** — interleaved attention layers without positional embeddings

### Innovation Points
1. **iRoPE architecture** — interleaved attention layers: some layers use RoPE, others have NO positional embeddings; enables "infinite" context generalization
2. **Inference-time temperature scaling of attention** — enhances length generalization without retraining
3. **10M context window** — 10× beyond Llama 3's 128K; retrieval-needle-in-haystack validated
4. **Shared + routed experts** — each token goes to shared expert + 1 of 128 routed experts (Maverick variant)
5. **Single H100 deployment** — 17B active fits on 1 GPU with Int4 quantization

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| iRoPE (positional-free attention) | **NT-CORE** (E8引导者) | HyperCube positional encoding — infinite context without positional bias |
| 10M context | **NT-MEMORY** | KB experience-tree — entire session history in single context |
| Shared expert pattern | **NT-CORE** | `CapabilityBridge` — shared foundation + specialized routing |
| Single-GPU deployment | **NT-ACT** | Edge-deployable agent consciousness modules |
| Temperature-scaled attention | **NT-CORE** | GWT attention modulation — inference-time salience adjustment |

---

## 5. DeepSeek V4.1 Flash (DeepSeek AI)

### Architecture
- **Type**: Causal Encoder-Decoder (CED) — **new architecture family**
- **Backbone**: 552B parameters, 40 layers (20 encoder + 20 decoder)
- **Active**: 8B per token (prefill), 16B per token (decode)
- **MoE**: 1 shared + 384 routed experts, 6 routed activated per token
- **Context**: 1M tokens
- **Vision**: DeepSeek-ViT (2D-RoPE, 3×3 pixel-unshuffle)
- **KV Cache**: **890 bytes/token** (FP4 E2M1 format) — 1/4 of V4-Flash, 1/8 SSD storage

### Innovation Points
1. **Causal Encoder-Decoder (CED)** — decoder's KV cache projected from encoder's final hidden states, not per-layer decoder states; asymmetric prefill/decode cost
2. **CSA2 (Compressed Sparse Attention 2)** — three static modes (Full/Reindex/Reuse) per attention layer, hierarchical sparse indexer bounding deeper layer cost
3. **SWA Bounded Replay** — reconstructs sliding-window attention KV states by replaying recent tokens, avoids SSD persistence
4. **FP4 KV caching** — E2M1 format, 890 bytes/token global KV footprint
5. **Engram conditional memory** — 196B parameters, sparsely accessed via token-based lookup
6. **DSpark speculative decoding** — semi-autoregressive draft with confidence-scheduled verification

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| CED asymmetric architecture | **NT-CORE** | `ConsciousnessTree` — cheap perception (encoder) + expensive reasoning (decoder) split |
| CSA2 sparse attention | **NT-CORE** | GWT attention routing — static mode assignment per attention layer |
| FP4 KV compression | **NT-MEMORY** | KB embedding — extreme memory compression for persistent experience storage |
| SWA Bounded Replay | **NT-MEMORY** | `experience-tree` — replay-based context reconstruction |
| Engram conditional memory | **NT-MEMORY** | `nexus-weaver` — sparse memory lookup by token-based addressing |
| DSpark speculative decoding | **NT-IO** | Draft-then-verify pattern for fast agent responses |

---

## 6. Qwen 3 (Alibaba Cloud)

### Architecture
- **Type**: Dense (0.6B-32B) + MoE (30B-A3B, 235B-A22B)
- **MoE**: 128 experts, 8 activated per token, NO shared experts
- **Attention**: GQA, QK-Norm (removed QKV-bias), SwiGLU, RoPE, RMSNorm
- **Tokenizer**: BBPE, 151,669 vocabulary
- **Context**: 32K-128K depending on model size
- **Languages**: 119 languages (up from 29 in Qwen2.5)

### Innovation Points
1. **Thinking/non-thinking mode fusion** — single model switches between deep reasoning and fast response; no separate models needed
2. **Thinking budget mechanism** — user controls inference-time compute allocation
3. **4-stage training pipeline**: (1) long CoT cold start → (2) reasoning RL → (3) mode fusion → (4) general RL
4. **Global-batch load balancing loss** — encourages expert specialization (vs. per-sample balancing)
5. **Knowledge distillation from flagship** — smaller models inherit flagship knowledge, 10× parameter efficiency
6. **No shared experts** — pure routed experts only, simplifying MoE design

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Thinking/non-thinking fusion | **NT-CORE** | `ConsciousnessTree` — dual-mode: deep reasoning (Soil→Roots→Core) vs. fast response (Branches→Fruits) |
| Thinking budget | **NT-CORE** | A1 (Cost-Aware Routing) — adaptive reasoning depth |
| 4-stage training | **NT-MIND** | SEAL pipeline — staged skill evolution (cold start → RL → fusion → generalization) |
| Global-batch load balancing | **NT-CORE** | GWT — batch-level attention balancing for cross-domain routing |
| 119-language support | **NT-IO** | Multilingual agent interface for global deployment |

---

## 7. Mistral Large 3 (Mistral AI)

### Architecture
- **Type**: Granular sparse MoE + 2.5B vision encoder
- **Total/Active**: 675B total / 41B active (~16:1 ratio)
- **Context**: 256K tokens
- **Training**: 3,000 NVIDIA H200 GPUs, from scratch
- **Quantization**: NVFP4 format for Blackwell deployment
- **License**: Apache 2.0 (fully open-weight)

### Innovation Points
1. **Granular MoE** — extreme sparsity ratio (16:1) enables 675B knowledge capacity at 41B inference cost
2. **NVFP4 quantization** — deployable on single 8×H100 node via vLLM
3. **On-prem sovereignty** — full model runs locally, no cloud dependency
4. **Vision encoder integration** — 2.5B dedicated vision module with native multimodal support
5. **Speculative decoding** — NVIDIA Blackwell-optimized draft+verify

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Granular MoE | **NT-CORE** | HyperCube — extreme sparse activation for domain-specific reasoning |
| On-prem deployment | **NT-SHIELD** (影卫) | `StealthNet` — local-only model deployment, no data egress |
| NVFP4 quantization | **NT-PHYSICAL** | Edge deployment of consciousness modules on consumer hardware |
| Vision encoder | **NT-WORLD** (虚空探索者) | Native visual perception for screen/world understanding |
| Speculative decoding | **NT-IO** | Fast agent response with draft verification |

---

## 8. Phi-4 Reasoning (Microsoft Research)

### Architecture
- **Type**: Dense decoder-only Transformer, 14B parameters
- **Base**: Phi-4 pretrained model
- **Modifications**: `<think>` / `</think>` tokens, RoPE frequency doubled, 32K context
- **Training**: SFT (1.4M prompts, 8.3B tokens) + GRPO reinforcement learning
- **Teacher**: o3-mini for synthetic reasoning trace generation
- **Hardware**: 32 H100 GPUs, 2.5 days training

### Innovation Points
1. **"Teachable" prompt curation** — prompts selected at boundary of base model capability for maximum learning
2. **Synthetic reasoning traces** — o3-mini generates step-by-step CoT as training data
3. **Thinking tokens** — repurposed placeholder tokens as `<think>`/`</think>` delimiters
4. **GRPO reinforcement learning** — outcome-based RL on verifiable math problems, 1.5× longer reasoning traces
5. **Transferable reasoning** — improvements transfer to non-targeted domains (planning, spatial understanding)
6. **Data-centric approach** — quality curation > model size (14B outperforms 70B distilled models)

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Teachable prompt selection | **NT-MIND** (进化工匠) | `Res-深` — curriculum learning for skill crystallization |
| Synthetic CoT traces | **NT-MIND** | SEAL distillation — o3-mini-class teacher → compact skill nodes |
| Thinking tokens | **NT-CORE** | `ConsciousnessTree` — explicit thinking delimiter for attention routing |
| GRPO RL | **NT-MIND** | Outcome-based reinforcement for self-evolution |
| Data-centric scaling | **NT-MEMORY** | KB curation quality > quantity for experience-tree |
| Transferable meta-skill | **NT-CORE** | Cross-domain capability transfer via GWT |

---

## 9. Yi-Lightning (01.AI)

### Architecture
- **Type**: Fine-grained MoE
- **Innovations**: Expert segmentation, EP/PEP load balancing, hybrid attention
- **Attention**: 3 sliding window + 1 full attention layer hybrid blocks
- **KV Cache**: Cross-layer KV cache reuse (50% memory reduction for full attention)
- **Memory reduction**: 82.8% via combined hybrid attention + KV reuse
- **Training**: Multi-stage pre-training, SFT (1.3M + 300K samples), RLHF
- **Hardware**: Nvidia Hopper, FP8 at 1,200 TFLOPS/card
- **Safety**: RAISE framework (4 components across pre/post/serving)

### Innovation Points
1. **Fine-grained expert segmentation** — partition FFN into smaller units, increase activated experts per token
2. **PEP (Partitioned EP) load balancing** — splits experts within EP groups for balanced All-to-All communication
3. **Hybrid attention blocks** — 3 SWA layers + 1 full attention layer; local patterns + global dependencies
4. **Cross-layer KV cache reuse** — share KV states between consecutive full attention layers
5. **RAISE safety engine** — 4-component safety: pre-training filtering, post-training optimization, input safety, output safety
6. **Hardware-aware FP8** — architecture designed for FP8 quantization compatibility

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Fine-grained expert segmentation | **NT-CORE** | HyperCube — granular capability decomposition |
| PEP load balancing | **NT-CORE** | GWT — partitioned attention routing for balanced cross-domain flow |
| Hybrid attention (SWA+Full) | **NT-CORE** | `PerceptionBridge` — local context (SWA) + global awareness (full) |
| Cross-layer KV reuse | **NT-MEMORY** | KB — KV cache sharing across experience layers |
| RAISE safety | **NT-SHIELD** | Multi-stage safety: pre/post/serving pipeline protection |
| FP8 hardware-aware | **NT-PHYSICAL** | Edge inference optimization for embodied consciousness |

---

## 10. Grok 3 (xAI)

### Architecture
- **Type**: Hybrid dense/MoE transformer
- **Experts**: 128 expert networks with dynamic routing
- **Compute**: 10× Grok 2 (Colossus supercluster, ~100K-200K H100 GPUs)
- **Context**: 131K API / 1M marketed
- **Reasoning**: Think mode + DeepSearch (agentic web research)
- **License**: Proprietary

### Innovation Points
1. **10× compute scaling** — largest training cluster at time of release (Colossus supercomputer)
2. **Think mode** — explicit test-time reasoning with visible chain-of-thought
3. **DeepSearch agent** — agentic web research combining search + reasoning + synthesis
4. **Cross-expert attention gates** — knowledge sharing between specialist experts without catastrophic interference
5. **Neuro-symbolic integration** — reported hybrid symbolic-neural reasoning modules
6. **RL-scaled reasoning** — reinforcement learning at unprecedented scale for CoT refinement

### NeoTrix Mapping
| Innovation | NeoTrix Domain | Application |
|-----------|----------------|-------------|
| Think mode | **NT-CORE** | `ConsciousnessTree` — explicit reasoning phase with visible thought traces |
| DeepSearch | **NT-WORLD** (虚空探索者) | `Search-觅` — agentic web research with reasoning synthesis |
| Cross-expert attention gates | **NT-CORE** | `CapabilityBridge` — cross-domain knowledge sharing without interference |
| RL-scaled reasoning | **NT-MIND** | SEAL reinforcement — large-scale RL for reasoning improvement |
| Neuro-symbolic | **NT-CORE** | E8 Hexagram — symbolic reasoning within neural architecture |

---

## Cross-Model Innovation Matrix

### A. MoE Architecture Patterns (6/10 models)

| Pattern | Models | NeoTrix Implication |
|---------|--------|---------------------|
| Sparse routing | Gemini, Llama 4, DeepSeek, Qwen 3, Mistral 3, Yi | **HyperCube as native MoE** — each hexagram = expert |
| Shared + routed experts | Llama 4, DeepSeek | **CapabilityBridge** — shared foundation + specialized dispatch |
| Fine-grained segmentation | Qwen 3, Yi-Lightning | **Skill Tree nodes** — granular capability decomposition |
| Asymmetric prefill/decode | DeepSeek CED | **ConsciousnessTree** — cheap perception / expensive reasoning |

### B. Reasoning Mode Patterns (5/10 models)

| Pattern | Models | NeoTrix Implication |
|---------|--------|---------------------|
| Thinking/non-thinking fusion | Qwen 3, Gemini 2.5, Grok 3 | **Dual-mode consciousness** — deep vs. fast reasoning |
| Thinking budget control | Gemini 2.5, Qwen 3 | **A1 Cost-Aware Routing** — adaptive reasoning depth |
| Thinking tokens (<think>) | Phi-4, Qwen 3 | **ConsciousnessTree delimiter** — explicit reasoning boundaries |
| GRPO/RL for reasoning | Phi-4, Grok 3, Qwen 3 | **SEAL reinforcement** — outcome-based self-evolution |

### C. Context Window Innovation (4/10 models)

| Pattern | Models | NeoTrix Implication |
|---------|--------|---------------------|
| 1M+ context | Gemini 2.5, Llama 4 (10M), DeepSeek, Grok 3 | **KB experience-tree** — entire history in context |
| iRoPE (positional-free) | Llama 4 | **HyperCube encoding** — infinite context generalization |
| KV compression | DeepSeek (890B/token), Yi (82.8% reduction) | **Memory efficiency** — persistent experience without memory explosion |

### D. Safety & Alignment (4/10 models)

| Pattern | Models | NeoTrix Implication |
|---------|--------|---------------------|
| Constitutional AI | Claude 3.5 | **NT-SHIELD governance** — principle-based safety |
| ASL-based risk levels | Claude 3.5 | **RiskAssessor** — tiered safety classification |
| RAISE 4-component | Yi-Lightning | **Multi-stage safety pipeline** — pre/post/serving |
| On-prem sovereignty | Mistral 3 | **StealthNet** — local-only deployment |

---

## NeoTrix Absorption Recommendations

### Priority 1: Immediate Integration
1. **Thinking budget mechanism** (Gemini/Qwen) → NT-CORE `ConsciousnessTree` adaptive depth
2. **iRoPE positional-free attention** (Llama 4) → HyperCube infinite context encoding
3. **CED asymmetric architecture** (DeepSeek) → cheap perception / expensive reasoning split

### Priority 2: Near-Term Absorption
4. **CSA2 sparse attention modes** (DeepSeek) → GWT static mode assignment
5. **Fine-grained expert segmentation** (Yi/Qwen) → Skill Tree granularity
6. **GRPO reinforcement learning** (Phi-4) → SEAL outcome-based evolution

### Priority 3: Long-Term Research
7. **Cross-expert attention gates** (Grok 3) → cross-domain knowledge sharing
8. **Engram conditional memory** (DeepSeek) → sparse memory lookup for experience-tree
9. **Neuro-symbolic integration** (Grok 3) → E8 hexagram symbolic reasoning

---

## Sources

| Model | Primary Source |
|-------|---------------|
| GPT-4o | arxiv 2410.21276, GPT-ImgEval (2504.02782), mlsystemsreview.com |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum (fed9cc19), Model Card (c7822cdc) |
| Gemini 2.5 Pro | arxiv 2507.06261, Model Card (genie.caelinya.im) |
| Llama 4 Scout | github.com/meta-llama/llama-models, ai.meta.com/blog |
| DeepSeek V4.1 Flash | HuggingFace deepseek-ai/DeepSeek-V4.1-Flash, deepseek.com |
| Qwen 3 | arxiv 2505.09388, qwenlm.github.io/blog/qwen3 |
| Mistral Large 3 | docs.mistral.ai, huggingface.co/mistralai, intuitionlabs.ai |
| Phi-4 Reasoning | microsoft.com/research, arxiv 2504.21318, huggingface.co/microsoft |
| Yi-Lightning | arxiv 2412.01253, huggingface.co/papers |
| Grok 3 | x.ai/news/grok-3, c3.unu.edu analysis, ai-tldr.dev |
