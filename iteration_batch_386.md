# Iteration Batch 386 — Computer Vision 2026 Advances

**Date**: 2026-09-06
**Domain**: NT-SENSE (L2 Perception) — Computer Vision Pipeline

---

## 1. Research Sources

### Object Detection (2026)
| Paper | Venue | Key Advance |
|-------|-------|-------------|
| OV-DEIM (arXiv:2603.07022) | arXiv 2026 | Real-time DETR-style open-vocabulary detector. Eliminates NMS via one-to-one set prediction. GridSynthetic augmentation for rare categories. DINOv3 backbone. |
| YOLOE / YOLOE26 (Ultralytics 2025-2026) | Product | RepRTA + LRPC modules for zero-cost open-vocabulary. Re-parameterizable: open-vocab at inference cost = 0 when closed-set. YOLOE26-L: 36.8% LVIS mAP at 161 FPS (T4). |
| WeDetect (CVPR 2026) | CVPR 2026 | Retrieval-based dual-tower OVOD. No cross-modal fusion layer. Proposal embeddings enable object retrieval in historical data. LMM-based REC integration. |
| DeCo-DETR (ICLR 2026) | ICLR 2026 | Vision-only OVOD: eliminates text encoder at inference via Dynamic Hierarchical Concept Pool (DHCP). LLaVA-generated prototypes + CLIP alignment. 135ms inference. |
| OmDet-Turbo | arXiv 2024+ | Efficient Fusion Head: 100.2 FPS TensorRT on A100. ELA-Encoder/Decoder replaces heavy cross-attention. |

### Instance Segmentation (2026)
| Paper | Venue | Key Advance |
|-------|-------|-------------|
| SAM 2 (Meta, arXiv:2408.00714) | arXiv 2024 | Foundation model for promptable visual segmentation in images AND videos. Streaming memory architecture. 3x fewer interactions, 6x faster than SAM on images. SA-V dataset (largest video segmentation). |
| X2SAM (arXiv:2605.00891) | arXiv 2026 | Unified MLLM for segmentation: conversational text + visual prompts → masks across images and videos. Mask Memory module for temporal consistency. |
| GeoSAM2 (CVPR 2026) | CVPR 2026 | 3D part segmentation via multi-view 2D SAM2 mask prediction. LoRA + residual geometry fusion. Prompt-controllable without text. |
| YOLOE Segmentation | Product | Built-in instance segmentation via mask prediction branch (YOLACT-style). Open-vocab masks for any prompted class. |

### Panoptic Segmentation (2026)
| Paper | Venue | Key Advance |
|-------|-------|-------------|
| DVPSFormer (arXiv:2607.26165) | arXiv 2026 | Online depth-aware video panoptic segmentation. Explicit Scene Discretization (ESD) + D2C depth head. Online Majority Voting (OMV) for temporal classification. 18x faster than offline. |
| ST-CFNet (ICASSP 2026) | ICASSP 2026 | Real-time 4D panoptic LiDAR segmentation. TWA-P2G temporal feature aggregation. 70.2 LSTQ on SemanticKITTI. |
| UP-Fuse (arXiv:2602.19349) | arXiv 2026 | Uncertainty-aware LiDAR-Camera fusion. Range-view representation. Hybrid 2D-3D decoder. Robust to camera degradation/failure. 5.7 FPS. |
| LiDAR-SAM2 (arXiv:2608.25418) | arXiv 2026 | Distills SAM2 video priors into LiDAR. Zero human annotation. Single click → consistent 4D mask tracks. |
| VideoCUPS (arXiv:2606.04925) | arXiv 2026 | First unsupervised video panoptic segmentation. Monocular depth + motion + visual cues → pseudo-labels. Video DropLoss. |
| Geometry-aided VPS (ISPRS 2026) | ISPRS 2026 | Stereoscopic VPS with depth-constrained kernel segmentation + optical flow association. |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-CV-001: Open-Vocabulary Detection Not Implemented
**File**: `nt_sense_cv.rs:54-58`
**Gap**: `ObjectDetector` struct stores a `detector_type: String` and `classes: Vec<String>` — a closed-set paradigm. No text encoder, no vision-language alignment, no prompt mechanism.
**2026 State-of-the-art**: OV-DEIM, YOLOE, WeDetect, DeCo-DETR all achieve real-time open-vocabulary detection. The field has moved to zero-shot category recognition via CLIP-style alignment or prototype pools.
**Impact**: NT-SENSE cannot detect novel/unseen object categories at runtime — a critical limitation for autonomous perception.

### DEFECT-CV-002: No Video Segmentation / Temporal Memory
**File**: `nt_sense_cv.rs` — entire module is image-only
**Gap**: No streaming memory, no temporal propagation, no video-level segmentation.
**2026 State-of-the-art**: SAM 2's streaming memory architecture processes video frames one-at-a-time with a memory bank. X2SAM extends this to MLLM-guided video segmentation. DVPSFormer achieves online 4D panoptic segmentation.
**Impact**: NeoTrix video pipeline (`nt_world_video_pipeline.rs`) cannot perform per-object tracking across frames — it operates frame-by-frame without instance identity persistence.

### DEFECT-CV-003: No Panoptic Segmentation Capability
**File**: `nt_sense_cv.rs:62-65`
**Gap**: `ImageSegmenter` has only `segmenter_type` and `num_segments` — no semantic/instance/stuff distinction. Panoptic segmentation unifies thing/stuff classification with instance separation.
**2026 State-of-the-art**: DVPSFormer fuses depth + semantics + tracking in one pass. ST-CFNet achieves real-time 4D LiDAR panoptic. UP-Fuse handles multi-modal fusion with uncertainty awareness.
**Impact**: NeoTrix cannot produce panoptic outputs (semantic class + instance ID per pixel), limiting autonomous scene understanding.

### DEFECT-CV-004: NMS Dependency — No End-to-End Detection
**File**: `nt_sense_cv.rs:26` — `nms_threshold: f64` in CVConfig
**Gap**: The design assumes NMS post-processing. 2026 DETR-style detectors (OV-DEIM, DeCo-DETR, YOLOE26) eliminate NMS via one-to-one set prediction, achieving lower latency and better scaling with vocabulary size.
**Impact**: NMS becomes a bottleneck as vocabulary grows; category-dependent post-processing cost scales linearly with class count.

### DEFECT-CV-005: No Multi-Modal Sensor Fusion
**File**: `nt_physical/video_post_processor.rs` — purely RGB post-processing
**Gap**: No LiDAR-camera fusion, no depth-aware processing, no uncertainty-guided modality weighting.
**2026 State-of-the-art**: UP-Fuse dynamically fuses LiDAR and camera with uncertainty maps. LiDAR-SAM2 distills 2D video models into 3D LiDAR. DVPSFormer couples depth estimation with panoptic segmentation.
**Impact**: For physical embodiment scenarios (robotics, autonomous driving), NeoTrix cannot fuse multi-modal sensor data.

### DEFECT-CV-006: No Interactive/Promptable Segmentation
**File**: `nt_sense_cv.rs` — no prompt interface
**Gap**: No point/box/mask prompt API. No user-in-the-loop refinement.
**2026 State-of-the-art**: SAM 2 accepts clicks, boxes, or masks on any frame and propagates across video. GeoSAM2 enables 3D part segmentation via 2D prompts. X2SAM accepts conversational instructions.
**Impact**: NeoTrix cannot support interactive segmentation workflows — a key requirement for robotics and human-in-the-loop perception.

### DEFECT-CV-007: No Unsupervised/Zero-Shot Segmentation
**File**: entire CV pipeline
**Gap**: No self-supervised learning, no pseudo-label generation, no zero-shot generalization.
**2026 State-of-the-art**: VideoCUPS generates panoptic pseudo-labels from monocular video without any supervision. DeCo-DETR builds concept pools from LLaVA descriptions without text encoders.
**Impact**: NeoTrix requires fully supervised training data for every new domain — cannot bootstrap from unlabeled video.

---

## 3. Design Suggestions

### SUGGESTION-CV-001: Add Open-Vocabulary Detection to NT-SENSE
Adopt a dual-tower retrieval architecture (à la WeDetect/YOLOE) or prototype-based approach (à la DeCo-DETR's DHCP). For NeoTrix's Rust-native constraint, the DeCo-DETR approach is preferable: pre-compute concept prototypes from LLaVA+CLIP offline, store in KB, run detection without text encoder at inference. The `ObjectDetector` struct should gain:
```rust
pub struct OpenVocabDetector {
    concept_pool: ConceptPrototypePool,  // pre-computed from LLaVA+CLIP
    backbone: VisionBackbone,             // DINOv3 or similar
    query_decoder: TransformerDecoder,    // DETR-style set prediction
    nms_free: bool,                       // always true
}
```

### SUGGESTION-CV-002: Integrate SAM 2 Streaming Memory into Video Pipeline
Add a `MemoryBank` module to `nt_world_video_pipeline.rs` that stores per-object embeddings and prompts across frames. The memory attention module conditions current-frame features on past predictions, enabling:
- Prompt propagation across frames
- Instance identity persistence
- Interactive refinement at any frame

### SUGGESTION-CV-003: Implement Panoptic Head with ESD Pattern
Following DVPSFormer's Explicit Scene Discretization: treat panoptic queries as discrete scene representations, use a D2C (discrete-to-continuous) head for depth, and add Online Majority Voting for temporal classification consistency. This unifies semantic segmentation, instance detection, and depth estimation in one pass.

### SUGGESTION-CV-004: Remove NMS — Adopt End-to-End Set Prediction
Replace the NMS-based detection paradigm with DETR-style one-to-one Hungarian matching. This eliminates the `nms_threshold` config parameter entirely and provides O(1) post-processing regardless of vocabulary size.

### SUGGESTION-CV-005: Add Uncertainty-Aware Multi-Modal Fusion
Following UP-Fuse's pattern: implement an uncertainty head that predicts per-feature reliability under camera degradation. When fusing LiDAR + camera, dynamically weight modalities based on learned uncertainty maps. Store in KB as a fusion configuration profile.

### SUGGESTION-CV-006: Design Prompt API for Interactive Segmentation
Define a unified prompt interface:
```rust
pub enum SegmentationPrompt {
    Point { x: f32, y: f32, positive: bool },
    Box { x1: f32, y1: f32, x2: f32, y2: f32 },
    Mask { mask: BitVec, frame_idx: usize },
    Text { instruction: String },
}
```
Integrate with SAM 2's streaming architecture for video-level prompt propagation.

### SUGGESTION-CV-007: Add Zero-Shot Pseudo-Label Generation
Implement a `PseudoLabelGenerator` that produces panoptic pseudo-labels from unlabeled video using:
1. Self-supervised depth estimation (monocular)
2. Optical flow for motion segmentation
3. Self-supervised visual features for semantic grouping
4. Video DropLoss-inspired training on pseudo-labels

This enables domain adaptation without manual annotation — critical for NeoTrix's self-evolution philosophy.

---

## 4. Priority Matrix

| Defect | Severity | Effort | Priority |
|--------|----------|--------|----------|
| CV-001 (No open-vocab detection) | HIGH | MEDIUM | P1 |
| CV-002 (No video temporal memory) | HIGH | HIGH | P1 |
| CV-003 (No panoptic segmentation) | HIGH | HIGH | P2 |
| CV-004 (NMS dependency) | MEDIUM | LOW | P2 |
| CV-005 (No multi-modal fusion) | MEDIUM | HIGH | P3 |
| CV-006 (No interactive prompts) | MEDIUM | MEDIUM | P2 |
| CV-007 (No zero-shot bootstrapping) | LOW | HIGH | P3 |

---

## 5. Meta-Observation

The 2026 CV landscape has converged on three architectural principles that NeoTrix's `nt_sense_cv.rs` completely misses:
1. **NMS-free end-to-end prediction** — DETR-style one-to-one matching eliminates post-processing bottlenecks
2. **Streaming memory for video** — SAM 2's memory bank enables per-object identity persistence across frames
3. **Prompt-based interaction** — All foundation models now accept multi-modal prompts (clicks/boxes/text) for interactive refinement

NeoTrix's current CV pipeline is a static, closed-set, single-image stub. The gap is architectural, not just missing features — the entire detection/segmentation paradigm needs to shift from "process each frame independently" to "maintain a persistent, promptable, open-vocabulary scene representation."
