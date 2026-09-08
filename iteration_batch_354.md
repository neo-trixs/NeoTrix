# Iteration Batch 354 — Research Loop

**Date**: 2026-09-06
**Domains**: Knowledge Representation | Semantic Web | Concept Learning

---

## 1. Sources Cited

| # | Source | Date | Topic |
|---|--------|------|-------|
| S1 | KDD 2026 KnowKG Tutorial (bdi-lab.github.io) | Aug 2026 | KG embedding from structural to generative reasoning |
| S2 | "Knowledge Graph Re-engineering Along the Ontological Continuum" (arXiv:2605.22093) | May 2026 | Ontological continuum for KG construction |
| S3 | "Ontology-guided and neighborhood-aware embedding for industrial KG completion" (Data & Knowledge Engineering 164, 2026) | Jun 2026 | Cross-layer grouping + GAT for ontology-aware KGE |
| S4 | "OntoKG: Ontology-Oriented KG Construction with Intrinsic-Relational Routing" (arXiv:2604.02618) | Apr 2026 | Intrinsic vs relational property routing on Wikidata |
| S5 | "Reasoning with Ontology Graph: OntGQA" (ACL 2026) | Sep 2026 | Type-constrained KGQA over relation-centric ontology |
| S6 | "Avoiding unproductive SPARQL queries through optimized indices" (Springer WWW 29:32, 2026) | May 2026 | Interactive SPARQL query formulation with index optimization |
| S7 | "Efficient Cloud-edge Collaborative Approaches to SPARQL Queries" (arXiv:2601.15992) | Jan 2026 | Cloud-edge SPARQL for bandwidth-limited environments |
| S8 | "Expressive Querying and Scalable Management of Large RDF Archives" (Semantic Web 17:3, 2026) | May 2026 | RDF archiving with SPARQL over versioned graphs |
| S9 | "Dynamic update method for intelligence knowledge graphs" (Frontiers of CS 20, 2026) | Jan 2026 | Graph embedding-based dynamic KG update |
| S10 | "Unifying Concept Representation Learning" (ICLR 2026 Workshop) | Apr 2026 | Convergence of NeSy, XAI, CRL for concept quality |
| S11 | "Learning Like Humans: Analogical Concept Learning" (CVPR 2026) | Mar 2026 | ATCG: analogical textual concepts for generalized category discovery |
| S12 | "Emergent Analogical Reasoning in Transformers" (ICML 2026 Spotlight) | Feb 2026 | Mechanistic grounding of analogy in transformer circuits |
| S13 | "On Evaluating Abstraction and Analogy in Humans and Machines" (Melanie Mitchell, Current Directions in Psychological Science, 2026) | Apr 2026 | Evaluation methodology for abstraction/analogy |
| S14 | "Unlocking LLM Creativity in Science through Analogical Reasoning" (arXiv:2605.11258) | May 2026 | Analogical reasoning for scientific discovery |
| S15 | HydraDB: Enterprise Knowledge Graph Statistics 2026 | Aug 2026 | KG market trends, GraphRAG, hybrid retrieval |

---

## 2. Defects Identified in NeoTrix Design

### DEFECT-354-1: KnowledgeRepresentationEngine has no ontology embedding injection

**Location**: `nt_core_knowledge_repr.rs:1-311`
**Evidence**: The `KnowledgeRepresentationEngine` stores ontology classes and properties as plain `Vec` containers with no embedding projection. The `semantic_retrieve` method (line 279-305) uses raw `String::contains` substring matching — not vector similarity.
**Gap vs. research**: S3 (Chen & Dai 2026) demonstrates that ontology-guided embedding with cross-layer grouping + GAT on neighbor context outperforms all baselines on industrial KGs. NeoTrix's ontology layer is structurally disconnected from the VSA HyperCube embedding space.
**Suggestion**: Implement `OntologyGuidedEmbedder` that projects `OntologyClass` nodes into VSA space, then clusters entity embeddings around their ontological concepts (cross-layer grouping). Wire GAT aggregation over neighbor embeddings. This directly bridges the current ontology↔VSA gap.

### DEFECT-354-2: VSA HyperCube lacks multi-relational structure encoding

**Location**: `nt_core_hcube/ghrr_vsa.rs`, `ffi/vsa_hypercube.rs`
**Evidence**: `GhrrHyperCube` implements bundle/bind/permute but has no mechanism for encoding typed multi-relational edges (e.g., `is_a`, `has_property`, `causes`). The `VSAOperation` enum (ffi/types.rs:146) only has `Bind`, `Unbind`, `Bundle`, `Similarity` — no relation-aware operations.
**Gap vs. research**: S1 (KDD 2026 KnowKG) and S4 (OntoKG) show that modern KG embeddings must distinguish intrinsic properties (node attributes) from relational properties (edges). NeoTrix treats all VSA operations as relation-agnostic.
**Suggestion**: Add `RelationalBind(rel_type: RelationTag, ...)` to `VSAOperation` that modulates the phase/permute by relation type. This enables the VSA to encode `(entity, relation, entity)` triples as structured hyper-vectors rather than flat bundles.

### DEFECT-354-3: Reasoner uses naive string matching, no type-constrained inference

**Location**: `nt_core_knowledge_repr.rs:247-276` (`infer` method, `check_preconditions`)
**Evidence**: `check_preconditions` (line 274-276) does `query.contains(p.as_str())` — a pure substring check. No OWL/RDFS reasoning, no type constraints on entities, no subsumption hierarchy traversal.
**Gap vs. research**: S5 (OntGQA, ACL 2026) shows that type-constrained reasoning over ontology graphs dramatically improves KGQA accuracy. S2 (Daga et al. 2026) demonstrates the "ontological continuum" where reasoning depth depends on how explicitly the schema is modeled.
**Suggestion**: Replace `InferenceRule` with an `OntologyReasoner` that traverses the class hierarchy (`parent_classes`) and property domain/range constraints. Implement subsumption checking: if `query.subject` is a subclass of `rule.precondition_class`, the rule fires. Add OWL-like property characteristic reasoning (transitivity, symmetry, inverse).

### DEFECT-354-4: CausalInventor analogies are hardcoded, not learned

**Location**: `causal_inventor.rs:60-113`
**Evidence**: `build_analogies()` returns a fixed `vec!` of 10 `CrossDomainMapping` entries with hardcoded Chinese domain names. No mechanism to discover new analogies from KB data. The `domain_index` is populated from knowledge entries but never used for analogy discovery.
**Gap vs. research**: S11 (CVPR 2026, ATCG) shows that analogical concept learning requires retrieving related memories and drawing structural mappings between them — not using predefined templates. S12 (ICML 2026) demonstrates that transformers can emergently learn relational structure transfer for analogy.
**Suggestion**: Implement `AnalogyDiscovery` that: (1) encodes KB entities/relations into VSA vectors, (2) computes structural similarity between subgraphs across domains using VSA binding/unbinding, (3) proposes new `CrossDomainMapping` entries ranked by structural alignment score. Replace the hardcoded vec with a data-driven pipeline.

### DEFECT-354-4b: No continuous/dynamic KG update mechanism

**Location**: `nt_core_knowledge_repr.rs` (entire file) — all methods are `&self` or mutate in-memory `HashMap`/`Vec` with no persistence or delta update.
**Evidence**: No `update_entity()`, no `merge_facts()`, no versioning. Adding an entity (line 224) just inserts into a HashMap. No temporal versioning of facts.
**Gap vs. research**: S9 (Chen et al. 2026) proposes graph-embedding-based dynamic update for intelligence KGs. S8 (Pelgrin et al. 2026) addresses expressive querying over RDF archives (versioned graphs). NeoTrix's KB has no temporal dimension.
**Suggestion**: Add `FactVersioned { fact: Fact, timestamp: DateTime, valid_until: Option<DateTime> }` and implement `merge_new_facts(facts: Vec<FactVersioned>)` that handles conflicts, deduplication, and temporal validity. This aligns with the SEAL pipeline's evolution model where knowledge accumulates over cycles.

### DEFECT-354-5: No SPARQL/RDF interop layer

**Location**: Entire codebase — grep for `sparql`, `rdf` returns only incidental mentions (keyword lists, test strings). No SPARQL endpoint, no RDF serialization.
**Evidence**: The KB is SQLite-backed with custom schema. No RDF triples, no SPARQL queries, no linked data export.
**Gap vs. research**: S6 (Springer WWW 2026), S7 (arXiv 2026), and S15 (HydraDB 2026) all confirm that SPARQL endpoints and RDF interoperability are standard for enterprise KGs. NeoTrix's KB is a silo.
**Suggestion**: Add `nt_memory_sparql` module with: (1) RDF triple serialization of KB nodes/edges, (2) a minimal SPARQL query translator that converts SPARQL patterns to SQLite queries, (3) linked data export (JSON-LD). This enables external tool interop and GraphRAG integration without replacing the SQLite backend.

### DEFECT-354-6: Concept quality metrics are absent

**Location**: `nt_core_knowledge_repr.rs:173-181` (KRStats) — only counts entities/relations/facts, no quality measures.
**Evidence**: S10 (ICLR 2026 Workshop on Unifying Concept Representation Learning) establishes that high-quality concepts must satisfy criteria from NeSy (compositionality), XAI (interpretability), and CRL (causal invariance). NeoTrix tracks quantity, not quality.
**Gap vs. research**: S13 (Melanie Mitchell 2026) argues that evaluating abstraction requires measuring structural alignment, not just retrieval accuracy.
**Suggestion**: Extend `KRStats` with: `concept_coherence: f64` (intra-class VSA similarity), `concept_discrimination: f64` (inter-class VSA distance), `causal_invariance: f64` (robustness to intervention). These metrics enable the SEAL pipeline to track concept quality over evolution cycles.

---

## 3. Summary

| Category | Count |
|----------|-------|
| Sources cited | 15 |
| Concrete defects found | 7 (DEFECT-354-1 through -6, including -4b) |
| Actionable suggestions | 7 |

**Cross-cutting themes**:
1. **Ontology ↔ Embedding gap**: The ontology layer and VSA HyperCube operate in parallel, never intersecting. 2026 research (S3, S4, S5) unanimously shows ontology-guided embedding outperforms both standalone approaches.
2. **Static vs. Dynamic**: The KB is a snapshot, not a living graph. Dynamic update (S9) and versioned querying (S8) are now baseline expectations.
3. **Analogical reasoning is templated, not learned**: The CausalInventor's hardcoded analogies are a placeholder. CVPR/ICML 2026 work (S11, S12) shows analogical reasoning must be structural and emergent.
4. **No interop layer**: SPARQL/RDF is the lingua franca of enterprise KGs. Without it, NeoTrix KB remains isolated.
