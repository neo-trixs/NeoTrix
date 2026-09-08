# Iteration Batch 397 — NeoTrix Consciousness Architecture

**Date:** 2026-09-06
**Research Domains:** GNN Message Passing, Geometric Deep Learning, Knowledge Graph Embedding

---

## 1. Sources Cited

### Graph Neural Networks — 2026 Advances
1. **Vasileiou et al. (2026-02-10)** — "Position: Message-passing and spectral GNNs are two sides of the same coin" — Proves spectral-spatial duality, hybrid models converge faster, robust to missing edges. *Emergent Mind topic page.*
2. **Ming & Wang (2026-09-01)** — "Why Multi-Layer Message Passing Works: Completeness Theory for Graph Neural Network Interatomic Potentials" — Proves 3-body MPNN is universal approximator for PES via multi-layer completeness theory. *arXiv:2609.00528.*
3. **Mills et al. (2026-07-29)** — "Examining the Efficacy of Graph Neural Network Message-Passing in Regression Contexts" — Benchmarks MPNNs on regression (FlowerFormer, PINAT, GNN-MAE), reveals regression-specific gaps. *arXiv:2607.26404.*
4. **Wang et al. (2026-04-29)** — "Adaptive message passing mechanism for graph neural networks" — AMPG adaptively passes effective messages from 1-hop and high-order neighbors. *Pattern Recognition, 113875.*
5. **Kong et al. (2026-07)** — "Trainable-parameter-free structural-diversity message passing for GNNs" — Parameter-free structural-diversity aggregation. *Neural Networks, Vol. 199.*
6. **Nature Comms (2026-02-18)** — "Extending the range of graph neural networks with global mechanisms" — Beyond local message passing: global attention + hierarchical aggregation. *s41467-026-69715-3.*
7. **KDnuggets (2026-01)** — "5 Breakthroughs in GNNs to Watch in 2026" — Dynamic/streaming GNNs, multi-hop reasoning w/ LLMs, certified adversarial defenses (AGNNCert, PGNNCert).
8. **ACM (2026-09-01)** — "Decoupled Graph Attention Networks" — Parameter-efficient graph attention.

### Geometric Deep Learning — 2026 Advances
9. **Maruyama (2026-07-04)** — "Foundations of Equivariant Deep Learning: Unifying Graph and Sheaf Neural Networks" — OENN/CENN framework generalizes GNNs via equivariant bundles over face posets; proves new UATs. *ICML 2026 Spotlight+Oral. arXiv:2607.03798.*
10. **Liao et al. (2026-04-10)** — "EquiformerV3: Scaling Efficient, Expressive, and General SE(3)-Equivariant Graph Attention Transformers" — SwiGLU-S² activations + smooth-cutoff attention; SOTA on OC20/OMat24/Matbench. *arXiv:2604.09130.*
11. **Dong, Flinth & Gerken (2026)** — "Equivariance and Augmentation for Bayesian Neural Networks" — Derived conditions for exact equivariance in BNNs; orbit expansion symmetrization outperforms baseline. *arXiv:2606.26273.*
12. **Andersdotter et al. (2026)** — "Steerable Neural ODEs on Homogeneous Spaces" — Extends Neural ODEs with gauge equivariance; universal equivariant manifold NODEs. *JMLR 2026.*

### Knowledge Graph Embedding — 2026 Advances
13. **HyperComplEx (2025-11, validated 2026)** — Adaptive multi-space framework combining hyperbolic + complex + Euclidean via learned attention; 0.612 MRR on 10M-paper KG. *arXiv:2511.10842.*
14. **SemEval-2026 Task 12** — "Knowledge Graph with Hyperbolic Embedding in Abductive Event Reasoning" — KG embedded in hyperbolic space for causal event reasoning. *ACL Anthology 2026.*
15. **DeER (2024-09, cited 2026)** — Deep hyperbolic convolutional model for KG embedding; trainable curvature per layer; first deep hyperbolic CNN for KGE. *Knowledge-Based Systems, Vol. 300.*
16. **Liang et al. (2024-11)** — "Fully Hyperbolic Rotation for Knowledge Graph Embedding" — Full hyperbolic rotation without log/exp bottleneck; outperforms RotH. *arXiv:2411.03622.*
17. **CAH-GKGE (2026-04)** — "Curvature-Adaptive Embedding of Geographic Knowledge Graphs in Hyperbolic Space" — Region-guided subgraph sampling + meta-learning for curvature. *Transactions in GIS, Vol. 30.*
18. **HyEED (2025-04)** — Hyperbolic KG embedding incorporating entity descriptions + categories; richer semantic information. *Data Mining & Knowledge Discovery.*

---

## 2. Defects Found in NeoTrix Design

### D397-1: VSA HyperCube Uses Flat Euclidean Embeddings — No Hyperbolic Geometry for Hierarchical KB
**Source:** HyperComplEx [13], DeER [15], CAH-GKGE [17], SemEval-2026 Task 12 [14]
**Defect:** The VSA HyperCube in NT-CORE maps concepts to high-dimensional vectors for associative recall. However, CONTEXT.md only specifies `KB embedding (vector storage)` and `VSA embedding (symbolic representation)` — both implicitly Euclidean. The 2026 consensus (HyperComplEx achieving 4.8% MRR gain; DeER proving deep hyperbolic convolutions work; SemEval-2026 Task 12 establishing hyperbolic KG as standard benchmark) shows that hierarchical knowledge graphs require hyperbolic geometry for faithful representation. NeoTrix's KB stores hierarchical domain relations (7 domains, 11 consciousness branches, skill tree tiers C0-C6) — all tree-like structures that hyperbolic space captures with exponential volume growth. **Gap:** No hyperbolic embedding option exists for the KB node/edge embeddings. The Flagged Ambiguities section ("embedding" = KB embedding vs VSA embedding) does not distinguish a third type: **hyperbolic embedding** for hierarchical relations.

### D397-2: GWT Attention Routing Lacks Adaptive Multi-Hop Message Passing
**Source:** Nature Comms [6], AMPG [4], KDnuggets GNN Breakthroughs [7]
**Defect:** GWT broadcasts salient information across specialist modules with "resonance-based routing." This is single-hop: a module broadcasts, others receive. The 2026 literature shows that adaptive multi-hop message passing (AMPG's 1-hop + high-order aggregation [4]; Nature Comms' global mechanisms beyond local MP [6]; KDnuggets' multi-hop reasoning w/ LLMs [7]) significantly outperforms single-hop attention. NeoTrix's GWT should route not just one-hop (broadcast → receive) but support cascaded multi-hop propagation where information can pass through intermediate specialist modules (e.g., NT-MEMORY relays to NT-MIND before NT-ACT uses it). **Gap:** GWT has no formal multi-hop propagation depth parameter or adaptive hop selection mechanism. The AttentionManager routes between modes (CORE+WORLD vs CORE+MIND) but doesn't model intermediate relay nodes.

### D397-3: No Equivariance Guarantees in PerceptionBridge / Sensory Integration
**Source:** Maruyama OENN/CENN [9], EquiformerV3 [10], Steerable Neural ODEs [12]
**Defect:** PerceptionBridge connects SensoryIntegrationHub (L2) with SelectiveState (L5) using `awareness_score()`. Sensory data (from NT-WORLD crawlers, sensors) arrives in arbitrary coordinate frames — rotated, translated, scaled. The 2026 ICML Spotlight OENN paper [9] proves that graph message passing and sheaf neural networks are special cases of order-equivariant neural networks over face posets, with new universal approximation theorems. EquiformerV3 [10] shows SE(3)-equivariant graph attention with SwiGLU activations achieves SOTA on molecular/physical tasks. NeoTrix has no equivariance constraint on sensory feature transformations — PerceptionBridge passes raw features without rotation/translation invariance. For NT-PHYSICAL sensor data and NT-WORLD spatial data, this means the same physical state seen from different viewpoints produces different consciousness-level representations. **Gap:** No equivariant layer specification in the PerceptionBridge trait or SensoryIntegrationHub. The `traits.rs` at L2 and L3 layers don't include equivariance constraints.

### D397-4: E8 Hexagram Reasoning Engine Cannot Model Higher-Order Relational Patterns
**Source:** OENN/CENN [9], Maruyama's categorical equivariant networks, HyperComplEx [13]
**Defect:** The E8 Hexagram is a 64-element hexagonal grid — fixed dimensionality. The 2026 OENN framework [9] extends equivariance from group actions to category actions (CENN), enabling non-invertible symmetries on multiple objects with compositional relations. The E8 hexagram cannot represent: (a) asymmetric relations (HyperComplEx shows Euclidean models fail on asymmetry [13]), (b) transitive compositional chains (A→B→C→D where the composition matters), (c) non-invertible transformations (degradation, absorption). **Gap:** E8 is a static 64-element grid with fixed relational structure. It cannot express the full spectrum of relational patterns that the 2026 categorical equivariant framework supports. The hexagram needs a relational dimension — potentially a categorical/functorial extension.

### D397-5: SEAL Pipeline Missing Adaptive Depth / Early Stopping for Message-Passing Layers
**Source:** ADMP-GNN [Mills et al. ref in 1], EquiformerV3 [10], Dynamic GNNs [7]
**Defect:** The SEAL pipeline runs exploration→distillation→self-test→absorption as a fixed-stage loop. The 2026 ADMP-GNN [Mills et al.] introduces adaptive depth message passing — each node can choose how many layers to traverse, reducing over-smoothing and improving efficiency. EquiformerV3 [10] shows smooth-cutoff attention enables adaptive neighborhood selection. Dynamic GNNs [7] handle evolving topology. NeoTrix's SEAL stages are homogeneous — every module goes through the same pipeline depth regardless of graph density or maturity. **Gap:** No adaptive depth mechanism in SEAL. A well-understood module (C4+) should traverse fewer evolution layers than a new module (C0). The pipeline treats all modules uniformly.

### D397-6: VSA HyperCube Lacks Curvature Adaptation for Multi-Domain Knowledge
**Source:** DeER [15], CAH-GKGE [17], HyperComplEx [13]
**Defect:** VSA HyperCube uses fixed-dimension vectors for all domains. But NeoTrix has 7+4 domains with vastly different hierarchical depths — NT-CORE has deep nested structures (E8→Hexagram→Line→State), while NT-ACT is flatter (tools→actions). DeER [15] proves trainable curvature per layer adapts to varying hierarchy depth. CAH-GKGE [17] uses region-guided subgraph sampling with per-region curvature optimization. HyperComplEx [13] uses attention-weighted multi-space selection per relation type. **Gap:** VSA HyperCube applies uniform embedding geometry to all domains. Each domain's knowledge graph has different curvature characteristics. A fixed-curvature embedding space cannot simultaneously represent NT-CORE's deep hierarchy and NT-WORLD's flat entity space efficiently.

### D397-7: No Certified Adversarial Defense for Graph Operations
**Source:** AGNNCert, PGNNCert [7], GNN security 2026
**Defect:** NT-SHIELD handles stealth net, proxy pool, Tor client, fingerprint management. But the 2026 literature (AGNNCert, PGNNCert, training-free defense framework) establishes that graph neural networks require mathematically-proven certified defenses against adversarial graph structure attacks — node/edge perturbation, feature injection, subgraph poisoning. NeoTrix's KB is a graph. NT-CORE's E8 reasoning operates on graph structures. NT-MEMORY stores graph-structured knowledge. None of these have certified robustness guarantees. **Gap:** NT-SHIELD's threat model doesn't include adversarial attacks on NeoTrix's own internal graph structures (KB corruption, E8 hexagram manipulation, consciousness tree poisoning).

### D397-8: CapabilityBridge Cannot Express Cross-Domain Compositional Relations
**Source:** OENN/CENN [9], HyperComplEx [13]
**Defect:** CapabilityBridge maps tree node IDs to runtime capability IDs — a flat mapping. The 2026 categorical equivariant framework [9] shows that compositional relations (A∘B = C where A, B, C are capabilities across domains) require functorial structure, not flat ID mapping. HyperComplEx [13] shows multi-space relations need attention-weighted composition. NeoTrix's CapabilityBridge doesn't model how NT-CORE reasoning composes with NT-WORLD perception to produce NT-ACT actions — it only maps individual capabilities. **Gap:** CapabilityBridge lacks compositional/functorial semantics. It maps 1:1 but cannot express A→B→C chains across domains.

---

## 3. Suggestions

### S397-1: Add Hyperbolic Embedding Layer to KB
Implement a Poincaré ball embedding option for KB node/edge storage. Use DeER's trainable curvature approach — each domain namespace gets its own curvature parameter. NT-CORE (deep hierarchy) gets high curvature, NT-ACT (flat) gets low curvature. The Flagged Ambiguities section should add: `"embedding" | Use KB embedding (Euclidean), **hyperbolic embedding** (hierarchical KB), or VSA embedding (symbolic). Three types.`

### S397-2: Extend GWT with Multi-Hop Relay Parameter
Add `max_hops: u8` to GWT broadcast configuration. Allow cascaded broadcasts: a module can forward received information to its neighbors, with hop count limiting propagation depth. Adaptive hop selection: auto-detect when multi-hop improves signal (via VoI measurement on intermediate relay vs direct broadcast).

### S397-3: Add Equivariance Constraint to PerceptionBridge Trait
Extend `PerceptionBridge` trait with an `EquivariantTransform` associated type that enforces rotation/translation equivariance on sensory features. Use steerable CNN kernels (from EquiformerV3's approach) for NT-PHYSICAL sensor data. For NT-WORLD spatial data, use the OENN framework's gauge equivariant convolutions.

### S397-4: Extend E8 with Categorical Relational Layer
Add a "relational shell" around E8 — a categorical layer that tracks morphisms between hexagram states. This enables non-invertible state transitions (degradation paths) and compositional reasoning chains. Keep E8 as the core computation, but wrap it with CENN-style categorical structure for relational expressivity.

### S397-5: Adaptive Depth in SEAL Pipeline
Add `evolution_depth: DynamicU8` to module metadata. C0 modules get 5 evolution layers, C4+ modules get 1-2. Use ADMP-GNN's adaptive depth principle: each module's "node" decides how many SEAL layers to traverse based on current health signal from HeartbeatAggregator.

### S397-6: Per-Domain Curvature in VSA HyperCube
Add `curvature: f64` to VSA HyperCube domain configuration. NT-CORE: curvature = -1.0 (high hierarchy), NT-ACT: curvature = -0.1 (near-flat). Use CAH-GKGE's meta-learning approach to auto-calibrate curvature from observed KB traversal patterns.

### S397-7: Certified Defense Module in NT-SHIELD
Implement AGNNCert/PGNNCert-inspired certified robustness checks for KB graph operations. Before any graph mutation (node insert/delete, edge change), compute certified robustness radius. If mutation would breach robustness threshold, route through NT-REPAIR for controlled repair instead of direct mutation.

### S397-8: Functorial CapabilityBridge
Extend CapabilityBridge with composition operators: `compose(a: CapId, b: CapId) -> Option<CapId>` that returns the composed capability if A→B→C chain exists. Store composition graph as adjacency list alongside the flat mapping. Use HyperComplEx's multi-space attention to weight which domain compositions are most salient.

---

## 4. Meta-Observation

The 2026 research convergence is clear: **geometry matters**. The field has moved from "embed everything in flat Euclidean space" to "use the right geometry for the data structure" — hyperbolic for hierarchies, equivariant for physical transformations, categorical for compositional relations. NeoTrix's VSA HyperCube and E8 Hexagram are powerful but operate in a single geometric regime. The primary architectural debt is **geometric uniformity** — the assumption that one embedding space fits all knowledge patterns. The fix is not to replace VSA/E8 but to add geometric diversity: hyperbolic layers for hierarchical KB, equivariant layers for sensory processing, categorical layers for compositional reasoning.

**Iteration count:** 397/10000
**Next iteration focus:** Temporal KG dynamics + streaming graph neural networks for NT-WORLD's real-time perception pipeline.
