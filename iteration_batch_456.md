# Iteration Batch 456 — CV Architecture Defect Analysis

**Date**: 2026-09-06
**Focus**: Image Classification, Object Detection, Image Segmentation — 2026 State-of-the-Art vs NeoTrix Design

---

## Sources Cited

### Image Classification (6 sources)
1. **TuringViT** (arXiv 2606.24253) — Turing Linear Attention, dynamic-resolution pretraining, VISTA-Curation data pipeline. Outperforms SigLIP2 with 10% data.
2. **Chain-of-Models Pre-Training** (CVPR 2026) — Inverse knowledge transfer across model families, 7.09× acceleration, performance-lossless.
3. **ProgResViT** (arXiv 2609.03216, Sep 2026) — Progressive resolution + width for adaptive ViT inference. 84.9% top-1 with early-exit.
4. **AdaPerceiver** (CVPR 2026) — Unified adaptivity across depth, width, and tokens. 85.4% accuracy, 36% higher throughput than FlexiViT-L.
5. **Franca** (CVPR 2026) — Open-source VFM matching DINOv2/SigLIP2 with Matryoshka clustering + RASA spatial disentanglement.
6. **Xray-Visual** (arXiv 2602.16918) — Joint image-video ViT with EViT token reorganization, 15B image-text pairs, LLM2CLIP text encoder.

### Object Detection (6 sources)
7. **TinyFormer** (arXiv 2605.25046) — YOLO-DETR hybrid: Parallel Bi-fusion Module + Spatial Semantic Adapter. 60.2% AP on COCO.
8. **HA-DETR** (Nature Sci Rep 2026) — Hybrid conv+attention decoder, Decoupled Gamma Loss. 48.4 AP at 68 FPS (R18).
9. **YOLO-Master** (CVPR 2026) — Efficient Sparse MoE for YOLO. Dynamic routing, 42.4% AP at 1.62ms.
10. **Le-DETR** (CVPR 2026) — Low-cost efficient DETR. 55.1 mAP with 80% less pretraining data.
11. **Super Sparse DETR** (CVPR 2026) — Structured sparsity with DAM+CS scoring. Deterministic acceleration post-export.
12. **RF-DETR** (Roboflow, Jan 2026) — ViT backbone DETR with MaskDINO segmentation head. Fewer false positives than YOLO.

### Image Segmentation (4 sources)
13. **SAM 2** (ICLR 2025, Meta) — Streaming memory transformer for video+image segmentation. 6× faster than SAM, 3× fewer interactions.
14. **SAM2-UNet** (Visual Intelligence, Jan 2026) — SAM2 Hiera encoder + U-Net decoder with adapters. SOTA on 18 datasets.
15. **X2SAM** (arXiv 2605.00891, May 2026) — Unified segmentation MLLM: text+visual prompts across images and videos via Mask Memory.
16. **CoCa** (Abhik Sarkar summary, Jun 2026) — Contrastive+captioning single-pass foundation model for classification+retrieval+captioning.

---

## Defects Found

### DEFECT-1: CV Pipeline Uses Mock/Stale Backend (CRITICAL)
**File**: `nt_sense_cv.rs:15-43`
**Current**: `CVConfig::default()` sets `model_backend: "opencv"`. All `detect_objects()`, `segment_image()`, `extract_features()` return hardcoded mock data (lines 177-257).
**Gap vs Research**: 2026 vision is dominated by ViT foundation models (TuringViT, Franca, DINOv2). No OpenCV pipeline matches transformer-based performance. The mock backend violates R-P79 (external tech absorption must connect to production paths).
**Suggestion**: Replace `model_backend: String` with a trait-object `Box<dyn VisionEncoder>` supporting at minimum: (a) ViT-based classification with dynamic resolution (ProgResViT/AdaPerceiver pattern), (b) DETR-based detection (Le-DETR/RT-DETR pattern), (c) SAM2-based segmentation with prompt encoding. Add a `VisionBackend` enum with `OpenCV`, `OnnxRuntime`, `LlamaCpp`, `RemoteApi` variants.

### DEFECT-2: ViT Encoder Lacks Linear Attention (HIGH)
**File**: `vit.rs:115-157` (`scaled_dot_product_attention`)
**Current**: Pure quadratic softmax attention: O(n²) in sequence length. No linear attention alternative.
**Gap vs Research**: TuringViT (2026) replaces most softmax layers with Turing Linear Attention (TLA), reducing sequence-length-to-latency from quadratic to near-linear. AdaPerceiver adds token/depth/width adaptivity. ProgResViT adds progressive resolution early-exit.
**Suggestion**: Add `AttentionMode` enum: `Softmax` (current), `Linear(TLA)`, `Window`, `Progressive(early_exit_threshold)`. The `multi_head_attention` function at line 159 should dispatch on this enum. Register token support (DINOv2/Xray-Visual pattern) is also missing — add optional register tokens to `JepaViTEncoder::new`.

### DEFECT-3: No Dynamic-Resolution / Adaptive Inference (HIGH)
**File**: `vit.rs:206` (`JepaViTEncoder::new`)
**Current**: Fixed `num_patches` at construction time. No support for variable input resolutions or early-exit confidence gating.
**Gap vs Research**: ProgResViT (Sep 2026) processes images progressively: low-res narrow subnetwork first, exits when confident, else refines with higher-res wider subnetwork. AdaPerceiver supports runtime-configurable depth/width/tokens. TuringViT has native dynamic-resolution pretraining.
**Suggestion**: Add `AdaptiveConfig { max_rounds: usize, confidence_threshold: f64, resolution_schedule: Vec<usize> }` to `JepaViTEncoder`. Implement progressive encoding that reuses intermediate representations across resolution rounds. The `encode` method should accept a confidence callback for early termination.

### DEFECT-4: Object Detection is Bounding-Box Only, No End-to-End DETR (HIGH)
**File**: `nt_sense_cv.rs:54-58` (`ObjectDetector` struct)
**Current**: `ObjectDetector` stores `detector_type: String` + `model_path` — no actual detection logic, no DETR-style set prediction, no NMS-free pipeline.
**Gap vs Research**: 2026 real-time detection converges on end-to-end DETR variants: Le-DETR (55.1 mAP, 80% less pretraining), YOLO-Master (MoE for adaptive computation), HA-DETR (hybrid conv+attention decoder). YOLO26 adds Hungarian matching. Super Sparse DETR achieves deterministic acceleration via structured sparsity.
**Suggestion**: Define `DetectionHead` trait with `predict(features) -> Vec<DetectionSet>` using Hungarian matching (no NMS). Add `DetectionBackend` enum: `YoloV13`, `LeDetr`, `YoloMaster(MoE)`, `SuperSparseDetr`. The `detect_objects` method must accept feature maps from the ViT encoder, not just raw images.

### DEFECT-5: Segmentation Has No Prompt Encoding or Temporal Memory (HIGH)
**File**: `nt_sense_cv.rs:62-65` (`ImageSegmenter` struct)
**Current**: `ImageSegmenter` stores `segmenter_type: String` + `num_segments: u32`. No prompt-based segmentation, no video memory, no mask propagation.
**Gap vs Research**: SAM 2 (ICLR 2025) introduced streaming memory for video segmentation with point/box/mask prompts. X2SAM (May 2026) adds LLM-conditioned Mask Memory for instruction-based segmentation across images and videos. SAM2-UNet (Jan 2026) uses Hiera encoder + U-Net decoder with adapters for 18-task SOTA.
**Suggestion**: Add `SegmentationPrompt` enum: `Point(f64, f64, bool)`, `Box(BoundingBox)`, `Mask(Vec<u8>)`, `Text(String)`. Add `MemoryBank` struct (SAM2 pattern) with `memory_encoder`, `memory_attention`, `memory_bank: VecDeque<FrameMemory>`. The `segment_image` method must accept prompts and maintain temporal state for video sequences.

### DEFECT-6: No Unified Image+Video Foundation Model (MEDIUM)
**File**: `visual_cortex.rs:4-7` (`VisualCortex` struct)
**Current**: `VisualCortex` only handles static file scans (`scan_from_file`). No video processing, no temporal modeling, no streaming architecture.
**Gap vs Research**: Xray-Visual (2026) jointly processes images and videos with 3D tokenization ViT. SAM 2 treats images as single-frame videos with streaming memory. X2SAM unifies image and video segmentation in one MLLM. CoCa unifies classification+retrieval+captioning in one forward pass.
**Suggestion**: Add `TemporalEncoder` trait with `encode_frame(frame, memory) -> (embedding, updated_memory)`. The `VisualCortex` should maintain a `FrameBuffer` and call `TemporalEncoder::encode_frame` in streaming fashion. Add `UnifiedVisionModel` that combines classification (CoCa-style dual loss), detection (DETR-style set prediction), and segmentation (SAM2-style prompt+memory) in a shared encoder.

### DEFECT-7: No MoE / Adaptive Computation for Perception (MEDIUM)
**File**: `nt_sense_cv.rs:140-272` (entire `ComputerVisionPipeline` impl)
**Current**: Static processing pipeline — every image goes through the same fixed computation path regardless of scene complexity.
**Gap vs Research**: YOLO-Master (CVPR 2026) uses Efficient Sparse MoE to dynamically allocate experts per input based on scene complexity. AdaPerceiver adapts depth/width/tokens at runtime. ProgResViT exits early for easy images. Super Sparse DETR prunes non-critical FFN channels.
**Suggestion**: Add `AdaptiveCompute` trait: `compute(input, budget) -> (output, actual_cost)`. Integrate MoE routing (YOLO-Master pattern) into the ViT encoder: `ExpertRouter { experts: Vec<Box<dyn ViTLayer>>, top_k: usize }`. Add inference budget parameter to all perception methods.

### DEFECT-8: No Data Curation Pipeline for Vision Training (MEDIUM)
**File**: Design gap — no equivalent to VISTA-Curation or CoM-PT
**Current**: No training data curation, no model family acceleration, no inverse knowledge transfer mechanism documented.
**Gap vs Research**: TuringViT's VISTA-Curation achieves SOTA with 10% data via image-video scoring and temporal aggregation. Chain-of-Models Pre-Training (CVPR 2026) accelerates model family training by 7.09× through inverse knowledge transfer from small to large models.
**Suggestion**: Define `DataCurationPipeline` trait in NT-WORLD or NT-MIND with: `score_image_text_pair(image, text) -> f64`, `curate_video_clips(video) -> Vec<CuratedClip>`, `select_best_caption(candidates) -> String`. Add `ModelFamilyTrainer` that implements CoM-PT pattern for training multiple ViT variants efficiently.

### DEFECT-9: No Matryoshka / Multi-Resolution Representation (MEDIUM)
**File**: `vit.rs:9-25` (`JepaViTEncoder` struct)
**Current**: Single fixed embedding dimension. No nested sub-embeddings for efficient retrieval at multiple granularities.
**Gap vs Research**: Franca (CVPR 2026) introduces Matryoshka multi-head clustering: nested multi-resolution representations from shared projection layers. This enables efficient KNN at compressed dimensions without retraining.
**Suggestion**: Add `matryoshka_dims: Vec<usize>` field to `JepaViTEncoder`. The `encode` method should produce embeddings at multiple dimensionalities (e.g., 64, 128, 256, 512) from the same forward pass. Store `MatryoshkaHead` projections per dimension tier.

### DEFECT-10: No Open-Source VFM Integration Path (LOW)
**File**: Design gap — no documented integration with Franca, DINOv2, SigLIP2, or SAM2 weights
**Current**: `JepaViTEncoder` uses random initialization (line 208-218). No weight loading, no pre-trained model adapter, no HuggingFace integration.
**Gap vs Research**: Franca (CVPR 2026) is fully open-source (data+code+weights) matching proprietary models. SAM2-UNet uses pre-trained SAM2 Hiera backbone. AdaPerceiver uses pre-trained SoViT-150M.
**Suggestion**: Add `WeightSource` enum: `Random(seed)`, `HuggingFace(repo_id, revision)`, `Local(path)`, `ONNX(path)`. Implement `load_pretrained(source: WeightSource) -> Result<Self>` on `JepaViTEncoder`. Add `AdapterConfig` for parameter-efficient fine-tuning (LoRA/adapter pattern from SAM2-UNet).

---

## Summary

| Severity | Count | Defect IDs |
|----------|-------|------------|
| CRITICAL | 1 | DEFECT-1 |
| HIGH | 4 | DEFECT-2, DEFECT-3, DEFECT-4, DEFECT-5 |
| MEDIUM | 4 | DEFECT-6, DEFECT-7, DEFECT-8, DEFECT-9 |
| LOW | 1 | DEFECT-10 |

**Total defects**: 10
**Research coverage**: 16 sources across 3 CV domains (classification, detection, segmentation)
**Key architectural gap**: The entire vision stack (`nt_sense_cv.rs`) is a mock/stub with no real model integration. The ViT encoder (`vit.rs`) is a clean but minimal implementation missing 2026-critical features (linear attention, dynamic resolution, MoE, Matryoshka). No DETR or SAM2 pathway exists.

**Recommended priority order**:
1. DEFECT-1 (unblocks all others — replace mock backend)
2. DEFECT-4 + DEFECT-5 (detection + segmentation need real model paths)
3. DEFECT-2 + DEFECT-3 (ViT efficiency + adaptivity)
4. DEFECT-6 + DEFECT-7 (unified model + MoE)
5. DEFECT-8 + DEFECT-9 + DEFECT-10 (training curation, Matryoshka, weight loading)
