# Iteration Batch 705 — Knowledge Representation / Semantic Web / Knowledge Graph Research

**Date**: 2026-09-06  
**Domain**: Knowledge Representation, Semantic Web, Knowledge Graphs  
**Previous Batch**: 704 (AoS→SoA, SIMD, PGO/LTO, false sharing, Matrix SDK)  

---

## Sources Consulted

### Knowledge Representation
1. **NeurOWL** — Neuro-symbolic framework for incomplete OWL ontology reasoning (arXiv:2607.15776, Jul 2026)
2. **OntoExpand** — SPARQL-based ontology expansion via CONSTRUCT queries (OASIcs SLATE 2026, Jul 2026)
3. **Baobab** — SROIQ→SDD compilation for neuro-symbolic learning (arXiv:2608.17741, Aug 2026)
4. **Semantic Units Framework** — FAIR/CLEAR knowledge infrastructures (Nature Scientific Data, Jun 2026)
5. **OntGQA** — Type-constrained KGQA with ontology graph reasoning (ACL 2026)
6. **LLM for OWL Proofs** — LLM evaluation on OWL proof construction (WWW 2026)
7. **Moose** — EL++→SDD compilation with reasoning-shortcut awareness (arXiv:2608.12961, Aug 2026)

### Semantic Web
8. **SPARQL 1.2 Working Draft** — Triple terms for native reification (W3C, Jun 2026)
9. **RDF 1.2 Interop** — Query rewriting between triple terms and legacy reification (SEMANTiCS 2026)
10. **Generative SPARQL** — GenOp operator for LLM-augmented SPARQL (arXiv:2606.23875)
11. **RENSA** — Metadata-driven federated SPARQL query generation (arXiv:2608.28963, Aug 2026)
12. **INTERACSPARQL** — Interactive SPARQL refinement with NL explanations (ACL Findings 2026)
13. **Nemo v0.10** — Explainable Datalog/RDF rule engine with federated reasoning (ESWC 2026)

### Knowledge Graph
14. **Neo4j Virtual Graph** — Zero-copy graph reasoning on Snowflake/Databricks (May 2026)
15. **Neo4j Knowledge Layer** — Graph Intelligence Platform architecture (Jun 2026)
16. **Neo4j Document Intelligence** — Auto-extraction of knowledge graphs from documents (Jun 2026)
17. **Knowledge Graphs as Accountability Layer** — Traceability for high-stakes AI (Jun 2026)
18. **Karpathy LLM Wiki → Graph** — Scaling personal knowledge bases with graph (Aug 2026)
19. **Neo4j 2026 Changelog** — Cypher improvements, SHOW CURRENT GRAPH TYPE (GitHub)

---

## NEW Defects Found in NeoTrix

### DEFECT-705-01: No Neuro-Symbolic OWL Reasoning Layer
**Severity**: HIGH  
**Domain**: NT-CORE / NT-MEMORY  
**Finding**: NeurOWL (F1=0.970 on real ontologies) and Baobab (SROIQ→SDD compilation) demonstrate that neuro-symbolic reasoning over OWL ontologies is production-viable in 2026. NeoTrix's VSA HyperCube provides only vector embeddings — no logical entailment. When VSA similarity fails (e.g., incomplete ontologies, novel concept combinations), there is no fallback to formal reasoning.  
**Impact**: NeoTrix cannot infer `A ⊑ B` when the ontology is incomplete, which is the common case in real-world knowledge bases. NeurOWL's Stage 2 (logical bridging) + Stage 3 (embedding bridging) architecture could be adapted for HyperCube's concept space.  
**Fix**: Add an optional SDD-based reasoning layer beneath VSA embeddings that handles subsumption queries when embedding similarity falls below confidence threshold.

### DEFECT-705-02: No Ontology Expansion / Inference Generation
**Severity**: HIGH  
**Domain**: NT-MEMORY / NT-MIND  
**Finding**: OntoExpand demonstrates that SPARQL CONSTRUCT queries can expand ontologies by inferring new triples from existing axioms, as an alternative to full OWL reasoners. NeoTrix's KB has FTS5 search but no inference layer — it retrieves what is stored, never what is implied.  
**Impact**: Every knowledge query returns only explicitly asserted facts. A query like "what modules depend on nt_core_self?" requires explicit `depends_on` edges, not inferred transitive closure. This cripples the ability to reason about cross-domain dependencies.  
**Fix**: Implement a CONSTRUCT-equivalent inference pass in the KB pipeline that pre-materializes transitive closures and subsumption inferences on write, with lazy re-evaluation on schema change.

### DEFECT-705-03: No Reasoning Shortcut Detection in Concept Learning
**Severity**: MEDIUM  
**Domain**: NT-MIND / NT-CORE  
**Finding**: Moose (arXiv:2608.12961) formalizes reasoning shortcuts in OWL EL — when partial supervision allows multiple ontology-consistent completions, independent concept predictors collapse onto unintended semantics. NeoTrix's SEAL pipeline uses independent concept extraction without checking for these shortcuts.  
**Impact**: When NeoTrix learns new concepts from partial observations (e.g., during skill crystallization), it may encode incorrect concept relationships that happen to satisfy training data but violate the ontology's intended semantics.  
**Fix**: Add a BEARS-ensemble or justification-anchored mixture step to the SEAL concept crystallization phase, ensuring latent concept assignments are checked against ontology-consistent completions.

### DEFECT-705-04: No FAIR Semantic Modularization
**Severity**: MEDIUM  
**Domain**: NT-MEMORY  
**Finding**: The Semantic Units Framework (Nature Scientific Data, Jun 2026) addresses 12 core limitations of OWL/RDF modeling including negation, cardinality, and complex class axioms through semantic modularization. NeoTrix's KB stores flat triples with no semantic unit boundaries — all knowledge is in one monolithic graph with no modular reasoning.  
**Impact**: Query performance degrades as the KB grows because there is no way to scope reasoning to a relevant semantic unit. Cross-domain queries scan the entire graph. The framework's four resource categories (some-instance, most-instances, every-instance, all-instances) map directly to NeoTrix's domain model but are not implemented.  
**Fix**: Partition the KB into semantic units (one per NT-* domain) with explicit unit boundaries, enabling scoped reasoning within units and controlled cross-unit inference.

### DEFECT-705-05: No LLM Proof Construction / Completeness Verification
**Severity**: MEDIUM  
**Domain**: NT-CORE / NT-SHIELD  
**Finding**: WWW 2026 "LLM for OWL Proofs" shows that LLMs can extract, simplify, and explain OWL proofs, but performance drops 38-47% with incomplete premises or noise. NeoTrix has no mechanism to verify whether its own reasoning chains are logically complete or contain gaps.  
**Impact**: When NeoTrix's consciousness core makes a decision based on KB knowledge, there is no audit trail showing which axioms contributed. Under incomplete knowledge, the system cannot distinguish "I don't know" from "this is false."  
**Fix**: Implement a proof-trace layer that records the axioms used in each reasoning step, with a completeness checker that flags when the premise set is incomplete (insufficient axioms to derive the conclusion).

### DEFECT-705-06: No SPARQL 1.2 Triple Term Support
**Severity**: LOW-MEDIUM  
**Domain**: NT-MEMORY / NT-IO  
**Finding**: SPARQL 1.2 (W3C Working Draft Jun 2026) introduces triple terms for native reification. RDF 1.2 interop (SEMANTiCS 2026) provides query rewriting between triple terms and legacy patterns. NeoTrix's KB uses custom edge metadata but no standardized reification — statement-level metadata (provenance, confidence, timestamps) is ad-hoc.  
**Impact**: When NeoTrix ingests external knowledge from RDF sources, it cannot natively represent statement-level metadata (e.g., "this edge was asserted by model X with confidence Y at time T") without custom schema hacks. The interop layer's CONSTRUCT-based mapping approach could standardize this.  
**Fix**: Adopt triple-term semantics for the KB's edge metadata model. Each edge can carry a reified context node with provenance, confidence, and temporal validity.

### DEFECT-705-07: No Generative Query Operator (GenOp)
**Severity**: MEDIUM  
**Domain**: NT-IO / NT-CORE  
**Finding**: Generative SPARQL (arXiv:2606.23875) extends SPARQL with a GenOp operator that calls LLMs mid-query, producing typed solution mappings. NeoTrix's query path is purely symbolic — LLM calls and KB queries are separate pipelines.  
**Impact**: NeoTrix cannot interleave retrieval with generation in a single query pass. For example, "find all modules with incomplete tests and suggest fixes" requires two separate steps: KB query → LLM generation. GenOp would allow a single composed query with fix generation inline.  
**Fix**: Implement a GenOp-equivalent in the KB query layer that allows LLM calls as first-class query operators, with typed solution mappings and fixpoint semantics for recursive dependencies.

### DEFECT-705-08: No Federated Knowledge Source Selection
**Severity**: LOW-MEDIUM  
**Domain**: NT-WORLD / NT-MEMORY  
**Finding**: RENSA (arXiv:2608.28963) uses rich metadata profiles to eliminate runtime ASK queries for federated SPARQL source selection. Neo4j Virtual Graph enables zero-copy graph reasoning over Snowflake/Databricks. NeoTrix's crawl pipeline has no metadata-driven source selection — it crawls blindly.  
**Impact**: When NeoTrix needs knowledge from multiple sources (papers, repos, docs), it cannot determine which source is most likely to have the answer without actually querying each one. This wastes I/O and time.  
**Fix**: Build a metadata profile per knowledge source (class/authority distributions, coverage statistics) and use it for source selection before crawling.

### DEFECT-705-09: No Interactive Query Refinement / Error Recovery
**Severity**: LOW-MEDIUM  
**Domain**: NT-IO  
**Finding**: INTERACSPARQL (ACL Findings 2026) demonstrates that tool-augmented self-correction loops with NL explanations significantly boost SPARQL generation accuracy without fine-tuning. NeoTrix's LLM query generation has no self-correction loop — failed queries are retried identically.  
**Impact**: When the consciousness core generates a KB query that returns empty results, it cannot diagnose whether the failure is due to wrong entity names, missing data, or logical errors. INTERACSPARQL's AST-based explanation + tool lookup pattern would fix this.  
**Fix**: Add an AST-based query explanation step after query generation, with entity/property lookup tools for self-correction before execution.

### DEFECT-705-10: No Document-to-Knowledge-Graph Pipeline
**Severity**: LOW  
**Domain**: NT-WORLD / NT-MEMORY  
**Finding**: Neo4j Document Intelligence (Jun 2026) automates the path from documents → entity extraction → knowledge graph with no manual schema design. NeoTrix's crawl pipeline extracts text but does not auto-generate knowledge graph structure.  
**Impact**: When NeoTrix ingests papers or docs, it stores raw text and BM25-indexed chunks but does not extract entities/relationships into the KB graph. This forces the LLM to re-derive structure at query time.  
**Fix**: Add an entity extraction + relationship extraction pass to the crawl pipeline that feeds extracted triples directly into the KB, with schema auto-generation from document sampling.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources consulted | 19 |
| NEW defects found | 10 |
| HIGH severity | 2 (DEFECT-705-01, 705-02) |
| MEDIUM severity | 4 (705-03, 705-04, 705-05, 705-07) |
| LOW-MEDIUM severity | 3 (705-06, 705-08, 705-09) |
| LOW severity | 1 (705-10) |

### What's NEW vs Previous Batches
- **Neuro-symbolic reasoning** (NeurOWL/Baobab/Moose): First concrete evidence that SDD compilation from OWL DL is production-viable — NeoTrix's VSA HyperCube has no formal reasoning fallback
- **Ontology expansion** (OntoExpand): SPARQL CONSTRUCT as lightweight inference — NeoTrix KB has zero inference capability
- **Reasoning shortcuts** (Moose): Formal analysis of concept learning failures in OWL EL — NeoTrix SEAL pipeline is vulnerable
- **FAIR semantic units** (Nature): 12 OWL limitations addressed by modularization — NeoTrix KB is monolithic
- **LLM proof construction** (WWW 2026): 38-47% degradation with incomplete premises — NeoTrix has no completeness checking
- **SPARQL 1.2 triple terms** (W3C): Native reification standard — NeoTrix edge metadata is ad-hoc
- **GenOp** (Generative SPARQL): LLM-as-query-operator — NeoTrix separates retrieval and generation
- **Federated source selection** (RENSA): Metadata profiles eliminate runtime probing — NeoTrix crawls blindly
- **Interactive refinement** (INTERACSPARQL): Self-correction loops for query generation — NeoTrix retries identically
- **Document→KG automation** (Neo4j): Auto entity/relationship extraction — NeoTrix stores raw text only

### Cross-Iteration Insight
Batch 704 identified SIMD/AoS cache and PGO/LTO hardware-level defects. Batch 705 reveals a complementary **knowledge-level defect cluster**: NeoTrix's KB is a flat triple store with no inference, no modularization, no proof tracing, and no neuro-symbolic fallback. The 2026 knowledge representation landscape has moved to hybrid neuro-symbolic systems (NeurOWL, Baobab, Moose) that combine formal reasoning with neural flexibility — NeoTrix's VSA HyperCube sits in neither camp fully. The highest-priority fixes are DEFECT-705-01 (neuro-symbolic layer) and DEFECT-705-02 (inference generation), as they unlock the ability to reason beyond what is explicitly stored.
