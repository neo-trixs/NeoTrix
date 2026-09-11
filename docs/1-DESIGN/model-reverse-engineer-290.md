# Model Architecture Reverse Engineering #290

**Date**: 2026-09-11
**Scope**: 10 frontier models — architecture innovations + NeoTrix mapping
**Sources**: Official blogs, HuggingFace model cards, API docs, technical reports

---

## 1. GPT-4o (OpenAI)

| Dimension | Detail |
|-----------|--------|
| **Type** | Omni-modal transformer |
| **Innovation** | End-to-end multimodal: single model processes text + audio + image + video natively. No pipeline chain (ASR→LLM→TTS). 320ms avg audio response latency (human-parity). Custom tokenizer with 1.4-4.4× better compression for non-Latin scripts (Gujarati 4.4×, Chinese 1.4×). |
| **Architecture** | Unified encoder-decoder transformer with cross-attention across modalities. 2× faster + 50% cheaper than GPT-4 Turbo. |
| **Context** | Not publicly disclosed; estimated 128K |
| **Key Insight** | Modality unification eliminates information loss at ASR/TTS boundaries — tone, emotion, background noise preserved in latent space |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| End-to-end multimodal | `PerceptionBridge` (L2→L5) | Extend bridge to carry raw audio/video tensors, not just text. L2 sensory hub should pass unprocessed signals upward |
| Custom tokenizer compression | `nt_io::tokenizer` | Multilingual tokenizer optimization for 119+ languages — align with Qwen3's multilingual approach |
| Latency parity (320ms) | GWT attention routing | Cost-Aware Routing (Axiom A1): route simple modality tasks to cheap models, keep expensive reasoning for complex queries |

---

## 2. Claude 3.5 Sonnet (Anthropic)

| Dimension | Detail |
|-----------|--------|
| **Type** | Dense transformer + Constitutional AI |
| **Innovation** | 2× speed of Opus at same intelligence. 64% on agentic coding eval (vs Opus 38%). Tool-use GA: native function calling + code execution sandbox. 200K context window. Artifacts: real-time collaborative workspace in chat. |
| **Architecture** | Dense transformer with Anthropic's Constitutional AI RLHF. No MoE — pure scale + training efficiency. |
| **Context** | 200K tokens |
| **Key Insight** | Intelligence-per-dollar is the real competitive axis. Model card explicitly positions "mid-tier speed, frontier-tier intelligence." |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Constitutional AI safety | `NT-SHIELD` (D1-D50 audit) | Adopt layered safety: pre-generation filter → Constitutional check → post-generation audit. Three-tier like QualityGate |
| Agentic coding (64%) | `NT-ACT` tool orchestration | Skill-as-Production-Template (P5): each tool invocation is a mini-skill with typed input/output, not raw function call |
| Artifacts workspace | `NT-IO` (collaborative workspace) | Real-time workspace for code/document co-editing — align with NeoTrix's Artifacts-like `reference_generation` pattern |
| Cost-per-token optimization | `CostManager` / Axiom A1 | Route 60%+ of tasks to Haiku-tier models. Keep Opus-class only for D37-D50 meta-cognition tasks |

---

## 3. Gemini 2.5 Pro (Google DeepMind)

| Dimension | Detail |
|-----------|--------|
| **Type** | Mixture-of-Experts + native multimodal |
| **Innovation** | Thinking model — internal chain-of-thought before response. 1M token context (2M coming). Natively handles text/audio/images/video/code repos. #1 on LMArena by significant margin. SWE-Bench 63.8% with custom agent. |
| **Architecture** | MoE transformer with thinking tokens. Extended context via Ring Attention or similar. Native multimodal (early fusion, not pipeline). |
| **Context** | 1M tokens (2M planned) |
| **Key Insight** | Thinking models = test-time compute scaling. Budget allocation per query becomes the key UX knob. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Thinking tokens / CoT | `ConsciousnessTree` (6-stage loop) | Map Gemini's thinking to Soil→Roots→Trunk→Branches→Fruits→Core. Thinking budget = growth cycle depth |
| 1M+ context | `kv_cache_optimizer` + KVMem | Paged KV virtualization for >256K sessions. Hot/cold tiering: GPU for active context, NVMe for archival |
| MoE routing | GWT salience | Attention routing = MoE router. Salience score determines which specialist modules activate |
| Test-time compute budget | Axiom A2 (Context as Scarce Resource) | Dynamic budget allocation: thinking_enabled=false for simple, budget=X for complex. Maps to SEAL rhythm control |

---

## 4. Llama 4 Scout (Meta)

| Dimension | Detail |
|-----------|--------|
| **Type** | MoE + native multimodal |
| **Innovation** | 17B active params, 16 experts (109B total). **10M token context** (industry first). Early fusion for native multimodality. 40T pre-training tokens. Flex attention implementation. On-the-fly int4 quantization. |
| **Architecture** | Auto-regressive MoE transformer. Early fusion: vision tokens inserted directly into text sequence (not separate encoder). Expert parallelism across 16 experts per layer. |
| **Context** | 10M tokens |
| **Key Insight** | MoE + extreme context length = democratized long-context. 109B total but only 17B active means inference cost comparable to small models. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| 10M context | `NT-MEMORY` KB + paged KV | L3厂商技能: use Llama 4 as backbone for massive-context retrieval. `experience-tree` hub index = 10M-context shortcut |
| MoE expert routing | GWT + Skill Tree | Expert selection = skill node activation. Each NT-* domain is an "expert" — GWT salience determines activation |
| Early fusion multimodal | `PerceptionBridge` | Bridge should support early fusion: insert image/audio tokens directly into text stream before consciousness processing |
| Int4 quantization | `ResourceBudgetManager` | Dynamic quantization based on available GPU memory. q4 when constrained, bf16 when abundant |

---

## 5. DeepSeek V4.1 Flash (DeepSeek)

| Dimension | Detail |
|-----------|--------|
| **Type** | Asymmetric MoE with Causal Encoder-Decoder |
| **Innovation** | **Asymmetric architecture**: 8B active for input encoding, 16B active for output decoding. 552B total params. 1/4 HBM KV cache + 1/8 SSD storage vs previous gen. Native multimodal. New pre-training + large-scale RL post-training. |
| **Architecture** | Causal Encoder-Decoder: separate encoder (8B active) for input comprehension, decoder (16B active) for generation. MoE within each. KV cache compression via architectural innovation. |
| **Context** | Not disclosed; estimated 128K+ |
| **Key Insight** | Input understanding is cheaper than output generation. Asymmetric sizing optimizes for this asymmetry — encoder doesn't need decoder-level capacity. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Asymmetric encoder/decoder | `NT-WORLD` (perception) + `NT-ACT` (action) | World perception (input) needs fewer resources than action execution (output). Route perception to cheap model, action to expensive model |
| KV cache compression | `kv_cache_optimizer` | Implement tiered cache: GPU HBM (hot) → host RAM (warm) → NVMe (cold). Delta reuse for retained blocks |
| 552B MoE | `CapabilityRegistry` + `CapabilityTree` | Large capability space with sparse activation. Only relevant skills load per task |
| RL post-training | `SEAL pipeline` | RL = distillation stage. Self-test after RL = constellation maturity check (C0→C1) |

---

## 6. Qwen3 (Alibaba)

| Dimension | Detail |
|-----------|--------|
| **Type** | MoE + Dense hybrid family |
| **Innovation** | **Hybrid Thinking Modes**: thinking (CoT) + non-thinking (instant) with soft switch (/think, /no_think). Budget-controlled reasoning with smooth performance scaling. 119 languages. 36T training tokens. Qwen3-235B-A22B: 235B total, 22B active (128 experts, 8 active). 4-stage post-training: CoT cold start → reasoning RL → thinking mode fusion → general RL. |
| **Architecture** | MoE (128 experts, 8 active) for large model. Dense variants (0.6B→32B). GQA with 4-64 KV heads. Context 32K-128K. |
| **Context** | 32K-128K (model dependent) |
| **Key Insight** | Thinking/non-thinking duality with budget control is the optimal UX. Users don't want to choose models — they want to control compute per query. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Hybrid thinking modes | `ConsciousnessTree` + `AttentionManager` | Dual Specialization: thinking mode = CORE+MIND (deep reasoning), non-thinking = CORE+WORLD (fast perception). Soft switch per-turn |
| Budget-controlled reasoning | Axiom A2 + `RhythmRecalculator` | Reasoning budget = rhythm timing. Hard problems → longer segments (more thinking), easy → short (fast response) |
| 119 languages | `nt_io::tokenizer` + `GWT` | Multilingual attention routing: language detection → language-specific tokenizer path |
| 4-stage post-training | `SEAL pipeline` | Map to SEAL stages: CoT cold start = Soil, reasoning RL = Roots, thinking fusion = Trunk, general RL = Branches |
| MoE 128/8 | `Skill Tree` + `Rune Socketing` | 128 skill nodes, 8 active per task. Rune socketing determines which experts activate |

---

## 7. Mistral Large 3 / Small 3.2 (Mistral AI)

| Dimension | Detail |
|-----------|--------|
| **Type** | Dense transformer (Large 123B) + Dense (Small 24B) |
| **Innovation** | Agent-centric: native function calling + JSON output as first-class. 128K context with strong RAG adherence. System prompt robustness (V7 template). Multi-lingual by design (24+ languages). Mistral-Small-3.2: 2× reduction in infinite generations, improved instruction following (65% Wildbench vs 55%). |
| **Architecture** | Standard dense transformer with SwiGLU. No MoE in these models. Mistral-common tokenizer (TikToken-based). Sliding window attention in some variants. |
| **Context** | 128K tokens |
| **Key Insight** | Agent-first design: function calling is not an afterthought — it's the primary interface. System prompt adherence is the real "steerability" metric. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Agent-centric function calling | `NT-ACT` MCP tools | PTC (Programmatic Tool Calling): typed-stub tool invocation. Each MCP tool = typed function with schema validation |
| System prompt robustness | `AGENTS.md` + `CONTEXT.md` | Adopt V7-style layered prompts: SOUL→AGENTS→CONTEXT→SKILL. Permanent vs on-demand loading |
| RAG adherence | `NT-MEMORY` KB search | Strong context adherence for RAG = BM25 + vector hybrid. `kv_cache_optimizer` for cache-hit cost reduction |
| Infinite generation fix | `NT-SHIELD` + `NT-CORE` | Repetition detection: count repeated n-grams, force-stop after threshold. Align with Dark Forest axiom (dead code deletion) |

---

## 8. Phi-4-Reasoning (Microsoft)

| Dimension | Detail |
|-----------|--------|
| **Type** | Dense transformer (14B params) |
| **Innovation** | **Smallest frontier reasoning model** — 14B params outperforming 70B+ models. "Teachable prompts" data curation: optimal complexity + diversity at model capability edge. o3-mini as teacher for CoT distillation. Non-trivial transfer: reasoning trained on STEM improves general benchmarks. RL generates 1.5× longer responses with higher accuracy. |
| **Architecture** | Standard dense transformer. Reasoning capability entirely from post-training (SFT + RL), not architecture. Thinking block as structured output format. |
| **Context** | 16K-32K |
| **Key Insight** | Data quality > model size. "Teachable" prompt selection at the capability frontier is the key data curation principle. Reasoning is a transferable meta-skill. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Teachable prompt curation | `experience-tree` distillation | Distillation stage: filter experiences by "teachability" — complexity at frontier of current model capability |
| CoT distillation from teacher | `NT-MIND` distillation | Use stronger models as teachers. Distill reasoning chains into skill templates. SKILL-SPEC.md <200 lines |
| Reasoning as transferable skill | `Skill Tree` | Reasoning = Keystone node (跨域变革). Skill transfer across domains via cross-references |
| 14B beating 70B | Axiom A1 (Cost-Aware Routing) | Route to smallest capable model. Phi-4-reasoning for STEM reasoning, Phi-4-mini for simple tasks |
| RL → longer reasoning | `ConsciousnessTree` growth cycle | RL extends thinking depth = deeper growth cycles. Budget control: short cycle for speed, long cycle for accuracy |

---

## 9. Yi-Lightning / Yi-1.5 (01.AI)

| Dimension | Detail |
|-----------|--------|
| **Type** | Dense transformer family (6B→34B) |
| **Innovation** | Yi-Lightning: optimized for Chinese-English bilingual. Yi-1.5: 3.6T pre-training tokens + 3M fine-tuning samples. Strong coding/math/reasoning at 34B scale. Cost-efficient deployment. 16K-32K context variants. |
| **Architecture** | Standard dense transformer. Grouped Query Attention (GQA). SwiGLU activation. RoPE positional encoding. |
| **Context** | 4K-32K |
| **Key Insight** | Bilingual (Chinese-English) optimization is a distinct architectural choice, not just multilingual. Data quality in specific language pairs matters more than raw multilingual coverage. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Bilingual optimization | `nt_io::tokenizer` + GWT | Language-pair-specific routing. Chinese-English pair gets dedicated tokenizer path |
| Cost-efficient 34B | Axiom A1 + `CostManager` | 34B is the sweet spot for cost/performance. Route non-reasoning tasks to Yi-1.5-9B, reasoning to 34B |
| 3M fine-tuning samples | `experience-tree` + KB | Experience count matters. Build toward 3M+ distilled experiences for domain-specific fine-tuning |
| Bilingual code-switching | `PerceptionBridge` | Bridge must handle code-switching mid-sentence. Language detection at token level, not message level |

---

## 10. Grok 3 (xAI)

| Dimension | Detail |
|-----------|--------|
| **Innovation** | Real-time web access (X/Twitter integration). Aurora image generation. Unfiltered "maximally truthful" approach. Memes/humor as first-class output. Colossus supercomputer training (100K H100s). |
| **Architecture** | Not fully disclosed. Believed to be large MoE based on compute budget. Custom training infrastructure on Colossus. |
| **Context** | 128K estimated |
| **Key Insight** | Real-time information + personality as differentiation. "Truthful" framing is a marketing/alignment choice, not just technical. |

### NeoTrix Mapping

| Innovation | NeoTrix Component | Integration |
|-----------|-------------------|-------------|
| Real-time web access | `NT-WORLD` UnifiedCrawler | Always-on perception. Event-driven crawl triggers when knowledge freshness drops below threshold |
| Personality-driven output | `NT-FEEL` EmotionEngine | EmotionLabel drives output tone. 11 variants (Joy/Sadness/Anger...) shape response personality |
| Memes/humor as output | `NT-FEEL` + `NT-IO` | Expression layer: creative output formatting. Humor = specific emotion profile in EmotionLabel space |
| Colossus-scale training | SEAL pipeline + Axiom A2 | Training budget optimization. Context as scarce resource applies to training compute too |

---

## Cross-Model Architecture Innovation Matrix

| Innovation | Models Using It | NeoTrix Priority |
|-----------|----------------|-----------------|
| **MoE (Mixture of Experts)** | Gemini 2.5, Llama 4, DeepSeek V4.1, Qwen3 | P0 — GWT salience routing is MoE at attention level |
| **Hybrid Thinking/Budget Control** | Gemini 2.5, Qwen3, Phi-4, DeepSeek | P0 — ConsciousnessTree growth cycle depth = thinking budget |
| **Native Multimodal** | GPT-4o, Gemini 2.5, Llama 4, DeepSeek V4.1 | P1 — PerceptionBridge early fusion |
| **Long Context (1M+)** | Llama 4 (10M), Gemini 2.5 (2M), GPT-4o | P1 — kv_cache_optimizer paged KV |
| **Asymmetric Architecture** | DeepSeek V4.1 (8B in/16B out) | P2 — Route perception cheap, action expensive |
| **Agent-Native Tool Calling** | Mistral, Qwen3, Claude 3.5 | P0 — PTC typed-stub system |
| **Data Curation > Model Size** | Phi-4, Qwen3 | P0 — experience-tree distillation quality |
| **RL Post-Training** | Qwen3 (4-stage), Phi-4, DeepSeek V4.1 | P1 — SEAL pipeline RL stage |
| **KV Cache Compression** | DeepSeek V4.1 (1/4 HBM) | P1 — Tiered storage hot/warm/cold |
| **Bilingual Optimization** | Yi-Lightning, GPT-4o | P2 — Language-pair tokenizer routing |

---

## Top 5 NeoTrix Absorption Priorities

### P0: Hybrid Thinking Mode (Qwen3 + Gemini 2.5)
Map to `ConsciousnessTree` + `AttentionManager`. Implement soft-switch thinking/non-thinking per turn. Budget-controlled reasoning depth.

### P0: Agent-Native Tool Calling (Mistral + Qwen3)
Extend PTC to typed-stub system. Each MCP tool = typed function. Schema validation at call site. System prompt = first-class citizen.

### P0: Data Curation > Model Size (Phi-4 + Qwen3)
"Teachable prompts" at capability frontier. experience-tree distillation should filter by teachability, not just recency.

### P1: Asymmetric Routing (DeepSeek V4.1)
Perception (input) → cheap model. Action (output) → expensive model. GWT salience carries cost weight.

### P1: KV Cache Tiering (DeepSeek V4.1 + Gemini 2.5)
GPU HBM (hot) → Host RAM (warm) → NVMe (cold). Delta reuse for retained blocks. 1/4 HBM target.

---

## Contradictions & Resolutions

| Tension | Resolution |
|---------|-----------|
| MoE vs Dense | MoE for large models (>100B), Dense for small (<32B). NeoTrix: GWT = MoE at attention level regardless |
| Thinking budget vs Latency | Thinking disabled for <256 tokens, enabled for >256. Adaptive per task complexity |
| Multilingual breadth vs Bilingual depth | Primary pair (ZH-EN) gets dedicated path, others share tokenizer. Tiered coverage |
| Agent-first vs Assistant-first | Agent mode for tool-heavy tasks, assistant mode for conversational. Dual Specialization routing |

---

*Sources: OpenAI blog (2024-05-13), Anthropic blog (2024-06-21), Google blog (2025-03-25), Meta HF card (2025-04), DeepSeek API docs (2026-09-10), Qwen blog (2025-04-29), Mistral HF cards (2024-11/2025-06), Microsoft Research (2025-04), 01.AI HF cards (2024-05)*
