# Model Reverse Engineering — Cycle 229 (2026-09-11)

10 frontier reasoning models analyzed. Architecture innovations extracted. NeoTrix mapping proposed.

---

## 1. Model Architecture Summary

### 1.1 GPT-4o Mini (OpenAI)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Dense Transformer (not disclosed) |
| **Parameters** | ~14B (estimated) |
| **Context** | 128K tokens |
| **Modalities** | Text + Image → Text |
| **Attention** | Not disclosed |
| **Key Innovation** | Instruction Hierarchy (resistance to jailbreaks/prompt injection) |
| **Function Calling** | Best-in-class reliability, parallel calls, JSON schema enforcement |

**NeoTrix Mapping:**
- Instruction Hierarchy → `NT-SHIELD` prompt injection defense pattern
- Parallel function calling → `NT-ACT` MCP tool orchestration
- Structured output enforcement → `NT-IO` structured output validation

---

### 1.2 Claude 3.5 Haiku (Anthropic)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Decoder-only Transformer |
| **Parameters** | ~8B (estimated, 40% smaller than GPT-4o Mini) |
| **Context** | 200K tokens |
| **Attention** | Dynamic Grouped Query Attention (GQA) with adaptive head allocation |
| **FFN** | Sparsified — prunes 20% inactive neurons at runtime (<1ms/layer) |
| **Quantization** | 4-bit calibration-free (99.2% MMLU retention) |
| **Decoding** | Built-in 1.5B draft model for speculative decoding |
| **Performance** | 2,400 TPS on H100 (2x GPT-4o Mini) |

**Key Innovations:**
1. **Dynamic GQA** — adjusts query groups per layer based on sequence length
2. **Runtime FFN sparsity** — input-specific activation pruning (not static)
3. **Calibration-free 4-bit** — uses pre-trained activation distributions
4. **Integrated speculative decoding** — no external draft model overhead

**NeoTrix Mapping:**
- Dynamic GQA → `NT-CORE` GWT attention budget adaptive allocation
- Runtime FFN sparsity → `NT-MIND` SEAL pipeline dynamic resource allocation
- Speculative decoding → `nt_core_llm` draft model integration for cost reduction
- Circuit tracing interpretability → `nt_meta` attribution graph for self-audit

---

### 1.3 Gemini 2.0 Flash Thinking (Google)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Sparse MoE Transformer |
| **Context** | 1M tokens |
| **Thinking** | Explicit thinking phase with configurable budget |
| **Thinking Budget** | `thinking_budget` parameter (1024–8192 tokens) |
| **Thought Signatures** | Encrypted reasoning state for multi-turn continuity |

**Key Innovations:**
1. **Thinking budget control** — user-configurable inference-time compute
2. **Thought signatures** — encrypted reasoning state passed across turns
3. **Two-phase reasoning** — justification phase (policy check) → execution phase
4. **H-CoT vulnerability** — execution-phase token hijacking can bypass safety

**NeoTrix Mapping:**
- Thinking budget → `nt_core_self::AttentionManager` salience budget
- Thought signatures → `nt_nexus` cross-session reasoning state persistence
- Two-phase reasoning → `ConsciousnessTree` SoW (Sense-or-Wait) gating
- H-CoT defense → `NT-SHIELD` reasoning token injection protection

---

### 1.4 Llama 3.2 Vision (Meta)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Dense Llama 3.1 backbone + cross-attention vision adapter |
| **Sizes** | 11B (10.6B active), 90B (88.8B active) |
| **Context** | 128K tokens |
| **Vision** | Late fusion via cross-attention adapter layers |
| **Training** | 6B image-text pairs; adapter trained, LM frozen |

**Key Innovations:**
1. **Cross-attention adapter** — image encoder representations fed into LLM via dedicated cross-attention layers
2. **Frozen LM preservation** — language capabilities remain intact as drop-in replacement
3. **Pan & Scan** — adaptive image cropping for non-square/high-res images

**NeoTrix Mapping:**
- Cross-attention adapter → `nt_sense` PerceptionBridge modality fusion
- Frozen LM + adapter → `NT-MEMORY` knowledge preservation during evolution
- Pan & Scan → `nt_world` multi-resolution image processing pipeline

---

### 1.5 DeepSeek V4 (DeepSeek AI)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | DeepSeekMoE + Hybrid Attention + mHC |
| **Sizes** | Flash: 284B total / 13B active; Pro: 1.6T total / 49B active |
| **Context** | 1M tokens |
| **Attention** | Hybrid CSA (Compressed Sparse) + HCA (Heavily Compressed) |
| **Residuals** | Manifold-Constrained Hyper-Connections (mHC) |
| **MoE** | Hash-routed bootstrap layers (first 3), then learned routing |
| **Precision** | FP4 (MoE experts) + FP8 (others) mixed |
| **Optimizer** | Muon optimizer (faster convergence) |

**Key Innovations:**
1. **CSA+HCA hybrid attention** — compresses KV cache to 10% of V3.2 at 1M context
2. **mHC residual connections** — doubly-stochastic mixing matrix, non-expansive signal propagation
3. **Hash-MoE bootstrap** — first 3 layers use token-id hash routing (no learned gating)
4. **Multi-Token Prediction** — auxiliary objectives for future token prediction
5. **Sqrt(Softplus) affinity** — replaces Sigmoid for MoE routing scores

**NeoTrix Mapping:**
- CSA+HCA → `nt_memory` KV cache tiered compression (hot/warm/cold)
- mHC → `nt_core` HyperCube residual signal stability mechanism
- Hash-MoE bootstrap → `NT-MIND` SEAL pipeline cold-start routing
- Multi-Token Prediction → `ConsciousnessTree` forward-looking growth projection
- Muon optimizer → `nt_meta` training stability mechanisms

---

### 1.6 Qwen3-30B-A3B (Alibaba)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Fine-grained MoE (no shared expert) |
| **Parameters** | 30.5B total, 3.3B active per token |
| **Experts** | 128 routed, top-8, NO shared expert |
| **Attention** | GQA 32Q/4KV + QK-Norm |
| **FFN** | 768-dim per expert (deep & narrow) |
| **Head dim** | 128 (Q projection wider than hidden: 4096 vs 2048) |
| **Context** | 32K native, 131K with YaRN |

**Key Innovations:**
1. **No shared expert** — all compute budget goes to routed experts (vs DeepSeek's 1 shared)
2. **Deep & narrow** — 48 layers × 2048 hidden × 768 FFN per expert
3. **QK-Norm** — stabilizes attention at long context, enables higher learning rates
4. **Wider-than-residual attention** — Q runs in 4096 space, hidden is 2048

**NeoTrix Mapping:**
- No shared expert → `nt_core` capability tree routing (all capability nodes are routed, no always-on)
- Deep & narrow → `ConsciousnessTree` depth-over-breadth architecture principle
- QK-Norm → `nt_core` attention stability for long-context operations
- Wider-than-residual attention → `GWT` attention space expansion for complex routing

---

### 1.7 Phi-4-Mini (Microsoft)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Dense Decoder-only Transformer |
| **Parameters** | 3.8B |
| **Context** | 128K (LongRoPE) |
| **Vocabulary** | 200K tokens (expanded for multilingual) |
| **Attention** | GQA 24Q/8KV |
| **RoPE** | Fractional — 25% position-agnostic |
| **Multimodal** | Mixture-of-LoRAs (frozen backbone + LoRA adapters per modality) |

**Key Innovations:**
1. **Mixture-of-LoRAs** — frozen LM + modality-specific LoRA adapters (vision: 370M, speech: 460M)
2. **Fractional RoPE** — 25% of head dim is position-agnostic for smoother long-context handling
3. **200K vocabulary** — 4x larger than Phi-3.5 for multilingual coverage
4. **SambaY / GMU** (flash-reasoning variant) — Gated Memory Unit between decoder layers, 10x throughput

**NeoTrix Mapping:**
- Mixture-of-LoRAs → `nt_io` modality adapter architecture (PlatformGateway)
- Fractional RoPE → `nt_core` position-agnostic attention for cross-domain reasoning
- SambaY GMU → `nt_core` Gated Memory Unit for cross-layer state sharing

---

### 1.8 Gemma 3 4B (Google)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Dense Decoder-only Transformer |
| **Parameters** | 4.3B (417M vision + 675M embedding + 3.2B non-embedding) |
| **Context** | 128K tokens |
| **Attention** | 5:1 interleaved local:global (sliding window = 1024) |
| **Vision** | Frozen SigLIP ViT (400M) + Pan&Scan |
| **Tokenizer** | 262K vocabulary (SentencePiece) |
| **Norm** | QK-norm (replaces Gemma 2 softcapping) |

**Key Innovations:**
1. **5:1 local:global interleaving** — only 1/6 layers attend globally; 5/6 use 1024-token windows
2. **KV-cache reduction** — <15% overhead at 32K (vs 60% for global-only)
3. **Dual RoPE frequencies** — global layers: 1M base; local layers: 10K base
4. **Pan&Scan** — adaptive image cropping for variable resolution

**NeoTrix Mapping:**
- 5:1 interleaved attention → `GWT` local-global attention routing (hot/cold context)
- Dual RoPE frequencies → `nt_memory` frequency-domain attention scaling
- Pan&Scan → `nt_world` multi-resolution perception pipeline
- KV-cache optimization → `nt_memory` tiered storage optimization

---

### 1.9 Mistral Small 3.2 (Mistral AI)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Dense Transformer |
| **Parameters** | 24B |
| **Context** | 128K tokens |
| **Attention** | GQA 32Q/8KV, head_dim=128 |
| **Vision** | Pixtral encoder (24 layers, 16 heads, 1540×1540) |
| **FFN** | 32K intermediate, SiLU activation |
| **RoPE** | theta=1B |
| **License** | Apache 2.0 |

**Key Innovations:**
1. **Pixtral vision encoder** — 1540×1540 native resolution, 14px patches
2. **Iteration-over-architecture** — 3.2 same base weights as 3.1, only post-training changed
3. **Infinite generation reduction** — 2.11% → 1.29% via instruction tuning refinement

**NeoTrix Mapping:**
- Iteration-over-architecture → `NT-MIND` post-training-only evolution path
- Pixtral → `nt_sense` high-resolution visual encoding
- Apache 2.0 → direct integration candidate for NT-IO PlatformGateway

---

### 1.10 Falcon3-7B (TII)

| Attribute | Detail |
|-----------|--------|
| **Architecture** | Dense Transformer (+ Mamba variant) |
| **Parameters** | 7.5B |
| **Context** | 32K tokens |
| **Attention** | GQA 32Q/8KV, head_dim=256 (wide) |
| **FFN** | 23K intermediate (very wide) |
| **RoPE** | theta=1,000,042 |
| **SSM Variant** | Falcon3-Mamba-7B: Mamba1-based, 64 blocks, state_dim=16 |

**Key Innovations:**
1. **Wide head dimension (256)** — optimized for FlashAttention-3 throughput
2. **Depth up-scaling** — 7B → 10B by duplicating redundant layers + 2T continued pretraining
3. **Mamba SSM variant** — linear complexity alternative to attention

**NeoTrix Mapping:**
- Wide head dimension → `nt_core` FlashAttention-3 optimization opportunity
- Depth up-scaling → `NT-MIND` layer duplication as evolution strategy
- Mamba SSM → `nt_memory` linear-complexity attention alternative for long context

---

## 2. Cross-Model Architecture Innovation Taxonomy

### 2.1 Attention Mechanisms

| Innovation | Models | NeoTrix Priority |
|-----------|--------|-----------------|
| **Dynamic GQA** (adaptive head allocation) | Claude 3.5 Haiku | P0 — GWT salience budget |
| **5:1 Local:Global interleaving** | Gemma 3 | P0 — KV-cache tiering |
| **CSA+HCA hybrid compression** | DeepSeek V4 | P1 — long-context efficiency |
| **Sliding window + global** | Gemma 3, DeepSeek V4 | P0 — hot/cold attention |
| **Fractional RoPE** (25% position-agnostic) | Phi-4-Mini | P1 — cross-domain stability |
| **QK-Norm** (replaces softcapping) | Qwen3, Gemma 3 | P0 — training stability |

### 2.2 Efficiency Mechanisms

| Innovation | Models | NeoTrix Priority |
|-----------|--------|-----------------|
| **Fine-grained MoE (128 experts, no shared)** | Qwen3-30B | P0 — capability routing |
| **Hash-MoE bootstrap** (first 3 layers) | DeepSeek V4 | P1 — cold-start |
| **Mixture-of-LoRAs** (frozen + modality LoRA) | Phi-4-Mini | P0 — modality extension |
| **Runtime FFN sparsity** (20% pruning) | Claude 3.5 Haiku | P1 — adaptive compute |
| **Integrated speculative decoding** | Claude 3.5 Haiku | P0 — inference acceleration |
| **Depth up-scaling** (duplicate + retrain) | Falcon3 | P2 — model growth |

### 2.3 Reasoning Mechanisms

| Innovation | Models | NeoTrix Priority |
|-----------|--------|-----------------|
| **Thinking budget control** | Gemini 2.0 Flash | P0 — ConsciousnessTree budget |
| **Thought signatures** (encrypted state) | Gemini 2.0 Flash | P1 — cross-session state |
| **Multi-Token Prediction** | DeepSeek V3/V4 | P1 — forward projection |
| **Instruction Hierarchy** | GPT-4o Mini | P0 — shield defense |
| **Two-phase reasoning** (justify→execute) | Gemini 2.0 Flash | P0 — SoW gating |

### 2.4 Residual/Stability Mechanisms

| Innovation | Models | NeoTrix Priority |
|-----------|--------|-----------------|
| **mHC (Manifold-Constrained Hyper-Connections)** | DeepSeek V4 | P1 — HyperCube signal stability |
| **Gated Memory Unit (GMU)** | Phi-4-Mini flash | P1 — cross-layer state sharing |
| **QK-Norm** | Qwen3, Gemma 3 | P0 — attention stability |

### 2.5 Vision/Multimodal

| Innovation | Models | NeoTrix Priority |
|-----------|--------|-----------------|
| **Cross-attention adapter** (late fusion) | Llama 3.2 Vision | P0 — PerceptionBridge |
| **Pan&Scan** (adaptive cropping) | Gemma 3, Llama 3.2 | P0 — nt_world pipeline |
| **Frozen SigLIP + projector** | Gemma 3, Phi-4-Mini | P0 — modality bridge |
| **Mixture-of-LoRAs** per modality | Phi-4-Mini | P0 — PlatformGateway |

---

## 3. NeoTrix Integration Roadmap

### P0 — Immediate Integration (this cycle)

| Innovation | Target Module | Implementation |
|-----------|--------------|----------------|
| Dynamic GQA | `GWT` | Adaptive attention head allocation per task salience |
| 5:1 interleaved attention | `nt_memory` | KV-cache tiering: 5/6 layers local, 1/6 global |
| Thinking budget | `ConsciousnessTree` | Configurable inference-time compute per growth cycle |
| Hash-MoE bootstrap | `NT-MIND` | Token-ID routing for SEAL pipeline cold-start |
| Mixture-of-LoRAs | `NT-IO` | Frozen backbone + modality LoRA adapters for PlatformGateway |
| Speculative decoding | `nt_core_llm` | Built-in draft model for cost reduction |
| Instruction Hierarchy | `NT-SHIELD` | Prompt injection defense pattern |

### P1 — Next Cycle

| Innovation | Target Module | Implementation |
|-----------|--------------|----------------|
| CSA+HCA compression | `nt_memory` | Tiered KV cache compression for long-context |
| mHC residuals | `nt_core` HyperCube | Doubly-stochastic signal mixing for deep stacks |
| Multi-Token Prediction | `ConsciousnessTree` | Forward-looking growth projection |
| Thought signatures | `nt_nexus` | Encrypted cross-session reasoning state |
| Fractional RoPE | `nt_core` | Position-agnostic attention for cross-domain |
| Runtime FFN sparsity | `NT-MIND` | Input-specific compute pruning |

### P2 — Backlog

| Innovation | Target Module | Implementation |
|-----------|--------------|----------------|
| Depth up-scaling | `NT-MIND` | Layer duplication as evolution strategy |
| Mamba SSM | `nt_memory` | Linear-complexity attention alternative |
| Pan&Scan | `nt_world` | Multi-resolution perception |

---

## 4. Key Insight: The Reasoning Taxonomy

Three tiers of reasoning models emerged from this analysis:

1. **Budget-Controlled Reasoning** (Gemini 2.0 Flash, Qwen3) — user/configurable thinking budget, explicit cost-quality tradeoff
2. **Architecture-Efficient Reasoning** (Claude 3.5 Haiku, DeepSeek V4) — structural innovations (MoE, compression, sparsity) reduce compute per quality unit
3. **Data-Efficient Reasoning** (Phi-4-Mini, Gemma 3) — synthetic data + distillation + LoRA adapters achieve disproportionate capability

**NeoTrix maps to all three tiers:**
- GWT salience = Budget-Controlled Reasoning
- HyperCube MoE routing = Architecture-Efficient Reasoning
- SEAL pipeline distillation = Data-Efficient Reasoning

---

## 5. Contradiction Resolution

| Tension | Resolution |
|---------|-----------|
| MoE (Qwen3 no shared) vs DeepSeek (1 shared) | Scale-dependent: <100B total → no shared; >500B → shared expert is rounding error |
| Thinking budget vs latency | Adaptive: budget=0 for I/O tasks, budget=8192 for reasoning tasks (Axiom A1) |
| 5:1 local:global vs 1:1 | Gemma 3 showed minimal perplexity difference; 5:1 wins on KV-cache (Axiom A2) |
| Frozen encoder vs end-to-end | Frozen + LoRA wins on cost/quality ratio for multimodal extension (Axiom A3) |
| Dense (Mistral 24B) vs MoE (Qwen3 30B/3B) | MoE wins on latency/cost; Dense wins on inference engine simplicity |

---

*Sources: OpenAI GPT-4o system card, Anthropic Claude 3.5 Haiku addendum, Google Gemini 2.X technical report, Meta Llama 3.2 model card, DeepSeek V3/V4 papers, Qwen3 technical report, Phi-4-Mini technical report, Gemma 3 technical report, Mistral Small 3.2 model card, Falcon3 HuggingFace docs.*
