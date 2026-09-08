# Iteration Batch 511 — Research Loop

**Date:** 2026-09-06
**Focus:** DSP, Audio Processing, Time-Series Analysis — 2026 Advances

---

## 1. Sources Cited

| # | Source | Date | Field |
|---|--------|------|-------|
| S1 | EG3 — "Digital Filter Design: Balancing Performance and Constraints" (eg3.com) | Apr 7, 2026 | DSP |
| S2 | APDSP 2026 — 10th Asia-Pacific DSP Conference, Chengdu | Mar 28-30, 2026 | DSP |
| S3 | Signal 2026 — 13th Intl Conf on Signal and Image Processing, Sydney | Jun 20-21, 2026 | DSP |
| S4 | MathWorks — Digital Filter Design R2026a update | 2026 | DSP |
| S5 | Isolate Audio — "Speech Enhancement Techniques: A Practical Guide for 2026" | Jul 6, 2026 | Audio |
| S6 | ICASSP 2026 URGENT Speech Enhancement Challenge (arXiv:2601.13531) | Jan 20, 2026 | Audio |
| S7 | MDPI Algorithms — "An In-Depth Review of Speech Enhancement Algorithms" | Feb 2026 | Audio |
| S8 | Interspeech 2026 — "Speech Enhancement Based on Drifting Models" (arXiv:2604.24199) | Apr 2026 | Audio |
| S9 | arXiv eess.AS Jun 2026 — 391 entries, BSS/separation/diarization papers | Jun 2026 | Audio |
| S10 | AI Code Invest — "Time-Series Anomaly Detection in 2026: From Classical Methods to Foundation Models" | Apr 3 / Jul 19, 2026 | TS |
| S11 | VLDB 2026 — HYDRA, MUFASA, DMRAD papers (ACM SIGMOD/VLDB) | May 2026 | TS |
| S12 | TimeRadar — KDD 2026 foundation model for TS anomaly detection | 2026 | TS |
| S13 | Forecast2Anomaly (F2A) — TSFM adapted for proactive anomaly prediction (arXiv:2511.03149) | Nov 2025 | TS |
| S14 | Springer — "Bridging Continual Learning and Green Cloud Computing for Sustainable TS Anomaly Detection" | Sep 2026 | TS |
| S15 | Guideflow — "12 Best Noise Cancellation Software Tools for 2026" | Aug 18, 2026 | Audio |

---

## 2. Research Findings

### 2.1 Digital Signal Processing (DSP)

**F1 — Adaptive FIR for ANC under 3ms latency.** Adaptive filters (128-512 tap FxLMS) in ANC headphones deliver 20-30 dB attenuation below 1 kHz. Production systems (AirPods Pro, Sony XM5) use hybrid feedforward+feedback paths. Hardware benchmarks: ESP32-S3 512pt FFT ~50μs, STM32F4 ~120μs, TI C6748 ~5μs, Zynq FPGA <1μs [S1].

**F2 — Coefficient quantization causes 25-35 dB stopband degradation.** A 60-tap FIR designed in float and naively truncated to Q15 degrades from 80 dB to 45-55 dB stopband. Fix: design directly in target word length using `scipy.signal.remez` quantization constraints, not post-hoc truncation [S1].

**F3 — Minimum-phase FIR via SciPy 1.14.** New reliable minimum-phase conversion halves group delay while preserving stopband attenuation. Tradeoff: passband phase distortion [S1].

**F4 — Multirate/Graph signal processing.** APDSP 2026 highlights "Graph Analysis, Spectral Graph Theory, Wavelets over graphs" as frontier topics in digital/multirate DSP [S2].

**F5 — ML-driven filter design.** Signal 2026 conference emphasizes "Machine Learning and Neural Networks" for detection, estimation, classification alongside traditional filter design, indicating convergence of ML and classical DSP [S3].

### 2.2 Audio Processing

**F6 — ICASSP 2026 URGENT Challenge.** 80+ teams registered, 29 submitted. Focus: universal speech enhancement systems handling diverse distortions, domains, and input conditions. Track 1: signal-level SE; Track 2: downstream task enhancement [S6].

**F7 — Drifting Models for SE.** Interspeech 2026 paper evolves the mapping function's pushforward distribution to match target distributions, enabling adaptive SE that tracks non-stationary noise without retraining [S8].

**F8 — Generative vs Discriminative SE.** June 2026 arXiv comparison: generative methods (diffusion, flow) show better perceptual quality but higher hallucination risk; discriminative methods (CNN+LSTM) are more robust but less natural. Tradeoff quantified across SNR regimes [S9].

**F9 — Blind Source Separation via block-diagonal spatial covariance.** Fast multichannel NMF for distributed microphone arrays — ICASSP 2026 [S9].

**F10 — AI noise cancellation market.** ANC headphones market: $20.38B (2025) → $39.25B (2030), 14% CAGR. AI-based audio enhancement market grew 63% Y-o-Y [S15].

**F11 — Audio-Visual fusion.** CVPR 2026: Stable Hybrid Cross-Attention Fusion for Audio-Visual Event Recognition — multimodal signal fusion as new DSP frontier [S9].

### 2.3 Time-Series Analysis

**F12 — Foundation models for TS anomaly detection.** TimesFM (Google, 100B points pre-trained), Chronos (Amazon, tokenized T5), MOMENT (CMU, masked reconstruction) — all deliver zero-shot competitive anomaly detection without per-dataset training [S10].

**F13 — HYDRA: Multi-level hierarchy-driven anomaly detection.** VLDB/SIGMOD 2026 — addresses multi-granularity anomalies via hierarchical approach [S11].

**F14 — TimeRadar: Domain-Rotatable Foundation Model.** KDD 2026 — foundation model that generalizes across domains via rotation-based pre-training [S12].

**F15 — Forecast2Anomaly (F2A).** Adapts TSFM for proactive anomaly prediction via joint forecast-anomaly loss + anomaly-sensitive RAG. Outperforms SOTA on 16 datasets [S13].

**F16 — Detect-Explain-Remediate shift.** Field moving from detection alone toward integrated systems combining LLMs for explanation, multimodal foundation models, and edge deployment of distilled detectors [S10].

**F17 — DMRAD: Dynamic Decomposition + Memory-Aware Reconstruction.** IEEE TKDE 2026 — noise-resilient multivariate TS anomaly detection via dynamic decomposition [S11].

**F18 — Benchmark rigour crisis.** Kim et al. 2022 showed many published benchmark results inflated by point-adjust metrics. Under stricter metrics, classical methods (Isolation Forest, Matrix Profile) perform comparably with deep models [S10].

---

## 3. Defects Found in NeoTrix Design

### D1 — No DSP Filter Chain Abstraction
**Gap:** NT-PHYSICAL defines `audio_sync_library` and `video_temporal_stabilizer` but has no abstraction for digital filter design (FIR/IIR/adaptive/polyphase). The 2026 DSP landscape shows that filter chain composition — from anti-alias through polyphase resampling to adaptive ANC — is a first-class concern. NeoTrix lacks a `NtFilterChain` trait that composes filter stages with latency budget tracking and coefficient quantization awareness.

**Impact:** Any NT-PHYSICAL audio pipeline will require hand-rolled filter implementations. The 25-35 dB quantization degradation (F2) will silently destroy performance in embedded deployments.

### D2 — No Coefficient Quantization Awareness
**Gap:** No design artifact addresses fixed-point coefficient quantization. The EG3 research (S1) demonstrates that naively porting float-designed FIR to Q15 costs 25-35 dB stopband. NeoTrix should have a `FilterDesignSpec` type that carries target word-length and validates coefficient precision at design time.

**Impact:** Production audio pipelines on ARM Cortex-M4/M55 or ESP32-S3 will silently degrade.

### D3 — Missing Adaptive Filter Infrastructure
**Gap:** ANC systems use 128-512 tap FxLMS adaptive filters updated at 48-96 kHz. NeoTrix has no adaptive filter abstraction, no Normalized LMS implementation, and no FxLMS secondary-path modeling. This is critical for NT-PHYSICAL audio feedback cancellation and NT-SHIELD active noise control.

**Impact:** Cannot build ANC-capable hardware integration.

### D4 — No Multirate/Polyphase Resampling Module
**Gap:** Polyphase decomposition cuts compute by exactly M-fold in resampling. Every audio resampler and SDR uses this. NeoTrix has no polyphase filter bank abstraction.

**Impact:** Audio sample-rate conversion will be ad-hoc and unoptimized.

### D5 — No Time-Series Foundation Model Integration
**Gap:** NeoTrix's NT-MEMORY and NT-MIND modules have no integration point for pre-trained TS foundation models (TimesFM, Chronos, MOMENT). The 2026 landscape (F12, F14) shows these deliver zero-shot anomaly detection that collapses time-to-deployment from weeks to hours. The `HeartbeatAggregator` could use a TSFM for predictive health monitoring instead of simple thresholding.

**Impact:** System health monitoring remains reactive (threshold-based) instead of predictive (forecast-based).

### D6 — No Detect-Explain-Remediate Pipeline
**Gap:** NeoTrix anomaly detection (SelfTest, converge_check) flags anomalies but provides no LLM-based explanation or remediation suggestion. The 2026 frontier (F16) shows the field has moved to integrated detect-explain-remediate systems. NT-META should coordinate this three-stage pipeline.

**Impact:** Self-healing loops (NT-REPAIR) cannot automatically explain why they triggered or suggest corrective actions.

### D7 — No Multimodal Signal Fusion for Perception
**Gap:** CVPR 2026 audio-visual fusion (F11) and ICASSP 2026 multichannel BSS (F9) show that multimodal signal fusion is now standard. NT-WORLD's `PerceptionBridge` handles attention-gated perception but has no explicit audio-visual cross-modal fusion pathway.

**Impact:** NT-WORLD cannot fuse audio scene classification with visual scene understanding.

### D8 — Speech Enhancement Not Modeled as Source Separation Problem
**Gap:** 65% of SE failures occur in multi-speaker environments where separation is needed, not enhancement (S5). NeoTrix conflates "audio cleanup" into a single pipeline. The design should distinguish enhancement (reduce noise on target speech) from separation (untangle competing sources) as fundamentally different capabilities with different architectures.

**Impact:** Audio processing pipelines will misapply enhancement when separation is needed, producing worse results.

### D9 — No Graph Signal Processing for Sensor Arrays
**Gap:** APDSP 2026 highlights "Spectral Graph Theory, Wavelets over graphs" as frontier DSP. NT-PHYSICAL sensor arrays (distributed microphones, IoT sensor meshes) could benefit from graph-based signal processing for spatial filtering and distributed estimation.

**Impact:** Distributed sensor processing remains pointwise instead of exploiting spatial correlations.

### D10 — No Foundation-Model-Anomaly Bridge in SEAL Pipeline
**Gap:** The SEAL pipeline's convergence_check uses statistical thresholds. The 2026 TSFM frontier shows that pre-trained models can detect anomalies zero-shot. The SEAL pipeline should have a stage where TSFM-based detection supplements statistical checks, enabling the pipeline to detect anomalies it was not explicitly trained for.

**Impact:** SEAL pipeline is blind to novel anomaly types not covered by existing SelfTest implementations.

---

## 4. Suggestions

| ID | Suggestion | Priority | Layers |
|----|-----------|----------|--------|
| S-G1 | Design `NtFilterChain` trait composing FIR/IIR/adaptive/polyphase stages with latency budget tracking, `FilterDesignSpec` carrying target word-length, and automatic coefficient quantization validation | High | L3 (NT-PHYSICAL), L1 (NT-ACT) |
| S-G2 | Implement `NtAdaptiveFilter` abstraction with Normalized LMS, FxLMS, and secondary-path modeling for ANC | High | L3 (NT-PHYSICAL) |
| S-G3 | Add `NtPolyphaseBank` for M-fold efficient resampling (decimation/interpolation) | Medium | L1 (NT-IO), L3 (NT-PHYSICAL) |
| S-G4 | Integrate TSFM adapter in NT-MIND: `TsfmBridge` that wraps TimesFM/Chronos/MOMENT for zero-shot anomaly detection on system metrics, feeding results into `HeartbeatAggregator` | High | L5 (NT-MIND), L5 (NT-CORE) |
| S-G5 | Extend NT-META with `DetectExplainRemediatePipeline`: Stage 1 (SelfTest/converge_check detects), Stage 2 (LLM generates natural-language explanation), Stage 3 (suggests remediation actions for NT-REPAIR) | High | L6 (NT-META), L3 (NT-REPAIR) |
| S-G6 | Add `NtAudioVisualFusion` module in NT-WORLD for cross-modal signal integration (audio scene + visual scene) | Medium | L2 (NT-WORLD) |
| S-G7 | Separate `NtSpeechEnhancement` from `NtSourceSeparation` in NT-IO audio pipeline, with explicit routing based on noise type diagnosis (steady vs. competing sources) | Medium | L1 (NT-IO) |
| S-G8 | Add graph signal processing primitives (`NtGraphSignalProcessor`) for distributed sensor arrays in NT-PHYSICAL | Low | L3 (NT-PHYSICAL) |
| S-G9 | Add TSFM-based anomaly detection as a SEAL Phase-0 supplement alongside statistical convergence_check, enabling detection of novel anomaly types | Medium | L5 (NT-CORE), L6 (NT-META) |
| S-G10 | Benchmark foundation model inference latency against edge deployment targets (ESP32-S3 50μs, STM32F4 120μs) to ensure TSFM integration doesn't violate real-time constraints | High | L3 (NT-PHYSICAL), L1 (NT-ACT) |

---

## 5. Cross-Cutting Observations

1. **The DSP-audio-TS boundary is dissolving.** Adaptive filtering (DSP) powers ANC (audio) which generates time-series telemetry that needs anomaly detection (TS). NeoTrix should model this as a single `SignalPipeline` trait with composable stages.

2. **Zero-shot is the new default.** Foundation models (F12, F14) make per-dataset training optional. NeoTrix's SEAL pipeline should assume zero-shot capability is available and fall back to training only when fine-tuning improves metrics.

3. **Coefficient quantization is a first-class concern.** The 25-35 dB degradation (F2) from naive float→fixed conversion is not a deployment detail — it's an architectural constraint that must propagate through the design.

4. **The field rewards simplicity.** Isolation Forest and Matrix Profile (classical methods) remain competitive with transformers on many benchmarks (F18). NeoTrix should always benchmark classical baselines before adopting deep methods.
