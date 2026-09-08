# Iteration Batch 401 — Graph Infrastructure Research

**Date**: 2026-09-06
**Focus**: Graph databases, GQL standard, graph analytics, GraphRAG

---

## Sources Cited

### Graph Databases (2026)
1. Neo4j 2026 Changelog — `github.com/neo4j/neo4j/wiki/Neo4j-2026-changelog` (glob-pattern import, transaction deadlock metrics, COSI path optimization)
2. Neo4j Community Edition advances — `neo4j.com/blog/graph-database/community-edition/` (March 2026: vector search in CE, GQL alignment in Cypher 25, Fleet Manager, one-click cloud deploy)
3. Neo4j AI Product Keynote July 2026 — `go.neo4j.com` (enterprise knowledge layer, type optimization dimension, relational atomic read/writes)
4. Graph Databases & Knowledge Graphs 2026 Deep Dive — `youngju.dev` (Composite Database, Vector Index, Parallel Runtime, 10x faster pattern matching in 5.x)
5. Neo4j Trends 2025-2026 — `calmops.com` (GraphRAG, Graph ML, multi-model, serverless Neo4j, federated graph, AI-native graphs)
6. Property Graph Techniques in Relational Databases (DBKDA 2026) — `arxiv.org/abs/2608.11001v1` (unified relational + property graph via REF keys, GQL SQL/PGQ interop)
7. Code Property Graphs + LLMs (SVM'26) — `arxiv.org/pdf/2603.24837` (codebadger: LLMs + CPG for vulnerability detection, 15-40% F1 improvement, backward slicing)
8. Graphs RAG at Scale — `arxiv.org/abs/2603.22340` (LPG + RDF hybrid GraphRAG, dynamic document retrieval without pre-specifying doc count)

### GQL Standard
9. GQL Language Guide — Microsoft Fabric (May 2026, ISO-standardized, same ISO committee as SQL)
10. GQL Statistics 2026 — `hydradb.com/blog/statistics-graph-query-languages` (GQL = first new ISO DB query language since SQL 1987, 610-page spec, 400+ referenced papers)
11. Property Graphs at Scale (CAiSE 2024) — `springer.com` (vision: single declarative graph language, data integration, scalable processing)
12. Optimizing Navigational Graph Queries — `arxiv.org/html/2406.05417v2` (regular queries, novel optimization techniques, orders-of-magnitude improvement)

### Graph Analytics
13. Neo4j GDS Library v2026.07 — `neo4j.com/docs/graph-data-science/current/` (GDS Agent for Advanced Graph Algorithmic Reasoning, NODES 2026)
14. Data Analytics Algorithms in Property Graph Databases (survey) — `sciencedirect.com` (45 algorithms classified and explained)
15. C++ Graph Library Proposal (P3126r4/P3128r4) — `open-std.org` (ISO C++ standard graph library: generic traversal, named requirements, Boost Graph successor)
16. 10 Top Python Graph Libraries 2026 — `falkordb.com` (NetworkX, igraph, graph-tool, rustworkx, RAPIDS cuGraph GPU, DGL GNN, GraphBLAS)
17. Analytics-Augmented Generation (AAG) — `arxiv.org/abs/2602.21604` (intent-driven graph analytics, algorithm-centric interaction)

### Graph Pattern Matching
18. Efficient Graph Matching with Pattern Reduction (ICDE 2026) — `computer.org` (inclusion relationships between matching sets, redundancy reduction)
19. GHL: Extensible Graph Pattern Matching Library (ICGT 2026) — `IBM graph-hook-library` (2.3x/100x/207x faster than iGraph/NetworkX/Graph-tool)
20. Graph Pattern Matching in Large-Scale Networks (Nature Index) — (subgraph isomorphism, graph simulation, continuous matching, vertex-centric distributed matching)
21. HFrame: GNN for Subgraph Homomorphism — `arxiv.org/abs/2507.20226` (first GNN-based framework for subgraph homomorphism)

### GraphRAG
22. MemGraphRAG (May 2026) — `arxiv.org/abs/2606.00610` (memory-based multi-agent system, fixes fragmented/inconsistent graph construction)
23. GraphRAG-R1 (Web Conference 2026) — `arxiv.org/abs/2507.23581` (process-constrained RL for graph retrieval, hybrid graph-textual retrieval)
24. EA-GraphRAG (Feb 2026) — `arxiv.org/abs/2602.03578` (adaptive routing: simple queries→RAG, complex→GraphRAG, latency optimization)
25. Microsoft GraphRAG — `github.com/microsoft/graphrag` (35K+ stars, community detection, local/global search)

### Apache AGE
26. Apache AGE — `age.apache.org` (PostgreSQL graph extension, openCypher + SQL hybrid, PG18 support in progress)
27. Snowflake: Graph Queries in Postgres with Apache AGE — `snowflake.com` (May 2026, no data movement, SQL+Cypher together, one transaction)

---

## Defects Found in NeoTrix Design

### DEFECT-001: KB Uses SQLite — Missing Property Graph Capabilities
**Gap**: NeoTrix KB is SQLite-backed with nodes/edges/embeddings/BM25. The 2026 landscape shows property graph databases (Neo4j 5.x, Apache AGE) offer native graph traversal, composite databases, and vector indexes in a single store. SQLite lacks native graph joins, variable-length path queries, and graph-native indexing.
**Severity**: HIGH
**Evidence**: Neo4j 5.x provides 10x faster pattern matching via native parallel runtime; Apache AGE runs graph queries inside PostgreSQL with zero data movement.
**Suggestion**: Design a dual-mode KB: SQLite for fast local state + optional property graph backend (Apache AGE on PostgreSQL) for graph-native operations. The KB should expose a graph query interface (GQL or openCypher subset) that translates to SQLite adjacency lists or delegates to the property graph backend.

### DEFECT-002: No GQL Standard Alignment
**Gap**: GQL (ISO/IEC 39075) was published April 2024 and is the first new ISO database query language since SQL in 1987. NeoTrix KB query language is ad-hoc. No mention of GQL conformance or even GQL-inspired syntax.
**Severity**: HIGH
**Evidence**: Microsoft Fabric, Neo4j (Cypher 25 aligns with GQL), Oracle 26ai all implement GQL. The GQL spec is 610 pages with 400+ referenced papers.
**Suggestion**: Define a KB query DSL that is GQL-aligned. At minimum, support MATCH/RETURN/WITH/FILTER clauses and pattern matching syntax from GQL. This enables future portability and ecosystem integration.

### DEFECT-003: Missing GraphRAG Integration Pattern
**Gap**: NeoTrix has KB search (BM25 + embeddings) but no graph-aware retrieval pattern. 2026 research shows GraphRAG (hybrid graph-vector retrieval) significantly outperforms plain vector search for multi-hop reasoning.
**Severity**: HIGH
**Evidence**: EA-GraphRAG (Feb 2026) shows adaptive routing between vector RAG and GraphRAG improves both accuracy and latency. MemGraphRAG fixes fragmented graph construction with memory-based multi-agent systems.
**Suggestion**: Design a retrieval adapter that routes queries: simple factoid → BM25/embedding, multi-hop relational → graph traversal. The KB edges should be queryable as a property graph, enabling `MATCH (n)-[:DEPENDS_ON]->(m)` style traversals.

### DEFECT-004: No Graph Algorithm Library
**Gap**: NeoTrix has no built-in graph analytics algorithms. The 2026 landscape includes 45+ classified algorithms in property graph databases (centrality, community detection, path finding, link prediction). C++ is proposing an ISO Graph Library (P3126/P3128).
**Severity**: MEDIUM
**Evidence**: Neo4j GDS v2026.07 adds GDS Agent for algorithmic reasoning. RAPIDS cuGraph provides GPU-accelerated graph analytics. HFrame (GNN) shows ML-augmented graph matching outperforms traditional approaches.
**Suggestion**: Integrate a lightweight graph algorithm layer: at minimum PageRank, community detection (Louvain), shortest path, and triangle counting over the KB graph. The C++ Graph Library proposal (P3128r4) provides a reference architecture for generic traversal patterns.

### DEFECT-005: No Continuous/Incremental Graph Matching
**Gap**: NeoTrix KB is static between sessions. 2026 research shows continuous subgraph matching for dynamic graphs — incremental pattern maintenance, temporal indexing, window-based filtering — is essential for real-time knowledge evolution.
**Severity**: MEDIUM
**Evidence**: Nature Index survey on graph pattern matching (2026) highlights continuous matching techniques that interleave incremental updates with query maintenance. Neo4j 2026 adds glob-pattern support for incremental import.
**Suggestion**: Design an incremental graph update pipeline: when new experience/knowledge is absorbed, update the KB graph incrementally (add nodes/edges, update weights) rather than rebuilding. Support temporal versioning on edges.

### DEFECT-006: Missing Code Property Graph (CPG) for Self-Analysis
**Gap**: NeoTrix analyzes its own codebase but doesn't use Code Property Graphs. CPGs combine AST + control flow + data flow into a single graph, enabling vulnerability detection and semantic code analysis.
**Severity**: MEDIUM
**Evidence**: codebadger (SVM'26) shows LLMs + CPG achieve 15-40% F1 improvement in vulnerability detection. CPG-guided backward slicing reveals root causes invisible to key-value lookups.
**Suggestion**: Build a CPG ingestion pipeline for NeoTrix's own source code. Store as a property graph in KB with edges: AST_PARENT, CFG_EDGE, DFEdge. Enable queries like "find all data flows from user input to unsafe operations."

### DEFECT-007: No Federated/Multi-Graph Query Support
**Gap**: NeoTrix KB is a single graph. Neo4j 5.x Composite Databases allow querying multiple graphs in one query. Apache AGE enables querying multiple named graphs simultaneously.
**Severity**: LOW
**Evidence**: Neo4j Composite Database (2026) combines multiple graphs. Snowflake + Apache AGE (May 2026) enables graph queries across lakehouse tables without data movement.
**Suggestion**: Design KB namespaces as virtual graphs that can be queried independently or composed. A single query should be able to traverse across domain-specific subgraphs (e.g., cross NT-CORE and NT-MEMORY edges).

### DEFECT-008: No GPU-Accelerated Graph Analytics Path
**Gap**: NeoTrix has no GPU path for graph analytics. RAPIDS cuGraph (NVIDIA) and GPU-accelerated graph algorithms show 10-100x speedups for large-scale graph operations.
**Severity**: LOW (for current scale)
**Evidence**: RAPIDS cuGraph + nx-cuGraph (Jun 2026) bridges NetworkX API to GPU. FalkorDB offers GPU-accelerated Redis graph.
**Suggestion**: Design an optional GPU acceleration path for graph analytics. The architecture should allow delegating computationally expensive algorithms (community detection on large graphs, all-pairs shortest paths) to GPU when available.

### DEFECT-009: Missing Graph Schema Evolution/Versioning
**Gap**: NeoTrix KB edges and nodes have no schema evolution strategy. As the knowledge base grows, node types and relationship types need versioned schemas.
**Severity**: LOW
**Evidence**: EvolveGDB (ICGT 2026) addresses model-driven graph schema transformation. GHL (IBM, ICGT 2026) provides graph pattern matching with rewriting for schema evolution.
**Suggestion**: Define a schema registry for KB node/edge types. Support backward-compatible schema evolution: new optional properties, new edge types, deprecated-but-still-queryable types.

### DEFECT-010: No Analytics-Augmented Generation (AAG) Pattern
**Gap**: NeoTrix LLM interactions don't leverage graph analytics results to augment generation. AAG (Feb 2026) shows intent-driven graph analytics can improve LLM reasoning quality.
**Severity**: LOW
**Evidence**: AAG (arXiv:2602.21604) envisions analytics-augmented generation where graph algorithm results (centrality scores, community labels, path analysis) are injected into LLM prompts.
**Suggestion**: Design a pipeline where graph analytics results (PageRank of knowledge nodes, community clusters of related concepts, shortest paths between concepts) are surfaced as context to the LLM during reasoning.

---

## Summary

| Metric | Count |
|--------|-------|
| Sources cited | 27 |
| Defects found | 10 |
| HIGH severity | 3 |
| MEDIUM severity | 3 |
| LOW severity | 4 |

**Priority Actions**:
1. **DEFECT-001 + DEFECT-002**: Design GQL-aligned graph query interface for KB (HIGH)
2. **DEFECT-003**: Design hybrid graph-vector retrieval adapter (HIGH)
3. **DEFECT-004**: Integrate lightweight graph algorithm library (MEDIUM)
4. **DEFECT-005**: Design incremental graph update pipeline (MEDIUM)
