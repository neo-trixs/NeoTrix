# Iteration Batch 493 — Research Loop: Image Processing, Video Analysis, Visual Tracking

**Date**: 2026-09-06  
**Focus**: External research advances in image restoration, video anomaly detection, multi-object tracking  
**Method**: Web search → NeoTrix codebase gap analysis → defect identification → suggestions

---

## 1. Sources Cited

### Image Processing (6 sources)
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S1 | UniLDiff (Cheng et al.) | CVPR 2026 | Diffusion-prior all-in-one restoration with Degradation-Aware Feature Fusion (DAFF) + Detail-Aware Expert Module (DAEM). Handles 11 degradation types in one model. |
| S2 | IQPIR (Xiao et al.) | CVPR 2026 | Image Quality Prior from NR-IQA models guides restoration toward perceptually optimal outputs. Dual-codebook architecture. |
| S3 | UARE (Li et al.) | CVPR 2026 | Unified vision-language model combining IQA + restoration + enhancement in a single model. Two-stage progressive training. |
| S4 | RAR (Chen et al.) | CVPR 2026 | Restore-Assess-Repeat iterative framework. IQA and IR share a common latent space for end-to-end training. |
| S5 | DPC-Net | arXiv 2608.20141 | Dual-prior collaborative network: VLM-supervised degradation-semantic coupling + low-level visual prior knowledge bases. |
| S6 | PixRestore | arXiv 2608.16793 | VAE-free pixel-space Diffusion Transformer for unified restoration. 50M params, single-step inference, 44ms latency. |

### Video Analysis (7 sources)
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S7 | CLARA (CLIP+Adapters) | Springer 2026 | Lightweight CLIP adaptation for weakly-supervised VAD. Spatial + MLP + Augmented Temporal adapters. 89.13% AUC on UCF-Crime. |
| S8 | Alert-CLIP (Zhu et al.) | CVPR 2026 | Multi-level alignment (video-label, region-text, region-semantic) for anomaly awareness in CLIP embeddings. |
| S9 | Strictly Causal SSM VAD | arXiv 2608.24810 | O(1) state-space model for streaming VAD. 1300+ FPS on Apple M3 Pro. Closed-form decay↔latency relationship. |
| S10 | VTO | ACM MM 2026 | Process-supervised RL for multi-tool orchestration in VAD. 12 specialized vision tools. 10.2% accuracy gain over SFT. |
| S11 | LAVIDA | CVPR 2026 | Zero-shot VAD using MLLM + Anomaly Exposure Sampler. No VAD training data required. |
| S12 | ALeRT | Springer 2026 | Disagreement-aware late fusion of 3DCNN+MViT. 5.7-6.0 events/s, 3.1ms fusion overhead. |
| S13 | YOLO+CLIP VAD | MIPR 2026 | YOLOv11n-pose + CLIP ViT-B/32. 51 FPS, 89.26% AUROC on CUHK Avenue. |

### Visual Tracking (7 sources)
| # | Paper | Venue | Key Advance |
|---|-------|-------|-------------|
| S14 | PLANET | arXiv 2609.00924 | World-grounded queries: lifts 2D tracking to 3D via scene geometry in queries. SOTA across 3 benchmarks. |
| S15 | DROID-W | CVPR 2026 | Dynamics-aware SLAM: differentiable uncertainty-aware BA with DINOv2 features. 10 FPS real-time. |
| S16 | HyperSSM | CVPR 2026 | Hypergraph + SSM for collaborative MOT motion reasoning. Cross-object mutual constraint for noise suppression. |
| S17 | SAMURAI++ | CVPR 2026 | Adapts SAM2 to MOT via hierarchical trajectory memory. Training-free, unifies TbD+TbQ paradigms. |
| S18 | YesTrack | ECCV 2026 | Referring MOT via MLLM Yes/No verification. No text generation needed. TCP+TRP temporal constraints. |
| S19 | MATR | ICLR 2026 | Motion-Aware Transformer for MOT. Explicit motion prediction reduces query collisions. 71.3 HOTA on DanceTrack. |
| S20 | RoSe-SLAM | IROS 2026 | Semantic-aware Gaussian Splatting SLAM. 2D foundation model features for dynamic distractor filtering. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-IMG-001: No All-in-One Image Restoration Pipeline
**Severity**: HIGH  
**Evidence**: `nt_physical/video_post_processor.rs:12-55` defines `ColorAlignmentMode`, `StabilizationMode`, `SuperResolutionMode` as separate modes, but there is **no unified degradation-aware restoration pipeline**. The 2026 state-of-the-art (UniLDiff, UARE, RAR) all demonstrate that single-model multi-degradation restoration significantly outperforms separate pipelines.  
**Gap**: NeoTrix treats color alignment, denoising, super-resolution, and deblurring as independent steps. Modern approaches use degradation-aware feature fusion (DAFF) to handle compound degradations jointly.  
**Suggestion**: Implement a `UnifiedImageRestorer` in `nt_physical` that uses a VLM-based degradation classifier to detect compound degradations, then routes through a diffusion-prior restoration backbone. Mirror the RAR pattern: assess → restore → re-assess in latent space.

### DEFECT-IMG-002: Missing Perceptual Quality Guidance (No IQA Integration)
**Severity**: MEDIUM  
**Evidence**: `visual_consistency.rs` uses only structural consistency scores (pixel-level comparison). No NR-IQA model integration exists anywhere in the codebase.  
**Gap**: IQPIR (S2) and UARE (S3) prove that NR-IQA-derived quality priors significantly improve restoration by guiding toward perceptually optimal outputs rather than just L2/L1 fidelity.  
**Suggestion**: Add an `IQAQualityPrior` module to `nt_core` that wraps a pre-trained NR-IQA model (e.g., MUSIQ, MANIQA). Feed quality scores as conditioning signals into `VideoPostProcessor` and `VisualConsistencyManager` to steer restoration toward perceptual quality, not just pixel fidelity.

### DEFECT-IMG-003: No Iterative Restore-Assess-Repeat Loop
**Severity**: MEDIUM  
**Evidence**: `VideoPostProcessor` applies transformations in a single pass (lines 120-310). No feedback loop exists to evaluate whether restoration succeeded.  
**Gap**: RAR (S4) demonstrates that iterative assess-restore cycles in shared latent space outperform single-pass restoration, especially for unknown/degraded inputs.  
**Suggestion**: Add a `RestoreAssessRepeat` trait to `nt_physical` that takes IQA feedback and decides whether to re-run restoration. Use the latent-space integration pattern from RAR rather than decode-encode between modules.

### DEFECT-VID-004: No Streaming Anomaly Detection Capability
**Severity**: HIGH  
**Evidence**: `nt_world_video_pipeline.rs:54-80` only does frame deduplication via grayscale 16×16 descriptors. The `AnomalyDetector` in `nt_core_telemetry.rs:305` is a rolling z-score over metrics—**not** a video anomaly detector. No spatial-temporal anomaly detection exists.  
**Gap**: 2026 VAD research (CLARA, Alert-CLIP, SSM-based streaming) provides real-time anomaly detection at 51-1300 FPS. NeoTrix has zero video-level anomaly understanding.  
**Suggestion**: Implement `VideoAnomalyDetector` in `nt_world` layer. Use the CLARA pattern (S7): CLIP backbone + spatial adapter + temporal adapter. For edge deployment, use the SSM streaming approach (S9) which achieves 1300 FPS with O(1) state. Feed results into GWT for attention routing on anomalous regions.

### DEFECT-VID-005: No Multi-Modal VAD Fusion
**Severity**: MEDIUM  
**Evidence**: `VideoPipeline` processes only grayscale 16×16 descriptors. No RGB+optical flow+skeleton fusion. No disagreement-aware routing between complementary experts.  
**Gap**: ALeRT (S12) shows that disagreeing experts (3DCNN for motion + MViT for context) fused via confidence-weighted routing outperform any single expert by 3-8%.  
**Suggestion**: Add a `MultiModalVADFusion` module that combines: (1) motion features via lightweight 3DCNN, (2) appearance features via CLIP adapter, (3) skeleton-based features via YOLOv11-pose. Use Product-of-Experts fusion with temperature-scaled confidence weighting.

### DEFECT-VID-006: No Zero-Shot / Open-Vocabulary Anomaly Detection
**Severity**: MEDIUM  
**Evidence**: No zero-shot anomaly detection capability in the codebase. All anomaly detection requires pre-trained models on specific domains.  
**Gap**: LAVIDA (S11) achieves zero-shot VAD using pseudo-anomalies + MLLM without any VAD training data. Alert-CLIP (S8) supports open-vocabulary anomaly descriptions.  
**Suggestion**: Implement `ZeroShotAnomalyDetector` using the LAVIDA pattern: segment objects → transform into pseudo-anomalies → MLLM semantic scoring. This enables detection of previously unseen anomaly types, critical for NeoTrix's adaptive perception.

### DEFECT-VIS-007: No 3D-Aware Multi-Object Tracking
**Severity**: HIGH  
**Evidence**: `nt_world_video_pipeline.rs` does flat frame comparison only. No object tracking, no identity management, no 3D scene understanding. `VisualConsistencyManager` handles per-element consistency but not temporal object tracking.  
**Gap**: PLANET (S14) and SAMURAI++ (S17) demonstrate that lifting tracking to 3D (via scene geometry or hierarchical trajectory memory) dramatically improves association accuracy, especially under occlusion.  
**Suggestion**: Implement `WorldGroundedTracker` in `nt_world` that: (1) detects objects via foundation model, (2) lifts 2D detections to 3D world coordinates, (3) maintains identity via hierarchical trajectory memory (short-term motion + long-term appearance). Feed tracking results into `VisualConsistencyManager` for cross-shot consistency.

### DEFECT-VIS-008: No Dynamic Environment Handling for SLAM
**Severity**: MEDIUM  
**Evidence**: No SLAM or dynamic scene understanding exists in NeoTrix. `nt_physical/embodied_physics.rs` handles basic physics but not spatial mapping.  
**Gap**: DROID-W (S15) and RoSe-SLAM (S20) show that per-pixel uncertainty estimation + semantic features from foundation models enable robust tracking in dynamic scenes.  
**Suggestion**: Add `DynamicSLAM` capability to `nt_physical` that estimates per-pixel uncertainty via multi-view feature inconsistency (DINOv2). This provides spatial awareness for embodied agents and enables dynamic object filtering.

### DEFECT-VIS-009: No Cross-Object Motion Reasoning
**Severity**: LOW  
**Evidence**: Object-level processing in NeoTrix is independent per element. No collaborative motion reasoning between objects.  
**Gap**: HyperSSM (S16) shows that hypergraph-based cross-object motion correlation suppresses individual noise and handles occlusion via motion transfer from similar objects.  
**Suggestion**: Add a `CollaborativeMotionReasoner` that groups objects by motion similarity and performs joint inference. Useful for crowd surveillance, multi-character animation, and traffic monitoring scenarios.

### DEFECT-VIS-010: No MLLM-Based Visual Verification
**Severity**: LOW  
**Evidence**: `VisualConsistencyManager` uses heuristic similarity scoring. No natural language reasoning about visual content.  
**Gap**: YesTrack (S18) and VTO (S10) demonstrate that MLLM-based Yes/No verification or multi-step tool orchestration significantly outperforms fixed-pipeline approaches for visual reasoning tasks.  
**Suggestion**: Add `VisualVerificationAgent` to `nt_io` that uses MLLM for discriminative visual queries (Yes/No format, avoiding text generation overhead). Integrate with GWT attention routing for selective visual verification.

---

## 3. Summary Table

| Defect ID | Domain | Severity | Source(s) | Core Gap |
|-----------|--------|----------|-----------|----------|
| DEFECT-IMG-001 | Image | HIGH | S1,S4,S5,S6 | No unified degradation-aware restoration |
| DEFECT-IMG-002 | Image | MEDIUM | S2,S3 | No NR-IQA perceptual quality guidance |
| DEFECT-IMG-003 | Image | MEDIUM | S4 | No iterative restore-assess loop |
| DEFECT-VID-004 | Video | HIGH | S7,S8,S9 | No streaming video anomaly detection |
| DEFECT-VID-005 | Video | MEDIUM | S12 | No multi-modal VAD expert fusion |
| DEFECT-VID-006 | Video | MEDIUM | S8,S11 | No zero-shot/open-vocabulary anomaly |
| DEFECT-VIS-007 | Tracking | HIGH | S14,S17 | No 3D-aware multi-object tracking |
| DEFECT-VIS-008 | Tracking | MEDIUM | S15,S20 | No dynamic SLAM / spatial awareness |
| DEFECT-VIS-009 | Tracking | LOW | S16 | No cross-object motion reasoning |
| DEFECT-VIS-010 | Tracking | LOW | S10,S18 | No MLLM visual verification |

---

## 4. Priority Recommendations

1. **P0 (Immediate)**: DEFECT-VID-004 — Streaming video anomaly detection. NeoTrix has zero video anomaly understanding. The SSM-based approach (S9) is lightweight enough for edge deployment.
2. **P0 (Immediate)**: DEFECT-VIS-007 — 3D-aware MOT. Without object tracking, all cross-shot consistency is frame-level only. SAMURAI++ (S17) is training-free.
3. **P1 (Next Sprint)**: DEFECT-IMG-001 — Unified image restoration. Replace separate color/denoise/super-res with degradation-aware pipeline.
4. **P1 (Next Sprint)**: DEFECT-VID-006 — Zero-shot anomaly detection via LAVIDA pattern for open-world robustness.
5. **P2 (Backlog)**: DEFECT-IMG-002, DEFECT-IMG-003, DEFECT-VID-005, DEFECT-VIS-008 — These require more integration work but provide incremental value.
6. **P3 (Future)**: DEFECT-VIS-009, DEFECT-VIS-010 — Nice-to-have for advanced scenarios.
