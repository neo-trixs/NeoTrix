# Model Architecture Reverse-Engineering #302

> 10 models: GPT-4o, Claude 3.5 Sonnet, Gemini 2.5 Pro, Llama 4 Scout, DeepSeek V4.1 Flash, Qwen 3, Mistral Large 3, Phi-4 Reasoning, Yi-Lightning, Grok 3
> Date: 2026-09-11 | Sources: technical reports, system cards, blog posts, HuggingFace docs

---

## 1. GPT-4o

**Source**: GPT-4o System Card (arXiv:2410.21276), GPT-ImgEval (arXiv:2504.02782), MLSysReview analysis

| Dimension | Detail |
|-----------|--------|
| Architecture | Autoregressive omni model — single neural network, end-to-end across text, vision, audio |
| Parameters | Undisclosed (est. 200B+ MoE) |
| Context | 128K tokens |
| Training | Joint multimodal pre-training (not staged CLIP+LLM+TTS) |
| Modalities | Text + image + audio input; text + audio + image output |
| Audio | Neural audio codec tokenizer (~50-75 tokens/sec), 232ms median latency |
| Image | AR + diffusion-based decoder head (empirically confirmed by GPT-ImgEval classifier) |

### Key Innovations

1. **Unified token stream**: Text BPE + image patch tokens + audio codec tokens consumed by single transformer stack — cross-modal attention via self-attention, not separate cross-modal layers
2. **End-to-end multimodal training**: Replaces staged CLIP+Whisper+TTS pipeline; collapses 3-stage pipeline (2.8s) → single network (232ms)
3. **Diffusion image head**: AR generates latent, diffusion decoder produces pixels (evidence: GPT-ImgEval trained binary classifier on VAR vs diffusion images, 10K samples, consistent classification as diffusion-based)
4. **Cost-per-token inversion**: Despite matching GPT-4 Turbo, priced 50% lower — implies MoE or architectural efficiency gains

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Unified token stream | NT-IO (modality bridge) | Extend `nt_io::modality_bridge` to support unified tokenization across text/audio/image within a single attention stream |
| End-to-end multimodal | NT-WORLD (perception) | Collapse staged perception pipeline in `nt_world::sensory_hub` → single forward pass |
| Diffusion image head | NT-ACT (generation) | Adopt hybrid AR+diffusion pattern for image/video generation tasks in `nt_act::generation_engine` |
| Cost inversion | NT-CORE (GWT) | Implement cost-aware routing: simple modal tasks → cheap path, complex reasoning → expensive path |

---

## 2. Claude 3.5 Sonnet

**Source**: Model Card Addendum (Anthropic), Anthropic blog posts, upgraded model card (2024-10)

| Dimension | Detail |
|-----------|--------|
| Architecture | Dense transformer (evolution of Claude 3 family) |
| Parameters | Undisclosed |
| Context | 200K tokens |
| Training | Constitutional AI + RLHF; staged pre-training |
| Modalities | Text + image input → text output |
| Safety | ASL-2 classification; RSP-aligned |
| Speed | 2x Claude 3 Opus at lower cost |

### Key Innovations

1. **Constitutional AI (CAI)**: Self-supervised alignment via constitutional principles — model critiques and revises its own outputs against a set of principles
2. **Agentic coding**: Solves 64% of real-world open-source PRs (upgraded to 78%); iterative self-correcting agent loop — search → view → edit → run tests → fix
3. **Computer use**: Screenshot → GUI command generation; OSWorld benchmark 14.9% → 22% with more steps
4. **Tool use / function calling**: Native integration for multi-step workflows with external tools
5. **Hierarchical responsibility scaling**: RSP defines ASL thresholds with council-based escalation

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Constitutional AI | NT-GOVERNANCE (Gov-衡) | Formalize `gov/steward` skill with constitution document + self-critique loop |
| Agentic coding loop | NT-ACT (Dev-匠) | Enhance `dev/implementer` with iterative search→edit→test→fix cycles |
| Computer use | NT-WORLD (perception) + NT-ACT | Add GUI screenshot parsing to `nt_world::vision` + action generation to `nt_act::automation` |
| Tool calling | NT-IO (interface) | Strengthen MCP tool-calling with structured output validation |
| RSP thresholds | NT-SHIELD (Rev-明) | Implement capability threshold gating with escalation protocols in `rev/officer` |

---

## 3. Gemini 2.5 Pro

**Source**: arXiv:2507.06261, Model Card, Google DeepMind technical report (July 2025)

| Dimension | Detail |
|-----------|--------|
| Architecture | Sparse Mixture-of-Experts (MoE) transformer |
| Parameters | Undisclosed (est. >1T total, sparse routing) |
| Context | 1M+ tokens |
| Training | TPUv5p, multi-datacenter, synchronous data-parallel |
| Modalities | Native multimodal: text, vision, audio input → text output |
| Training stability | Major advances in signal propagation + optimization dynamics |
| Distillation | k-sparse distribution approximation for smaller models |

### Key Innovations

1. **Controllable thinking budget**: User sets token budget for internal reasoning; performance scales smoothly with budget — bridges reasoning and non-reasoning in one model
2. **Native multimodal + 1M context**: Process 3-hour videos, entire codebases, interleaved audio+text+video in single context window
3. **Sparse MoE with training stability breakthroughs**: Solved large-scale MoE training instabilities (loss spikes, divergences) through signal propagation improvements
4. **K-sparse distillation**: Approximate teacher's next-token distribution with k-sparse subset — reduces storage overhead while maintaining quality
5. **Thinking budget as cost lever**: Budget control enables quality-latency-cost tradeoff at inference time

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Thinking budget | NT-CORE (E8 reasoning) | Implement `nt_core::thinking_budget` — allocate compute per task complexity, GWT salience-modulated |
| Long context | NT-MEMORY (KB) | Extend `nt_memory::kv_cache` to support 1M+ token working sets with tiered storage |
| Training stability | NT-MIND (SEAL) | Adopt signal propagation diagnostics in SEAL pipeline phase monitoring |
| K-sparse distillation | NT-MIND (distillation) | Use k-sparse teacher distribution for `nt_mind::distillation` — compress knowledge without full vocab overhead |
| MoE routing | NT-CORE (GWT) | Map MoE expert routing to GWT salience-based specialist activation |

---

## 4. Llama 4 Scout

**Source**: Meta AI blog, MODEL_CARD.md (GitHub), technical details from announcement

| Dimension | Detail |
|-----------|--------|
| Architecture | MoE with early fusion for native multimodality |
| Parameters | 17B active / 109B total (16 experts) |
| Context | 10M tokens (industry-leading) |
| Training | ~40T tokens multimodal data |
| Modalities | Multilingual text + image input → text + code output |
| Quantization | BF16, on-the-fly int4 (fits single H100) |

### Key Innovations

1. **iRoPE architecture**: Interleaved attention layers — some layers use RoPE, some have NO positional embeddings. Inference-time temperature scaling of attention for length generalization. "i" = infinite context goal
2. **10M context window**: 256K pre-training + mid-training extension; retrieval needle-in-haystack validated at 10M tokens
3. **Early fusion multimodality**: Text and vision tokens fused at input embedding level (not separate encoder) — enables joint pre-training on unlabeled text+image+video
4. **Alternating dense/MoE layers**: Mix of dense and MoE layers for inference efficiency; shared expert + routed experts per token
5. **Mid-training recipe**: Dedicated training phase between pre-training and post-training for capability enhancement (long context extension, specialized datasets)

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| iRoPE (positional-free layers) | NT-CORE (HyperCube) | Experiment with position-free attention layers in `nt_core::attention` for infinite-context support |
| 10M context | NT-MEMORY | Design tiered KV cache: hot (GPU) → warm (CPU) → cold (SSD) for 10M-scale sessions |
| Early fusion | NT-WORLD (perception) | Fuse modality tokens at embedding stage in `nt_world::multimodal_fusion` |
| Alternating dense/MoE | NT-CORE (GWT) | Alternate specialist (MoE) and generalist (dense) layers in GWT routing |
| Mid-training | NT-MIND (SEAL) | Formalize SEAL Phase-2.5 as "mid-training" for targeted capability injection |

---

## 5. DeepSeek V4.1 Flash

**Source**: HuggingFace README, DeepSeek blog (2026-09-10), tech report PDF

| Dimension | Detail |
|-----------|--------|
| Architecture | Causal Encoder-Decoder (CED) MoE |
| Parameters | 552B total, 8B prefill active / 16B decode active |
| Context | 1M tokens |
| Training | 45T tokens from scratch; sparse attention at 64K, extended to 1M |
| Experts | 384 routed + 1 shared per MoE layer, 6 activated per token |
| KV Cache | 890 bytes/token — 1/4 of DeepSeek-V4-Flash |

### Key Innovations

1. **Causal Encoder-Decoder (CED)**: 20-layer encoder + 20-layer decoder; decoder's KV cache projected from encoder's final hidden states (not each decoder layer) — asymmetric compute: cheap input (8B), rich output (16B)
2. **CSA2 (Compressed Sparse Attention 2)**: Three static modes per layer (Full/Reindex/Reuse); Hierarchical Sparse Indexer bounds deeper indexer cost independent of context length; FP4 KV caching (E2M1 format)
3. **SWA Bounded Replay**: Reconstructs sliding window attention KV by replaying only recent n_win tokens — avoids SSD persistence, 1/8 footprint vs V4-Flash
4. **Engram conditional memory**: 196B parameters, sparsely accessed via token-based lookup — external memory bank separate from transformer layers
5. **DSpark speculative decoding**: Semi-autoregressive draft generation with confidence-scheduled verification
6. **DeepSeek-ViT**: Vision encoder trained from scratch with 2D-RoPE + 3×3 pixel-unshuffle downsampling

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| CED architecture | NT-CORE (core reasoning) | Design asymmetric prefill/decode split in `nt_core::reasoning` — cheap perception, rich generation |
| CSA2 sparse attention | NT-MEMORY (kv_cache) | Implement layered attention modes (Full/Sparse/Reuse) in `kv_cache_optimizer.rs` |
| SWA Bounded Replay | NT-MEMORY | Add bounded replay for sliding window states — avoid full KV persistence |
| Engram memory | NT-MEMORY (KB) | Extend KB with sparsely-accessed conditional memory bank (196B-class external memory) |
| Speculative decoding | NT-IO (inference) | Implement draft-verify speculative decoding in `nt_io::inference_engine` |
| DeepSeek-ViT | NT-WORLD (perception) | Adopt 2D-RoPE + pixel-unshuffle for efficient vision encoding |

---

## 6. Qwen 3

**Source**: arXiv:2505.09388, Qwen blog, GitHub

| Dimension | Detail |
|-----------|--------|
| Architecture | Dense + MoE (both available) |
| Parameters | 0.6B–32B dense; 30B-A3B / 235B-A22B MoE |
| Context | 32K–128K tokens |
| Training | 36T tokens, 119 languages |
| MoE config | 128 experts, 8 activated per token, no shared experts |
| Modes | Thinking + Non-thinking (unified in one model) |

### Key Innovations

1. **Thinking/Non-thinking mode fusion**: Single model with dynamic mode switching via `/think` and `/no_think` tags — eliminates need for separate chat vs reasoning models
2. **Thinking budget mechanism**: User specifies budget (in K tokens); performance scales smoothly and predictably with budget — enables task-adaptive compute allocation
3. **Strong-to-Weak Distillation**: 4-stage pipeline — flagship model generates on-policy data → student mimics logits (KL divergence minimization). Qwen3-4B rivals Qwen2.5-72B-Instruct
4. **Four-stage post-training**: (1) Long CoT cold start → (2) Reasoning RL → (3) Thinking mode fusion → (4) General RL — progressive capability integration
5. **Global-batch load balancing loss**: Encourages expert specialization in MoE (replaces per-sample loss)
6. **119 languages**: Expanded from 29 in Qwen2.5 via cross-lingual transfer

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Mode fusion | NT-CORE (E8 + GWT) | Implement dynamic mode switching in `nt_core::reasoning` — `/think` routes to deep E8 path, `/no_think` to fast GWT path |
| Thinking budget | NT-CORE | Budget-aware compute allocation — map to GWT salience weights |
| Strong-to-Weak distillation | NT-MIND (distillation) | Implement 4-stage distillation pipeline in `nt_mind::skill_crystallization` |
| Four-stage post-training | NT-MIND (SEAL) | Model SEAL pipeline stages after Qwen3's 4-phase approach |
| Global-batch load balancing | NT-CORE (MoE routing) | Adopt global-batch loss for expert routing in any MoE-style subsystem |
| Multilingual | NT-IO (interface) | Extend language support in `nt_io::multilingual` from 29 to 119+ languages |

---

## 7. Mistral Large 3

**Source**: Mistral AI blog, HuggingFace README, technical documentation (Dec 2025)

| Dimension | Detail |
|-----------|--------|
| Architecture | Granular Mixture-of-Experts transformer |
| Parameters | 675B total, 41B active |
| Context | 256K tokens |
| Training | 3000× H200 GPUs, from scratch |
| Vision | Native 2.5B vision encoder (fused) |
| Precision | BF16, FP8, NVFP4 formats |
| Modalities | Text + image → text |

### Key Innovations

1. **Granular MoE**: "Granular" = fine-grained expert segmentation with very high total/active ratio (~16:1) — maximizes knowledge capacity while keeping inference cost at 40-50B dense equivalent
2. **Native vision encoder (2.5B)**: Tightly fused into model backbone — enables OCR, document understanding, visual Q&A without external adapters
3. **NVFP4 deployment**: Full 675B model on single 8×H100 node via 4-bit quantization — practical deployment for open-weight model
4. **Speculative decoding**: Partnered with NVIDIA for draft-verify inference acceleration
5. **Prefill/decode disaggregation**: NVIDIA Blackwell-optimized serving with separated prefill and decode phases

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Granular MoE | NT-CORE (GWT) | Adopt fine-grained expert segmentation for GWT specialist modules — maximize capacity per activation |
| Native vision | NT-WORLD (perception) | Integrate 2.5B-class vision encoder into `nt_world::vision_hub` as first-class modality |
| NVFP4 quantization | NT-IO (inference) | Support FP4/FP8 quantization paths in inference engine |
| Speculative decoding | NT-IO | Implement draft-verify in `nt_io::inference_engine` |
| Disaggregated serving | NT-IO (server) | Separate prefill/decode phases in web server for better resource utilization |

---

## 8. Phi-4 Reasoning

**Source**: arXiv:2504.21318, Microsoft Research, Azure Model Card

| Dimension | Detail |
|-----------|--------|
| Architecture | Dense decoder-only transformer (same as Phi-4) |
| Parameters | 14B |
| Context | 32K tokens (extended from 16K via RoPE frequency doubling) |
| Training | SFT on 1.4M prompts + RL (GRPO) on 6K math problems |
| Tokens | 8.3B unique tokens, 16B total training tokens |
| Teacher | o3-mini (medium effort) for reasoning trace generation |

### Key Innovations

1. **"Teachable" prompt curation**: Prompts filtered for optimal complexity — at the boundary of base model capability. Not too easy (no learning), not too hard (no signal)
2. **Reasoning token repurposing**: Two placeholder tokens → `<think>` / `</think>` markers. Simple but effective — separates reasoning trace from final answer
3. **RoPE frequency doubling**: Base frequency doubled to extend context from 16K → 32K. Minimal architectural change, large capability gain
4. **Outcome-based RL (GRPO)**: Group Relative Policy Optimization on ~6K math problems — even small RL dataset (6,400 problems) significantly improves accuracy
5. **Reasoning as transferable meta-skill**: SFT on reasoning improves general benchmarks (not just STEM) — 30-60% gains on algorithmic/planning tasks never seen in training
6. **SFT data additivity**: Domains can be individually optimized then combined without interference — modular data curation

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Teachable prompts | NT-MIND (SEAL) | Implement prompt complexity scoring in SEAL exploration phase — select "edge of capability" examples |
| Reasoning tokens | NT-CORE (E8) | Add `<think>` / `</think>` markers to E8 hexagram reasoning — separate reasoning trace from conclusion |
| RoPE frequency doubling | NT-MEMORY (kv_cache) | Support dynamic RoPE base frequency scaling for context extension |
| GRPO RL | NT-MIND (RL) | Implement outcome-based RL with group relative rewards in SEAL Phase-4 |
| Transferable reasoning | NT-CORE | Validate that E8 reasoning improvements transfer across domains |
| Modular data curation | NT-MIND | Adopt additive domain optimization in skill crystallization pipeline |

---

## 9. Yi-Lightning

**Source**: arXiv:2412.01253, 01.AI technical report (Dec 2024)

| Dimension | Detail |
|-----------|--------|
| Architecture | Enhanced MoE transformer |
| Parameters | Undisclosed (fine-grained MoE) |
| Context | Not specified (long-context optimized) |
| Training | Multi-stage pre-training + SFT + RLHF |
| Parallelism | Expert parallelism + pipeline parallelism + context parallelism |
| FP8 | 1,200 TFLOPS/card on Hopper GPUs |

### Key Innovations

1. **Partitioned EP load balancing (PEP)**: Three-level load balancing hierarchy: Switch-Transformer (per-expert) → EP group balancing → PEP (partition-level). Solves All-to-All communication imbalance that plagues fine-grained MoE
2. **Hybrid attention blocks**: 3 sliding window attention + 1 full attention per block — captures local patterns + global dependencies. 82.8% memory reduction
3. **Cross-layer KV cache reuse**: Share KV states between consecutive full attention layers — halves memory for full attention components
4. **RAISE safety engine**: 4-component framework across pre-training (RAISE-1: content filtering), post-training (RAISE-2: reward engineering), input (RAISE-3: prompt analysis), output (RAISE-4: real-time detection)
5. **Hardware-aware FP8 design**: Architecture designed for FP8 quantization compatibility from ground up — not bolted on

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| PEP load balancing | NT-CORE (GWT routing) | Implement 3-level routing balance in GWT — per-module → per-domain → per-partition |
| Hybrid attention | NT-MEMORY (kv_cache) | Adopt sliding window + full attention pattern — reduce KV cache for long sessions |
| Cross-layer KV reuse | NT-MEMORY | Share KV states between similar attention layers — 50% memory reduction |
| RAISE safety | NT-SHIELD (Rev-明) | Formalize 4-phase safety pipeline: pre-input → in-context → post-output → cross-session |
| Hardware-aware design | NT-PHYSICAL | Align subsystem designs with target hardware characteristics (GPU architecture awareness) |

---

## 10. Grok 3

**Source**: xAI blog (2025-02-19), AI/TLDR specs, Perplexity analysis, TechTarget

| Dimension | Detail |
|-----------|--------|
| Architecture | Transformer-based (MoE implied, undisclosed) |
| Parameters | Undisclosed (est. ~1.5T) |
| Context | 131K tokens (API); 1M claimed capability |
| Training | Colossus supercluster (~200K H100 GPUs), 10x compute of Grok 2 |
| Modes | Think (reasoning) + DeepSearch (agentic research) |
| RL | Large-scale reinforcement learning for chain-of-thought refinement |
| Modalities | Text (API); image/video understood via multimodal training |

### Key Innovations

1. **Think mode with temporal scaling**: Reasoning duration from seconds to minutes — model self-corrects, explores alternatives, backtracks. Dynamic test-time compute allocation
2. **DeepSearch agent**: Real-time web + X corpus search → synthesis → reasoning → report. Combines retrieval-augmented generation with multi-step reasoning
3. **Cross-expert attention gates** (from architecture analysis): Allow knowledge sharing between MoE experts without catastrophic interference — novel routing mechanism
4. **10x compute scaling**: Colossus cluster at unprecedented scale; validates brute-force compute + RL as path to reasoning
5. **Hierarchical memory layers**: Extended context coherence through layered memory management
6. **Neuro-symbolic integration** (from analysis): Combines transformer pattern matching with structured reasoning modules

### NeoTrix Mapping

| Innovation | NeoTrix Subsystem | Action |
|-----------|-------------------|--------|
| Think mode scaling | NT-CORE (E8) | Implement E8 reasoning with temporal budget — seconds for simple, minutes for complex |
| DeepSearch agent | NT-WORLD (crawl) + NT-ACT | Build `nt_world::deep_search` — real-time retrieval + reasoning + synthesis pipeline |
| Cross-expert gates | NT-CORE (GWT) | Add attention gates between GWT specialist modules for knowledge sharing |
| 10x compute | NT-MIND (SEAL) | Validate SEAL pipeline at scale — ensure quality scales with compute budget |
| Hierarchical memory | NT-MEMORY (KB) | Implement layered memory: session → episode → semantic → episodic |
| Neuro-symbolic | NT-CORE (E8 + HyperCube) | Strengthen E8 ↔ HyperCube symbolic reasoning pathway |

---

## Cross-Model Innovation Matrix

| Innovation Pattern | Models | NeoTrix Priority |
|-------------------|--------|-----------------|
| **Thinking budget / temporal scaling** | Gemini 2.5, Qwen 3, Grok 3, Phi-4 | **P0** — implement in NT-CORE E8 reasoning |
| **MoE with fine-grained routing** | Gemini 2.5, Llama 4, DeepSeek V4.1, Yi-Lightning, Mistral 3, Qwen 3 | **P0** — GWT salience routing as MoE analog |
| **KV cache compression** | DeepSeek V4.1 (CSA2/FP4), Yi-Lightning (hybrid attention), Llama 4 (iRoPE) | **P1** — extend `kv_cache_optimizer.rs` |
| **End-to-end multimodal** | GPT-4o, Llama 4, Mistral 3 | **P1** — fuse perception pipeline |
| **Strong-to-Weak distillation** | Gemini 2.5 (k-sparse), Qwen 3 (4-stage), Phi-4 (teacher-student) | **P1** — NT-MIND distillation pipeline |
| **Agentic tool use** | Claude 3.5 (computer use), Grok 3 (DeepSearch), Gemini 2.5 | **P1** — NT-ACT orchestration |
| **Reasoning as transferable skill** | Phi-4 (meta-skill), Qwen 3 (mode fusion) | **P2** — validate cross-domain transfer |
| **Safety frameworks** | Claude 3.5 (RSP), Yi-Lightning (RAISE), Gemini 2.5 (eval suite) | **P2** — NT-SHIELD multi-phase |
| **Speculative decoding** | DeepSeek V4.1 (DSpark), Mistral 3 | **P2** — NT-IO inference acceleration |
| **Hardware-aware design** | Yi-Lightning (FP8-native), Mistral 3 (NVFP4) | **P3** — NT-PHYSICAL alignment |

---

## Architectural Convergence Observations

1. **MoE is universal**: 8/10 models use or imply MoE. The debate is over — sparse activation is the dominant paradigm for scaling.

2. **Thinking budget is the new API parameter**: Gemini 2.5, Qwen 3, Grok 3 all expose thinking budget as user control. NeoTrix should adopt this for GWT salience modulation.

3. **Context windows are diverging**: 128K (GPT-4o) → 256K (Mistral) → 1M (Gemini/DeepSeek) → 10M (Llama 4). NeoTrix must support tiered memory architecture.

4. **Distillation is the efficiency frontier**: Every major lab distills from flagship to smaller models. Qwen3-4B rivals Qwen2.5-72B. Phi-4 (14B) rivals DeepSeek-R1 (671B). NT-MIND must formalize this.

5. **End-to-end multimodal wins**: GPT-4o's 232ms latency proves joint training beats staged pipelines. NT-WORLD should collapse its perception stack.

6. **RL is the post-training dominant**: GRPO (Phi-4), outcome-based RL (Qwen 3), large-scale RL (Grok 3) — reinforcement learning from verifiable rewards is the standard alignment approach.

---

## Source References

| Model | Primary Source |
|-------|---------------|
| GPT-4o | arXiv:2410.21276, arXiv:2504.02782, MLSysReview |
| Claude 3.5 Sonnet | Anthropic Model Card Addendum (2024-06, 2024-10) |
| Gemini 2.5 Pro | arXiv:2507.06261, Google DeepMind |
| Llama 4 Scout | Meta AI blog, GitHub MODEL_CARD.md |
| DeepSeek V4.1 Flash | HuggingFace README, DeepSeek blog (2026-09-10) |
| Qwen 3 | arXiv:2505.09388, Qwen blog |
| Mistral Large 3 | Mistral AI blog, HuggingFace, Tech Documentation |
| Phi-4 Reasoning | arXiv:2504.21318, Microsoft Research |
| Yi-Lightning | arXiv:2412.01253, 01.AI |
| Grok 3 | xAI blog, AI/TLDR, Perplexity analysis |
