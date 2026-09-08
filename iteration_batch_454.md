# Iteration Batch 454 — Research Loop: CAD / Geometric Modeling / Design Automation

**Date:** 2026-09-06
**Sources:** 24 sources across 3 domains

---

## 1. CAD Software — 2026 State of the Art

### Sources
1. **Steininger et al., "AI-enhanced CAD: predictive modelling of operations"** (Proceedings of the Design Society, Vol 6, Aug 2026) — Graph-based CAD assistant using 4-layer GAT to predict next modelling operation. Top-5 accuracy 94% on real CATIA V5 models. Structural + operational graph encoding.
2. **TexoCAD Blog, "AI CAD trends in 2026"** (Mar 2026) — Scorecard: text-to-CAD improved but limited to simple parts. B-Rep generation got better. Parametric AI generation still research-stage. Real impact = AI-assisted search/documentation, not geometry gen.
3. **Mesh Mayhem, "AI-Generated Parametric Models: 2026 Guide"** (Mar 2026) — VibeCAD (NL→OpenSCAD), Backflip AI (scan→parametric), Orbit Forge 3D (AI procedural). 4 workflows: NL→code, template-driven, mesh→parametric, hybrid.
4. **Mandloi, "CAD-Genesis"** (IJCA Vol 187, May 2026) — Open-source add-in converting NL to parametric models in SolidWorks/Fusion 360. Handles mates/constraints, mass properties, feature trees.
5. **PTC, "AI, CAD, and the Next Era of Engineering"** (May 2026) — AI transforming CAD from tool to intelligent partner. Simulation-coupled design.
6. **CoLab Software, "AI CAD in 2026: Generative CAD, Review, and ROI"** (Aug 2026) — Topology optimization and constraint-based generative design are most mature. Design review AI delivering real ROI.

### Key Defects in NeoTrix Design

**DEFECT-CAD-01: No B-Rep kernel or CSG engine.** NeoTrix has no native solid modeling representation. Current 3D pipeline (VSA HyperCube for symbolic knowledge) cannot represent watertight solids, compute mass properties, or perform Boolean operations. The entire CAD tooling gap means NeoTrix cannot serve as a design tool — only a meta-reasoning layer.

**DEFECT-CAD-02: Missing parametric feature graph encoding.** Steininger 2026 shows GAT on parametric feature DAGs achieves 94% top-5 prediction accuracy. NeoTrix has no concept of "feature tree" or "design history" — its graph structures (HyperCube edges, KB relations) are flat entity-relation triples, not ordered feature dependencies with constraints.

**DEFECT-CAD-03: No NL→CAD pipeline.** VibeCAD and CAD-Genesis demonstrate NL→OpenSCAD or NL→SolidWorks parametric generation. NeoTrix's consciousness_task accepts NL instructions but routes to existing capability nodes — there is no dedicated CAD generation domain or tool registration for geometry creation.

---

## 2. Geometric Modeling — 2026 State of the Art

### Sources
7. **MechCodex, "Solid Modeling Fundamentals: B-rep, CSG, and Parametric Features"** (Jul 2026) — Comprehensive reference: B-rep = topology + geometry separation, Euler-Poincaré validity, CSG = Boolean tree of primitives, NURBS = universal surface representation.
8. **EmergentMind, "Boundary Representations (B-Reps)"** (updated Mar 2026) — NeuroNURBS achieves 80-96% memory reduction via parametric encoding. BRepGPT, DTGBrepGen, HoLa = generative B-Rep models. FoV-Net achieves rotation invariance.
9. **EmergentMind, "Constructive Solid Geometry (CSG)"** (updated Jan 2026) — D²CSG, CAPRI-Net for neural CSG parsing. CSG favored for inherent watertightness. Bidirectional programming in CSG-based CAD (2024).
10. **Wang et al., "Construction of complex NURBS volume parametric models with G¹ continuity"** (Visual Computing, Apr 2026) — Volumetric NURBS models with G¹ continuity for complex topology. Trivariate spline construction.
11. **WorldMetrics, "Best NURBS Software 2026"** (Jun 2026) — BRL-CAD combines CSG + NURBS. Multi-resolution modeling emerging.
12. **Better STEP dataset** (Izadyar et al., 2025) — Open B-Rep HDF5 dataset for ML training on analytic surfaces.

### Key Defects in NeoTrix Design

**DEFECT-GEO-01: No topology-geometry separation.** NeoTrix's KB stores embeddings (dense vectors) and graph edges (symbolic relations) but has no concept of the B-rep paradigm: topology (adjacency, adjacency graph) vs geometry (coordinates, parametric equations). This means it cannot represent or reason about solid model validity (Euler-Poincaré, watertightness, orientability).

**DEFECT-GEO-02: Missing NURBS/spline representation.** NeoTrix has no parametric surface representation. VSA HyperCube maps concepts to vectors — it cannot represent a NURBS surface S(u,v) with control points, weights, knot vectors. Any CAD integration would require an entirely new geometry kernel.

**DEFECT-GEO-03: No neural CAD representation learning.** NeuroNURBS (2024) and FoV-Net (2026) demonstrate compact parametric encoding of B-Reps with 80-96% memory savings. NeoTrix has no equivalent "geometry embedding" module — its embeddings are semantic (word/concept level), not geometric (surface/point level).

---

## 3. Design Automation — 2026 State of the Art

### Sources
13. **Kamran, "AI Generative Design & Topology Optimization: 2026 Guide"** (Jun 2026) — Generative design produces 40-50% lighter parts. Topology optimization = remove material; generative = explore many candidates. Software: nTopology, Altair Inspire, Fusion 360.
14. **Leo AI, "Best Topology Optimization Tools (2026)"** (May 2026) — AI as intelligence layer for setting up optimization problems. Training on 1M+ pages of engineering standards.
15. **Leo AI, "Best AI for CAD Generation in 2026"** (May 2026) — Text-to-CAD = "renderings, not engineering designs." Generative/topo-opt = real winners. Parametric automation quietly saving weeks. Search-first-then-optimize workflow.
16. **PatSnap, "Generative Design & Topology Optimization 2026"** (Jun 2026) — 55+ patents analyzed (2000-2026). 15+ active Autodesk patents. 5 dominant AI-driven technical approaches.
17. **PatSnap, "Generative AI Topology Optimization"** (Apr 2026) — Heaviest patent concentration post-2020. Pipeline: design space → AI-optimized topology → manufacturable output.
18. **CoLab, "How Generative Design Works: 2026 Guide"** (Mar 2026) — Practical comparison of optimization-based tools vs text-to-CAD.

### Key Defects in NeoTrix Design

**DEFECT-AUT-01: No topology optimization / generative design capability.** NeoTrix's SEAL pipeline (explore→distill→absorb) operates on code and knowledge artifacts, not geometric solids. There is no module for FEA-driven material removal, load-constraint specification, or manufacturing constraint encoding. Topology optimization requires a geometry kernel + solver — neither exists.

**DEFECT-AUT-02: Missing manufacturing constraint encoding.** Generative design tools encode DFM rules (minimum wall thickness, draft angles, tool access, print orientation). NeoTrix has no "manufacturing domain" in its KB schema. Its node types (skill, concept, experience) cannot represent physical manufacturing constraints.

**DEFECT-AUT-03: No design-space ↔ solver feedback loop.** Modern pipelines couple AI-generated geometry with FEA simulation, then iterate. NeoTrix's SEAL loop distills experience from code edits — it has no concept of "run simulation → evaluate stress → modify geometry → re-simulate." The physics-based feedback loop is absent.

---

## Summary: 9 Defects Found

| ID | Domain | Defect | Severity |
|----|--------|--------|----------|
| DEFECT-CAD-01 | CAD | No B-Rep/CSG kernel | Critical |
| DEFECT-CAD-02 | CAD | No parametric feature graph | High |
| DEFECT-CAD-03 | CAD | No NL→CAD pipeline | Medium |
| DEFECT-GEO-01 | Geometric | No topology-geometry separation | Critical |
| DEFECT-GEO-02 | Geometric | No NURBS/spline representation | High |
| DEFECT-GEO-03 | Geometric | No neural CAD representation learning | Medium |
| DEFECT-AUT-01 | Automation | No topology optimization / generative design | High |
| DEFECT-AUT-02 | Automation | No manufacturing constraint encoding | Medium |
| DEFECT-AUT-03 | Automation | No design-space ↔ solver feedback loop | High |

---

## Suggestions

1. **Short-term (P1):** Register an `nt_world_cad` domain module with B-Rep and CSG primitives using an existing open-source kernel (OpenCASCADE via `opencascade-sys` crate). This gives watertight solid modeling, Boolean ops, and STEP/IGES I/O immediately.

2. **Medium-term (P2):** Build a `ParametricFeatureGraph` type mirroring the DAG structure from Steininger 2026 — nodes = features (extrude, fillet, boolean), edges = dependency order + constraint relations. Feed into GWT attention routing for design-aware reasoning.

3. **Medium-term (P2):** Add a `ManufacturingConstraint` node type to the KB with attributes: min_wall_thickness, draft_angle, print_orientation, material_class. This enables DFM-aware generative design within the SEAL pipeline.

4. **Long-term (P3):** Explore NeuroNURBS-style parametric encoding for compact B-Rep storage in the KB — encode surfaces as control points + weights + knots rather than UV-grids, achieving 80-96% storage reduction for geometric data.

5. **Long-term (P3):** Implement a physics feedback loop: GeometryGenerator → MeshGenerator → FEA Solver (e.g., `feanite` or `nalgebra`-based) → StressEvaluator → GeometryRefiner, orchestrated by SEAL pipeline phases.
