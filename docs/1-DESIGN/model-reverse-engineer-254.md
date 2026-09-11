# Model Architecture Reverse Engineering — 254 Batch

> 10 frontier models dissected. Architectural innovations extracted. NeoTrix mappings drawn.
> Date: 2026-09-11 | Status: research-complete

---

## 1. Model Architectures at a Glance

| Model | Params (Total/Active) | Architecture | Context | Key Innovation |
|-------|----------------------|-------------|---------|----------------|
| **GPT-4o** | Undisclosed | Dense transformer, end-to-end multimodal | 128K | Unified audio/text/vision tokenization, 232ms latency |
| **Claude 3.5 Sonnet** | Undisclosed | Dense transformer | 200K | Constitutional AI, ASL safety framework, agentic coding |
| **Gemini 2.5 Pro** | Undisclosed (MoE) | Sparse MoE transformer | 1M+ | Native multimodal (text/audio/video/code), thinking budget |
| **Llama 4 Scout** | 109B total / 17B active | MoE (16 experts) | 10M | iRoPE (interleaved attention w/o positional embeddings), early fusion |
| **DeepSeek V4.1 Flash** | 552B total / 8-16B active | MoE CED (Causal Encoder-Decoder) | 1M | CSA2 sparse attention, engram memory, DSpark speculative decoding |
| **Qwen 3-235B** | 235B total / 22B active | MoE (128 experts, 8 active) | 128K | Thinking/non-thinking mode fusion, thinking budget, global-batch load balance |
| **Mistral Large 3** | 675B total / 41B active | Granular MoE + 2.5B vision encoder | 256K | Eagle speculative decoding, NVFP4 quantization, Apache 2.0 |
| **Phi-4 Reasoning** | 14B (dense) | Dense decoder-only transformer | 32K | Reasoning tokens (`<think>`), GRPO RL, 1.4M curated SFT prompts |
| **Yi-Lightning** | MoE (fine-grained) | Enhanced MoE | 200K+ | Fine-grained expert segmentation, EP load balancing, 82.8% KV cache reduction |
| **Grok 3** | ~1.5T (estimated) | MoE with sparse attention | 1M | 100K H100 training, RL-based reasoning, DeepSearch agent |

---

## 2. Architectural Innovation Taxonomy

### 2.1 Mixture-of-Experts (MoE) — The Dominant Paradigm

**8 of 10 models** use MoE. This is no longer optional; it is the default architecture for frontier models.

| Model | Total/Active Ratio | Experts | Routing | Notable |
|-------|-------------------|---------|---------|---------|
| Llama 4 Scout | 109B/17B (6.4:1) | 16 routed + 1 shared | Top-k routing | Shared expert + routed experts |
| Llama 4 Maverick | 400B/17B (23.5:1) | 128 routed + 1 shared | Top-k routing | Extreme sparsity ratio |
| DeepSeek V4.1 Flash | 552B/8-16B (34-69:1) | 384 routed + 1 shared | 6 active per token | Asymmetric prefill/decode activation |
| Qwen 3-235B | 235B/22B (10.7:1) | 128 total, 8 active | No shared experts | Global-batch load balancing loss |
| Mistral Large 3 | 675B/41B (16.5:1) | Granular MoE | Top-k routing | First Mistral MoE since Mixtral |
| Yi-Lightning | Fine-grained MoE | Fine-grained segmentation | EP group balancing | Partitioned EP load balancing (PEP) |
| Grok 3 | ~1.5T (estimated) | MoE layers | Sparse attention | Largest known training cluster |

**NeoTrix Mapping → NT-CORE GWT Attention Routing:**
- MoE routing = GWT salience routing. Each token activates a subset of specialists, exactly like GWT broadcasting salient signals to selected modules.
- **Axiom A1 (Cost-Aware Routing)**: MoE decouples total capacity from per-token cost. Map to GWT: cheap models for I/O tasks, expensive models for reasoning — same principle, different granularity.
- **Pattern P1 (Model Routing / Delegation)**: The 8-of-10 MoE consensus validates NeoTrix's GWT-based task routing. Route to cheapest capable model = route to cheapest capable expert.
- **Shared experts** (Llama 4, DeepSeek) map to NT-CORE's always-on foundation modules; **routed experts** map to domain-specific specialists.

### 2.2 KV Cache Engineering — The Memory Frontier

The industry's hottest problem: how to process long contexts without exploding memory.

| Model | KV Cache Innovation | Memory Reduction |
|-------|-------------------|-----------------|
| Yi-Lightning | Hybrid attention (3 sliding window + 1 full) + cross-layer KV reuse | 82.8% reduction |
| DeepSeek V4.1 Flash | CSA2 (Compressed Sparse Attention 2) + FP4 KV caching | 890 bytes/token (1/4 of V4-Flash) |
| DeepSeek V4.1 Flash | SWA Bounded Replay (replay recent n_win tokens) | Persistent KV = 1/8 of V4-Flash |
| Llama 4 Scout | iRoPE (interleaved attention w/o positional embeddings) | Enables 10M context |
| Gemini 2.5 Pro | 1M+ token context via architectural improvements | Native long-context |
| Yi-Lightning | Cross-layer KV cache sharing between consecutive full attention layers | 50% full-attention memory |

**NeoTrix Mapping → NT-MEMORY + KVMem paged KV:**
- **Axiom A2 (Context as Scarce Resource)**: Every model is solving the same problem — fitting long contexts into finite GPU memory. This validates NeoTrix's KVMem paged KV approach.
- **Hybrid attention** (Yi-Lightning: 3 sliding + 1 full) maps to NeoTrix's tiered memory: hot (GPU), warm (host), cold (NVMe). Most attention heads focus on local context; only a few need global — same principle as GWT's salience filtering.
- **Cross-layer KV reuse** maps to NT-NEXUS cross-session memory: don't recompute what you've already computed.
- **SWA Bounded Replay** maps to experience-tree lazy branch loading: only reconstruct what you need.

### 2.3 Native Multimodality — End-to-End Fusion

The shift from "bolted-on vision encoder" to "natively multimodal" is complete.

| Model | Modality Integration | Input | Output |
|-------|---------------------|-------|--------|
| GPT-4o | End-to-end joint training | Text + Audio + Image + Video | Text + Audio + Image |
| Gemini 2.5 Pro | Native multimodal (MoE) | Text + Audio + Image + Video | Text |
| Llama 4 Scout | Early fusion (MetaCLIP encoder) | Text + Image | Text + Code |
| DeepSeek V4.1 Flash | DeepSeek-ViT + MLP projector | Text + Image | Text |
| Mistral Large 3 | Integrated 2.5B vision encoder | Text + Image | Text |
| Claude 3.5 Sonnet | Multimodal input | Text + Image | Text |

**NeoTrix Mapping → NT-WORLD PerceptionBridge:**
- **PerceptionBridge** (L2→L5 attention-gated bridge) is the NeoTrix analog. GPT-4o's end-to-end multimodal training = PerceptionBridge's `awareness_score()` filtering sensory events.
- **Early fusion** (Llama 4) maps to NT-WORLD's unified token stream: text, vision, and audio tokens processed by the same transformer.
- **Vision encoder adaptation** (Llama 4: MetaCLIP trained with frozen Llama) maps to NT-WORLD's Crawler adapters: specialized encoders fine-tuned for domain-specific perception.

### 2.4 Reasoning Architecture — Thinking Models

The "thinking model" paradigm is now universal: models that reason step-by-step before answering.

| Model | Reasoning Approach | Mechanism |
|-------|-------------------|-----------|
| Gemini 2.5 Pro | Native thinking with controllable budget | User sets thinking token budget |
| Qwen 3 | Thinking + non-thinking mode fusion | Dynamic mode switching, thinking budget |
| Phi-4 Reasoning | `<think>` tokens + GRPO RL | 1.4M curated SFT prompts, 32K context |
| Grok 3 | RL-based chain-of-thought | Seconds to minutes of reasoning |
| DeepSeek V4.1 Flash | Thinking mode (default on) | Low/high/max effort settings |

**NeoTrix Mapping → NT-CORE E8 + NT-MIND SEAL Pipeline:**
- **Thinking budget** (Gemini 2.5, Qwen 3) maps to GWT's salience-weighted attention allocation: spend more compute on harder problems.
- **Mode fusion** (Qwen 3: thinking ↔ non-thinking) maps to NeoTrix's Dual Specialization: Weapon Set I (acquisition) vs Weapon Set II (evolution), switchable by context.
- **`<think>` tokens** (Phi-4) map to ConsciousnessTree's 6-stage feedback loop: explicit internal reasoning before output.
- **GRPO RL** (Phi-4) maps to NT-MIND's SEAL distillation: self-play reinforcement to improve reasoning traces.

### 2.5 Speculative Decoding — Inference Acceleration

| Model | Speculative Approach | Speedup |
|-------|---------------------|---------|
| DeepSeek V4.1 Flash | DSpark (semi-autoregressive draft, 3-stage, confidence-scheduled) | Drafts 5 tokens at once |
| Mistral Large 3 | Eagle speculative decoding (custom draft model) | 3 speculative tokens |
| Yi-Lightning | Custom MoE operators (1,200 TFLOPS/card FP8) | 100%+ operator improvement |

**NeoTrix Mapping → NT-ACT + NT-PHYSICAL:**
- **Speculative decoding** = NeoTrix's DSpark-like pattern: predict multiple next states, then verify. Maps to NT-ACT's tool-calling: predict tool chain, execute in batch.
- **Eagle draft model** maps to NT-MIND's distillation: small fast model predicts, large model verifies. Teacher-student pattern.

### 2.6 Quantization & Hardware Efficiency

| Model | Precision | Deployment |
|-------|-----------|-----------|
| Mistral Large 3 | NVFP4 (4-bit), FP8 | Single 8×H100 node |
| DeepSeek V4.1 Flash | MXFP4/MXFP8 mixed, BF16 embeddings | 511 GB checkpoint |
| Yi-Lightning | FP8 native, custom MoE operators | Hopper-optimized |
| Llama 4 Scout | BF16 (on-the-fly int4) | Single H100 GPU |

**NeoTrix Mapping → NT-SHIELD + resource management:**
- **Mixed precision** (MXFP4/MXFP8) maps to NeoTrix's ResourceBudgetManager: allocate precision based on task criticality.
- **Single-node deployment** (Mistral, Llama 4) validates NeoTrix's edge-first philosophy: run on consumer hardware when possible.

### 2.7 Safety & Alignment Architecture

| Model | Safety Framework | Mechanism |
|-------|-----------------|-----------|
| Claude 3.5 Sonnet | Constitutional AI + ASL-2 | Principle-based RL, 4x compute threshold |
| GPT-4o | Preparedness Framework | 4 risk categories (cyber/CBRN/persuasion/autonomy) |
| Yi-Lightning | RAISE (4-component) | Pre/during/post safety pipeline |
| Phi-4 Reasoning | Responsible AI SFT | Safety data mixed into reasoning SFT |

**NeoTrix Mapping → NT-SHIELD + NT-GOVERNANCE:**
- **Constitutional AI** maps to NT-GOVERNANCE: principle-level rules enforced at training/inference time.
- **ASL thresholds** map to RiskAssessor's score-based gating (R-P82): ≥60 requires human confirmation, ≥80 auto-rejects.
- **RAISE 4-component** maps to NT-SHIELD's layered defense: stealth net + proxy pool + fingerprint + audit.

---

## 3. Cross-Model Pattern Matrix

| Pattern | GPT-4o | Claude 3.5 | Gemini 2.5 | Llama 4 | DeepSeek V4.1 | Qwen 3 | Mistral L3 | Phi-4R | Yi-Light | Grok 3 |
|---------|--------|-----------|-----------|---------|--------------|--------|-----------|--------|----------|--------|
| MoE | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ |
| Native multimodal | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — |
| Thinking mode | — | — | ✓ | — | ✓ | ✓ | — | ✓ | — | ✓ |
| Long context (>256K) | — | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | ✓ | ✓ |
| Speculative decoding | — | — | — | — | ✓ | — | ✓ | — | — | — |
| Open weights | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | — | — |
| RL post-training | — | ✓ | — | — | — | ✓ | — | ✓ | ✓ | ✓ |

**Convergence points:**
1. MoE is universal (8/10)
2. Native multimodality is table stakes (6/10)
3. Thinking/reasoning modes are the new differentiator (5/10)
4. Open weights winning (5/10)

---

## 4. NeoTrix Absorption Opportunities

### 4.1 Immediate吸收 (R-P79: 同session接线)

| Innovation | Source | NeoTrix Target | Action |
|-----------|--------|---------------|--------|
| MoE routing → GWT salience | All MoE models | nt_core GWT | Formalize salience-weighted expert routing as GWT sub-module |
| Hybrid attention (3 sliding + 1 full) | Yi-Lightning | nt_memory KV cache | Implement tiered attention: local sliding + sparse global |
| Thinking budget control | Gemini 2.5, Qwen 3 | nt_core consciousness_tick | Add compute-budget parameter to growth cycle |
| CSA2 compressed sparse attention | DeepSeek V4.1 | nt_memory | Implement 3-mode attention (Full/Reindex/Reuse) |
| Asymmetric prefill/decode activation | DeepSeek V4.1 | nt_core推理 | Different compute for input parsing vs output generation |
| GRPO RL for reasoning | Phi-4 | nt_mind SEAL | Rule-based reward model for self-play |

### 4.2 Architecture-level吸收

| Pattern | Evidence | NeoTrix Design Implication |
|---------|----------|--------------------------|
| **Decouple capacity from cost** | MoE (8/10 models) | GWT must support heterogeneous model routing — not just "which specialist" but "which cost tier" |
| **Context is the bottleneck** | KV cache engineering (6/10) | KVMem paged KV is correct direction; add compressed attention (CSA2-style) |
| **Reasoning is a meta-skill** | Phi-4 SFT generalizes to unseen tasks | SEAL pipeline should separate "reasoning skill" from "domain knowledge" |
| **Safety is architectural, not bolt-on** | Claude ASL, RAISE, Preparedness | NT-SHIELD must be wired into training loop, not just inference guard |
| **Open weights + efficient quantization** | Llama, Qwen, Mistral, DeepSeek | NeoTrix should support NVFP4/MXFP8 inference for edge deployment |

---

## 5. Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| MoE complexity vs. inference simplicity | DeepSeek V4.1's asymmetric activation: cheap prefill, expensive decode. Map to NeoTrix: cheap perception, expensive reasoning. |
| Long context vs. memory cost | Yi-Lightning's 82.8% KV reduction proves hybrid attention works. Adopt for NeoTrix's >256K sessions. |
| Thinking mode latency vs. accuracy | Qwen 3's budget control: let user choose. Map to GWT: adaptive compute allocation based on task difficulty. |
| Open weights vs. safety | Claude's Constitutional AI is not open, but Phi-4's safety SFT is. NeoTrix should use open safety methods. |
| Speculative decoding vs. model size | Mistral Eagle + NVFP4: small draft model + quantized main model. Map to NT-MIND distillation. |

---

## 6. Key Takeaways

1. **MoE is the architecture.** If you're not using MoE, you're paying too much per token.
2. **KV cache is the bottleneck.** Every frontier model is engineering around it — hybrid attention, compression, cross-layer reuse.
3. **Thinking models are the new standard.** Controllable reasoning budgets (Gemini 2.5, Qwen 3) will replace static models.
4. **Native multimodality wins.** Bolted-on vision encoders are dying. End-to-end training is the way.
5. **Open weights + quantization = edge deployment.** Llama 4 Scout fits on a single H100. Mistral Large 3 runs on 8×H100 with NVFP4.
6. **Safety must be architectural.** Constitutional AI, ASL thresholds, RAISE — all are structural, not patch-on.
7. **Speculative decoding is the inference accelerant.** DSpark, Eagle — predict multiple tokens, verify in batch.
8. **The 14B parameter model (Phi-4) outperforms 70B distilled models.** Data curation > parameter count.

---

## Sources

- OpenAI GPT-4o System Card (arXiv:2410.21276)
- Anthropic Claude 3 Model Card + 3.5 Sonnet Addendum
- Google DeepMind Gemini 2.5 Technical Report (arXiv:2507.06261)
- Meta Llama 4 Model Card (github.com/meta-llama)
- DeepSeek V4.1-Flash README (HuggingFace)
- Qwen3 Technical Report (arXiv:2505.09388)
- Mistral Large 3 Technical Documentation + HuggingFace README
- Microsoft Phi-4-reasoning Technical Report (arXiv:2504.21318)
- 01.AI Yi-Lightning Technical Report (arXiv:2412.01253)
- xAI Grok 3 Blog + DeepWiki analysis
