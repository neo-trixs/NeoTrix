#!/usr/bin/env python3
"""
NeoTrix Knowledge Brain 性能基准测试脚本
"""
import sqlite3, time, random, statistics, json, os
import numpy as np

DB = os.path.expanduser("~/.neotrix/knowledge.db")

def benchmark_query(name, query_fn, iterations=100):
    for _ in range(5):
        query_fn()
    
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        query_fn()
        elapsed = (time.perf_counter() - start) * 1000
        times.append(elapsed)
    
    return {
        "name": name,
        "iterations": iterations,
        "min_ms": min(times),
        "max_ms": max(times),
        "mean_ms": statistics.mean(times),
        "median_ms": statistics.median(times),
        "p95_ms": sorted(times)[int(iterations * 0.95)],
        "p99_ms": sorted(times)[int(iterations * 0.99)],
    }

def run_benchmarks():
    print("=" * 60)
    print("NeoTrix Knowledge Brain 性能基准测试")
    print("=" * 60)
    
    def fts_search():
        terms = ["system design", "architecture", "microservices", "database", "scalability", "consensus", "distributed", "cache"]
        term = random.choice(terms)
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT n.id, n.title FROM nodes n JOIN nodes_fts f ON n.rowid = f.rowid WHERE nodes_fts MATCH ? LIMIT 20", (term,))
        cur.fetchall()
        conn.close()
    
    def vector_search():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT vector FROM embeddings ORDER BY random() LIMIT 100")
        vecs = cur.fetchall()
        conn.close()
        target_vec = np.frombuffer(vecs[0][0], dtype=np.float32)
        for v, in vecs:
            v_arr = np.frombuffer(v, dtype=np.float32)
            _ = np.dot(np.frombuffer(vecs[0][0], dtype=np.float32), v_arr)
    
    def neighbor_traversal():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT id FROM nodes ORDER BY random() LIMIT 1")
        row = cur.fetchone()
        if row:
            nid = row[0]
            cur.execute("SELECT target_id FROM edges WHERE source_id = ? UNION SELECT source_id FROM edges WHERE target_id = ?", (nid, nid))
            cur.fetchall()
        conn.close()
    
    def multi_hop_reasoning():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT source_id, target_id FROM edges WHERE relation_type = 'leads_to' LIMIT 1")
        row = cur.fetchone()
        if row:
            dst = row[1]
            cur.execute("SELECT target_id FROM edges WHERE source_id = ?", (dst,))
            cur.fetchall()
        conn.close()
    
    def abstraction_filter():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        levels = ["building_block", "pattern", "architecture", "case_study"]
        level = random.choice(levels)
        cur.execute("""
            SELECT n.id, n.title FROM nodes n
            JOIN node_dimensions nd ON n.id = nd.node_id
            WHERE nd.abstraction = ?
            ORDER BY random() LIMIT 50
        """, (level,))
        cur.fetchall()
        conn.close()
    
    def gap_report_query():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT id, gap_type, severity, status FROM knowledge_gap_reports WHERE status = 'pending' ORDER BY severity DESC LIMIT 10")
        cur.fetchall()
        conn.close()
    
    def multimodal_query():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT node_id, img_hash FROM multimodal_index WHERE img_hash IS NOT NULL ORDER BY random() LIMIT 20")
        cur.fetchall()
        conn.close()
    
    def edge_aggregation():
        conn = sqlite3.connect(DB, timeout=30)
        cur = conn.cursor()
        cur.execute("SELECT relation_type, COUNT(*) as cnt FROM edges GROUP BY relation_type ORDER BY cnt DESC")
        cur.fetchall()
        conn.close()
    
    benchmarks = [
        ("FTS 全文检索", fts_search, 200),
        ("向量相似度 (100向量)", vector_search, 50),
        ("邻居遍历 (1-hop)", neighbor_traversal, 300),
        ("多跳推理 (2-hop)", multi_hop_reasoning, 200),
        ("抽象层级筛选", abstraction_filter, 200),
        ("缺口报告查询", gap_report_query, 300),
        ("多模态索引查询", multimodal_query, 300),
        ("边类型聚合统计", edge_aggregation, 100),
    ]
    
    print("=" * 60)
    print("NeoTrix Knowledge Brain 性能基准测试")
    print("=" * 60)
    
    results = []
    for name, fn, iters in benchmarks:
        print(f"\n🔬 测试: {name} ({iters} 次)...")
        r = benchmark_query(name, fn, iters)
        results.append(r)
        print(f"   min={r['min_ms']:.2f}ms  median={r['median_ms']:.2f}ms  "
              f"mean={r['mean_ms']:.2f}ms  p95={r['p95_ms']:.2f}ms  p99={r['p99_ms']:.2f}ms")
    
    print("\n" + "=" * 60)
    print("基准测试汇总")
    print("=" * 60)
    print(f"{'测试项':<30} {'median(ms)':>10} {'p95(ms)':>10} {'目标':>10} {'状态':>8}")
    print("-" * 70)
    
    targets = {
        "FTS 全文检索": 20,
        "向量相似度 (100向量)": 50,
        "邻居遍历 (1-hop)": 10,
        "多跳推理 (2-hop)": 15,
        "抽象层级筛选": 15,
        "缺口报告查询": 10,
        "多模态索引查询": 10,
        "边类型聚合统计": 50,
    }
    
    all_pass = True
    for r in results:
        target = targets.get(r['name'], 100)
        status = "✅ PASS" if r['p95_ms'] <= target else "❌ FAIL"
        if status == "❌ FAIL": all_pass = False
        print(f"{r['name']:<30} {r['median_ms']:>10.2f} {r['p95_ms']:>10.2f} {target:>10} {status:>8}")
    
    print("-" * 70)
    print(f"总体: {'✅ 全部通过' if all_pass else '❌ 存在超标项'}")
    
    with open('/tmp/benchmark_results.json', 'w') as f:
        json.dump(results, f, indent=2)

if __name__ == "__main__":
    run_benchmarks()
