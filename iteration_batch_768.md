# Iteration Batch 768 — Quantization, Local Inference, Model Formats

**Date:** 2026-09-07
**Context:** Iteration 768 of 10000+ research loop. Batch 767 validated RouteLLM 14% frontier routing. This batch targets quantization landscape, inference engine evolution, and model format convergence for NeoTrix tiered inference architecture.

---

## 1. Quantization — What's NEW

### 1.1 GGUF K-Quants Are the Dominant Local Format (2026 Reality)
- **Q4_K_M** = ~30-40% of all quantized model downloads on HuggingFace (onnx#7691)
- GGUF v2.1 spec (early 2026) added native H100 CUDA kernels, closing the GPU performance gap with AWQ/GPTQ
- K-quant variants (Q4_K_M, Q5_K_M, Q6_K) now have excellent kernel coverage across all platforms
- **Key insight:** GGUF Q5_K_M often matches or beats AWQ-INT4 at same effective bit-width due to better block-level precision allocation

### 1.2 AWQ Marlin Kernel — 10.9x Speedup from Kernel Alone
- AWQ without Marlin: 67 t/s. AWQ with Marlin: **741 t/s** on H200 (tensorrigs.com)
- Marlin kernel = INT4→FP16 dequant in registers, no HBM round-trip
- Requires Ampere (SM80) or newer — RTX 3000+
- AutoAWQ deprecated → **llm-compressor** (v0.10+) is the maintained tool from vLLM team

### 1.3 GPTQ Toolchain Migration
- AutoGPTQ archived April 2025 → **GPTQModel** (v5+) is drop-in replacement
- GPTQ v2.3 (March 2026) added AMD Instinct MI300X support
- EXL2 (derived from GPTQ): fastest single-stream tok/s on consumer NVIDIA, beats vLLM AWQ by 10-25%
- EXL2 limitation: no continuous batching, NVIDIA-only

### 1.4 FP8 Emerging as Production Sweet Spot on Hopper+
- FP8 (E4M3): ~1% quality loss vs FP16, 2x VRAM savings
- Native hardware support on H100/H200/B100 and RTX 4090/5090 (Ada/Blackwell transformer engine)
- AWQ v1.8 added FP8 support
- For H100 deployments where quality matters more than INT4 VRAM savings: FP8 wins

### 1.5 Quality Benchmark Data (2026 Consolidated)

| Format | Bits/param | Quality loss vs FP16 | Best runtime |
|--------|-----------|---------------------|--------------|
| FP8 (E4M3) | 8.0 | ~1% | vLLM/SGLang on Hopper |
| Q8_0 (GGUF) | 8.5 | ~1% | llama.cpp/Ollama |
| Q5_K_M (GGUF) | 5.7 | ~2% | llama.cpp/Ollama |
| AWQ-INT4 | 4.25 | ~2% | vLLM (Marlin) |
| Q4_K_M (GGUF) | 4.83 | ~2.5% | llama.cpp/Ollama |
| GPTQ-INT4 | 4.25 | ~3-5% | vLLM (Marlin) |
| Q3_K_M (GGUF) | 3.9 | ~5-8% | llama.cpp only |

### 1.6 ONNX Cannot Represent GGUF K-Quants (Open Issue onnx#7691)
- Q4_K_M alone = 30-40% of HF downloads, but ONNX has no two-level hierarchical block scales
- ONNX DequantizeLinear: single-level `block_size` only; K-quant needs `[256, 32]` hierarchy
- ONNX lacks 2/3/5/6-bit integer types (only int4/uint4/int8/uint8)
- MatMulNBits (com.microsoft): loses precision when flattening hierarchical scales
- **Recommendation:** Extend QDQ (Option A) as primary path for GGUF→ONNX interop
- ONNX RFC #8214: extensible quantization type system with URI-based format declarations — addresses the gap

---

## 2. Local Inference — What's NEW

### 2.1 Inference Engine Landscape Consolidated

| Engine | Model Format | Concurrency | Primary Use |
|--------|-------------|-------------|-------------|
| **llama.cpp** | GGUF | Single-user | CPU/GPU hybrid, portable |
| **Ollama** | GGUF (via llama.cpp) | Serial (queue-based) | Easiest setup, model registry |
| **vLLM** | SafeTensors/GPTQ/AWQ/FP8 | Continuous batching (PagedAttention) | Production multi-user |
| **SGLang** | SafeTensors/GPTQ/AWQ | Continuous batching | Structured output, agentic |
| **ExLlamaV3** | EXL2/GPTQ | Single-stream optimized | Consumer NVIDIA max tok/s |
| **TensorRT-LLM** | FP8/INT4/INT8 | Continuous batching | Max NVIDIA throughput |
| **MLX** | MLX-format quants | Limited | Apple Silicon optimized |

### 2.2 Ollama 0.24.x (May 2026)
- Native integration with Claude Code, OpenAI Codex, Copilot CLI via `ollama launch`
- Apple Silicon: switched to MLX backend on 0.19+ for improved performance
- Serial processing: degrades beyond 5-6 concurrent users
- 10-30% overhead vs direct llama.cpp calls (acceptable for convenience)

### 2.3 vLLM PagedAttention Performance
- At 10 concurrent users: vLLM ≈ 793 aggregate t/s vs Ollama ≈ 41 t/s (VRLA Tech)
- PagedAttention: 19-27% memory waste reduction vs naive KV cache
- GPU utilization: 85-92% under concurrent load
- vLLM v0.21.0 (May 2026): NVIDIA + AMD ROCm + Google TPU + Intel Gaudi plugins
- **Critical:** GGUF in vLLM = 93 t/s, 958ms TTFT — never use GGUF in vLLM
- vLLM requires 20-30% more VRAM than llama.cpp for same model (paging buffers)

### 2.4 llm-d Project — Disaggregated Serving
- Splits prefill (prompt processing) and decode (token generation) across different hardware
- Each stage optimized/scaled independently on Kubernetes
- Novel architecture for hyperscale inference

### 2.5 Speculative Decoding maturing
- Both llama.cpp and vLLM support draft-model speculative decoding
- Small "draft" model generates candidate tokens, verified by large model in single forward pass
- Multiplies effective throughput when draft accuracy is high

---

## 3. Model Format — What's NEW

### 3.1 GGUF vs ONNX: Distinct Niches Solidified

| Dimension | GGUF | ONNX |
|-----------|------|------|
| LLM-specific optimization | Deep (K-quants, metadata) | Good (ORT-GenAI) |
| Single-file bundle | Yes (weights+tokenizer+template) | No (tokenizer external) |
| Mobile NPU (Apple Neural Engine, Qualcomm Hexagon) | Not fully exploited | Yes (CoreML/QNN EPs) |
| Native Windows AI | Via llama.cpp | DirectML/Windows ML |
| Browser inference | None | ONNX Runtime Web (WASM/WebGPU) |
| Non-LLM models | LLMs primarily | Any neural network |

### 3.2 ONNX Quantization Extensibility (RFC #8214)
- Proposes `QuantizationProto` with URI-based format declarations
- Supports both GGUF-style packed streams and multi-component GPTQ/AWQ layouts
- `DequantizeExtensible` op with fallback embedded `FunctionProto` decoder
- Version formats independently through URI/version — no ONNX IR version bump per new format
- **Significance:** Could close the GGUF↔ONNX interop gap for NeoTrix cross-platform deployment

### 3.3 GGUF v2.1 + v2 Spec Evolution
- GGUF v2.1 (early 2026): H100 CUDA kernels, improved cross-platform performance
- GGUF format: mmap-friendly, single-file, extensible metadata KV pairs
- K-quant structure: hierarchical [super_block, sub_block] with separate scales per level
- On the fly decompression without GPU (enterprise streaming capability)

---

## 4. Defects & Improvements for NeoTrix Tiered Inference Architecture

### 4.1 DEFECT: Format-Runtime Coupling Creates Vendor Lock-in Risk
**Finding:** AWQ and GPTQ only run on NVIDIA via specific kernels. GGUF only runs on llama.cpp/Ollama. MLX only on Apple Silicon. No single format works everywhere.
**Impact:** NeoTrix tiered architecture (local→cloud) cannot use one format across all tiers.
**Fix:** RouteLLM must maintain format-aware routing: GGUF for local/Apple/CPU tier, AWQ for NVIDIA cloud tier, MLX for Apple-only tier. Format selection is a routing decision, not just model selection.
**Priority:** HIGH — this is a core architectural constraint

### 4.2 DEFECT: GGUF in vLLM = 79% Throughput Loss (958ms TTFT)
**Finding:** Running GGUF in vLLM produces 93 t/s vs 741 t/s for AWQ — a 7.97x penalty. 958ms time-to-first-token.
**Impact:** If NeoTrix accidentally routes GGUF models to vLLM serving tier, performance collapses.
**Fix:** NeoTrix provider registry must enforce format-runtime compatibility matrix. GGUF→llama.cpp/Ollama only. AWQ→vLLM/SGLang only.
**Priority:** CRITICAL

### 4.3 DEFECT: Ollama Concurrency Ceiling at 5-6 Users
**Finding:** Ollama processes requests serially. Beyond 5-6 concurrent users, latency degrades rapidly. vLLM handles 100+ concurrent users.
**Impact:** NeoTrix local tier cannot serve team workloads via Ollama.
**Fix:** Implement concurrency threshold routing: single-user → Ollama, multi-user → vLLM (if NVIDIA available) or llama.cpp server mode with manual batching.
**Priority:** HIGH

### 4.4 DEFECT: Q3_K_M and Below Breaks Reasoning/Coding Tasks
**Finding:** Q3_K_M: 5-8% perplexity degradation, ~5% MMLU drop. Reasoning benchmarks degrade 3x faster than aggregate numbers suggest. Code generation (HumanEval) drops ~10 points at Q3.
**Impact:** NeoTrix agent tasks (code generation, multi-step reasoning) will fail at Q3 or below.
**Fix:** Set Q4_K_M as absolute minimum for agent/coding tasks. Q5_K_M for reasoning-critical paths. Reserve Q3 only for non-reasoning classification/chat.
**Priority:** HIGH

### 4.5 IMPROVEMENT: Marlin Kernel Activation Verification
**Finding:** AWQ without Marlin = 67 t/s. AWQ with Marlin = 741 t/s. Same model, same hardware, 10.9x difference. Users often don't know if Marlin is active.
**Impact:** NeoTrix cloud tier could silently run AWQ without Marlin, getting 10x worse performance.
**Fix:** Add Marlin kernel detection in provider health check. Log kernel mode. Alert if AWQ running without Marlin on Ampere+.
**Priority:** HIGH

### 4.6 IMPROVEMENT: FP8 as Hopper Tertiary Tier
**Finding:** FP8 = ~1% quality loss, 2x VRAM savings, native H100 support. Better quality than INT4 at higher VRAM cost.
**Impact:** For H100 deployments where quality matters (medical, legal, financial), FP8 is the optimal tier between FP16 and INT4.
**Fix:** Add FP8 as a third quantization tier in NeoTrix: Tier 1 = FP16 (quality-critical), Tier 2 = FP8 (Hopper production), Tier 3 = INT4 AWQ (cost-optimized).
**Priority:** MEDIUM

### 4.7 IMPROVEMENT: EXL2 for Consumer NVIDIA Single-User
**Finding:** EXL2 via ExLlamaV2/TabbyAPI: 10-25% faster than vLLM AWQ for single-stream. Dual-3090 NVLink under $2000 is highest single-stream setup.
**Impact:** NeoTrix developer workstation tier could benefit from EXL2 for max personal throughput.
**Fix:** Add EXL2/ExLlamaV3 as optional local provider for NVIDIA single-user scenarios. Route only single-user requests to it.
**Priority:** MEDIUM

### 4.8 IMPROVEMENT: ONNX Interop for NPU/Browser/Windows
**Finding:** ONNX reaches hardware GGUF doesn't: mobile NPUs (Apple Neural Engine, Qualcomm Hexagon), native Windows (DirectML), browser (ORT Web).
**Impact:** NeoTrix future mobile/edge/browser deployments need ONNX path, even though current local inference is GGUF-native.
**Fix:** Track ONNX RFC #8214 (extensible quantization type system). When GGUF→ONNX bridge matures, add ONNX execution provider for edge/mobile/browser tiers.
**Priority:** LOW (future-facing)

### 4.9 IMPROVEMENT: Model Size > Precision for Quality
**Finding:** Quality impact of quantization is smaller on larger models. 70B Q4 degrades less than 7B Q4. Larger models have more weight redundancy.
**Impact:** NeoTrix should prefer larger models at lower precision over smaller models at higher precision, when VRAM allows.
**Fix:** In cost-equivalent VRAM budget, prefer 70B Q4 over 7B Q8 for agent tasks. Quality benchmarks confirm this.
**Priority:** MEDIUM

### 4.10 IMPROVEMENT: Speculative Decoding as Throughput Multiplier
**Finding:** Both llama.cpp and vLLM support draft-model speculative decoding. Multiplies effective throughput when draft accuracy is high.
**Impact:** NeoTrix can 2-3x throughput without hardware upgrade by enabling speculative decoding on compatible workloads.
**Fix:** Add speculative decoding configuration per provider. Enable for chat/drafting workloads where draft accuracy is high. Disable for high-entropy/creative tasks.
**Priority:** MEDIUM

---

## 5. Sources Cited

| # | Source | URL | Date | Key Finding |
|---|--------|-----|------|-------------|
| 1 | RunLocalAI — Quantization Formats | runlocalai.co/systems/quantization-formats | 2026-05-07 | Format comparison matrix, quality vs VRAM tradeoff table |
| 2 | AI/TLDR — GGUF vs GPTQ vs AWQ | ai-tldr.dev/learn/local-open-models/quantization-and-formats/ | 2026-06-12 | Deprecation of AutoGPTQ/AutoAWQ, Marlin kernel importance |
| 3 | Presenc AI — LLM Quantization Benchmarks | presenc.ai/research/local-llm-quantization-quality-benchmarks-2026 | 2026-05-07 | Perplexity/MMLU/GSM8K benchmarks across formats |
| 4 | Neural Digest — Model Quantization Guide | neural-digest.com/what-is-model-quantization-and-which-format-should-you-use-for-local-llms-in-2026/ | 2026-05-13 | GPTQ v2.3 MI300X, AWQ v1.8 FP8, throughput benchmarks |
| 5 | TensorRigs — LLM Quantization Guide | tensorrigs.com/blog/llm-quantization-guide/ | 2026-04-02 | AWQ 741 t/s with Marlin vs 67 t/s without |
| 6 | Inferencerig — GGUF vs AWQ vs GPTQ | inferencerig.com/performance/llm-quantization-explained | 2026-04-20 | GGUF in vLLM = 93 t/s (anti-pattern) |
| 7 | D-Central — Ollama vs vLLM vs llama.cpp | d-central.tech/ollama-vs-vllm-vs-llama-cpp/ | 2026-06-15 | Ollama 0.24.x, MLX backend, concurrency limits |
| 8 | StackCompare — Inference Engine Comparison | stackcompare.net/ollama-vs-vllm-vs-llama-cpp | 2026-04-13 | PagedAttention throughput advantage |
| 9 | Red Hat — llama.cpp vs vLLM | developers.redhat.com/articles/2026/06/15/llamacpp-vs-vllm | 2026-06-15 | llm-d disaggregated serving, GuideLLM benchmarks |
| 10 | Oflight — Engine Comparison | oflight.co.jp/en/columns/local-llm-inference-engine-comparison-2026 | 2026-08-13 | MLX Apple Silicon, TensorRT-LLM, engine decision tree |
| 11 | ONNX Issue #7691 — GGUF K-quant Support | github.com/onnx/onnx/issues/7691 | 2026-02-27 | ONNX cannot represent K-quant hierarchy |
| 12 | ONNX RFC #8214 — Extensible Quantization | github.com/onnx/onnx/pull/8214 | 2026 | URI-based quantization type system |
| 13 | VRLA Tech — Engine Comparison | vrlatech.com/llm-inference-engine-comparison-2026/ | 2026-06-16 | vLLM 793 t/s vs Ollama 41 t/s at 10 concurrent |
| 14 | DEV Community — Ollama vs llama.cpp vs vLLM | dev.to/thurmon_demich | 2026-05-20 | GGUF vs SafeTensors format split |
| 15 | Stabilarity — GGUF and ONNX Enterprise | hub.stabilarity.com | 2026-08-01 | Enterprise latency/throughput/CI/CD comparison |
| 16 | Jarhalab — GGUF vs ONNX | formats.jarhalab.com/comparisons/gguf-vs-onnx | 2026-06-04 | GGUF for local LLM, ONNX for cross-runtime ML |

---

## 6. Summary — Key Takeaways for NeoTrix

1. **Format-runtime coupling is the #1 constraint**: GGUF→llama.cpp, AWQ→vLLM, MLX→Apple. No format works everywhere. NeoTrix routing must enforce this.
2. **Marlin kernel detection is critical**: 10.9x performance difference between AWQ-with-Marlin and AWQ-without. Must be verified in provider health checks.
3. **Q4_K_M is the quality floor**: Below Q4, reasoning/coding degrades 3x faster than aggregate metrics suggest. Agent tasks must not go below Q4.
4. **FP8 deserves a tier**: Between FP16 and INT4, FP8 on Hopper is the quality-optimal production format (~1% loss, 2x VRAM savings).
5. **Ollama is single-user only**: Concurrency ceiling at 5-6 users. Multi-user requires vLLM.
6. **ONNX interop is future-facing**: Track RFC #8214 for GGUF→ONNX bridge. Needed for mobile NPU/browser/Windows edge tiers.
7. **Larger models > higher precision**: 70B Q4 beats 7B Q8 for agent tasks. Prefer size when VRAM allows.
