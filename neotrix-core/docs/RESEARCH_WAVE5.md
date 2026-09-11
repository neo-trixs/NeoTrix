# RESEARCH WAVE 5 — Parallel Deep Web Search Results

**Date**: 2026-09-11
**Searches**: 48 parallel searches across 6 batch dimensions
**Status**: Complete

---

## Batch 1: Reverse Engineering & Capability Probing (#1-8)

### #1 NightVision — Black-box inference of LLM architectural properties
- **URL**: https://arxiv.org/abs/2607.01313
- **Key**: Black-box inference of LLM architectural properties (hidden size, depth, layer type) via restrictive API
- **Implication**: Feasible to reconstruct model architecture with limited API access

### #2 Incompressible Knowledge Probes — Estimating parameter counts
- **URL**: https://arxiv.org/abs/2406.17639
- **Key**: Estimate parameter counts via factual capacity measurement; approximate LLM size from output behavior
- **Implication**: No architecture details needed — probe output distribution to estimate scale

### #3 Inside-Out hidden factual knowledge
- **URL**: https://arxiv.org/abs/2503.15299
- **Key**: Internal vs external knowledge measurement; models encode facts internally vs in context
- **Implication**: Hidden representations contain extractable factual knowledge

### #4 Bayesian Prompt Ensembles (ACL 2024)
- **URL**: https://aclanthology.org/2024.findings-emnlp.1282/
- **Key**: Uncertainty estimation for black-box LLMs via prompt ensembles; model-free uncertainty
- **Implication**: Can probe model confidence without weight access

### #5 Scaling Up Membership Inference (NAACL 2025)
- **URL**: https://aclanthology.org/2025.findings-naacl.long.545/
- **Key**: MIA at sentence/paragraph/document/dataset scales; training data membership detection
- **Implication**: Can detect if specific data was used to train the model

### #6 RoFL — Robust fingerprinting of LLMs
- **URL**: https://arxiv.org/abs/2505.12682
- **Key**: Robust fingerprinting of LLMs via black-box identification; model attribution
- **Implication**: Can fingerprint unknown models via API probing

### #7 Software architecture reconstruction
- **URL**: https://ieeexplore.ieee.org/document/1301957
- **Key**: Generic software architecture reconstruction from runtime traces (2004, less relevant)
- **Implication**: Limited direct applicability to LLM-specific RE

### #8 Clone What You Can't Steal
- **URL**: https://arxiv.org/abs/2509.00973
- **Key**: Black-box LLM replication via logit leakage + distillation; functional cloning
- **Implication**: Even without weights, functional cloning is feasible

---

## Batch 2: Breaking Limitations (#9-16)

### #9 Semantic compression for context window extension
- **URL**: https://arxiv.org/abs/2312.09571
- **Key**: 6-8x context window extension via semantic compression; lossy but effective
- **Implication**: Practical technique for handling long-context without full KV cache

### #10 KV Cache Optimization Strategies survey
- **URL**: https://arxiv.org/abs/2603.20397 (Dell, Mar 2026)
- **Key**: 5 directions: eviction, compression, hybrid memory, novel attention, combination
- **Implication**: Comprehensive survey of KV cache management landscape

### #11 Self-Tuning Sparse Attention / AFBS-BO (MiTA 2026)
- **URL**: https://arxiv.org/abs/2603.18417
- **Key**: Automated sparse attention hyperparameters; Bayesian optimization for attention patterns
- **Implication**: Can auto-tune sparse attention without manual tuning

### #12 Scaling Linear Attention with Sparse State Expansion (ICLR 2026)
- **URL**: https://arxiv.org/abs/2601.03666
- **Key**: SSE and SSE-H architectures for linear attention; O(n) complexity
- **Implication**: Viable alternative to quadratic attention for long sequences

### #13 PASTA parallel decoding (MIT CSAIL, Jul 2025)
- **URL**: https://arxiv.org/abs/2506.03096
- **Key**: Parallel structure annotation for LLM inference; ~2x speedup
- **Implication**: Structure-aware parallelism can nearly double throughput

### #14 LLM Inference Optimizations (Argonne, Mar 2026)
- **URL**: https://arxiv.org/abs/2507.04636
- **Key**: GQA, MoE, pruning, quantization, speculative decoding survey
- **Implication**: Comprehensive optimization landscape

### #15 Dynamic batching (memory-aware + SLA-constrained)
- **URL**: https://arxiv.org/abs/2503.05248
- **Key**: 8-28% throughput gains via dynamic batching with memory awareness
- **Implication**: Simple batching optimization yields significant gains

### #16 BOUTE — heterogeneous model+GPU serving
- **URL**: https://arxiv.org/abs/2602.10729
- **Key**: 157% improvement or 15-61% cost reduction via heterogeneous serving
- **Implication**: Match model complexity to GPU capability for cost optimization

---

## Batch 3: Universal Architecture Patterns (#17-24)

### #17 Universal model interface / provider abstraction
- **URL**: https://crates.io/crates/multi_llm
- **Key**: `multi_llm` Rust crate — unified interface across LLM providers
- **Implication**: Existing Rust ecosystem has provider abstraction primitives

### #18 Multi-model orchestration / Pick and Spin (arXiv:2512.22402)
- **URL**: https://arxiv.org/abs/2512.22402
- **Key**: Joint optimization routing across multiple models; cost-quality Pareto
- **Implication**: Multi-model routing is solvable as joint optimization

### #19 Model capability registry / llm-registry
- **URL**: https://pypi.org/project/llm-registry/0.6.3/
- **Key**: llm-registry PyPI package for model capability tracking
- **Implication**: Lightweight registry pattern exists for model capability management

### #20 Cross-model knowledge transfer / Embedding-Converter (ACL 2025)
- **URL**: https://aclanthology.org/2025.acl-long.757/
- **Key**: 100x faster model switching via embedding conversion
- **Implication**: Embedding alignment enables instant model switching

### #21 Model routing optimization — ERoL / MixLLM / Brick
- **Key papers**:
  - **ERoL** (EMNLP 2025 Findings): Exploration-driven RL for MoE expert routing; 8.9x higher MRR
  - **MixLLM** (NAACL 2025): Dynamic contextual-bandit routing; 97.25% of GPT-4 quality at 24.18% cost
  - **Brick** (arXiv:2606.13241): Spatial capability routing for Mixture-of-Models; 76.98% accuracy
  - **SoftMoE** (ICML 2026): Soft differentiable routing; learns layer-wise expert allocation
- **Implication**: Dynamic routing across model pools is mature; contextual-bandit approaches dominate

### #22 MoRA — High-rank updating for PEFT
- **URL**: https://arxiv.org/abs/2405.12130
- **Key**: Square matrix for high-rank updating while maintaining same parameter count; outperforms LoRA on memory tasks
- **Implication**: LoRA rank bottleneck can be overcome without parameter increase

### #23 DiSRouter — Distributed self-routing for LLM selections (ICLR 2026)
- **URL**: https://arxiv.org/abs/2510.19208
- **Key**: Distributed routing via self-awareness; no centralized router needed
- **Implication**: Decentralized routing scales better than centralized approaches

### #24 Knowledge distillation survey (arXiv:2503.12067, Mar 2025)
- **URL**: https://arxiv.org/abs/2503.12067
- **Key**: Comprehensive survey covering KD, quantization, pruning, low-rank factorization
- **Implication**: Distillation remains primary compression method for production deployment

---

## Batch 4: Production & Ops (#24-32)

### #25 MLOps pipeline automation (MLflow)
- **URL**: https://mlflow.org/articles/what-is-canary-deployment-ai
- **Key**: Canary deployment for AI models; progressive rollout with automated rollback
- **Implication**: MLflow provides production-grade canary deployment observability

### #26 Model monitoring/observability/canary deployment (MLflow 2026 guide)
- **URL**: https://mlflow.org/articles/tags/ml-model-deployment-process
- **Key**: Progressive deployment sequence: Shadow → Canary → Champion/Challenger → Full
- **Implication**: Standard deployment pipeline for production ML models

### #27 Model versioning / ML pipeline orchestration (MLflow 2026)
- **URL**: https://mlflow.org/classical-ml/model-registry
- **Key**: Stage-based lifecycle: Development → Staging → Production → Archived; aliases (@champion)
- **Implication**: Model registry is essential for production model management

### #28 MLflow model registry lifecycle management
- **Key features**:
  - Version control with automatic tracking
  - Model lineage and traceability
  - Aliases: `@champion`, `@candidate` for deployment
  - Tags for categorization
  - Stage transitions: Staging → Production → Archived
  - Role-based access controls
- **Implication**: Registry is the backbone of MLOps model lifecycle

### #29 A/B testing for ML models (GitHub project)
- **URL**: https://github.com/krishna4002/A-B-Testing-for-ML-Models-in-Production
- **Key**: Champion vs Challenger A/B testing with MLflow; random selection; SQLite logging
- **Implication**: A/B testing is straightforward with MLflow + FastAPI

### #30 KV cache eviction policies (H2O, StreamingLLM, SnapKV, etc.)
- **Key papers**:
  - **H2O** (NeurIPS 2023): Heavy-Hitter Oracle; 20% cache retention → 29x throughput improvement
  - **StreamingLLM**: Sliding window + attention sinks; simple but effective
  - **SnapKV**: Attention weight-coupled pooling
  - **DefensiveKV** (arXiv:2510.13334): Worst-case risk management for eviction
  - **KV-Direct** (arXiv:2603.19664): Residual stream checkpointing; 100% token match at all budgets
  - **PagedEviction** (EACL 2026 Findings): Block-wise eviction for vLLM
  - **Conf-KV**: Confidence-aware eviction; 91.4% retrieval accuracy at 32K tokens
- **Implication**: KV cache eviction is critical; H2O is baseline; confidence-aware approaches are frontier

### #31 Model pipeline parallel inference
- **URL**: https://docs.vllm.ai/en/latest/serving/parallelism_scaling
- **Key**: vLLM supports tensor parallel (TP) + pipeline parallel (PP); Ray for multi-node
- **Implication**: Production-grade distributed inference is available via vLLM

### #32 Streaming inference optimization / edge serving
- **Key papers**:
  - **HELIOS** (arXiv:2504.10724): Adaptive model + early-exit selection; 1.48x throughput, 1.39x lower latency
  - **StreamServe** (arXiv:2604.09562): Disaggregated prefill-decode with adaptive speculation; 2.4x throughput, 8.8x lower latency
  - **SpecEdge** (NeurIPS 2025): Edge-assisted inference; 1.91x cost efficiency, 2.22x server throughput
  - **LMEdge** (arXiv:2607.17175): QoS-aware orchestration on edge clusters; BILP optimization
- **Implication**: Edge-cloud hybrid inference is viable; speculative decoding at edge reduces costs

---

## Batch 5: Efficiency & Deployment (#33-40)

### #33 Structured pruning (NAACL 2024 Findings)
- **Key**: Importance-based row/column pruning; 94.4% performance at 20% reduction
- **Implication**: Structured pruning is practical for LLM compression

### #34 Activation checkpointing / kernel fusion (Megatron-LM)
- **Key**: Selective recompute, bias_dropout_fusion, apply_rope_fusion; 30-40% memory reduction
- **Implication**: Kernel fusion is essential for LLM training efficiency

### #35 Edge deployment (arXiv:2602.13628)
- **Key**: ECLD framework: pruning + distillation + quantization for MEC
- **Implication**: Combined compression pipeline for edge deployment

### #36 Scalable AI inference — performance analysis
- **URL**: https://arxiv.org/abs/2604.20420
- **Key**: FP16 ONNX delivers lowest latency; adaptive batching in BentoML; K3s deployment
- **Implication**: ONNX + adaptive batching is optimal for production inference

### #37 ML inference scheduling with predictable latency
- **URL**: https://arxiv.org/abs/2512.18725
- **Key**: Interference prediction for concurrent batches; concurrent batch execution reduces HoL blocking
- **Implication**: Interference-aware scheduling is critical for multi-model serving

### #38 Symphony — deferred batch scheduling
- **URL**: https://arxiv.org/abs/2308.07470
- **Key**: Deferred batch scheduling; 20% GPU usage for same workload as baseline
- **Implication**: Deferring batch dispatch improves GPU utilization dramatically

### #39 Paella — software-defined GPU scheduling (SOSP 2023)
- **URL**: https://dl.acm.org/doi/10.1145/3600006.3613163
- **Key**: Co-design compiler + scheduler; bypass built-in GPU scheduler
- **Implication**: Software-defined GPU scheduling enables arbitrary scheduling algorithms

### #40 Sarathi-Serve — chunked prefill scheduling
- **URL**: https://arxiv.org/abs/2403.02310
- **Key**: Chunked-prefills; stall-free scheduling; 2.6x higher serving capacity than vLLM
- **Implication**: Chunked prefill is fundamental for LLM serving efficiency

---

## Batch 6: Multimodal Remaining (#41-48)

### #41 Video temporal modeling (arXiv:2602.00683)
- **Key**: Recurrent adapters, state space layers for video temporal modeling
- **Implication**: Temporal modeling via recurrent adapters is practical for video LLMs

### #42 UniSonate (ACL 2026)
- **Key**: Unified speech/music/sound effect generation via flow-matching
- **Implication**: Flow-matching unifies audio modality generation

### #43 e5-omni (arXiv:2601.03666)
- **Key**: Cross-modal alignment for omni-modal embeddings
- **Implication**: Omni-modal embeddings are practical for retrieval

### #44 NIRVANA (arXiv:2509.14230)
- **Key**: NTK-informed structured pruning for LLM compression
- **Implication**: NTK-aware pruning outperforms uniform pruning

### #45 RT-2 (Google DeepMind)
- **Key**: Vision-language-action model for robotics; cross-modal transfer
- **Implication**: VLA models bridge perception and action

### #46 Cross-modal alignment — AlignCLIP, FuseLIP, COLA
- **Key papers**:
  - **AlignCLIP** (ICLR 2025): Parameter sharing + intra-modality separation reduces modality gap
  - **FuseLIP** (arXiv:2506.03096): Early fusion of discrete tokens; single encoder for all modalities
  - **COLA** (NeurIPS 2025): Optimal transport-based alignment for adversarial robustness
  - **CLIP BoW** (ICLR 2026): CLIP is BoW cross-modally but not uni-modally; linear transform fixes it
- **Implication**: Cross-modal alignment is solvable; early fusion outperforms late fusion

### #47 Omni-modal architecture — jina-v5-omni, Fusion Embedding, Qwen3-Omni
- **Key papers**:
  - **jina-embeddings-v5-omni** (arXiv:2605.08384): GELATO — frozen towers + 0.35% trained connectors
  - **Fusion Embedding** (arXiv:2607.18666): Qwen3-VL-Embedding-2B + frozen audio tower; 16.4M trained params
  - **Qwen3-Omni** (arXiv:2509.17765): Thinker-Talker MoE; 234ms end-to-end latency; 119 languages
  - **InteractiveOmni** (HuggingFace): 4B-8B unified omni-modal model
  - **OmniEncoder** (arXiv:2605.01506): 25fps joint audio-visual encoding; 3D RoPE
  - **vLLM-Omni**: OmniRouter + AR/DiT disaggregation
- **Implication**: Omni-modal models are production-ready; frozen towers + lightweight connectors is the pattern

### #48 Model compression — pruning, quantization, distillation
- **Key papers**:
  - **TACL 2024 Survey**: Comprehensive taxonomy — quantization, pruning, KD, low-rank factorization
  - **STUN** (ACL 2025): Structured-then-unstructured pruning for MoE; 40% sparsity with no loss
  - **Revisiting Pruning vs Quantization** (EMNLP 2025): Quantization consistently outperforms pruning
  - **Prompt Compression Survey** (NAACL 2025): Hard/soft prompt compression methods
  - **Model Compression Survey** (Frontiers 2025): Historical overview from 1980s to modern LLMs
- **Implication**: Quantization > pruning for LLMs; combined approaches yield best compression

---

## Cross-Batch Key Insights

### Model Routing (Batch 3)
- **MixLLM**: 97.25% GPT-4 quality at 24.18% cost via contextual-bandit routing
- **DiSRouter**: Distributed self-routing eliminates centralized bottleneck
- **Brick**: Spatial capability routing outperforms domain-based routing

### KV Cache (Batch 4)
- **H2O**: 20% cache → 29x throughput (NeurIPS 2023, foundational)
- **KV-Direct**: Residual stream checkpointing achieves 100% token match
- **Conf-KV**: Confidence-aware eviction is the frontier

### Scheduling (Batch 5)
- **Sarathi-Serve**: Chunked prefill is fundamental; 2.6x over vLLM
- **FastServe**: Preemptive scheduling via skip-join MLFQ
- **SLAI**: SLO-aware scheduling; 53% median TTFT reduction

### Multimodal (Batch 6)
- **jina-v5-omni**: Frozen towers + 0.35% trained connectors (GELATO)
- **Fusion Embedding**: Qwen3-VL-Embedding-2B + frozen audio; 16.4M params
- **Qwen3-Omni**: Thinker-Talker MoE; 234ms latency; 119 languages

### Compression (Batch 6)
- **Quantization > Pruning** for LLMs (EMNLP 2025)
- **STUN**: Structured→unstructured for MoE; 40% sparsity
- **EI-BERT**: Cross-distillation achieves 1.91MB model (KDD 2025)
- **Progressive²**: Teacher-student co-evolving distillation

### Production MLOps (Batch 4)
- **MLflow Model Registry**: Aliases (@champion), stages, versioning
- **Canary Deployment**: Shadow → Canary → Champion/Challenger → Full
- **A/B Testing**: Champion vs Challenger with MLflow + FastAPI

---

## Raw Search Results

<details>
<summary>Click to expand full search results for all 48 searches</summary>

### Search #1 — NightVision
- URL: https://arxiv.org/abs/2607.01313
- Black-box inference of LLM architectural properties via restrictive API

### Search #2 — Incompressible Knowledge Probes
- URL: https://arxiv.org/abs/2406.17639
- Estimate parameter counts via factual capacity measurement

### Search #3 — Inside-Out hidden factual knowledge
- URL: https://arxiv.org/abs/2503.15299
- Internal vs external knowledge measurement

### Search #4 — Bayesian Prompt Ensembles
- URL: https://aclanthology.org/2024.findings-emnlp.1282/
- Uncertainty estimation for black-box LLMs

### Search #5 — Scaling Up Membership Inference
- URL: https://aclanthology.org/2025.findings-naacl.long.545/
- MIA at multiple scales

### Search #6 — RoFL
- URL: https://arxiv.org/abs/2505.12682
- Robust fingerprinting of LLMs

### Search #7 — Software architecture reconstruction
- URL: https://ieeexplore.ieee.org/document/1301957
- Generic software architecture reconstruction

### Search #8 — Clone What You Can't Steal
- URL: https://arxiv.org/abs/2509.00973
- Black-box LLM replication via logit leakage

### Search #9 — Semantic compression
- URL: https://arxiv.org/abs/2312.09571
- 6-8x context window extension

### Search #10 — KV Cache Optimization survey
- URL: https://arxiv.org/abs/2603.20397
- 5 directions for KV cache optimization

### Search #11 — Self-Tuning Sparse Attention
- URL: https://arxiv.org/abs/2603.18417
- Automated sparse attention hyperparameters

### Search #12 — Scaling Linear Attention
- URL: https://arxiv.org/abs/2601.03666
- SSE and SSE-H for linear attention

### Search #13 — PASTA parallel decoding
- URL: https://arxiv.org/abs/2506.03096
- ~2x speedup via structure-aware parallelism

### Search #14 — LLM Inference Optimizations
- URL: https://arxiv.org/abs/2507.04636
- GQA, MoE, pruning, quantization, speculative decoding

### Search #15 — Dynamic batching
- URL: https://arxiv.org/abs/2503.05248
- 8-28% throughput gains

### Search #16 — BOUTE
- URL: https://arxiv.org/abs/2602.10729
- 157% improvement or 15-61% cost reduction

### Search #17 — multi_llm Rust crate
- URL: https://crates.io/crates/multi_llm
- Unified interface across LLM providers

### Search #18 — Pick and Spin
- URL: https://arxiv.org/abs/2512.22402
- Joint optimization routing

### Search #19 — llm-registry
- URL: https://pypi.org/project/llm-registry/0.6.3/
- Model capability tracking

### Search #20 — Embedding-Converter
- URL: https://aclanthology.org/2025.acl-long.757/
- 100x faster model switching

### Search #21 — Model routing optimization
- ERoL (EMNLP 2025), MixLLM (NAACL 2025), Brick (arXiv:2606.13241), SoftMoE (ICML 2026)
- Dynamic routing across model pools

### Search #22 — MoRA
- URL: https://arxiv.org/abs/2405.12130
- High-rank updating for PEFT

### Search #23 — DiSRouter
- URL: https://arxiv.org/abs/2510.19208
- Distributed self-routing (ICLR 2026)

### Search #24 — Knowledge distillation survey
- URL: https://arxiv.org/abs/2503.12067
- Comprehensive KD survey

### Search #25 — Canary deployment (MLflow)
- URL: https://mlflow.org/articles/what-is-canary-deployment-ai
- Progressive rollout with automated rollback

### Search #26 — Model monitoring/observability
- URL: https://mlflow.org/articles/tags/ml-model-deployment-process
- Shadow → Canary → Champion/Challenger → Full

### Search #27 — Model versioning
- URL: https://mlflow.org/classical-ml/model-registry
- Stage-based lifecycle management

### Search #28 — MLflow model registry
- Version control, aliases, tags, stage transitions

### Search #29 — A/B testing
- URL: https://github.com/krishna4002/A-B-Testing-for-ML-Models-in-Production
- Champion vs Challenger

### Search #30 — KV cache eviction
- H2O, StreamingLLM, SnapKV, DefensiveKV, KV-Direct, PagedEviction, Conf-KV
- H2O: 20% cache → 29x throughput

### Search #31 — Pipeline parallel
- URL: https://docs.vllm.ai/en/latest/serving/parallelism_scaling
- vLLM TP + PP

### Search #32 — Streaming inference / edge
- HELIOS, StreamServe, SpecEdge, LMEdge
- Edge-cloud hybrid inference

### Search #33 — Structured pruning
- Importance-based; 94.4% performance at 20% reduction

### Search #34 — Activation checkpointing / kernel fusion
- Selective recompute; 30-40% memory reduction

### Search #35 — Edge deployment
- ECLD framework: pruning + distillation + quantization

### Search #36 — Scalable AI inference
- URL: https://arxiv.org/abs/2604.20420
- FP16 ONNX + adaptive batching

### Search #37 — ML inference scheduling
- URL: https://arxiv.org/abs/2512.18725
- Interference prediction

### Search #38 — Symphony
- URL: https://arxiv.org/abs/2308.07470
- Deferred batch scheduling; 20% GPU usage

### Search #39 — Paella
- URL: https://dl.acm.org/doi/10.1145/3600006.3613163
- Software-defined GPU scheduling

### Search #40 — Sarathi-Serve
- URL: https://arxiv.org/abs/2403.02310
- Chunked prefill; 2.6x over vLLM

### Search #41 — Video temporal modeling
- Recurrent adapters, state space layers

### Search #42 — UniSonate
- Unified speech/music/sound via flow-matching

### Search #43 — e5-omni
- Cross-modal alignment for omni-modal embeddings

### Search #44 — NIRVANA
- NTK-informed structured pruning

### Search #45 — RT-2
- Vision-language-action model

### Search #46 — Cross-modal alignment
- AlignCLIP, FuseLIP, COLA, CLIP BoW

### Search #47 — Omni-modal architecture
- jina-v5-omni, Fusion Embedding, Qwen3-Omni, InteractiveOmni, OmniEncoder, vLLM-Omni

### Search #48 — Model compression
- TACL 2024 Survey, STUN, Revisiting Pruning vs Quantization, Prompt Compression Survey

</details>
