# Iteration Batch 374 — Graph Algorithms, Network Optimization, Combinatorial Optimization

**Date**: 2026-09-06
**Research Scope**: 3 domains × 3 queries = 9 search vectors, 40+ papers/sources
**Goal**: Identify defects in NeoTrix design doc against latest 2026 advances

---

## Sources Cited

| # | Source | Domain | Key Advance |
|---|--------|--------|-------------|
| 1 | arXiv:2601.23207 — "Learning to Execute Graph Algorithms Exactly with GNNs" (Morris et al., 2026) | Graph Algorithms | Exact learnability proofs for LOCAL-model algorithms via NTK theory; GNN ensemble learns BFS/DFS/Bellman-Ford exactly |
| 2 | alphaXiv:2602.13106 — "Which Algorithms Can GNN Learn?" (Morris et al., ICML 2026) | Graph Algorithms | General framework for MPNN learnability; impossibility results for SSSP/MST; differentiable regularization loss for Bellman-Ford |
| 3 | arXiv:2601.19094 — "FloydNet: DP-style Global Refinement for Graph Reasoning" | Graph Algorithms | 3-WL expressive power via learned Floyd-Warshall operator; >99% on CLRS-30; exact TSP solutions at rates exceeding heuristics |
| 4 | arXiv:2603.01786 — "GFlowNets for Shortest Paths" | Graph Algorithms | Non-acyclic GFlowNets find exact shortest paths via flow minimization; competitive on Rubik's Cube |
| 5 | arXiv:2606.19185 — "AGDN: Anisotropic Graph Diffusion for TSP" | Combinatorial Optimization | MixScore transition matrix + anisotropic diffusion; outperforms all GNN baselines on TSP across sizes |
| 6 | arXiv:2601.13465 — "GNNs are Heuristics" (Min et al., 2026) | Combinatorial Optimization | Unsupervised NAR GNN as single-pass TSP heuristic; snapshot ensembling + MC dropout; 4.39% gap on TSP-100 without search |
| 7 | arXiv:2603.27922 — "GEAKG: Generative Executable Algorithm Knowledge Graphs" | Combinatorial Optimization | LLM-generated executable knowledge graphs with ACO learning; zero-shot cross-domain transfer |
| 8 | ACL 2026 — "AgentGL: Agentic Graph Learning with LLMs via RL" | Graph Algorithms | LLM-driven graph learning with RL; agentic exploration of graph structures |
| 9 | arXiv:2607.23467 — "DCGA: Joint Routing and Flow Allocation" | Network Optimization | Double-Channel Graph Attention for pickup-and-delivery; state-of-the-art on LinerLib |
| 10 | arXiv:2607.23116 — "KAYROS: Anytime+Exact TD-VRP Solver" | Network Optimization | First open-source anytime+exact TD-VRP solver; 468 optimality certificates; Poryos2026 benchmark |
| 11 | arXiv:2609.00859 — "RLEA: RL-Enhanced LLM Agents for VRP" | Network Optimization | Multi-agent LLM framework for VRP modeling; Soft Q-learning planner; 16.67% higher success rate |
| 12 | arXiv:2608.24859 — "POLAR: Cross-Problem VRP with Representation Disentanglement" | Network Optimization | Progressive Layered Extraction encoder; 21.3% gap reduction on 16 VRP variants |
| 13 | arXiv:2603.07568 — "Constraints Matrix Diffusion for VRP" | Network Optimization | Discrete noise graph diffusion for constraint assignment matrix; 378-combinatorial space evaluation |
| 14 | IJOC 2026 — "RouteOpt: Modular Exact VRP Solver" | Network Optimization | First open-source modular exact VRP solver; parallel B&B with node restoration |
| 15 | arXiv:2602.16012 — "CaR: Construct-and-Refine for VRP" | Network Optimization | Construction-improvement-shared representation; 6-8x speedup over neural baselines on hard-constrained VRP |
| 16 | arXiv:2602.00488 — "OD-DEAL: Large-Scale CVRP via Adversarial Learning" | Network Optimization | GFlowNet + HGS-BCC expert distillation; sub-second inference to 10,000 nodes |
| 17 | Springer 2026 — "GARNET: Random Walk Encoding + Rewiring for TSP" | Network Optimization | D-RRWP + random rewiring + GRASS attention; 0.05% gap on TSP-20 |
| 18 | MDPI 2026 — "GCRL-TSP: Heatmap-Assisted RL for Large TSP" | Network Optimization | Two-stage heatmap generation + PPO; >2x speed on TSP-200/500/1000 |
| 19 | arXiv:2604.06940 — "NICO-TSP: Neural Improvement for CO" | Combinatorial Optimization | Edge-centric 2-opt improvement; two-stage IL+RL training; step-efficient improvement |
| 20 | arXiv:2608.09042 — "DualCert: Constraint-Coupled TSP Learning" | Combinatorial Optimization | Constraint-coupled learning on KKT manifold; 0.0573% mean gap on TSP-1000; 67.1% below NeuroLKH |
| 21 | arXiv:2603.20702 — "DRLGA-TSP: Structural vs Numerical Parameter Decoupling" | Combinatorial Optimization | Structural plasticity >> numerical tuning for scalability; 45% gap reduction on rl5915 |
| 22 | arXiv:2606.22776 — "GeoRouteNet: Geometry-Aware NAR TSP" | Combinatorial Optimization | Centered node offsets + learnable radial bases; 3.60% gap on TSPLIB |
| 23 | MDPI 2026 — "DTALAN: Dynamic Topology-Aware Linear Attention for TSP" | Combinatorial Optimization | Temporal locality-aware attention; 0.55% gap on TSP-100; linear decoder complexity |
| 24 | arXiv:2603.28796 — "GaloisSAT: Hybrid GPU-CPU SAT Solver" | SAT/Satisfiability | Finite field algebraic SAT on GPU; 8.41x speedup on SAT-SAT; 1.29x on UNSAT |
| 25 | arXiv:2603.07176 — "Learning to Rank Initial Branching Order for SAT" | SAT/Satisfiability | GNN predicts initial branching order for CDCL; speedups on random 3-CNF and pseudo-industrial |
| 26 | IJCSIT 2026 — "GMTSAT: Multi-Slot Global Updates for SAT" | SAT/Satisfiability | Learnable global latent slots + gated message passing; outperforms NeuroSAT on variable assignment |
| 27 | IOP 2026 — "Deep Learning for SLS SAT Solvers with Performance Bounds" | SAT/Satisfiability | GNN as oracle factory for SLS; LLL loss with performance guarantees; 14% more instances solved |
| 28 | arXiv:2507.01825 — "MILP-SAT-GNN: GNN SAT Solver" | SAT/Satisfiability | SAT→MILP→bipartite GNN; universal approximation with RNI; permutation/equivalence invariance |
| 29 | Nature Comms 2026 — "AutoModSAT: LLM-driven SAT Solver Optimization" | SAT/Satisfiability | LLM heuristic discovery for SAT; 40% improvement over baseline; 30% over SOTA solvers |
| 30 | arXiv:2605.16632 — "Learning to Cube: Neuro-Symbolic SAT Post-Training" | SAT/Satisfiability | Transformer-based cubing heuristics for C&C; SFT+DPO surpasses frontier LLMs; pass@5=53 |
| 31 | arXiv:2608.14569 — "Certified Correctness in Neural Constraint Reasoning" (ICML 2026) | Constraint Satisfaction | Position paper: neural CSP needs symbolic integration for certified correctness |
| 32 | arXiv:2603.20801 — "LNS meets Iterative Neural Constraint Heuristics" | Constraint Satisfaction | ConsFormer-LNS: destroy/repair decomposition; stochastic destroy > greedy; greedy repair > sampling |
| 33 | arXiv:2604.02350 — "Differentiable Symbolic Planning (DSP)" | Constraint Satisfaction | Feasibility channel + global Φ aggregation + sparsemax; 97.4% on planning, 96.4% on SAT |
| 34 | JAIR 2026 — "Scaling Neuro-symbolic Problem Solving" | Constraint Satisfaction | E-PLL loss for learning constraints+objectives; no solver call at training; protein design application |
| 35 | arXiv:2602.18419 — "Hard CSP Benchmarks: Classical vs GNN" | Constraint Satisfaction | Classical algorithms (FMS) still outperform GNNs on hard CSPs; GNNs degrade on 4-SAT/5-col |
| 36 | CP 2026 — "Neurosymbolic Large Neighbourhood Search" | Constraint Satisfaction | CP + Masked Language Model for LNS; constrained text/molecule generation |
| 37 | CP 2026 — "From Literals to Atomic Constraints: CDCL for CP" | Constraint Satisfaction | Native CDCL for CP; atomic constraints replace SAT literals; CPIP nogoods |
| 38 | arXiv:2609.00577 — "GeoPAR: Geometry-Guided Parallel Autoregressive for Multi-Agent CO" | Combinatorial Optimization | Projection-window sparse geometry + conflict-aware assignment; scalable multi-agent routing |
| 39 | arXiv:2605.19721 — "LaGCO-RL: Projecting Latent RL Actions for GCO" | Combinatorial Optimization | Continuous latent action space; 16.2x faster inference; 40% better generalization |
| 40 | arXiv:2608.12443 — "SSPO: Structure-Aware Preference Optimization for NCO" | Combinatorial Optimization | Dissimilarity-weighted leave-one-out baseline; resolves gradient polarization + baseline redundancy |
| 41 | arXiv:2601.08696 — "Population-Based NCO" | Combinatorial Optimization | Neural population operators: cNI (contextual improvement) + cNC (conditioned constructive); diversity-aware restarts |
| 42 | OpenReview 2026 — "GCNCO: Gradient-Consistent NCO" | Combinatorial Optimization | Header-encoder-decoder with gradient consistency across COPs; foundational model for CO |
| 43 | arXiv:2602.18141 — "μ-ChebNet: Geometry-Induced Diffusion on Graphs" | Graph Algorithms | Learned node-wise density modifies Laplacian propagation geometry without rewiring |
| 44 | arXiv:2604.02927 — "LOGGIA: Delay-Aware Neural Routing" | Network Optimization | Log-space link weight GNN + delay-aware training; distributed deployment wins over centralized |
| 45 | MDPI 2026 — "CA-GAR: Congestion-Aware Adaptive Routing via GAT" | Network Optimization | Multi-head GAT + dynamic cost optimization; 50% delay reduction under bursty traffic |
| 46 | arXiv:2605.07113 — "Max-Cut via Feasibility-Preserving GNNs" | Combinatorial Optimization | GNN as neural SDP proxy in branch-and-bound; 10.6x speedup; self-supervised without SDP labels |
| 47 | arXiv:2604.27786 — "Expressive Power of GNNs to Solve Linear SDPs" | Combinatorial Optimization | 2-FWL is necessary+ sufficient for SDP; VC-2-FMPNN architecture; 80% warm-start speedup |
| 48 | PMLR 2025 — "BOPO: Preference Optimization for NCO" | Combinatorial Optimization | Best-anchored preference pairs + objective-guided pairwise loss; architecture-agnostic |
| 49 | ACM 2026 — "Applicability of NCO: Critical View" | Combinatorial Optimization | Metaheuristics on Pareto front with NCO; NCO faster but lower quality; transferability partial |
| 50 | arXiv:2604.23921 — "Crystal Structure Prediction via GNN-Combinatorial Optimization" | Combinatorial Optimization | Expander graphs + Gumbel-Sinkhorn for CSP; outperforms classical heuristics on materials |

---

## Defects Found

### D374-1: No Learned Graph Algorithm Execution Framework
**Severity**: HIGH
**Research**: Sources 1, 2, 3
**Finding**: 2026 proves GNNs can learn to execute graph algorithms *exactly* (BFS, DFS, Bellman-Ford) under bounded-degree constraints via NTK theory. FloydNet achieves 3-WL expressivity with learned DP operators, scoring >99% on CLRS-30. NeoTrix has no equivalent capability.
**NeoTrix Gap**: The `nt_core_hcube` module uses VSA HyperCube for knowledge representation but lacks a learned graph algorithm execution layer. The E8 Hexagram reasoning engine operates on symbolic states without learned traversal algorithms.
**Suggestion**: Add a `nt_core_graph_algo_learner` module that:
- Implements GNN-based exact algorithm execution for LOCAL-model algorithms
- Integrates with HyperCube as the graph substrate
- Provides learned BFS/DFS/Bellman-Ford as composable primitives for reasoning
- Uses FloydNet-style DP refinement for global graph reasoning tasks

### D374-2: No Neural Combinatorial Optimization Engine
**Severity**: HIGH
**Research**: Sources 5, 6, 19, 20, 40, 41, 42
**Finding**: NCO has matured dramatically: single-pass GNN heuristics (4.39% TSP-100 gap), constraint-coupled learning (0.0573% gap on TSP-1000), population-based neural operators, structure-aware preference optimization, and gradient-consistent cross-problem encoders. These are production-ready.
**NeoTrix Gap**: NeoTrix has no NCO engine. The SEAL pipeline handles evolution but not combinatorial optimization. NT-ACT handles tool orchestration but not combinatorial problem solving.
**Suggestion**: Add a `nt_core_nco_engine` module that:
- Provides learned TSP/VRP/CSP solvers as primitives
- Integrates with GWT for attention-based solution construction
- Supports both construction-based (autoregressive) and improvement-based (2-opt) policies
- Uses SSPO/BOPO training for variance-reduced policy gradients

### D374-3: No SAT/Constraint Solver Integration
**Severity**: HIGH
**Research**: Sources 24, 25, 29, 30, 31, 33, 34, 37
**Finding**: 2026 saw breakthroughs: GaloisSAT (8.41x speedup via GPU finite-field algebra), AutoModSAT (LLM-driven heuristic discovery, 40% improvement), Differentiable Symbolic Planning (96.4% SAT accuracy), and native CDCL for CP. Neural+symbolic integration is essential.
**NeoTrix Gap**: NeoTrix has no SAT solver or constraint programming integration. The ConsciousnessTree runs symbolic reasoning on E8 states but cannot solve SAT/CSP instances.
**Suggestion**: Add a `nt_core_sat_bridge` module that:
- Wraps CDCL solvers (Kissat/CaDiCaL) with GNN warm-start initialization
- Provides differentiable SAT/CSP checking for the SEAL pipeline
- Integrates with HyperCube for constraint-encoded knowledge queries
- Supports AutoModSAT-style LLM heuristic discovery for custom constraint domains

### D374-4: Missing Cross-Domain Gradient Consistency
**Severity**: MEDIUM
**Research**: Source 42
**Finding**: GCNCO proves that aligning gradient directions and magnitudes across multiple COPs via feature rotation matrices enables a shared encoder to learn generalized solving strategies. This is a foundational-model approach for CO.
**NeoTrix Gap**: The SEAL pipeline evolves modules independently. The CapabilityBridge maps between evolution and runtime views but does not enforce gradient consistency across domains. Each domain's SelfTest operates in isolation.
**Suggestion**: Add gradient consistency constraints to the SEAL pipeline:
- Align optimization trajectories across NT-CORE (reasoning), NT-ACT (action), and NT-MIND (evolution)
- Use feature rotation matrices to homogenize the optimization landscape
- Track cross-domain gradient alignment as a new SelfTest metric

### D374-5: No Population-Based Search in SEAL Pipeline
**Severity**: MEDIUM
**Research**: Sources 41, 40, 6
**Finding**: Population-based NCO (PB-NCO) with contextual improvement + conditioned constructive policies outperforms single-solution methods. Structure-aware preference optimization (SSPO) resolves gradient signal polarization by weighting co-sampled solutions by structural dissimilarity.
**NeoTrix Gap**: The SEAL pipeline runs single-trajectory evolution (one exploration → one distillation → one absorption). No population of candidate solutions is maintained. The experience-tree writes single snapshots, not populations.
**Suggestion**: Extend SEAL to population-based evolution:
- Maintain a population of candidate architectures/solutions per cycle
- Use SSPO-style structural dissimilarity weighting for experience selection
- Implement diversity-aware restarts (cNC pattern) when population stagnates
- Track population diversity as a health metric in HeartbeatAggregator

### D374-6: No Delay-Aware/Online Neural Routing
**Severity**: MEDIUM
**Research**: Sources 44, 45, 9
**Finding**: LOGGIA proves that neural routing must model communication and inference delays; distributed deployment outperforms centralized. CA-GAR achieves 50% delay reduction via GAT-based dynamic cost optimization under bursty traffic.
**NeoTrix Gap**: GWT broadcasts salient information across modules but assumes instantaneous broadcast (no communication delay modeling). The EventBus is synchronous within processes but has no delay-aware routing for cross-domain signals.
**Suggestion**: Add delay-aware signal routing to GWT:
- Model inference latency per module as a routing cost
- Use GAT-based link weight prediction for inter-module communication
- Implement distributed (per-domain) state observation, matching LOGGIA's Local-Multi finding
- Add congestion detection for EventBus under high-throughput conditions

### D374-7: No Feasibility-Preserving Neural Verification
**Severity**: MEDIUM
**Research**: Sources 31, 46, 47
**Finding**: Feasibility-preserving GNNs can serve as neural SDP proxies in exact branch-and-bound (10.6x speedup). Certified correctness requires symbolic integration — neural-only methods fail under distribution shift (Source 31).
**NeoTrix Gap**: SelfTest tiers (T1/T2/T3) check existence, registration, and production wiring but do not verify mathematical feasibility of outputs. The ConsciousnessTree health checks are structural, not constraint-based.
**Suggestion**: Add feasibility verification to SelfTest framework:
- T4 tier: mathematical feasibility checking (SDP/LP constraint satisfaction)
- Use GNN-based SDP proxy for fast feasibility pre-screening
- Integrate symbolic verification as a post-condition for module outputs
- Track constraint violation rates in HeartbeatAggregator

### D374-8: No Executable Algorithm Knowledge Graph
**Severity**: MEDIUM
**Research**: Source 7
**Finding**: GEAKG demonstrates that executable algorithm knowledge graphs with ACO-learned traversal can transfer zero-shot across domains. Nodes store executable operators, edges encode learned composition patterns.
**NeoTrix Gap**: The KB stores static knowledge (nodes, edges, embeddings). The experience-tree stores distilled experiences. Neither represents executable algorithmic procedures that can be composed and traversed.
**Suggestion**: Add an executable knowledge graph layer to NT-MEMORY:
- Nodes = executable operators (Rust functions, tool invocations, SEAL stages)
- Edges = ACO-learned composition patterns
- Traversal generates solution procedures
- Integrates with existing KB via new namespace `executable_algo_kg`

### D374-9: Missing Structural Plasticity Control
**Severity**: LOW
**Research**: Source 21
**Finding**: DRLGA-TSP proves structural parameters (population size, operator switching) are more decisive than numerical parameters (crossover/mutation rates) for scalability. Structural plasticity prevents stagnation.
**NeoTrix Gap**: Rune Socketing provides per-module configuration (5 colors) but lacks dynamic structural reconfiguration. The Constellation maturity ladder (C0-C6) is static per module, not dynamically adapted.
**Suggestion**: Add structural plasticity to Rune Socketing:
- Dynamic rune slot count based on optimization phase
- Operator switching (different Runewords per cycle stage)
- Population-level structural parameters for SEAL pipeline
- Track structural adaptation rate as a health signal

### D374-10: No Geometry-Aware Propagation in HyperCube
**Severity**: LOW
**Research**: Sources 17, 22, 43
**Finding**: μ-ChebNet learns node-wise density to modify Laplacian propagation geometry without rewiring. GeoRouteNet uses centered node offsets + learnable radial bases for geometry-aware TSP. GARNET combines D-RRWP + rewiring for multi-hop structural encoding.
**NeoTrix Gap**: VSA HyperCube uses fixed high-dimensional vector operations. No learned geometry-aware propagation. The E8 Hexagram uses fixed hexagonal grid adjacency without adaptive propagation.
**Suggestion**: Add geometry-aware propagation to HyperCube:
- Learn node-wise density weights for HyperCube vector propagation
- Use D-RRWP-style multi-hop encoding for long-range concept relationships
- Implement adaptive propagation geometry that changes per reasoning task
- Track propagation quality via attention resonance scores

---

## Summary

| Category | Count | Severity |
|----------|-------|----------|
| Graph Algorithm Execution | 1 | HIGH |
| Neural Combinatorial Optimization | 1 | HIGH |
| SAT/Constraint Integration | 1 | HIGH |
| Gradient Consistency | 1 | MEDIUM |
| Population-Based Search | 1 | MEDIUM |
| Delay-Aware Routing | 1 | MEDIUM |
| Feasibility Verification | 1 | MEDIUM |
| Executable Knowledge Graphs | 1 | MEDIUM |
| Structural Plasticity | 1 | LOW |
| Geometry-Aware Propagation | 1 | LOW |
| **Total** | **10** | **3H/5M/2L** |

**Top Priority**: D374-1 (learned graph algorithms), D374-2 (NCO engine), D374-3 (SAT bridge) — these represent fundamental capabilities that 2026 has proven are learnable and deployable.
