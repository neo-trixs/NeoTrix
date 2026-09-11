# Model Reverse-Engineer 269 — 10-Model Architecture Extraction & NeoTrix Mapping

**Date**: 2026-09-11
**Models**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4, Qwen3, Mistral Large 3, Phi-4-reasoning, Yi-Lightning, Grok 3

---

## 1. Per-Model Architecture Innovations

### 1.1 GPT-4o (OpenAI)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | Undisclosed (est. ~200B params). Unified end-to-end multimodal — text, audio, image, video processed by single neural net, not chained pipelines |
| **Key Innovation** | **Omni-modal unification**: eliminates ASR→LLM→TTS cascade. Sub-400ms audio response (vs 5.4s for GPT-4 Turbo chain). Unified token space across modalities |
| **Context** | 128K tokens |
| **Reasoning** | Non-reasoning (fast), with o-series as separate reasoning models |
| **Training** | RLHF + custom post-training. Data up to Oct 2023 |

### 1.2 Claude 3.5 Sonnet (Anthropic)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | Undisclosed dense Transformer. "Architectural tweaks" + AI-generated training data |
| **Key Innovation** | **Constitutional AI (CAI)** post-training with RLAIF. Artifacts system (real-time workspace generation). SWE-bench Verified 49% (state-of-the-art at release) |
| **Context** | 200K tokens |
| **Reasoning** | Non-reasoning. Claude 3.5 Haiku achieves 40.6% SWE-bench, showing small-model agentic capability |
| **Training** | RLHF + DPO. Emphasis on instruction following, agentic coding |

### 1.3 Gemini 2.5 Pro (Google DeepMind)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | Transformer with native multimodal input (text, audio, image, video, PDF). Native audio output via causal audio representations |
| **Key Innovation** | **Native thinking mode** — model decides inference-time compute budget per query. 1M token context window (1,048,576 tokens). Audio: 32 tokens/second, streaming dialog. Video: up to 3 hours processing. Deep Think experimental mode for maximum reasoning |
| **Context** | 1M tokens |
| **Reasoning** | **Thinking budget** — adaptive reasoning compute per query. Single model handles both fast and deep reasoning |
| **Training** | RL for thinking capability. LearnLM integration for educational alignment. 200+ language pre-training |

### 1.4 Llama 4 Scout (Meta)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | **MoE (Mixture-of-Experts)**: 17B active / 109B total params, 16 experts. Auto-regressive Transformer with early fusion for native multimodality |
| **Key Innovation** | **iRoPE** for 10M token context. Early fusion multimodality (text+image+video in pre-training, not post-hoc). FP8 training at 390 TFLOPs/GPU. **Behemoth distillation** — 288B active teacher model distills knowledge to Scout/Maverick |
| **Context** | **10M tokens** (Scout), 1M (Maverick) |
| **Reasoning** | Non-reasoning |
| **Training** | ~40T tokens. 200 languages. FP8 precision without quality loss |

### 1.5 DeepSeek V4 (DeepSeek AI)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | **MoE (DeepSeekMoE)**: Pro=1.6T total / 49B active; Flash=285B / 13B active. Hybrid attention: **CSA** (Compressed Sparse Attention, m=4) + **HCA** (Heavily Compressed Attention, m'=128). **mHC** (Manifold-Constrained Hyper-Connections) replaces residual connections |
| **Key Innovation** | **Hybrid Attention Architecture**: CSA compresses KV 4× with overlapping windows + Lightning Indexer for top-k block selection; HCA compresses 128× with dense attention. **mHC**: doubly-stochastic matrix constraint (Birkhoff polytope) for stable signal propagation through 61 layers. **Muon Optimizer**: matrix-level orthogonalized gradient updates. **Multi-Token Prediction (MTP)** retained from V3. Hash-MoE bootstrap (first 3 layers use static token→expert mapping) |
| **Context** | 1M tokens |
| **Reasoning** | 3 modes: Non-think (fast), Think High, Think Max |
| **Training** | 32T tokens (Flash), 33T (Pro). FP4 routed experts. Post-training: domain expert cultivation + on-policy distillation |

### 1.6 Qwen3 (Alibaba)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | Dense + MoE variants. Flagship Qwen3-235B-A22B (235B total, 22B active). BBPE tokenizer (151,669 vocab). QK-Norm (removed QKV-bias from Qwen2) |
| **Key Innovation** | **Unified thinking/non-thinking modes** in single model. Thinking budget mechanism for adaptive compute. Global-batch load balancing loss for MoE expert specialization. 36T token pre-training. PDF text extraction via fine-tuned Qwen2.5-VL. 119 language support (up from 29) |
| **Context** | 32K tokens (dense) |
| **Reasoning** | **Dual-mode**: thinking (complex reasoning) + non-thinking (fast responses) in same model |
| **Training** | ~36T tokens. Multi-stage data curation with domain annotations |

### 1.7 Mistral Large 3 (Mistral AI)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | **Granular MoE**: 41B active / 675B total, **128 experts per layer**. Multi-Latent Attention. 2.5B vision encoder |
| **Key Innovation** | **Granular MoE** — 128 experts (vs typical 8-16) for finer-grained specialization. **EAGLE speculative decoding** for inference acceleration. NVFP4 quantization for single-node deployment (8×A100/8×H100). Prefill/decode disaggregated serving. Apache 2.0 license |
| **Context** | 256K tokens |
| **Reasoning** | Non-reasoning (reasoning version planned) |
| **Training** | 3000 H200 GPUs from scratch |

### 1.8 Phi-4-reasoning (Microsoft)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | Dense decoder-only Transformer, **14B parameters**. Same as Phi-4 base with 2 modifications: repurposed `<think></think>` tokens, extended context to 32K |
| **Key Innovation** | **Data-centric SFT** — 1.4M carefully curated "teachable" prompts with o3-mini generated reasoning traces. **Phi-4-reasoning-plus**: short RL phase on top of SFT for longer traces. Competes with 5-50× larger models (DeepSeek-R1-Distill-Llama-70B, approaches full DeepSeek-R1 on AIME 2025) |
| **Context** | 32K tokens |
| **Reasoning** | Chain-of-thought with `<think></think>` markers. Parallel test-time compute (Maj@N, Best of 64) |
| **Training** | SFT on 1.4M prompts + o3-mini traces. Plus variant adds outcome-based RL |

### 1.9 Yi-Lightning (01.AI)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | **Enhanced MoE**: ~100B total params. Fine-grained expert segmentation + balanced expert routing + cross-layer KV cache sharing |
| **Key Innovation** | **Fine-grained expert segmentation** (more experts, smaller per-expert). Cross-layer KV cache sharing for inference efficiency. FP8 quantization co-designed with hardware. RAISE safety framework (4-component: data filtering, post-training, serving, monitoring) |
| **Context** | 128K tokens (estimated) |
| **Reasoning** | Non-reasoning |
| **Training** | BPE tokenizer (100,352 vocab). Multi-stage pre-training + SFT + RLHF |

### 1.10 Grok 3 (xAI)

| Dimension | Detail |
|-----------|--------|
| **Architecture** | Hybrid dense/MoE architecture (undisclosed). Estimated ~1.2T params |
| **Key Innovation** | **Colossus supercluster** — 100K→200K H100 GPUs (5× predecessor). 200M GPU hours, ~$420M training cost. **Test-time compute at scale (TTCS)**. Three reasoning modes: Think, Big Brain (extra compute), DeepSearch (web agent). Largest synthetic dataset ever assembled |
| **Context** | 131K tokens (API), claimed 1M |
| **Reasoning** | **Multi-tier reasoning**: Think (chain-of-thought) → Big Brain (extended compute) → DeepSearch (web-augmented agent) |
| **Training** | 10× compute of predecessor. RL for reasoning. Knowledge from real-time X/Twitter data |

---

## 2. Cross-Model Innovation Matrix

| Innovation | GPT-4o | Claude 3.5 | Gemini 2.5 | Llama 4 | DeepSeek V4 | Qwen3 | Mistral L3 | Phi-4-r | Yi-Light | Grok 3 |
|-----------|--------|------------|------------|---------|-------------|-------|------------|---------|----------|--------|
| **MoE** | — | — | — | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| **Native Multimodal** | ✓ | — | ✓ | ✓ | — | ✓ (VL) | ✓ | — | — | — |
| **Thinking/Reasoning Mode** | — | — | ✓ | — | ✓ | ✓ | — | ✓ | — | ✓ |
| **1M+ Context** | — | — | ✓ | ✓ (10M) | ✓ | — | — | — | — | — |
| **Hybrid Attention** | — | — | — | — | ✓ | — | — | — | — | — |
| **mHC (Hyper-Connections)** | — | — | — | — | ✓ | — | — | — | — | — |
| **Muon Optimizer** | — | — | — | — | ✓ | — | — | — | — | — |
| **Speculative Decoding** | — | — | — | — | — | — | ✓ (EAGLE) | — | — | — |
| **Distillation from Larger Teacher** | — | — | — | ✓ | ✓ | ✓ | — | ✓ | — | ✓ |
| **Open Weights** | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| **Test-Time Compute Scaling** | — | — | ✓ | — | ✓ | ✓ | — | ✓ | — | ✓ |
| **KV Cache Optimization** | — | — | — | — | ✓ | — | — | — | ✓ | — |

---

## 3. NeoTrix Mapping — Architectural Innovations → NT-* Domains

### 3.1 Attention & Routing

| External Innovation | NeoTrix Component | NT Domain | Integration Path |
|---------------------|-------------------|-----------|------------------|
| **Gemini Thinking Budget** | GWT salience + AttentionManager | NT-CORE | Adaptive compute allocation per task — GWT broadcasts "thinking depth" signal; ConsciousnessTree adjusts phi/complexity |
| **DeepSeek CSA/HCA Hybrid Attention** | PerceptionBridge (L2→L5) | NT-WORLD / NT-CORE | Layered attention: CSA for fine-grained crawl results (L2), HCA for broad knowledge aggregation (L5). Lightning Indexer → GWT salience scoring |
| **Grok Multi-tier Reasoning** | ConsciousnessTree 6-stage loop | NT-META | Think/Big Brain/DeepSearch map to Soil→Roots→Trunk→Branches→Fruits→Core. DeepSearch = NT-WORLD crawl integration |
| **Llama 4 iRoPE (10M context)** | KVMem paged KV | NT-MEMORY | iRoPE-inspired positional encoding for >256K sessions. Aligns with Axiom A2 (Context as Scarce Resource) |

### 3.2 Mixture-of-Experts (MoE)

| External Innovation | NeoTrix Component | NT Domain | Integration Path |
|---------------------|-------------------|-----------|------------------|
| **Mistral Granular MoE (128 experts)** | Skill Tree 3-tier nodes | NT-MIND | Fine-grained skill specialization: Small Passive → Notable Passive → Keystone. 128-expert inspiration for finer-grained domain mapping |
| **DeepSeek Hash-MoE Bootstrap** | Constellation C0-C4 maturity | NT-MIND | Static token→expert mapping for bootstrap layers → C0 (compile) stage uses deterministic routing; C4+ uses learned routing |
| **Yi-Lightning Cross-layer KV Sharing** | KB embedding cache | NT-MEMORY | Shared KV across domain modules reduces redundancy. Maps to KB node deduplication |
| **DeepSeek mHC (Manifold-Constrained Hyper-Connections)** | E8 Hexagram residual paths | NT-CORE | Doubly-stochastic mixing matrix → E8 hexagram state transitions. Non-expansive transformations ensure signal stability across 11 ConsciousnessTree branches |

### 3.3 Training & Optimization

| External Innovation | NeoTrix Component | NT Domain | Integration Path |
|---------------------|-------------------|-----------|------------------|
| **DeepSeek Muon Optimizer** | SEAL pipeline optimization | NT-MIND | Matrix-level gradient orthogonalization → SEAL stage weight updates. Faster convergence for skill crystallization |
| **Phi-4 Data-Centric SFT** | experience-tree absorption | NT-MEMORY | "Teachable" prompt curation → experience-tree branch selection. o3-mini traces → consciousness_task quality calibration |
| **Llama 4 Behemoth Distillation** | Dual Specialization (Weapon Set I/II) | NT-CORE | Teacher→student knowledge transfer → Ascendancy weapon set switching. Large model guides small model specialization |
| **Qwen3 Global-Batch Load Balancing** | GWT attention routing | NT-CORE | Expert specialization encouragement → GWT salience ensures balanced activation across NT-* domains |

### 3.4 Context & Memory

| External Innovation | NeoTrix Component | NT Domain | Integration Path |
|---------------------|-------------------|-----------|------------------|
| **DeepSeek 1M context + CSA** | KVMem paged KV | NT-MEMORY | Compressed sparse attention for long-context crawl results. KV cache tiering: GPU→Host→NVMe |
| **Llama 4 Scout 10M context** | experience-tree lazy loading | NT-MEMORY | Hub index at session start, branches loaded on-demand via route table matching |
| **Gemini 1M context + native audio/video** | NT-WORLD multimodal pipeline | NT-WORLD | Video up to 45min processing. Audio: 32 tok/sec. Maps to UnifiedCrawler content extraction |
| **DeepSeek Multi-Token Prediction** | ConsciousnessTree parallel branches | NT-META | MTP = predict multiple future states. CT branches (Soil→Roots→Trunk) can be speculatively evaluated in parallel |

### 3.5 Safety & Alignment

| External Innovation | NeoTrix Component | NT Domain | Integration Path |
|---------------------|-------------------|-----------|------------------|
| **Claude Constitutional AI (CAI)** | NT-SHIELD egress guard | NT-SHIELD | CAI principles → Egress Privacy Guard trust tiers. Self-imposed behavioral constraints during training |
| **Yi-Lightning RAISE** | CleanupCoordinator + RiskAssessor | NT-SHIELD / NT-META | 4-component safety: data filtering (NT-WORLD), post-training (NT-MIND), serving (NT-IO), monitoring (NT-META) |
| **Gemini 2.5 security against prompt injection** | NT-SHIELD sandbox | NT-SHIELD | Egress Policy defense-in-depth: deny-wins, `*.suffix` subdomain matching. Prompt injection → sandbox isolation |
| **Qwen3 119-language safety** | Multilingual KB indexing | NT-MEMORY | Expanded language safety coverage. BBPE tokenizer (151,669 vocab) → KB node normalization |

### 3.6 Inference & Deployment

| External Innovation | NeoTrix Component | NT Domain | Integration Path |
|---------------------|-------------------|-----------|------------------|
| **Mistral EAGLE Speculative Decoding** | Dual Specialization fast-path | NT-CORE | Draft model = Weapon Set II (fast I/O). Verify = Weapon Set I (deep reasoning). Speculative decode → GWT cost-aware routing |
| **Mistral Prefill/Decode Disaggregation** | NT-IO provider routing | NT-IO | Prefill = batch ingestion (NT-WORLD crawl). Decode = real-time serving (NT-IO API). Hardware-aware split |
| **DeepSeek Flash (13B active, FP4)** | Ordered Backend Router | NT-WORLD | Lightweight model for I/O tasks. FP4 routed experts → cheap routing for simple queries (Axiom A1: Cost-Aware Routing) |
| **Grok Colossus (200K H100s)** | NT-ACT parallel task management | NT-ACT | Massive parallelism → ParallelTaskManager GPU batch scheduling. 390 TFLOPs/GPU target |

---

## 4. Key Patterns Identified

### P1: MoE is the Default Architecture for 2025-2026
7/10 models use MoE. DeepSeek V4 pushes to 49B active from 1.6T total. Mistral goes to 128 experts per layer. **NeoTrix implication**: Skill Tree should model expert routing — each domain module = an "expert" with learned activation.

### P2: Unified Thinking/Non-Thinking in Single Model
Gemini 2.5, DeepSeek V4, Qwen3 all unify fast and deep reasoning in one model via thinking budgets. **NeoTrix implication**: ConsciousnessTree's phi/complexity signal should dynamically switch between fast (NT-IO) and deep (NT-CORE) processing.

### P3: Context Windows Exploding (128K → 1M → 10M)
Llama 4 Scout achieves 10M tokens. DeepSeek V4 achieves 1M with compressed attention. **NeoTrix implication**: KVMem paged KV is essential. Axiom A2 (Context as Scarce Resource) validated by industry trend.

### P4: Data-Centric > Compute-Centric
Phi-4-reasoning (14B params) outperforms 5-50× larger models via curated data. **NeoTrix implication**: experience-tree quality > experience-tree quantity. SEAL pipeline should prioritize data curation over parameter scaling.

### P5: Distillation from Larger Teachers is Universal
Llama 4 (Behemoth→Scout/Maverick), Qwen3 (flagship→small), Phi-4-reasoning (o3-mini→Phi-4), Grok 3 (synthetic data). **NeoTrix implication**: Dual Specialization should implement teacher-student knowledge transfer. NT-MIND skill crystallization should distill from larger domain modules.

---

## 5. Priority Absorption Queue

| Priority | Innovation | Source | NeoTrix Target | Complexity |
|----------|-----------|--------|----------------|------------|
| **P0** | Hybrid Attention (CSA/HCA) | DeepSeek V4 | NT-WORLD PerceptionBridge | High |
| **P0** | Thinking Budget Mechanism | Gemini 2.5 / Qwen3 | NT-CORE GWT | Medium |
| **P0** | mHC (Manifold-Constrained Hyper-Connections) | DeepSeek V4 | NT-CORE E8 Hexagram | High |
| **P1** | Granular MoE (128 experts) | Mistral Large 3 | NT-MIND Skill Tree | Medium |
| **P1** | Cross-layer KV Cache Sharing | Yi-Lightning | NT-MEMORY KB cache | Medium |
| **P1** | Muon Optimizer | DeepSeek V4 | NT-MIND SEAL | High |
| **P2** | EAGLE Speculative Decoding | Mistral Large 3 | NT-IO provider routing | Medium |
| **P2** | Hash-MoE Bootstrap | DeepSeek V4 | NT-MIND constellation C0 | Low |
| **P2** | Multi-Token Prediction | DeepSeek V4 | NT-META ConsciousnessTree | Medium |
| **P3** | 10M Context via iRoPE | Llama 4 Scout | NT-MEMORY KVMem | High |
| **P3** | Native Audio/Video Processing | Gemini 2.5 | NT-WORLD multimodal | High |
| **P3** | RAISE Safety Framework | Yi-Lightning | NT-SHIELD | Medium |

---

## 6. Source References

| Model | Source | Date |
|-------|--------|------|
| GPT-4o | arxiv.org/abs/2410.21276 (System Card) | Aug 2024 |
| Claude 3.5 Sonnet | anthropic.com/research/claude-3-5-sonnet | Jun 2024 |
| Gemini 2.5 Pro | arxiv.org/html/2507.06261v1, blog.google | Mar 2025 |
| Llama 4 Scout | ai.meta.com/blog/llama-4-multimodal-intelligence | Apr 2025 |
| DeepSeek V4 | fe-static.deepseek.com/.../deepseek-V4-model-card-EN.pdf | Apr 2026 |
| Qwen3 | arxiv.org/html/2505.09388 | May 2025 |
| Mistral Large 3 | mistral.ai/news/mistral-3 | Dec 2025 |
| Phi-4-reasoning | microsoft.com/.../phi_4_reasoning.pdf | Apr 2025 |
| Yi-Lightning | arxiv.org/pdf/2412.01253v5 | Dec 2024 |
| Grok 3 | x.ai/news/grok-3, deeplearning.ai/the-batch | Feb 2025 |
