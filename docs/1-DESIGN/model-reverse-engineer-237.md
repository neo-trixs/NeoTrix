# Model Reverse Engineering — Cycle 237

> 2026-09-11 | 10 Models | Architectural Innovations & NeoTrix Mapping

---

## 1. GPT-4o (OpenAI)

| Dimension | Detail |
|-----------|--------|
| Architecture | Decoder-only Transformer, natively multimodal (text + audio + image unified) |
| Context | 128K tokens, 16K output |
| Key Innovation | **Omni architecture** — single neural network trained end-to-end across text, vision, audio. Replaces cascade of separate ASR→LLM→TTS pipelines. Unified tokenizer mixing BPE text tokens, image patches, and discretised audio frames. New tokenizer achieves 1.1x–4.4x compression improvement on non-English scripts. |
| Training | Multi-trillion-token web corpus + image-text pairs + licensed audio. RLHF with model-graded rewards and red-teaming under Preparedness Framework. |
| Special | 320ms median spoken response latency. Structured Outputs with strict JSON schema adherence. Function calling with parallel tool calls. |

### NeoTrix Mapping

| GPT-4o Pattern | NeoTrix Component | Gap/Action |
|----------------|-------------------|------------|
| Unified multimodal tokenizer | **NT-IO** multimodal input pipeline | Align: NeoTrix currently handles text→LLM. Need unified tokenizer across modalities for agent perception. |
| Omni single-network architecture | **NT-PHYSICAL** sensory integration | Map: Replace cascade approach in `SensoryIntegrationHub` with unified embedding space. |
| RLHF with model-graded rewards | **NT-MIND** SEAL reward modeling | Extend reward model to include self-graded signals, not just human preference. |
| Structured Outputs | **NT-ACT** tool calling | Already partially implemented via MCP. Verify strict schema adherence matches GPT-4o parity. |

---

## 2. Claude Sonnet 5 (Anthropic)

| Dimension | Detail |
|-----------|--------|
| Architecture | Transformer with hybrid local-global sparse attention |
| Context | 200K tokens (estimated) |
| Key Innovation | **Hybrid sparse attention** — alternates local sliding window (1024 tokens) with global sparse attention (every 64th token) across 36 layers. Reduces FLOPs by 62% vs dense attention at 100K tokens. GQA with 8 query groups per KV head cuts memory by 37%. Optional lossless context compression for repeated patterns (22% reduction). |
| Training | RLHF + Constitutional AI. Cyber safeguards enabled by default. Agentic performance (coding, tool use) approaching Opus-class at Sonnet pricing. |
| Special | Near-Opus intelligence at Sonnet pricing. Computer use capability (GUI screenshot interpretation → tool calls). 3-tier pricing: Start / Build / Scale. |

### NeoTrix Mapping

| Claude Pattern | NeoTrix Component | Gap/Action |
|----------------|-------------------|------------|
| Hybrid sparse attention | **nt_core** attention routing (GWT) | Map: GWT salience routing already conceptually similar. Implement attention cost modulation — local window for routine queries, global sparse for long-range dependencies. |
| Context compression | **NT-MEMORY** KV cache | Extend `kv_cache_optimizer.rs` with lossless compression for repeated context patterns. |
| Computer use (screenshot → tool calls) | **NT-WORLD** perception bridge | Direct map: `PerceptionBridge` should support GUI screenshot parsing → MCP tool call generation. |
| Cyber safeguards by default | **NT-SHIELD** egress guard | Align: Egress Privacy Guard already implements trust-tiered filtering. Add real-time behavioral detection layer. |

---

## 3. Gemini 2.5 Pro (Google DeepMind)

| Dimension | Detail |
|-----------|--------|
| Architecture | Sparse Mixture-of-Experts (MoE) Transformer, natively multimodal |
| Context | 1M+ tokens (2M coming), 65K output |
| Key Innovation | **Controllable thinking budget** — user sets token budget for internal reasoning. Performance scales with budget allocation. Deep Think mode: parallel hypothesis generation + critique before final answer. MoE decouples total capacity from per-token computation cost. |
| Training | TPUv5p, 8960-chip pods across multiple datacenters. Distillation from teacher models. First Gemini on TPUv5p. |
| Special | Processes 3-hour video natively. Emergent multimodal coding (convert video → interactive app). MoE routing: dynamic token→expert assignment. Full Pareto frontier coverage (Pro→Flash→Flash-Lite). |

### NeoTrix Mapping

| Gemini Pattern | NeoTrix Component | Gap/Action |
|----------------|-------------------|------------|
| Controllable thinking budget | **nt_core** reasoning control | Direct map to GWT attention modulation. Implement `thinking_budget` parameter in ConsciousnessTree that allocates computation based on task complexity. |
| Deep Think (parallel hypotheses) | **NT-MIND** SEAL exploration | Extend SEAL pipeline to spawn parallel hypothesis branches in exploration phase, then critique/vote before distillation. |
| MoE dynamic routing | **CapabilityBridge** runtime routing | Map: CapabilityBridge already bridges evolution view ↔ runtime view. Add dynamic expert activation based on token type (similar to MoE routing). |
| 1M+ token context | **nt_core** kv_cache_optimizer | Extend paged KV to support 1M+ tokens. Already in design (KVMem paged KV virtualization from 2026-09-08 axioms). |

---

## 4. Llama 4 Scout/Maverick (Meta)

| Dimension | Detail |
|-----------|--------|
| Architecture | MoE with early-fusion multimodality. iRoPE (interleaved attention layers without positional embeddings + RoPE on most layers). |
| Context | Scout: 10M tokens. Maverick: 1M tokens |
| Key Innovation | **iRoPE architecture** — interleaved attention layers without positional embeddings + inference-time temperature scaling for length generalization. Early fusion: text + vision tokens unified in single backbone from pretraining start. Alternating dense + MoE layers for inference efficiency. Scout: 17B active, 109B total, 16 experts. Maverick: 17B active, 400B total, 128 experts. |
| Training | 40T tokens (Scout), 22T tokens (Maverick). Lightweight SFT + online RL + lightweight DPO. Vision encoder: MetaCLIP adapted with frozen Llama. |
| Special | Scout fits single H100 with int4 quantization. Maverick fits single H100 host with FP8. 10M token context enables whole-codebase reasoning. |

### NeoTrix Mapping

| Llama 4 Pattern | NeoTrix Component | Gap/Action |
|-----------------|-------------------|------------|
| iRoPE (position-free interleaved layers) | **nt_core** positional encoding | Map: Implement interleaved attention without position embeddings for ultra-long context. Aligns with "infinite context" goal. |
| Early fusion multimodality | **NT-PHYSICAL** sensory integration | Direct map: Fuse all modality tokens into unified backbone from start, not late-attach. Replaces cascade in `SensoryIntegrationHub`. |
| MoE with shared + routed experts | **CapabilityRegistry** expert routing | Map: Each MoE expert = a capability node. Shared expert = core capabilities always active. Routed experts = domain-specific nodes activated by task. |
| Single-H100 deployment | **NT-IO** local inference | Align: NeoTrix should support local Ollama deployment with quantized models for privacy-sensitive workloads. |

---

## 5. DeepSeek V4.1 Flash

| Dimension | Detail |
|-----------|--------|
| Architecture | **Causal Encoder-Decoder (CED)** — 40-layer Transformer: 20-layer causal encoder + 20-layer decoder. 552B backbone, 8B active (prefill), 16B active (decode). |
| Context | 1M tokens, 384K output |
| Key Innovation | **Asymmetric activation architecture** — input-heavy agentic workloads get cheap encoder (8B), generation gets expensive decoder (16B). **Compressed Sparse Attention 2 (CSA2)** — three static modes (Full/Reindex/Reuse) sharing KV data across layers. **FP4 KV caching** at 890 bytes/token (1/4 of V4 Flash). **SWA Bounded Replay** — replay recent window instead of SSD persistence (1/8 KV footprint). **Engram conditional memory** — 196B parameters sparsely accessed via token-based lookup. |
| Training | 45T multimodal tokens from scratch. Sparse attention trained at 64K, extended to 1M at 34T tokens. |
| Special | CED split: encoder processes input cheaply, decoder generates expensively. Hierarchical Sparse Indexer bounds deeper layer cost independent of context length. DSpark speculative decoding for generation speedup. |

### NeoTrix Mapping

| DeepSeek Pattern | NeoTrix Component | Gap/Action |
|------------------|-------------------|------------|
| Asymmetric encoder-decoder | **NT-WORLD** perception + **NT-ACT** action | **CRITICAL INSIGHT**: Input perception (NT-WORLD) should be cheap encoder, action generation (NT-ACT) should be expensive decoder. Currently both use same compute budget. Implement cost-aware bifurcation. |
| CSA2 KV sharing across layers | **NT-MEMORY** KV cache | Extend `kv_cache_optimizer.rs` with cross-layer KV sharing (Full/Reindex/Reuse modes). Maps to PSA-2026-042. |
| FP4 KV caching | **NT-MEMORY** quantization | Implement FP4 (E2M1) KV storage with per-16-channel scaling. 890 bytes/token target. |
| Engram conditional memory | **NT-MEMORY** knowledge base | Map: Engram = sparse KB lookup by token signature. NeoTrix KB already supports node lookup; add token-based conditional access pattern. |
| SWA Bounded Replay | **NT-MEMORY** sliding window | Implement: for sliding-window attention layers, replay recent tokens instead of persisting KV to disk. |

---

## 6. Qwen 3 (Alibaba)

| Dimension | Detail |
|-----------|--------|
| Architecture | Dense + MoE variants. GQA, SwiGLU, RoPE, RMSNorm. QK-Norm (replaces QKV-bias). MoE: 128 experts, 8 activated per token, no shared experts. |
| Context | Up to 128K (dense), 1M (2507 update) |
| Key Innovation | **Unified thinking/non-thinking mode** — single model switches between chain-of-thought reasoning (thinking) and direct response (non-thinking) via mode tokens. Thinking budget mechanism for adaptive computation. **Qwen3.8-Next**: Gated DeltaNet + global attention hybrid, Gated Residual (4-branch residual stream), n-gram embedding tables held off-accelerator. |
| Training | 36T tokens, 119 languages. Knowledge distillation from flagship to smaller models. |
| Special | Fine-grained expert segmentation. Global-batch load balancing loss for expert specialization. Seamless mode switching eliminates need for separate reasoning + chat models. |

### NeoTrix Mapping

| Qwen 3 Pattern | NeoTrix Component | Gap/Action |
|----------------|-------------------|------------|
| Unified thinking/non-thinking | **ConsciousnessTree** mode switching | Direct map: ConsciousnessTree should switch between "thinking" (deep SEAL exploration) and "non-thinking" (direct tool execution) based on task complexity. |
| Thinking budget mechanism | **GWT** attention allocation | Extend GWT salience scoring with explicit budget parameter. Maps to Cost-Aware Routing axiom (A1). |
| QK-Norm (stable training) | **nt_core** attention stability | Adopt QK-Norm in attention layers for training stability. Small but critical architectural improvement. |
| n-gram embedding off-accelerator | **NT-MEMORY** external embeddings | Map: Prefetch frequently-used embeddings from host memory, not compute them every forward pass. Aligns with paged KV virtualization. |
| Global-batch load balancing | **CapabilityBridge** routing optimization | Extend expert routing to use global-batch balancing, not per-token. Prevents expert collapse in capability nodes. |

---

## 7. Mistral Large 3 (Mistral AI)

| Dimension | Detail |
|-----------|--------|
| Architecture | Granular Mixture-of-Experts. 675B total, 41B active. 2.5B vision encoder. |
| Context | 256K tokens |
| Key Innovation | **Granular MoE** — fine-grained expert segmentation for more nuanced knowledge decomposition. Open-weight Apache-2.0 license at frontier scale. Trained from scratch on 3000 H200s. |
| Training | From scratch on 3000 NVIDIA H200 GPUs. Multimodal + multilingual (40+ languages). NVFP4 format for efficient deployment. |
| Special | Largest open-weight model from major lab. FP8 on single B200/H200 node. NVFP4 on single H100/A100 node. Speculative decoding supported. |

### NeoTrix Mapping

| Mistral Pattern | NeoTrix Component | Gap/Action |
|-----------------|-------------------|------------|
| Granular MoE segmentation | **CapabilityRegistry** fine-grained nodes | Map: Split coarse capability nodes into finer-grained segments. Each segment = a mini-expert with specialized knowledge. |
| Open-weight frontier deployment | **NT-IO** self-hosted inference | Align: NeoTrix should support self-hosted Mistral Large 3 via vLLM for air-gapped deployments. |
| NVFP4 quantization | **NT-MEMORY** storage optimization | Adopt NVFP4 format for model weight storage. Reduces memory footprint while maintaining quality. |
| Speculative decoding | **NT-ACT** generation acceleration | Implement DSpark-style speculative decoding in action generation pipeline. |

---

## 8. Phi-4-reasoning-vision-15B (Microsoft)

| Dimension | Detail |
|-----------|--------|
| Architecture | **Mid-fusion** VLM — SigLIP-2 vision encoder + MLP projector + Phi-4-Reasoning backbone. Dynamic-resolution input (up to 3600 visual tokens). |
| Context | 128K tokens |
| Key Innovation | **Hybrid THINK/NOTHINK mode** — single 15B model switches between chain-of-thought (math/science) and direct inference (perception). 20% reasoning data mix. Dynamic-resolution vision encoder (SigLIP-2 Naflex variant). Mid-fusion as practical trade-off: expressivity vs efficiency. |
| Training | 200B multimodal tokens (vs 1T+ for competitors). 240 B200 GPUs, 4 days. Data quality > data quantity. |
| Special | Competitive with models 10× its size. 84.8% AI2D, 83.3% ChartQA, 88.2% ScreenSpot-V2. Open-weight. |

### NeoTrix Mapping

| Phi-4 Pattern | NeoTrix Component | Gap/Action |
|---------------|-------------------|------------|
| Hybrid THINK/NOTHINK | **ConsciousnessTree** task-adaptive reasoning | Direct map: Mode tokens (`<think>`/`</think>`) control reasoning depth. Implement in NT-CORE as adaptive depth control. |
| Mid-fusion architecture | **NT-PHYSICAL** sensory integration | Align: Mid-fusion (frozen encoder + trainable projector + frozen LLM) is more practical than full early fusion. Adopt for cost-constrained deployments. |
| Data quality > quantity | **NT-MIND** distillation pipeline | Reinforce: SEAL distillation should prioritize curated data over raw volume. Phi-4 achieves frontier performance with 5× less training data. |
| Dynamic-resolution vision | **NT-WORLD** perception bridge | Extend PerceptionBridge with dynamic-resolution image processing. Support up to 3600 visual tokens for high-fidelity GUI grounding. |

---

## 9. Yi-Lightning (01.AI)

| Dimension | Detail |
|-----------|--------|
| Architecture | Enhanced MoE with fine-grained expert segmentation. Cross-layer KV cache sharing. Hybrid attention (3 sliding window + 1 full attention). |
| Context | Not explicitly specified |
| Key Innovation | **Cross-layer KV cache reuse** — shares KV states between consecutive full attention layers, reducing memory by 50%. **Partitioned EP load balancing (PEP)** — splits experts into partitions for balanced token distribution across All-to-All communication. **Hybrid attention blocks** — 3 sliding window + 1 full attention captures both local and global patterns. 82.8% memory reduction for long sequences. |
| Training | FP8 quantization aligned with Hopper GPU architecture. 1200 TFLOPS/card at FP8 on Hopper. Expert-parallel MoE operator. |
| Special | Ranked 6th on Chatbot Arena, 2nd in Chinese/Math/Coding. RAISE safety framework (4-component). Multi-stage training with synthetic data construction. |

### NeoTrix Mapping

| Yi-Lightning Pattern | NeoTrix Component | Gap/Action |
|----------------------|-------------------|------------|
| Cross-layer KV cache reuse | **NT-MEMORY** KV cache | Direct map: Share KV states between consecutive attention layers. 50% memory reduction. Extend `kv_cache_optimizer.rs`. |
| Partitioned EP load balancing | **CapabilityBridge** routing | Implement partitioned load balancing for capability nodes. Prevents routing imbalance in All-to-All expert communication. |
| 3:1 sliding/full attention ratio | **GWT** attention pattern | Map: Most attention heads handle local context (sliding window), few handle global (full). Implement as GWT default routing pattern. |
| FP8 hardware-aware design | **NT-IO** inference optimization | Align model architecture with GPU quantization capabilities. Design attention layers for FP8 compatibility from the start. |
| RAISE safety framework | **NT-SHIELD** safety pipeline | Map: 4-component safety (pre-training + post-training + serving + monitoring). Align with NT-SHIELD egress guard + audit trail. |

---

## 10. Grok 3 / Grok 4 (xAI)

| Dimension | Detail |
|-----------|--------|
| Architecture | Sparse MoE Transformer (Grok-1: 314B, 8 experts). Enhanced attention mechanisms. |
| Context | Grok 3: 1M tokens. Grok 4.1: 2M tokens. Grok 4.6: 500K tokens. |
| Key Innovation | **Large-scale RL at pretraining scale** — Grok 4 trained with RL on 200K GPU Colossus cluster, extending verifiable training data beyond math/coding into additional domains. **Native tool training** — web browsing, real-time search, code execution, MCP connections trained into model via RL, not external orchestration. **Multi-agent capabilities** (Grok 4.20). |
| Training | Grok 3: 10× compute of previous SOTA. Grok 4: 6× compute efficiency improvement. Colossus supercluster 200K H100 GPUs. |
| Special | DeepSearch agent — real-time knowledge synthesis across web/X. Multi-agent mode (SuperGrok Heavy, $300/mo). Voice mode with thinking. SpaceX integration. |

### NeoTrix Mapping

| Grok Pattern | NeoTrix Component | Gap/Action |
|--------------|-------------------|------------|
| RL at pretraining scale | **NT-MIND** SEAL reinforcement | Map: Scale SEAL exploration beyond post-training. Use RL during capability acquisition (not just distillation). |
| Native tool training via RL | **NT-ACT** tool integration | **CRITICAL**: Tools should be trained into the model, not just orchestrated externally. MCP tool calling should be learned behavior, not prompt-driven. |
| Multi-agent capabilities | **ConsciousnessTree** multi-agent | Map: ConsciousnessTree already routes across specialist modules. Extend to spawn parallel agent instances for complex tasks. |
| DeepSearch (real-time synthesis) | **NT-WORLD** live perception | Align: NT-WORLD crawl pipeline should support real-time knowledge synthesis across live sources. |
| Hierarchical memory layers | **NT-MEMORY** tiered memory | Map: Implement hierarchical memory — working (hot), session (warm), KB (cold) — matching Grok's memory layering. |

---

## Cross-Model Synthesis: Top 10 Architectural Convergences

| # | Convergence | Models | NeoTrix Status | Priority |
|---|-------------|--------|----------------|----------|
| 1 | **MoE as universal architecture** | Gemini 2.5, Llama 4, DeepSeek V4.1, Mistral Large 3, Yi-Lightning, Grok 3 | CapabilityBridge concept exists, not implemented | **P0** |
| 2 | **Unified thinking/non-thinking mode** | Gemini 2.5, Qwen 3, Phi-4 | ConsciousnessTree has mode concept, no dynamic switching | **P0** |
| 3 | **KV cache compression** | DeepSeek (890B/token), Yi-Lightning (82.8% reduction), Claude (GQA) | kv_cache_optimizer.rs exists, needs extension | **P0** |
| 4 | **Asymmetric compute** | DeepSeek (8B encode/16B decode), GPT-4o (cheap perception/expensive reasoning) | Not implemented | **P1** |
| 5 | **Controllable thinking budget** | Gemini 2.5, Qwen 3 | Cost-Aware Routing axiom (A1) defined, not wired | **P1** |
| 6 | **Natively multimodal early fusion** | GPT-4o, Llama 4, Gemini 2.5 | Cascade approach in NT-PHYSICAL | **P1** |
| 7 | **Hybrid sparse attention** | Claude (local/global), Yi-Lightning (3:1 SWA:Full), Llama 4 (iRoPE) | GWT routing exists, no attention pattern specialization | **P2** |
| 8 | **RL-trained tool use** | Grok 4 (native tools via RL), Claude (computer use) | MCP tools are prompt-driven, not trained | **P2** |
| 9 | **Granular expert segmentation** | Mistral Large 3, Yi-Lightning, Qwen 3 | CapabilityRegistry uses coarse nodes | **P2** |
| 10 | **Off-accelerator memory** | Qwen3.8-Next (n-gram tables), DeepSeek (Engram) | NT-MEMORY fully on-disk | **P3** |

---

## NeoTrix Architecture Implications

### Immediate Actions (P0)

1. **MoE Routing for CapabilityRegistry** — Implement dynamic expert activation where each capability node acts as a routed expert. Shared experts = always-active core capabilities. Routed experts = domain-specific nodes activated by task signature. Source: Gemini 2.5, Llama 4, DeepSeek V4.1.

2. **Adaptive Thinking Mode** — Extend ConsciousnessTree with explicit `<think>`/`</think>` mode switching. Budget parameter controls computation depth. Source: Qwen 3, Phi-4, Gemini 2.5.

3. **KV Cache v2** — Extend `kv_cache_optimizer.rs` with:
   - Cross-layer KV sharing (Yi-Lightning: 50% memory reduction)
   - CSA2-style Full/Reindex/Reuse modes (DeepSeek)
   - FP4 quantization target: 890 bytes/token (DeepSeek)
   - Lossless compression for repeated patterns (Claude)

### Strategic Direction (P1)

4. **Asymmetric Perception-Action** — Split NT-WORLD (perception) as cheap encoder and NT-ACT (action) as expensive decoder. 8B/16B ratio from DeepSeek proves this is viable for agentic workloads.

5. **GWT Budget Control** — Wire Cost-Aware Routing (Axiom A1) into GWT salience scoring. Budget parameter = max tokens for internal reasoning. Maps to Gemini 2.5 controllable thinking + Qwen 3 thinking budget.

6. **Multimodal Early Fusion** — Replace cascade in SensoryIntegrationHub with unified backbone. Fuse text + vision + audio tokens from pretraining start. Source: GPT-4o, Llama 4.

### Research Horizon (P2-P3)

7. **Hybrid Attention Pattern** — Implement 3:1 sliding window to full attention ratio in GWT. Most heads = local context (cheap), few heads = global dependencies (expensive). Source: Yi-Lightning, Claude.

8. **Trained Tool Use** — Move from prompt-driven MCP calls to RL-trained tool invocation. Source: Grok 4 native tool training.

9. **Off-Accelerator Memory** — Prefetch frequently-used embeddings from host memory. Source: Qwen3.8-Next n-gram tables, DeepSeek Engram.

---

## Appendix: Model Parameter Summary

| Model | Total Params | Active Params | Experts | Context | Modalities |
|-------|-------------|---------------|---------|---------|------------|
| GPT-4o | ~1.8T (est.) | ~1.8T | Dense | 128K | Text, Audio, Image |
| Claude Sonnet 5 | ~200B (est.) | ~200B | Dense | 200K | Text, Image |
| Gemini 2.5 Pro | ~1.5T (est.) | Sparse MoE | MoE | 1M+ | Text, Audio, Image, Video |
| Llama 4 Scout | 109B | 17B | 16 | 10M | Text, Image |
| Llama 4 Maverick | 400B | 17B | 128 | 1M | Text, Image |
| DeepSeek V4.1 Flash | 552B | 8B/16B | 384 | 1M | Text, Image |
| Qwen3-235B-A22B | 235B | 22B | 128 | 128K–1M | Text |
| Mistral Large 3 | 675B | 41B | Granular | 256K | Text, Image |
| Phi-4-r-v-15B | 15B | 15B | Dense | 128K | Text, Image |
| Yi-Lightning | ~100B | MoE | Fine-grained | — | Text |
| Grok 3 | ~1.5T (est.) | MoE | 8+ (est.) | 1M | Text, Image, Audio |
| Grok 4 | undisclosed | MoE | undisclosed | 2M | Text, Image, Audio, Video |
