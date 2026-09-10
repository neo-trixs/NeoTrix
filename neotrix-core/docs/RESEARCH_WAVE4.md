# RESEARCH WAVE 4 — Deep Dive: Architecture, Training, Reasoning, Memory, Multimodal, Efficiency

**Generated**: 2026-09-10 | **Searches**: 48 parallel queries | **Sources**: 200+ papers/blogs/docs

---

## BATCH 1: Model Architecture Deep Dive

### 1. LLaMA 3 Architecture — Dense Transformer with GQA
- **Technique**: Standard decoder-only transformer with Grouped-Query Attention (GQA), SwiGLU activation, RoPE positional embeddings
- **Source**: Meta LLaMA 3 Technical Report (arXiv:2407.21783)
- **Innovation**: 128K vocabulary (4x more efficient than LLaMA 2), GQA across all sizes for inference efficiency, 15T+ training tokens with 4x more code
- **Code Sketch**: `GQA: n_kv_heads=8 for 8B/70B, RoPE base=500000, SwiGLU FFN`
- **NT-Domain**: NT-CORE (attention mechanisms), NT-MEMORY (tokenization)
- **Priority**: P0 — Foundation architecture for NeoTrix models
- **Effort**: Low — well-documented, open weights

### 2. Mixtral 8x7B — Sparse MoE with Top-2 Routing
- **Technique**: Sparse Mixture-of-Experts: 8 experts per layer, top-2 routing per token
- **Source**: Mistral AI (arXiv:2401.04088)
- **Innovation**: 46.7B total params but only 12.9B active per token — 6x faster inference than LLaMA 2 70B at matching quality
- **Code Sketch**: `MoE layer: router(x) -> top2 softmax -> sum(gate_i * expert_i(x))`
- **NT-Domain**: NT-CORE (MoE routing), NT-ACT (efficient inference)
- **Priority**: P0 — MoE pattern for NeoTrix capability routing
- **Effort**: Medium — requires Expert Parallelism for training

### 3. DeepSeek V3 — Auxiliary-Loss-Free MoE + MLA + MTP
- **Technique**: Multi-head Latent Attention (MLA) + DeepSeekMoE + auxiliary-loss-free load balancing + Multi-Token Prediction
- **Source**: DeepSeek-V3 Technical Report (arXiv:2412.19437)
- **Innovation**: 671B total / 37B active; auxiliary-loss-free load balancing eliminates performance degradation from balancing losses; MTP enables speculative decoding; FP8 training
- **Code Sketch**: `MLA: compress KV via low-rank projection; MTP: predict next-N tokens as auxiliary objective`
- **NT-Domain**: NT-CORE (MLA attention), NT-MIND (MTP training)
- **Priority**: P0 — Most innovative architecture; MLA for KV cache compression
- **Effort**: High — complex training infrastructure

### 4. Qwen 2.5 — Fine-Grained MoE + QKV Bias + BBPE
- **Technique**: Dense Transformer with GQA, SwiGLU, RoPE, QKV bias, RMSNorm; MoE variants with fine-grained expert segmentation and shared experts
- **Source**: Qwen2.5 Technical Report (arXiv:2412.15115)
- **Innovation**: 151K BBPE vocabulary, 22 control tokens for tool use, progressive long-context training up to 1M tokens via YARN+DCA, fine-grained expert segmentation
- **Code Sketch**: `BBPE tokenizer with 151K vocab; MoE: 128 experts, 8 active per token`
- **NT-Domain**: NT-IO (tool-use tokens), NT-MEMORY (long context)
- **Priority**: P1 — Best multilingual tokenizer; long context techniques
- **Effort**: Medium

### 5. Gemma 2 — Sliding Window + Attention Softcapping + Distillation
- **Technique**: Decoder-only transformer with interleaved local/global attention, attention logit softcapping (50.0), final logit softcapping (30.0), GeGLU activation
- **Source**: Gemma 2 Technical Report (arXiv:2408.00118)
- **Innovation**: Sliding window (4096) on every other layer + full attention on alternate layers; softcapping prevents attention logit explosion; 2B/9B trained via knowledge distillation from larger models
- **Code Sketch**: `attn_logits = soft_cap * tanh(attn_logits / soft_cap); window=4096 on even layers`
- **NT-Domain**: NT-CORE (attention stability), NT-MIND (distillation)
- **Priority**: P1 — Distillation recipe for small model training
- **Effort**: Low — open weights

### 6. Phi-3 — Data-Optimal Small Model + BlockSparse Attention
- **Technique**: Standard decoder with GQA, SwiGLU, muP hyperparameter transfer, alternating dense/blocksparse attention layers
- **Source**: Microsoft Phi-3 Technical Report (arXiv:2404.14219)
- **Innovation**: 3.8B params rivaling Mixtral 8x7B; data-quality-first approach (filtered web + synthetic); blocksparse attention for KV cache savings; LongRoPE for 128K context
- **Code Sketch**: `Alternating: dense_attn_layer, blocksparse_attn_layer, dense_attn_layer...`
- **NT-Domain**: NT-CORE (efficient attention), NT-ACT (on-device deployment)
- **Priority**: P0 — Blueprint for NeoTrix small models
- **Effort**: Low — phone-deployable at 4-bit

### 7. Claude 3.5 Sonnet — Dense Architecture with Constitutional AI
- **Technique**: Dense transformer decoder-only; 200K context; Constitutional AI (RLHF + CAI)
- **Source**: Anthropic (anthropic.com)
- **Innovation**: Extended thinking mode with internal reasoning chains; tool-use during reasoning; 1M context in Sonnet 5; memory files for long-term tracking
- **Code Sketch**: N/A (proprietary)
- **NT-Domain**: NT-MIND (reasoning), NT-SHIELD (safety alignment)
- **Priority**: P2 — Reference for reasoning architecture
- **Effort**: N/A — proprietary

### 8. Gemini 1.5 Pro — MoE with 10M Token Context
- **Technique**: Sparse MoE transformer with native multimodal input; 128K default, experimental 10M token context
- **Source**: Google DeepMind (arXiv:2403.05530)
- **Innovation**: >99% recall on Needle-in-a-Haystack at 1M tokens; in-context learning from single document; MoE for efficient serving; multimodal native (text+image+audio+video)
- **Code Sketch**: N/A (proprietary)
- **NT-Domain**: NT-WORLD (multimodal perception), NT-MEMORY (ultra-long context)
- **Priority**: P1 — Reference for long-context architecture
- **Effort**: N/A — proprietary

---

## BATCH 2: Training Infrastructure

### 9. Megatron-DeepSpeed — 3D Parallelism Framework
- **Technique**: Combined tensor + pipeline + data parallelism with ZeRO optimization
- **Source**: Microsoft/NVIDIA (github.com/deepspeedai/Megatron-DeepSpeed)
- **Innovation**: MoE training support, curriculum learning, activation checkpointing, distributed optimizer; supports trillion-parameter models
- **Code Sketch**: `deepspeed.initialize(model, optimizer, mpu=mpu) → DeepSpeedEngine`
- **NT-Domain**: NT-CORE (distributed training)
- **Priority**: P0 — Primary training framework
- **Effort**: Medium — well-documented

### 10. FSDP2 (PyTorch) — Per-Parameter Sharding
- **Technique**: DTensor-based dim-0 per-parameter sharding with all-gather/reduce-scatter
- **Source**: PyTorch 2.12+ (docs.pytorch.org)
- **Innovation**: Per-parameter sharding (vs FSDP1's flat-parameter); preserves FQNs for state_dict; supports HSDP (2D mesh); mixed precision policy; offload policy
- **Code Sketch**: `fully_shard(module, mesh=mesh, mp_policy=MixedPrecisionPolicy(param_dtype=bf16))`
- **NT-Domain**: NT-CORE (training infrastructure)
- **Priority**: P0 — Native PyTorch path for NeoTrix training
- **Effort**: Low — first-class PyTorch support

### 11. 3D Parallelism — TP + PP + DP Composition
- **Technique**: Tensor parallelism (intra-node) + pipeline parallelism (inter-node) + data parallelism (across nodes)
- **Source**: Multiple (MegaScale, vTrain, SC24)
- **Innovation**: Nonuniform Tensor Parallelism (NTP) for fault tolerance; synergistic TP+PP schedule reducing both types of bubbles; PipeFill filling pipeline bubbles with other jobs
- **Code Sketch**: `TP=8 (intra-node), PP=16 (inter-node), DP=remaining GPUs`
- **NT-Domain**: NT-CORE (distributed training)
- **Priority**: P1 — For large-scale NeoTrix training
- **Effort**: High — requires cluster infrastructure

### 12. Gradient Checkpointing — Activation Memory Reduction
- **Technique**: Save subset of activations, recompute rest during backward pass
- **Source**: HuggingFace docs, NeMo docs
- **Innovation**: 86% memory reduction when combined with FSDP + LC-CE loss; selective checkpointing for frozen modules; ~20% compute overhead
- **Code Sketch**: `activation_checkpointing="selective"` or `gradient_checkpointing=True`
- **NT-Domain**: NT-CORE (memory optimization)
- **Priority**: P0 — Always enable for NeoTrix training
- **Effort**: Low — one-line config

### 13. FP8 Mixed Precision Training
- **Technique**: FP8 for linear layer GEMMs, BF16 for attention/norms, FP32 for optimizer states
- **Source**: DeepSeek-V3, NVIDIA Transformer Engine, FP8-LM
- **Innovation**: 2x theoretical throughput vs BF16; DeepSeek-V3 achieves stable training at 671B params; end-to-end FP8 with token-level importance sampling for RL; 15-22% real-world speedup
- **Code Sketch**: `FP8 for linear GEMMs, BF16 for attention, FP32 master weights`
- **NT-Domain**: NT-CORE (training efficiency)
- **Priority**: P0 — Use H100/B100 FP8 for NeoTrix training
- **Effort**: Medium — requires Hopper+ GPU

### 14. Training Stability — Loss Spike Mitigation
- **Technique**: AdaGC (adaptive per-tensor gradient clipping), SPAM (spike-aware Adam with momentum reset), dynamic sparsity warm-up
- **Source**: ICML 2026, ICLR 2025, arXiv:2502.11034
- **Innovation**: AdaGC reduces spike scores to zero across Llama-2 7B, Mixtral 8x1B, ERNIE 10B; SPAM resets momentum after spikes; SMET warm-up for newly regrown sparse parameters
- **Code Sketch**: `AdaGC: clip_norm = max(global_clip, ema clipped_norm * z_threshold)`
- **NT-Domain**: NT-CORE (training stability)
- **Priority**: P1 — Critical for large-scale training
- **Effort**: Low — optimizer-agnostic wrapper

### 15. Data Pipeline — NeMo Curator
- **Technique**: GPU-accelerated text curation pipeline: extraction → cleaning → quality filtering → deduplication (exact/fuzzy/semantic) → classification
- **Source**: NVIDIA NeMo Curator (github.com/NVIDIA-NeMo/Curator)
- **Innovation**: RAPIDS-accelerated (cuDF/cuGraph/cuML); 16x speedup on fuzzy dedup; streaming execution with >99% GPU utilization; used for Nemotron-CC (8T+ tokens)
- **Code Sketch**: `Pipeline: TextClean → Dedup(MinHash+LSH) → QualityFilter → DomainClassify`
- **NT-Domain**: NT-WORLD (data acquisition), NT-MEMORY (KB curation)
- **Priority**: P0 — Standard data pipeline for NeoTrix pretraining
- **Effort**: Medium — production-ready

### 16. Curriculum Learning for Pretraining
- **Technique**: Order training data from easy to hard using difficulty scores (Flesch Reading Ease, token count, domain)
- **Source**: EACL 2026, ICLR 2026 (arXiv:2601.21698)
- **Innovation**: CL accelerates convergence by up to 3.5%; incompatibility with LR decay can be fixed via model weight averaging; helps by stabilizing within-phase optimization
- **Code Sketch**: `Sort data by difficulty_score ascending; use quadratic pacing function`
- **NT-Domain**: NT-MIND (training optimization)
- **Priority**: P2 — Experimental optimization
- **Effort**: Low — data ordering change only

---

## BATCH 3: Reasoning & Planning

### 17. Long Chain-of-Thought (Long CoT) Reasoning
- **Technique**: Extended reasoning traces with deep reasoning, revisiting connections, and logical node exploration
- **Source**: Science China 2026 (arXiv:2503.09567)
- **Innovation**: Long CoT integrates three capabilities: deep reasoning, backtracking, and self-reflection; "Aha Moment" emergence during training; overthinking phenomenon where excessive tokens harm accuracy
- **Code Sketch**: N/A — training paradigm
- **NT-Domain**: NT-MIND (reasoning enhancement)
- **Priority**: P0 — Core reasoning paradigm for NeoTrix
- **Effort**: High — requires RL training

### 18. Tree-of-Thought with Novelty-Based Search
- **Technique**: Build thought trees, evaluate novelty of each node, prune branches with low novelty
- **Source**: arXiv:2605.06040
- **Innovation**: Novelty metric estimated via LLM self-knowledge; reduces token cost by pruning redundant branches; outperforms standard ToT on planning benchmarks
- **Code Sketch**: `novelty(node) = 1 - max_similarity(node, all_seen_nodes); prune if novelty < threshold`
- **NT-Domain**: NT-MIND (reasoning), NT-ACT (planning)
- **Priority**: P1 — For complex reasoning tasks
- **Effort**: Medium

### 19. Reasoning Tokens / Extended Thinking
- **Technique**: Models generate internal reasoning traces before final answer; adjustable "reasoning effort" levels
- **Source**: OpenAI o1/o3, DeepSeek-R1, Qwen3-Thinking
- **Innovation**: o3-mini achieves higher accuracy than o1-mini without longer chains (thinks harder, not longer); hybrid reasoning models with configurable effort; ChainPrune removes redundant reasoning steps
- **Code Sketch**: N/A — model capability
- **NT-Domain**: NT-MIND (reasoning), NT-IO (API integration)
- **Priority**: P0 — NeoTrix should support reasoning token streaming
- **Effort**: Medium

### 20. LLM Planning Capabilities & Limitations
- **Technique**: LLM-as-Planner with external verifiers (LLM-Modulo framework)
- **Source**: PlanGenLLMs Survey (ACL 2025), ICML 2024
- **Innovation**: LLMs alone achieve ~12% plan success rate; LLM-Modulo with external verifiers significantly improves; hybrid neuro-symbolic approach (LLM + PDDL planner) is most reliable
- **Code Sketch**: `while not verified: plan = LLM.generate(problem); verified = verifier.check(plan)`
- **NT-Domain**: NT-ACT (agentic planning)
- **Priority**: P1 — For NeoTrix agent capabilities
- **Effort**: High

### 21. Deep-Thinking Ratio (DTR) — Measuring Reasoning Effort
- **Technique**: Track depth-wise stabilization of token predictions across layers; tokens with late stabilization are "deep-thinking tokens"
- **Source**: arXiv:2602.13517 (Google, 2026)
- **Innovation**: DTR correlates with accuracy (r=0.96) across benchmarks; token count is unreliable proxy; enables adaptive compute allocation
- **Code Sketch**: `DTR = count(tokens where prediction stabilizes after layer L) / total_tokens`
- **NT-Domain**: NT-MIND (reasoning monitoring), NT-CORE (attention analysis)
- **Priority**: P1 — Diagnostic tool for NeoTrix reasoning
- **Effort**: Medium

---

## BATCH 4: Memory & Context

### 22. Ultra-Long Context Training (128K → 4M tokens)
- **Technique**: Continued pretraining with YaRN RoPE scaling + special document separators + upsampling long documents
- **Source**: arXiv:2504.06214
- **Innovation**: Progressive extension from 128K → 1M → 2M → 4M tokens; document separators prevent cross-document attention; 1B token training corpus sufficient for extension
- **Code Sketch**: `RoPE_base *= 128 for 1M, *= 256 for 2M, *= 512 for 4M`
- **NT-Domain**: NT-MEMORY (long context)
- **Priority**: P0 — NeoTrix needs ultra-long context
- **Effort**: Medium — Megatron-LM based

### 23. LongRoPE — Extending Context to 2M+ Tokens
- **Technique**: Non-uniform positional interpolation with progressive extension strategy
- **Source**: Microsoft Research (ICML 2024)
- **Innovation**: 8x extension without fine-tuning; 2048K context with only 1K fine-tuning steps; readjust on 8K to recover short-context performance
- **Code Sketch**: `LongRoPE: search for optimal non-uniform interpolation factors per dimension`
- **NT-Domain**: NT-MEMORY (positional encoding)
- **Priority**: P1 — For extending NeoTrix context
- **Effort**: Medium

### 24. RAG vs Long Context — Self-Route Hybrid
- **Technique**: Route queries to RAG or long-context based on model self-reflection
- **Source**: EMNLP 2024 (arXiv:2407.16833)
- **Innovation**: LC consistently outperforms RAG when resourced sufficiently; Self-Route reduces cost by 39-65% while matching LC performance; RAG remains cost-effective for simple retrieval
- **Code Sketch**: `if model.can_answer_from_retrieved(query, chunks): return RAG_answer else: return LC_answer`
- **NT-Domain**: NT-MEMORY (retrieval), NT-ACT (routing)
- **Priority**: P0 — NeoTrix should implement Self-Route
- **Effort**: Low

### 25. LIGHT — Multi-System Long-Term Memory
- **Technique**: Three complementary memory systems: episodic (long-term), working (short-term), scratchpad (salient facts)
- **Source**: ICLR 2026 (arXiv:2510.27246)
- **Innovation**: 3.5-12.7% improvement over baselines; episodic memory for old facts, working memory for recent context, scratchpad for accumulated knowledge; scales to 10M token conversations
- **Code Sketch**: `memory = {episodic: KB_retrieval, working: sliding_window, scratchpad: accumulated_facts}`
- **NT-Domain**: NT-MEMORY (architecture), NT-CORE (attention)
- **Priority**: P0 — Directly applicable to NeoTrix memory architecture
- **Effort**: Medium

### 26. Context Parallelism for Long Sequences
- **Technique**: Split sequence dimension across GPUs with ring attention or similar
- **Source**: NVIDIA NeMo (developer.nvidia.com)
- **Innovation**: Mandatory for >32K sequences; scales to 1M tokens on Llama 3 8B; activation recomputation + context parallelism + offloading stack
- **Code Sketch**: `cp_size = num_gpus_for_sequence; split tokens across cp_size GPUs`
- **NT-Domain**: NT-CORE (distributed training)
- **Priority**: P0 — For NeoTrix long-context training
- **Effort**: Medium

---

## BATCH 5: Multimodal Fusion

### 27. Vision-Language Model Architecture Evolution
- **Technique**: Evolution from two-tower → LLM backbone → native multimodal → omni-modal → world-action models
- **Source**: Survey (arXiv:2501.02189, 2026)
- **Innovation**: ERA 3a (2025-2026): native multimodal input with early fusion; ERA 3b: omni-modal unified I/O; ERA 4: world-action models with persistent state
- **Code Sketch**: N/A — architectural taxonomy
- **NT-Domain**: NT-WORLD (multimodal perception)
- **Priority**: P1 — Architecture roadmap for NeoTrix
- **Effort**: N/A — reference

### 28. NEO-ov — Native One-Vision Model
- **Technique**: Encoder-free monolithic architecture; unified autoregressive modeling for single-image, multi-image, video, and spatial intelligence
- **Source**: arXiv:2605.28820
- **Innovation**: Eliminates pre-trained vision encoder; patch embedding directly into LLM; outperforms modular VLMs at 2B and 8B scales; spatial intelligence via native visual modeling
- **Code Sketch**: `image_tokens = patch_embed(image); text_tokens = word_embed(text); joint = transformer(cat(image_tokens, text_tokens))`
- **NT-Domain**: NT-WORLD (native multimodal)
- **Priority**: P1 — For NeoTrix multimodal capability
- **Effort**: High

### 29. TemporalVLM — Video Temporal Reasoning
- **Technique**: Time-aware clip encoder + BiLSTM for global temporal aggregation; segment-wise encoding with spatial + temporal streams
- **Source**: ACL 2026 (fateh-etal-2026-temporalvlm)
- **Innovation**: First LSTM integration in Video-LLMs; outperforms on temporal reasoning (+24.3 R@1 for temporal grounding); IndustryASM dataset for manufacturing
- **Code Sketch**: `clip_features = time_aware_encoder(video_clips); global = BiLSTM(clip_features); answer = LLM(global + query)`
- **NT-Domain**: NT-WORLD (video understanding)
- **Priority**: P1 — For NeoTrix video capabilities
- **Effort**: Medium

### 30. Qwen2.5-Omni — Thinker-Talker Architecture
- **Technique**: Thinker (LLM for text) + Talker (dual-track autoregressive for speech); TMRoPE for time-aligned multimodal positional encoding
- **Source**: arXiv:2503.20215
- **Innovation**: Simultaneous text + speech generation; time-interleaving for video+audio; end-to-end training of both components; streaming output
- **Code Sketch**: `Thinker: hidden = transformer(text+image+audio); Talker: speech_tokens = autoregressive(hidden)`
- **NT-Domain**: NT-IO (multimodal output), NT-WORLD (multimodal input)
- **Priority**: P1 — For NeoTrix speech generation
- **Effort**: High

---

## BATCH 6: Efficiency & Deployment

### 31. Knowledge Distillation for LLMs — AMiD Framework
- **Technique**: α-mixture assistant distribution for distillation; aligns teacher-student via tunable divergence
- **Source**: ICLR 2026 (arXiv:2510.15982)
- **Innovation**: Generalizes forward KL and reverse KL; α-mixture distribution provides optimal trade-off; outperforms both no-assistant and limited-assistant methods
- **Code Sketch**: `loss = D_alpha_beta(mixture(alpha, teacher, student), student)`
- **NT-Domain**: NT-MIND (model compression)
- **Priority**: P0 — For NeoTrix model compression pipeline
- **Effort**: Medium

### 32. RL-Aware Distillation (RLAD)
- **Technique**: Trust Region Ratio Distillation (TRRD) — PPO/GRPO-style likelihood-ratio objective for on-policy distillation
- **Source**: arXiv:2602.22495
- **Innovation**: Distills reasoning traces from RL-trained teachers; outperforms offline distillation and standard GRPO; maintains reasoning capabilities in student models
- **Code Sketch**: `TRRD_loss = ratio * advantage; ratio = pi_student / pi_teacher_mixture`
- **NT-Domain**: NT-MIND (reasoning distillation)
- **Priority**: P1 — For distilling NeoTrix reasoning models
- **Effort**: Medium

### 33. Structured Pruning — Bonsai (Forward-Pass Only)
- **Technique**: Perturbative module importance estimation via random sub-model evaluation; no gradients needed
- **Source**: arXiv:2402.05406 (2026)
- **Innovation**: Operates in ≤24GB memory; outperforms gradient-based methods (LLM-Pruner, LoRAPrune); underdetermined regression for importance estimation
- **Code Sketch**: `importance(module) = regression_coefficients(random_submodel_evaluations)`
- **NT-Domain**: NT-MIND (model compression)
- **Priority**: P1 — For NeoTrix model compression
- **Effort**: Low

### 34. Kernel Fusion — CUDA Optimization
- **Technique**: Combine multiple GPU operations into single kernel; intermediate results stay in registers
- **Source**: NVIDIA Technical Blog (2026)
- **Innovation**: Manual fusion achieves 3x speedup; torch.compile for implicit fusion; cuda.compute for explicit Python-based fusion; CuTe DSL for MoE training kernels (1.3-2x speedup)
- **Code Sketch**: `fused_kernel = fuse(softmax, matmul, activation) → single CUDA launch`
- **NT-Domain**: NT-CORE (inference optimization)
- **Priority**: P1 — For NeoTrix inference performance
- **Effort**: High — requires CUDA expertise

### 35. LLM Kernel Optimization — LLM-based Agents
- **Technique**: Kernel Forge (MCTS-based), KernelPro (micro-profiling tools), multi-agent systems for kernel optimization
- **Source**: arXiv:2607.24762, arXiv:2606.26453
- **Innovation**: Kernel Forge achieves 2.83x on softmax with 50 iterations; KernelPro achieves 2.42x/4.69x/5.30x on Levels 1/2/3; semantic feedback transforms hardware metrics into natural language guidance
- **Code Sketch**: N/A — tooling
- **NT-Domain**: NT-CORE (kernel optimization)
- **Priority**: P2 — For NeoTrix performance optimization
- **Effort**: High

### 36. Edge Deployment — Quantization + On-Device Inference
- **Technique**: 4-bit PTQ for mobile NPUs; NPU-aware adaptive quantization; SIMD-friendly weight packing
- **Source**: EdgeFlow (arXiv:2604.09083), Arm Developer Blog
- **Innovation**: 4.07x cold-start reduction; 4-bit as optimal operating point (30-50% throughput gain); sub-billion models now handle practical tasks; ParetoQ shows 2-bit models learn fundamentally different representations
- **Code Sketch**: `quantize(model, bits=4, scheme='per_channel', calibration_data=val_set)`
- **NT-Domain**: NT-ACT (on-device deployment)
- **Priority**: P0 — For NeoTrix mobile deployment
- **Effort**: Medium

---

## Cross-Cutting Patterns for NeoTrix

| Pattern | Sources | NeoTrix Application |
|---------|---------|-------------------|
| **MoE for Cost-Efficient Scaling** | Mixtral, DeepSeek V3, Qwen 2.5, Gemini 1.5 | NT-CORE capability routing via MoE |
| **MLA for KV Cache Compression** | DeepSeek V3 | NT-MEMORY context management |
| **FP8 Training** | DeepSeek V3, NVIDIA Transformer Engine | NT-CORE training infrastructure |
| **FSDP2 Per-Parameter Sharding** | PyTorch 2.12+ | NT-CORE training foundation |
| **Long CoT + Reasoning Tokens** | o1/o3, DeepSeek-R1, Qwen3 | NT-MIND reasoning enhancement |
| **LIGHT Memory Architecture** | ICLR 2026 | NT-MEMORY multi-system design |
| **Self-Route RAG/LC** | EMNLP 2024 | NT-MEMORY retrieval routing |
| **Knowledge Distillation (AMiD)** | ICLR 2026 | NT-MIND model compression |
| **Structured Pruning (Bonsai)** | arXiv 2026 | NT-MIND model compression |
| **Kernel Fusion (CuTe DSL)** | NVIDIA 2026 | NT-CORE inference optimization |
| **Edge 4-bit Quantization** | EdgeFlow, Arm | NT-ACT mobile deployment |
| **Native Multimodal (NEO-ov)** | arXiv 2026 | NT-WORLD multimodal architecture |
| **Video Temporal Reasoning** | TemporalVLM, V-CORE | NT-WORLD video understanding |

---

## Priority Matrix

### P0 (Implement Now)
1. FSDP2 for NeoTrix training infrastructure
2. FP8 mixed precision on H100/B100
3. Gradient checkpointing (always-on)
4. Knowledge Distillation pipeline (AMiD)
5. Edge 4-bit quantization for mobile
6. Ultra-long context training (1M+ tokens)
7. Self-Route RAG/LC hybrid
8. LIGHT memory architecture
9. Phi-3-style small model blueprint
10. MoE architecture for capability routing

### P1 (Next Quarter)
11. MLA for KV cache compression
12. Long CoT reasoning training
13. AdaGC training stability
14. Structured pruning (Bonsai)
15. Kernel fusion optimization
16. Video temporal reasoning
17. Native multimodal architecture
18. Context parallelism
19. NeMo Curator data pipeline
20. RL-aware distillation (RLAD)

### P2 (Future)
21. Curriculum learning
22. Tree-of-Thought with novelty search
23. LLM-based kernel optimization agents
24. DTR reasoning diagnostics
25. World-action models
