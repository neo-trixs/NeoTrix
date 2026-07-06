#!/usr/bin/env python3
"""Deep-absorb Fable-5 / Open-SWE-Traces community datasets into NeoTrix KB."""
import sqlite3, json, time, os, hashlib

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))
NOW = int(time.time())

def ndig(s):
    return hashlib.md5(s.encode()).hexdigest()[:20]

DATASETS = [
    {
        "name": "nvidia_open_swe_traces_207k",
        "title": "NVIDIA Open SWE Traces 207K",
        "summary": "207K SWE-bench trajectories across 9 programming languages from NVIDIA. Covers real-world software engineering tasks with multi-language bug fixes and feature implementations.",
        "weight": 0.25,
        "tags": ["swe-bench", "nvidia", "multi-language", "207k"],
    },
    {
        "name": "fable5_sft_traces_kelexine_4k",
        "title": "Fable-5 Kelexine SFT Traces 4K",
        "summary": "4,665 Kelexine SFT (supervised fine-tuning) trajectories from the Fable-5 swarm. High-quality reasoning traces for code generation and software engineering tasks.",
        "weight": 0.10,
        "tags": ["fable-5", "kelexine", "sft", "reasoning"],
    },
    {
        "name": "fable5_swarm_traces_sft_4k",
        "title": "Fable-5 Swarm-AI SFT Traces 4K",
        "summary": "4,683 Swarm-AI SFT trajectories from the Fable-5 swarm. Multi-agent collaborative reasoning traces for complex problem-solving.",
        "weight": 0.08,
        "tags": ["fable-5", "swarm-ai", "sft", "multi-agent"],
    },
    {
        "name": "priming_hybrid_ssm_fable",
        "title": "Priming Hybrid SSM (Fable)",
        "summary": "Priming Hybrid SSM paper (arXiv 2605.08301). Novel state-space model architecture with priming mechanisms for enhanced long-context reasoning in code tasks.",
        "weight": 0.06,
        "tags": ["ssm", "priming", "hybrid", "arxiv", "paper"],
    },
    {
        "name": "retrieval_aware_distill_ssm",
        "title": "Retrieval-Aware Distilled SSM",
        "summary": "Retrieval-Aware Distilled SSM paper (arXiv 2602.11374). Distilled state-space model with retrieval-augmented generation for efficient code reasoning.",
        "weight": 0.05,
        "tags": ["ssm", "distillation", "retrieval", "arxiv", "paper"],
    },
    {
        "name": "open_swe_agent_thinking_dual",
        "title": "Open SWE Agent Dual-Mode Thinking",
        "summary": "Open SWE Agent dual-mode thinking paper (arXiv 2606.16038). Dual-mode reasoning approach alternating between fast and slow thinking patterns for SWE-bench tasks.",
        "weight": 0.15,
        "tags": ["swe-agent", "dual-mode", "reasoning", "arxiv", "paper"],
    },
]

def main():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()

    c.execute("BEGIN TRANSACTION")

    # Create parent hub node
    hub_id = "community_e8_datasets_hub"
    hub_meta = json.dumps({
        "source": "fable5-absorption",
        "type": "dataset-hub",
        "count": len(DATASETS),
        "domain": "community-datasets",
        "quality_score": 0.95,
    })
    c.execute("""INSERT OR IGNORE INTO nodes
        (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata)
        VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)""",
        (hub_id, "Concept", "E8 Community Datasets",
         f"Fable-5 / Open-SWE-Traces community datasets ({len(DATASETS)} datasets). Injected by Fable-5 deep absorption.",
         "", "neotrix://community-datasets/e8", "neotrix.local", "en", 1.0, 0.95, NOW, NOW, hub_meta))
    print(f"  ✅ Hub node: {hub_id}")

    dataset_ids = {}
    for ds in DATASETS:
        ds_id = f"community_dataset_{ds['name']}"
        meta = json.dumps({
            "source": "fable5-absorption",
            "type": "community-dataset",
            "weight": ds["weight"],
            "tags": ds["tags"],
        })
        c.execute("""INSERT OR IGNORE INTO nodes
            (id, node_type, title, summary, content, url, domain, language, confidence, importance, created_at, updated_at, metadata)
            VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)""",
            (ds_id, "Concept", ds["title"], ds["summary"],
             "", f"neotrix://community-datasets/{ds['name']}", "neotrix.local", "en", 1.0, ds["weight"], NOW, NOW, meta))
        dataset_ids[ds["name"]] = ds_id
        print(f"  ✅ Dataset node: {ds_id} — {ds['title']} (weight={ds['weight']})")

    # Create edges from hub to each dataset
    for name, ds_id in dataset_ids.items():
        ds = next(d for d in DATASETS if d["name"] == name)
        eid = f"re-{ndig(f'{hub_id}{ds_id}')}"
        c.execute("""INSERT OR IGNORE INTO edges
            (id, source_id, target_id, relation_type, weight, description, created_at)
            VALUES (?,?,?,?,?,?,?)""",
            (eid, hub_id, ds_id, "contains", ds["weight"], f"E8 Community Hub → {ds['title']}", NOW))

    print(f"  ✅ Created {len(dataset_ids)} 'contains' edges from hub")

    # Create related edges between datasets that share themes
    # Group by theme: SSM papers, Fable-5 traces, SWE-bench
    ssm_papers = ["priming_hybrid_ssm_fable", "retrieval_aware_distill_ssm"]
    fable_traces = ["fable5_sft_traces_kelexine_4k", "fable5_swarm_traces_sft_4k"]
    swe_related = ["nvidia_open_swe_traces_207k", "open_swe_agent_thinking_dual"]

    related_groups = [ssm_papers, fable_traces, swe_related]
    for group in related_groups:
        for i in range(len(group)):
            for j in range(i+1, len(group)):
                src = dataset_ids[group[i]]
                tgt = dataset_ids[group[j]]
                eid = f"re-{ndig(f'{src}{tgt}')}"
                c.execute("""INSERT OR IGNORE INTO edges
                    (id, source_id, target_id, relation_type, weight, description, created_at)
                    VALUES (?,?,?,?,?,?,?)""",
                    (eid, src, tgt, "related", 0.7, f"Thematic link: {group[i]} ↔ {group[j]}", NOW))

    # Cross-theme edges with lower weight
    cross_pairs = [
        ("nvidia_open_swe_traces_207k", "fable5_sft_traces_kelexine_4k", 0.4, "SWE-bench ↔ Kelexine SFT"),
        ("nvidia_open_swe_traces_207k", "fable5_swarm_traces_sft_4k", 0.35, "SWE-bench ↔ Swarm-AI SFT"),
        ("open_swe_agent_thinking_dual", "fable5_sft_traces_kelexine_4k", 0.4, "Dual-mode ↔ Kelexine SFT"),
        ("open_swe_agent_thinking_dual", "fable5_swarm_traces_sft_4k", 0.4, "Dual-mode ↔ Swarm-AI SFT"),
        ("priming_hybrid_ssm_fable", "fable5_sft_traces_kelexine_4k", 0.3, "SSM ↔ Kelexine SFT"),
        ("retrieval_aware_distill_ssm", "fable5_swarm_traces_sft_4k", 0.3, "Distilled SSM ↔ Swarm-AI SFT"),
    ]
    for s, t, w, desc in cross_pairs:
        src = dataset_ids[s]
        tgt = dataset_ids[t]
        eid = f"re-{ndig(f'{src}{tgt}')}"
        c.execute("""INSERT OR IGNORE INTO edges
            (id, source_id, target_id, relation_type, weight, description, created_at)
            VALUES (?,?,?,?,?,?,?)""",
            (eid, src, tgt, "related", w, desc, NOW))

    conn.commit()

    # Verification
    nodes_count = c.execute("SELECT COUNT(*) FROM nodes WHERE id LIKE 'community_dataset_%'").fetchone()[0]
    edges_count = c.execute("SELECT COUNT(*) FROM edges WHERE source_id = 'community_e8_datasets_hub'").fetchone()[0]
    total_edges = c.execute("""SELECT COUNT(*) FROM edges WHERE source_id IN
        (SELECT id FROM nodes WHERE id LIKE 'community_dataset_%' OR id = 'community_e8_datasets_hub')""").fetchone()[0]

    conn.close()

    print(f"\n{'═' * 60}")
    print(f"  Fable-5 Deep Absorption Complete!")
    print(f"  Datasets injected: {nodes_count}")
    print(f"  Hub 'contains' edges: {edges_count}")
    print(f"  Total edges (hub + datasets): {total_edges}")
    print(f"{'═' * 60}")

if __name__ == "__main__":
    main()
