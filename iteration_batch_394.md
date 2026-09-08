# Iteration Batch 394 — Video Generation / Editing / Understanding Research

**Date**: 2026-09-06
**Focus**: Text-to-video generation, video editing, video understanding — 2026 advances
**Research Sources**: 15 papers/sources across 3 domains

---

## Sources Cited

### Text-to-Video Generation
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S1 | StreamDiT (Kodaira et al., 2026) | CVPR 2026 | Real-time streaming T2V at 16 FPS on single H100; moving buffer + progressive denoising + multistep distillation; 4B-param model |
| S2 | EditStream (2026) | arXiv 2608.21424 | Unified autoregressive framework: T2V/I2V/V2V/editing/reference-guided in one DiT; two-stage distillation (VMM + AR unrolling) |
| S3 | BiVidGen (2026) | arXiv 2608.14043 | MLLM-DiT fusion: MLLM generates discrete semantic visual tokens → DiT renders via multi-layer cross-attention; semantic planning beyond text |
| S4 | GEARS (2026) | SIGGRAPH Asia 2026 | Test-time scaling for video diffusion; diagnosis-guided candidate recycling turns failed samples into editable priors; 1.3B model matches 14B |
| S5 | LoViC (Jiang et al., 2026) | CVPR 2026 | Long video via context compression; FlexFormer compresses arbitrary-length context; Interpolated RoPE; prediction/retrodiction/interpolation/multi-shot |
| S6 | LoL (Cui et al., 2026) | CVPR 2026 | Sink-collapse mitigation via multi-head RoPE jitter; first 12-hour continuous streaming video; training-free RoPE fix for autoregressive models |
| S7 | Vidu S1 (2026) | arXiv 2607.03118 | Real-time interactive T2V with voice control; 42 FPS at 540p on consumer GPU; infinite-length without blur/drift; TurboDiffusion + TurboServe |
| S8 | ReasonDiff (Pan et al., 2026) | CVPR 2026 | OOD unpaired text-image→video; VisionNarrator MLLM module for temporal reasoning over unpaired inputs; AlignFormer with temporal anchor attention |

### Video Editing
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S9 | DreamStyle (Li et al., 2026) | CVPR 2026 | Unified video stylization: text/style-image/first-frame-guided in one model; token-specific LoRA; scalable data curation pipeline |
| S10 | VideoCoF (Yang et al., 2026) | CVPR 2026 | Chain-of-Frames: "see→reason→edit" with reasoning tokens predicting edit regions; no masks needed; RoPE alignment for 16× length extrapolation |
| S11 | VideoPainter (2026) | arXiv 2503.05639 | Dual-branch inpainting: lightweight context encoder (6% params) + inpainting region ID resampling for any-length video; 390K+ clip dataset |
| S12 | NOVA (Pan et al., 2026) | CVPR 2026 | Sparse Control + Dense Synthesis: multi-keyframe guidance + source detail injection; pair-free training; consistency-aware keyframe editing |
| S13 | RFDM (Salehi et al., 2026) | CVPR 2026 | Residual flow diffusion for video editing: causal, variable-length, per-frame I2V backbone with temporal residual prediction; competitive with 3D V2V |
| S14 | EchoStyle (Li et al., 2026) | ECCV 2026 | Reverse data synthesis for video stylization; init-follow-mode for long video; V-Style20k dataset (20K paired videos); open-source |

### Video Understanding
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S15 | CapQuiz (2026) | ACL 2026 | Reference-free video caption evaluation via MCQ answering; CapF1 = factuality × coverage; 1,204 videos, 23K QA pairs, 10 question types |
| S16 | DyLaR (2026) | arXiv 2608.04124 | Dynamic Latent Reasoning: perception latents → adaptive reasoning latents; 18.5 tokens/query vs 1,220 CoT; +4.2% accuracy across 9 benchmarks |
| S17 | EGAgent (Rege et al., 2026) | ACL 2026 | Agentic long video understanding via entity scene graphs; multi-hop cross-modal reasoning; 50+ hour video; SOTA on EgoLifeQA |
| S18 | EgoGraph (2026) | arXiv 2602.23709 | Training-free temporal knowledge graph for egocentric video; entity schema + temporal relational modeling across days; SOTA on EgoLifeQA |
| S19 | R4DSG (2026) | ACM MM 2026 | Relative 4D scene graph memory for object-centric QA in long egocentric video; anchor-relative transitions; +6.7% over EgoRAG-Text |
| S20 | POVQA (Dahal et al., 2026) | CVPRW 2026 | 1fps pooled images for long-video VQA; SFT + DPO; rationale supervision; 64.7% zero-shot transfer to TVQA |

---

## Defects Found

### DEFECT-394-1: No Streaming Video Generation Pipeline [HIGH]
**Design Gap**: NT-IO's `video_generation` module (referenced in `evolution-roadmap.md:60` as `nt_io_hyperframes`) has no streaming/inference-time generation capability. The current design assumes batch offline generation.

**2026 Evidence**: StreamDiT (S1) achieves real-time 16 FPS at 512p on a single H100 via moving buffer + progressive denoising + multistep distillation. Vidu S1 (S7) reaches 42 FPS at 540p on consumer GPUs using TurboDiffusion + TurboServe. LoL (S6) demonstrates 12-hour continuous streaming with RoPE jitter to prevent sink-collapse. EditStream (S2) unifies T2V/I2V/V2V/editing in a single autoregressive DiT with two-stage distillation.

**Root Cause**: The design doc treats video generation as a stateless function (prompt → video) rather than a streaming stateful process with moving context buffers, KV-cache management, and progressive denoising schedules.

**Suggestion**: Implement a `StreamingVideoGenerator` trait in NT-IO with: (a) moving buffer of denoising frames, (b) multistep distillation for reduced NFE, (c) RoPE jitter for infinite-length stability, (d) consumer-GPU optimization path (TurboServe-style).

---

### DEFECT-394-2: No MLLM-Guided Semantic Planning for Video [MEDIUM]
**Design Gap**: NT-CORE's reasoning pipeline uses E8 hexagram lattice + GWT attention for text-based reasoning. There is no mechanism for an MLLM to generate semantic visual tokens that guide DiT-based rendering.

**2026 Evidence**: BiVidGen (S3) shows that discrete semantic visual tokens from an MLLM, injected via multi-layer cross-attention into a DiT, significantly improve semantic alignment and temporal coherence over text-only conditioning. The key insight: text is insufficient for high-level video planning; explicit visual-token conditioning via cross-attention provides structured guidance.

**Root Cause**: The architecture treats "understanding" and "generation" as separate pipelines. NT-CORE reasons in text/E8-space, NT-IO generates video, but there is no bridge where understanding produces structured visual tokens that guide generation.

**Suggestion**: Add a `SemanticVisualPlanner` component at the L5 cognition layer that: (a) accepts MLLM hidden states, (b) produces discrete semantic visual tokens via EMA-based tokenizer, (c) injects them into DiT generation via cross-attention conditioning. This bridges NT-CORE reasoning with NT-IO video generation.

---

### DEFECT-394-3: No Test-Time Scaling for Video Generation [MEDIUM]
**Design Gap**: The SEAL pipeline has no test-time compute scaling mechanism. Once a model is trained, inference is fixed-cost. There is no diagnosis → recycling → refinement loop at inference time.

**2026 Evidence**: GEARS (S4) achieves 1.3B model performance comparable to 14B via diagnosis-guided candidate recycling — turning failed generation samples into editable priors rather than discarding them. This is a training-free improvement that doubles effective model capacity at inference cost.

**Root Cause**: The evolution roadmap (C0-C6 maturity) focuses on training-time evolution but has no inference-time adaptation. The `SelfTest` mechanism (T1-T3) detects problems but does not fix them at inference.

**Suggestion**: Add `InferenceAdaptation` to the SEAL pipeline: (a) Stage-Aware Scheduler that evaluates generated candidates, (b) Candidate Recycler that diagnoses recoverable failures via multi-dimensional reward feedback, (c) manifold-aware latent SDEdit for repair. This is training-free and can be added as an inference wrapper.

---

### DEFECT-394-4: No Unified Video Editing Framework [HIGH]
**Design Gap**: NT-ACT's editing capabilities are fragmented across `shot_continuity` (temporal) and `resource_budget` (cost), with no unified editing model. The `VideoCoF`-style "see→reason→edit" paradigm is completely absent.

**2026 Evidence**: VideoCoF (S10) achieves SOTA on video editing with only 50K training pairs by enforcing a Chain-of-Frames reasoning process: first predict edit region (reasoning tokens), then execute edit. DreamStyle (S9) unifies text/style-image/first-frame stylization in one model via token-specific LoRA. NOVA (S12) decouples sparse control (semantic guidance) from dense synthesis (source preservation) for pair-free training.

**Root Cause**: The design assumes editing is a post-processing step on generated video. The 2026 paradigm treats editing as an integrated reasoning process where the model must understand WHAT to edit before HOW to edit, using temporal reasoning tokens.

**Suggestion**: Implement a `UnifiedVideoEditor` in NT-ACT with: (a) Chain-of-Frames reasoning (reasoning tokens → edit region → edit execution), (b) Sparse Control + Dense Synthesis dual-branch architecture, (c) Token-specific LoRA for multi-condition support, (d) RoPE alignment for length extrapolation beyond training duration.

---

### DEFECT-394-5: No Entity-Graph Temporal Reasoning for Video [HIGH]
**Design Gap**: NT-MEMORY's `character_interaction_graph` manages character arcs for dynamic comics but has no temporal entity-graph reasoning for video understanding. There is no mechanism for tracking entities, relationships, and habits across multi-day video streams.

**2026 Evidence**: EGAgent (S17) achieves SOTA on EgoLifeQA (57.5%) by constructing entity scene graphs from audio transcripts + scene descriptions, then using agentic planning for multi-hop cross-modal reasoning. EgoGraph (S18) builds training-free temporal knowledge graphs with cross-entity dependencies accumulated across days. R4DSG (S19) introduces relative 4D scene graph memory with anchor-relative transitions for object-state reasoning.

**Root Cause**: The `character_interaction_graph` is designed for narrative fiction (dynamic comics), not real-world video understanding. It lacks: temporal annotation (timestamps on edges), cross-day persistence, habit inference, and multi-hop query decomposition.

**Suggestion**: Extend NT-MEMORY with `TemporalEntityGraph`: (a) Entity schema: person/object/location nodes with temporal attributes, (b) Relationship edges with (t_start, t_end) intervals, (c) Incremental graph construction from video stream, (d) LLM-based temporal reasoning with relative time expressions, (e) Integration with PerceptionBridge for visual grounding.

---

### DEFECT-394-6: No Adaptive Latent Reasoning for Video QA [MEDIUM]
**Design Gap**: NT-CORE's PerceptionBridge filters sensory events via `awareness_score()` but provides no mechanism for adaptive reasoning allocation. Every query gets the same processing depth regardless of complexity.

**2026 Evidence**: DyLaR (S16) shows that adaptive routing (perception latents → conditional reasoning latents) reduces token usage from 1,220→18 per query while improving accuracy from 54.0→58.2 across 9 video benchmarks. The key insight: perception-oriented questions need only grounding, while reasoning-oriented questions need additional latent reasoning. POVQA (S20) demonstrates 1fps pooled images + rationale supervision achieves strong long-video VQA under strict context budgets.

**Root Cause**: The consciousness architecture allocates attention uniformly via GWT. There is no mechanism to skip reasoning for simple perception tasks or allocate extra reasoning budget for complex inference tasks.

**Suggestion**: Add adaptive routing to the PerceptionBridge: (a) Ground question in perception latents (query-relevant visual evidence), (b) Decision token determines whether reasoning latents are needed, (c) Reasoning latents only generated when question requires temporal/relational inference, (d) Rationale supervision for latent reasoning quality.

---

### DEFECT-394-7: No Reference-Free Caption Quality Evaluation [LOW]
**Design Gap**: NT-MEMORY generates video captions for KB storage but has no mechanism to evaluate caption quality without reference answers. The standard metrics (BLEU/CIDEr) suffer from the one-to-many problem.

**2026 Evidence**: CapQuiz (S15) introduces reference-free evaluation via MCQ answering: a caption is good if you can answer fine-grained questions about the video from it alone. CapF1 = CapP (factuality) × CapR (coverage). 10 question types across 24 domains provide diagnostic analysis. MovieRecapsQA shows vision-centric questions yield the lowest scores, indicating visual perception is the primary bottleneck.

**Root Cause**: The KB captioning pipeline (NT-MEMORY) generates captions but has no feedback loop for quality assessment. Without evaluation, there is no signal for improvement.

**Suggestion**: Add `CaptionQualityEvaluator` to NT-MEMORY: (a) Generate MCQ from video via VLM, (b) Answer MCQ from caption only, (c) Compute CapF1 (factuality × coverage), (d) Route low-quality captions for regeneration. This creates a self-improving captioning loop.

---

## Priority Matrix

| Priority | Defect | Effort | Impact |
|----------|--------|--------|--------|
| P1 | DEFECT-394-5 (Entity Graph) | Medium | High — enables multi-day video understanding |
| P1 | DEFECT-394-4 (Unified Editing) | High | High — fills major capability gap |
| P1 | DEFECT-394-1 (Streaming Gen) | High | High — enables real-time applications |
| P2 | DEFECT-394-2 (Semantic Planning) | Medium | Medium — improves generation quality |
| P2 | DEFECT-394-6 (Adaptive Reasoning) | Low | Medium — efficiency + accuracy |
| P2 | DEFECT-394-3 (Test-Time Scaling) | Medium | Medium — training-free quality boost |
| P3 | DEFECT-394-7 (Caption Eval) | Low | Low — quality feedback loop |

---

## Cross-Cutting Patterns

1. **Streaming is the new baseline**: StreamDiT, Vidu S1, LoL, EditStream all converge on real-time streaming as the default generation mode. NeoTrix's batch-only assumption is obsolete.

2. **MLLM as planner, DiT as renderer**: BiVidGen, EditStream, VideoCoF all use MLLMs for high-level semantic planning and DiT for low-level rendering. NeoTrix's E8/GWT reasoning operates in a separate namespace from video generation.

3. **Entity graphs replace clip retrieval**: EGAgent, EgoGraph, R4DSG all move from unstructured clip retrieval to structured entity graphs with temporal annotations. NeoTrix's KB uses embeddings, not structured entity graphs for video understanding.

4. **Adaptive reasoning over uniform processing**: DyLaR, POVQA, CapQuiz all show that allocating reasoning resources per-query improves both efficiency and quality. NeoTrix's uniform attention allocation is wasteful.

5. **Training-free improvements dominate**: GEARS (test-time scaling), LoL (RoPE jitter), EgoGraph (training-free graph construction) — many 2026 advances require no retraining, only architectural wrappers.
