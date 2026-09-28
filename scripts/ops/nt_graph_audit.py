#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_graph_audit — 晶体记忆图体检（推理链底座的持续监控，零构建可跑）.

## 为什么要有这个（而不是只靠 Rust 单测）

推理链的前提选择完全依赖记忆图（入邻域 / 度数 / 跨域边 / 枢纽）。
图一旦坏掉——最典型的是 **monoculture 枢纽**（一个锚点吞掉整个域）——
链会静默变垃圾：前提同指率崩掉，而所有**记忆级**验收都发现不了
（每条记忆单独看都合法）。2026-09-28 就真出现过：`M-062005` 判分器锚点
入度 935，925 份 rubric 全指向它，跨域却彼此毫无语义关联。

本脚本把这类**图级**指标做成可随时跑的体检，与 Rust 活库测试
（`nt_graph_index::live::live_graph_index`）**互为交叉验证**：
两边独立实现同一套口径，数字对不上就说明有一边算错了。

口径对齐 Rust `Adjacency`（`nt_graph_index.rs`）：
  - 悬空目标**排除**出图结构（保留在 diagnosis 里）
  - 一律去重 + 排序 → 输出确定
  - 总度 = 出度 + 入度

用法:
  python3 scripts/ops/nt_graph_audit.py                 # 体检活库
  python3 scripts/ops/nt_graph_audit.py --json          # 机器可读
  python3 scripts/ops/nt_graph_audit.py --selftest
退出码: 0=健康 1=有 FAIL 项 2=库不可解析
"""
import argparse
import collections
import json
import os
import re
import sys

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
VALID_TYPES = {"Fact", "Pattern", "Causal", "Contradiction",
               "Counterfactual", "Experience", "Lesson", "Solution"}
MID_RE = re.compile(r"^M-\d{6}$")
# 源码里硬编码的 M-id 字面量（如 BASE_MID = 479512、evidence ["M-477231"]）
LITERAL_MID = re.compile(rb"M-(\d{6})")
CODE_EXT = (".py", ".rs")

# 门限（与 Rust 活库测试的 mega_hubs(400) 同值，两边必须一致）
MONOCULTURE_IN_DEG = 400
MIN_CROSS_DOMAIN_EDGES = 5   # Transcend 的 cross_domain 门下界代理
MAX_CONTENT_DUPES = 0


def scan_script_literals(root, live_max):
    """扫 ops 源码里硬编码的 M-id 字面量，报出**超过活库 max** 的（= 必悬挂）。

    为什么需要：2026-09-28 那 3,201 条悬挂边的源头，就是一批脚本把
    M-477231 / M-479262 / M-479512 / M-479628 之类**外来 id 空间**写死
    （`nt_hf_smelt_chains.read_base` 的 fallback、`nt_corpus_mine.BASE_MID`、
    `nt_ascend_engine` 的 evidence、`nt_web_mine` 的 mids）。那些 id 在活库
    根本不存在 → 一跑就重新注入悬空边，而**记忆级验收发现不了**（每条记忆
    单独看都合法）。故在**源码层**拦，比在数据层删更根本。
    同 nt_lock_audit.py 的思路：静态扫描、零构建、可随时跑。
    """
    hits = []
    for dirpath, _dirs, files in os.walk(root):
        for fn in sorted(files):
            if not fn.endswith(CODE_EXT):
                continue
            p = os.path.join(dirpath, fn)
            try:
                with open(p, "rb") as fh:
                    blob = fh.read()
            except OSError:
                continue
            rel = os.path.relpath(p)
            for m in LITERAL_MID.finditer(blob):
                v = int(m.group(1))
                if v <= live_max:
                    continue
                line_no = blob[:m.start()].count(b"\n") + 1
                line_start = blob.rfind(b"\n", 0, m.start()) + 1
                line_txt = blob[line_start:m.start()]
                # 跳过注释/文档字符串里的示意 id（本仓大量用 `#` 写说明），
                # 以及显式 opt-out。否则告警被噪声淹没，真问题反而看不见。
                if b"#" in line_txt:
                    continue
                if rel == os.path.relpath(__file__):
                    continue          # 审计器自身文档里就举了这些反例
                hits.append((rel, line_no, v))
    # 去重（同一 id 在同一文件多处出现只报一次），按 id 降序
    uniq = {}
    for rel, line, v in hits:
        uniq.setdefault((rel, v), line)
    return sorted(((v, rel, line) for (rel, v), line in uniq.items()), reverse=True)


def build_graph(mem):
    """mem: {id: memory_dict} → (out, inn, dangling_targets_set)"""
    out = {i: sorted({t for t in m["connections"] if t in mem}) for i, m in mem.items()}
    out = {i: v for i, v in out.items() if v}
    inn = collections.defaultdict(list)
    for i, ts in out.items():
        for t in ts:
            inn[t].append(i)
    inn = {k: sorted(set(v)) for k, v in inn.items()}
    dangling = {t for m in mem.values() for t in m["connections"] if t not in mem}
    return out, inn, dangling


def audit(path, verbose=True, scan_root=None):
    raw = open(path, encoding="utf-8").read()
    d = json.loads(raw)
    cocoons = d.get("cocoons", {})
    mem = {}
    for cid, c in cocoons.items():
        for m in c.get("memories", []):
            mem.setdefault(m["id"], m)
    out, inn, dangling = build_graph(mem)
    n = len(mem)
    live_max = max((int(i.split("-")[1]) for i in mem
                    if MID_RE.match(i)), default=0)

    indeg = {i: len(inn.get(i, [])) for i in mem}
    outdeg = {i: len(out.get(i, [])) for i in mem}
    total = {i: outdeg[i] + indeg[i] for i in mem}
    isolated = sum(1 for i in mem if total[i] == 0)
    connected = n - isolated
    cross = sum(1 for i, ts in out.items()
                for t in ts if mem[i]["domain"] != mem[t]["domain"])
    hubs = sorted(((indeg[i], i) for i in mem), reverse=True)
    dup_content = n - len({m["content"] for m in mem.values()})
    bad_type = sum(1 for m in mem.values() if m["memory_type"] not in VALID_TYPES)
    nonmid = sum(1 for m in mem.values() for t in m["connections"] if not MID_RE.match(t))
    oversized = sum(1 for c in cocoons.values()
                    if len(c["memories"]) > c["retention_policy"]["max_memories"])
    keymismatch = sum(1 for k, c in cocoons.items() if k != c["id"])
    canonical = (json.dumps(d, ensure_ascii=False, indent=2) + "\n") == raw

    fails, warns = [], []
    if n == 0:
        fails.append("库为空 —— CocoonStore::load() 可能解析失败（查非法 memory_type）")
    if bad_type:
        fails.append("非法 memory_type %d 条（会让整库 load 归零）" % bad_type)
    if dangling:
        fails.append("悬空连边目标 %d 个" % len(dangling))
    if nonmid:
        fails.append("非 M-%06d 形态的连边 %d 条" % (6, nonmid))
    if dup_content > MAX_CONTENT_DUPES:
        fails.append("内容重复 %d 条" % dup_content)
    if oversized:
        fails.append("超 retention 上限的茧 %d 个" % oversized)
    if keymismatch:
        fails.append("茧键≠体内 id %d 个" % keymismatch)
    if not canonical:
        warns.append("非规范 pretty 格式（parse→dump 无法字节还原）")
    mega = [(i, k) for k, i in hubs if k >= MONOCULTURE_IN_DEG]
    if mega:
        fails.append("monoculture 枢纽：%s" % mega[:3])
    if cross < MIN_CROSS_DOMAIN_EDGES:
        warns.append("跨域边仅 %d 条（Transcend 门需 cross_domain>=5）" % cross)

    literals = []
    if scan_root and os.path.isdir(scan_root):
        literals = scan_script_literals(scan_root, live_max)
        if literals:
            warns.append("源码含 %d 处**超过活库 max(M-%06d)** 的硬编码 M-id "
                         "（一跑即重新注入悬空边）" % (len(literals), live_max))

    if verbose:
        print("=== 晶体记忆图体检 ===")
        print("  memories=%d  unique=%d  cocoons=%d  domains=%d"
              % (n, len({m["id"] for m in mem.values()}), len(cocoons),
                 len({m["domain"] for m in mem.values()})))
        print("  edges=%d  cross_domain_edges=%d  isolated=%d  connected_ratio=%.3f"
              % (sum(len(v) for v in out.values()), cross, isolated,
                 connected / n if n else 0.0))
        print("  总度分布 top8: %s" % sorted(
            collections.Counter(total.values()).items())[:8])
        print("  最大枢纽 top5 (入度): %s" % [(i, k) for k, i in hubs[:5]])
        print("  内容重复=%d  非法类型=%d  悬空=%d  非M-id=%d  超限茧=%d  键不符=%d"
              % (dup_content, bad_type, len(dangling), nonmid, oversized, keymismatch))
        print("  规范往返字节一致=%s" % canonical)
        # 枢纽的域纯度：一个锚点的入邻居跨多少个域（跨得越多越泛化）
        top_id = hubs[0][1] if hubs else None
        if top_id:
            doms = collections.Counter(mem[i]["domain"] for i in inn.get(top_id, []))
            print("  最宽枢纽 %s 入邻域域分布=%s" % (top_id, dict(doms.most_common(4))))
        for w in warns:
            print("  WARN: %s" % w)
        for f in fails:
            print("  FAIL: %s" % f)
        if scan_root and os.path.isdir(scan_root):
            print("  --- 源码硬编码 M-id 扫描（%s, live_max=M-%06d）---"
                  % (scan_root, live_max))
            if not literals:
                print("    clean: 无超界硬编码 M-id")
            for v, rel, line in literals[:12]:
                print("    M-%06d  %s:%d" % (v, rel, line))
            if len(literals) > 12:
                print("    … 另有 %d 处" % (len(literals) - 12))
        print("  VERDICT: %s" % ("FAIL" if fails else "PASS"))
    return {"memories": n, "edges": sum(len(v) for v in out.values()),
            "cross_domain_edges": cross, "isolated": isolated,
            "connected_ratio": round(connected / n, 4) if n else 0.0,
            "max_in_degree": hubs[0][0] if hubs else 0,
            "content_dupes": dup_content, "illegal_type": bad_type,
            "dangling": len(dangling), "non_mid": nonmid,
            "oversized_cocoons": oversized, "cocoon_key_mismatch": keymismatch,
            "canonical": canonical, "live_max_mid": live_max,
            "script_literals_over_max": [{"id": "M-%06d" % v, "file": rel, "line": ln}
                                         for v, rel, ln in literals],
            "fails": fails, "warns": warns}


def selftest():
    def mk(i, dom, conns):
        return {"id": i, "domain": dom, "connections": conns, "content": i,
                "memory_type": "Fact"}
    mem = {"A": mk("A", "d1", []), "P1": mk("P1", "d2", ["A"]),
           "P2": mk("P2", "d2", ["A"]), "X": mk("X", "d3", ["GONE"])}
    out, inn, dang = build_graph(mem)
    assert set(out.keys()) == {"P1", "P2"}, out          # A/X 无有效出边
    assert inn["A"] == ["P1", "P2"], inn
    assert dang == {"GONE"}, dang                          # 悬空被排除出结构
    assert out["P1"] == ["A"]
    # 确定性
    o2, i2, _ = build_graph(dict(reversed(list(mem.items()))))
    assert o2 == out and i2 == inn
    assert MID_RE.match("M-000001") and not MID_RE.match("X")
    assert len(VALID_TYPES) == 8
    # 源码扫描：只报超界字面量
    import tempfile
    with tempfile.TemporaryDirectory() as t:
        with open(os.path.join(t, "a.py"), "wb") as fh:
            fh.write(b'X = 479512  # M-479512\nY = "M-000123"\nZ = "M-479262"\n')
        hits = scan_script_literals(t, 65651)
        # 注释里的 M-479512 不报；裸字面量 479262 报
        assert [h[0] for h in hits] == [479262], hits
        assert hits[0][2] == 3, hits
        assert scan_script_literals(t, 999999) == [], "阈值之上不应报"
    print("selftest ok: build-graph/dangling-excluded/deterministic/regex/literal-scan")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cocoons", default=COCOONS)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--scan-scripts", default="",
                    help="同时扫该目录下源码里超界的硬编码 M-id（如 scripts/ops）")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    if not os.path.exists(args.cocoons):
        print("ABORT: %s 不存在" % args.cocoons)
        return 2
    try:
        r = audit(args.cocoons, verbose=not args.json,
                  scan_root=args.scan_scripts or None)
    except ValueError as e:
        print("FAIL: 库不可解析: %s" % e)
        return 2
    if args.json:
        print(json.dumps(r, ensure_ascii=False, indent=1))
    return 1 if r["fails"] else 0


if __name__ == "__main__":
    sys.exit(main() or 0)
