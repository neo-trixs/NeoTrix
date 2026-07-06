#!/usr/bin/env python3
"""Inject research findings from external absorption cycles into NeoTrix SQLite KB.

Creates Insight nodes for each finding, Concept nodes for key concepts,
and cross-reference edges. Skips existing nodes (by title).

Usage:
    python3 scripts/absorb-research-findings.py
"""
import sqlite3, json, time, os, hashlib

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())

def ndig(s):
    return hashlib.md5(s.encode()).hexdigest()[:20]

def retry(c, sql, p, n=3):
    for i in range(n):
        try:
            return c.execute(sql, p)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < n - 1:
                time.sleep(0.5)
                continue
            raise

FINDINGS = [
    {
        "title": "E8-Transformer GraphResonator",
        "summary": "E8-Transformer softly quantizes hidden states into the 240 roots of the E8 root system with an E8GraphResonator associative memory. Tokens map to nearest roots; resonance biases predictions toward contextually related tokens. Achieves validation loss 0.35-0.45 compared to GPT-2 Small's ~3.0, and extrapolates gracefully to 1500 tokens vs hard loops at 300-500.",
        "source": "github.com/Liber1917/sovereign-lila-e8",
        "tags": ["E8 Root System", "Graph Resonance"],
        "related": [],
    },
    {
        "title": "E8-LoRA Parameter-Efficient Fine-Tuning",
        "summary": "Injects E8 root system geometry as a parallel frozen path via a fixed orthogonal matrix from the E8 lattice, scaled by one learnable parameter per layer. Acts as a symmetry filter guiding representations toward densest sphere packing in 8 dimensions.",
        "source": "doi.org/10.5281/zenodo.18787441",
        "tags": ["E8 Root System"],
        "related": [0],
    },
    {
        "title": "Kuramoto Oscillator Spectral Seeding",
        "summary": "Initializing natural frequencies omega from eigenvectors of the coupling graph Laplacian (weighted by output separation and inverse eigenvalue) eliminates random-initialization basin failure entirely — 46/100 to 100/100 seeds exceeding 60% test accuracy. Kuramoto equilibrium encodes loss gradients w.r.t. natural frequency.",
        "source": "arxiv.org/abs/2604.10272",
        "tags": ["Kuramoto Oscillator", "Spectral Graph Theory"],
        "related": [3],
    },
    {
        "title": "Kuramoto Oscillatory Phase Encoding",
        "summary": "Initializes phases at the first layer with multi-frequency 2D positional embeddings so rotation is meaningful in early layers. Phase dynamics start from spatial positions and gradually evolve to semantic-relationally synchronized states.",
        "source": "arxiv.org/abs/2604.07904",
        "tags": ["Kuramoto Oscillator"],
        "related": [2],
    },
    {
        "title": "TopK Sparse Autoencoder Standard",
        "summary": "TopK SAE directly sets L0 sparsity via k parameter — no L1 penalty needed. Outperforms ReLU SAEs on sparsity-reconstruction frontier, gap grows with scale. AuxK loss prevents dead latents. Encoder = decoder^T weight tying reduces dead latents. Standard k values: 32-64 for 16K dict, 64-128 for 32K dict, 128-256 for 65K dict.",
        "source": "arxiv.org/abs/2406.04093",
        "tags": ["Sparse Autoencoder", "Neural Feature Interpretability"],
        "related": [5, 6],
    },
    {
        "title": "AdaptiveK Input-Dependent Sparsity",
        "summary": "Uses linear probes to detect input complexity and dynamically sets k — outperforms fixed-sparsity on reconstruction fidelity, explained variance, and interpretability across 10 models.",
        "source": "aclanthology.org/2026.findings-acl.1187",
        "tags": ["Sparse Autoencoder", "Mixture of Experts"],
        "related": [4, 6],
    },
    {
        "title": "SoftSAE Differentiable Top-K",
        "summary": "Uses differentiable Soft Top-K to learn input-dependent sparsity. Suitable for streaming/online scenarios where input complexity varies.",
        "source": "arxiv.org/abs/2605.06610",
        "tags": ["Sparse Autoencoder", "Neural Feature Interpretability"],
        "related": [4, 5],
    },
]

CONCEPTS = [
    "E8 Root System",
    "Graph Resonance",
    "Kuramoto Oscillator",
    "Spectral Graph Theory",
    "Sparse Autoencoder",
    "Mixture of Experts",
    "Neural Feature Interpretability",
]


def upsert_node(c, node_type, title, summary, url, meta):
    existing = c.execute(
        "SELECT id FROM nodes WHERE title=? AND node_type=? LIMIT 1",
        (title, node_type),
    ).fetchone()
    if existing:
        return existing[0], False

    kid = f"rs-{ndig(url or title)}"
    retry(
        c,
        "INSERT OR IGNORE INTO nodes "
        "(id,node_type,title,summary,content,url,domain,language,confidence,importance,created_at,updated_at,metadata) "
        "VALUES (?,?,?,?,?,?,?,'en',0.9,0.7,?,?,?)",
        (kid, node_type, title, summary, "", url, "research", NOW, NOW, json.dumps(meta)),
    )
    row = c.execute("SELECT id FROM nodes WHERE id=? LIMIT 1", (kid,)).fetchone()
    if row:
        return row[0], True
    row = c.execute("SELECT id FROM nodes WHERE title=? AND node_type=? LIMIT 1", (title, node_type)).fetchone()
    return (row[0], False) if row else (kid, True)


def upsert_edge(c, src, tgt, rel, weight, desc):
    existing = c.execute(
        "SELECT id FROM edges WHERE source_id=? AND target_id=? AND relation_type=? LIMIT 1",
        (src, tgt, rel),
    ).fetchone()
    if existing:
        return False
    eid = f"re-{ndig(f'{src}{tgt}{rel}')}"
    retry(
        c,
        "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) "
        "VALUES (?,?,?,?,?,?,?)",
        (eid, src, tgt, rel, weight, desc, NOW),
    )
    return True


def kv_put(c, namespace, key, value):
    retry(
        c,
        "INSERT OR REPLACE INTO kv_store (namespace,key,value,updated_at) VALUES (?,?,?,?)",
        (namespace, key, value, NOW),
    )


def main():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()

    print("═══ Research Findings Absorption ═══\n")

    # Phase 1: Create Concept nodes
    concept_ids = {}
    for name in CONCEPTS:
        nid, created = upsert_node(c, "Concept", name, "", "", {"source": "research_absorption"})
        concept_ids[name] = nid
        status = "NEW" if created else "EXISTS"
        print(f"  [{status}] Concept: {name} → {nid}")

    concepts_created = sum(1 for name in CONCEPTS if concept_ids[name] == f"rs-{ndig(name or name)}" or True)
    # Count actual newly created
    conc_count_initial = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Concept'").fetchone()[0]

    # Phase 2: Create Insight nodes + edges to concepts
    insight_ids = {}
    new_insights = 0
    for i, f in enumerate(FINDINGS):
        nid, created = upsert_node(
            c,
            "Insight",
            f["title"],
            f["summary"],
            f"https://{f['source']}",
            {"source": f["source"], "tags": f["tags"], "absorbed_at": NOW},
        )
        insight_ids[i] = nid
        if created:
            new_insights += 1
        status = "NEW" if created else "EXISTS"
        print(f"  [{status}] Insight: {f['title']} → {nid}")

    # Phase 3: Edge — Insight → Concept
    edge_count = 0
    for i, f in enumerate(FINDINGS):
        for tag in f["tags"]:
            if tag in concept_ids:
                ok = upsert_edge(
                    c, insight_ids[i], concept_ids[tag], "relates_to", 0.85,
                    f"Insight relates to concept {tag}",
                )
                if ok:
                    edge_count += 1

    # Phase 4: Cross-reference edges between related findings
    for i, f in enumerate(FINDINGS):
        for j in f.get("related", []):
            if j < len(FINDINGS):
                ok = upsert_edge(
                    c, insight_ids[i], insight_ids[j], "references", 0.7,
                    f"Cross-reference: {f['source']}",
                )
                if ok:
                    edge_count += 1

    conn.commit()

    # Phase 5: Record metadata in kv_store
    payload = json.dumps({
        "insights_injected": new_insights,
        "concepts_injected": len(CONCEPTS),
        "edges_created": edge_count,
        "total_findings": len(FINDINGS),
        "total_concepts": len(CONCEPTS),
        "absorbed_at": NOW,
    })
    kv_put(c, "research_absorption", "cycle-research-findings-2026-07-05", payload)
    conn.commit()

    # Phase 6: Verify counts
    total_insights = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Insight'").fetchone()[0]
    total_concepts = c.execute("SELECT COUNT(*) FROM nodes WHERE node_type='Concept'").fetchone()[0]
    edges_between = c.execute(
        "SELECT COUNT(*) FROM edges WHERE source_id IN ({}) AND target_id IN ({}) AND relation_type='relates_to'".format(
            ",".join("?" for _ in insight_ids.values()),
            ",".join("?" for _ in concept_ids.values()),
        ),
        list(insight_ids.values()) + list(concept_ids.values()),
    ).fetchone()[0]
    related_edges = c.execute(
        "SELECT COUNT(*) FROM edges WHERE relation_type='references' AND source_id IN ({})".format(
            ",".join("?" for _ in insight_ids.values()),
        ),
        list(insight_ids.values()),
    ).fetchone()[0]

    conn.close()

    print(f"\n═══ Summary ═══")
    print(f"  Insights created this run: {new_insights}")
    print(f"  Concepts in KB:            {total_concepts}")
    print(f"  Edges created this run:    {edge_count} ({edges_between} relates_to, {related_edges} references)")
    print(f"  Total Insights in KB:      {total_insights}")
    print(f"  KV namespace 'research_absorption': written ✓")
    print(f"\nDone.")


if __name__ == "__main__":
    main()
