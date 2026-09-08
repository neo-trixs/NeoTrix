# Iteration Batch 344 — Research Loop

**Date**: 2026-09-06
**Sources**: 10 papers across 3 domains (CVPR 2026, HPCA 2026, arXiv 2026)

---

## Sources Cited

| # | Paper | Venue | URL |
|---|-------|-------|-----|
| S1 | GHPT: Real-Time Relightable Gaussian Splatting using Hybrid Path Tracing | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Bo_GHPT_Real-Time_Relightable_Gaussian_Splatting_using_Hybrid_Path_Tracing_CVPR_2026_paper.pdf |
| S2 | TRON: Tracing Rays to Orchestrate a Neural Renderer for 3D Gaussian Reconstructions | arXiv 2606.11314 | https://arxiv.org/html/2606.11314v1 |
| S3 | GRTX: Efficient Ray Tracing for 3D Gaussian-Based Rendering | HPCA 2026 | https://doi.org/10.1109/hpca68181.2026.11408531 |
| S4 | StaR-KVQA: Structured Reasoning Traces for Implicit-Knowledge VQA | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Wen_StaR-KVQA_Structured_Reasoning_Traces_for_Implicit-Knowledge_Visual_Question_Answering_CVPR_2026_paper.pdf |
| S5 | DyLaR: Dynamic Latent Reasoning for Video Understanding and QA | arXiv 2608.04124 | https://arxiv.org/html/2608.04124 |
| S6 | VQ-VA World: Towards High-Quality Visual Question-Visual Answering | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Gou_VQ-VA_World_Towards_High-Quality_Visual_Question-Visual_Answering_CVPR_2026_paper.pdf |
| S7 | 3PT: 3D-Object Perception Transformer | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Kalra_3D-Object_Perception_Transformer_3PT_CVPR_2026_paper.pdf |
| S8 | Uni3R: Unified 3D Reconstruction and Semantic Understanding via Generalizable Gaussian Splatting | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Sun_Uni3R_Unified_3D_Reconstruction_and_Semantic_Understanding_via_Generalizable_Gaussian_CVPR_2026_paper.pdf |
| S9 | Ov3R: Open-Vocabulary Semantic 3D Reconstruction from RGB Videos | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Gong_Ov3R_Open-Vocabulary_Semantic_3D_Reconstruction_from_RGB_Videos_CVPR_2026_paper.pdf |
| S10 | SAM 3D: 3Dfy Anything in Images | CVPR 2026 | https://openaccess.thecvf.com/content/CVPR2026/papers/Chen_SAM_3D_3Dfy_Anything_in_Images_CVPR_2026_paper.pdf |

---

## Defects Found

### DEF-001: No 3D Gaussian Splatting (3DGS) pipeline in NT-PHYSICAL
**Severity**: High
**Source**: S1 (GHPT), S2 (TRON), S3 (GRTX)
**Observation**: 2026 consensus: 3DGS is the dominant real-time 3D representation (replacing NeRF). GHPT achieves real-time relighting via hybrid path tracing on Gaussians; TRON combines Gaussian ray tracing + neural renderer for interactive photoreal rendering at 625ms first frame; GRTX optimizes BVH for Gaussian primitives. NeoTrix's NT-PHYSICAL has `video_post_processor` (frame-level) and `video_temporal_stabilizer` (temporal alignment) but no 3D Gaussian scene representation, no G-buffer rendering, no hybrid path tracing, and no inverse rendering pipeline.
**Gap**: `nt_physical` lacks any 3D scene reconstruction or real-time relighting capability. The architecture only handles 2D frame sequences (video post-processing), not 3D volumetric scene representation.
**Suggestion**: Add `nt_physical::gaussian_splat` module implementing:
- 3DGS scene reconstruction (PGSR-style multi-view depth rendering)
- Hybrid path tracing pipeline (G-buffer → ray tracing → neural refinement)
- Material decomposition (albedo, roughness, metallic maps)
- Environment map relighting interface
- Constellation target: C0→C1

### DEF-002: No Gaussian-Aware Hardware Acceleration Abstraction
**Severity**: Medium
**Source**: S3 (GRTX)
**Observation**: GRTX introduces ray-space transformations to convert anisotropic Gaussians to unit spheres, reducing BVH size and traversal overhead by significant margins. Hardware traversal checkpointing eliminates redundant node visits during multi-round tracing. NeoTrix's NT-PHYSICAL has no abstraction for GPU compute acceleration strategies, no BVH management, and no hardware-specific optimization paths.
**Gap**: No `AccelerationStructure` trait or `RayTracingPipeline` abstraction exists. GPU compute decisions are implicit in downstream platform adapters (ComfyUI/SD WebUI), not in NeoTrix core.
**Suggestion**: Add `nt_physical::ray_accel` module with:
- `trait AccelerationStructure { fn build(primitives: &[Gaussian]) -> BVH; fn traverse(ray: &Ray) -> HitRecord; }`
- Hardware checkpoint abstraction for iterative refinement
- Platform-aware backend selection (Vulkan ray query / Metal / CUDA)

### DEF-003: NT-WORLD PerceptionBridge has no visual reasoning trace support
**Severity**: High
**Source**: S4 (StaR-KVQA), S5 (DyLaR), S6 (VQ-VA World)
**Observation**: StaR-KVQA introduces dual-path structured reasoning traces (symbolic relation paths + natural-language explanations) for visual QA, achieving +11.3% over baselines on OK-VQA. DyLaR demonstrates that adaptive latent reasoning (perception latents → conditional reasoning latents) reduces token usage from 1220→18 while improving accuracy from 54.0→58.2 across 9 video benchmarks. VQ-VA World establishes visual question→visual answer paradigm requiring world knowledge and multi-step reasoning. NeoTrix's PerceptionBridge filters sensory events via `awareness_score()` but provides no mechanism for structured visual reasoning traces, no adaptive reasoning allocation, and no visual→visual answer mode.
**Gap**: PerceptionBridge is a binary attention gate (awareness_score threshold). It cannot: (a) generate structured reasoning traces over visual entities, (b) conditionally allocate reasoning budget based on question complexity, (c) produce visual (image) outputs as answers.
**Suggestion**: Extend PerceptionBridge with:
- `ReasoningTrace` type: dual-path (symbolic relation graph + NL explanation)
- Adaptive routing: perception-only questions skip reasoning latents, complex questions engage full trace
- VQ-VA output mode: when `AnswerMode::Visual`, route through `nt_io::reference_generation` for image synthesis
- KB namespace `domain_nt_world` for trace storage and cross-session learning

### DEF-004: No unified 3D object perception pipeline
**Severity**: High
**Source**: S7 (3PT)
**Observation**: 3PT unifies detection, segmentation, and 6DoF pose estimation in a single framework, achieving +23.6 detection AP and +29.1 pose AP on BOP-Industrial. It uses "early-fusion" (3D-conditioned detection) rather than "late-fusion" (separate embedding + similarity matching). NeoTrix's NT-WORLD has `UnifiedCrawler`, `AssetRegistry`, `MediaAssetRegistry`, but no 3D object perception — no 6DoF pose estimation, no depth-free refinement, no multi-view render-and-compare loop.
**Gap**: NT-WORLD perception is 2D (fetch, parse, classify). No module handles 3D spatial understanding of objects, which is essential for: AR/VR content creation (dynamic漫) , robotic integration (NT-PHYSICAL sensors), and scene reconstruction for video post-production.
**Suggestion**: Add `nt_world::perception_3d` module with:
- 3D-conditioned detection (early-fusion with CAD model priors)
- Multi-view RGB refinement loop (render→compare→MV-PnP→hypothesis selection)
- Integration with `nt_physical::gaussian_splat` for scene-aware detection
- Expose as MCP tool via NT-ACT for downstream agents

### DEF-005: No feed-forward 3D reconstruction with semantic understanding
**Severity**: High
**Source**: S8 (Uni3R), S9 (Ov3R)
**Observation**: Uni3R performs feed-forward 3D reconstruction + open-vocabulary semantic segmentation in a single pass from unposed multi-view images, using Cross-View Transformer to fuse multi-view information. Ov3R extends this to RGB video streams with CLIP-informed reconstruction (CLIP3R) and 2D-3D fused descriptors combining CLIP + DINO + 3D-CLIP features. Both achieve state-of-the-art on RE10K, ScanNet, Mip-NeRF360. NeoTrix has no feed-forward 3D reconstruction — only 2D content extraction via NT-WORLD.
**Gap**: `nt_world` has `UnifiedCrawler` for web content and `AssetRegistry` for managing assets, but no pipeline that takes raw multi-view images/video and produces a unified 3D scene representation with open-vocabulary semantics. This blocks: (a) automatic 3D asset creation for dynamic漫 scenes, (b) scene understanding for robot navigation (NT-PHYSICAL), (c) semantic search over 3D environments.
**Suggestion**: Add `nt_world::scene_reconstruction` module with:
- Cross-View Transformer for multi-view fusion
- 3DGS + semantic feature field output (per-Gaussian semantic embedding)
- Open-vocabulary query interface: `fn query_scene(scene: &Scene, text: &str) -> Vec<SemanticSegment>`
- Integration with KB `domain_nt_world` for persistent 3D scene storage

### DEF-006: NT-IO lacks visual question→visual answer (VQ-VA) mode
**Severity**: Medium
**Source**: S6 (VQ-VA World)
**Observation**: VQ-VA World demonstrates that open-source models can achieve 53.06 on IntelligentBench (vs 82.64 GPT-Image) when trained with 1.8M high-quality interleaved image-text samples. The paradigm: given an image + question, produce an image answer (not text). NeoTrix's NT-IO has `ReferenceBasedGeneration` and `ReferenceVideoMode` (image→image, video→video), but no mode where the answer itself is a generated image conditioned on both a source image and a knowledge-requiring question.
**Gap**: `nt_io::reference_generation` handles style transfer and reference-based generation, but has no VQ-VA pipeline that: (a) reasons over world knowledge, (b) generates visual answers to visual questions, (c) integrates with Knowledge Base for fact grounding.
**Suggestion**: Add `nt_io::visual_answer_generation` module:
- Pipeline: question → knowledge retrieval (KB) → reasoning trace → diffusion generation
- Training data: integrate VQ-VA World's 1.8M samples or equivalent web-scraped pairs
- Expose as MCP tool: `visual_answer(image_url, question) -> image_url`

### DEF-007: NT-ACT has no single-image-to-3D reconstruction capability
**Severity**: Medium
**Source**: S10 (SAM 3D)
**Observation**: SAM 3D predicts full 3D shape, texture, and layout from a single image using a generative model trained on ~1M annotated images with 3.14M untextured meshes. Achieves 5:1 human preference win rate over prior SOTA on real-world objects. NeoTrix's NT-ACT has `ProductionOrchestrator` and `ModelAdapter` but no single-image-to-3D reconstruction pipeline.
**Gap**: No module in NT-ACT or NT-WORLD can take a single image and produce a complete 3D asset (shape + texture + layout). This is critical for: (a) generating 3D props for dynamic漫 scenes from concept art, (b) populating virtual environments, (c) robotics manipulation (NT-PHYSICAL).
**Suggestion**: Add `nt_act::single_image_to_3d` module:
- SAM 3D-style generative reconstruction: image → shape S + texture T + pose R + location t + scale s
- Integration with `nt_world::media_asset_registry` for asset storage
- Integration with `nt_physical::gaussian_splat` for scene insertion

### DEF-008: Cross-domain 3D perception gap between NT-WORLD and NT-PHYSICAL
**Severity**: High
**Source**: S7 (3PT), S8 (Uni3R), S9 (Ov3R)
**Observation**: Uni3R and Ov3R both demonstrate that reconstruction and semantic understanding must be unified, not decoupled. 3PT shows that detection, segmentation, and pose estimation must be jointly trained for cross-domain generalization. NeoTrix's architecture has NT-WORLD (perception) and NT-PHYSICAL (embodiment) as separate layers with no shared 3D representation. NT-WORLD produces 2D content; NT-PHYSICAL handles sensors/motors but has no 3D scene understanding.
**Gap**: The L2→L3 boundary has no 3D scene representation bridge. Information flows: NT-WORLD (2D) → PerceptionBridge → NT-CORE (L5). But NT-PHYSICAL (L3) cannot consume 2D perception outputs for 3D manipulation. There is no shared 3D scene graph that both domains can read/write.
**Suggestion**: Add a `SceneGraphBridge` connecting L2 and L3:
- Shared data structure: `SceneGraph` with nodes (objects, camera, lights) and edges (spatial relations)
- NT-WORLD populates from reconstruction (Uni3R/Ov3R) or single-image (SAM 3D)
- NT-PHYSICAL consumes for manipulation planning, collision avoidance, sensor placement
- Bidirectional: NT-PHYSICAL sensor data enriches SceneGraph for NT-WORLD perception

---

## Summary

| # | Defect | Severity | Affected Domain |
|---|--------|----------|-----------------|
| DEF-001 | No 3D Gaussian Splatting pipeline | High | NT-PHYSICAL |
| DEF-002 | No Gaussian-aware HW acceleration | Medium | NT-PHYSICAL |
| DEF-003 | PerceptionBridge has no visual reasoning traces | High | NT-WORLD / NT-CORE |
| DEF-004 | No unified 3D object perception | High | NT-WORLD |
| DEF-005 | No feed-forward 3D reconstruction + semantics | High | NT-WORLD |
| DEF-006 | NT-IO lacks VQ-VA mode | Medium | NT-IO |
| DEF-007 | No single-image-to-3D in NT-ACT | Medium | NT-ACT |
| DEF-008 | Cross-domain 3D gap (L2↔L3) | High | NT-WORLD↔NT-PHYSICAL |

**Pattern**: NeoTrix's 2026 design is optimized for 2D content creation (dynamic漫, video post-processing) but lacks the 3D scene understanding, reconstruction, and real-time rendering capabilities that are now standard in CVPR 2026 research. The six-layer architecture's L2-L3 boundary is the critical chokepoint — it assumes 2D perception feeds into physical embodiment, but 2026 research shows these must share a unified 3D representation.

**Priority Actions**:
1. **P0**: Add `nt_physical::gaussian_splat` (DEF-001) — unlocks real-time 3D rendering for all downstream modules
2. **P0**: Add `SceneGraphBridge` (DEF-008) — unblocks L2↔L3 3D data flow
3. **P1**: Add `nt_world::scene_reconstruction` (DEF-005) — feed-forward 3D from multi-view
4. **P1**: Extend PerceptionBridge with reasoning traces (DEF-003) — visual reasoning capability
5. **P2**: Add VQ-VA and single-image-to-3D (DEF-006, DEF-007) — content creation pipeline
