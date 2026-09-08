# Iteration Batch 510 — Rendering, 3D Graphics, Hardware Advances (2026-09-06)

## Sources Cited

| # | Source | Year | Domain |
|---|--------|------|--------|
| S1 | Lin et al., "ReSTIR PT Enhanced," ACM SIGGRAPH 2026 (DOI:10.1145/3804494) | 2026 | Rendering |
| S2 | Padilla et al., "HiPR: Hierarchical Progressive Rendering," SIGGRAPH Real-Time Live! 2026 | 2026 | Rendering |
| S3 | NVIDIA RTX Path Tracing SDK v1.8.1, github.com/NVIDIAGameWorks/Path-Tracing-SDK | 2026 | Rendering |
| S4 | CAPCOM/NVIDIA GDC 2026: "Real-Time Path Tracing in RE ENGINE" (RE Engine + SER + ReSTIR GI + DLSS RR) | 2026 | Rendering |
| S5 | SIGGRAPH 2026 Course: "Advances in Real-Time Rendering in Games" (Activision, EA, Sony, IOI, Roblox, Meta) | 2026 | Rendering |
| S6 | Olejnik, "Variable Rate Ray Tracing in Call of Duty: MW4," SIGGRAPH 2026 | 2026 | Rendering |
| S7 | Greenberg, "ORCA: Online Radiance Cache Acceleration," SIGGRAPH 2026 | 2026 | Rendering |
| S8 | Zhang et al., "AnchorSplat: Feed-Forward 3DGS with 3D Geometric Priors," CVPR 2026 | 2026 | 3D Graphics |
| S9 | Bui et al., "EcoSplat: Efficiency-controllable Feed-forward 3DGS," CVPR 2026 | 2026 | 3D Graphics |
| S10 | Veichta et al., "ZipSplat: Fewer Gaussians, Better Splats," arXiv 2606.05102, 2026 | 2026 | 3D Graphics |
| S11 | Polansky et al., "Eulerian Gaussian Splatting using Hashed Probability Pyramids," CVPR 2026 | 2026 | 3D Graphics |
| S12 | Jeong et al., "3D Gaussian Splatting at Arbitrary Resolutions with Compact Proxy Anchors," CVPR 2026 | 2026 | 3D Graphics |
| S13 | Wu et al., "ViewSplat: View-Adaptive 3D Gaussian Splatting," CVPR 2026 | 2026 | 3D Graphics |
| S14 | Wu et al., "iSplat: Iterative Learning for Fine-Grained Gaussian Splatting," CVPR 2026 | 2026 | 3D Graphics |
| S15 | KhronosGroup, "Vulkan Roadmap 2026" (github.com/KhronosGroup/Vulkan-Docs) | 2026 | Hardware/API |
| S16 | Khronos, "Vulkan Descriptor Heaps (VK_EXT_descriptor_heap)" + NVIDIA blog (2026-06-25) | 2026 | Hardware/API |
| S17 | W3C, "WebGPU Candidate Recommendation Draft" (W3C TR/webgpu, 2026-09-01) | 2026 | Hardware/API |
| S18 | Khronos, "WebGL+WebGPU SIGGRAPH 2026" (webgpu.h stable, Compat Mode, cooperative matrices, bindless) | 2026 | Hardware/API |
| S19 | Khronos, "SIGGRAPH 2026: How to write a Vulkan application in 2026" (shader objects, dynamic rendering, descriptor heap) | 2026 | Hardware/API |
| S20 | NVIDIA RTX Path Tracing SDK: ReSTIR DI/GI, RTXDI, OMM, NRD integration | 2026 | Rendering |

---

## Defects Found in NeoTrix Design (Against 2026 State-of-the-Art)

### DEF-510-R1: No Scene-Change-Aware Render Scheduling
**Gap**: CONTEXT.md defines `PerceptionBridge` and `GWT` for attention routing, but NeoTrix has no equivalent for rendering: no mechanism to prioritize which pixels/tiles to re-render after a scene modification.
**Evidence**: HiPR (S2) demonstrates that hierarchical progressive rendering with light-path dependency tracking + perceptual salience weighting makes interactive path tracing feel real-time. Without this, any NeoTrix-backed render pipeline wastes compute re-rendering unchanged regions.
**Suggestion**: Add `RenderSchedulingBridge` as a new perception-layer component. Implement tile-based priority scheduling that tracks light-path dependencies from modified scene elements outward, ordered by perceptual impact. Connect to GWT attention routing for cross-domain prioritization.

### DEF-510-R2: Missing Variable Rate Ray Tracing (VRRT)
**Gap**: No concept of dynamic per-pixel ray budget allocation exists in the architecture.
**Evidence**: Call of Duty MW4 (S6) deploys VRRT with GPU-driven frame-level scheduler that redistributes rays spatially while maintaining constant total ray count per frame. This eliminates performance spikes under heavy camera motion.
**Suggestion**: Extend `nt_physical::video_post_processor` with a `VariableRateRayTracer` module. Implement temporal gradient pre-pass for disocclusion detection and a GPU-driven scheduler that redistributes rays spatially. Budget guarantee: constant total ray count per frame.

### DEF-510-R3: No Online Radiance Cache (ORCA-style)
**Gap**: No instantaneous, history-independent radiance caching exists. All lighting computations are stateless or depend on temporal history that breaks under dynamic scene changes.
**Evidence**: ORCA (S7) provides a custom radiance cache that requires no temporal history—all data structures are instantaneous. This makes it ideal for fully dynamic scenes and can be integrated into existing path tracers with minimal changes.
**Suggestion**: Implement `NtRadianceCache` as a spatially-varying, history-independent radiance cache. Use screen-space hash grid for cache lookup. Integrate as a first-class primitive in the path tracing pipeline, positioned before denoising.

### DEF-510-R4: No 3DGS Feed-Forward Architecture
**Gap**: NeoTrix has no feed-forward (single-pass, no per-scene optimization) 3D scene reconstruction capability. The design only references per-optimization approaches.
**Evidence**: AnchorSplat (S8), EcoSplat (S9), ZipSplat (S10), and ViewSplat (S13) all demonstrate that feed-forward 3DGS can predict 3D Gaussians from multi-view images in a single forward pass, eliminating the need for per-scene iterative optimization. This is critical for real-time AR/VR and robotic applications.
**Suggestion**: Add `FeedForward3DGS` to `nt_core` as a new knowledge representation primitive. Anchor-based or token-based architecture for predicting Gaussians from multi-view images without per-scene optimization. Connect to VSA HyperCube for symbolic scene representation.

### DEF-510-R5: No Adaptive Gaussian Density Control
**Gap**: The architecture has no mechanism for adaptive density control of 3D Gaussian primitives—no heuristic or learned method for splitting, merging, or pruning Gaussians based on scene complexity.
**Evidence**: Eulerian Gaussian Splatting (S11) replaces all hand-tuned Adaptive Density Control (ADC) heuristics with a probabilistic framework where primitive locations are samples from a learnable density. Hashed Probability Pyramids enable gradient-based control over primitive population density.
**Suggestion**: Replace any future ADC heuristics with an Eulerian probabilistic density field. Implement `HashedProbabilityPyramid` for memory-efficient, globally-normalized density control. Use control variates for low-variance gradient estimation during stochastic training.

### DEF-510-R6: No Resolution-Adaptive Rendering
**Gap**: No support for continuous resolution adaptation in scene representations. All representations are tied to a fixed training resolution.
**Evidence**: "3D Gaussian Splatting at Arbitrary Resolutions" (S12) demonstrates continuous resolution adaptability via resolution-embedding fused into anchor features and a pixel coverage gate. This is critical for AR/VR zoom scenarios.
**Suggestion**: Add `ResolutionAdaptiveRenderer` to `nt_physical`. Implement resolution-embedding in anchor features, pixel coverage gate for dynamic Gaussian activation at arbitrary scales, and residual anchor predictor for compact storage with on-demand leaf anchor reconstruction.

### DEF-510-R7: No View-Dependent Attribute Modulation
**Gap**: No mechanism for viewpoint-conditioned modulation of rendering attributes. View-dependent effects (specular highlights, reflections) cannot be handled without per-scene optimization.
**Evidence**: ViewSplat (S13) demonstrates view-adaptive splatting via per-pixel View MLPs that predict residual offsets conditioned on target camera pose. This achieves SOTA fidelity at 90-154 FPS without per-scene optimization.
**Suggestion**: Implement `ViewAdaptiveModulator` as a hypernetwork architecture. Given canonical Gaussian primitives + target camera pose, predict per-pixel residual updates for position, scale, rotation, opacity, and color. Connect to NT-FEEL for emotional rendering of view-dependent effects.

### DEF-510-R8: Missing Vulkan Descriptor Heap Integration
**Gap**: No reference to `VK_EXT_descriptor_heap` in the graphics abstraction layer. The descriptor binding model assumes legacy descriptor sets.
**Evidence**: Khronos (S16) released VK_EXT_descriptor_heap (2026-06-25) as a complete overhaul of Vulkan's descriptor system, providing direct access to descriptor memory. NVIDIA drivers 610+ support it. It substantially simplifies resource binding and is especially useful for ray tracing shaders and dynamic texture indexing.
**Suggestion**: Update the graphics backend abstraction to support descriptor heap mode. Implement dual-mode binding: legacy descriptor sets (compatibility) + descriptor heap (performance). Expose heap-based resource management through a unified `NtDescriptorAllocator` interface.

### DEF-510-R9: Missing WebGPU Compatibility Mode Awareness
**Gap**: No consideration of WebGPU Compatibility Mode (OpenGL ES 3.1 backend) for cross-platform deployment.
**Evidence**: Khronos (S18) reports WebGPU Compatibility Mode runs on >90% of machines that support WebGL2 (96% penetration). Chromium added Compatibility Mode on older Android (2026-02). This is the path to universal GPU compute on the web.
**Suggestion**: If NeoTrix deploys to web platforms, implement a `WebGPUCompatLayer` that targets Compatibility Mode as baseline. Key restriction: limited storage buffer support in vertex shaders. Progressive enhancement to Core Mode when available. Use `webgpu.h` (stable C API, 2025-09) for native+WebAssembly deployment.

### DEF-510-R10: No Cooperative Matrix / ML Hardware Integration
**Gap**: No reference to subgroup-based cooperative matrix multiplication for ML inference acceleration.
**Evidence**: Vulkan Roadmap 2026 (S15) requires `VK_KHR_cooperative_matrix`. WebGPU (S18) is actively standardizing subgroup-based cooperative matrix multiplication for ML. NVIDIA RTX Neural Shading uses neural networks in shaders. This enables GPU-native ML inference without CPU round-trips.
**Suggestion**: Add `NtMLAccelerator` to `nt_physical` that leverages cooperative matrix extensions for on-GPU ML inference. Target use cases: neural denoising, neural radiance caching, neural texture synthesis. Expose through a compute shader interface with workgroup-level matrix multiply operations.

### DEF-510-R11: Missing Opacity Micromaps (OMM) for Alpha Testing
**Gap**: No support for Opacity Micromaps, which accelerate ray tracing of alpha-tested geometry (vegetation, hair, foliage).
**Evidence**: NVIDIA RTX PT SDK v1.8.1 (S3, S20) integrates OMM for fast ray-traced alpha testing. Call of Duty MW4 (S6) uses OMM for performance. This is a key optimization for any scene with complex transparency.
**Suggestion**: Add OMM generation and rendering support to the ray tracing pipeline. Implement as a pre-processing step that encodes per-triangle opacity bitmasks into hardware-accelerated micromaps. Integrate with the BVH construction pipeline.

### DEF-510-R12: No Shader Execution Reordering (SER) Support
**Gap**: No mechanism for Shader Execution Reordering to improve warp coherence in ray tracing shaders.
**Evidence**: DXR 1.2 SER is NVIDIA's recommendation for path-traced games (S4). RE Engine's optimization journey showed SER + bindless resources reduced frame time from 21ms to 13.3ms. Without SER, coherence can drop to 12% active threads/warp.
**Suggestion**: Implement SER integration in the ray tracing pipeline. Key optimization: combine static SRV references into bindless resources to avoid instruction duplication (24K→12K instructions). Use `VK_NV_ray_tracing_reorder` or DXR 1.2 SER depending on backend.

### DEF-510-R13: No Token-Based Scene Compression
**Gap**: No token-based representation for compressing scene primitives. All representations are pixel-aligned or grid-based.
**Evidence**: ZipSplat (S10) achieves 6x fewer Gaussians than pixel-aligned methods by using k-means clustering of visual tokens into compact scene tokens, then decoding each token into a group of Gaussians. A single trained model spans the quality-efficiency curve by adjusting compression ratio at inference.
**Suggestion**: Implement `TokenSceneCompressor` that aggregates multi-view visual features into compact scene tokens via k-means clustering. Decode tokens into Gaussian primitives with unconstrained 3D positions. Support inference-time compression ratio control for quality-efficiency tradeoff.

### DEF-510-R14: No GPU-Driven Frame Scheduling for Stable Performance
**Gap**: No GPU-driven scheduler for stable frame timing under variable workload.
**Evidence**: Call of Duty MW4 (S6) deploys a fully GPU-driven frame-level scheduler that redistributes rays spatially while maintaining constant total ray count per frame, guaranteeing stable GPU cost and eliminating performance spikes even under heavy camera motion.
**Suggestion**: Implement `NtGPUScheduler` as a compute-shader-based frame-level resource allocator. Input: per-pixel complexity estimates from previous frame. Output: ray budget redistribution map. Constraint: constant total ray budget per frame. Integrate with GWT attention routing for cross-domain priority.

### DEF-510-R15: No Denoiser Guide Buffer / Path Space Decomposition
**Gap**: No guide buffer generation or path space decomposition for denoiser integration.
**Evidence**: NVIDIA RTX PT SDK (S3) uses path-space layer decomposition and guide-buffer generation to support real-time denoising (DLSS-RR, NRD ReLAX/ReBLUR). This decomposes the rendering into direct lighting, indirect lighting, and specular layers for per-layer denoising.
**Suggestion**: Implement `NtPathDecomposer` that separates rendering into 3 path-space layers: direct lighting, diffuse indirect, specular indirect. Generate guide buffers encoding per-pixel layer membership. Expose to denoiser integration for layer-aware denoising.

---

## Summary

| Category | Defects | Severity |
|----------|---------|----------|
| Rendering (Ray/Path Tracing) | DEF-510-R1, R2, R3, R11, R12, R14, R15 | HIGH |
| 3D Graphics (Gaussian Splatting) | DEF-510-R4, R5, R6, R7, R13 | HIGH |
| Graphics Hardware/API | DEF-510-R8, R9, R10 | MEDIUM |
| **Total** | **15** | |

## Priority Ranking

1. **DEF-510-R12** (SER) — 2x-3x performance regression without it; foundational for all path tracing
2. **DEF-510-R4** (Feed-Forward 3DGS) — eliminates per-scene optimization bottleneck; enables real-time applications
3. **DEF-510-R2** (VRRT) — stable frame timing under variable workload; production-proven in AAA games
4. **DEF-510-R1** (Render Scheduling) — interactive path tracing requires scene-change-aware prioritization
5. **DEF-510-R8** (Descriptor Heap) — Vulkan 2026 baseline; simplifies resource binding for all rendering
6. **DEF-510-R10** (Cooperative Matrix) — ML-on-GPU foundation; enables neural rendering primitives
7. **DEF-510-R3** (ORCA Cache) — history-independent radiance cache; critical for dynamic scenes
8. **DEF-510-R5** (Eulerian Density) — replaces brittle heuristics with principled optimization
9. **DEF-510-R6** (Resolution-Adaptive) — required for AR/VR zoom scenarios
10. **DEF-510-R13** (Token Compression) — 6x compression with quality-efficiency control
