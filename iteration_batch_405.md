# Iteration Batch 405 — External Research → Design Defect Analysis

**Date**: 2026-09-06
**Research Domains**: Knowledge Graph Construction, Ontology Engineering, Semantic Reasoning

---

## Sources Cited

### Knowledge Graph Construction
1. **OntoKG** — Ontology-Oriented Knowledge Graph Construction with Intrinsic-Relational Routing: declarative schema design from outset for ontology analysis, entity disambiguation, domain customization, LLM-guided extraction (arXiv:2604.02618, Apr 2026)
2. **AutoSchemaKG** — Autonomous Knowledge Graph Construction through Dynamic Schema Induction from Web-Scale Corpora: LLM-driven schema induction achieving 92% semantic alignment with human-crafted schemas, 900M+ nodes, 5.9B edges (ACL 2026, Jul 2026)
3. **SciGraph-LLM** — Automatic Knowledge Graph Construction from Scientific Papers: domain-specific extraction pipelines with fine-tuned models for relationship extraction (WSDM 2026, May 2026)
4. **LLM-KGC Survey** — LLM-empowered knowledge graph construction: comprehensive overview of LLMs transforming KG construction across ontology engineering, knowledge extraction, and knowledge fusion (arXiv:2510.20345, Oct 2025)
5. **AOCG-LLM+** — Automatic ontology construction and generation method: LLM-based ontology construction with validation and consistency checking (Complex & Intelligent Systems, Jul 2026)
6. **Multimodal KG** — Construction and refined extraction techniques of knowledge graph based on LLMs: multimodal knowledge integration pipeline combining rule-based systems with ontological structures (Nature Scientific Reports, Feb 2026)
7. **LLMs as External Memory** — Automatic Ontology Construction Using LLMs as an External Layer of Memory, Verification, and Planning: hybrid architecture where LLMs serve as external memory for ontology construction (arXiv:2604.20795, Apr 2026)

### Ontology Engineering
8. **Open Ontologies** — Tool-Augmented Ontology Engineering with Stable Matching Alignment: Rust-based system integrating LLM-driven construction with formal OWL reasoning and ontology alignment via MCP (arXiv:2605.09184, May 2026)
9. **OM-2026** — 21st International Workshop on Ontology Matching: OAEI 2026 campaign with tracks for complex alignments, multilingual matching, Bio-ML, biodiversity, digital humanities, tabular data to KG matching (ISWC 2026, Oct 2026)
10. **Ontology Engineering 2026** — Enterprise AI foundation: OWL/RDF/SPARQL/SHACL stack with Protégé validation, governance, and version control (OvalEdge, Jul 2026)
11. **Complex Ontology Alignment** — Using LLMs for complex alignment with ontology modules: prompt-based approach leveraging internal ontology structure for discoverability (arXiv:2404.10329)
12. **Ontology Alignment SLR** — Systematic literature review of ontology alignment analysis 2021-2024: categorization of alignment techniques, evaluation metrics, and application domains (Wiley, 2025)

### Semantic Reasoning
13. **Neuro-Symbolic AI Agents** — Architectures, integration dimensions: 178-paper survey analyzing neuro-symbolic AI across knowledge representation (44%), learning/inference (63%), logic/reasoning (35%), explainability (28%), meta-cognition (5%) (ScienceDirect, May 2026)
14. **LogicGraph** — Benchmarking Multi-Path Logical Reasoning via Neuro-Symbolic Generation and Verification: first benchmark for multi-path logical reasoning with solver-verified minimal proofs (arXiv:2602.21044, Feb 2026)
15. **CoTu at EXACT 2026** — Neuro-Symbolic Reasoning for Transparent Educational QA: unified neuro-symbolic pipeline with Z3 solver verification, achieving highest technical score 13.44/15 (arXiv:2607.14735, Jul 2026)
16. **NeuroGraph** — AI Graph-Driven Neuro-Symbolic Framework for Explainable Threat Reasoning: graph-driven neuro-symbolic integration for manufacturing threat detection (arXiv:2609.00604, Sep 2026)
17. **Neuro-Symbolic LLM Reasoning** — Towards Improving Reasoning Abilities of Large Language Models: comprehensive review of neuro-symbolic approaches for enhancing LLM reasoning (arXiv:2508.13678, Aug 2025)
18. **Hybrid Neuro-Symbolic IoT** — Predictive cybernetic decision support: edge-deployed neural networks with cloud-based SMT reasoning for constraint certification (Future Generation Computer Systems, 2026)
19. **NeSy Association** — Neurosymbolic AI Conference 2026: premier international conference on neural-symbolic learning and reasoning since 2005 (NeSy 2026, Sep 2026)

---

## Defects Identified

### DEFECT-KG01: No Declarative Schema Management
**Source**: OntoKG (#1), Open Ontologies (#8)
**Evidence**: OntoKG demonstrates that routing every property as either intrinsic (node attribute) or relational (traversable edge) produces a declarative schema that is portable across storage backends and independently reusable for downstream tasks. The schema is designed from the outset for ontology analysis, entity disambiguation, and domain customization. Open Ontologies (Rust-based, #8) integrates LLM-driven construction with formal OWL reasoning via MCP. NeoTrix's KB schema is **hardcoded in SQLite** (`src-tauri/src/domain/plugins/kb.rs:58-88`) with fixed tables (`kb_nodes`, `kb_edges`, `kb_kv`, `kb_docs`). The "kind" field on nodes is a simple string, not a typed property with intrinsic/relational classification. There is no declarative schema that can be ported across storage backends or reused for ontology analysis.
**Severity**: Structural
**Location**: `KB` (NT-MEMORY), `src-tauri/src/domain/plugins/kb.rs:58-88`
**Gap**: The KB cannot evolve its schema dynamically, cannot port to alternative storage backends (e.g., Neo4j, Amazon Neptune, in-memory graphs), and cannot support ontology-level tasks that require intrinsic vs relational property classification. The fixed schema prevents the system from benefiting from OntoKG's declarative approach that makes structural decisions explicit and inspectable.
**Suggestion**: Implement `DeclarativeSchemaLayer`: (1) Define a schema DSL that classifies properties as intrinsic or relational (per OntoKG's intrinsic-relational routing), (2) Generate storage-specific schemas from the declarative definition (SQLite, Neo4j, RDF/OWL), (3) Add schema versioning with migration support, (4) Expose schema introspection API for downstream ontology analysis tasks. This enables the KB to be "portable across storage backends and independently reusable" as demonstrated by OntoKG.

### DEFECT-KG02: No Autonomous Schema Induction
**Source**: AutoSchemaKG (#2), LLM-KGC Survey (#4)
**Evidence**: AutoSchemaKG processes 50M+ documents to induce schemas with 92% semantic alignment to human-crafted schemas, simultaneously extracting knowledge triples and inducing comprehensive schemas directly from text. It models both entities and events, employing conceptualization to organize instances into semantic categories. NeoTrix's KB has no schema induction capability. The "kind" field on nodes is **manually assigned** during knowledge ingestion (e.g., "concept", "entity", "event"). There is no LLM-driven schema induction from content, no conceptualization of instances into categories, and no automatic schema refinement based on extracted knowledge patterns.
**Severity**: Behavioral
**Location**: `KB` (NT-MEMORY), `nt_mind::knowledge_engine`, `nt_world::asset_registry`
**Gap**: When NeoTrix ingests new knowledge (via `doc_ingest` or crawling), it cannot automatically discover new entity types, relation types, or schema patterns. Human operators must manually define the schema. AutoSchemaKG demonstrates that LLM-driven schema induction can achieve near-human quality with zero manual intervention, making KB construction autonomous at scale.
**Suggestion**: Add `SchemaInductionEngine` to NT-MIND: (1) Analyze ingested documents to discover entity types and relation patterns (per AutoSchemaKG's LLM-driven extraction), (2) Cluster discovered patterns into semantic categories via conceptualization, (3) Propose schema refinements (new node types, edge types, constraints) with confidence scores, (4) Human-in-the-loop approval for schema changes above confidence threshold, (5) Schema evolution tracking with versioning and rollback. Wire to SEAL pipeline for schema refinement absorption cycles.

### DEFECT-KG03: No Provenance-Aware Knowledge Extraction
**Source**: Multimodal KG (#6), LLMs as External Memory (#7)
**Evidence**: The multimodal KG framework (#6) incorporates a pipeline that combines rule-based systems with ontological structures to extract and link entities from diverse data sources, creating an adaptive knowledge network with provenance tracking. LLMs as External Memory (#7) uses LLMs for verification and planning in ontology construction. NeoTrix's KB stores extracted knowledge but **lacks provenance metadata**. The `kb_nodes` table has `created_at` but no source attribution, extraction confidence, or verification status. The `doc_ingest` function stores raw text but doesn't track which LLM or extraction method produced the knowledge.
**Severity**: Behavioral
**Location**: `KB` (NT-MEMORY), `src-tauri/src/domain/plugins/kb.rs:227-238`
**Gap**: When knowledge is extracted from external sources, the KB cannot trace back to the original source, extraction method, confidence score, or verification status. This prevents quality assessment, conflict resolution (when two sources disagree), and trust-based routing. The 2026 literature emphasizes that "knowledge graphs fail on alignment drift, weak extraction, stale entities, unclear provenance" (#4 survey).
**Suggestion**: Extend KB schema with provenance fields: (1) Add `source_url`, `source_type`, `extraction_method`, `extraction_confidence`, `verification_status` to `kb_nodes`, (2) Implement provenance-aware query that filters by confidence threshold, (3) Add temporal decay for unverified knowledge (staleness tracking), (4) Support conflict resolution when multiple sources provide contradictory information for the same entity, (5) Wire to GWT attention routing so high-confidence knowledge receives higher salience.

### DEFECT-ONT01: No Formal Ontology Representation (OWL/RDF)
**Source**: Open Ontologies (#8), Ontology Engineering 2026 (#10), Complex Ontology Alignment (#11)
**Evidence**: Open Ontologies (#8) is a Rust-based system that integrates LLM-driven construction with formal OWL reasoning and ontology alignment via MCP. The 2026 ontology engineering stack includes OWL/RDF for knowledge representation, SPARQL for querying, SHACL for validation, and Protégé for visual editing. Complex Ontology Alignment (#11) uses ontology modules to improve discoverability for hard-to-automate alignment tasks. NeoTrix's knowledge representation is **VSA HyperCube** (binary vectors with bind/bundle/permute) — a symbolic architecture but not a formal ontology language. There is no OWL/RDF representation, no Description Logic reasoning, no class hierarchies, no property restrictions, and no formal semantics.
**Severity**: Structural
**Location**: `VSA HyperCube` (NT-CORE), `KB` (NT-MEMORY), `nt_core_knowledge`
**Gap**: NeoTrix cannot interoperate with the Semantic Web ecosystem. It cannot import/export OWL ontologies, cannot run SPARQL queries, cannot validate data against SHACL shapes, and cannot reason over class hierarchies with formal semantics. The VSA HyperCube provides associative recall and analogical reasoning but lacks the formal logical foundation that ontology engineering requires. When NeoTrix needs to align its internal knowledge with external ontologies (Wikidata, domain ontologies), it has no formal representation to align against.
**Suggestion**: Add `OntologyLayer` to NT-MEMORY: (1) OWL/RDF serialization for KB nodes and edges (mapping intrinsic properties to data properties, relational properties to object properties), (2) SPARQL query interface for structured knowledge queries, (3) SHACL validation for knowledge quality constraints, (4) Description Logic reasoning engine (via existing Rust OWL libraries like `owl-rs` or FFI to HermiT/Pellet), (5) Bidirectional sync between VSA HyperCube (associative) and OWL (logical) representations. Wire to Open Ontologies MCP server for external ontology alignment.

### DEFECT-ONT02: No Ontology Alignment Mechanism
**Source**: OAEI 2026 (#9), Ontology Alignment SLR (#12)
**Evidence**: OAEI 2026 (#9) is the premier evaluation initiative for ontology matching, with tracks for complex correspondences, multilingual matching, Bio-ML, biodiversity, digital humanities, and tabular data to KG matching. The 21st International Workshop on Ontology Matching emphasizes that "ontology matching is a key interoperability enabler for the Semantic Web" and "enables knowledge and data expressed with matched ontologies to interoperate." Ontology Alignment SLR (#12) categorizes alignment techniques across string-based, structure-based, instance-based, and neural approaches. NeoTrix has **no ontology alignment capability**. When encountering external knowledge graphs (Wikidata, DBpedia, domain ontologies), it cannot establish correspondences between its internal concepts and external entities.
**Severity**: Gap
**Location**: `KB` (NT-MEMORY), `nt_world` (perception), `nt_io` (interfaces)
**Gap**: NeoTrix operates as an isolated knowledge silo. It cannot discover that its internal "module" concept corresponds to Wikidata's "software module" entity, or that its "capability" aligns with a domain ontology's "function" class. This prevents cross-ontology query answering, knowledge fusion from multiple sources, and participation in linked data ecosystems. The OAEI 2026 tracks demonstrate that modern systems must handle complex alignments (not just 1:1 mappings), multilingual alignment, and alignment with incomplete ontologies.
**Suggestion**: Implement `OntologyAlignmentModule` in NT-MEMORY: (1) String-based alignment (edit distance, n-gram similarity) for lexical matching, (2) Structure-based alignment using VSA HyperCube similarity for semantic matching, (3) Instance-based alignment using KB node embeddings, (4) LLM-based complex alignment (per #11's prompt-based approach) for deep semantic correspondences, (5) Alignment storage with confidence scores and provenance, (6) Continuous alignment refinement via OAEI-style evaluation. Wire to NT-WORLD for external ontology discovery and NT-IO for Semantic Web integration.

### DEFECT-NSR01: No Neuro-Symbolic Reasoning Integration
**Source**: Neuro-Symbolic AI Agents Survey (#13), Neuro-Symbolic LLM Reasoning (#17)
**Evidence**: The 178-paper survey (#13) establishes neuro-symbolic AI as a transformative paradigm synergizing neural networks' pattern recognition with symbolic reasoning's logical structure. Integration dimensions include knowledge representation (44%), learning/inference (63%), logic/reasoning (35%), explainability (28%), and meta-cognition (5%). Neuro-Symbolic LLM Reasoning (#17) reviews approaches for enhancing LLM reasoning with neuro-symbolic techniques. NeoTrix has **neural components** (LLM integration via `nt_io`, embeddings via VSA HyperCube) and **symbolic components** (E8 hexagram reasoning, rule-based gates) but they operate independently. There is no formal neuro-symbolic integration layer where neural pattern recognition feeds into symbolic reasoning or symbolic constraints guide neural generation.
**Severity**: Structural
**Location**: `E8 Hexagram` (NT-CORE), `VSA HyperCube` (NT-CORE), LLM integration (NT-IO), `nt_core_reasoning`
**Gap**: The system cannot jointly optimize neural and symbolic components. When LLM generates code (neural) and E8 reasoning validates it (symbolic), there is no gradient flow or bidirectional information exchange. The survey identifies that "neural methods excel at pattern recognition but lack interpretability, struggle with systematic reasoning" while "symbolic approaches offer explicit reasoning but are brittle in uncertain environments." NeoTrix's architecture has these components but not their synergy.
**Suggestion**: Implement `NeuroSymbolicBridge` in NT-CORE: (1) Neural→Symbolic: LLM outputs are parsed into symbolic representations (first-order logic, constraints) for verification, (2) Symbolic→Neural: Symbolic constraints guide LLM generation via constrained decoding or prompt engineering, (3) Joint optimization: Train neural components with symbolic loss terms (per #13's integration dimensions), (4) Meta-cognitive oversight: Use symbolic reasoning to monitor neural confidence and trigger fallback when uncertainty is high. Wire to E8 reasoning for symbolic verification and LLM integration for neural generation.

### DEFECT-NSR02: No Explainable Reasoning Chains
**Source**: CoTu at EXACT 2026 (#15), NeuroGraph (#16)
**Evidence**: CoTu (#15) demonstrates transparent reasoning via Z3 solver with verifiable step-by-step rationale, achieving highest technical score (13.44/15) in EXACT 2026. Every answer is accompanied by a "verifiable, step-by-step rationale" in first-order logic that can be checked with automated theorem provers. NeuroGraph (#16) provides explainable threat reasoning in manufacturing via graph-driven neuro-symbolic integration. NeoTrix's E8 hexagram reasoning produces states and a `reasoning_chain` vector (`types.rs:106`) but this chain is **not formal or verifiable**. It's a sequence of state transitions, not a logical proof with premises, inference steps, and conclusions that can be independently verified.
**Severity**: Behavioral
**Location**: `E8 Hexagram` (NT-CORE), `ffi/e8_reasoning.rs`, `nt_core_reasoning`
**Gap**: When NeoTrix makes a decision (route attention, activate capability, self-heal a module), the reasoning is opaque. Users cannot verify that the decision follows logically from the premises. The system cannot prove its reasoning is correct, only that it produced a state transition. This contrasts with CoTu's approach where every answer has a formal proof that can be checked by Z3.
**Suggestion**: Add `FormalReasoningLayer` to E8: (1) Translate E8 state transitions into first-order logic propositions, (2) Generate proof obligations for each reasoning step, (3) Verify proofs using an SMT solver (Z3 via FFI), (4) Produce human-readable reasoning traces with premises, inference rules, and conclusions, (5) Store reasoning chains in KB for audit trail and learning from past reasoning patterns. Wire to NT-META for reasoning quality monitoring.

### DEFECT-NSR03: No Multi-Path Reasoning Exploration
**Source**: LogicGraph (#14), Neuro-Symbolic AI Agents Survey (#13)
**Evidence**: LogicGraph (#14) is the first benchmark for multi-path logical reasoning, demonstrating that "many real-world reasoning problems admit multiple valid derivations, requiring models to explore diverse logical paths rather than committing to one route." Each instance is associated with an "exhaustive set of minimal proofs." The neuro-symbolic survey (#13) identifies meta-cognition as crucial but appearing in only 5% of papers. NeoTrix's E8 hexagram reasoning follows a **single path per state transition** — the hexagram selection is deterministic given the current state and input, with no exploration of alternative reasoning paths.
**Severity**: Behavioral
**Location**: `E8 Hexagram` (NT-CORE), `ffi/e8_reasoning.rs`, `nt_core_ttc` (test-time compute)
**Gap**: When E8 reasoning encounters a problem with multiple valid solutions, it commits to the first path discovered and does not explore alternatives. This can lead to suboptimal decisions when the first path is not the best. LogicGraph shows that multi-path exploration with proof diversity improves reasoning robustness. The `nt_core_ttc` module (test-time compute) allocates compute budget but doesn't allocate it across reasoning paths.
**Suggestion**: Implement `MultiPathExplorer` in E8 reasoning: (1) When entering a reasoning state, generate K alternative next states (beam search), (2) For each alternative, compute proof completeness and soundness scores, (3) Use test-time compute budget to explore promising paths in parallel, (4) Select final path based on proof quality + resource cost tradeoff, (5) Store all explored paths and their scores in KB for learning from reasoning diversity. Wire to `nt_core_ttc` for compute allocation across paths.

### DEFECT-ONT03: No Description Logic Reasoning Engine
**Source**: Ontology Engineering 2026 (#10), Complex Ontology Alignment (#11), Neuro-Symbolic AI Agents Survey (#13)
**Evidence**: The 2026 ontology engineering stack (#10) includes OWL with formal Description Logic semantics, enabling reasoning over class hierarchies, property restrictions, and consistency checking. Complex Ontology Alignment (#11) uses ontology modules that leverage DL expressivity for discoverability. The neuro-symbolic survey (#13) identifies "logic and reasoning" as a key integration dimension (35% of papers). NeoTrix has **no Description Logic reasoning engine**. The E8 hexagram is a state machine, not a DL reasoner. There is no subsumption reasoning (is A a subclass of B?), no consistency checking (does the knowledge base contain contradictions?), and no classification (what are the implicit class memberships?).
**Severity**: Structural
**Location**: `KB` (NT-MEMORY), `E8 Hexagram` (NT-CORE), `nt_core_knowledge`
**Gap**: When NeoTrix's KB contains hierarchical knowledge (e.g., "NT-CORE is a domain", "E8 is a reasoning engine"), it cannot reason over the hierarchy. It cannot answer "what are all reasoning engines?" (subsumption), detect "is this knowledge consistent?" (consistency), or infer "what implicit capabilities exist?" (classification). This limits the KB's expressive power compared to OWL-based systems.
**Suggestion**: Add `DLReasoningEngine` to NT-MEMORY: (1) Parse KB schema and instances into OWL DL axioms, (2) Use tableaux algorithm or FFI to existing DL reasoner for subsumption, consistency, and classification, (3) Cache inferred knowledge (implicit relationships) for fast query answering, (4) Detect and report inconsistencies in the KB, (5) Wire to SEAL pipeline so DL reasoning validates knowledge absorption. This brings NeoTrix's KB to OWL DL expressivity level.

---

## Summary

| ID | Domain | Severity | Core Issue |
|---|---|---|---|
| DEFECT-KG01 | Knowledge Graph | Structural | No declarative schema management (intrinsic/relational routing) |
| DEFECT-KG02 | Knowledge Graph | Behavioral | No autonomous schema induction from content |
| DEFECT-KG03 | Knowledge Graph | Behavioral | No provenance-aware knowledge extraction |
| DEFECT-ONT01 | Ontology | Structural | No formal OWL/RDF ontology representation |
| DEFECT-ONT02 | Ontology | Gap | No ontology alignment mechanism for interoperability |
| DEFECT-ONT03 | Ontology | Structural | No Description Logic reasoning engine |
| DEFECT-NSR01 | Semantic Reasoning | Structural | No neuro-symbolic reasoning integration |
| DEFECT-NSR02 | Semantic Reasoning | Behavioral | No explainable, verifiable reasoning chains |
| DEFECT-NSR03 | Semantic Reasoning | Behavioral | No multi-path reasoning exploration |

**Total defects**: 9 (4 Structural, 4 Behavioral, 1 Gap)
**Priority ranking**: KG01 > ONT01 > NSR01 > ONT03 > KG02 > NSR02 > NSR03 > KG03 > ONT02

---

## Design Optimization Suggestions

### Short-term (Next Sprint)
1. **Provenance Metadata** (KG03): Add `source_url`, `extraction_confidence`, `verification_status` to KB schema — low effort, high value for knowledge quality
2. **Explainable Traces** (NSR02): Formalize E8 reasoning chains as verifiable proofs — medium effort, critical for trust

### Medium-term (Next Quarter)
3. **Declarative Schema** (KG01): Implement schema DSL with intrinsic/relational classification — enables backend portability
4. **Neuro-Symbolic Bridge** (NSR01): Integrate LLM outputs with E8 symbolic verification — core architectural improvement
5. **Schema Induction** (KG02): LLM-driven schema discovery from ingested content — enables autonomous KB growth

### Long-term (Next Half)
6. **OWL/RDF Layer** (ONT01): Full Description Logic representation with reasoning — Semantic Web interoperability
7. **Ontology Alignment** (ONT02): Cross-ontology matching and knowledge fusion — ecosystem participation
8. **Multi-Path Explorer** (NSR03): Beam search over reasoning alternatives — reasoning quality improvement
9. **DL Reasoning Engine** (ONT03): Subsumption, consistency, classification — formal knowledge reasoning
