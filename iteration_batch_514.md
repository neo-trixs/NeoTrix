# Iteration Batch 514 — Model Optimization / Training / Inference Research

**Date**: 2026-09-06
**Research Domain**: Model compression, distributed training, inference optimization
**Defect Analysis**: Gap mapping from external 2026 advances to NeoTrix design

---

## 1. Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | GeniusTechLab — AI Model Compression 2026 | 2026-07-01 | Pruning/quantization/distillation overview, hybrid pipelines |
| S2 | arXiv:2604.04988 — Prune-Quantize-Distill Ordered Pipeline | 2026-04-05 | Ordered compression pipeline research |
| S3 | arXiv:2602.09130 — UniComp: Unified Evaluation of LLM Compression | 2026-02 | Soft pruning vs hard pruning, multi-technique evaluation |
| S4 | Springer J. Electr. Eng. Technol. — Pipeline of PKDQ | 2026-03 | Integrated pruning+distillation+quantization pipeline |
| S5 | NVIDIA Model Optimizer GitHub | 2026-08 | AutoQuantize, Minitron pruning+distillation, NVFP4 |
| S6 | arXiv:2601.09865 — Muon-Optimized Distillation & Quantization | 2026-01-14 | Edge deployment optimization framework |
| S7 | NVIDIA Blog — Pruning and Distilling LLMs with TensorRT Model Optimizer | 2026-05-07 | Minitron pruning + Qwen3 depth pruning results |
| S8 | MyEngineeringPath — LLM Inference Optimization 2026 | 2026-03-20 | Inference optimization pipeline stages |
| S9 | QSCompute — Edge AI Inference Optimization 2026 | 2026-06-25 | TensorRT/ONNX/OpenVINO benchmarks on edge hardware |
| S10 | arXiv:2604.16145 — Training Time Prediction for Mixed Precision Distributed Training | 2026-04-17 | Precision-aware training time prediction |
| S11 | youngju.dev — Distributed Training Deep-Dive 2026 | 2026-05-16 | FSDP2+torch.compile, Megatron-Core, FP8/BF16/MXFP4 |
| S12 | Springer — Efficient Training of LLMs on Distributed Infrastructures | 2026-06-01 | Survey of distributed training optimizations |
| S13 | SEADA — Mixed-Precision DNN Optimization on Multi-Precision Architectures | 2026 | Per-layer precision selection, energy-efficiency co-optimization |
| S14 | TensorRT Docs — Inference Library 2026-08 | 2026-08-04 | FP8/INT4, weight streaming, multi-device inference |
| S15 | ONNX Runtime — Cross-platform Inference | 2026 | Execution providers, on-device training, web deployment |

---

## 2. Defects Found

### DEFECT-MO-001: No Structured Pruning Pipeline (Critical)

**Current State**: `quantization_engine.rs` (L1-36) documents quantization formats extensively (GGUF, GPTQ, AWQ, FP8, NVFP4) but has **zero** pruning logic. The `ContextCompressor` in `nt_core_bank/compressor.rs` compresses context strings and memory entries — not model weights.

**2026 Advance**: NVIDIA's Minitron pipeline (S5, S7) demonstrates structured pruning + distillation achieving 33% smaller models at 50% speed with 90% quality retention. The Prune-Quantize-Distill ordered pipeline (S2, S4) shows the **order matters**: prune → quantize → distill in that sequence yields 20x compression at <3% accuracy loss. UniComp (S3) shows soft pruning (low-rank clone) substantially outperforms hard pruning for multilingual and reasoning tasks.

**Gap**: NeoTrix has no `Pruner` module, no structured/unstructured pruning abstractions, no sparsity pattern tracking (block sparsity, 2:4 patterns for Blackwell), and no prune-then-distill pipeline orchestration.

**Impact**: Users cannot compress models beyond quantization alone. The SEAL evolution pipeline has no mechanism to discover that a model is too large for deployment hardware and trigger a prune+distill cycle.

### DEFECT-MO-002: Missing Mixed-Precision Runtime Awareness (High)

**Current State**: `quantization_engine.rs` (L853-869) generates `MixedPrecisionRule` structs, but these are static rules applied at quantization time — not dynamically adjusted at inference based on input complexity.

**2026 Advance**: SEADA (S13) demonstrates per-layer precision selection using bit-level entropy achieves 57% energy reduction vs 8-bit baseline with zero accuracy loss. The precision-aware training predictor (S10) shows mixed precision causes 2.4x training time variance — ignoring precision in scheduling yields 147.85% MAPE. Adaptive quantization (S1) dynamically adjusts precision based on input complexity at runtime.

**Gap**: NeoTrix quantizes once at static precision. No runtime adaptive quantization that shifts precision per-layer or per-input-difficulty. No precision-aware scheduling for multi-model inference.

**Impact**: 30-50% energy savings are left on the table. Models running on heterogeneous hardware (edge + cloud) cannot adapt precision dynamically.

### DEFECT-MO-003: No Knowledge Distillation Engine (High)

**Current State**: The term "distill" appears extensively in NT-MIND (`nt_mind_distiller.rs`, `control_distillation.rs`, `experience_tree`) but exclusively for **session knowledge distillation** — extracting behavioral patterns from conversation logs. Zero infrastructure exists for **model knowledge distillation** (teacher→student weight transfer).

**2026 Advance**: NVIDIA's Minitron (S5) achieves 2.6x vLLM throughput and 2.6x memory reduction via pruning + two-phase distillation + FP8. Muon-optimized distillation (S6) integrates distillation with quantization-aware training for edge deployment. Bielik 7B (S5) retains 90% quality from 70B teacher via distillation.

**Gap**: No `DistillationTrainer`, no teacher-student model management, no intermediate representation alignment (feature/attention distillation), no multi-teacher ensemble distillation.

**Impact**: Cannot produce optimized smaller models for edge deployment. The NT-PHYSICAL embodiment layer cannot generate lightweight models for physical devices.

### DEFECT-MO-004: Speculative Decoding Incomplete (Medium)

**Current State**: `speculative_decoding.rs` exists in `nt_shield_local_inference` with `SpeculativeDecoder`, `AcceptanceStats`, and `M5SpeculativeBenchmarks`. However, the module is imported but the actual draft model selection and spec_decode_args implementation appear minimal.

**2026 Advance**: Speculative decoding is now a standard optimization in serving frameworks (S8 lists it as "Month 1+ optimization"). NVIDIA's Nemotron 3 Super quantized checkpoints (S5) are specifically optimized for speculative decoding with draft models.

**Gap**: Need verification that the speculative decoding pipeline actually wires draft model selection to main model inference, with acceptance rate monitoring and adaptive draft length.

**Impact**: Missing 2-4x inference speedup for autoregressive generation tasks.

### DEFECT-MO-005: No ONNX/TensorRT Serving Integration (High)

**Current State**: `inference_runtime.rs` exists in `nt_shield_local_inference` but the codebase references ONNX and TensorRT only in documentation and KV cache config strings. No actual ONNX Runtime session management, no TensorRT engine building pipeline, no cross-platform execution provider abstraction.

**2026 Advance**: ONNX Runtime provides 15+ execution providers across NVIDIA/Intel/AMD/ARM/NPU (S9, S15). TensorRT achieves 6x lower latency than generic runtimes on NVIDIA hardware (S9). The edge benchmark (S9) shows TensorRT INT8 delivers 476 FPS vs 122 FPS FP32 on Jetson Orin.

**Gap**: No `OnnxSession` wrapper, no `TensorRT` engine builder, no execution provider abstraction, no benchmarking against ONNX/TensorRT for model selection.

**Impact**: Local model inference cannot leverage hardware-specific optimizations. The `LocalInferenceEngine` falls back to llama.cpp GGUF exclusively.

### DEFECT-MO-006: Missing NVFP4 / Blackwell Sparsity Support (Medium)

**Current State**: `quantization_engine.rs` mentions NVFP4 and Blackwell (L7-8, L976) but the actual `generate_mixed_precision_rules` function (L875) generates rules without Blackwell-specific 2:4 structured sparsity patterns.

**2026 Advance**: NVIDIA Blackwell B200 accelerates 2:4 sparsity patterns 2x (S1). AutoQuantize (S5) assigns mixed-precision automatically across model layers for NVFP4. Nemotron 3 Ultra 550B was quantized to NVFP4 with 5.9x throughput improvement (S5).

**Gap**: No hardware-aware sparsity pattern generation. No Blackwell 2:4 sparsity mask application. No NVFP4 calibration pipeline beyond format declaration.

**Impact**: Cannot achieve the 2x speedup from hardware-native sparsity on Blackwell GPUs.

### DEFECT-MO-007: No Distributed Training Time Predictor (Low)

**Current State**: The SEAL pipeline and NT-MIND evolution daemon run sequentially without awareness of training time across heterogeneous GPU clusters.

**2026 Advance**: Precision-aware distributed training time predictor (S10) achieves 9.8% MAPE vs 147.85% without precision awareness. FSDP2 + torch.compile is now the default distributed training approach (S11). Mixed precision causes 2.4x training time variation (S10).

**Gap**: No training cost estimator, no precision-aware time prediction for SEAL evolution cycles, no multi-GPU resource planning.

**Impact**: SEAL evolution cycles cannot estimate GPU time/cost for self-improvement runs.

---

## 3. Suggestions

### SUGGESTION-MO-001: Add `nt_core_pruner` Module to NT-CORE

Create a structured pruning module in `neotrix-core/src/unified/core/nt_core_pruner/` implementing:
- **Structured Pruner**: Remove entire channels/filters/layers (hardware-friendly)
- **Unstructured Pruner**: Individual weight removal with block sparsity (4x4 blocks for GPU alignment)
- **2:4 Sparsity Generator**: Blackwell-optimized mask generation
- **Pruning Scheduler**: Progressive pruning with fine-tuning recovery between stages
- **Sparsity Tracker**: Record sparsity patterns per layer for hardware optimization

Reference: NVIDIA Minitron pipeline (S5, S7) — width pruning + depth pruning.

### SUGGESTION-MO-002: Extend SEAL Pipeline with Prune→Quantize→Distill Stages

Modify the SEAL evolution pipeline in `nt_mind` to add compression stages:
```
SEAL Phase-0: Converge Check
SEAL Phase-1: Explore (existing)
SEAL Phase-2: Distill Knowledge (existing — session distillation)
NEW: SEAL Phase-2.5: Model Compression
  → Detect oversized models (memory check vs target hardware)
  → Structured prune to target sparsity
  → Quantize to target bitwidth (GPTQ/AWQ/FP8/NVFP4)
  → Distill from teacher to recover quality
  → Validate quality benchmark
SEAL Phase-3: Absorb (existing)
```

### SUGGESTION-MO-003: Add Adaptive Quantization Runtime

Enhance `KVCacheOptimizer` and `InferenceRuntime` with:
- **Input Complexity Analyzer**: Classify input difficulty (simple/complex/critical)
- **Dynamic Precision Shifter**: Switch FP8→INT8 for simple inputs, FP16 for complex
- **Layer-wise Precision Controller**: Use SEADA-style bit-level entropy for per-layer precision
- **Energy Budget Enforcer**: Under power constraints, dynamically reduce precision

### SUGGESTION-MO-004: Build ONNX Runtime + TensorRT Integration Layer

Create `nt_shield_local_inference/onnx_runtime.rs` and `tensorrt_runtime.rs`:
- `OnnxSession`: Load ONNX models with execution provider selection (CPU/CUDA/OpenVINO/NPU)
- `TensorRTBuilder`: Convert ONNX→TensorRT engine with INT8/FP8 calibration
- `RuntimeSelector`: Auto-select best runtime based on hardware profile + model + latency requirement
- `BenchmarkHarness`: A/B test runtime performance before deployment

### SUGGESTION-MO-005: Create `DistillationTrainer` in NT-ACT

Add model distillation infrastructure in `nt_act` (action domain — training is an action):
- `DistillationTrainer`: Teacher-student training loop with loss weighting
- `FeatureAligner`: Intermediate layer activation matching
- `AttentionDistiller`: Attention pattern transfer from teacher to student
- `MultiTeacherEnsemble`: Ensemble knowledge from multiple teachers
- `DistillationValidator`: Quality benchmark comparison pre/post distillation

### SUGGESTION-MO-006: Add Blackwell Hardware Profile to `ModelSelector`

Extend `model_selector.rs` hardware profiles (L257+) with:
- Blackwell B200/B100 profiles with 2:4 sparsity support flag
- NVFP4 native compute throughput estimates
- Transformer Engine FP8 training capability
- Grace Blackwell NVLink interconnect bandwidth

---

## 4. Priority Matrix

| Defect | Severity | Effort | Priority |
|--------|----------|--------|----------|
| MO-001 No structured pruning | Critical | High | P1 |
| MO-003 No knowledge distillation | High | High | P1 |
| MO-005 No ONNX/TensorRT integration | High | Medium | P1 |
| MO-002 No mixed-precision runtime | High | Medium | P2 |
| MO-006 No Blackwell sparsity | Medium | Low | P2 |
| MO-004 Speculative decoding incomplete | Medium | Low | P2 |
| MO-007 No training time predictor | Low | Medium | P3 |

---

## 5. Codebase References

| Current Module | Path | Status |
|---------------|------|--------|
| QuantizationEngine | `neotrix-core/src/unified/layers/embodiment/nt_shield/nt_shield_impl/quantization_engine.rs` | Exists — quantization only |
| KVCacheOptimizer | `...kv_cache_optimizer.rs` | Exists — cache optimization only |
| LocalInferenceEngine | `...nt_shield_local_inference.rs` | Exists — llama.cpp focused |
| SpeculativeDecoder | `...speculative_decoding.rs` | Exists — partial |
| ModelSelector | `...model_selector.rs` | Exists — model catalog only |
| ContextCompressor | `neotrix-core/src/neotrix/nt_core_bank/compressor.rs` | Exists — context compression only |
| SessionDistiller | `...nt_mind_distiller.rs` | Exists — session knowledge distillation |
| No Structured Pruner | — | **Missing** |
| No Model Distillation Trainer | — | **Missing** |
| No ONNX Runtime Wrapper | — | **Missing** |
| No TensorRT Integration | — | **Missing** |
| No Adaptive Quantization | — | **Missing** |
| No Blackwell Sparsity | — | **Missing** |
