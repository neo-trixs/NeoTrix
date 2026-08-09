#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
kb_dedup.py — KB 重复节点去重（对标网文"设定库清理"）

修复设定一致性检查发现的重复定义缺陷（如 HKUDS/OpenHarness 113x 重复）。
策略：同 title 节点组保留"最佳"节点（有 summary + content 最长），
其余节点的边迁移到保留节点后删除。

用法:
  python3 scripts/kb_dedup.py --dry-run          # 预览将合并多少组/节点
  python3 scripts/kb_dedup.py --apply            # 实际执行去重
  python3 scripts/kb_dedup.py --apply --min-dup 2  # 只处理重复>=2 的组
"""

import argparse
import json
import sqlite3
import sys
import time
from pathlib import Path

KB = str(Path.home() / ".neotrix" / "knowledge.db")


def open_db():
    conn = sqlite3.connect(KB, timeout=60)
    conn.execute("PRAGMA journal_mode=WAL")
    return conn


def find_duplicate_groups(conn, min_dup):
    """找同 title 重复节点组"""
    cur = conn.cursor()
    cur.execute(
        "SELECT title, COUNT(*) c FROM nodes "
        "WHERE title IS NOT NULL AND title != '' "
        "GROUP BY title HAVING c >= ? ORDER BY c DESC",
        (min_dup,),
    )
    groups = []
    for title, count in cur.fetchall():
        cur.execute(
            "SELECT id, node_type, summary, content, url, domain, metadata, source_episode "
            "FROM nodes WHERE title = ? ORDER BY created_at",
            (title,),
        )
        nodes = cur.fetchall()
        groups.append((title, count, nodes))
    return groups


def pick_keep(nodes):
    """选择保留节点: 有 summary + content 最长者优先"""
    def score(n):
        s = 0
        if n[2]:  # summary
            s += 10
        if n[3]:  # content
            s += min(len(n[3]), 500)
        if n[6] and n[6] != "{}":  # metadata
            s += 5
        return s
    return max(nodes, key=score)


def migrate_edges(conn, keep_id, dup_ids):
    """把重复节点的边迁移到保留节点（去重）。

    策略：先删除 keep 与 dup 之间同类型的冗余边（keep 已有同类型边时，
    dup 的边是冗余的），再整体迁移，避免 UNIQUE 冲突且不误删有效边。
    """
    cur = conn.cursor()
    migrated = 0
    for dup_id in dup_ids:
        # 出边: 先删 keep 已有的同类型冗余边, 再迁移 dup→keep
        cur.execute(
            "DELETE FROM edges WHERE source_id = ? AND (target_id, relation_type) IN "
            "(SELECT target_id, relation_type FROM edges WHERE source_id = ?)",
            (keep_id, dup_id),
        )
        cur.execute(
            "UPDATE edges SET source_id = ? WHERE source_id = ? AND target_id != ?",
            (keep_id, dup_id, keep_id),
        )
        migrated += cur.rowcount
        # 入边: 先删 keep 已有的同类型冗余边, 再迁移 dup→keep
        cur.execute(
            "DELETE FROM edges WHERE target_id = ? AND (source_id, relation_type) IN "
            "(SELECT source_id, relation_type FROM edges WHERE target_id = ?)",
            (keep_id, dup_id),
        )
        cur.execute(
            "UPDATE edges SET target_id = ? WHERE target_id = ? AND source_id != ?",
            (keep_id, dup_id, keep_id),
        )
        migrated += cur.rowcount
    # 删除自环 (keep→keep)
    cur.execute(
        "DELETE FROM edges WHERE source_id = ? AND target_id = ?",
        (keep_id, keep_id),
    )
    return migrated


def delete_dups(conn, dup_ids):
    """删除重复节点（nodes + nodes_fts + embeddings 级联）"""
    cur = conn.cursor()
    for dup_id in dup_ids:
        cur.execute("DELETE FROM nodes WHERE id = ?", (dup_id,))
    return len(dup_ids)


def main():
    ap = argparse.ArgumentParser(description="KB 重复节点去重")
    ap.add_argument("--dry-run", action="store_true", help="只统计不执行")
    ap.add_argument("--apply", action="store_true", help="实际执行")
    ap.add_argument("--min-dup", type=int, default=2, help="重复阈值 (默认 2)")
    args = ap.parse_args()

    conn = open_db()
    groups = find_duplicate_groups(conn, args.min_dup)
    total_dups = sum(g[1] - 1 for g in groups)
    print(f"=== KB 重复节点去重 ===")
    print(f"  重复组: {len(groups)} 组, 可删除重复节点: {total_dups}")

    if args.dry_run or not args.apply:
        print("\n  Top 10 重复组 (dry-run):")
        for title, count, _ in groups[:10]:
            print(f"    {count}x: {title[:60]}")
        print("\n  [dry-run] 未执行。加 --apply 实际去重。")
        conn.close()
        return

    # 执行去重
    merged_groups = 0
    deleted = 0
    migrated_edges = 0
    for title, count, nodes in groups:
        keep = pick_keep(nodes)
        keep_id = keep[0]
        dup_ids = [n[0] for n in nodes if n[0] != keep_id]
        if not dup_ids:
            continue
        migrated_edges += migrate_edges(conn, keep_id, dup_ids)
        deleted += delete_dups(conn, dup_ids)
        merged_groups += 1

    conn.commit()
    # 清理悬空边（指向已删除节点的边无意义）
    cur = conn.cursor()
    cur.execute(
        "DELETE FROM edges WHERE NOT EXISTS (SELECT 1 FROM nodes n WHERE n.id = edges.source_id) "
        "OR NOT EXISTS (SELECT 1 FROM nodes n WHERE n.id = edges.target_id)"
    )
    dangling = cur.rowcount
    conn.commit()
    print(f"\n  [apply] 完成: 合并 {merged_groups} 组, 删除 {deleted} 重复节点, 迁移 {migrated_edges} 条边, 清理悬空边 {dangling} 条")
    conn.close()


if __name__ == "__main__":
    main()