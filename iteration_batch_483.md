# Iteration Batch 483 — Rendering & Light Transport Research

**Date:** 2026-09-06
**Research Domains:** Ray Tracing, Rasterization, Global Illumination

---

## Sources Cited

### Ray Tracing / Path Tracing (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| 1 | NVIDIA GDC 2026 — Path Tracing Roadmap (windowsreport.com, 2026-03-13) | NVIDIA claims 10,000× path tracing improvement from Pascal→RTX, targeting 1,000,000× with AI-driven RT core + Tensor core + DLSS advances. Future GPUs combine neural rendering with hardware RT for film-quality real-time. |
| 2 | SIGGRAPH 2026 Advances in Real-Time Rendering (advances.realtimerendering.com) | **ORCA (Online Radiance Cache Acceleration)** — temporal-history-independent radiance cache that dramatically speeds path tracing. **Variable Rate Ray Tracing (VRRT)** — gradient pre-pass + disocclusion detection + GPU-driven frame-level ray scheduler redistributes rays spatially at constant total ray budget per frame. |
| 3 | NVIDIA Technical Blog (2026-03-12, 2026-06-25) | RTX Mega Geometry foliage system for large natural scenes; path-traced hair rendering on RTX 50 series; OptiX Toolkit for debugging RT apps. |
| 4 | GPU Reporter (2026-09-01) | Mesh shaders replacing parts of vertex/index pipeline; VRS for peripheral/low-motion regions; temporal accumulation + denoising from sparse samples. |
| 5 | GPU Reporter (2026-01-04/06) | Neural denoisers predict clean images from noisy samples; WebGPU bringing low-level GPU to browsers; NeRF/learned scene representations for view synthesis. |
| 6 | SuperRenders (2026-07-17) | 800+ games support RT; convergence of real-time and offline rendering nearly complete; hybrid rasterized+RT remains standard. |
| 7 | Metavert (2026-01-22) | AI-assisted upscaling (DLSS) makes full path tracing viable at lower sample counts; neural rendering reconstructs high-res from low-res RT images. |

### Rasterization / GPU Rendering (2026)

| # | Source | Key Finding |
|---|--------|-------------|
| 8 | GPU Reporter (2026-09-01) | Hardware-accelerated ray traversal + mesh processing + texture compression offload from shader cores. Mesh shaders + amplification shaders for flexible work distribution and LOD control. |
| 9 | Fast Gaussian Rasterization (GitHub) | Geometry-shader-based global CUDA sorted 3D Gaussian Splatting rasterizer — 5-10× speedup over vanilla diff-gaussian-rasterization. SH eval in vertex shader as TODO. |
| 10 | TopSwiss (2026-02-19) | Rasterization remains crucial even with ray tracing; culling + Z-buffering foundational. |
| 11 | SIGGRAPH 2026 Papers (realtimerendering.com) | DeepMill++: Neural guidance meets rasterization for accessibility analysis. |

### Global Illumination / Light Transport (2025-2026)

| # | Source | Key Finding |
|---|--------|-------------|
| 12 | Differentiable Light Transport with Gaussian Surfels (SIGGRAPH Asia 2025, arXiv 2509.18497) | Radiosity reformulated on Gaussian surfels in SH coefficient space — enables differentiable GI for both diffuse and specular. 10× speedup over auto-diff. Monte-Carlo + Hybrid solvers for progressive refinement. |
| 13 | TransGI (arXiv 2506.09909) | Real-time dynamic GI via object-centric neural transfer model + real-time radiance-sharing lighting system. |
| 14 | Path-Traced Inverse Rendering with GI in 3D Gaussian Fields (arXiv 2606.09606, June 2026) | Combines 3DGS rasterization with path-traced GI for inverse rendering. Multi-bounce path tracing recovers cleaner albedo than radiosity-only. |
| 15 | Study of GI Model in Real-Time Rendering (IJERT, April 2026) | Convergence of real-time/offline GI quality driven by hardware + algorithms + AI. Neural radiance caching (Müller et al. 2021) now integrated into production engines. |

---

## Defects Identified in NeoTrix Design

### D483-1: No Radiance Cache Strategy for Perception Pipeline
**Severity:** HIGH | **Domain:** NT-WORLD (L2 Perception) + NT-CORE (L5 Cognition)

The NeoTrix design doc defines VSA HyperCube for knowledge representation and PerceptionBridge for attention-gated sensory flow, but has **no radiance caching mechanism** for visual perception processing. The 2026 industry standard (ORCA at SIGGRAPH 2026) demonstrates that temporal-history-independent radiance caches are essential for real-time path-traced perception. NeoTrix's `SensoryIntegrationHub` processes raw sensory input without a persistent, accelerating cache for recurring lighting/visual patterns.

**Impact:** When NT-WORLD crawls visual content or processes rendered scenes, it re-computes lighting transport from scratch each frame instead of leveraging cached radiance data. This creates a performance bottleneck for any visual reasoning task.

**Suggestion:** Add a `RadianceCache` component to `nt_world_sense` that stores pre-computed radiance in SH coefficient space (following the Gaussian surfel approach from Source 12). Wire it to `PerceptionBridge` so that attention-gated visual events can query cached lighting before full path tracing.

---

### D483-2: No Variable Rate Processing for GWT Attention Routing
**Severity:** HIGH | **Domain:** NT-CORE (GWT) + NT-WORLD

The GWT (Global Workspace Theory) attention mechanism broadcasts salient information across specialist modules, but operates at uniform resolution across all input dimensions. The 2026 advancement of **Variable Rate Ray Tracing (VRRT)** (Source 2) shows that selectively increasing sampling in high-saliency regions while reducing in low-saliency regions yields constant total cost with dramatically improved quality in critical areas.

**Impact:** GWT's `SelectiveState` wastes compute by treating all sensory dimensions equally — it cannot focus ray-tracing budget on perceptually important regions of a visual scene.

**Suggestion:** Extend GWT's attention routing with a `VariableRateAdapter` that maps `awareness_score()` to ray/sampling density. High-awareness regions get denser sampling; low-awareness regions get coarser treatment. This directly mirrors VRRT's temporal gradient pre-pass + disocclusion detection approach.

---

### D483-3: Missing Mesh Shader / Amplification Shader Integration
**Severity:** MEDIUM | **Domain:** NT-PHYSICAL (L3) + NT-WORLD (L2)

NeoTrix's `nt_physical` domain defines sensors and motors for embodiment, and `nt_world` handles perception, but neither domain accounts for **mesh shaders** or **amplification shaders** — the 2026 replacement for parts of the traditional vertex/index pipeline (Source 4, 8). These GPU primitives enable flexible work distribution and LOD control essential for complex geometric perception.

**Impact:** When NeoTrix processes large-scale visual data (e.g., 3DGS scenes, architectural models), it falls back to traditional vertex pipeline which is suboptimal for massive geometry counts and procedural content.

**Suggestion:** Add a `GeometryProcessor` abstraction in `nt_physical` that wraps mesh shader + amplification shader dispatch. Expose it to `nt_world` for geometry-heavy perception tasks. This aligns with the Dark Forest principle — if geometry processing exists, it must connect to consumers.

---

### D483-4: No Neural Denoising Layer in Perception Pipeline
**Severity:** HIGH | **Domain:** NT-WORLD (L2) + NT-MIND (L5)

The 2026 rendering consensus (Sources 5, 6, 7) establishes that neural denoisers are no longer optional — they are integral to real-time rendering pipelines. NeoTrix's `SensoryIntegrationHub` processes raw sensor data without a denoising step, and `nt_mind` has no module for learning-based noise reduction.

**Impact:** Visual perception from noisy real-world sensors (cameras, depth sensors) produces degraded input to the consciousness pipeline. Without denoising, downstream VSA embedding and HyperCube mapping operate on corrupted data.

**Suggestion:** Create `nt_world_denoise` module implementing a lightweight neural denoiser (similar to NVIDIA's approach where denoisers predict clean images from sparse/noisy samples). Wire it as a pre-processing stage in the PerceptionBridge before `awareness_score()` evaluation.

---

### D483-5: No Gaussian Splatting Rasterization Path
**Severity:** MEDIUM | **Domain:** NT-WORLD (L2) + NT-PHYSICAL (L3)

3D Gaussian Splatting (3DGS) has become a dominant scene representation in 2026, with both rasterization-based (Source 9: 5-10× speedup) and path-traced (Source 14) approaches. NeoTrix's design has no Gaussian splatting rasterization path — it relies on traditional mesh-based rendering.

**Impact:** NeoTrix cannot efficiently render or perceive scenes represented as Gaussian splatting, which is increasingly the standard for neural scene capture, NeRF, and novel view synthesis.

**Suggestion:** Add a `GaussianRasterizer` component to `nt_world` that wraps the geometry-shader-based sorted rasterization approach (Source 9). Expose it through the CapabilityBridge so that perception and rendering paths can select between mesh and Gaussian primitives.

---

### D483-6: No Differentiable Light Transport for Self-Test Verification
**Severity:** MEDIUM | **Domain:** NT-MIND (SEAL) + NT-CORE

The SIGGRAPH Asia 2025 work (Source 12) demonstrates that differentiable light transport enables efficient gradient computation for inverse rendering and geometry reconstruction. NeoTrix's SelfTest (T1-T3) system has no mechanism for **differentiable verification** of visual/rendering outputs — it can only check existence and registration, not gradient-based convergence of rendering quality.

**Impact:** SEAL pipeline cannot verify that visual output converges to physically correct results. Self-healing (NT-REPAIR) has no gradient signal to guide rendering repairs.

**Suggestion:** Add a `DifferentiableRenderer` capability to NT-REPAIR that uses the adapted radiosity Gaussian surfel framework for gradient-based quality verification. This would enable T3-level production wiring for rendering quality checks — the detection function (`evaluate()`) would compute gradient error against reference lighting.

---

### D483-7: No WebGPU / Browser Rendering Integration
**Severity:** LOW | **Domain:** NT-IO (L1)

WebGPU is now bringing low-level GPU access to browsers (Source 5), enabling high-fidelity rendering in web apps without plugins. NeoTrix's `nt_io` domain defines LLM providers, CLI, web server, ACP, and LSP, but has no WebGPU integration path.

**Impact:** NeoTrix cannot deliver rendering capabilities through web interfaces or cloud streaming clients. The design is locked to native GPU APIs.

**Suggestion:** Add a `WebGPURenderer` adapter to `nt_io` that exposes rendering capabilities through the browser. This enables cloud rendering streaming and web-based visualization without requiring native GPU access on the client.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 15 |
| Defects found | 7 |
| HIGH severity | 3 |
| MEDIUM severity | 3 |
| LOW severity | 1 |
| Domains affected | NT-WORLD, NT-CORE, NT-PHYSICAL, NT-MIND, NT-REPAIR, NT-IO, GWT |

## Key Takeaway

The 2026 rendering landscape has converged on three pillars: **(1)** hardware-accelerated path tracing with neural reconstruction, **(2)** variable-rate adaptive sampling tied to perceptual saliency, and **(3)** Gaussian splatting as a first-class scene representation. NeoTrix's consciousness architecture lacks integration points for all three. The most critical gap is the absence of a radiance cache in the perception pipeline (D483-1) and the missing neural denoising layer (D483-4), which together would address the core performance bottleneck for any visual reasoning task.
