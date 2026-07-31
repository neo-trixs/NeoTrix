#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
NeoTrix 全库本源溯源 (Full KB Source-Core Tracing)
====================================================
为全库所有节点写入知识本源溯源字段 (knowledge_source) + 能力映射 (absorbed_capability)。

本源 = 5 道之本源 (E8 形式 / VSA 表示 / GWT 意识 / ConsciousnessTree 演化 / Reality 行动)。
复用 absorb_to_capability.py 的 map_node / map_source_core。

用法:
  python3 scripts/absorb_full_kb.py --dry-run     # 只输出分布报告
  python3 scripts/absorb_full_kb.py --apply       # 写入 KB (分批事务, 幂等)
  python3 scripts/absorb_full_kb.py --apply --limit 10000
  python3 scripts/absorb_full_kb.py --apply --types article,concept
"""

import argparse
import json
import os
import sqlite3
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import absorb_to_capability as atc

KB_PATH = os.path.expanduser("~/.neotrix/knowledge.db")
BATCH = 5000

ALL_SOURCES = ("E8", "VSA", "GWT", "ConsciousnessTree", "Reality")


def trace_node(node_type, title, content, url, meta_json, topics=None):
    """对单个节点做本源溯源 + 能力映射, 返回 (cap_meta, source_meta) 或 (None, None)."""
    cap = atc.map_node(node_type, title or '', content or '', url)
    if cap is None:
        return None, None
    core, core_domain, trace_kws = atc.map_source_core(title or '', content or '', url, node_type)
    if core is None and topics:
        topic_blob = ' '.join(topics)
        if topic_blob:
            core, core_domain, trace_kws = atc.map_source_core(topic_blob, '', '', node_type)
    if core is None:
        core, core_domain, trace_kws = atc.fallback_source(title, node_type)
    branch, capability, evidence = cap
    now = time.strftime('%Y-%m-%dT%H:%M:%S')
    cap_meta = {'branch': branch, 'capability': capability, 'evidence': evidence, 'mapped_at': now}
    source_meta = {'source_core': core, 'primary_domain': core_domain,
                   'trace_path': trace_kws, 'mapped_at': now}
    return cap_meta, source_meta


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--apply', action='store_true')
    ap.add_argument('--dry-run', action='store_true')
    ap.add_argument('--limit', type=int, default=None)
    ap.add_argument('--types', type=str, default=None,
                    help='逗号分隔 node_type 白名单, 默认全部')
    args = ap.parse_args()

    conn = sqlite3.connect(KB_PATH)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA busy_timeout=30000")
    cur = conn.cursor()

    type_filter = ""
    params = []
    if args.types:
        types = [t.strip() for t in args.types.split(',') if t.strip()]
        type_filter = f" AND node_type IN ({','.join('?' for _ in types)})"
        params.extend(types)

    cur.execute("""SELECT COUNT(*) FROM nodes WHERE 1=1 """ + type_filter, params)
    total = cur.fetchone()[0]
    if args.limit and args.limit < total:
        total = args.limit
    print(f'[trace] 全库节点: {total}', flush=True)

    dist = {}
    cap_dist = {}
    no_cap = 0
    t0 = time.time()
    done = 0
    if args.apply:
        conn.execute("BEGIN")

    cur.execute("""SELECT id, node_type, title, content, url, metadata FROM nodes
                   WHERE 1=1 """ + type_filter + """ LIMIT ?""",
                params + [args.limit or total])
    for nid, node_type, title, content, url, meta_json in cur.fetchall():
        topics = []
        if meta_json:
            try:
                md = json.loads(meta_json)
                topics = md.get('topics') or []
                if md.get('description'):
                    topics.append(md['description'])
            except Exception:
                topics = []
        cap_meta, source_meta = trace_node(node_type, title, content, url, meta_json, topics)
        if cap_meta is None:
            no_cap += 1
            done += 1
            if done % BATCH == 0:
                print(f'  ... {done}/{total} ({(time.time()-t0):.0f}s)', flush=True)
            continue
        dist[source_meta['source_core']] = dist.get(source_meta['source_core'], 0) + 1
        cap_dist[cap_meta['capability']] = cap_dist.get(cap_meta['capability'], 0) + 1
        done += 1
        if args.apply:
            # 合并已有 metadata + 新字段, 整体序列化 (避免 json_set 双重编码)
            cur.execute("SELECT metadata FROM nodes WHERE id=?", (nid,))
            row = cur.fetchone()
            meta = {}
            if row and row[0]:
                try:
                    meta = json.loads(row[0])
                except Exception:
                    meta = {}
            meta['absorbed_capability'] = cap_meta
            meta['knowledge_source'] = source_meta
            cur.execute("UPDATE nodes SET metadata=? WHERE id=?",
                        (json.dumps(meta, ensure_ascii=False), nid))
        if done % BATCH == 0:
            if args.apply:
                conn.execute("COMMIT")
                conn.execute("BEGIN")
            print(f'  ... {done}/{total} ({(time.time()-t0):.0f}s)', flush=True)

    if args.apply:
        conn.execute("COMMIT")

    print()
    print('=== 全库本源溯源报告 ===')
    for core in ALL_SOURCES:
        print(f'  {core:<18} {dist.get(core, 0):>7} ({100*dist.get(core,0)//max(total,1)}%)')
    print(f'  未映射能力: {no_cap}')
    print()
    print('=== 能力分布 (top 15) ===')
    for cap, cnt in sorted(cap_dist.items(), key=lambda x: -x[1])[:15]:
        print(f'  {cap:<12} {cnt}')
    print(f'\n[trace] done in {(time.time()-t0):.0f}s')


if __name__ == '__main__':
    main()
