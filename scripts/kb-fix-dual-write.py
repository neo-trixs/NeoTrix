#!/usr/bin/env python3
"""KB Fix: Eliminate dual-write anti-pattern.

Phase 1: Backfill 4047 orphan records from knowledge_nodes → nodes
Phase 2: Update 3 Python scripts to stop writing to knowledge_nodes/knowledge_edges

This is a one-time migration. After this, all writes go only to `nodes` and `edges`.
"""
import sqlite3, time, os

KB = os.environ.get("KB_PATH", os.path.expanduser("~/.neotrix/knowledge.db"))

def sql_retry(c, sql, params, max_retries=3):
    for i in range(max_retries):
        try:
            return c.execute(sql, params)
        except sqlite3.OperationalError as e:
            if "locked" in str(e) and i < max_retries - 1:
                time.sleep(0.5)
                continue
            raise

def main():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    c = conn.cursor()

    # ── Phase 1: Backfill knowledge_nodes → nodes ──
    print("═══ Phase 1: Backfill knowledge_nodes → nodes ═══")
    orphans = c.execute("""
        SELECT k.id, k.node_type, k.title, k.summary, k.url, k.domain,
               k.language, k.confidence, k.importance, k.created_at, k.updated_at, k.metadata
        FROM knowledge_nodes k
        WHERE NOT EXISTS (SELECT 1 FROM nodes n WHERE n.id = k.id)
    """).fetchall()

    print(f"  Found {len(orphans)} orphan records in knowledge_nodes")
    inserted = 0
    for row in orphans:
        (kid, ntype, title, summary, url, domain,
         lang, conf, imp, created, updated, meta_s) = row
        try:
            sql_retry(c, """INSERT OR IGNORE INTO nodes
                (id, node_type, title, summary, content, url, domain,
                 language, confidence, importance, created_at, updated_at, metadata)
                VALUES (?, ?, ?, ?, '', ?, ?, ?, ?, ?, ?, ?, ?)""",
                (kid, ntype, title or "", summary or "",
                 url or "", domain or "", lang or "en",
                 conf or 1.0, imp or 0.5, created, updated, meta_s or "{}"))
            inserted += 1
        except Exception as e:
            print(f"    ⚠️  Failed to insert {kid}: {e}")

    conn.commit()

    # ── Phase 2: Backfill knowledge_edges → edges ──
    orphan_edges = c.execute("""
        SELECT ke.id, ke.source_id, ke.target_id, ke.relation_type,
               ke.weight, ke.description, ke.created_at
        FROM knowledge_edges ke
        WHERE NOT EXISTS (SELECT 1 FROM edges e WHERE e.id = ke.id)
    """).fetchall()
    e_inserted = 0
    for row in orphan_edges:
        (eid, src, tgt, rtype, weight, desc, created) = row
        try:
            sql_retry(c, """INSERT OR IGNORE INTO edges
                (id, source_id, target_id, relation_type, weight, description, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?)""",
                (eid, src, tgt, rtype, weight or 1.0, desc or "", created))
            e_inserted += 1
        except Exception as e:
            print(f"    ⚠️  Failed to insert edge {eid}: {e}")
    conn.commit()

    # ── Final stats ──
    total_nodes = c.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]
    kn_count = c.execute("SELECT COUNT(*) FROM knowledge_nodes").fetchone()[0]
    total_edges = c.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
    ke_count = c.execute("SELECT COUNT(*) FROM knowledge_edges").fetchone()[0]

    conn.close()
    print(f"\n{'═' * 60}")
    print(f"  Phase 1 complete: {inserted}/{len(orphans)} nodes backfilled")
    print(f"  Phase 2 complete: {e_inserted}/{len(orphan_edges)} edges backfilled")
    print(f"  nodes table: {total_nodes}")
    print(f"  knowledge_nodes table: {kn_count} (still exists, scripts will stop writing)")
    print(f"  edges table: {total_edges}")
    print(f"  knowledge_edges table: {ke_count}")
    print(f"{'═' * 60}")

if __name__ == "__main__":
    main()
