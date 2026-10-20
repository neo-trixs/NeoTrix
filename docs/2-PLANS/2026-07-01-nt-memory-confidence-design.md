# Epistemic Confidence Layer for nt_memory_kb

**Blind Spot**: No structured confidence measurement for KB facts
**Sources**: Kappa Graph κ(G) (epistemic grounding), TCM triple confidence measurement (Wu et al. 2024), ADR-044 probabilistic truth convergence
**Target**: nt_memory_kb (L3 Memory — KnowledgeNode has only a single flat `confidence: f64`)
**Priority**: P1
**Estimated Effort**: 8 days (5 phases)

---

## 1. Problem Statement

The current `KnowledgeNode` struct (`nt_memory_types.rs:15`) stores a single flat `confidence: f64` field. This is insufficient for epistemic integrity:

- **No decomposition**: A confidence of 0.85 could mean "85% source reliability" or "85% consensus agreement" or "8.5 sources". These are fundamentally different epistemic states.
- **No decay**: Facts become stale. Without time-aware decay, an unverified 2023 fact has the same confidence as a verified 2026 fact.
- **No contradiction preservation**: When sources disagree, the system picks a winner by overwriting. Contradictory evidence is lost.
- **No consensus detection**: 10 independent sources saying the same thing = same confidence as 1 source saying it once.
- **No retrieval strategy**: Search returns the same results regardless of whether the task is "critical operation" (needs high confidence) or "exploration" (welcome low-confidence leads).

Kappa Graph (κ(G)) demonstrated that a continuous grounding score (-1.0 to +1.0) combined with a confidence score (0.0 to 1.0, modeled as hyperbolic saturation) provides an **honest, decomposed** picture of what a knowledge graph actually knows — and doesn't know.

---

## 2. Architecture

### 2.1 Four-Component Epistemic Model

```
EpistemicConfidence
├── source_confidence    (0.0 - 1.0)  Source reliability
│     Based on: source provenance, source domain authority,
│     cross-referencing count, historical accuracy.
│
├── grounding_confidence (0.0 - 1.0)  Evidence support strength
│     Based on: number of supporting vs contradicting sources,
│     evidence triangulation, Michaelis-Menten saturation.
│     Range: 0.0 = lone witness = unknown
│            0.5 = 2 sources = moderate
│            1.0 = 10+ diverse sources = well-grounded
│
├── consensus_confidence (0.0 - 1.0)  Independent agreement
│     Based on: number of independent source clusters,
│     authenticated diversity (semantic diversity × grounding sign).
│     Low = echo chamber (same source family agreeing with itself).
│
└── recency_confidence   (0.0 - 1.0)  Temporal freshness
      Based on: time since last reconfirmation,
      exponential decay with λ per fact type.
      1.0 = confirmed today, 0.1 = below threshold = auto-archive.
```

### 2.2 Key Data Structures

```rust
/// Structured epistemic confidence for a fact or node.
/// Four orthogonal dimensions that compose into an aggregate score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicConfidence {
    /// Source reliability (0.0 - 1.0).
    pub source_confidence: f64,
    /// Evidence grounding via saturation (0.0 - 1.0).
    pub grounding_confidence: f64,
    /// Independent source agreement (0.0 - 1.0).
    pub consensus_confidence: f64,
    /// Temporal freshness with decay (0.0 - 1.0).
    pub recency_confidence: f64,
    /// When this confidence was last computed (Unix timestamp).
    pub computed_at: i64,
    /// When the source was last confirmed (Unix timestamp).
    pub last_confirmed_at: i64,
}

impl EpistemicConfidence {
    /// Aggregate via weighted harmonic mean.
    /// Weights: source=0.30, grounding=0.30, consensus=0.25, recency=0.15
    pub fn aggregate(&self) -> f64 {
        const W1: f64 = 0.30;
        const W2: f64 = 0.30;
        const W3: f64 = 0.25;
        const W4: f64 = 0.15;
        let numerator = W1 + W2 + W3 + W4;
        let denominator = W1 / (self.source_confidence + 1e-10)
            + W2 / (self.grounding_confidence + 1e-10)
            + W3 / (self.consensus_confidence + 1e-10)
            + W4 / (self.recency_confidence + 1e-10);
        numerator / denominator
    }

    /// Apply time decay: recency *= exp(-lambda * elapsed_days)
    /// lambda = 0.01 for general facts, 0.05 for time-sensitive.
    /// Recomputes recency_confidence and computed_at.
    pub fn decay(&mut self, elapsed_days: f64, lambda: f64) {
        let decay_factor = (-lambda * elapsed_days).exp();
        self.recency_confidence = self.recency_confidence * decay_factor;
        self.recency_confidence = self.recency_confidence.max(0.0).min(1.0);
        self.computed_at = chrono::Utc::now().timestamp();
    }

    /// On reconfirmation: move all components toward 1.0.
    /// strength = 0.0 (no effect) to 1.0 (full reconfirm).
    /// recency jumps to 1.0 always.
    pub fn reconfirm(&mut self, strength: f64) {
        let s = strength.max(0.0).min(1.0);
        self.source_confidence = self.source_confidence + (1.0 - self.source_confidence) * s * 0.3;
        self.grounding_confidence = self.grounding_confidence + (1.0 - self.grounding_confidence) * s * 0.2;
        self.consensus_confidence = self.consensus_confidence + (1.0 - self.consensus_confidence) * s * 0.1;
        self.recency_confidence = 1.0;
        self.last_confirmed_at = chrono::Utc::now().timestamp();
        self.computed_at = chrono::Utc::now().timestamp();
    }

    /// Create a zero-confidence (epistemic null) state.
    pub fn unknown() -> Self {
        Self {
            source_confidence: 0.0,
            grounding_confidence: 0.0,
            consensus_confidence: 0.0,
            recency_confidence: 0.5, // neutral — no decay info available
            computed_at: chrono::Utc::now().timestamp(),
            last_confirmed_at: 0,
        }
    }

    /// Check if this confidence is above the minimum threshold.
    pub fn is_above_threshold(&self, min_aggregate: f64) -> bool {
        self.aggregate() >= min_aggregate
    }
}

/// A search result with full epistemic provenance.
pub struct UncertainResult {
    pub node: KnowledgeNode,
    pub confidence: EpistemicConfidence,
    pub contradictions: Vec<ContradictingFact>,
}

/// A fact that contradicts the primary result.
pub struct ContradictingFact {
    pub source_node_id: String,
    pub claim: String,
    pub relation: RelationType,  // usually Contradicts
    pub confidence: EpistemicConfidence,
    pub source_provenance: String,
}

/// Retrieval strategy controlling confidence filtering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RetrievalStrategy {
    /// Only return results above a confidence threshold.
    /// For critical operations: min_confidence >= 0.7
    Conservative { min_confidence: f64 },
    /// Rank by confidence descending, no filter.
    /// Default strategy.
    Balanced,
    /// Include low-confidence + contradictions.
    /// For research/exploration tasks.
    Exploratory,
    /// Weight the query embedding similarity by confidence.
    /// confidence_weighted_score = embedding_score * confidence.aggregate()
    ConfidenceWeighted { confidence_weight: f64 },
}
```

### 2.3 Confidence Scale Semantics

| Aggregate Score | Label | Meaning | Action |
|----------------|-------|---------|--------|
| 0.90 - 1.00 | Established | Multiple diverse sources, recently confirmed | Trust fully |
| 0.70 - 0.89 | Well-supported | 3+ sources, moderate diversity | Use with confidence |
| 0.50 - 0.69 | Moderate | 2 sources OR 1 authoritative source | Use, flag for reconfirm |
| 0.30 - 0.49 | Tentative | 1 source, no contradiction | Use with caution |
| 0.10 - 0.29 | Weak | Single source, stale | Do not use for decisions |
| 0.00 - 0.09 | Unknown | No evidence or auto-archived | Treat as nonexistent |

---

## 3. Core Algorithms

### 3.1 Grounding via Michaelis-Menten Saturation

Adopted from Kappa Graph's epistemic model: **hyperbolic (Michaelis-Menten) saturation**, not sigmoid.

**Why hyperbolic, not sigmoid?**
- Sigmoid implies the first evidence is weakest (slow start). This is wrong for knowledge: the **first source** is the most important (going from 0 to 1 sources = infinite ratio improvement).
- Hyperbolic starts steep: `f(x) = x / (x + k)`. Derivative is highest at x=0.
- Correctly models diminishing returns: source #2 adds less than source #1.

```rust
/// Compute grounding confidence using Michaelis-Menten saturation.
/// composite = weighted sum of evidence signals.
/// k = half-saturation constant (default 2.0).
fn grounding_saturation(composite: f64, k: f64) -> f64 {
    composite / (composite + k)
}

fn compute_grounding(
    source_count: usize,
    supporting_count: usize,
    contradicting_count: usize,
    relationship_types: usize,
) -> f64 {
    // Contribution scales:
    // - sources: count / 5.0  (5 sources = 1.0 contribution)
    // - support ratio: (support - contradict) / max(1, total)  normalized to 0-1
    // - relationship diversity: unique types / total possible (capped at 1.0)
    let support_ratio = if source_count > 0 {
        (supporting_count as f64 - contradicting_count as f64)
            / source_count.max(1) as f64
    } else {
        0.0
    };
    let support_contribution = support_ratio.max(0.0).min(1.0);

    let source_contribution = (source_count as f64 / 5.0).min(1.0);
    let diversity_contribution = (relationship_types as f64 / 10.0).min(1.0);

    let composite = source_contribution * 0.4
        + support_contribution * 0.4
        + diversity_contribution * 0.2;

    // k=2.0: composite of 2.0 → 50% confidence
    grounding_saturation(composite, 2.0)
}
```

### 3.2 Authenticated Diversity (Kappa Graph ADR-044)

Raw diversity (number of sources) can be gamed by echo chambers. **Authenticated diversity** multiplies diversity by saturated grounding sign.

```rust
/// Compute authenticated diversity: guards against echo chambers.
/// A fact backed by 10 sources from the same document family has
/// low authenticated diversity, even though source_count is high.
fn authenticated_diversity(
    grounding: f64,        // raw grounding score (-1.0 to 1.0)
    source_entities: &[SourceEntity],  // source entities with domain/type info
) -> f64 {
    // Semantic diversity: unique domains / total sources
    let mut domains: HashSet<&str> = HashSet::new();
    for src in source_entities {
        if let Some(ref domain) = src.domain {
            domains.insert(domain.as_str());
        }
    }
    let diversity = if source_entities.is_empty() {
        0.0
    } else {
        domains.len() as f64 / source_entities.len() as f64
    };

    // Saturated grounding to prevent noise amplification
    let k = 0.3;
    let saturated_grounding = grounding / (grounding.abs() + k);

    saturated_grounding * diversity
}
```

### 3.3 Consensus Detection

```rust
/// Detect whether multiple sources agree or contradict on a fact cluster.
/// Returns consensus score and any contradictions found.
fn detect_consensus(
    conn: &Connection,
    primary_node_id: &str,
) -> Result<ConsensusInfo, String> {
    // Find all edges from primary_node_id with type Supports or Contradicts
    let edges = nt_memory_store::get_edges_for_node(conn, primary_node_id)?;

    let mut support_count = 0usize;
    let mut contradict_count = 0usize;
    let mut supporting_sources: Vec<String> = Vec::new();
    let mut contradicting_sources: Vec<String> = Vec::new();

    for edge in &edges {
        match edge.relation_type {
            RelationType::Supports => {
                support_count += 1;
                supporting_sources.push(edge.target_id.clone());
            }
            RelationType::Contradicts => {
                contradict_count += 1;
                contradicting_sources.push(edge.target_id.clone());
            }
            _ => {}
        }
    }

    let total = (support_count + contradict_count) as f64;
    let consensus = if total == 0.0 {
        0.5  // neutral: no evidence either way
    } else {
        (support_count as f64 - contradict_count as f64) / total
    };
    // Normalize to 0.0-1.0: consensus=-1.0 → 0.0, consensus=0.0 → 0.5, consensus=1.0 → 1.0
    let normalized_consensus = (consensus + 1.0) / 2.0;

    let contradictions: Vec<ContradictingFact> = contradicting_sources
        .iter()
        .filter_map(|source_id| {
            let source_node = nt_memory_store::get_node(conn, source_id).ok()??;
            Some(ContradictingFact {
                source_node_id: source_id.clone(),
                claim: source_node.summary.clone().unwrap_or_default(),
                relation: RelationType::Contradicts,
                confidence: EpistemicConfidence::unknown(),
                source_provenance: source_node.url.clone().unwrap_or_default(),
            })
        })
        .collect();

    Ok(ConsensusInfo {
        support_count,
        contradict_count,
        consensus_score: normalized_consensus,
        contradictions,
    })
}
```

### 3.4 Decay Functions

```rust
/// Decay lambda by fact type.
fn decay_lambda_for_fact_type(fact_type: &str) -> f64 {
    match fact_type {
        "permanent" => 0.001,    // e.g., "Einstein discovered relativity"
        "dynamic" => 0.05,       // e.g., "CEO of Company X"
        "time_sensitive" => 0.1, // e.g., "Stock price of Y"
        "trend" => 0.03,         // e.g., "LLM benchmark state-of-art"
        "source" => 0.005,       // metadata about sources
        _ => 0.01,               // default general knowledge
    }
}

/// Auto-archive: move node to archive when recency_confidence < 0.1.
fn should_auto_archive(confidence: &EpistemicConfidence) -> bool {
    confidence.recency_confidence < 0.1
    && confidence.last_confirmed_at > 0
    && {
        let days_since = (chrono::Utc::now().timestamp() - confidence.last_confirmed_at) as f64
            / 86400.0;
        days_since > 30.0  // only auto-archive if 30+ days stale
    }
}
```

---

## 4. Integration with Existing Code

### 4.1 KnowledgeNode Changes

The existing flat `confidence: f64` on `KnowledgeNode` is **deprecated** in favor of a structured field:

```rust
// nt_memory_types.rs: KnowledgeNode changes

pub struct KnowledgeNode {
    // ... existing fields unchanged ...
    pub confidence: f64,  // DEPRECATED — kept for backward compat,
                          // always reflect epistemic.aggregate()

    // NEW:
    pub epistemic: Option<EpistemicConfidence>,  // structured confidence
    pub epistemic_strategy: Option<RetrievalStrategy>,  // how this node was retrieved
}
```

**Migration**: On write, `confidence` is auto-synced from `epistemic.aggregate()`. On read, `epistemic` is populated if missing via `compute_epistemic_from_edges()`.

### 4.2 ConfidenceStore (SQLite-backed)

New table for confidence tracking:

```sql
CREATE TABLE IF NOT EXISTS epistemic_confidence (
    node_id TEXT PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    source_confidence REAL NOT NULL DEFAULT 0.0,
    grounding_confidence REAL NOT NULL DEFAULT 0.0,
    consensus_confidence REAL NOT NULL DEFAULT 0.0,
    recency_confidence REAL NOT NULL DEFAULT 0.5,
    aggregate_score REAL NOT NULL DEFAULT 0.0,
    computed_at INTEGER NOT NULL,
    last_confirmed_at INTEGER NOT NULL DEFAULT 0,
    lambda REAL NOT NULL DEFAULT 0.01,
    auto_archive INTEGER NOT NULL DEFAULT 0,
    metadata TEXT  -- JSON for extended info
);

CREATE TABLE IF NOT EXISTS contradiction_log (
    id TEXT PRIMARY KEY,
    primary_node_id TEXT NOT NULL REFERENCES nodes(id),
    contradicting_node_id TEXT NOT NULL REFERENCES nodes(id),
    claim TEXT,
    contradicting_claim TEXT,
    detected_at INTEGER NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    resolution_note TEXT
);

CREATE INDEX IF NOT EXISTS idx_epistemic_aggregate
    ON epistemic_confidence(aggregate_score DESC);
CREATE INDEX IF NOT EXISTS idx_epistemic_auto_archive
    ON epistemic_confidence(auto_archive) WHERE auto_archive = 1;
CREATE INDEX IF NOT EXISTS idx_contradiction_primary
    ON contradiction_log(primary_node_id);
```

**ConfidenceStore API**:

```rust
pub struct ConfidenceStore {
    conn: Connection,
    cache: LruCache<String, EpistemicConfidence>,
}

impl ConfidenceStore {
    pub fn new(conn: Connection) -> Self;
    pub fn store_confidence(&self, node_id: &str, epistemic: &EpistemicConfidence) -> Result<(), String>;
    pub fn get_confidence(&self, node_id: &str) -> Result<Option<EpistemicConfidence>, String>;
    pub fn get_confidence_batch(&self, node_ids: &[String]) -> Result<HashMap<String, EpistemicConfidence>, String>;
    pub fn apply_decay(&self, lambda: f64, older_than_days: i64) -> Result<u64, String>;
    pub fn reconfirm(&self, node_id: &str, strength: f64) -> Result<(), String>;
    pub fn log_contradiction(&self, primary: &str, contradicting: &str, claim: &str, contra_claim: &str) -> Result<(), String>;
    pub fn get_contradictions(&self, node_id: &str) -> Result<Vec<ContradictingFact>, String>;
    pub fn auto_archive(&self) -> Result<u64, String>;  // returns count archived
    pub fn purge_archived(&self, older_than_days: i64) -> Result<u64, String>;
}
```

### 4.3 search_with_confidence Integration

```rust
// nt_memory_search.rs — new method

impl KnowledgeBase {
    /// Search with epistemic confidence filtering and ranking.
    /// Returns UncertainResults with full provenance.
    pub fn search_with_confidence(
        &self,
        query: &str,
        strategy: RetrievalStrategy,
        limit: usize,
    ) -> Result<Vec<UncertainResult>, String> {
        // Step 1: Run existing search for broader pool
        let raw_results = self.search_fused(query, limit * 3)?;

        // Step 2: Attach epistemic confidence to each result
        let node_ids: Vec<String> = raw_results.iter().map(|r| r.node.id.clone()).collect();
        let confidence_map = self.confidence_store.get_confidence_batch(&node_ids)?;

        let mut uncertain: Vec<UncertainResult> = raw_results
            .into_iter()
            .filter_map(|r| {
                let epistemic = confidence_map.get(&r.node.id).cloned()
                    .or_else(|| Some(EpistemicConfidence::unknown()));
                let contradictions = self.confidence_store
                    .get_contradictions(&r.node.id).ok().unwrap_or_default();
                epistemic.map(|e| UncertainResult {
                    node: r.node,
                    confidence: e,
                    contradictions,
                })
            })
            .collect();

        // Step 3: Filter & rank per strategy
        match strategy {
            RetrievalStrategy::Conservative { min_confidence } => {
                uncertain.retain(|u| u.confidence.aggregate() >= min_confidence);
            }
            RetrievalStrategy::Balanced => {
                // No filter, rank by confidence
            }
            RetrievalStrategy::Exploratory => {
                // Include all, boost contradictions in ranking
            }
            RetrievalStrategy::ConfidenceWeighted { confidence_weight } => {
                // Already have uncertain results; weight by confidence
            }
        }

        // Step 4: Sort by confidence descending
        uncertain.sort_by(|a, b| {
            b.confidence.aggregate()
                .partial_cmp(&a.confidence.aggregate())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        uncertain.truncate(limit);
        Ok(uncertain)
    }
}
```

### 4.4 GWT Integration

```rust
// Events emitted to GWT for consciousness-level awareness

pub enum EpistemicEvent {
    ConfidenceUpdated {
        node_id: String,
        new_aggregate: f64,
        delta: f64,
    },
    ContradictionDetected {
        primary_node: String,
        contradicting_node: String,
        primary_confidence: f64,
        contradicting_confidence: f64,
    },
    AutoArchived {
        node_ids: Vec<String>,
        count: usize,
    },
    ConsensusShift {
        node_id: String,
        old_consensus: f64,
        new_consensus: f64,
    },
}
```

---

## 5. Module Map & File Changes

```
neotrix-core/src/neotrix/l3_memory_impl/
├── nt_memory_kb/
│   ├── nt_memory_types.rs         ← MODIFY: add EpistemicConfidence, UncertainResult,
│   │                                    ContradictingFact, RetrievalStrategy, ConsensusInfo
│   │                                    Add epistemic field to KnowledgeNode
│   ├── nt_memory_confidence.rs    ← NEW file: ConfidenceStore, all computation functions,
│   │                                    decay/reconfirm, saturation, authenticated diversity
│   ├── nt_memory_search.rs        ← MODIFY: add search_with_confidence(), attach_confidence()
│   ├── nt_memory_store.rs         ← MODIFY: add epistemic_confidence + contradiction_log tables
│   │                                    Update insert/update to populate epistemic
│   ├── nt_memory_graph.rs         ← MODIFY: add edge-based consensus detection helpers
│   └── mod.rs                     ← MODIFY: add pub mod nt_memory_confidence, re-exports
```

---

## 6. Implementation Plan

### Phase 1: EpistemicConfidence + decay/reconfirm (1 day)

- Add `EpistemicConfidence` struct to `nt_memory_types.rs`
- Add `RetrievalStrategy` enum
- Implement `aggregate()`, `decay()`, `reconfirm()`, `unknown()`
- Unit tests: saturation curve, decay math, reconfirm bounds

### Phase 2: ConfidenceStore (1 day)

- Create `nt_memory_confidence.rs` with `ConfidenceStore`
- SQLite schema for `epistemic_confidence` + `contradiction_log` tables
- CRUD operations: `store_confidence`, `get_confidence`, `get_confidence_batch`
- Migration: populate `epistemic_confidence` from existing `confidence` field
- Unit tests: store/retrieve, batch, schema migration

### Phase 3: Contradiction Detection (2 days)

**Day 1**: Build `detect_consensus()` — scan edges for Supports/Contradicts relationships
- `log_contradiction()`, `get_contradictions()`
- `ConsensusInfo` struct with support/contradict counts
- Authenticated diversity computation

**Day 2**: Build auto-archive engine
- `auto_archive()`: marks nodes with recency < 0.1 and last_confirmed > 30 days
- `purge_archived()`: removes archived nodes older than N days
- Background task in `nt_mind_background_loop` for periodic decay + archive
- Tests: contradiction chain, archive threshold, decay over time

### Phase 4: search_with_confidence Integration (2 days)

**Day 1**: Wire `search_with_confidence()` into `nt_memory_search`
- `attach_confidence()` helper
- Filter + rank per `RetrievalStrategy`
- Expose through `KnowledgeBase::search_with_confidence()`

**Day 2**: CLI + REPL integration
- Add `--confidence-strategy conservative|balanced|exploratory|weighted` flag to search command
- Show confidence breakdown + contradictions in search output
- GWT event emission on each search

### Phase 5: Auto-reconfirm Pipeline in SEAL (2 days)

**Day 1**: Add `ReconfirmStage` to SEAL pipeline
- Stage runs every 10 ticks
- Queries KB for nodes with aggregate_confidence between 0.3 - 0.6 (tentative)
- Re-extracts entities from source documents
- Updates confidence via `reconfirm(strength)` where strength ∝ evidence match

**Day 2**: Tests + tuning
- End-to-end: seed → decay → reconfirm → verify aggregate increases
- Lambda tuning per domain
- Benchmark: 10K nodes, decay all, measure throughput

---

## 7. Integration with Existing `confidence` Field

| Scenario | Current Behavior | New Behavior |
|----------|-----------------|--------------|
| New node created | `confidence: f64` set by ingester | `epistemic: EpistemicConfidence` computed from source metadata; `confidence` synced to `aggregate()` |
| Node read | `confidence` returned as-is | `epistemic` loaded from `ConfidenceStore`; `confidence` synced for backward compat |
| Node updated | `confidence` overwritten | `reconfirm(strength)` or full recompute; `aggregate()` synced to `confidence` |
| Search | Sort by `score` only | Sort by `score × confidence.aggregate()` when strategy is Balanced |
| Old code reading `confidence` | Direct field access | Field still exists, auto-synced — **zero breakage** |

---

## 8. Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Decay causes valid facts to auto-archive prematurely | Medium | Medium | Configurable lambda per NodeType; manual override via `reconfirm(1.0)`; audit log for all archives |
| Contradiction detection floods the system | Low | Medium | Dedup contradictions; group by claim cluster; batch notifications |
| Performance: every search now loads confidence for all candidates | Medium | High | `ConfidenceStore.get_confidence_batch()` with single SQL query; LRU cache (1000 entries); async confidence loading |
| Backward compat: `confidence: f64` consumers expect immediate field access | Low | Low | Sync `epistemic.aggregate()` ↔ `confidence` on every read/write. Field is always populated. |
| LLM reconfirm costs for auto-reconfirm pipeline | Medium | Low | Rate-limit to 50 nodes/tick; skip nodes confirmed < 7 days ago; use mini model for re-extraction |

---

## 9. Success Criteria

1. **Decomposition**: Every `KnowledgeNode` has a populated `EpistemicConfidence` with all 4 components > 0.0 within 1 week of production
2. **Decay correctness**: After 30 days with `lambda=0.01`, `recency_confidence` drops from 1.0 to ~0.74; after 90 days to ~0.41
3. **Auto-archive accuracy**: < 1% false positive rate on archived nodes (verified by manual audit)
4. **Search improvement**: `search_with_confidence(Conservative{0.7})` returns results with 95% factual accuracy vs current ~80% (measured by held-out fact-check set)
5. **Performance**: `search_with_confidence` adds < 5ms overhead vs raw search (cached confidence, batch SQL)

---

## 10. References

- **Kappa Graph κ(G)**: [github.com/aaronsb/knowledge-graph-system](https://github.com/aaronsb/knowledge-graph-system) — Epistemic confidence model with Michaelis-Menten saturation, authenticated diversity (ADR-044, ADR-063, ADR-070)
- **TCM — Triple Confidence Measurement**: Wu, T. et al. (2024). "Triple confidence measurement in knowledge graph with multiple heterogeneous evidences." *World Wide Web*, 27, 70. — Combines explicit (concept paths, neighbor subgraphs) + embedding evidences for confidence measurement
- **G-UQ — Graph Neural Network Uncertainty**: Trivedi, P. et al. (2024). "Accurate and Scalable Estimation of Epistemic Uncertainty for Graph Neural Networks." *ICLR 2024*. — Graph anchoring strategies for uncertainty quantification
- **Probabilistic Truth Convergence**: ADR-044 in κ(G) — Saturation-based grounding prevents "truth inflation" from hub nodes
