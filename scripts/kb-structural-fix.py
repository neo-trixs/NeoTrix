#!/usr/bin/env python3
"""KB Structural Fix — repair ~/.neotrix/knowledge.db integrity issues.

Fixes:
  1. Broken edges (orphan source_id/target_id)
  2. Case-inconsistent node_type (→ PascalCase)
  3. Missing domain field (extract from URL)
  4. Duplicate URL merge (keep longest content)
  5. Summary report
"""

import sqlite3
import urllib.parse
import sys
import os

DB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")


def connect():
    conn = sqlite3.connect(DB_PATH)
    conn.execute("PRAGMA busy_timeout = 30000")
    conn.execute("PRAGMA journal_mode = WAL")
    conn.row_factory = sqlite3.Row
    return conn


def count_nodes(conn):
    return conn.execute("SELECT COUNT(*) FROM nodes").fetchone()[0]


def count_edges(conn):
    return conn.execute("SELECT COUNT(*) FROM edges").fetchone()[0]


def fix_broken_edges(conn):
    print("[1/5] Fixing broken edges ...")
    before = count_edges(conn)
    deleted = conn.execute(
        """DELETE FROM edges
           WHERE source_id NOT IN (SELECT id FROM nodes)
              OR target_id NOT IN (SELECT id FROM nodes)"""
    ).rowcount
    conn.commit()
    after = count_edges(conn)
    print(f"  → Deleted {deleted} broken edges ({before} → {after})")
    return deleted


def fix_case_types(conn):
    print("[2/5] Normalizing node types to PascalCase ...")
    changes = 0
    rows = conn.execute(
        "SELECT id, node_type FROM nodes WHERE node_type IS NOT NULL"
    ).fetchall()
    for row in rows:
        orig = row["node_type"]
        if not orig:
            continue
        if orig[0].islower():
            normalized = orig[0].upper() + orig[1:]
            if normalized != orig:
                conn.execute(
                    "UPDATE nodes SET node_type = ? WHERE id = ?",
                    (normalized, row["id"]),
                )
                changes += 1
    conn.commit()
    print(f"  → Normalized {changes} node types")
    return changes


def fix_missing_domains(conn):
    print("[3/5] Filling missing domain fields ...")
    filled = 0
    rows = conn.execute(
        "SELECT id, url FROM nodes WHERE domain IS NULL OR domain = ''"
    ).fetchall()
    for row in rows:
        url = row["url"]
        if url and url.strip():
            parsed = urllib.parse.urlparse(url.strip())
            domain = parsed.netloc.lower() if parsed.netloc else "unknown"
        else:
            domain = "unknown"
        conn.execute("UPDATE nodes SET domain = ? WHERE id = ?", (domain, row["id"]))
        filled += 1
    conn.commit()
    print(f"  → Filled {filled} missing domain fields")
    return filled


def fix_duplicate_urls(conn):
    print("[4/5] Merging duplicate URLs ...")
    dup_groups = conn.execute(
        """SELECT url, COUNT(*) as cnt
           FROM nodes
           WHERE url IS NOT NULL AND url != ''
           GROUP BY url
           HAVING cnt > 1"""
    ).fetchall()

    merged_count = 0
    removed_count = 0

    for group in dup_groups:
        url = group["url"]
        dup_nodes = conn.execute(
            "SELECT id, content FROM nodes WHERE url = ? ORDER BY LENGTH(content) DESC",
            (url,),
        ).fetchall()
        if len(dup_nodes) < 2:
            continue
        keep_id = dup_nodes[0]["id"]
        remove_ids = [r["id"] for r in dup_nodes[1:]]

        for rid in remove_ids:
            try:
                conn.execute(
                    """INSERT OR IGNORE INTO edges (source_id, target_id, relation_type, metadata)
                       SELECT ?, target_id, relation_type, metadata
                       FROM edges WHERE source_id = ?""",
                    (keep_id, rid),
                )
                conn.execute(
                    """INSERT OR IGNORE INTO edges (source_id, target_id, relation_type, metadata)
                       SELECT source_id, ?, relation_type, metadata
                       FROM edges WHERE target_id = ?""",
                    (keep_id, rid),
                )
                conn.execute(
                    "DELETE FROM edges WHERE source_id = ? OR target_id = ?",
                    (rid, rid),
                )
                conn.execute("DELETE FROM nodes WHERE id = ?", (rid,))
                removed_count += 1
            except sqlite3.Error as e:
                print(f"  ⚠ Error merging node {rid}: {e}")
        merged_count += 1

    conn.commit()
    print(
        f"  → Merged {merged_count} groups, removed {removed_count} duplicate nodes"
    )
    return merged_count, removed_count


def report(conn, stats):
    print("\n" + "=" * 60)
    print("KB Structural Fix — Summary Report")
    print("=" * 60)

    empty = conn.execute(
        "SELECT COUNT(*) FROM nodes WHERE content IS NULL OR content = ''"
    ).fetchone()[0]
    orphaned = conn.execute(
        """SELECT COUNT(*) FROM nodes n
           WHERE NOT EXISTS (
               SELECT 1 FROM edges e
               WHERE e.source_id = n.id OR e.target_id = n.id
           )"""
    ).fetchone()[0]

    print(f"  Nodes before:             {stats['nodes_before']}")
    print(f"  Nodes after:              {count_nodes(conn)}")
    print(f"  Edges before:             {stats['edges_before']}")
    print(f"  Edges after:              {count_edges(conn)}")
    print(f"  Broken edges deleted:     {stats['broken']}")
    print(f"  Types normalized:         {stats['types']}")
    print(f"  Domains filled:           {stats['domains']}")
    print(f"  Duplicate groups merged:  {stats['dup_groups']}")
    print(f"  Duplicate nodes removed:  {stats['dup_removed']}")
    print(f"  Empty content nodes:      {empty}")
    print(f"  Orphaned nodes:           {orphaned}")
    print("=" * 60)


def main():
    if not os.path.exists(DB_PATH):
        print(f"Error: KB not found at {DB_PATH}")
        sys.exit(1)

    conn = connect()
    stats = {
        "nodes_before": count_nodes(conn),
        "edges_before": count_edges(conn),
        "broken": 0,
        "types": 0,
        "domains": 0,
        "dup_groups": 0,
        "dup_removed": 0,
    }

    try:
        stats["broken"] = fix_broken_edges(conn)
    except Exception as e:
        print(f"  ✗ Fix 1 failed: {e}")

    try:
        stats["types"] = fix_case_types(conn)
    except Exception as e:
        print(f"  ✗ Fix 2 failed: {e}")

    try:
        stats["domains"] = fix_missing_domains(conn)
    except Exception as e:
        print(f"  ✗ Fix 3 failed: {e}")

    try:
        stats["dup_groups"], stats["dup_removed"] = fix_duplicate_urls(conn)
    except Exception as e:
        print(f"  ✗ Fix 4 failed: {e}")

    try:
        report(conn, stats)
    except Exception as e:
        print(f"  ✗ Report generation failed: {e}")

    conn.close()


if __name__ == "__main__":
    main()
