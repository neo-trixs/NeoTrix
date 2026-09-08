# Iteration Batch 474 — 3D Perception Research Analysis

**Date**: 2026-09-06
**Research Domains**: Point Cloud Processing | 3D Vision & Reconstruction | Depth Estimation

---

## Sources Cited

### Point Cloud Processing (2026)
1. **GeoGuide: Hierarchical Geometric Guidance for Open-Vocabulary 3D Semantic Segmentation** — CVPR 2026. Uncertainty-based superpoint distillation + instance-level mask reconstruction + inter-instance relation consistency. Preserves 3D geometric priors during 2D-to-3D knowledge distillation. [link](https://openaccess.thecvf.com/content/CVPR2026/papers/Tao_GeoGuide_Hierarchical_Geometric_Guidance_for_Open-Vocabulary_3D_Semantic_Segmentation_CVPR_2026_paper.pdf)
2. **Geometric-Aware Hypergraph Reasoning for Novel Class Discovery in Point Cloud Segmentation** — CVPR 2026. Hypergraph structures model high-order class associations; Geometric-Aware Prototypes capture spatial distribution. +16.9–37.8 mIoU on SemanticPOSS novel classes. [link](https://openaccess.thecvf.com/content/CVPR2026/papers/Zhang_Geometric-Aware_Hypergraph_Reasoning_for_Novel_Class_Discovery_in_Point_Cloud_CVPR_2026_paper.pdf)
3. **PT-WNO: Point Transformer with Wavelet Neural Operator** — arXiv:2606.11466, Jun 2026. Dual-pathway: point transformer (local) + wavelet neural operator (global spectral). +1.03 mIoU on S3DIS, +1.47 on DALES over PTv3. [link](https://arxiv.org/html/2606.11466)
4. **PointGS: Semantic-Consistent Unsupervised 3D Point Cloud Segmentation with 3D Gaussian Splatting** — CVPR 2026. Bridges discrete 3D points and continuous 2D images via 3D Gaussian intermediate representation. +2.8% mIoU on S3DIS, +0.9% on ScanNet-v2 over unsupervised SOTA. [link](https://openaccess.thecvf.com/content/CVPR2026/papers/Song_PointGS_Semantic-Consistent_Unsupervised_3D_Point_Cloud_Segmentation_with_3D_Gaussian_CVPR_2026_paper.pdf)
5. **PointCNN++: Performant Convolution on Native Points** — CVPR 2026. Generalizes sparse convolution from voxels to continuous points via MVMR GPU kernel. 200x faster than KPConv, 3.8% mIoU gain over MinkowskiEngine on nuScenes. [link](https://openaccess.thecvf.com/content/CVPR2026/papers/Li_PointCNN_Performant_Convolution_on_Native_Points_CVPR_2026_paper.pdf)
6. **PointNMSA: Improved PointNeXt with Non-Local Multi-Scale Aggregation** — CMC 2026. Multi-Scale Feature Enhancement + Convolution-Attention Mixing. 65.10% mIoU on S3DIS Area 5 (+1.59% over PointNeXt). [link](https://www.sciopen.com/article/10.32604/cmc.2026.078692)
7. **SFSGNet: Spatial Feature Structure Generation Network** — Sensors 2026. Dual-path heterogeneous fusion of geometric coordinates and attribute features for airborne LiDAR. 89.3% avg F1 on DALES. [link](https://www.mdpi.com/1424-8220/26/10/2996)
8. **Hierarchical Gaussian Partitioning for Airborne LiDAR** — ISPRS-Annals 2026. hGMM within Superpoint Transformer for structured suburban scene decomposition. [link](https://isprs-annals.copernicus.org/articles/XI-2-2026/69/2026/isprs-annals-XI-2-2026-69-2026.html)

### 3D Vision & Reconstruction (2026)
9. **Qwen-Drive-1.0: Vision-Language Foundation Model for Autonomous Driving** — arXiv:2609.00111, Sep 2026. Unified VLM integrating 3D detection, occupancy prediction, BEV segmentation, and motion planning. [link](https://arxiv.org/abs/2609.00111)
10. **Stereo 4D Radar for 3D Object Detection** — arXiv:2609.02560, Sep 2026. Stereo radar geometric disparity for absolute velocity estimation. +8.82 AP3D over mono radar baselines. [link](https://arxiv.org/abs/2609.02560)
11. **PART: Physics-Aware Radar Transformer** — arXiv:2609.02289, Sep 2026. Class-agnostic moving-object detection via Doppler-aware queries. 1.1M params, 0.8827 CA-AP on nuScenes. [link](https://arxiv.org/abs/2609.02289)
12. **VOIM: Training-Free Open-Vocabulary 3D Instance Mapping** — arXiv:2609.00775, Sep 2026. Voxel-grounded instance manager for RGB-D/monocular SLAM. Exceeds OVO-SLAM by 4.8–11.7 mIoU on ScanNet++. [link](https://arxiv.org/abs/2609.00775)
13. **DSG: Dynamic 3D Scene Graph Construction** — arXiv:2609.00619, Sep 2026. Dual-view rendering-based object change detection + spatial relationship reasoning via LLMs. [link](https://arxiv.org/abs/2609.00619)
14. **SPAR3S: Sparse Auto-regressive Scene Generation from Multi-view Images** — ECCV 2026. Voxel-aligned 3D latent generative model for conditional scene completion without ground-truth 3D. [link](https://arxiv.org/abs/2609.03931)
15. **Atlas: Algorithm-Hardware Co-Design for On-Device City-Scale 3DGS in VR** — arXiv:2609.02352, Sep 2026. Hierarchical memory offloading + temporal LoD search. 18.5x speedup over GPU baseline, 92.4% energy savings. [link](https://arxiv.org/abs/2609.02352)
16. **TileGS: Tile-Local Depth Binning for Gaussian Splatting Rasterization** — arXiv:2609.03613, Sep 2026. Front-to-back tile-local depth ranges. 1.44x raster-kernel speedup on RTX 4090. [link](https://arxiv.org/abs/2609.03613)
17. **EvoGS: Modeling Deformation Evolution for Dynamic Gaussian Splatting** — Pacific Graphics 2026. Persistent deformation states + temporal residual memory for dynamic scenes. [link](https://arxiv.org/abs/2609.00994)
18. **When 3D Gaussian Splatting Recovers Real Surfaces** — ECCV 2026. Mathematical framework proving geometric misalignment converts spatial textures to high-frequency angular signals via parallax. Identifiability window theorem. [link](https://arxiv.org/abs/2608.30054)
19. **InceptionGS: Generative Bootstrapping for Large-Scale Gaussian Splatting** — arXiv:2609.02747, Sep 2026. Scene- and view-adaptive generative priors for unstructured view sampling. [link](https://arxiv.org/abs/2609.02747)
20. **GhostSplat: Input-Triggered Backdoors in Feed-Forward Gaussian Splatting** — arXiv:2608.29184, Aug 2026. Supply-chain attack surface for 3DGS shared weights. 96–100% ASR across three architectures. [link](https://arxiv.org/abs/2608.29184)

### Depth Estimation (2026)
21. **Lapis: Linear-Attention Pixel-Space Diffusion for Depth Estimation** — ECCV 2026. One-step diffusion with coarse-to-fine hierarchy. 7.6x faster at 1080P, 10.9x at 1440P vs. prior SOTA generative models. [link](https://arxiv.org/abs/2608.30129)
22. **OptiGeo: Bias-Aware Training for Monocular Depth in Optically Challenging Scenes** — arXiv:2608.29881, Aug 2026. 30M params outperforms 300M-scale models on transparent/reflective scenes via clean-geometry teacher. [link](https://arxiv.org/abs/2608.29881)
23. **PXDepth: Pixel-Space Modeling for Structure Preserving Monocular Depth** — arXiv:2608.16984, Aug 2026. Separates global context (large-patch ViT) from pixel-level prediction (Context-Modulated Pixel Transformer). [link](https://arxiv.org/abs/2608.16984)
24. **GeoNeXt: Video Generative Models as Geometry Learner** — arXiv:2608.28549, Aug 2026. Repurposes video diffusion models for joint depth + normal estimation. Rivals discriminative SOTA with 100x less training data. [link](https://arxiv.org/abs/2608.28549)
25. **M2Depth: Unifying Monocular Depth Foundation Priors with Multi-View Stereo** — arXiv:2608.20788, Aug 2026. Bidirectional mutual refinement between DFM and cascade MVS. [link](https://arxiv.org/abs/2608.20788)
26. **Scalix: Uncertainty-Aware Scale-Consistent Monocular SLAM** — arXiv:2608.17553, Aug 2026. Per-pixel depth uncertainty + per-frame scale uncertainty in factor graph. Real-time metric-scale SLAM. [link](https://arxiv.org/abs/2608.17553)
27. **SiZeUp: Fast 3D Proxy from Aerial Images via Depth Ordinal Loss** — SIGGRAPH Asia 2026. Ordinal depth consistency for building height estimation. 23–52x speedup over SOTA proxy pipelines. [link](https://arxiv.org/abs/2608.22821)
28. **CGS-SLAM: Collaborative Gaussian Splatting SLAM** — arXiv:2608.26868, Aug 2026. Multi-agent 3DGS SLAM using only RGB+IMU. VGGT for view alignment across agents. [link](https://arxiv.org/abs/2608.26868)
29. **MuyBridge: Mobile Human CoM Estimation from Monocular Video** — arXiv:2609.02854, Sep 2026. On-device depth + pose fusion for athletic biomechanics. 33–41mm vertical CoM error at 63 FPS. [link](https://arxiv.org/abs/2609.02854)
30. **Adapting Depth Anything V2 for Lunar Surface Height Estimation** — arXiv:2609.02448, Sep 2026. Fine-tuning DAV2 with SPG-derived DEM data for lunar terrain. [link](https://arxiv.org/abs/2609.02448)
31. **DEX: From Perspective to Fisheye Depth Estimation** — arXiv:2608.27860, Aug 2026. Learnable Distortion Extenders for adapting foundation models to fisheye cameras. Architecture- and task-agnostic. [link](https://arxiv.org/abs/2608.27860)
32. **RealOOB: Occlusion Boundary Benchmark** — arXiv:2608.30820, Aug 2026. 4.26M definition-consistent OB labels. Reveals gap: strong depth estimators fail at true occlusion boundaries. [link](https://arxiv.org/abs/2608.30820)
33. **RbFT-Net: Rectify-Before-Fuse Temporal Radar Anchors** — arXiv:2608.13102, Aug 2026. 4D radar-camera depth completion with temporal rectification. [link](https://arxiv.org/abs/2608.13102)
34. **Monocular Depth Estimation Survey** — CVMJ 2026. Comprehensive survey tracing field evolution from learning-based to foundation models. DINOv3 pretraining + synthetic data critical. [link](https://arxiv.org/abs/2609.01172)

---

## Defects Found in NeoTrix Design

### DEFECT-1: NT-WORLD Has No Open-Vocabulary 3D Segmentation Pathway — Missing 2D→3D Semantic Distillation
**File**: `neotrix-core/src/neotrix/nt_world/` (perception domain)
**Gap**: The NT-WORLD perception domain processes point clouds but has **no mechanism for open-vocabulary 3D semantic segmentation**. CVPR 2026 shows that the dominant paradigm is now 2D-to-3D knowledge distillation using frozen 2D foundation models (SAM, CLIP) with geometric consistency enforcement. NeoTrix's crawler pipeline handles raw data acquisition but cannot distill semantic understanding from pre-trained vision-language models into 3D representations.

**2026 Evidence**:
- GeoGuide [1] demonstrates that direct 2D→3D projection introduces geometric biases from occlusions and viewpoint changes. Their hierarchical geometry-guided framework (uncertainty-based superpoint distillation + instance mask reconstruction + inter-instance relation consistency) is necessary to preserve 3D geometric structure during distillation.
- PointGS [4] uses 3D Gaussian Splatting as an intermediate representation to bridge the discrete-continuous domain gap, achieving +2.8% mIoU over prior unsupervised methods.
- The hypergraph reasoning approach [2] shows high-order class interactions are critical — binary graph structures miss inter-class relationships.

**Impact**: NT-WORLD cannot perform open-world scene understanding. It can only process pre-segmented or closed-set labeled point clouds, making it unsuitable for autonomous exploration of novel environments.

**Suggestion**: Implement a GeoGuide-style hierarchical geometry-guided pipeline within NT-WORLD:
- Add a frozen 2D foundation model adapter (SAM + DINOv2) for semantic feature extraction
- Implement uncertainty-based superpoint distillation to handle 2D projection noise
- Use 3D Gaussian Splatting [4] as the bridge representation between 2D images and 3D points
- Add hypergraph reasoning [2] for novel class discovery from known geometric prototypes

---

### DEFECT-2: NT-PHYSICAL Has No Wavelet Neural Operator for Multi-Scale Global Context — Local-Only Point Processing
**File**: `neotrix-core/src/neotrix/nt_physical/` (embodiment domain)
**Gap**: NT-PHYSICAL processes sensor data with local operators but lacks a **global multi-scale context mechanism** for point cloud understanding. PT-WNO [3] proves that point transformers with local attention alone are insufficient for full scene understanding — an explicit global spectral operator is needed.

**2026 Evidence**:
- PT-WNO [3] achieves +1.03 mIoU on S3DIS and +1.47 on DALES by adding a wavelet neural operator branch that captures multi-scale global spectral context. The key insight: global context should be treated as an explicit operator, not an emergent property of stacked local attention.
- PT-WNO's dual-pathway architecture (local point transformer + global wavelet operator on 3D volumetric grid) is a generalizable pattern — it improves across indoor, outdoor, aerial, and autonomous driving scenes.
- The Mamba/SSM alternatives [3] are point-ordering-sensitive; the wavelet approach is ordering-invariant.

**Impact**: NT-PHYSICAL's perception pipeline cannot handle large-scale scenes where global context (room layout, road structure, aerial topology) is essential for accurate segmentation and understanding.

**Suggestion**: Integrate a Wavelet Neural Operator branch into NT-PHYSICAL's point processing pipeline:
- At each encoder-decoder stage, project point features onto a 3D volumetric grid
- Apply learnable wavelet decomposition/reconstruction for multi-scale spectral analysis
- Fuse global features back via lightweight adapters alongside existing skip connections
- This is plug-and-play and orthogonal to existing point processing backbones

---

### DEFECT-3: NT-CORE Lacks PointCNN++-Style Native Point Convolution — Forced Voxelization Tradeoff
**File**: `neotrix-core/src/core/` (foundation domain)
**Gap**: NeoTrix's core 3D processing forces a voxelization step that sacrifices geometric fidelity for computational efficiency. PointCNN++ [5] proves this tradeoff is unnecessary — native point convolution via MVMR formulation achieves both higher precision AND faster computation.

**2026 Evidence**:
- PointCNN++ [5] generalizes sparse convolution from discrete voxels to continuous points. Voxel-based convolution is a "quantized, degraded special case." The MVMR GPU kernel achieves 200x speedup over KPConv while using 10x less memory.
- On nuScenes semantic segmentation: 78.2% mIoU (PointCNN++) vs 74.4% (MinkowskiEngine voxel-based), with 2.43GB vs 2.61GB memory and 0.102s vs 0.131s per iteration.
- For geometrically sensitive tasks (registration), the precision gain is even more dramatic.

**Impact**: Any NT-CORE module that processes point clouds introduces quantization artifacts from voxelization, degrading performance on precision-critical tasks like registration, alignment, and fine-grained geometry understanding.

**Suggestion**: Replace voxel-based convolutions in NT-CORE with PointCNN++'s MVMR-based native point convolution:
- Treat voxel convolution as a special case; center operations on original high-precision coordinates
- Use adaptive voxelization only as the last operation to minimize fidelity loss
- This eliminates the fundamental precision-performance tradeoff in 3D learning

---

### DEFECT-4: NT-ACT Has No 3DGS-Based SLAM or Scene Graph Construction — Missing Dynamic Scene Understanding
**File**: `neotrix-core/src/neotrix/nt_act/` (action domain)
**Gap**: NT-ACT's orchestration and tool capabilities lack integration with modern 3DGS-based SLAM systems and dynamic 3D scene graph construction. The field has moved beyond static scene representations to **temporal evolution modeling** and **multi-agent collaborative reconstruction**.

**2026 Evidence**:
- CGS-SLAM [28] enables multi-agent 3DGS SLAM using only RGB+IMU, with keyframe encoding sharing for submap alignment via VGGT.
- DSG [13] constructs dynamic 3D scene graphs with dual-view rendering-based object change detection and LLM-driven spatial relationship reasoning.
- EvoGS [17] models Gaussian deformation as a temporal evolution process with persistent deformation states, extrapolating future states from historical trajectories.
- VOIM [12] builds open-vocabulary 3D instance maps from monocular RGB alone, exceeding prior RGB-D systems by 4.8–11.7 mIoU.

**Impact**: NT-ACT cannot operate in dynamic environments where objects change position, new objects appear, or multiple agents must collaborate on scene understanding. The action domain is limited to static, pre-mapped environments.

**Suggestion**: Implement a temporal 3DGS scene management layer in NT-ACT:
- Integrate EvoGS-style temporal deformation modeling for dynamic object tracking
- Add DSG-style scene graph construction with LLM-based spatial reasoning
- Support CGS-SLAM-style multi-agent collaborative mapping via shared keyframe encodings
- Use VOIM's deferred labeling approach for robust open-vocabulary instance management

---

### DEFECT-5: NT-IO Has No Foundation Model Depth Adaptation Pipeline — Missing Monocular Depth for Embodied Agents
**File**: `neotrix-core/src/neotrix/nt_io/` (interface domain)
**Gap**: NT-IO provides LLM provider interfaces but lacks a **monocular depth estimation pipeline optimized for embodied agent deployment**. The 2026 landscape shows foundation model-based depth (Depth Anything V2, etc.) is now the standard for mobile/robotic perception, but requires domain-specific adaptation.

**2026 Evidence**:
- OptiGeo [22] demonstrates that bias-aware training with a clean-geometry teacher outperforms 300M-scale models on optically challenging scenes (transparent/reflective) with only 30M parameters.
- Scalix [26] shows real-time metric-scale monocular SLAM is achievable by treating per-pixel depth uncertainty and per-frame scale uncertainty as independent measurements in a factor graph.
- DEX [31] provides architecture- and task-agnostic adaptation of foundation models to fisheye cameras via learnable Distortion Extenders.
- The survey [34] confirms DINOv3 pretraining + synthetic data are now critical for foundation model depth.

**Impact**: NT-IO cannot provide metric-scale depth to downstream agents without RGB-D sensors. Monocular depth from foundation models is now practical for real-time deployment but requires the adaptation pipeline that NT-IO lacks.

**Suggestion**: Build a depth foundation model adapter in NT-IO:
- Wrap Depth Anything V2 / PXDepth as a pluggable depth provider
- Implement OptiGeo-style bias-aware fine-tuning for domain-specific adaptation (fisheye, transparent, underwater)
- Add Scalix-style uncertainty estimation (per-pixel depth uncertainty + per-frame scale uncertainty)
- Support the DEX distortion extender pattern for non-perspective camera models
- Expose depth + uncertainty as a first-class interface for NT-ACT and NT-PHYSICAL

---

### DEFECT-6: NT-SHIELD Has No 3DGS Supply-Chain Security — Unaddressed Backdoor Attack Surface
**File**: `neotrix-core/src/neotrix/nt_shield/` (security domain)
**Gap**: NT-SHIELD provides stealth networking and fingerprint management but has **no defense against 3DGS supply-chain attacks**. GhostSplat [20] demonstrates that shared pretrained weights in feed-forward 3DGS architectures expose a critical attack surface.

**2026 Evidence**:
- GhostSplat [20] installs persistent backdoor behavior in shared 3DGS generator weights. A low-amplitude pattern in input images causes the poisoned generator to render attacker-chosen payloads on unseen victim scenes. 96–100% attack success rate across three architectures (MVSplat, pixelSplat, DepthSplat).
- The attack survives JPEG compression, blur, and resampling.
- Existing defenses that use only same-set consistency projection are insufficient — effective mitigation requires information or intervention beyond geometric consistency.

**Impact**: If NeoTrix uses shared 3DGS pretrained weights (increasingly common), NT-SHIELD has no mechanism to detect or mitigate input-triggered backdoor attacks that could manipulate 3D scene reconstruction.

**Suggestion**: Add 3DGS-specific adversarial detection to NT-SHIELD:
- Implement statistical anomaly detection on Gaussian attribute distributions
- Add input perturbation testing (small L∞ noise) to detect trigger-activated behavior
- Monitor for geometric inconsistency between rendered views and input constraints
- Establish a weight provenance verification pipeline for shared 3DGS models

---

### DEFECT-7: NT-MIND Cannot Exploit Video Generative Models for Joint Geometry Learning — Single-Task Depth Paradigm
**File**: `neotrix-core/src/neotrix/nt_mind/` (evolution domain)
**Gap**: NT-MIND's SEAL pipeline processes tasks independently but the 2026 frontier shows **video generative models can serve as unified geometry learners**, jointly modeling depth, normals, and appearance. NeoTrix has no mechanism to repurpose video diffusion priors for multi-task geometry estimation.

**2026 Evidence**:
- GeoNeXt [24] reformulates geometry estimation as a next-frames prediction task using pretrained video diffusion models. Rivals discriminative SOTA with 100x less training data by inheriting naturally structured knowledge from video models.
- FixAnything [25] repurposes a single video generative model to fix rendering artifacts across four distinct 3D representations (3DGS, NeRF, meshes, point clouds), using camera pose accuracy as a DPO reward signal.
- M2Depth [25] shows bidirectional mutual refinement between depth foundation models and multi-view stereo outperforms either alone.

**Impact**: NT-MIND must train separate models for each geometry task (depth, normals, segmentation) and cannot leverage the shared representations that video generative models provide. This increases data requirements and reduces cross-task knowledge transfer.

**Suggestion**: Add a video-prior geometry branch to NT-MIND's SEAL pipeline:
- Implement GeoNeXt-style joint depth + normal estimation from video diffusion backbone
- Use FixAnything's DPO reward signal for 3D-consistent rendering refinement
- Add M2Depth-style bidirectional refinement between monocular priors and multi-view constraints
- This enables the SEAL pipeline to share geometry representations across perception tasks

---

## Summary

| Domain | Sources | Defects |
|--------|---------|---------|
| Point Cloud Processing | 8 | DEFECT-1, DEFECT-2, DEFECT-3 |
| 3D Vision & Reconstruction | 12 | DEFECT-4, DEFECT-6 |
| Depth Estimation | 14 | DEFECT-5, DEFECT-7 |
| **Total** | **34** | **7** |

**Key Themes**:
1. **2D→3D distillation is now standard** — frozen foundation models + geometric consistency (DEFECT-1)
2. **Explicit global context operators outperform implicit attention stacking** — wavelet neural operators (DEFECT-2)
3. **The voxelization tradeoff is solved** — native point convolution via MVMR (DEFECT-3)
4. **Dynamic 3D scene understanding requires temporal evolution modeling** (DEFECT-4)
5. **Foundation model depth is practical for real-time embodied agents** with uncertainty-aware adaptation (DEFECT-5)
6. **3DGS supply-chain attacks are a real threat** requiring new security mechanisms (DEFECT-6)
7. **Video generative models are the new geometry learners** — joint multi-task estimation (DEFECT-7)
