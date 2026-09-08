# Iteration Batch 366 — Edge AI / TinyML / Model Deployment

**Date**: 2026-09-06
**Cycle**: External research → defect identification → design suggestions

---

## Sources Cited

| # | Source | Published | Topic |
|---|--------|-----------|-------|
| S1 | [On-Device LLMs Survey — Springer AI Review](https://link.springer.com/article/10.1007/s10462-026-11538-1) | 2026-05 | Comprehensive taxonomy of quantization, pruning, distillation, low-rank, hybrid pipelines; unified ALEM protocol (Accuracy-Latency-Energy-Memory); KV cache as first-class subsystem |
| S2 | [GOE — arXiv 2608.28652](https://arxiv.org/abs/2608.28652) | 2026-08 | Generalized Optimization Engine for edge AI inference — HW/model-agnostic architecture; compression method (not bit-width) determines accuracy survival on GPU-less edge CPU |
| S3 | [NVIDIA Jetson Blog — Frontier Reasoning at Edge](https://developer.nvidia.com/blog/frontier-reasoning-reaches-the-edge-how-to-deploy-and-optimize-models-on-nvidia-jetson/) | 2026-09 | MoE models (Nemotron 3.5 Lightning 30B/3B active) + NVFP4 + speculative decoding (DSpark/DFlash2) = 6.28× decode speedup; model-specific optimal config |
| S4 | [Multi-LoRA On-Device — ACL 2026](https://aclanthology.org/2026.findings-acl.2106.pdf) | 2026 | Runtime LoRA as frozen graph inputs + Dynamic Self-Speculative Decoding (DS2D); 4-6× memory/latency improvement on Samsung Galaxy S24/S25 |
| S5 | [MobileLLM-Flash — ACL Industry 2026](https://aclanthology.org/2026.acl-industry.51.pdf) | 2026 | Latency-guided NAS with hardware-in-the-loop; skip attention > SWA; shallow-and-wide optimal at low latency; 1.8× prefill / 1.6× decode speedup |
| S6 | [EdgeTune — ACM/IEEE EMS 2026](https://dl.acm.org/doi/10.1145/3774906.3802769) | 2026 | On-device LLM personalization: GradCut (importance-aware adapter placement) + data-free reuse-or-re-tune policy; 70-80% energy reduction |
| S7 | [PartInfer — ICLR 2026](https://openreview.net/pdf?id=3sbM94O8Ts) | 2026 | Neuron-level partial loading + partial computation; 13× speedup over disk-offloading; task-specific neuron profiling |
| S8 | [Trend-Aware TinyML Co-Design — SoutheastCon 2026](https://doi.org/10.1109/southeastcon63549.2026.11476185) | 2026-02 | INT8 + structured sparsity + on-device adaptation under hard flash/SRAM budgets; "knee region" where activations dominate SRAM; multi-MCU scaling reduces deadline miss 3-6× |
| S9 | [State of Edge AI on MCUs 2026](https://shawnhymel.com/3125/state-of-edge-ai-on-microcontrollers-in-2026/) | 2026-01 | TinyML ecosystem review: LiteRT for MCU (dominant), microTVM (discontinued), neuromorphic (Loihi 2, niche), vendor-optimized paths winning over open runtimes |
| S10 | [MLPerf Tiny v1.4 — MLCommons](https://mlcommons.org/2026/07/mlperf-tiny-v1-4-results/) | 2026-07 | Energy as first-class metric; streaming wake-word added; ST NPU 96% speedup; Qualcomm Sensing Hub <0.30ms; Syntiant NDP120 3.3% duty cycle |
| S11 | [Ariel-ML — ISIoT 2026](https://arxiv.org/html/2512.09800v2) | 2026-05 | First Rust embedded TinyML toolkit with multi-core parallelization on heterogeneous MCUs; 1.6× speedup on RP2040/RP2350 |
| S12 | [TinyDéjàVu — ISIoT 2026](https://arxiv.org/html/2512.09786) | 2026 | SSM-based streaming inference: 90% RAM reduction, 2-20× latency reduction for sliding window sensor data on MCU |
| S13 | [TinyVLM — arXiv 2603.00136](https://doi.org/10.48550/arxiv.2603.00136) | 2026-02 | Zero-shot VLM on MCU: 285KB RAM, 892KB flash, 26 FPS on STM32H7; Matryoshka distillation for nested embeddings |
| S14 | [HYPERTINYPW — MLSys 2026](https://arxiv.org/abs/2603.24916v1) | 2026 | Compression-as-generation: micro-MLP synthesizes PW kernels from codes at load time; 6.31× smaller at same F1 |
| S15 | [MLflow Model Serving](https://mlflow.org/articles/what-is-model-serving-infrastructure/) | 2026-08 | Serving ≠ deployment; output quality checks in health probes; drift + schema validation from day one |
| S16 | [KServe A/B Testing](https://oneuptime.com/blog/post/2026-02-09-kserve-ab-model-testing/view) | 2026-02 | Traffic routing with Knative revisions; canary percent; Prometheus metrics per revision; automated rollback |
| S17 | [LLM A/B Testing on K8s](https://www.kubenatives.com/p/ab-testing-llm-models-kubernetes) | 2026-07 | Shadow traffic (zero user risk); thumbs-up rate + LLM-as-judge + human eval; automated rollback via Alertmanager |
| S18 | [Netflix In-House LLM Serving](https://netflixtechblog.com/in-house-llm-serving-at-netflix-a5a8e799ea2c) | 2026-07 | vLLM + Triton unified; response_format silently dropped bug; Versioned deployment for schema-breaking changes; logits processing moved to batch level in V1 |
| S19 | [LLM A/B Testing Statistics — Atlan](https://atlan.com/know/ab-testing-llm-applications/) | 2026-07 | LLM output varies 15% at temp=0; 4× samples for half-detectable gap; randomization by user not request; 5 testable layers isolated |
| S20 | [MLOps Reference Architecture 2026](https://infrasketch.net/blog/mlops-system-design) | 2026-02 | Three maturity levels; CI/CD for ML = code + data + model; automated retraining triggers; canary/blue-green/shadow/A/B |

---

## Defects Found

### DEFECT-1: No Energy-First Metric in NT-PHYSICAL Power Management

**Evidence**: S10 (MLPerf Tiny v1.4) establishes energy-per-inference as the first-class metric for edge AI — not latency alone. S1 (ALEM protocol) formalizes Accuracy-Latency-Energy-Memory as a four-tuple. Syntiant NDP120 runs at 3.3% duty cycle, making idle power dominant. S8 identifies a "knee region" where peak activations dominate SRAM, and small architectural changes cause large deployability differences.

**NeoTrix gap**: `nt_physical::PowerManager` (`mod.rs:121-125`) tracks only `power_mode: PowerMode` (Normal/LowPower/Critical) and `battery_level: f64`. There is no per-inference energy accounting, no duty cycle management, no activation-SRAM budget tracking, and no energy-latency Pareto analysis. The safety kernel checks only `battery_level < 0.1` (line 268) — a coarse threshold with no connection to inference workload energy cost.

**Suggestion**: Extend `PowerManager` with:
1. `energy_budget: EnergyBudget` struct tracking joules per inference window and duty cycle target
2. `activation_sram_tracker` that monitors peak activation memory against the "knee region" threshold identified by S8
3. `energy_pareto()` method returning the Accuracy-Latency-Energy frontier for model selection
4. Integration with `inference_runtime.rs` to report per-inference energy draw back to the power manager

**Priority**: P1 — energy-blind power management fails at MCU scale

---

### DEFECT-2: No On-Device Personalization / Adapter Hot-Swapping

**Evidence**: S4 (Samsung Multi-LoRA) demonstrates runtime LoRA switching as frozen graph inputs — 8 use cases with a single foundation model, 3-6× latency improvement, dynamic task switching without recompilation. S6 (EdgeTune) achieves 70-80% energy reduction for continual personalization via GradCut + data-free reuse-or-re-tune policy.

**NeoTrix gap**: `inference_runtime.rs` (`InferenceRuntime`) manages static model loading with `model_path` + `backend` selection. There is no adapter/LoRA slot mechanism, no runtime weight-swap, no importance-aware gradient routing (GradCut), and no reuse-or-re-tune decision policy. Every new task requires full model reload — the opposite of the multi-LoRA paradigm.

**Suggestion**: Add `AdapterSlotManager` to `inference_runtime`:
1. Define `AdapterSlot { id, rank, frozen_graph_placeholder, cached_weight_delta }` — mirrors S4's one-hot mask approach
2. Implement `switch_adapter(slot_id, lora_weights)` without recompilation, using the frozen-inference-graph input pattern
3. Add GradCut-style importance scoring: track per-matrix gradient magnitude, skip low-yield adapter paths
4. Cache adapter patches across model releases with S6's data-free reuse-or-re-tune policy (KL divergence threshold on cached vs. fresh adapter)

**Priority**: P1 — core edge deployment gap

---

### DEFECT-3: No Speculative Decoding Infrastructure

**Evidence**: S3 (NVIDIA Jetson) shows NVFP4 + speculative decoding delivers 6.28× decode throughput. Critical finding: the optimal speculative decoding method differs per model (DSpark for Nemotron 3.5 Lightning, DFlash2 for Qwen3.8-27B). S4 introduces Dynamic Self-Speculative Decoding (DS2D) achieving 2.3× speedup without a draft model. Both require empirical validation against application-specific prompts.

**NeoTrix gap**: `inference_runtime.rs` defines no speculative decoding abstraction. The `start_inference` method (line 188) uses a simple loop with no draft-model orchestration, no token acceptance rate tracking, and no workload-category-aware configuration (writing vs. reasoning vs. summarization vs. RAG). The `InferenceConfig` has `num_threads` but no speculative decoding parameters.

**Suggestion**: Add `SpeculativeDecoder` module:
1. `enum SpecMethod { DFlash, DSpark, MTP, EAGLE3, DS2D }` with per-model default selection
2. `struct SpecConfig { method, draft_model_path, acceptance_rate_threshold, max_draft_tokens }` 
3. `fn validate_spec_config(model, prompts, method) -> SpecBenchResult` — runs the prompt-category benchmark pattern from S3
4. Integrate with `InferenceConfig`: when spec decoding is enabled, track acceptance rate per-session and auto-switch method if acceptance drops below threshold
5. For MCU-class targets (S7, S12), add a `PartialComputation` mode that dynamically computes only the most relevant neurons per prompt

**Priority**: P2 — significant throughput improvement available

---

### DEFECT-4: No Streaming Inference / Temporal Operator Optimization

**Evidence**: S12 (TinyDéjàVu) demonstrates transforming temporal operators into SSMs achieves 90% RAM reduction and 2-20× speedup for sliding-window sensor data on MCUs. S8 confirms that for streaming workloads, duty cycle management (97% idle for Syntiant NDP120) is more important than peak throughput.

**NeoTrix gap**: `nt_physical::Sensor` struct (`mod.rs:23-32`) stores static `SensorType` and `current_value` — no streaming buffer, no sliding window, no temporal operator support. The `read_sensor` method (line 220) returns a single snapshot, not a stream. There is no SSM-based inference path for time-series sensor data, and no overlap-rate optimization for redundant computation elimination.

**Suggestion**: Add `StreamingInferenceEngine` to NT-PHYSICAL:
1. `struct TemporalBuffer { data: RingBuffer<f64>, window_size: usize, overlap_rate: f64 }` — S12's sliding window
2. `fn to_ssm(temporal_op) -> SSMTransform` — convert convolutional temporal operators to state-space models for streaming
3. `fn preheat_ssm(model, initial_window)` followed by `fn stream_infer(model, new_data)` — S12's two-phase pattern
4. Duty cycle integration: `StreamingInferenceEngine` reports idle/active ratio to `PowerManager` for battery-aware scheduling

**Priority**: P2 — critical for sensor-based edge deployments

---

### DEFECT-5: No Model Deployment Lifecycle / A-B Testing / Shadow Mode

**Evidence**: S15 (MLflow) defines serving ≠ deployment — serving adds autoscaling, SLA enforcement, versioning, and output monitoring. S16-S17 (KServe, Kubernetes) demonstrate canary/shadow/A-B deployment patterns with automated rollback. S18 (Netflix) identifies that `response_format` was silently dropped in production — a schema-validation bug caught only because of output monitoring. S19 quantifies LLM output variance at 15% even at temperature=0, requiring 4× samples for half-detectable gaps.

**NeoTrix gap**: The `nt_act::production_orchestrator` and `nt_io::platform_gateway` handle multi-task parallelism and multi-platform integration, but there is no model deployment lifecycle management. No canary/shadow/A-B traffic routing, no model version registry with promotion workflows, no output quality health probes, no automated rollback on regression. The `InferenceRuntime` loads a single model with no versioning or staged rollout capability.

**Suggestion**: Add `ModelDeploymentLifecycle` at L1 (Action layer):
1. `struct ModelVersion { id, status: ModelStatus (staging|champion|challenger|shadow|retired), metrics, traffic_weight }`
2. `struct ServingPolicy { champion_version, challenger_version, challenger_weight, shadow_version }` — mirrors S16's KServe pattern
3. `fn route_request(request, policy) -> Vec<ModelVersion>` — fan-out to champion + shadow, return champion to caller
4. `fn health_probe_with_smoke_test(model) -> HealthStatus` — S15's recommendation: test output quality, not just container liveness
5. `fn auto_rollback(policy, alert_threshold) -> PolicyUpdate` — Prometheus-style alerting on thumbs-up rate or LLM-as-judge degradation
6. Schema validation gate: parse model output against `input_schema` / `output_schema` to catch S18's `response_format` silent-drop class of bugs

**Priority**: P1 — production readiness gap

---

### DEFECT-6: No ALEM Metric Protocol for Edge Model Selection

**Evidence**: S1 (ALEM protocol) formalizes a four-dimensional evaluation: Accuracy, Latency, Energy, Memory — with explicit trade-offs. S2 (GOE) proves that "compression method, not bit-width, determines whether task accuracy survives deployment" on GPU-less edge CPU. S5 (MobileLLM-Flash) shows skip attention beats SWA in the Pareto frontier. S7 (PartInfer) demonstrates fine-grained neuron-level profiling beats coarse quantization.

**NeoTrix gap**: Model selection in NeoTrix is ad-hoc: `InferenceConfig` has `model_path` and `backend` but no structured evaluation protocol. The `OptimizationProfile` in `nt_shield_local_inference.rs` stores per-model profiles in KB but has no cross-model comparison framework, no Pareto frontier computation, and no hardware-aware ranking. There is no mechanism to select the optimal compression method for a given hardware target — the exact finding of S2.

**Suggestion**: Add `ALEMEvaluator` module:
1. `struct ALEMScore { accuracy: f64, latency_ms: f64, energy_joules: f64, memory_bytes: u64 }`
2. `fn evaluate_model(model, hardware_target, eval_set) -> ALEMScore` — unified benchmark
3. `fn pareto_frontier(scores: Vec<ALEMScore>) -> Vec<ALEMScore>` — extract non-dominated set
4. `fn select_optimal(model_candidates, hw_target, constraints: ALEMConstraints) -> ModelChoice` — S2's insight: choose compression method, not bit-width
5. Cache results in KB as `inference_profile` namespace for cross-session reuse

**Priority**: P2 — structured edge optimization

---

### DEFECT-7: No Rust-Native Multi-Core MCU Inference Path

**Evidence**: S11 (Ariel-ML) demonstrates the first Rust embedded TinyML toolkit with native multi-core parallelization on heterogeneous MCUs (ARM Cortex-M, RISC-V, ESP32). Achieves 1.6× speedup on dual-core RP2040 with 8% flash overhead. The IREE integration outperforms microTVM significantly. S9 confirms microTVM is discontinued — IREE is the viable compiler path.

**NeoTrix gap**: NeoTrix is written in Rust, yet `inference_runtime.rs` delegates to external C/C++ engines (llama.cpp, Ollama, vLLM) via process execution. There is no Rust-native inference path, no multi-core MCU scheduling, and no IREE integration. The `InferenceBackend` enum has `LlamaCpp | Ollama | Vllm | Mlx | TensorRt` — no `IreeRust` or embedded Rust option.

**Suggestion**: Add `IreeRustBackend` to `InferenceBackend`:
1. Integrate IREE as compiler backend for embedded Rust targets, following S11's Ariel-ML architecture
2. Add `MultiCoreScheduler` that partitions inference graph across MCU cores with greedy scheduling
3. Benchmark against C/C++ baselines to quantify Rust overhead (S11 shows comparable memory footprints)
4. Target ARM Cortex-M + RISC-V as first-class MCU families for NT-PHYSICAL sensor nodes

**Priority**: P3 — Rust-native edge advantage deferred

---

### DEFECT-8: No Streaming Workload Duty-Cycle Awareness

**Evidence**: S10 (MLPerf Tiny v1.4) added streaming wake-word as a new benchmark because most production wake-word systems run idle 97% of the time. Syntiant NDP120 achieves 3.3% duty cycle. The benchmark measures energy during both idle-listening and active inference — the ratio determines battery life.

**NeoTrix gap**: `nt_physical::PowerManager` has no concept of workload duty cycle. The `PowerMode` enum (Normal/LowPower/Critical) is a global state, not per-workload. There is no mechanism to measure idle vs. active inference time, no budget allocation across concurrent tasks (S8 shows multi-MCU scheduling reduces deadline miss 3-6×), and no energy-aware task prioritization.

**Suggestion**: Add `DutyCycleManager` to NT-PHYSICAL:
1. `struct WorkloadDutyCycle { task_id, active_ms, idle_ms, energy_per_inference }` — per-task tracking
2. `fn schedule_with_duty_budget(tasks, total_energy_budget) -> Schedule` — allocate energy across tasks
3. `fn monitor_duty_cycle(task_id) -> DutyCycleStats` — real-time idle/active ratio
4. Integration with S8's multi-MCU pattern: split inference across 2-4 MCUs under bursty arrivals

**Priority**: P2 — battery-life critical for IoT deployments

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 20 |
| Defects identified | 8 |
| P1 defects | 3 (Energy metrics, On-device personalization, Deployment lifecycle) |
| P2 defects | 4 (Speculative decoding, Streaming inference, ALEM protocol, Duty cycle) |
| P3 defects | 1 (Rust-native MCU path) |

**Key theme**: NeoTrix's NT-PHYSICAL and inference runtime lack the energy-awareness, deployment lifecycle, and edge-specific optimization protocols that the 2026 edge AI ecosystem has standardized. The design is server-centric; edge deployment requires energy-as-first-class-metric, on-device personalization (LoRA hot-swap), speculative decoding orchestration, and structured deployment strategies (canary/shadow/A-B with automated rollback).
