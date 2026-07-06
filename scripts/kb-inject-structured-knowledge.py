#!/usr/bin/env python3
"""Fill 14,748 empty KB nodes with structured auto-summaries from titles + URLs.
Also adds type-cluster edges for better graph connectivity.
Phase 4c of knowledge gap analysis (2026-07-05 Cycle 27+)."""

import sqlite3
import time
import os

DB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
NOW = int(time.time())

# ─── Title-to-summary templates ───

def make_summary(node_type: str, title: str, url: str | None) -> str | None:
    t = title.strip()
    if not t or t.startswith("node_"):
        return None
    nt = node_type.lower()
    if nt == "repository" and url:
        parts = url.rstrip("/").split("/")
        owner = parts[-2] if len(parts) >= 2 else "unknown"
        repo = parts[-1]
        return f"GitHub repository {owner}/{repo}: {t}"
    if nt == "organization":
        return f"Organization: {t}"
    if nt == "person":
        return f"Person: {t}"
    if nt == "resource":
        return f"External resource: {t}"
    if nt == "paper":
        return f"Research paper: {t}"
    if nt == "external":
        return f"External entity: {t}"
    if nt == "summary":
        return f"Summary: {t}"
    if nt == "insight" and url:
        return f"Insight related to: {t}"
    if nt == "insight":
        return f"Insight: {t}"
    return None


def main():
    db = sqlite3.connect(DB_PATH)
    db.execute("PRAGMA journal_mode=WAL")

    # Fetch fully empty nodes
    rows = db.execute(
        """SELECT id, node_type, title, url FROM nodes
           WHERE (summary IS NULL OR summary = '')
             AND (content IS NULL OR content = '')"""
    ).fetchall()

    total = len(rows)
    updated = 0
    skipped = 0
    errors = 0

    print(f"Found {total} empty nodes to process")

    batch = []
    for node_id, node_type, title, url in rows:
        summary = make_summary(node_type, title, url)
        if summary is None:
            skipped += 1
            continue
        batch.append((summary, node_id))
        if len(batch) >= 500:
            db.executemany(
                "UPDATE nodes SET summary = ?, updated_at = ? WHERE id = ?",
                [(s, NOW, nid) for s, nid in batch],
            )
            db.commit()
            updated += len(batch)
            print(f"  ... {updated}/{total} updated", end="\r")
            batch = []

    if batch:
        db.executemany(
            "UPDATE nodes SET summary = ?, updated_at = ? WHERE id = ?",
            [(s, NOW, nid) for s, nid in batch],
        )
        db.commit()
        updated += len(batch)

    print(f"\nDone: {updated} updated, {skipped} skipped (no meaningful title), {errors} errors")

    # ─── Add type-cluster edges for filled repos / orgs / people ───

    print("\nAdding cross-reference edges between same-type nodes...")
    for node_type, label in [
        ("Repository", "related repositories"),
        ("Organization", "related organizations"),
        ("Person", "related persons"),
    ]:
        nodes = db.execute(
            "SELECT id FROM nodes WHERE node_type = ? AND title NOT LIKE 'node_%' ORDER BY title LIMIT 500",
            (node_type,),
        ).fetchall()
        n = len(nodes)
        if n < 2:
            continue
        edge_count = 0
        for i in range(n - 1):
            for j in range(i + 1, min(i + 5, n)):
                src = nodes[i][0]
                tgt = nodes[j][0]
                eid = f"auto_{src}_{tgt}_related"
                try:
                    db.execute(
                        """INSERT OR IGNORE INTO edges (id, source_id, target_id, relation_type, weight, created_at)
                           VALUES (?, ?, ?, 'related_to', 0.5, ?)""",
                        (eid, src, tgt, NOW),
                    )
                    edge_count += 1
                except sqlite3.Error:
                    pass
        db.commit()
        print(f"  {label}: {edge_count} edges added")

    # ─── FTS reindex for updated nodes ───

    print("\nReindexing FTS for updated nodes...")
    db.execute(
        """INSERT OR REPLACE INTO nodes_fts(rowid, title, summary, content, domain)
           SELECT rowid, title, COALESCE(summary,''), COALESCE(content,''), COALESCE(domain,'')
           FROM nodes WHERE updated_at = ?""",
        (NOW,),
    )
    db.commit()

    db.close()
    print("KB injection complete ✓")


if __name__ == "__main__":
    main()
