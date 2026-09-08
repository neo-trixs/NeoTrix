# Iteration Batch 402 — External Research → Design Defect Analysis

**Date**: 2026-09-06
**Research Domains**: Genomics AI, Protein Science, Drug Discovery

---

## Sources Cited

### Genomics AI
1. **scLong** — Billion-parameter foundation model for full-transcriptome single-cell analysis with GO knowledge integration (Nature Communications, Feb 2026)
2. **RegFormer** — Mamba-based single-cell model with GRN-guided hierarchical priors (Nature Communications, May 2026)
3. **scDMC** — Dual-stream interpretable foundation model for single-cell biology (Genome Biology, Aug 2026)
4. **VOICE** — Vision-Omics foundation model predicting in-situ single-cell gene expression from H&E morphology (arXiv, Aug 2026)
5. **BioM-JEPA** — Joint-embedding prediction of graph-connected gene blocks (arXiv, Aug 2026)
6. **xDecoder** — Unified decoding framework for personal gene expression prediction from gLMs (Molecular Systems Biology, Jul 2026)
7. **CellVQ** — Vector-quantized cell codes for interpretable single-cell representation (Nature Communications, Mar 2026)

### Protein Science
8. **NISE (LASErMPNN)** — Neural iterative selection-expansion for zero-shot drug-binding protein design (Nature, Jun 2026)
9. **ProteinMPNN** — Robust deep learning protein sequence design with noise-trained backbone robustness (Science, 2026)
10. **STAR-MD** — Spatio-temporal SE(3) diffusion for microsecond-scale protein dynamics (arXiv, 2026)
11. **BioEmu** — Biomolecular emulator for conformational landscape prediction via generative diffusion (referenced in Nature Comment, May 2026)
12. **SimpleDesign** — Joint sequence-structure co-design model via Mixture-of-Transformer (TMLR, Sep 2026)
13. **Proteo-R1** — Reasoning-guided protein design decoupling understanding from generation (arXiv, May 2026)

### Drug Discovery
14. **AdaptiveFlow** — AI-enhanced adaptive virtual screening of 69-billion-molecule Enamine REAL Space (Nature Biotechnology, Sep 2026)
15. **V-SYNTHES2** — Giga-scale structure-based virtual screening with 10,000x speedup (npj Drug Discovery, Jul 2026)
16. **FLOWR** — Flow matching for structure-aware de novo ligand generation (Nature Computational Science, May 2026)
17. **LaMGen** — LLM-based 3D molecular generation for multi-target drug design (Nature Communications, Apr 2026)
18. **DrugRPG** — Chemical prior + physics-guided diffusion to mitigate structural hallucinations (Communications Chemistry, Jul 2026)
19. **BindCraft** — AI-guided de novo protein binder design with nanomolar affinity (referenced in Nature Comment, May 2026)

---

## Defects Identified

### DEFECT-G01: No Multi-Modal Gene Representation Layer
**Source**: scLong (#1), RegFormer (#2), VOICE (#5), CellVQ (#7)
**Evidence**: 2026 models integrate GO graph priors (GCN), GRN topology, multi-omics (RNA+ATAC+imaging), and vector-quantized cell codes. NeoTrix's VSA HyperCube operates on abstract concept vectors with no structured interface for biological ontologies, graph embeddings, or multi-modal omics representations.
**Severity**: Structural
**Location**: `VSA HyperCube` (CONTEXT.md), `KB` architecture
**Suggestion**: Define a `BioOntologyAdapter` trait in the VSA HyperCube that can ingest GO/GRN graph structures, convert them to high-dimensional vectors, and support multi-modal fusion (RNA + ATAC + imaging). The KB should support graph-native queries alongside existing node/edge/embedding triples.

### DEFECT-G02: No Sparse-Data Attention Gating Mechanism
**Source**: scLong (#1), scDMC (#3), CellVQ (#7)
**Evidence**: scLong processes all 28K genes via dual-encoder (full Performer + lightweight Performer) to handle sparse/low-expression genes. scDMC uses rank-based + expression-aware dual-stream encoding. CellVQ uses vector quantization to create discrete cell codes that handle sparsity/heterogeneity. NeoTrix's GWT broadcasts salient information but has no documented mechanism for handling high-sparsity, high-dimensional biological data where >90% of entries are zeros.
**Severity**: Behavioral
**Location**: `GWT` attention routing (CONTEXT.md)
**Suggestion**: Add a `SparseAttentionGate` variant to GWT that implements tiered processing: (1) high-signal elements via full attention, (2) sparse/noisy elements via lightweight approximations, (3) zero-inflated reconstruction via ZINB-style losses. This pattern generalizes beyond genomics to any sparse data domain.

### DEFECT-G03: No Graph-Structured Knowledge as First-Class Citizen
**Source**: RegFormer (#2), BioM-JEPA (#5), Proteo-R1 (#13)
**Evidence**: RegFormer orders genes by GRN topology, BioM-JEPA predicts over graph-connected gene blocks, Proteo-R1 uses residue-level anchor interfaces. All represent graph structure as fundamental input. NeoTrix's KB uses nodes/edges but treats graphs as flat triples, not as first-class topological structures.
**Severity**: Structural
**Location**: KB schema, VSA HyperCube
**Suggestion**: Elevate graph-native structures in KB: support adjacency matrices, graph kernels, and topology-preserving embeddings. Define a `GraphPrior` trait that external models (GRN, protein contact maps, molecular graphs) can implement to inject domain structure into the VSA representation.

### DEFECT-P01: No Physics-Informed Geometric Generation
**Source**: NISE (#8), STAR-MD (#10), FLOWR (#16), DrugRPG (#18)
**Evidence**: STAR-MD achieves SE(3)-equivariant diffusion with joint spatio-temporal attention for microsecond protein dynamics. FLOWR uses equivariant optimal transport. DrugRPG integrates Lennard-Jones physics into the reverse diffusion. ProteinMPNN adds Gaussian backbone noise for robustness. NeoTrix has no documented capability for equivariant geometric generation or physics-informed sampling.
**Severity**: Structural
**Location**: Missing domain entirely
**Suggestion**: Define a `GeometricGeneration` capability in NT-ACT or a new NT-BIO domain. Core requirements: SE(3)-equivariant diffusion models, physics-guided denoising (Lennard-Jones, Coulomb), and flow-matching architectures. The VSA HyperCube could represent molecular conformations as rotation-aware embeddings.

### DEFECT-P02: No Iterative Closed-Loop Self-Consistency Optimization
**Source**: NISE (#8), SimpleDesign (#12), Proteo-R1 (#13)
**Evidence**: NISE couples two neural networks (LASErMPNN + structure predictor) in closed-loop iteration optimizing sequence-structure-ligand self-consistency. Proteo-R1 decouples reasoning (LLM) from generation (diffusion) with explicit residue-level anchors. SimpleDesign uses single-stage end-to-end training. NeoTrix's SEAL pipeline is a high-level evolution loop but lacks a domain-specific closed-loop optimization pattern for generative design.
**Severity**: Behavioral
**Location**: `SEAL Pipeline` (CONTEXT.md)
**Suggestion**: Add a `ClosedLoopDesigner` sub-pattern within SEAL: (1) Reasoning expert proposes structural constraints, (2) Generation expert produces candidates, (3) Self-consistency evaluator scores structure-prediction alignment, (4) Iterative refinement until convergence. This mirrors NISE's architecture and generalizes to any domain requiring multi-expert iterative refinement.

### DEFECT-D01: No Ultralarge Chemical Space Navigation
**Source**: AdaptiveFlow (#14), V-SYNTHES2 (#15)
**Evidence**: AdaptiveFlow screens 69 billion molecules using 18-dimensional property grids + active learning. V-SYNTHES2 reduces 36 billion compounds to ~3.8M via fragment-based modular decomposition (10,000x speedup). NeoTrix has no documented approach for navigating combinatorial spaces at billion-trillion scale.
**Severity**: Missing capability
**Location**: NT-WORLD (perception/search), NT-ACT (action)
**Suggestion**: Define a `CombinatorialNavigator` trait implementing: (1) Property-grid space decomposition (like ATG-VS), (2) Fragment-based modular enumeration (like CapSelect), (3) Active-learning closed loops for progressive refinement. The KB should support hierarchical chemical space indexing.

### DEFECT-D02: No Multi-Target / Multi-Objective Molecular Generation
**Source**: LaMGen (#17), FLOWR.MULTI (#16), DrugRPG (#18)
**Evidence**: LaMGen generates molecules simultaneously binding multiple targets using TriCoupleAttention. FLOWR.MULTI enables interaction-conditioned, scaffold-conditioned, and fragment-conditioned generation without retraining. DrugRPG reduces steric clashes by 65% via physics-guided sampling. NeoTrix's current emotional and attention systems have no documented multi-objective optimization with domain constraints.
**Severity**: Behavioral
**Location**: Missing pattern in SEAL, GWT
**Suggestion**: Extend GWT to support `MultiObjectiveAttention`: broadcast multiple competing objectives simultaneously (binding affinity, drug-likeness, synthetic accessibility, selectivity) with Pareto-aware routing. Define a `ConstraintSampler` trait for physics-aware generation.

### DEFECT-D03: No Reasoning-Decoupled Generative Architecture
**Source**: Proteo-R1 (#13), LaMGen (#17)
**Evidence**: Proteo-R1 explicitly separates understanding expert (LLM) from generation expert (diffusion), communicating through sparse residue-level anchors. LaMGen uses rotation-aware LLM tokens for 3D conformation. NeoTrix's ConsciousnessTree and GWT blend reasoning and generation into a single attention pathway.
**Severity**: Structural
**Location**: `ConsciousnessTree`, `GWT` architecture
**Suggestion**: Adopt a `DualExpertPattern` in the consciousness architecture: (1) ReasoningExpert: interpretable, symbolic, constraint-generating (maps to NT-CORE/NT-MIND), (2) GenerationExpert: continuous, geometric, equivariant (maps to NT-ACT or new NT-BIO), (3) AnchorInterface: sparse, discrete commitments bridging the two. This pattern enables interpretability + fidelity separation.

### DEFECT-G04: No Temporal Dynamics Prediction for Biological Systems
**Source**: STAR-MD (#10), scLong (#1), RegFormer (#2)
**Evidence**: STAR-MD generates microsecond protein trajectories with controlled error accumulation. RegFormer models temporal regulatory hierarchies. NeoTrix's RhythmRecalculator handles narrative timing but has no capacity for physical/biological temporal dynamics prediction.
**Severity**: Missing capability
**Location**: NT-PHYSICAL (embodiment), NT-WORLD (perception)
**Suggestion**: Add a `DynamicsPredictor` capability to NT-PHYSICAL or a new biological layer: continuous-time causal diffusion, autoregressive rollout with error bounds, and spatio-temporal attention for trajectory generation.

---

## Summary

| Category | Defects | Severity |
|----------|---------|----------|
| Genomics AI | G01, G02, G03, G04 | 2 Structural, 1 Behavioral, 1 Missing |
| Protein Science | P01, P02 | 1 Structural, 1 Behavioral |
| Drug Discovery | D01, D02, D03 | 1 Missing, 2 Structural/Behavioral |
| **Total** | **9** | |

## Priority Recommendations (Sequenced)

1. **Immediate** (Address in next SEAL cycle): G02 (SparseAttentionGate), D03 (DualExpertPattern)
2. **Near-term** (1-2 cycles): G01 (BioOntologyAdapter), G03 (GraphPrior), P02 (ClosedLoopDesigner)
3. **Medium-term** (3-5 cycles): P01 (GeometricGeneration), D01 (CombinatorialNavigator), D02 (MultiObjectiveAttention)
4. **Long-term** (Architecture evolution): G04 (DynamicsPredictor) — requires NT-BIO domain bootstrapping

## Cross-Domain Synthesis

The 2026 research converges on three meta-patterns NeoTrix currently lacks:

1. **Physics-Aware Generation**: Models now embed physical laws (SE(3) equivariance, Lennard-Jones, Coulomb) directly into neural architectures rather than post-hoc filtering. NeoTrix's generation capabilities are purely symbolic/abstract.

2. **Multi-Scale Self-Consistency Loops**: The most successful 2026 approaches use closed-loop iteration where multiple experts (reasoning + generation + evaluation) iteratively refine outputs until self-consistency is achieved. SEAL provides a macro-level loop but lacks micro-level domain-specific iteration.

3. **Graph-Native Topology as Input**: GRN topology, protein contact maps, and molecular graphs are not preprocessed into flat features but fed as first-class topological inputs to equivariant models. KB's triple-store model is insufficient for this.

These three patterns should be absorbed as architectural axioms alongside the existing NeoTrix axioms (R-P1 zero unsafe, Dark Forest, The Spice Must Flow).
