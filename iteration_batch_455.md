# Iteration Batch 455 — Graph Analytics & Visualization Gap Analysis

**Date**: 2026-09-06
**Focus**: Graph analytics, social network analysis, graph visualization advances (2026)
**Sources**: 18 papers/reports, codebase deep-dive

---

## Sources Cited

| # | Source | Year | Key Advance |
|---|--------|------|-------------|
| S1 | Papazian & Helms, "Inflow and Outflow Centrality" (Applied Network Science, Feb 2026) | 2026 | GCN-inspired centrality incorporating node features with graph structure; non-biased toward hubs |
| S2 | Liu et al., "SPSLS: Shortest-Path Semi-Local Centrality" (J Big Data, Apr 2026) | 2026 | Semi-local centrality integrating path lengths + path counts + avg shortest paths; 1.2% SIR improvement |
| S3 | Nature Scientific Reports, "HSSLC: Hierarchical Scalable Semi-Local Centrality" (Mar 2026) | 2026 | Multi-level entropy-based neighborhood info + hierarchical key node identification; 2.8% Kendall improvement |
| S4 | Bendahman & Lotfi, "Influence Disparity Centrality" (J Computational Social Science, Mar 2026) | 2026 | Degree superiority + neighborhood non-redundancy; non-obvious bridge detection |
| S5 | Chen & Liu, "KSvoterank" (Int J ML & Cybernetics, Aug 2026) | 2026 | K-shell decomposition + enhanced voting for dispersed key nodes; mitigates rich-club effect |
| S6 | ScienceDirect, "RMCG: Restart Markov Chain Gravity Centrality" (Jul 2026) | 2026 | Deterministic analytical framework replacing stochastic RWR; zero-variance node influence estimation |
| S7 | KDD 2026, "BRAVA-GNN" (arXiv, Feb 2026) | 2026 | Lightweight GNN for betweenness centrality approximation; 54× fewer params, 214% Kendall improvement on road networks |
| S8 | ACL 2026 Findings, "GenNIE" (2026) | 2026 | LLM generative reasoning framework for node importance estimation; cross-domain transfer |
| S9 | SICDIM (Social Network Analysis and Mining, Jul 2026) | 2026 | Swarm intelligence (MRFO + PeSOA) for community detection via influence maximization |
| S10 | IMCCIS (ScienceDirect, Mar 2026) | 2026 | Community-level influence score combining intra/inter-community with adaptive weights |
| S11 | Yang et al., "MaDGNN" (Scientific Reports, Mar 2026) | 2026 | Deep RL framework (GNN + Munchausen DQN) for influence maximization on large-scale networks |
| S12 | SIMBA (arXiv, Aug 2026) | 2026 | Lightweight neural surrogate + batched multi-swap simulated annealing; diffusion-model-agnostic IM |
| S13 | HOEE (J AI & CR, Feb 2026) | 2026 | Higher-order edge enhancement for temporal community detection; triangle motif reconstruction |
| S14 | Community-IM++ (arXiv, Feb 2026) | 2026 | Community-based diffusion degree (CDD) + progressive budgeting; 100× faster than greedy |
| S15 | Context-KG (arXiv, Apr 2026) | 2026 | LLM-driven ontology-guided KG visualization; context-aware spatial layout + insight generation |
| S16 | NetworkCanvas (CHI 2026) | 2026 | Progressive network visualization with adaptive recommendations; 10 analytical question categories |
| S17 | Fling (arXiv, Aug 2026) | 2026 | Inductive graph layout via implicit neural fields; O(mN) per step vs O(N²) for stress |
| S18 | Scanopy C4 (Jul 2026) | 2026 | Multi-view C4-style topology visualization; edge-type-per-view separation |

---

## Defects Found in NeoTrix Architecture

### DEFECT-G01: Centrality Algorithms Stuck in Classical Era (CRITICAL)

**Location**: `nt_core_kg_traversal.rs:57-65`, `nt_core_kg_traversal.rs:220-230`

**Current state**: NeoTrix implements only 4 classical centrality metrics: degree, betweenness, closeness, eigenvector. These are O(N²) or O(NM) exact computations.

**2026 gap**:
- **Inflow/Outflow centrality** (S1) — GCN-inspired metrics that incorporate node features with structure, prioritizing nodes crucial to connectivity that degree centrality misses entirely
- **Semi-local centrality (SPSLS)** (S2) — Balances accuracy and computational complexity; captures path-level influence without global computation
- **Hierarchical entropy-based centrality (HSSLC)** (S3) — Multi-level entropy evaluation for weighted networks
- **Influence Disparity Centrality (IDC)** (S4) — Detects non-obvious bridges between weakly connected regions (directly relevant to NT-WORLD cross-domain discovery)

**Impact**: NeoTrix cannot identify structurally strategic nodes that bridge weakly connected regions. The KG traversal module systematically mis-ranks nodes in heterogeneous knowledge graphs, leading to suboptimal attention routing in GWT.

**Suggestion**: Add a `CentralitySuite` trait with pluggable implementations:
- `InflowCentrality` (feature-aware, S1)
- `SemiLocalCentrality` (SPSLS-inspired, S2)
- `InfluenceDisparity` (bridge detection, S4)
- Wire `nt_core_self_measure::eigenvector_centrality` output into KG traversal layer (currently isolated in self-measure, never consumed by graph analysis)

---

### DEFECT-G02: Community Detection is Placeholder-Only (CRITICAL)

**Location**: `nt_core_kg_traversal.rs:48-55` — `Community` struct exists but no algorithm implementation found in codebase

**Current state**: The `Community` struct has `id`, `nodes`, `density`, `modularity` fields but no actual detection algorithm. Grep across all `.rs` files returns zero hits for `louvain`, `leiden`, `modularity_optimization`, or `label_propagation`.

**2026 gap**:
- **SICDIM** (S9) — Swarm intelligence community detection achieving Q=0.4737 on PolBooks, NMI=0.6228
- **IMCCIS** (S10) — Community-based influence maximization with adaptive intra/inter-community weight tuning
- **HOEE** (S13) — Higher-order edge enhancement for temporal community detection; triangle motif reconstruction preserves temporal stability
- **Community-IM++** (S14) — Explicit inter-community influence modeling; 100× faster than greedy

**Impact**: NeoTrix's 7-domain faction structure (NT-CORE through NT-FEEL) is essentially unpartitionable. No algorithm can detect natural communities within the knowledge graph, meaning cross-domain knowledge flows are invisible. The ConsciousnessTree cannot identify which domains are forming natural clusters vs. which are isolated.

**Suggestion**:
1. Implement Louvain (scalable modularity optimization) as baseline
2. Add higher-order community detection (HOEE-inspired) for temporal knowledge graphs
3. Wire community detection into ConsciousnessTree `connectivity_score` computation (currently only counts wired leaves, ignores community structure)

---

### DEFECT-G03: Influence Propagation Model is Missing (HIGH)

**Location**: No dedicated influence propagation module found. `nt_core_retrieval.rs` has graph topology tool but no diffusion modeling.

**Current state**: NeoTrix has no model for how influence/attention/information propagates through its knowledge graph. The GWT attention routing is local (broadcasts salient info) but doesn't model cascading effects.

**2026 gap**:
- **MaDGNN** (S11) — Cascade-aware GNN capturing high-order topological features and diffusion effects
- **Pressure Threshold Model** (ICWSM 2026) — Extends Linear Threshold model; densely connected networks amplify pressure effects
- **SIMBA** (S12) — Diffusion-model-agnostic influence maximization; trains on observed diffusion outcomes
- **VNSDPSO** (Jul 2026) — Variable neighborhood search for influence maximization in heterogeneous networks

**Impact**: NeoTrix cannot predict which knowledge nodes will become influential if activated. The SEAL pipeline (NT-MIND) cannot plan which skills to absorb to maximize cross-domain knowledge propagation. The `attention_manager` in NT-CORE routes attention locally but cannot model cascading influence.

**Suggestion**: Add `InfluencePropagation` module to NT-CORE:
- IC/LT diffusion models for knowledge graph edges
- Community-aware seed selection (IMCCIS-inspired, S10)
- Integration with GWT attention routing: predict which broadcasts will cascade effectively

---

### DEFECT-G04: No Graph Visualization Layer (HIGH)

**Location**: No visualization module found. `grep` for `visualization` returns only string literals in hexagram descriptions and a game rendering mode enum.

**Current state**: NeoTrix generates rich graph data (KB nodes/edges, KG traversal results, consciousness tree topology) but has zero capability to render, explore, or interact with these graphs visually.

**2026 gap**:
- **Context-KG** (S15) — LLM-driven ontology-guided KG visualization with user preferences, type-aware regions, and automated insight generation
- **NetworkCanvas** (S16) — Progressive visualization with adaptive recommendations across 10 analytical question categories (centrality, community, path, anomaly, what-if)
- **Fling** (S17) — Inductive neural field layout; O(mN) vs O(N²) stress; unseen node placement in single forward pass
- **Scanopy C4** (S18) — Multi-view separation (physical/logical/workloads/applications), edge-type-per-view, C4 zoom levels

**Impact**: NeoTrix's knowledge graph, consciousness tree, and capability tree are invisible to users. No way to debug attention routing, verify community structure, or explore cross-domain knowledge flows. The `tool_graph_topology` MCP endpoint returns JSON that cannot be visually inspected.

**Suggestion**:
1. Add `nt_io::graph_viz` module with force-directed + hierarchical layouts
2. Implement Context-KG-style ontology-aware layout (NeoTrix already has domain ontology)
3. Add C4-style zoom for consciousness tree exploration (collapsed overview → domain → module → file)
4. Expose via Tauri webview or standalone web server

---

### DEFECT-G05: Temporal Graph Analysis Absent (HIGH)

**Location**: `nt_core_hcube/topology.rs` — `PersistentHomology` computes Betti numbers at static scales; no temporal dimension.

**Current state**: Persistent homology captures topological features at different scales but only for a static snapshot. No ability to track how the knowledge graph topology evolves over time.

**2026 gap**:
- **HOEE** (S13) — Higher-order edge enhancement for temporal community detection; reconstructs lost triangle motifs between snapshots
- **Dynamic Community Detection on Temporal Graphs** (arXiv, Aug 2026) — Diffusion-guided contrastive learning for evolving communities
- **RMCG** (S6) — Restart Markov chain captures steady-state influence; deterministic instead of stochastic

**Impact**: NeoTrix runs SEAL evolution cycles but cannot detect whether knowledge graph communities are stabilizing or fragmenting over time. The `topology_score` in `ConsciousnessReview` is a static snapshot, not a trajectory. Cannot detect temporal community drift.

**Suggestion**:
1. Add temporal snapshots to `PersistentHomology` — track Betti curves over evolution cycles
2. Implement temporal community detection (HOEE-inspired) for KB evolution tracking
3. Wire temporal topology trends into ConsciousnessTree growth metrics (Phase 5 review)

---

### DEFECT-G06: Centrality in Self-Measure is Isolated from Graph Layer (MEDIUM)

**Location**: `crates/neotrix-types/src/core/self_measure/self_measure_impl/pid.rs:136` — `eigenvector_centrality()` exists but operates on correlation matrix of subsystem metrics, not on KG

**Current state**: `eigenvector_centrality` is used for computing Phi (IIT integration measure) on the subsystem correlation matrix. This is fundamentally different from KG node centrality — it measures subsystem integration, not knowledge graph structure.

**2026 gap**: GenNIE (S8) demonstrates that LLM-based generative reasoning can unify topology perception and semantic importance estimation across domains.

**Impact**: Two separate centrality computations exist with no cross-talk. The Phi computation (consciousness integration measure) and KG centrality (knowledge importance) are computed independently. A unified framework could identify nodes that are both structurally central AND semantically important.

**Suggestion**: Add `UnifiedNodeImportance` that fuses:
- Structural centrality (KG-based)
- Semantic importance (embedding-based)
- Consciousness integration contribution (Phi-based)
- Use GenNIE-style LLM reasoning for cross-domain importance estimation

---

### DEFECT-G07: No GNN-Based Graph Learning (MEDIUM)

**Location**: No GNN modules found in codebase.

**Current state**: NeoTrix uses classical algorithms (Dijkstra, BFS, eigenvector iteration) for all graph computations. No graph neural network capability exists.

**2026 gap**:
- **BRAVA-GNN** (S7) — Lightweight GNN for betweenness approximation; 54× fewer parameters, generalizes to road networks
- **MaDGNN** (S11) — Cascade-aware GNN for influence maximization
- **SIMBA** (S12) — Lightweight 2-layer GNN surrogate for diffusion prediction

**Impact**: NeoTrix cannot learn from graph structure. Every computation is model-free (handcrafted algorithms). As the knowledge graph grows, classical algorithms become prohibitively expensive and miss patterns that GNNs would capture.

**Suggestion**: Add `nt_core::graph_learning` module:
- Lightweight GNN (BRAVA-inspired) for betweenness approximation on large KB
- GNN-based link prediction for knowledge graph completion
- Integration with VSA HyperCube: GNN learns embeddings that feed into vector symbolic operations

---

### DEFECT-G08: Dispatch Topology Ignores Graph-Theoretic Properties (MEDIUM)

**Location**: `nt_mind/evolution/agent_capability/mod.rs:814` — `DispatchTopology` routes tasks by `AttentionDomain` but doesn't use centrality/community/bridge metrics

**Current state**: `DispatchTopology` assigns agents to attention domains based on task type. It audits and repairs assignments but doesn't leverage graph-theoretic properties of the dispatch graph itself (e.g., which agent-domain assignments create bridges vs. clusters).

**2026 gap**:
- **KSvoterank** (S5) — K-shell + voting for dispersed key node identification; prevents rich-club effect in agent assignment
- **IDC** (S4) — Influence disparity centrality identifies non-obvious bridges

**Impact**: Agent assignments may cluster around a few "popular" domains, creating rich-club effects. Bridge agents (those connecting disparate domains) are not explicitly identified or protected.

**Suggestion**: Enhance `DispatchTopology` with:
- K-shell decomposition of the agent-domain assignment graph
- Bridge detection (IDC-inspired) to protect cross-domain agents
- Community-aware assignment to prevent domain clustering

---

## Summary

| Severity | Count | Defects |
|----------|-------|---------|
| CRITICAL | 2 | G01 (classical centrality), G02 (no community detection) |
| HIGH | 3 | G03 (no influence propagation), G04 (no visualization), G05 (no temporal analysis) |
| MEDIUM | 3 | G06 (isolated centrality), G07 (no GNN), G08 (dispatch ignores graph properties) |

**Key architectural insight**: NeoTrix's graph layer is structurally complete (nodes, edges, traversal, topology) but algorithmically frozen circa 2020. The 2026 research landscape has moved to:
1. **Feature-aware centrality** (node attributes + structure)
2. **Semi-local scalability** (avoid O(N²) global computation)
3. **LLM-integrated graph reasoning** (GenNIE, Context-KG)
4. **Temporal/dynamic analysis** (communities that evolve)
5. **Inductive neural layouts** (Fling — generalize to unseen nodes)

The most impactful single fix would be implementing community detection (DEFECT-G02) as it would unlock cross-domain knowledge flow visibility, which feeds directly into GWT attention routing optimization and SEAL evolution planning.
