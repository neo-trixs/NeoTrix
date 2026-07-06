#!/usr/bin/env python3
"""
Generate comprehensive evolution todo list from KB deep analysis + auto-absorb meta-cognition.

Usage:
  python3 scripts/generate-evolution-todo.py              # full analysis + todo
  python3 scripts/generate-evolution-todo.py --print-only  # just print, no DB writes
  python3 scripts/generate-evolution-todo.py --store-only  # just store to KB, no print
"""

import sqlite3, json, os, sys, time

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
NOW = int(time.time())

def get_db():
    db = sqlite3.connect(KB_PATH, timeout=60)
    db.execute("PRAGMA busy_timeout=30000")
    return db

def safe_fetchone(db, sql, params=None):
    try:
        c = db.execute(sql, params or [])
        return c.fetchone()
    except Exception:
        return None

def safe_fetchall(db, sql, params=None):
    try:
        c = db.execute(sql, params or [])
        return c.fetchall()
    except Exception:
        return []

def analyze(db):
    """Run deep analysis and return defect list."""
    defects = []

    # ── P0: Life-threatening defects ──

    # 1. Empty content
    empty = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content = ''")[0]
    total = safe_fetchone(db, "SELECT COUNT(*) FROM nodes")[0]
    if empty > 0:
        defects.append({
            "priority": "P0", "severity": 0.9,
            "title": f"{empty}/{total} nodes empty ({empty*100//total}%) — bulk content fill pipeline needed",
            "area": "kb-content",
            "detail": f"Node types: Insight({_count_type(db,'Insight',empty=True)}), Article({_count_type(db,'Article',empty=True)}), Concept({_count_type(db,'Concept',empty=True)}), Repository({_count_type(db,'Repository',empty=True)})"
        })

    # 2. Broken edges
    broken_src = safe_fetchone(db, "SELECT COUNT(*) FROM edges e LEFT JOIN nodes n ON e.source_id = n.id WHERE n.id IS NULL")[0]
    broken_tgt = safe_fetchone(db, "SELECT COUNT(*) FROM edges e LEFT JOIN nodes n ON e.target_id = n.id WHERE n.id IS NULL")[0]
    if broken_src + broken_tgt > 0:
        defects.append({
            "priority": "P0", "severity": 0.85,
            "title": f"{broken_src + broken_tgt} broken edges ({broken_src} source, {broken_tgt} target missing) — integrity crisis",
            "area": "kb-integrity",
            "detail": "Edges reference nodes that no longer exist. Likely from dual-write cleanup without cascade delete."
        })

    # 3. Zero embeddings
    emb_count = safe_fetchone(db, "SELECT COUNT(*) FROM embeddings")[0]
    if emb_count == 0:
        defects.append({
            "priority": "P0", "severity": 0.8,
            "title": "0 embeddings — semantic search dead, RAG pipeline non-functional",
            "area": "kb-embedding",
            "detail": "Must run kb-generate-embeddings.py with NEOTRIX_EMBEDDING_API_KEY set"
        })

    # 4. Repository metadata quality
    repos_no_meta = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type='Repository' AND (metadata IS NULL OR metadata='{}' OR metadata='')")[0]
    total_repos = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type='Repository'")[0]
    if repos_no_meta > 0:
        defects.append({
            "priority": "P0", "severity": 0.75,
            "title": f"{repos_no_meta}/{total_repos} repos have no metadata ({repos_no_meta*100//max(total_repos,1)}%) — useless stubs",
            "area": "kb-repo-quality",
            "detail": "Repository nodes created by scripts without fetching GitHub API for stars/language/topics"
        })

    # ── P1: Structural issues ──

    # 5. Duplicate URLs
    dup_rows = safe_fetchall(db, "SELECT url, COUNT(*) as cnt FROM nodes WHERE url != '' GROUP BY url HAVING cnt > 1")
    if dup_rows:
        total_dups = sum(r[1] for r in dup_rows)
        defects.append({
            "priority": "P1", "severity": 0.7,
            "title": f"{len(dup_rows)} duplicate URLs ({total_dups} extra copies) — waste and search noise",
            "area": "kb-dedup",
            "detail": f"Worst: {dup_rows[0][0][:80]} ({dup_rows[0][1]}x)"
        })

    # 6. Case-inconsistent node types
    case_issues = safe_fetchall(db, "SELECT node_type, COUNT(*) FROM nodes GROUP BY node_type HAVING node_type != substr(node_type, 1, 1) || substr(lower(node_type), 2)")
    if case_issues:
        defects.append({
            "priority": "P1", "severity": 0.65,
            "title": f"{len(case_issues)} lowercase-starting node types — schema violation",
            "area": "kb-schema",
            "detail": f"Types: {', '.join(f'{t}({c})' for t,c in case_issues)}"
        })

    # 7. Orphaned nodes (no edges)
    orphaned = safe_fetchone(db, "SELECT COUNT(*) FROM nodes n WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.source_id = n.id OR e.target_id = n.id)")[0]
    if orphaned > 0:
        defects.append({
            "priority": "P1", "severity": 0.6,
            "title": f"{orphaned} nodes have zero edges — disconnected knowledge islands",
            "area": "kb-connectivity",
            "detail": "Nodes exist in the DB but no relationship connects them to the rest of the graph"
        })

    # 8. No domain
    no_domain = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE domain IS NULL OR domain = ''")[0]
    if no_domain > 0:
        defects.append({
            "priority": "P1", "severity": 0.55,
            "title": f"{no_domain} nodes missing domain field — search and filtering degraded",
            "area": "kb-metadata",
            "detail": "Domain field is NULL or empty, causing domain-based queries to miss these nodes"
        })

    # 9. Crawl queue completed (no more to process)
    pending = safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='pending'")[0]
    failed = safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='failed'")[0]
    completed = safe_fetchone(db, "SELECT COUNT(*) FROM crawl_queue WHERE status='completed'")[0]
    defects.append({
        "priority": "P1", "severity": 0.65,
        "title": f"Crawl queue exhausted: {completed} done, {failed} failed, {pending} pending",
        "area": "kb-crawl",
        "detail": "Need to inject new seed URLs to continue external absorption"
    })

    # ── P2: Quality improvements ──

    # 10. Legacy dual-write tables
    legacy_nodes = safe_fetchone(db, "SELECT COUNT(*) FROM knowledge_nodes")[0]
    if legacy_nodes > 0:
        defects.append({
            "priority": "P2", "severity": 0.5,
            "title": f"{legacy_nodes} knowledge_nodes in legacy table — dual-write not fully migrated",
            "area": "kb-legacy",
            "detail": "Script pipeline writes to both nodes + knowledge_nodes; need to stop writing to legacy"
        })
    legacy_edges = safe_fetchone(db, "SELECT COUNT(*) FROM knowledge_edges")[0]
    if legacy_edges > 0:
        defects.append({
            "priority": "P2", "severity": 0.45,
            "title": f"{legacy_edges} knowledge_edges in legacy table — duplicate edge storage",
            "area": "kb-legacy",
            "detail": "Same as knowledge_nodes — dual-write to clean up"
        })

    # 11. ArXiv paper quality
    paper_empty = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type='Paper' AND (content IS NULL OR content = '')")[0]
    paper_total = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type='Paper'")[0]
    if paper_empty > 0:
        defects.append({
            "priority": "P2", "severity": 0.5,
            "title": f"{paper_empty}/{paper_total} Paper nodes empty ({paper_empty*100//max(paper_total,1)}%) — ArXiv fill pipeline",
            "area": "kb-content-paper",
            "detail": "arXiv abstract fetch failing for some papers; need better retry + HTML fallback"
        })

    # 12. Insight nodes empty
    insight_empty = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type='Insight' AND (content IS NULL OR content = '')")[0]
    if insight_empty > 0:
        defects.append({
            "priority": "P2", "severity": 0.4,
            "title": f"{insight_empty} empty Insight nodes — are these needed?",
            "area": "kb-housekeeping",
            "detail": "Insight nodes are auto-generated; may not need content, but should be documented"
        })

    # Sort by priority
    p_order = {"P0": 0, "P1": 1, "P2": 2}
    defects.sort(key=lambda d: (p_order.get(d["priority"], 99), -d["severity"]))
    return defects


def _count_type(db, node_type, empty=False):
    if empty:
        r = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type=? AND (content IS NULL OR content = '')", (node_type,))
    else:
        r = safe_fetchone(db, "SELECT COUNT(*) FROM nodes WHERE node_type=?", (node_type,))
    return r[0] if r else 0


def main():
    print_only = "--print-only" in sys.argv
    store_only = "--store-only" in sys.argv

    db = get_db()
    defects = analyze(db)

    # Store to KB
    if not print_only:
        now = int(time.time())
        for i, d in enumerate(defects):
            uuid_str = f"ev-{now:x}-{i:04x}"
            val = json.dumps({
                "title": d["title"],
                "priority": d["priority"],
                "severity": d["severity"],
                "area": d["area"],
                "detail": d.get("detail", ""),
                "ts": now,
            })
            db.execute(
                "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
                ("evolution_todo", uuid_str, val, now)
            )
        # Store aggregate
        agg = {
            "total": len(defects),
            "p0_count": sum(1 for d in defects if d["priority"] == "P0"),
            "p1_count": sum(1 for d in defects if d["priority"] == "P1"),
            "p2_count": sum(1 for d in defects if d["priority"] == "P2"),
            "areas": list(set(d["area"] for d in defects)),
            "generated_at": now,
        }
        db.execute(
            "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?, ?, ?, ?)",
            ("evolution_todo", "latest_aggregate", json.dumps(agg), now)
        )
        db.commit()

    # Print
    if not store_only:
        print()
        print("┌────────────────────────────────────────────────────────────────────────────────┐")
        print("│  NeoTrix Evolution Todo List — Full KB Deep Analysis                          │")
        print(f"│  Generated: {time.strftime('%Y-%m-%d %H:%M:%S')}                                            │")
        print("└────────────────────────────────────────────────────────────────────────────────┘")
        print()

        for i, d in enumerate(defects, 1):
            icon = {"P0": "🔴", "P1": "🟡", "P2": "🟢"}.get(d["priority"], "⚪")
            print(f"  {icon}  [{d['priority']}] #{i:2d}: {d['title']}")
            print(f"       Area: {d['area']} | Severity: {d['severity']:.2f}")
            if d.get("detail"):
                print(f"       {d['detail']}")
            print()

        print("┌────────────────────────────────────────────────────────────────────────────────┐")
        p0 = sum(1 for d in defects if d["priority"] == "P0")
        p1 = sum(1 for d in defects if d["priority"] == "P1")
        p2 = sum(1 for d in defects if d["priority"] == "P2")
        print(f"│  Total: {len(defects)} items  |  P0: {p0}  |  P1: {p1}  |  P2: {p2}  │")
        print("└────────────────────────────────────────────────────────────────────────────────┘")
        print()

    db.close()


if __name__ == "__main__":
    main()
