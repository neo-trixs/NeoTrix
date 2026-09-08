# Iteration Batch 645 — Computer Vision 2026 Survey

**Date:** 2026-09-06  
**Previous batch:** 644 (no VI→MCMC bridge, no RVNP misspecification, fixed GWT threshold, no tail calibration, no E-value adaptive CP)

---

## 1. Object Detection — 2026 State of the Art

### Key Systems Found

| System | mAP50:95 (COCO) | License | Architecture |
|--------|-----------------|---------|--------------|
| RF-DETR 2XL | **60.1** (COCO record) | Apache 2.0 (base) / PML 1.0 (XL) | DINOv2 backbone + deformable cross-attention, NMS-free |
| YOLO26-X | 56.9 | AGPL-3.0 | CNN + attention, NMS-free, 43% faster CPU than YOLO11 |
| YOLO12-N | 40.4 | AGPL-3.0 | 2.5M params, 1.6ms on T4 |
| RTMDet-X | 52.8 | MIT | Pure throughput: 300+ FPS |
| RF-DETR-N | 48.4 | Apache 2.0 | 30.5M params, 2.3ms |

**Sources:** [arXiv 2509.25164](https://arxiv.org/html/2509.25164v5), [JetBrains Blog 2026-07-07](https://blog.jetbrains.com/pycharm/2026/07/best-object-detection-models-for-machine-learning-in-2026/), [Roboflow Blog 2026-08-06](https://blog.roboflow.com/best-object-detection-models/), [Tictag Blog 2026-04-23](https://www.tictag.io/blog/yolo26-real-time-object-detection-model-for-edge-ai-2026)

### NEW Defects/Improvements Identified

**DEFECT-D645-1: YOLO26 anchor paradox — NMS-free design trades accuracy for speed**  
YOLO26 eliminates NMS entirely (end-to-end predictions), but the anchor-free approach increases optimization difficulty. Small models (YOLO26-N) show 2-3% AP degradation vs. NMS-based equivalents. The survey (malikharispk/A-Comprehensive-Survey) confirms: "anchor elimination reduces hyperparameter tuning but increases optimization difficulty." **NeoTrix gap:** NT-ACT's `ParallelTaskManager` and `ResourceBudgetManager` do not model the NMS-accuracy tradeoff when selecting detection backbones for multi-task orchestration. Need a **DetectionBackendSelector** that weighs NMS-free vs NMS-based tradeoffs per deployment target.

**DEFECT-D645-2: Transformer detector efficiency gap — no hybrid architecture routing**  
"No Transformer detector achieves >50% AP with <100 GFLOPs without CNN hybridization." RT-DETR-L requires 63% more computation than YOLOv12-M for only 0.5% AP gain. **NeoTrix gap:** NT-CORE's `CapabilityBridge` (evolution→runtime mapping) has no cost-accuracy Pareto front for detection backbone selection. Need a **ComputeAwareRouter** that tracks FLOP/AP tradeoff per model and routes tasks accordingly.

**DEFECT-D645-3: RF-DETR domain transfer gap in edge deployment**  
RF-DETR achieves 60.6 mAP on RF100-VL (domain transfer benchmark), but its XL/2XL models require PML licensing and server GPUs. Edge deployment still favors YOLO26-N (43% faster CPU inference). No unified model handles both edge AND domain-transfer scenarios. **NeoTrix gap:** NT-WORLD's `MediaAssetRegistry` has no model-domain compatibility matrix. Need a **ModelDomainPairing** that maps (model × deployment_target × domain) → expected AP.

---

## 2. Image Segmentation — 2026 State of the Art

### Key Systems Found

| System | Key Capability | Benchmark | Date |
|--------|---------------|-----------|------|
| **SAM 3** | Promptable Concept Segmentation (PCS) — text/exemplar prompts → all instances | 48.8 mask AP on LVIS (zero-shot), 75-80% human perf on SA-Co (270K concepts) | Nov 2025 |
| **SAM 3.1** | Object Multiplex — shared-memory joint multi-object tracking, linear cost scaling | March 2026 | March 2026 |
| SAM 2.1 | Streaming memory video segmentation, 6× faster than SAM | ICLR 2025 | Sep 2024 |
| Mask2Former | Closed-vocabulary SOTA: 57-58 mIoU ADE20K, ~58 PQ COCO | Steady | 2026 |
| OneFormer | Panoptic SOTA: unified single-query architecture | Steady | 2026 |

**Sources:** [Meta SAM3 GitHub](https://github.com/facebookresearch/sam3), [arXiv 2511.16719](https://arxiv.org/abs/2511.16719), [Meta Blog SAM3.1 2026-03-27](https://ai.meta.com/blog/segment-anything-model-3/), [CodeSOTA Segmentation 2026](https://www.codesota.com/guides/image-segmentation)

### NEW Defects/Improvements Identified

**DEFECT-D645-4: SAM 3 linear cost scaling — no sub-linear multi-object tracking**  
SAM 3.1 introduces "Object Multiplex" with shared-memory multi-object tracking, but inference cost still scales **linearly** with number of tracked objects. For scenes with 100+ objects (crowd counting, warehouse inventory), this becomes prohibitive. **NeoTrix gap:** NT-ACT's `ParallelTaskManager` assumes independent GPU tasks. SAM 3.1's shared-memory model requires a **SharedMemoryTaskScheduler** that batches object queries within a single forward pass.

**DEFECT-D645-5: SAM 3 concept ambiguity — no confidence calibration for text prompts**  
SAM 3's presence token improves discrimination between closely related text prompts (e.g., "a player in white" vs "a player in red"), but the paper acknowledges "some concepts remain inherently ambiguous (e.g., 'small window', 'cozy room')." No confidence calibration mechanism exists for text-prompt segmentation. **NeoTrix gap:** NT-META's `QualityControlPipeline` has no prompt-segmentation confidence scoring. Need a **PromptConfidenceCalibrator** that outputs P(correct segmentation | text prompt, image context).

**DEFECT-D645-6: SAM 3 fine-tuning dependency — no zero-shot for niche domains**  
"SAM 3 generally performs well in zero-shot settings, but will benefit from fine-tuning for niche domains." Eye segmentation study (Niehorster et al., arXiv 2603.17715) shows SAM 3 with concept prompts "iris" and "sclera" **completely fails** — worse than SAM 2 visual prompting. Zero-shot ≠ universal. **NeoTrix gap:** NT-WORLD's `ReferenceBasedGeneration` has no domain-adaptation readiness scoring. Need a **ZeroShotDomainProbe** that estimates fine-tuning ROI before deployment.

---

## 3. Vision Transformers — 2026 State of the Art

### Key Systems Found

| System | Key Innovation | Benchmark | Date |
|--------|---------------|-----------|------|
| **ViT-5** | Component-wise modernization: LayerScale + RMSNorm + RoPE + QK-Norm + Register tokens | 84.2% top-1 ImageNet-1k (Base), 1.84 FID (generation) | Feb 2026 |
| **UKAST** (Swin+KAN) | Rational-function KANs in Swin Transformer encoders | SOTA on 4 medical segmentation benchmarks, data-scarce efficient | Jan 2026 |
| CiUNet | Swin-CNN hybrid with cross-layer skip connections | Medical segmentation, lightweight, privacy-friendly | Aug 2026 |
| Swin Transformer V3 | Hierarchical patch merging, 2-4 bit quantization | Edge deployment, real-time | 2026 |
| Light-VTD | Multi-path token-fusion ViT for fatigue detection | Nature Scientific Reports | Jun 2026 |

**Sources:** [arXiv 2602.08071 (ViT-5)](https://arxiv.org/abs/2602.08071), [arXiv 2511.04084 (UKAST)](https://arxiv.org/abs/2511.04084), [Nature Sci Rep (Light-VTD)](https://www.nature.com/articles/s41598-026-41847-y.pdf), [api4ai Vision Transformers 2026](https://api4.ai/blog/vision-transformers-2026-state-of-the-art-amp-business-impact)

### NEW Defects/Improvements Identified

**DEFECT-D645-7: ViT-5 SwiGLU over-gating — vision ≠ language architecture transfer**  
ViT-5's key finding: "architectural choices optimized for language models do not trivially translate to optimal vision performance." SwiGLU activations (standard in LLMs like LLaMA/Qwen) cause **over-gating** in vision models, degrading performance especially in small models. ViT-5 deliberately excludes SwiGLU. **NeoTrix gap:** NT-CORE's `CapabilityBridge` maps language-domain architectural patterns to vision without checking for over-gating. Need a **CrossModalArchValidator** that flags architecture transfers likely to cause degradation.

**DEFECT-D645-8: ViT-5 component non-orthogonality — naive combination fails**  
"Existing architectural refinements are not strictly orthogonal: naively combining all modern components does not necessarily lead to optimal performance." ViT-5 had to carefully select component combinations. No systematic framework exists for predicting which component combinations work. **NeoTrix gap:** NT-MIND's `SEAL pipeline` explores architectural variants but has no **ComponentOrthogonalityChecker** that validates whether two refinements are additive or redundant before combining.

**DEFECT-D645-9: UKAST data efficiency — KANs solve ViT data hunger but add complexity**  
UKAST (Swin + rational KANs) achieves SOTA medical segmentation with fewer FLOPs than SwinUNETR, but: (1) adds GR-KAN complexity to every Swin block, (2) rational base functions require careful tuning, (3) interpretability gains are marginal vs. computational overhead. **NeoTrix gap:** NT-PHYSICAL's `VideoPostProcessor` has no adaptive architecture complexity budget. Need a **ComplexityBudgetEnforcer** that caps FLOPs while maximizing accuracy per task.

---

## Summary: 9 New Defects Found (Batch 645)

| ID | Domain | Defect | NeoTrix Gap |
|----|--------|--------|-------------|
| D645-1 | Detection | YOLO26 anchor paradox: NMS-free trades accuracy for speed | `ParallelTaskManager` ignores NMS-accuracy tradeoff |
| D645-2 | Detection | Transformer efficiency gap: no hybrid routing | `CapabilityBridge` lacks cost-accuracy Pareto front |
| D645-3 | Detection | RF-DETR edge/domain-transfer split | `MediaAssetRegistry` lacks model-domain pairing |
| D645-4 | Segmentation | SAM 3 linear multi-object cost | `ParallelTaskManager` assumes independent tasks |
| D645-5 | Segmentation | SAM 3 text prompt ambiguity, no confidence calibration | `QualityControlPipeline` lacks prompt confidence scoring |
| D645-6 | Segmentation | SAM 3 zero-shot fails on niche domains | `ReferenceBasedGeneration` lacks domain-adaptation probe |
| D645-7 | ViT | SwiGLU over-gating in vision (≠ language) | `CapabilityBridge` blindly transfers LLM arch patterns |
| D645-8 | ViT | Component non-orthogonality: naive combo fails | `SEAL pipeline` lacks component orthogonality checker |
| D645-9 | ViT | UKAST KAN complexity vs. interpretability tradeoff | `VideoPostProcessor` lacks complexity budget cap |

---

## Prior Batch Context (644)

Batch 644 identified 5 defects in probabilistic/attention systems:
1. No VI→MCMC preconditioning bridge
2. No model misspecification detection (RVNP)
3. Fixed GWT attention threshold
4. No tail event calibration
5. No E-value adaptive CP

**Batch 645 builds on:** The detection/segmentation/ViT defects here compound with batch 644's attention defects. Specifically:
- D645-2 (Transformer efficiency gap) + D644-3 (fixed GWT threshold) → GWT cannot dynamically allocate attention to high-FLOP detection backbones
- D645-5 (SAM 3 prompt ambiguity) + D644-4 (no tail calibration) → ambiguous text prompts produce uncalibrated confidence in segmentation masks
- D645-8 (component non-orthogonality) + D644-1 (no VI→MCMC bridge) → architectural component search lacks probabilistic exploration of the design space
