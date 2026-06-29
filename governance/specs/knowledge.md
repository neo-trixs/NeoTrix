# Knowledge Subsystem
> Version 1.0 — Status: ✅ active

## Purpose
The knowledge subsystem manages long-term memory, evidence tracking, and knowledge graph operations. Supports hypergraph relationships (n-ary), spreading activation retrieval, and competitive scoring for provenance-weighted answers.

## Exposed Operations

### Tools
- `add_entry(text: String, tags: Vec<String>) -> EntryId` — Store a new knowledge entry
- `query(embedding: Vec<f32>, top_k: u32) -> Vec<KnowledgeEntry>` — VSA-similarity search
- `add_evidence(entry_id: EntryId, source_url: String, quotation: String) -> EvidenceId` — Attach evidence to an entry
- `competitive_score(query: String) -> ScoringReport` — 6-dimension weighted score
- `add_hyperedge(participants: Vec<String>, relation: String) -> HyperedgeId` — Create n-ary relation

### Resources
- `knowledge://entry/{id}` — Single entry with evidence chain
- `knowledge://graph` — Full knowledge graph (entity, edge counts)
- `knowledge://evidence/stats` — Evidence verification state counts
- `knowledge://hypergraph/traversal/{start}` — BFS/DFS beam search result

## Configuration Schema

```rust
pub struct KnowledgeConfig {
    pub evidence_capacity: usize,         // default 5000
    pub graph_max_nodes: usize,           // default 100_000
    pub spread_decay: f64,                // activation decay per hop, default 0.85
    pub lru_eviction_threshold: usize,    // nodes before LRU eviction, default 1000
}
```

## Dependencies
- **VSA Core** (nt_core_hcube) — Embeddings for similarity search
- **Storage Engine** (nt-segstore) — Durable record persistence
- **Evidence Manager** (nt_core_knowledge) — Competitive scoring

## Error States

| State | Trigger | Recovery |
|-------|---------|----------|
| `GraphCapacityExceeded` | Nodes > max_nodes | LRU eviction, log warning |
| `EvidenceCapacityExceeded` | Evidence > capacity | Prune oldest unverified records |
| `ProvenanceMismatch` | Evidence hash ≠ stored hash | Re-verify from source, mark disputed |
| `IndexCorruption` | IVF centroids misaligned | Rebuild index from raw segments |
