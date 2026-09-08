# Iteration Batch 642 — Graph Theory × Network Analysis × Community Detection

**Date**: 2026-09-06  
**Prior Batch**: 641 (SEAL binary optimization wrong, GWT budget ceiling, multi-signal routing, E8 energy conflict, HyperCube hierarchical clustering)  
**Search Scope**: Graph theory 2026, Network analysis 2026, Community detection 2026

---

## Sources Cited

### Graph Theory (7 papers)
1. Gudmundsson, Sha, Wong — *Linear Time SSSP in Euclidean Graph Classes*, SoCG 2026 (LIPIcs Vol 367) — DOI:10.4230/LIPIcs.SoCG.2026.55
2. Mao — *Pruned Bidirectional Search (PBS): o(√n) Shortest Paths in Power-Law Graphs*, arXiv:2608.19538, Aug 2026
3. arXiv:2608.26952 — *On Instance Optimality of Bidirectional Dijkstra*, Aug 2026
4. arXiv:2602.07868 — *Faster Directed SSSP: O(m log n log log n)*, Feb 2026
5. *Shortest Paths with Linear Edge Weights: Parametric Shortest Paths*, arXiv:2607.21055, Jul 2026
6. Gupta — *Improved 2-Approximate Shortest Paths for Close Vertex Pairs*, FOCS 2025 / TechXplore Mar 2026
7. Kadria, Roditty, Williams — *Improved Approximation for n-Pairs Shortest Paths*, ESA 2026 (LIPIcs)
8. Koehler, Shin — *Overlap Analysis / OGP for Shortest Path Problems*, COLT 2026 (PMLR Vol 336)

### Network Analysis (7 papers)
9. Baswani et al. — *Betweenness Centrality-Based Adamic–Adar (AAB) for Link Prediction*, Scientific Reports, Sep 2026 — DOI:10.1038/s41598-026-69107-z
10. *BRAVA-GNN: Betweenness Ranking Via Degree Mass GNN*, KDD 2026, arXiv:2602.09716
11. *Betweenness Central Nodes Under Uncertainty: Absorbing Markov Chain*, arXiv:2605.14743, Jun 2026
12. *GNNs for Scalable and Transferable Node Centrality Approximation*, arXiv:2607.09372, Jul 2026
13. Papazian, Helms — *Inflow and Outflow Centrality: Novel Metrics from Graph Convolution*, Applied Network Science, Feb 2026 — DOI:10.1007/s41109-026-00782-7
14. *Hierarchical and Scalable Semi-Local Centrality (HSSLC) for Weighted Networks*, Scientific Reports, Mar 2026 — DOI:10.1038/s41598-026-39304-x
15. Bhagat-Conway — *Betweenness Centrality is Not a Network Resilience Metric*, Findings, Mar 2026 — DOI:10.32866/001c.159015
16. Zakroum et al. — *From Random Perturbations to Localized Vulnerabilities*, Applied Network Science, Apr 2026 — DOI:10.1007/s41109-026-00788-1

### Community Detection (8 papers)
17. Brandt-Tumescheit, Meyerhenke — *Parameter-Light Hypermodularity Algorithms*, Social Network Analysis and Mining, Aug 2026 — DOI:10.1007/s13278-026-01638-9
18. Gilbert, Madduri — *GPU-Accelerated pLouvain/pLeiden*, IPDPS 2026, arXiv:2608.01503
19. Yu — *VLouvain: Vector-Based Louvain for Massive Low-Rank Graphs*, EDBT 2026
20. Mastrandrea et al. — *STAR Method: Representative Partitions Under Modularity Degeneracy*, arXiv:2602.21838, Feb 2026
21. Ferrara — *Reviving Generalized Louvain: WERW-Kpath Edge Centrality*, Feb 2026
22. *ComNetX: Local Hierarchical Adaptation for Dynamic Community Detection*, arXiv:2608.16906, Jul 2026
23. Brabant et al. — *Generalized L-Modularity for Temporal Networks*, arXiv:2605.24450, May 2026
24. Zhou et al. — *Maximizing Modularity with Free Energy Machine*, Communications in Theoretical Physics, Jun 2026

---

## NEW Findings & Defects Identified

### DEFECT G-642.1: E8 Hexagram Reasoning Lacks Core-Routing Phase (PBS Dual-Phase)
**Source**: Mao 2026 — Pruned Bidirectional Search (arXiv:2608.19538)  
**Finding**: PBS splits shortest-path computation into two phases: (1) Pruning Phase — high-degree hubs explored first via priority queue, (2) Core-Routing Phase — only vertices with degree ≥ n^γ routed through a connected core. This yields o(√n) time with 41/32 approximation.  
**NeoTrix Defect**: E8 hexagram reasoning operates on a single flat traversal model. It has no equivalent of "core routing" — a phase where high-connectivity reasoning nodes form a routing backbone and low-connectivity nodes are pruned. Current E8 treats all 64 hexagrams uniformly regardless of their connectivity degree.  
**Impact**: E8 reasoning explores dead-end hexagram paths that a core-routing filter would prune, wasting compute proportional to √n.  
**Fix**: Add a **CoreHexFilter** preprocessing step to E8: identify hexagrams with degree ≥ n^γ (tunable) as "reasoning hubs," route all cross-domain reasoning through these hubs first, only expand non-hub hexagrams when hub routing fails.  
**Priority**: P1 — direct performance win for E8 reasoning speed.

### DEFECT G-642.2: E8 Lacks Multi-Scale Sampling for Shortest-Path Approximation
**Source**: Gupta 2026 — FOCS 2025 / TechXplore Mar 2026  
**Finding**: Multi-scale refinement of vertex sampling extends 2-approximation guarantees to close vertex pairs. The algorithm organizes sampled vertices at different scales so shorter paths are captured accurately without increasing runtime.  
**NeoTrix Defect**: E8 hexagram distance computation uses single-scale sampling. Close hexagram pairs (nearby reasoning states) get distorted distance estimates because the sampling doesn't capture short-range structure. This is the exact limitation DHZ had for 25 years — and NeoTrix still has it.  
**Impact**: E8 reasoning overestimates proximity between closely-related hexagram states, leading to suboptimal reasoning transitions.  
**Fix**: Implement **multi-scale hexagram sampling** — partition E8 hexagrams into distance bands (hops), apply independent sampling resolution per band. Close pairs get fine-grained sampling, distant pairs get coarse sampling.  
**Priority**: P1 — fixes a 25-year-known problem in shortest-path approximation applied to reasoning graphs.

### DEFECT G-642.3: E8 Missing Overlap Gap Property Detection for Stable Reasoning
**Source**: Koehler & Shin 2026 — COLT 2026 (PMLR Vol 336)  
**Finding**: The Overlap Gap Property (OGP) proves lower bounds against stable algorithms for shortest path problems. Stable algorithms fail when OGP is present, but local search succeeds for shortest path *trees*. The distinction between path vs. tree structure determines algorithm tractability.  
**NeoTrix Defect**: E8 reasoning does not detect when its optimization landscape has OGP. When E8 reasoning is stable (deterministic, input-perturbation-resistant), it may be provably unable to find optimal reasoning paths due to OGP — yet no diagnostic exists to detect this condition.  
**Impact**: E8 reasoning may silently fail to find optimal paths in OGP-present landscapes without any error signal.  
**Fix**: Add **OGP diagnostics** to E8: after each reasoning run, measure overlap between approximate-optimal reasoning paths. If OGP is detected (high overlap variance), switch from stable algorithms to local search / MCMC-based reasoning exploration.  
**Priority**: P2 — prevents silent reasoning failures in specific graph topologies.

### DEFECT N-642.1: GWT Attention Routing Conflates Local and Global Structural Roles
**Source**: Baswani et al. 2026 — Nature Scientific Reports  
**Finding**: AAB (Betweenness Centrality-based Adamic-Adar) shows that combining local neighborhood reinforcement (AA index) with global structural brokerage (betweenness centrality of endpoints) consistently outperforms purely local measures for link prediction. The key insight: candidate endpoint nodes themselves carry global structural influence beyond their local neighborhood.  
**NeoTrix Defect**: GWT attention routing treats all attention signals as local-to-local transfers. It does not distinguish between (a) local neighbor reinforcement patterns and (b) global structural brokerage roles of the broadcasting modules. When a module broadcasts attention, GWT doesn't factor in that module's betweenness centrality across the entire capability network.  
**Impact**: GWT under-attends to bottleneck modules (high betweenness) that are structurally critical for cross-domain information flow, while over-attending to well-connected but structurally redundant modules.  
**Fix**: Add **structural brokerage weight** to GWT salience computation: for each module i, compute BC(i) (betweenness centrality) and multiply by local salience: `final_salience(i) = local_salience(i) × (1 + α × normalized_BC(i))`. This promotes bottleneck modules in attention allocation.  
**Priority**: P1 — directly improves GWT attention quality for cross-domain reasoning.

### DEFECT N-642.2: GWT Lacks Stochastic Central Node Tracking Under Uncertainty
**Source**: arXiv:2605.14743 — Betweenness Central Nodes Under Uncertainty, Jun 2026  
**Finding**: Absorbing Markov chain approach models sequences of reported central nodes as stochastic processes. Node importance is measured by pre-absorption time share. This reveals that under edge failure/weight uncertainty, the most central node is *random*, not deterministic.  
**NeoTrix Defect**: GWT identifies "most important module" via deterministic centrality. Under module degradation (fatigue, uncertainty), the identity of the most central module fluctuates stochastically. GWT has no mechanism to track this distribution — it only sees the current snapshot.  
**Impact**: GWT's attention allocation is brittle under module degradation: it may suddenly reallocate attention when a "most central" module degrades, causing oscillation.  
**Fix**: Implement **stochastic centrality tracking** in GWT: maintain a Markov chain over module centrality rankings, weight attention by expected pre-absorption time share rather than instantaneous centrality. This smooths attention oscillation under degradation.  
**Priority**: P1 — prevents attention oscillation under module fatigue/degradation.

### DEFECT N-642.3: GWT Lacks Feature-Aware Attention (Inflow/Outflow Centrality Gap)
**Source**: Papazian & Helms 2026 — Applied Network Science  
**Finding**: Inflow/outflow centrality metrics, derived from graph convolution aggregation, directly incorporate node features with graph structure. They prioritize nodes that maintain graph connectivity by emphasizing contributions of otherwise poorly-connected neighbors.  
**NeoTrix Defect**: GWT attention routing is topology-only — it uses edge weights and module connectivity but ignores module feature vectors (capability embeddings, emotion state vectors, self-model uncertainty). Inflow/outflow centrality shows that feature-aware metrics identify structurally different important nodes than topology-only metrics.  
**Impact**: GWT misses modules that are topologically peripheral but feature-rich (high information content), which inflow centrality would catch.  
**Fix**: Extend GWT salience to **feature-weighted centrality**: compute inflow centrality for each module using both topology and module feature vectors (capability embeddings). Modules with high feature diversity that maintain connectivity get boosted attention.  
**Priority**: P2 — important for GWT to reason over module content, not just position.

### DEFECT N-642.4: Betweenness Centrality Not Valid for Module Resilience Assessment
**Source**: Bhagat-Conway 2026 — Findings  
**Finding**: Betweenness centrality is not a valid resilience metric. It only considers shortest paths, not the presence of longer alternatives. Networks with identical betweenness centrality distributions can have vastly different resilience profiles (e.g., single bridge vs. bridge + redundant long route).  
**NeoTrix Defect**: NeoTrix uses betweenness centrality-like measures to assess module resilience (which modules are "critical" to system function). This is theoretically invalid — a module with high betweenness may have redundant alternatives that make it non-critical for resilience.  
**Impact**: NeoTrix over-estimates the criticality of high-betweenness modules and may allocate unnecessary redundant resources to protect them, while ignoring truly vulnerable modules with low betweenness but no alternatives.  
**Fix**: Replace betweenness-based resilience scoring with **i-betweenness** (no alternative path exists) combined with **detour-cost ratio** (ratio of shortest path to best alternative path). Modules with high i-betweenness and high detour-cost are truly critical.  
**Priority**: P1 — prevents misallocation of resilience resources.

### DEFECT N-642.5: Vulnerability is Localized, Not Distributed — Need Local Vulnerability Profiles
**Source**: Zakroum et al. 2026 — Applied Network Science  
**Finding**: Network structural weaknesses are distributed across *localized regions*, not globally. The reachability index, which aggregates neighboring nodes' vulnerabilities with distance-dependent weighting, outperforms global centrality attacks for identifying critical nodes. Local neighborhoods drive vulnerability, not global position.  
**NeoTrix Defect**: NeoTrix's module health monitoring treats vulnerability as a global property (module-level health scores). It does not compute localized vulnerability profiles that capture how each module's *immediate neighborhood* contributes to its fragility.  
**Impact**: A module may appear healthy globally but be highly vulnerable due to its local neighborhood structure. NeoTrix would miss this until failure occurs.  
**Fix**: Add **local vulnerability profiling** to module health: for each module, compute l-hop neighborhood vulnerability signature using random walk-based reachability (as in Zakroum et al.). Aggregate with distance-dependent weighting. Trigger preemptive repair when local vulnerability exceeds threshold, even if global health score is nominal.  
**Priority**: P1 — proactive failure prevention.

### DEFECT M-642.1: HyperCube Missing Hypergraph Modularity for Higher-Order Interactions
**Source**: Brandt-Tumescheit & Meyerhenke 2026 — Social Network Analysis and Mining  
**Finding**: Hypergraph modularity captures higher-order interactions (3+ entities) that pairwise projections destroy. The h-louvain algorithm optimizes hypermodularity but requires parameter tuning. New parameter-light variants match or exceed h-louvain accuracy without tuning overhead.  
**NeoTrix Defect**: HyperCube knowledge representation uses pairwise edges only. When knowledge involves higher-order interactions (e.g., 3+ concepts forming an irreducible cluster), HyperCube forces pairwise decomposition, losing the higher-order signal. This is the same problem pairwise projections have for community detection.  
**Impact**: HyperCube under-represents emergent knowledge that only manifests in higher-order interactions. Knowledge clusters that are meaningful as triples+ appear as disconnected pairs.  
**Fix**: Implement **hyperedge support** in HyperCube: allow edges of degree ≥ 3 (hyperedges). Use parameter-light hypermodularity variant to detect higher-order knowledge clusters. Map hyperedges to VSA embedding dimensions that capture multi-concept binding.  
**Priority**: P2 — extends HyperCube expressiveness for emergent knowledge.

### DEFECT M-642.2: HyperCube Lacks GPU-Accelerated Hierarchical Clustering
**Source**: Gilbert & Madduri 2026 — IPDPS 2026 (pLouvain/pLeiden)  
**Finding**: GPU-parallelized Louvain/Leiden achieves 3.1x–8.8x speedups. pLeiden is the first parallel implementation preserving all sequential Leiden quality guarantees via spanning-tree-based refinement. LambdaCC objective generalizes modularity.  
**NeoTrix Defect**: HyperCube hierarchical clustering runs on CPU only. For large knowledge graphs (millions of nodes), clustering takes minutes to hours. GPU acceleration with quality guarantees would bring this to seconds.  
**Impact**: HyperCube knowledge updates are slow, delaying SEAL evolution cycles that depend on re-clustering after new knowledge absorption.  
**Fix**: Implement **GPU-accelerated Leiden clustering** for HyperCube using spanning-tree refinement approach. Use LambdaCC objective for quality guarantee. Target 5-8x speedup on knowledge graphs > 100K nodes.  
**Priority**: P1 — directly accelerates SEAL evolution cycles.

### DEFECT M-642.3: HyperCube Low-Rank Graph Scaling Bottleneck
**Source**: Yu 2026 — VLouvain, EDBT 2026  
**Finding**: VLouvain handles massive low-rank graphs (similarity graphs from embeddings) without explicit edge construction. It bypasses edge iteration entirely, achieving mathematical equivalence to Louvain while handling graphs where explicit edge storage is O(n²) prohibitive.  
**NeoTrix Defect**: When HyperCube embeddings produce similarity graphs (cosine similarity between VSA vectors), the graph can be low-rank with O(n²) potential edges. HyperCube currently requires explicit edge construction, which exhausts memory on large knowledge bases.  
**Impact**: HyperCube cannot scale to knowledge bases with > 100K concepts when embeddings produce dense similarity graphs.  
**Fix**: Implement **vector-based implicit clustering** for HyperCube low-rank cases: skip explicit edge construction, compute modularity gains directly from VSA embedding dot products (as VLouvain does with augmented feature vectors).  
**Priority**: P1 — critical for scaling to production knowledge base sizes.

### DEFECT M-642.4: HyperCube Partitions Suffer from Modularity Degeneracy
**Source**: Mastrandrea et al. 2026 — arXiv:2602.21838  
**Finding**: Modularity has a highly degenerate landscape — many structurally distinct partitions achieve close modularity values. The STAR method selects a representative partition that best captures structural features shared across degenerate solutions, without additional optimization.  
**NeoTrix Defect**: HyperCube knowledge clustering uses modularity-like quality functions. When the knowledge graph has degenerate partitions, HyperCube may output arbitrary partitions from the degenerate set, causing non-reproducible knowledge organization.  
**Impact**: Same knowledge absorbed in different sessions may be clustered differently, breaking knowledge consistency across sessions.  
**Fix**: Add **STAR post-processing** to HyperCube clustering: after initial clustering, generate multiple degenerate partitions, apply STAR to select the representative partition that best captures shared structural features. Store partition hash for reproducibility checks.  
**Priority**: P2 — prevents knowledge organization drift across sessions.

### DEFECT M-642.5: SEAL Missing Temporal Community Detection for Dynamic Knowledge
**Source**: Brabant et al. 2026 — arXiv:2605.24450; ComNetX arXiv:2608.16906  
**Finding**: Generalized L-Modularity extends community detection to temporal networks with weighted, directed, continuous, delayed, and multipartite interactions. ComNetX enables local hierarchical updates without full recomputation. Together they handle evolving communities efficiently.  
**NeoTrix Defect**: SEAL evolution cycles use static modularity for knowledge clustering. When knowledge evolves over time (new concepts appear, old ones fade), SEAL must re-cluster from scratch. It has no temporal community detection that tracks how knowledge communities evolve.  
**Impact**: SEAL re-clustering is O(n log n) per cycle even when only 1% of knowledge changed. Temporal methods could reduce this to O(Δ) where Δ is the change magnitude.  
**Fix**: Implement **temporal L-Modularity** in SEAL: maintain longitudinal modularity across knowledge snapshots, use ComNetX-style local hierarchical updates for incremental re-clustering. Only re-cluster affected knowledge regions when new concepts arrive.  
**Priority**: P1 — reduces SEAL evolution cycle cost from O(n log n) to O(Δ).

### DEFECT M-642.6: SEAL Modularity Optimization Lacks Physics-Based Approach
**Source**: Zhou et al. 2026 — Communications in Theoretical Physics  
**Finding**: Free Energy Machine (FEM) framework minimizes free energy (combining mean-field theory + simulated annealing + automatic differentiation) to maximize modularity. FEM achieves higher modularity than greedy methods by leveraging gradient-based optimization over the energy landscape.  
**NeoTrix Defect**: SEAL uses greedy Louvain-style modularity optimization, which gets trapped in local optima. FEM's statistical physics approach explores the energy landscape more thoroughly via simulated annealing + gradient descent.  
**Impact**: SEAL detects suboptimal knowledge clusters because greedy optimization misses the global optimum in rugged modularity landscapes.  
**Fix**: Replace SEAL's greedy modularity optimizer with **FEM-based optimization**: implement mean-field energy function for knowledge graph modularity, use automatic differentiation for gradients, apply simulated annealing schedule for global exploration.  
**Priority**: P2 — improves cluster quality but adds computational cost; evaluate tradeoff.

---

## Summary Table

| ID | Domain | Defect | Source | Priority |
|----|--------|--------|--------|----------|
| G-642.1 | E8 | No core-routing phase for hexagram traversal | PBS (Mao 2026) | P1 |
| G-642.2 | E8 | Single-scale sampling distorts close-pair distances | Multi-scale APSP (Gupta 2026) | P1 |
| G-642.3 | E8 | No OGP detection for stable reasoning failure | OGP (Koehler 2026) | P2 |
| N-642.1 | GWT | Conflates local/global structural roles | AAB (Nature 2026) | P1 |
| N-642.2 | GWT | No stochastic centrality tracking under uncertainty | Absorbing MC (arXiv 2026) | P1 |
| N-642.3 | GWT | Topology-only attention, ignores module features | Inflow/Outflow (2026) | P2 |
| N-642.4 | NeoTrix | BC not valid for resilience assessment | Bhagat-Conway 2026 | P1 |
| N-642.5 | NeoTrix | Vulnerability is localized, not global | Zakroum 2026 | P1 |
| M-642.1 | HyperCube | No hypergraph modularity for higher-order | HyperModularity (2026) | P2 |
| M-642.2 | HyperCube | No GPU-accelerated hierarchical clustering | pLouvain/pLeiden (IPDPS 2026) | P1 |
| M-642.3 | HyperCube | Low-rank graph memory bottleneck | VLouvain (EDBT 2026) | P1 |
| M-642.4 | HyperCube | Modularity degeneracy causes non-reproducible clusters | STAR (arXiv 2026) | P2 |
| M-642.5 | SEAL | Static modularity, no temporal community detection | L-Modularity + ComNetX (2026) | P1 |
| M-642.6 | SEAL | Greedy optimization trapped in local optima | FEM (Comm. Theor. Phys. 2026) | P2 |

**Total New Defects**: 14 (9 × P1, 5 × P2)  
**Domains Covered**: E8 (3), GWT (3), NeoTrix general (2), HyperCube (4), SEAL (2)  
**Connection to Batch 641**: G-642.1/G-642.2 extend the "multi-signal routing" fix from 641. N-642.1/N-642.2/N-642.3 extend the "GWT budget ceiling" fix. M-642.2/M-642.3 extend "HyperCube hierarchical clustering" fix. M-642.5 extends SEAL pipeline fixes.
