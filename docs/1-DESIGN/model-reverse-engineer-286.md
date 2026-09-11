# Model Reverse Engineering #286

**Date**: 2026-09-11  
**Purpose**: Reverse-engineer 10 frontier LLM architectures → extract innovations → map to NeoTrix  
**Models**: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3

---

## 1. Architecture Matrix

| Model | Architecture | Total Params | Active Params | Context | Key Innovation |
|-------|-------------|-------------|--------------|---------|---------------|
| GPT-4o | Dense (undisclosed) | ~200B est. | ~200B | 128K | Omni-modal end-to-end training |
| Claude 3.5 Sonnet | Dense | Undisclosed | Undisclosed | 200K | Constitutional AI, tool-use agentic |
| Gemini 2.5 Pro | MoE | Undisclosed | Undisclosed | 2.1M | Native multimodal, thinking budget |
| Llama 4 Scout | MoE | 109B | 17B (16E) | 192K | Early fusion, iRoPE 10M context |
| DeepSeek V4 Flash | MoE | 284B | 13B (256E) | 1M | CSA+HCA hybrid attention, mHC |
| Qwen 3 | MoE/Dense | 235B (MoE) | 22B (128E) | 128K | Hybrid thinking/non-thinking |
| Mistral Large 3 | Granular MoE | 675B | 41B (128E) | 256K | Multi-Latent Attention, top-4 routing |
| Phi-4 Reasoning | Dense | 14B | 14B | 128K | Synthetic data distillation, pivotal token DPO |
| Yi-Lightning | MoE | Undisclosed | Undisclosed | 128K | Fine-grained expert segmentation, KV sharing |
| Grok 3 | Hybrid Dense/MoE | ~2.7T est. | Undisclosed | 128K | 100K H100 cluster, DeepSearch |

---

## 2. Per-Model Architecture Deep Dive

### 2.1 GPT-4o — Omni-Modal End-to-End

**Core Architecture**: Undisclosed dense transformer. ~200B parameters estimated.  
**Modality**: Text, audio, image — trained end-to-end in a single model (replacing the 3-model pipeline).  
**Latency**: 232ms average response (vs 2.8s GPT-3.5, 5.4s GPT-4).  
**Key Innovation**:  
- **Unified Omni-Modal Network**: Single neural network processes all input/output modalities. Previous Voice Mode was pipeline (ASR → LLM → TTS); GPT-4o eliminates information loss from modality boundaries.  
- **2x faster, 50% cheaper** than GPT-4 Turbo with matched text/code quality.  
- **Real-time emotion/prosody** understanding and generation.

**NeoTrix Mapping**:  
- **NT-IO**: GPT-4o's omni-modal approach validates NeoTrix's **Egress Privacy Guard** design — all modalities flow through a single trust boundary, not per-modality filters.  
- **PerceptionBridge**: GPT-4o's unified modality confirms NeoTrix's L2→L5 attention-gated perception bridge is the correct architectural pattern (sensory integration before consciousness routing).  
- **A1 (Cost-Aware Routing)**: GPT-4o's 50% cost reduction validates cheap-for-simple-tasks routing.

---

### 2.2 Claude 3.5 Sonnet — Dense + Constitutional AI

**Core Architecture**: Dense transformer. 200K context window.  
**Key Innovation**:  
- **Constitutional AI (CAI)**: Self-supervised alignment via a set of principles, reducing human feedback dependency.  
- **Agentic Tool Use**: SWE-bench Verified 49% (state-of-the-art at launch). Can independently write, edit, execute code.  
- **Artifacts**: Real-time code/content generation visible to users.  
- **Speed**: 2x Claude 3 Opus speed, matching cost of Claude 3 Sonnet.

**NeoTrix Mapping**:  
- **NT-GOVERNANCE**: Constitutional AI maps to **gov-steward** policy enforcement — principle-based guardrails are the same pattern as NeoTrix's governing constitution.  
- **NT-ACT**: Claude's agentic coding (SWE-bench 49%) validates NeoTrix's **dev-implementer** skill as a production pattern.  
- **NT-SHIELD**: CAI's safety layer parallels **rev-officer** review dimensions.

---

### 2.3 Gemini 2.5 Pro — MoE + Native Thinking

**Core Architecture**: Mixture-of-Experts. 1M+ context window (2.1M).  
**Key Innovation**:  
- **Native Thinking Model**: Purpose-built reasoning with configurable thinking budget. Transparent chain-of-thought.  
- **Massive Context**: 1M tokens input, 65K output. Handles entire codebases + video.  
- **Native Multimodality**: Text, audio, video, PDF — processed in unified architecture.  
- **Performance**: GPQA 84.0%, AIME 2024 92.0%, Humanity's Last Exam 18.8%.

**NeoTrix Mapping**:  
- **GWT Attention Routing**: Gemini's thinking budget = NeoTrix's **GWT salience routing** — allocate compute proportional to task difficulty.  
- **NT-MEMORY**: 2.1M context validates NeoTrix's **KVMem paged KV** approach for long sessions (A2: Context as Scarce Resource).  
- **SEAL Pipeline**: Thinking/non-thinking modes map to NeoTrix's **Dual Specialization** (Weapon Set I/II routing).

---

### 2.4 Llama 4 Scout — MoE + Early Fusion

**Core Architecture**: MoE — 109B total, 17B active, 16 experts.  
**Key Innovation**:  
- **Early Fusion Multimodality**: Text and vision tokens fused early in the pipeline (not bolted on post-hoc). Enables training as a single unified system.  
- **iRoPE (Interpolated Rotary Position Embeddings)**: Novel position encoding enabling 10M token context window.  
- **Efficient Deployment**: Fits single H100 GPU with int4 quantization.  
- **40T token pretraining** on public + licensed data + Meta product data.

**NeoTrix Mapping**:  
- **NT-WORLD**: Early fusion validates NeoTrix's **SensoryIntegrationHub** — perceive then route, not modality-specific pipelines.  
- **R-P42 (Absorption)**: Llama 4's 40T token scale shows NeoTrix's knowledge absorption must handle massive ingestion with quality filtering.  
- **A2 (Context as Scarce Resource)**: iRoPE 10M context → NeoTrix's KV cache optimization for persistent agents.

---

### 2.5 DeepSeek V4 Flash — Hybrid Attention + Manifold Geometry

**Core Architecture**: MoE — 284B total, 13B active, 256 routed experts.  
**Key Innovation**:  
- **CSA + HCA Hybrid Attention**: Compressed Sparse Attention (low-compression, overlapping windows + Lightning Indexer) interleaved with Heavily Compressed Attention (high-compression, non-overlapping). Reduces KV cache to 10% of V3 at 1M tokens.  
- **mHC (Manifold-Constrained Hyper-Connections)**: Replaces residual connections. Projects onto doubly-stochastic matrix manifold (Birkhoff polytope) via Sinkhorn-Knopp iterations. Non-expansive signal propagation across deep stacks.  
- **Muon Optimizer**: Faster convergence, better training stability.  
- **Multi-Token Prediction (MTP)**: Retained from V3 for speculative decoding.  
- **Three reasoning modes**: Non-think / Think High / Think Max.

**NeoTrix Mapping**:  
- **HeartbeatAggregator**: mHC's manifold-constrained signal propagation = NeoTrix's health signal aggregation — structured, bounded, non-divergent.  
- **GWT Salience**: CSA/HCA hybrid = attention routing by information density — sparse for local, compressed for global. Direct GWT implementation pattern.  
- **KVMem**: DeepSeek's 10% KV cache at 1M tokens is the empirical proof for NeoTrix's paged KV virtualization strategy.  
- **A2 (Context as Scarce Resource)**: The most efficient context management in the industry — NeoTrix should study this for kv_cache_optimizer.rs.  
- **SEAL Phase-0 (Convergence Check)**: mHC's non-expansive property ensures stable signal propagation → maps to self-test stability verification.

---

### 2.6 Qwen 3 — Hybrid Thinking Mode

**Core Architecture**: MoE (235B total, 22B active, 128 experts) + Dense variants (0.6B-32B).  
**Key Innovation**:  
- **Hybrid Thinking/Non-Thinking Mode**: Seamless switching between deep reasoning and fast response. API/prompt tag control. Think budget management.  
- **Strong-to-Weak Distillation**: Small models (Qwen3-4B) match Qwen2.5-72B via distillation from flagship.  
- **119 Languages, 36T tokens**: Massive multilingual pretraining.  
- **Three-stage pretraining**: General (30T) → Reasoning (5T) → Long Context.  
- **Global-batch load balancing**: No shared experts, fine-grained expert segmentation.  
- **QK-Norm**: Replaced QKV-bias for stable training.

**NeoTrix Mapping**:  
- **Dual Specialization**: Qwen3's thinking/non-thinking switching = NeoTrix's **Weapon Set I/II** routing. AttentionManager routes by task type.  
- **SKILL-SPEC.md**: Qwen3's Strong-to-Weak Distillation validates NeoTrix's **skill crystallization** — distill expert knowledge into small, deployable nodes.  
- **A1 (Cost-Aware Routing)**: Think/Non-think switching is cost-aware routing at the model level. NeoTrix should adopt this at the agent routing level.  
- **Experience-tree**: 36T token pretraining with instance-level data optimization = NeoTrix's experience absorption pipeline quality control.

---

### 2.7 Mistral Large 3 — Granular MoE + Multi-Latent Attention

**Core Architecture**: Granular MoE — 675B total, 41B active, 128 experts.  
**Key Innovation**:  
- **Granular MoE with Top-4 Routing**: Fewer but larger experts, softmax-based routing (vs DeepSeek's no-aux-loss).  
- **Multi-Latent Attention (MLA)**: Compresses KV cache across multiple latent dimensions.  
- **256K Context Window**: Production-grade long-context for enterprise.  
- **Apache 2.0 Open Weights**: Fully open, trained from scratch on 3000 H200 GPUs.  
- **Vision Encoder (2.5B)**: Native multimodal with dedicated vision component.  
- **Speculative Decoding**: Eagle draft model for faster inference.

**NeoTrix Mapping**:  
- **Rune Socketing**: Mistral's granular expert selection = NeoTrix's **5-rune socketing** — specialized slots for specialized work.  
- **CapabilityRegistry**: Top-4 expert routing = capability routing in NeoTrix's runtime registry.  
- **Ordered Backend Router (R-P82)**: Mistral's production-grade reliability validates ordered fallback chain design.  
- **PlatformGateway**: Mistral's multi-platform deployment (Azure, Bedrock, NIM) = NeoTrix's PlatformGateway pattern.

---

### 2.8 Phi-4 Reasoning — Synthetic Data Distillation

**Core Architecture**: Dense decoder-only transformer — 14B parameters.  
**Key Innovation**:  
- **Synthetic Data as Primary Training Signal**: Trained on 1.4M STEM/coding prompts with o3-mini-generated reasoning chains. Surpasses its own teacher (GPT-4) on STEM QA.  
- **Pivotal Token Search for DPO**: New technique for creating DPO preference pairs by identifying tokens that most influence outcome quality.  
- **Reasoning/Non-Reasoning Toggle**: Phi-4-reasoning (SFT) vs Phi-4-reasoning-plus (SFT + RL). Different accuracy/token tradeoffs.  
- **15B Vision Variant**: Mid-fusion architecture with SigLIP-2 encoder + cross-modality projector.  
- **14B params competitive with 5-50x larger models** on reasoning tasks.

**NeoTrix Mapping**:  
- **SKILL-SPEC.md**: Phi-4's distillation pattern (o3-mini → 14B student) = NeoTrix's **skill crystallization pipeline**. Small, focused models from expert teachers.  
- **Experience-tree**: Pivotal token search = experience quality scoring. Identify high-impact learning moments.  
- **R-P79 (Absorption)**: Phi-4 proves you can surpass the teacher with proper data curation → NeoTrix's external absorption protocol.  
- **NT-MIND**: Phi-4's reasoning toggle = NeoTrix's **SEAL pipeline** modes — explore vs. exploit.  
- **A1 (Cost-Aware Routing)**: 14B model competing with 70B+ = cost-efficient routing validation.

---

### 2.9 Yi-Lightning — Fine-Grained Expert Segmentation

**Core Architecture**: Enhanced MoE with fine-grained expert segmentation.  
**Key Innovation**:  
- **Fine-Grained Expert Segmentation**: More granular expert splitting than standard MoE.  
- **Balanced Expert Routing**: Prevents expert collapse (overuse of few experts).  
- **Cross-Layer KV Cache Sharing**: KV cache shared across layers, reducing memory footprint.  
- **Hardware-Aware Design**: Architecture optimized for FP8 quantization compatibility.  
- **RAISE Safety Framework**: Four-component safety engine across full lifecycle.  
- **Chatbot Arena #6 overall**: Strong in Chinese, Math, Coding, Hard Prompts.

**NeoTrix Mapping**:  
- **KVMem**: Cross-layer KV sharing = NeoTrix's **paged KV virtualization** — share cache across attention heads/layers.  
- **NT-SHIELD**: RAISE framework = NeoTrix's **stealth net + audit** lifecycle security.  
- **HeartbeatAggregator**: Balanced expert routing = health signal distribution — prevent signal concentration in one module.  
- **Rune Socketing**: Fine-grained segmentation = granular rune allocation — specialized slots prevent resource hogging.

---

### 2.10 Grok 3 — Scale + Real-Time Knowledge

**Core Architecture**: Hybrid Dense/MoE — estimated 2.7T total parameters.  
**Key Innovation**:  
- **Colossus Supercomputer**: 100K H100 GPUs, 200M GPU hours, ~$420M training cost. Largest training cluster at launch.  
- **DeepSearch Mode**: Real-time internet search + deep probing for current information.  
- **Think Mode**: Chain-of-thought reasoning with self-correction.  
- **MMLU 89.7%**, AIME 82%, LiveCodeBench 79.4%.  
- **Real-time X Platform Integration**: Live social media context.

**NeoTrix Mapping**:  
- **NT-WORLD**: DeepSearch = NeoTrix's **Ordered Backend Router** — real-time knowledge acquisition with fallback chains.  
- **A2 (Context as Scarce Resource)**: Grok 3's 128K context vs Gemini's 2.1M shows context management is differentiating. NeoTrix needs paged KV.  
- **HeartbeatAggregator**: Grok's training scale (200M GPU hours) vs Phi-4's efficiency (14B beating 70B) shows the spectrum NeoTrix must bridge.  
- **NT-MEMORY**: Real-time knowledge integration = NeoTrix's **KB live ingestion** pipeline.

---

## 3. Cross-Model Innovation Taxonomy

### 3.1 Universal Innovations (All 10 Models)

| Innovation | Models | NeoTrix Module |
|-----------|--------|---------------|
| **MoE Architecture** | 8/10 (all except GPT-4o, Claude 3.5) | CapabilityRegistry (runtime expert routing) |
| **Multimodal Input** | 10/10 | SensoryIntegrationHub + PerceptionBridge |
| **Long Context (>128K)** | 9/10 | KVMem (paged KV virtualization) |
| **Reasoning Modes** | 7/10 | GWT salience routing + Dual Specialization |
| **Synthetic Data** | 8/10 | Experience-tree + SKILL crystallization |

### 3.2 Breakthrough Innovations (Unique to 1-2 Models)

| Innovation | Model(s) | NeoTrix Equivalent |
|-----------|---------|-------------------|
| **mHC Manifold Connections** | DeepSeek V4 | HeartbeatAggregator (bounded signal propagation) |
| **CSA+HCA Hybrid Attention** | DeepSeek V4 | GWT (sparse local + compressed global) |
| **iRoPE 10M Context** | Llama 4 Scout | KVMem extension target |
| **Pivotal Token DPO** | Phi-4 | Experience quality scoring |
| **Early Fusion** | Llama 4 Scout | SensoryIntegrationHub (pre-consciousness integration) |
| **Multi-Latent Attention** | Mistral Large 3 | Rune Socketing (multi-dimensional compression) |
| **Omni-Modal E2E** | GPT-4o | Egress Privacy Guard (single trust boundary) |
| **Cross-Layer KV Sharing** | Yi-Lightning | KVMem cache optimization |

---

## 4. Derived Axioms for NeoTrix

### A4: Hybrid Attention is the Future
- **Evidence**: DeepSeek V4 (CSA+HCA), Mistral (MLA), Qwen 3 (hybrid thinking)
- **Implication**: NeoTrix GWT should route attention differently for local vs. global information. Implement attention-type routing at the perception layer.

### A5: Small Models Can Beat Large Ones with Better Data
- **Evidence**: Phi-4 (14B beating 70B+), Qwen3-4B matching Qwen2.5-72B
- **Implication**: NeoTrix's skill crystallization should focus on data quality over model size. Strong-to-Weak Distillation is a first-class pattern.

### A6: Context Management is the New Frontier
- **Evidence**: DeepSeek V4 (10% KV at 1M), Llama 4 Scout (iRoPE 10M), Gemini (2.1M)
- **Implication**: NeoTrix's KVMem paged KV virtualization is the right bet. Extend with manifold-constrained signal propagation for stability.

### A7: Thinking/Non-Thinking Toggle is a Universal Pattern
- **Evidence**: DeepSeek V4 (3 modes), Qwen 3 (2 modes), Phi-4 (2 modes), Gemini (thinking budget)
- **Implication**: NeoTrix's Dual Specialization (Weapon Set I/II) should support dynamic switching with cost-aware routing.

### A8: Synthetic Data Surpasses Organic Data for Reasoning
- **Evidence**: Phi-4 (synthetic > organic), Qwen 3 (synthetic reasoning data), Yi-Lightning (synthetic data engineering)
- **Implication**: NeoTrix's experience-tree should prioritize synthetic experience generation for reasoning capabilities, not just organic learning.

---

## 5. NeoTrix Implementation Roadmap

### Phase 1: Immediate (This Sprint)
1. **Implement GWT Hybrid Attention** — route local vs. global information differently (from DeepSeek V4 CSA+HCA)
2. **Extend KVMem with cross-layer sharing** — from Yi-Lightning's cross-layer KV cache design
3. **Add thinking/non-thinking toggle to Dual Specialization** — from Qwen 3 / DeepSeek V4

### Phase 2: Next Sprint
4. **Implement mHC-style bounded signal propagation** — for HeartbeatAggregator stability
5. **Add pivotal token experience scoring** — from Phi-4's DPO innovation
6. **Early fusion for PerceptionBridge** — from Llama 4 Scout's approach

### Phase 3: Next Quarter
7. **iRoPE position encoding** — extend KVMem to 10M context
8. **Strong-to-Weak Distillation** — for skill crystallization pipeline
9. **Omni-modal egress guard** — single trust boundary for all modalities

---

## 6. Key Sources

| Source | Date | Key Data |
|--------|------|----------|
| OpenAI GPT-4o Announcement | 2024-05 | Omni-modal E2E, 232ms latency |
| Anthropic Claude 3.5 Sonnet | 2024-06 | CAI, SWE-bench 49% |
| Google Gemini 2.5 Pro | 2025-06 | MoE, 2.1M context, thinking |
| Meta Llama 4 Scout Card | 2025-04 | 109B/17B active, 16E, iRoPE, early fusion |
| DeepSeek V4 Technical Report | 2026-04 | CSA+HCA, mHC, Muon, 284B/13B |
| Qwen3 Technical Report (arXiv:2505.09388) | 2025-05 | Hybrid thinking, 36T tokens, 119 langs |
| Mistral Large 3 Docs | 2025-12 | 675B/41B, granular MoE, MLA |
| Phi-4 Reasoning Report (arXiv:2504.21318) | 2025-04 | Synthetic data, pivotal token DPO |
| Yi-Lightning Report (arXiv:2412.01253) | 2024-12 | Fine-grained experts, KV sharing, RAISE |
| Grok 3 / Colossus | 2025-02 | 100K H100, DeepSearch, hybrid MoE |

---

## 7. Cross-Source Pattern Consolidation

### Pattern P6: Hybrid Attention Routing
- **Definition**: Alternate between sparse local attention and compressed global attention within the same model.
- **Sources**: DeepSeek V4 (CSA+HCA), Mistral (MLA), Gemini (thinking budget)
- **NeoTrix Mapping**: GWT attention routing should implement attention-type dispatch.

### Pattern P7: Manifold-Constrained Signal Propagation
- **Definition**: Project residual connections onto a constrained manifold (Birkhoff polytope) to ensure non-expansive signal flow.
- **Sources**: DeepSeek V4 (mHC)
- **NeoTrix Mapping**: HeartbeatAggregator health signals should be manifold-constrained.

### Pattern P8: Early Fusion Multimodality
- **Definition**: Fuse modalities early in the processing pipeline, before consciousness routing.
- **Sources**: Llama 4 Scout (early fusion), GPT-4o (omni-modal E2E)
- **NeoTrix Mapping**: SensoryIntegrationHub should fuse before PerceptionBridge.

### Pattern P9: Synthetic Experience as Primary Learning
- **Definition**: Generate high-quality synthetic experiences using teacher models, surpassing organic data for reasoning tasks.
- **Sources**: Phi-4 (pivotal token DPO), Qwen 3 (synthetic reasoning), Yi-Lightning
- **NeoTrix Mapping**: Experience-tree should include synthetic experience generation stage.

### Pattern P10: Cost-Quality Pareto via Routing
- **Definition**: Achieve frontier quality at commodity cost through intelligent routing between thinking/non-thinking modes.
- **Sources**: Qwen 3 (2 modes), DeepSeek V4 (3 modes), Phi-4 (2 variants)
- **NeoTrix Mapping**: A1 (Cost-Aware Routing) should implement dynamic think/non-think switching.
