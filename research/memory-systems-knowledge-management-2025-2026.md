# Memory Systems & Knowledge Management for AI Desktop Assistants

**Research Date**: 2026-09-16
**Scope**: Local-first, encrypted, knowledge-graph-backed memory architectures for AI agents

---

## 1. Cortex — Local-First Memory (CRDT-Free Sync, HLC, AES-256-GCM)

**Source**: github.com/gambletan/cortex (MIT, Rust, 3.8MB binary)

### Architecture

```
4-Tier Memory Model:
┌─────────────────────────────────────────────┐
│ Working Memory     → session scratch pad    │
│ Episodic Memory    → raw experiences + TS   │
│ Semantic Memory    → promoted facts + prefs │
│ Procedural Memory  → learned workflows      │
└─────────────────────────────────────────────┘
         ↓ Consolidation Engine
  (decay stale episodes, promote recurring patterns)
```

### Data Structures

```rust
// Storage: SQLCipher (AES-256-GCM at page level)
// DB path: ~/.cortex/global.db
// Embedding: in-memory vector index (all-MiniLM-L6-v2, dim=384)

struct Memory {
    id: Uuid,
    tier: MemoryTier,         // Working | Episodic | Semantic | Procedural
    content: String,
    embedding: Option<[f32; 384]>,
    privacy: Privacy,         // Private(default) | Shared | Public
    metadata: MemoryMetadata,
    created_at: Timestamp,
    last_accessed: Timestamp,
    importance: f64,          // importance-aware exponential decay
}

// Belief system (Bayesian, self-correcting)
struct Belief {
    id: Uuid,
    statement: String,
    confidence: f64,           // 0.0–1.0
    observations: Vec<Observation>,  // CRDT add-only set
    valid_from: Timestamp,
    valid_until: Option<Timestamp>,
}
```

### Sync Protocol (CRDT-Free, No Server)

```rust
// Changelog-based, append-only oplog per device
// Each device writes to its own subfolder in iCloud/GDrive/Dropbox

struct SyncOp {
    device_id: String,
    hlc: HybridLogicalClock,  // physical TS + logical counter
    entity_id: Uuid,
    op_type: OpType,          // Insert | Update | Delete | PrivacyChange
    payload: EncryptedPayload, // AES-256-GCM
    hmac: [u8; 32],           // tamper-evident
}

// Merge: Last-Writer-Wins per entity (HLC ordering)
// Beliefs: CRDT add-only sets (merge all observations, confidence wins)
// Demoting Private retracts from other devices
```

### Encryption Stack

| Layer | Algorithm | Notes |
|-------|-----------|-------|
| DB at rest | SQLCipher AES-256-GCM | Page-level encryption |
| Sync oplog | AES-256-GCM per line | Argon2id KDF (m=64MiB, t=3) |
| Key rotation | Versioned `ENC2` envelopes | Forward secrecy, no re-encryption |
| Integrity | HMAC on manifest + every op | Plaintext injection rejected |
| Secure wipe | `zeroize` crate | Memory zeroed after use |

### Performance

| Operation | Latency |
|-----------|---------|
| Ingest | 62–156µs |
| Search (top-10) | 253–568µs |
| Belief update | 28µs |
| Context generation | 111µs |

### Key Takeaways for NeoTrix
- **HLC ordering** is simpler than CRDT for single-user multi-device — no conflict resolution needed, just LWW per entity
- **Per-memory privacy opt-in** with cross-device retraction is a strong UX pattern
- **4-tier consolidation** (episodic→semantic promotion with decay) mirrors NT-MEMORY's evolution needs
- **SQLCipher + in-memory vector** is the proven local-first stack

---

## 2. Heirloom — SQLite + XChaCha20-Poly1305 Encrypted Memory

**Source**: github.com/MayonaiseLover/heirloom (Rust)

### Architecture

```
heirloom.db (SQLite)     ← plaintext, active
    ↓ seal
heirloom.db.hlm          ← XChaCha20-Poly1305 encrypted
    ↓ unseal
heirloom.db              ← decrypted, active again
```

### Seal/Unseal Workflow

```bash
# Seal: encrypt DB at rest
HEIRLOOM_PASSPHRASE='correct horse battery staple' heirloom seal
# → heirloom.db becomes heirloom.db.hlm
# → plaintext shredded

# Unseal: decrypt for active use
HEIRLOOM_PASSPHRASE='correct horse battery staple' heirloom unseal
# → back to working heirloom.db
```

### Crypto Stack

```rust
// Encryption: XChaCha20-Poly1305 (192-bit random nonce)
// KDF: Argon2id (m=64 MiB, t=3, p=1)
// Key stored: OS keychain (macOS Keychain / Linux libsecret)

struct VaultFile {
    magic: [u8; 4],       // "HLM\0"
    version: u8,
    salt: [u8; 16],       // Argon2id salt
    nonce: [u8; 24],      // XChaCha20 nonce
    ciphertext: Vec<u8>,  // encrypted SQLite pages
}
```

### Why XChaCha20-Poly1305 over AES-256-GCM?

| Property | AES-256-GCM | XChaCha20-Poly1305 |
|----------|-------------|---------------------|
| Nonce size | 96-bit | 192-bit |
| Random collision risk | Higher (birthday at 2^48) | Lower (birthday at 2^96) |
| Hardware AES needed | Yes (AES-NI) | No (software-only) |
| Browser/WASM support | Good | Good |
| Key derivation | PBKDF2/HKDF | HKDF |

**XChaCha20-Poly1305 is preferred for local-first** because the 192-bit nonce makes random nonce reuse negligible without requiring a counter, and it runs efficiently without AES-NI hardware.

### MCP Integration

```
heirloom serve          # Start MCP server on stdio
heirloom viewer         # Local web viewer (loopback only)
heirloom watch          # Auto-capture daemon
```

### Key Takeaways
- **Seal/unseal pattern** is clean: DB is plaintext at runtime, encrypted at rest
- **12-word BIP-39 recovery key** for passphrase loss
- **No telemetry, no server** — pure local-first
- Similar to FERAL-AI's `memory.at_rest.py` pattern (ChaCha20-Poly1305 + HKDF + atomic write + integrity check)

---

## 3. MemPalace — Memory Palace Mining

**Source**: github.com/MemPalace/mempalace (Python, ChromaDB + SQLite)

### Architecture

```
Memory Palace Structure (Method of Loci):
┌─────────────────────────────────────────────────┐
│  WING (project/person)                          │
│  └── ROOM (topic within wing)                   │
│      └── HALL (memory type corridor)            │
│          └── CLOSET (compressed summary)        │
│              └── DRAWER (verbatim original)     │
│                                                  │
│  TUNNEL = cross-wing same-topic connection      │
└─────────────────────────────────────────────────┘

Storage:
  ChromaDB → semantic vector search (verbatim text)
  SQLite   → temporal knowledge graph (entity-relationship triples)
```

### Mining Flow

```python
# Project mining
mempalace mine ~/projects/myapp

# Conversation mining (Claude/ChatGPT/Slack exports)
mempalace mine ~/.claude/projects/ --mode convos

# Per-message sweep
mempalace sweep <session_id>

# Auto-classified extraction (5 memory types):
# Decisions, Preferences, Milestones, Problems, Emotional context
mempalace mine ~/chats/ --mode convos --extract general
```

### Knowledge Graph (Temporal)

```python
# Add triple with validity window
kg.add_triple("Kai", "works_on", "Orion", valid_from="2025-06-01")

# Invalidate (mark end date without deletion)
kg.invalidate("Kai", "works_on", "Orion", ended="2026-03-01")

# Query current state
kg.query_entity("Kai")
# → [Kai → works_on → Orion (ended), Kai → recommended → Clerk]

# Historical query
kg.query_entity("Maya", as_of="2026-01-20")
```

### Retrieval Hierarchy (4 layers)

| Layer | Content | Size | When Loaded |
|-------|---------|------|-------------|
| L0 | Identity | ~50 tokens | Always |
| L1 | Critical facts (AAAK) | ~120 tokens | Always |
| L2 | Room recall | On demand | Topic surfaces |
| L3 | Deep semantic search | On demand | Explicitly asked |

### Benchmark Reality Check

| Metric | Claimed | Independent Verification |
|--------|---------|-------------------------|
| LongMemEval R@5 (raw) | 96.6% | ✅ Confirmed — but this measures ChromaDB's embeddings, not the palace structure |
| Palace structure benefit | +34% | ⚠️ Measures wing+room filtering vs unfiltered — not a novel algorithm |
| AAAK compression | 30x | ❌ Drops accuracy from 96.6% to 84.2% (lossy) |
| LoCoMo | 100% | ⚠️ Used top_k=50 against datasets with 19-32 sessions (retrieves everything) |

**Critical insight**: The 96.6% score comes from ChromaDB's default all-MiniLM-L6-v2 embeddings on verbatim text. The palace hierarchy is not involved in the benchmark.

### Key Takeaways
- **Verbatim storage** (don't summarize) is the winning approach for retrieval quality
- **Temporal knowledge graph** with validity windows is genuinely useful for compliance
- **Spatial hierarchy** (wings/rooms) is a nice UX metaphor but doesn't improve retrieval
- **Pluggable backend** contract is good architecture (ChromaDB default, swap-able)

---

## 4. SYNAPSE — Spreading Activation Memory

**Source**: ACL 2026 Findings, github.com/hq0709/synapse

### Architecture

```
Unified Episodic-Semantic Graph:
┌────────────────────────────────────────────────────┐
│  Episodic Nodes (V_E)                              │
│  = (content, embedding, timestamp)                 │
│                                                    │
│  Semantic Nodes (V_S)                              │
│  = LLM-extracted concepts (entities, preferences)  │
│                                                    │
│  Edge Types:                                       │
│  ├── Temporal:  e_t → e_{t+1}                     │
│  ├── Abstraction: e ↔ s (bidirectional)           │
│  └── Association: s ↔ s (latent correlations)     │
└────────────────────────────────────────────────────┘
```

### Spreading Activation Algorithm

```python
# Dual-trigger anchor identification
def find_anchors(query, graph):
    lexical_anchors = bm25_retrieve(query, graph.episodic_nodes, top_k=k)
    semantic_anchors = dense_retrieve(query, graph.nodes, top_k=k)
    return union(lexical_anchors, semantic_anchors)

# Spreading activation (ACT-R inspired)
def spread(anchors, graph, T=3, S=0.8, delta=0.5, rho=0.01):
    a = zeros(|V|)
    a[anchors] = anchor_scores  # inject energy

    for t in range(T):
        a_new = zeros(|V|)
        for node_i in graph.nodes:
            for predecessor_j in graph.in_neighbors(node_i):
                w_ji = edge_weight(predecessor_j, node_i, rho)
                fan_j = out_degree(predecessor_j)
                a_new[node_i] += a[predecessor_j] * w_ji / fan_j * S
        a = a_new

    # Lateral inhibition (suppress competitors)
    a = sigmoid(gamma * (a - theta))  # gamma=5, theta=0.5
    return a

# Triple-signal hybrid scoring
def relevance_score(node_i, query_embedding, activation, pagerank):
    return (lambda1 * cosine_sim(node_i, query_embedding) +
            lambda2 * activation[i] +
            lambda3 * pagerank[i])
    # lambda = {0.5, 0.3, 0.2}
```

### Key Parameters

| Parameter | Default | Purpose |
|-----------|---------|---------|
| T (propagation steps) | 3 | Max hop depth |
| S (spreading factor) | 0.8 | Energy retention per hop |
| delta (retention) | 0.5 | Activation decay |
| rho (temporal decay) | 0.01 | Time-weighted edge strength |
| gamma (sigmoid steepness) | 5 | Inhibition aggressiveness |
| theta (inhibition threshold) | 0.5 | Competitive cutoff |
| K (max edges per node) | 15 | Sparsity constraint |
| epsilon (dormancy threshold) | 0.01 | GC cutoff |
| W (dormancy window) | 10 | GC observation period |

### Performance (LoCoMo Benchmark)

| Task | SYNAPSE | Best Baseline | Improvement |
|------|---------|---------------|-------------|
| Temporal Reasoning | 50.1 F1 | 45.9 (A-Mem) | +4.2 |
| Multi-Hop Reasoning | 35.7 F1 | 27.0 (A-Mem) | +8.7 |
| Adversarial Robustness | 96.6 F1 | 69.2 (LoCoMo) | +27.4 |
| Gold-evidence recall@30 | 0.804 | 0.683 (vectors only) | +17.7% |

### Key Takeaways
- **Spreading activation** solves the "Contextual Tunneling" problem — finding causally related but semantically distant memories
- **Dual-trigger** (BM25 + dense) prevents seed dependence
- **Lateral inhibition** prevents "Hub Explosion" in dense graphs
- **Fan effect** (activation dilution by out-degree) is architecturally enforced, not just rhetorical
- **Config is production-ready**: all hyperparameters in `config.py`, validated with sensitivity analysis

---

## 5. EverMemOS — MemCells → MemScenes → Reconstructive Recollection

**Source**: ACL 2026 Long Paper, github.com/EverMind-AI/EverMemOS

### Architecture

```
Three-Phase Lifecycle (engram-inspired):
┌──────────────────────────────────────────────────────┐
│ Phase I: Episodic Trace Formation                    │
│   Dialogue → MemCells (episode + facts + foresight)  │
│                                                      │
│ Phase II: Semantic Consolidation                     │
│   MemCells → MemScenes (thematic clusters)          │
│   + User Profile updates                             │
│                                                      │
│ Phase III: Reconstructive Recollection               │
│   Query → MemScene selection → Episode re-ranking   │
│   → Foresight filtering → Sufficiency verification  │
└──────────────────────────────────────────────────────┘
```

### MemCell Structure

```python
@dataclass
class MemCell:
    E: str                    # Episode narrative (semantic anchor)
    F: List[AtomicFact]       # Atomic facts (precise matching)
    P: List[Foresight]        # Time-bounded foresight with validity
    M: Metadata               # Timestamps, sources, IDs

    # Foresight: plans/constraints with validity window
    # e.g., "taking antibiotics" valid 2026-01-15 to 2026-01-25

@dataclass
class MemScene:
    C: List[MemCell]          # Member MemCells
    e_centroid: np.ndarray    # Running-mean embedding
    t_last: Timestamp         # Most recent timestamp
```

### Online Clustering Algorithm

```python
def assimilate(new_cell, scenes, tau, delta_max):
    """Online MemScene assignment — no batch reprocessing."""
    best_scene = find_nearest_centroid(new_cell.embedding, scenes)
    cell_in_scene = find_closest_cell(new_cell, best_scene)

    if (cosine_sim(new_cell, best_scene.e_centroid) > tau and
        time_gap(new_cell, cell_in_scene) < delta_max and
        no_profile_conflict(new_cell, best_scene)):
        # Merge into existing scene
        best_scene.C.append(new_cell)
        best_scene.e_centroid = update_running_mean(best_scene, new_cell)
    else:
        # Create new scene
        scenes.append(MemScene(C=[new_cell], ...))
```

### Reconstructive Recollection (Read Path)

```python
def retrieve(query, cells, scenes, top_N=10, top_K=10):
    # 1. Hybrid retrieval over atomic facts (dense + BM25 → RRF)
    cell_scores = rrf_fusion(dense_retrieve(query, cells),
                             bm25_retrieve(query, cells))

    # 2. Score scenes by max constituent cell relevance
    scene_scores = {s: max(cell_scores[c] for c in s.C) for s in scenes}
    selected_scenes = top_N(scene_scores)

    # 3. Pool episodes from selected scenes, re-rank
    episodes = flatten([s.episodes() for s in selected_scenes])
    selected_episodes = rerank(episodes, query, top_K)

    # 4. Foresight filtering: keep only time-valid
    valid_foresight = [f for f in all_foresight
                       if f.t_start <= now <= f.t_end]

    # 5. Sufficiency verification (LLM judge)
    context = serialize(selected_episodes, valid_foresight)
    if not sufficiency_check(context, query):
        context = query_rewrite_and_retry(query, context)

    return context
```

### Performance

| Benchmark | EverMemOS | Best Baseline | Improvement |
|-----------|-----------|---------------|-------------|
| LoCoMo overall | 93.05% | 86.05% (Zep) | +7.0% |
| LoCoMo multi-hop | — | — | +19.7% |
| LoCoMo temporal | — | — | +10.0% |
| LongMemEval overall | 83.00% | 76.30% (MemOS) | +6.7% |
| LongMemEval knowledge update | — | — | +20.6% |

### Key Takeaways
- **MemCell = (E, F, P, M)** is a clean atomic unit with episode, facts, and time-bounded foresight
- **Online clustering** (no batch reprocessing) is essential for real-time agents
- **Necessity and sufficiency** principle prevents context pollution
- **Foresight filtering** (expire old plans) is a novel and practical feature
- **User profile evolution** from scene summaries (not individual turns) separates stable traits from transient states

---

## 6. SEEM — Episodic Event Frames with Provenance Pointers

**Source**: ACL 2026 Long Paper, arxiv.org/abs/2601.06411

### Architecture

```
Dual-Layer Memory:
┌──────────────────────────────────────────────────────┐
│ Episodic Memory Layer (EML)                          │
│   Episodic Event Frames (EEFs) — narrative flow      │
│   Linked by provenance pointers to source passages   │
│                                                      │
│ Graph Memory Layer (GML)                             │
│   Relational triples (s, r, o, τ) — factual facts   │
│   Also linked by provenance pointers                 │
└──────────────────────────────────────────────────────┘
```

### Episodic Event Frame (EEF)

```python
@dataclass
class EEF:
    rho_eml: List[str]        # Provenance pointers → source passages
    v_sum: str                # Event summary (1-2 sentences)
    roles: List[EventRole]    # Semantic roles per sub-event

@dataclass
class EventRole:
    v_par: str    # Participants
    v_act: str    # Action
    v_tmp: str    # Time
    v_spa: str    # Location
    v_cau: str    # Causality
    v_man: str    # Manner
```

### Associative Fusion

```python
def fuse_if_same_event(prev_frame, new_frame):
    """LLM judge: do these frames describe the same event?"""
    delta = llm_judge(prev_frame, new_frame)
    if delta == 1:
        fused = merge_attributes(prev_frame, new_frame)
        fused.rho_eml = prev_frame.rho_eml + new_frame.rho_eml  # aggregate provenance
        return fused
    return new_frame
```

### Reverse Provenance Expansion (RPE)

```python
def retrieve_with_rpe(query, gml, eml):
    # 1. GML retrieval: find relevant facts
    K_top = gml_query(query)  # seed set

    # 2. Graph propagation (PersonalizedPageRank)
    P_ret = pagerank_propagation(K_top, gml)

    # 3. Map passages → event frames
    E_ret = {eml.frame_for(p) for p in P_ret}

    # 4. REVERSE PROVENANCE EXPANSION
    # Follow aggregated provenance pointers to get ALL related passages
    P_final = P_ret ∪ ∪{e.rho_eml for e in E_ret}

    return P_final
```

### Performance

| Metric | SEEM | HippoRAG 2 | Improvement |
|--------|------|------------|-------------|
| LoCoMo F1 | 61.1 | 58.3 | +2.8 |
| LoCoMo LLM-Judge | 78.0 | 76.5 | +1.5 |
| LongMemEval Accuracy | 65.0 | 60.6 | +4.4 |

### Key Takeaways
- **Provenance pointers** that aggregate during fusion are the key innovation — a single frame can point to scattered source passages
- **RPE mechanism** follows provenance chains to reconstruct complete event context from fragments
- **Frame semantics** (6 roles: who/what/when/where/why/how) provide high-density semantic anchors
- **Two-layer design** (narrative EML + factual GML) separates concerns cleanly

---

## 7. Vector Databases for Local Code Embeddings

### Comparison Matrix (2026)

| Feature | LanceDB | Qdrant | Chroma |
|---------|---------|--------|--------|
| **Deployment** | Embedded (in-process) | Server (Docker) or Edge | Embedded or Server |
| **Storage** | Lance columnar (disk) | In-memory + WAL | In-memory + SQLite |
| **Index** | IVF-PQ + HNSW | HNSW + quantization | HNSW |
| **Hybrid Search** | FTS + vector (native) | Sparse + dense (native) | Basic |
| **Filtered Search** | DataFusion SQL | Native payload index (pre-filter) | Python-side (post-filter) |
| **Quantization** | IVF-PQ | INT8/FP8/binary (4x savings) | None |
| **Max Reliable Scale** | Disk-bound (TBs) | 100M+ vectors | ~500K vectors |
| **Multimodal** | Text + image + audio | Text + sparse | Text |
| **Versioning** | Every write = immutable snapshot | Manual | Manual |
| **Rust SDK** | ✅ | ✅ (crate) | ❌ |
| **License** | Apache 2.0 | Apache 2.0 | Apache 2.0 |

### Performance Benchmarks (2026)

| Metric | LanceDB | Qdrant (INT8) | Chroma |
|--------|---------|---------------|--------|
| QPS @ 5M vectors | ~1,900 | ~2,400 | ~850 |
| P95 query latency | 18ms | 14ms | 42ms |
| Recall@10 | 0.97 | 0.98 | 0.96 |
| RAM @ 100K × 1536d | Disk-based | ~160MB | ~620MB |
| Ingestion 100K docs | 4-8 min | 60-90s | 45-90s |

### Recommendation for NeoTrix

**LanceDB** is the best fit for a Rust-native desktop AI assistant:
- **Rust SDK** (`lancedb` crate) — no FFI, no Python
- **Disk-based** — vectors live on disk, working set paged into memory
- **Lance format** — open, versioned, Arrow-compatible
- **Embedded** — no server process, single binary deployment
- **Hybrid search** — FTS + vector in one query

```rust
// LanceDB Rust usage
let db = lancedb::connect("~/.neotrix/vectors.lance")?;
let table = db.create_table("memories", sample_data)?;
let results = table
    .vector_search(query_embedding)?
    .limit(10)
    .execute()?;
```

**Alternative**: `sqlite-vec` for simpler integration if the graph is already in SQLite (PyCodeKG pattern).

---

## 8. Knowledge Graph Construction from Code

### Tools Survey

| Tool | Languages | Storage | MCP | Incremental | Key Feature |
|------|-----------|---------|-----|-------------|-------------|
| **PyCodeKG** | Python | SQLite + sqlite-vec | ✅ 19 tools | ✅ | 15-phase analysis pipeline, CodeRank (PageRank) |
| **CodeGraph** | 19 languages | SQLite + FTS5 | ✅ | ✅ (FSEvents) | Pre-indexed, 94% fewer tool calls |
| **Code-Nexus** | Python/TS/Rust | SQLite (WAL) | ✅ | ✅ (SHA256 diff) | Git temporal overlay, 3D WebGL viz |
| **sciogen** | 100+ languages | KuzuDB + ChromaDB + SQLite | ✅ | ✅ | Full symbol resolution with confidence scores |
| **Synaptiq** | Python/JS/TS/Go/Rust/C | KuzuDB (Cypher) | ✅ | ✅ (watch mode) | 11-phase analysis, Leiden community detection |
| **codingest** | 17 languages | KGLite (graph) | ✅ | ✅ | Revision-aware, multi-repo comparison |

### PyCodeKG — Deterministic Code Knowledge Graph

```python
# Edge types extracted from AST
CONTAINS    # module → class/function
CALLS       # function → function
IMPORTS     # module → module
INHERITS    # class → class
RESOLVES_TO # call_site → definition (across import aliases)

# Hybrid retrieval
1. Vector phase: sqlite-vec cosine similarity on function/class embeddings
2. Graph expansion: BFS along typed edges from seed hits

# Structure is ground truth; embeddings are acceleration layer
# When graph and vector disagree → graph wins
```

### CodeGraph — Pre-Indexed for AI Agents

```python
# Architecture
1. Extraction: tree-sitter AST → nodes (functions, classes) + edges (calls, imports)
2. Storage: SQLite + FTS5 (.codegraph/codegraph.db)
3. Resolution: references → definitions, imports → files, inheritance
4. Auto-Sync: OS file events → debounce (2s) → incremental update

# MCP tools for Claude Code
codegraph_search("UserService")     # Find symbols by name
codegraph_callers(node_id)          # Who calls this function
codegraph_callees(node_id)          # What this function calls
codegraph_impact(node_id, depth=2)  # Blast radius analysis
codegraph_context("fix login bug")  # Build relevant code context
```

### Implementation Pattern (Applicable to NeoTrix)

```rust
// Unified code knowledge graph pipeline
struct CodeKnowledgeGraph {
    // Storage: single SQLite file
    db: SqliteStore,      // graph topology + FTS5
    vec: LanceDbStore,    // semantic embeddings (optional)

    // Extraction: tree-sitter (100+ grammars)
    parsers: HashMap<Language, tree_sitter::Parser>,

    // Edge types
    edges: EdgeRegistry,  // CONTAINS, CALLS, IMPORTS, INHERITS, RESOLVES_TO
}

impl CodeKnowledgeGraph {
    fn index(&mut self, repo_path: &Path) {
        // 1. Walk files, filter by .gitignore
        // 2. Parse each file with tree-sitter
        // 3. Extract nodes + typed edges
        // 4. Resolve references (imports → definitions)
        // 5. Embed symbol summaries (not raw code)
        // 6. Store in SQLite + optional vector index
    }

    fn query_hybrid(&self, query: &str) -> Vec<CodeResult> {
        // 1. Vector similarity search (top-k)
        // 2. BFS expansion along graph edges
        // 3. Rank by combined score
        // 4. Return with file:line provenance
    }
}
```

---

## Synthesis: Design Recommendations for NeoTrix NT-MEMORY

### Storage Stack

```
┌─────────────────────────────────────────────┐
│ SQLite + SQLCipher (encrypted at rest)      │
│   ├── graph topology (nodes, edges)         │
│   ├── temporal knowledge graph              │
│   ├── FTS5 full-text search                 │
│   └── sync oplog (HLC-ordered)             │
│                                             │
│ LanceDB (vector embeddings)                 │
│   ├── code symbol embeddings                │
│   ├── conversation embeddings               │
│   └── hybrid search (vector + FTS)          │
│                                             │
│ Encryption: XChaCha20-Poly1305 + Argon2id  │
│ (preferred over AES-256-GCM for local-first)│
└─────────────────────────────────────────────┘
```

### Memory Architecture

```
借鉴来源 → NT-MEMORY 实现

Cortex:     4-tier model (Working/Episodic/Semantic/Procedural)
EverMemOS:  MemCell = (episode, facts, foresight, metadata)
SYNAPSE:    Spreading activation for retrieval
SEEM:       Provenance pointers for traceability
Heirloom:   Seal/unseal workflow for encryption at rest
MemPalace:  Verbatim storage (don't summarize for retrieval)
```

### Key Algorithms to Implement

1. **HLC ordering** for multi-device sync (Cortex pattern)
2. **Spreading activation** for memory retrieval (SYNAPSE algorithm)
3. **Online MemScene clustering** for consolidation (EverMemOS pattern)
4. **Provenance pointers** for traceability (SEEM pattern)
5. **Temporal knowledge graph** with validity windows (MemPalace pattern)

---

## Sources

1. Cortex: github.com/gambletan/cortex — Local-first AI memory, 3.8MB Rust binary
2. Heirloom: github.com/MayonaiseLover/heirloom — Encrypted SQLite memory, seal/unseal
3. MemPalace: github.com/MemPalace/mempalace — Memory palace mining, 96.6% LongMemEval
4. SYNAPSE: ACL 2026 Findings — Spreading activation episodic-semantic memory
5. EverMemOS: ACL 2026 Long Paper — MemCells → MemScenes → Reconstructive Recollection
6. SEEM: ACL 2026 Long Paper — Episodic Event Frames with provenance pointers
7. LanceDB: github.com/lancedb/lancedb — Disk-based vector DB, Rust SDK
8. Qdrant: qdrant.tech — Production vector DB with payload filtering
9. Chroma: github.com/chroma-core/chroma — Embedded vector DB for prototyping
10. PyCodeKG: github.com/Flux-Frontiers/pycode_kg — Python code knowledge graph
11. CodeGraph: github.com/elrudrakssh/codegraph — Pre-indexed code KG for AI agents
12. Code-Nexus: github.com/snagrecha/code-nexus — Temporal code KG with git overlay
13. sciogen: github.com/ayanbag/sciogen — Codebase intelligence layer with confidence scores
14. Synaptiq: github.com/scanadi/axon — 11-phase code knowledge graph
15. codingest: github.com/kkollsga/codingest — 17-language polyglot code graph
