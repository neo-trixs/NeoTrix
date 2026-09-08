# Iteration Batch 336 — Graph/Network Architecture Defect Analysis

**Date**: 2026-09-06
**Domain**: Graph Neural Networks, Network Science, Social Network Analysis
**Cycle**: 336/10000+

---

## 1. Sources Cited

### Graph Neural Networks / Foundation Models
| # | Source | Year | Key Contribution |
|---|--------|------|-----------------|
| S1 | AnyGraph: Graph Foundation Model in the Wild (ACL 2026, Xia & Huang) | 2026-07 | Unified GFM via Graph Mixture-of-Experts; zero-shot cross-domain transfer; 38 datasets; handles structure/feature heterogeneity |
| S2 | GraphBFF: Billion-Scale Graph Foundation Models (arXiv:2602.04768) | 2026-02 | Billion-parameter GFM for large-scale heterogeneous graphs; neural scaling laws for graphs |
| S3 | What Makes Graph Unified? Generative Sliding-Window Transformer for GFMs (arXiv:2607.27966) | 2026-07 | Principles for graph unification; sliding-window attention for graph tokenization |
| S4 | MxGPS: Multiplex Graph Transformers for Power Grid FM (arXiv:2607.13763) | 2026-07 | Multiplex graph transformers for cross-layer heterogeneous reasoning |
| S5 | ICML 2026 Workshop: Graph Foundation Models | 2026-07 | Cross-graph transfer; graph tokenization; pretraining objectives; scaling laws; schema as benchmark |
| S6 | ESANN 2026 Tutorial: Learning on Knowledge and Heterogeneous Graphs in GFM/LLM Era | 2026-03 | Unified formalism for typed heterogeneous graphs + temporal KGs; LLM-graph integration taxonomy |
| S7 | GraphSculptor: Pre-training Coreset for Graph SSL (IJCAI 2026) | 2026-05 | 10% coreset achieves 99.6% full-data performance; 90% pre-training time reduction |
| S8 | Generative and Contrastive Graph Representation Learning (Apple, arXiv:2505.11776) | 2025-09 | Community-aware node-level contrastive + graph-level contrastive unified framework |
| S9 | MUG: Meta-path-aware Universal Heterogeneous Graph Pre-Training (2026) | 2026-02 | O(10M-100M) node scale; type semantics; meta-path aware pre-training |

### Temporal Graph / Dynamic Learning
| # | Source | Year | Key Contribution |
|---|--------|------|-----------------|
| S10 | LTFDyG: Learnable Temporal Function-based Dynamic GNN (2026) | 2026-05 | Hierarchical encoding: local→global temporal-structural fusion; dual-channel encoding |
| S11 | TempReasoner: Neural Temporal Graph Networks for Event Timeline (Sci Rep 2026) | 2026-01 | Dynamic spatio-temporal attention; temporal consistency loss; 94.3% ordering accuracy; 127ms latency |
| S12 | GRHNet: Temporal KG Reasoning with Global+Recent History (2026) | 2026-05 | Entity-inter structural dependencies; 3% MRR improvement; concurrent event modeling |
| S13 | TCGAT: Temporal Causal Graph Attention Network (2026) | 2026-03 | Granger causality + temporal embeddings in GAT framework |
| S14 | DynaGen: Unified Temporal KG Reasoning with Dynamic Subgraphs (2025-12) | 2025-12 | Entity-centric dynamic subgraphs; dual-branch GNN encoder |
| S15 | DGOTTA: Test-Time Adaptation on Dynamic Graphs (arXiv:2608.27948) | 2026-08 | Temporal memory-aware online TTA; structural+semantic evolution handling |
| S16 | Explainable TKF via Temporal Relation Tree-Graph (Neural Networks, 2026) | 2026-06 | Explainable temporal relation trees; multi-hop reasoning |

### Community Detection / Network Science
| # | Source | Year | Key Contribution |
|---|--------|------|-----------------|
| S17 | DGAT-OCD: Overlapping Community Detection via Dynamic Graph Attention (2026) | 2026 | End-to-end; long-range cross-community dependencies + fine-grained local topology |
| S18 | Multi-objective Dynamic Community Detection (Taylor & Francis, 2026) | 2026-03 | Real-time processing; social recommendation + anomaly monitoring |
| S19 | IPCwalks: Influence Propagation-Controlled Walks for HIN Community Detection (2026) | 2026 | Metapath-guided random walks; influence propagation control; low-dim representation |
| S20 | cograph: Complex Network Analysis R Package (2026) | 2026-03 | 11 community algorithms (Louvain/Leiden/Infomap); 24 centrality metrics; heterogeneous/multi-layer support |
| S21 | Reddit Deplatforming and Cohort Mixing (Scientific Reports, 2026) | 2026-07 | Deplatforming effects on community structure; cross-platform migration patterns |

### Social Network Analysis / Opinion Dynamics
| # | Source | Year | Key Contribution |
|---|--------|------|-----------------|
| S22 | IntervenSim: Intervention-Aware Social Network Simulation (arXiv:2604.06600) | 2026-04 | LLM-based closed-loop intervention+interaction simulation; 41.6% MAPE improvement |
| S23 | Survey on Opinion Dynamics: From Rule-based to Data-driven (Neurocomputing, 2026) | 2026-07 | Comprehensive taxonomy; structural+textual data integration |
| S24 | AI Role in Combating Misinformation: Text Mining + SNA (Sci Rep, 2026) | 2026-07 | RoBERTa+GRU hybrid; contextual+sequential embeddings for misinformation |
| S25 | Misinformation Detection on OSN Using Deep Learning (Neurocomputing, 2026) | 2026-01 | Persistent challenge; conspiracy/health/ethnic dimensions |
| S26 | Neural-Symbolic Methods for KG Reasoning: A Survey (ACM TKDD, 2025) | 2025-02 | Neural-symbolic taxonomy; KG completion + rule learning; LLM+KG integration |
| S27 | Integrating Graphs, LLMs, and Agents: Reasoning and Retrieval (arXiv:2604.15951) | 2026-04 | Graph-based agent memory taxonomy; GraphRAG; agentic graph reasoning |
| S28 | GraphAgents: KG-guided Agentic AI for Cross-domain Design (arXiv:2602.07491) | 2026-02 | Knowledge graph-guided multi-agent reasoning |

---

## 2. Defects Found in NeoTrix Design

### DEFECT-G001: KB Schema Lacks Heterogeneous Graph Typing
**File**: `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:176-209`
**Severity**: HIGH
**Gap**: The KB schema defines `nodes` with a single `node_type TEXT` column and `edges` with `relation_type TEXT`. This is a homogeneous graph model. 2026 research (S1-S6) demonstrates that **heterogeneous graph foundation models** require typed schemas with: node type hierarchies, relation type constraints (schema/meta-graph), temporal edge properties, and multi-relational neighborhoods. NeoTrix's VSA HyperCube operates on undifferentiated vectors while modern GFMs (AnyGraph, GraphBFF) learn type-dependent projections and relation-specific attention.

**Impact**: Cannot model multi-entity, multi-relational systems (e.g., KB nodes of different types with type-constrained edges). The ESANN 2026 tutorial (S6) explicitly states that "differences across method families boil down to modeling choices—how we represent types, attributes, text, and time."

### DEFECT-G002: No Temporal Edge Properties in Graph Schema
**File**: `neotrix-core/src/unified/core/nt_core_kb_primitives.rs:200-209`
**Severity**: HIGH
**Gap**: Edges lack `valid_from`, `valid_to`, `confidence_over_time`, or `event_timestamp` columns. Temporal knowledge graph reasoning (S10-S16) is now a core capability: TempReasoner achieves 94.3% ordering accuracy with temporal consistency loss; DynaGen uses entity-centric dynamic subgraphs; DGOTTA handles structural+semantic evolution. NeoTrix has `temporal_facts` table (separate) but edges themselves are temporally static — you cannot model "fact X was true from T1 to T2" or "relationship R evolved over time."

**Impact**: The KB cannot answer temporal queries like "what did entity A know about B at time T?" or track relationship evolution. The SEAL pipeline's temporal awareness is disconnected from the graph structure.

### DEFECT-G003: Naive Community Detection Algorithm
**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/graph_analysis.rs:10-35`
**Severity**: MEDIUM
**Gap**: The `detect_communities()` method uses a simple BFS flood-fill that assigns all reachable unvisited nodes to the same community. This ignores edge weight, directionality, and density optimization. 2026 research (S17-S20) shows: DGAT-OCD captures overlapping communities via dynamic graph attention; IPCwalks uses influence propagation-controlled walks; the `cograph` R package alone implements 11 algorithms (Louvain, Leiden, Infomap, etc.). NeoTrix's code graph communities are structurally meaningless — they merely identify connected components.

**Impact**: Community-based operations (GWT attention routing by community, skill crystallization by domain clustering, module health scoring) produce unreliable results. The ConsciousnessTree's branch weakness detection cannot accurately identify underperforming code domains.

### DEFECT-G004: No Graph Pre-training / Self-supervised Learning Pipeline
**File**: `neotrix-core/src/unified/core/nt_core_hcube/vsa.rs` (entire VSA module)
**Severity**: HIGH
**Gap**: The VSA HyperCube uses fixed random vectors with bind/bundle/permute operations. There is no pre-training pipeline. 2026 research (S7-S9) shows: GraphSculptor achieves 99.6% of full-data performance with 10% coreset; MUG scales to O(100M) nodes with meta-path-aware pre-training; community-aware contrastive learning (S8) provides robust positive/negative pairs. NeoTrix's embeddings are static and never improve from data — they are initialized once and used forever.

**Impact**: Knowledge representations cannot improve from accumulated experience. The KB embeddings are frozen at creation time while the system processes billions of tokens. Each new graph traversal starts from scratch rather than building on learned structural patterns.

### DEFECT-G005: GWT Attention Routing Ignorant of Graph Structure
**File**: `neotrix-core/src/unified/core/nt_core_gwt/workspace.rs`
**Severity**: MEDIUM-HIGH
**Gap**: GWT broadcast salience is computed from urgency+novelty+coherence (resonance matrix) but does not incorporate graph centrality measures (PageRank, betweenness, eigenvector centrality) or community structure. 2026 research (S20, S27) shows that network-aware attention routing significantly outperforms structure-agnostic methods. The GWT's `SpecialistModule` competition doesn't consider which specialists are most central in the knowledge graph or which communities they bridge.

**Impact**: Attention allocation may prioritize peripheral modules over structurally important ones. The MARS dual-process (System 1 GWT + System 2 Tree) lacks graph-topological grounding — a specialist with high betweenness centrality (bridge between knowledge domains) should receive more attention than one in a dense local cluster.

### DEFECT-G006: No Overlapping / Fuzzy Community Support
**File**: `neotrix-core/src/unified/layers/cognition/nt_mind/graph_analysis.rs:11-35`
**Severity**: MEDIUM
**Gap**: Community detection assigns exactly one community ID per node (HashMap<String, usize>). Real-world knowledge graphs have overlapping communities — a concept can belong to multiple domains simultaneously. DGAT-OCD (S17) explicitly handles this with "long-range cross-community dependencies and fine-grained local topology." NeoTrix's skill domain mapping (Context.md UCN mapping table) shows that skills can span multiple domains (e.g., Res-深 maps to NT-MIND, but research skills could also belong to NT-CORE). The binary community assignment loses this nuance.

**Impact**: Cross-domain capabilities (like experience-tree which writes to NT-MEMORY but is triggered by any domain) get forcibly assigned to one community, breaking multi-domain routing.

### DEFECT-G007: No Misinformation / Claim Verification in NT-SHIELD
**File**: `neotrix-core/src/unified/layers/action/nt_act/` (social media modules)
**Severity**: MEDIUM
**Gap**: NeoTrix has social media posting (NT-ACT) but no claim verification pipeline. 2026 research (S24-S25) shows that misinformation detection requires: RoBERTa+GRU hybrid architectures for contextual+sequential pattern detection, network-level propagation analysis, and source credibility scoring. NeoTrix's NT-SHIELD handles network security but has no content-level truth verification.

**Impact**: When NeoTrix posts to social media or relays information, it cannot verify factual accuracy. The NT-SHIELD domain (影卫) protects against network threats but not information threats.

### DEFECT-G008: No LLM-Graph Integration Pattern
**File**: `neotrix-core/src/unified/core/nt_core_llm.rs` + `nt_core_knowledge_mgmt.rs`
**Severity**: MEDIUM-HIGH
**Gap**: NeoTrix's LLM interaction (nt_core_llm) and knowledge management are separate modules. 2026 research (S26-S28) demonstrates that the frontier is LLM-Graph integration: LLMs as graph encoders/aligners, graph-enhanced LLM reasoning (GraphRAG), and agentic graph reasoning (GraphAgents). NeoTrix has no mechanism for the LLM to directly query the knowledge graph during inference, nor for graph structure to condition LLM generation.

**Impact**: The consciousness core's task processing cannot leverage the structured knowledge in the KB during LLM inference. The E8 reasoning engine produces VSA embeddings but these never feed back into the LLM's reasoning process as structured graph context.

### DEFECT-G009: No Influence Propagation Model
**File**: `neotrix-core/src/unified/core/nt_core_gwt/resonance.rs`
**Severity**: LOW-MEDIUM
**Gap**: The GWT resonance matrix models attention salience but not information/influence propagation across the knowledge graph. IPCwalks (S19) and IntervenSim (S22) show that influence propagation requires: metapath-guided walks, intervention-aware dynamics, and source-crowd co-evolution modeling. NeoTrix's resonance is a static matrix multiplication, not a dynamic propagation model.

**Impact**: The system cannot predict how a knowledge update in one domain will propagate to others, or simulate the effect of attention allocation changes before committing them.

### DEFECT-G010: No Graph Tokenization for Transformer Integration
**File**: `neotrix-core/src/unified/core/nt_core_hcube/` (VSA module)
**Severity**: MEDIUM
**Gap**: NeoTrix's E8 Hexagram and VSA HyperCube use custom symbolic representations that are incompatible with graph transformer architectures. S3 (Generative Sliding-Window Transformer) and S5 (ICML 2026) identify graph tokenization as a core challenge for GFMs. NeoTrix has no mechanism to convert its graph structure into token sequences that can be processed by transformer-based reasoning.

**Impact**: Cannot leverage the emerging class of graph foundation models. The E8 hexagram representation is fixed-size and cannot scale to variable-complexity graphs. Cross-domain transfer (a key GFM property) is impossible with the current symbolic encoding.

---

## 3. Optimization Suggestions

### SUGGESTION-G001: Heterogeneous KB Schema Upgrade
**Priority**: P0
**Action**: Extend `nodes` table with typed schema support:
```sql
ALTER TABLE nodes ADD COLUMN node_type_schema TEXT;  -- reference to meta-graph type
ALTER TABLE nodes ADD COLUMN type_hierarchy TEXT;     -- parent types
ALTER TABLE edges ADD COLUMN valid_from INTEGER;      -- temporal start
ALTER TABLE edges ADD COLUMN valid_to INTEGER;        -- temporal end (NULL = current)
ALTER TABLE edges ADD COLUMN confidence_over_time BLOB;  -- time-series confidence
CREATE TABLE node_types (id TEXT PRIMARY KEY, parent_id TEXT, schema TEXT);
CREATE TABLE relation_types (id TEXT PRIMARY KEY, domain_types TEXT, range_types TEXT);
```
**Reference**: S1 (AnyGraph type-dependent projections), S6 (unified formalism for typed graphs)

### SUGGESTION-G002: Temporal Graph Engine
**Priority**: P0
**Action**: Implement `nt_core_temporal_graph` module with:
- Entity-centric dynamic subgraph construction (DynaGen pattern, S14)
- Temporal consistency loss for ordering (TempReasoner, S11)
- Granger causality tests for temporal edges (TCGAT, S13)
- Online test-time adaptation for evolving graphs (DGOTTA, S15)

**Reference**: S10-S16

### SUGGESTION-G003: Replace Naive Community Detection with Louvain/Leiden
**Priority**: P1
**Action**: Replace `detect_communities()` in `graph_analysis.rs:10-35` with:
- Louvain algorithm (modularity optimization) for fast detection
- Leiden algorithm (improved resolution) for overlapping communities
- Consensus clustering for robustness (cograph pattern, S20)
- Modularity scoring to assess community quality

**Reference**: S17-S20

### SUGGESTION-G004: Graph Self-supervised Pre-training Pipeline
**Priority**: P1
**Action**: Implement `nt_core_graph_pretrain` module:
- Community-aware contrastive learning (S8) for positive/negative pair generation
- GraphSculptor coreset selection (S7) for data-efficient pre-training
- Masked graph autoencoder (GraphMAE pattern) for structural reconstruction
- Meta-path-aware pre-training for heterogeneous graphs (MUG, S9)
- Integration with existing VSA: pre-train VSA embeddings on graph structure

**Reference**: S7-S9

### SUGGESTION-G005: Graph-Structured GWT Attention
**Priority**: P1
**Action**: Modify `workspace.rs` to incorporate:
- PageRank centrality scores for specialist modules
- Betweenness centrality for bridge detection across knowledge domains
- Community-aware attention allocation (prefer cross-community bridges)
- Influence propagation simulation before broadcast commitment

**Reference**: S20 (centrality metrics), S19 (influence propagation), S27 (graph-based agent memory)

### SUGGESTION-G006: Overlapping Community Representation
**Priority**: P2
**Action**: Extend community detection to support:
- Fuzzy community membership (probability distribution over communities)
- Multi-label community assignment per node
- Cross-community bridge strength measurement
- Integration with skill domain mapping for multi-domain skills

**Reference**: S17 (DGAT-OCD overlapping), S20 (cograph multi-cluster)

### SUGGESTION-G007: LLM-Graph Integration Layer
**Priority**: P1
**Action**: Implement `nt_core_graph_llm_bridge` module:
- Graph context retrieval: before LLM inference, retrieve relevant subgraph
- Graph-enhanced prompting: inject graph structure as structured context
- LLM-as-graph-encoder: use LLM to generate node/edge embeddings
- GraphRAG pattern (S27): retrieve subgraph → summarize → inject into prompt
- Bidirectional: LLM output updates graph (new edges from LLM reasoning)

**Reference**: S26 (neural-symbolic taxonomy), S27 (GraphLLM survey), S28 (GraphAgents)

### SUGGESTION-G008: Graph Transformer Tokenization
**Priority**: P2
**Action**: Design graph→token bridge:
- Convert node features + local subgraph → token sequences
- Sliding-window attention over graph tokens (S3)
- Positional encoding for graph structure (random walk positional encoding)
- Enable E8 Hexagram to process variable-size graph inputs via tokenization

**Reference**: S3 (sliding-window transformer), S5 (graph tokenization workshop)

---

## 4. Summary

| Metric | Count |
|--------|-------|
| Sources cited | 28 |
| Defects identified | 10 (3 HIGH, 3 MEDIUM-HIGH, 4 MEDIUM) |
| Optimization suggestions | 8 (2 P0, 4 P1, 2 P2) |

**Critical Path**: DEFECT-G001 (heterogeneous schema) + DEFECT-G002 (temporal edges) + DEFECT-G004 (no graph pre-training) form the core gap. These three defects prevent NeoTrix from leveraging the 2026 graph foundation model paradigm, where typed, temporal, pre-trained graph representations are the new baseline.
