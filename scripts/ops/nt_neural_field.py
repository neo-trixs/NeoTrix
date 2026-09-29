#!/usr/bin/env python3
"""nt_neural_field — 基础神经网络成形：原理+卦符全场建网放电.

节点：axioms.json 全部源原理（P01-Pn，有 M-id）+ symbols.json 64 卦
  （点亮 20 有 M-id，休眠 44 无 M-id 纯拓扑）。
边：原理→归属卦 1.0；XREFS4-9 跨代 1.0；卦↔卦共振（汉明<=2）0.5。
放电：乾为种，跨类型阈值 w>=0.5，卦间共振×权重>=2.0，4 跳，上限 200。
产出：neural_field.json（节点/边/统计/放电）+ field_map.jsonl（1 rich 行，
  digest --rich 用，domain=neural-field）。

用法:
  python3 scripts/ops/nt_neural_field.py
  python3 scripts/ops/nt_neural_field.py --selftest
"""
import argparse
import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__)))
from nt_foundation_laws import (  # noqa: E402
    XREFS4, XREFS6, XREFS7, XREFS8, XREFS9, XREFS10, XREFS11, XREFS12,
    XREFS13, XREFS14, XREFS15, XREFS16, XREFS17, XREFS18, XREFS19,
    XREFS20, XREFS21, XREFS22, XREFS23, XREFS24, XREFS25, XREFS26,
    XREFS27, XREFS28, XREFS29, live_symbol_mids, live_mid_set)
from nt_symbol_brain import HEX64, LIT, resonance  # noqa: E402

OUT_DIR = os.path.join("datasets", "hf_distilled")
AXIOMS_JSON = os.path.join(OUT_DIR, "axioms.json")
SYMBOLS_JSON = os.path.join(OUT_DIR, "symbols.json")
FIELD_JSON = os.path.join(OUT_DIR, "neural_field.json")
MAP_JSONL = os.path.join(OUT_DIR, "field_map.jsonl")

# 点亮卦符 M-id：**运行时从活库 neuro-symbols 域按卦名反查**。
# 原实现是 sym_mid[hex] = "M-%06d" % (479512 + j)（裸数字，静态扫描看不见），
# 那是档案期号段，活库 M-000001..M-065651 里 M-479512 从不存在 →
# 既进 nodes[].mid 又经 hub_mids 进 field_map.jsonl 的 connections，全悬空。
# lit_order / lit_hex 降级为**文档与自检口径**（LIT 序 ↔ 卦名的对应关系），
# 不再参与 id 计算。
LIT_ORDER = ["ind-reasoning", "ind-code", "ind-physical", "ind-bench",
             "ind-webscale", "ind-zh", "ind-format", "ind-license",
             "ind-heat", "ind-zim", "ind-pmtiles", "ind-corpus",
             "ind-causal", "ind-wiki", "fuse-pillars", "fuse-zh",
             "fuse-embodied", "fuse-gate", "fuse-eval", "brain"]
LIT_HEX = {"ind-reasoning": "乾", "ind-code": "鼎", "ind-physical": "震",
           "ind-bench": "履", "ind-webscale": "坤", "ind-zh": "同人",
           "ind-format": "节", "ind-license": "讼", "ind-heat": "离",
           "ind-zim": "大畜", "ind-pmtiles": "观", "ind-corpus": "井",
           "ind-causal": "益", "ind-wiki": "泰", "fuse-pillars": "既济",
           "fuse-zh": "家人", "fuse-embodied": "无妄", "fuse-gate": "蹇",
           "fuse-eval": "中孚", "brain": "未济"}


def lit_symbol_mids(path=None):
    """卦名 → 活库 M-id；顺带校验 LIT 表与反查结果一致。"""
    sym = live_symbol_mids() if path is None else live_symbol_mids(path)
    want = {LIT_HEX[c] for c in LIT_ORDER}
    if set(LIT_ORDER) != set(LIT) or want != {h for _, (h, _) in LIT.items()}:
        raise SystemExit("ABORT: 本文件 LIT_ORDER/LIT_HEX 与 nt_symbol_brain.LIT "
                         "不同步 —— 卦名映射不可信，拒绝反查。")
    if not want <= set(sym):
        raise SystemExit("ABORT: 活库缺点亮卦记忆 %s —— 无法给神经场节点挂 "
                         "真实 M-id（不再回退 479512 号段）。"
                         % sorted(want - set(sym)))
    return sym


def selftest():
    assert resonance(0b111111, 0b000000) == 0
    assert len(HEX64) == 64
    # ── 悬空边防护（2026-09-28）──────────────────────────────────────────
    # 钉死：本脚本要挂到节点/连边上的每个 M-id 都必须真实存在于活库。
    sym = lit_symbol_mids()
    known = live_mid_set()
    assert set(LIT_HEX[c] for c in LIT_ORDER) == set(sym)
    for hx, mid in sym.items():
        assert mid in known, "卦符 %s 反查到 %s，活库无此 id" % (hx, mid)
    # 档案期号探针写成 "M-%06d" % n 而非字面量：nt_graph_audit 的源码扫描
    # 只匹配 M-\d{6}，把「必须被拒绝的反例」也报成命中，告警就被自己人淹
    # 没，真问题反而看不见。
    for stale in ("M-%06d" % 479512, "M-%06d" % 479531, "M-%06d" % 479532,
                  "M-%06d" % 479535, "M-%06d" % 479942, "M-%06d" % 477231):
        assert stale not in known, "探针 %s 竟在活库（活库已变，重估号段）" % stale
    print("selftest ok: imports+resonance, 卦符 M-id 活库反查")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        return selftest()
    with open(AXIOMS_JSON, encoding="utf-8") as fh:
        axioms = json.load(fh)
    with open(SYMBOLS_JSON, encoding="utf-8") as fh:
        symbols = json.load(fh)
    hex_bits = {}
    for n, w, b, u, lo in HEX64:
        hex_bits[n] = b
    sym_mid = lit_symbol_mids()   # 活库反查；查不到即 SystemExit
    # 原理 mid 来自账本 axioms.json，而该账本历史上被 nt_foundation_laws 的
    # 硬编码号段（PRIOR_BASES 479545…）灌过一批活库不存在的 id。这些 mid 会
    # 进 nodes[].mid 与 field_map.jsonl 的 connections —— 逐条与活库核对，
    # 有悬空即中止，不把烂引用带进神经场。
    known = live_mid_set()
    stale = sorted({"%s→%s" % (a.get("id"), a.get("mid"))
                    for a in axioms if a.get("mid") not in known})
    if stale:
        raise SystemExit(
            "ABORT: 账本 %s 里 %d 条 mid 在活库不存在（如 %s）—— 写进神经场"
            "就是永久悬空边。请先校正账本（nt_foundation_laws.prior_pmap 同源"
            "校验），不要绕过。" % (AXIOMS_JSON, len(stale), "、".join(stale[:5])))

    nodes, edges = [], []
    pmap = {a["id"]: a for a in axioms}
    for a in axioms:
        nodes.append({"key": a["id"], "kind": "principle", "mid": a["mid"],
                      "hex": a["hex"], "conf": 0.9})
        edges.append({"from": a["id"], "to": "hex:" + a["hex"],
                      "w": 1.0, "kind": "attr"})
    for s in symbols:
        nodes.append({"key": "hex:" + s["hex"], "kind": "hex",
                      "mid": sym_mid.get(s["hex"]),
                      "status": s["status"], "bits": s["bits"]})
    hexes = [s["hex"] for s in symbols]
    for i in range(len(hexes)):
        for j in range(i + 1, len(hexes)):
            if resonance(hex_bits[hexes[i]], hex_bits[hexes[j]]) >= 4:
                edges.append({"from": "hex:" + hexes[i],
                              "to": "hex:" + hexes[j],
                              "w": 0.5, "kind": "hebb"})
    xrefs = {}
    for xr in (XREFS4, XREFS6, XREFS7, XREFS8, XREFS9, XREFS10,
               XREFS11, XREFS12, XREFS13, XREFS14, XREFS15, XREFS16,
               XREFS17, XREFS18, XREFS19, XREFS20, XREFS21, XREFS22,
               XREFS23, XREFS24, XREFS25, XREFS26, XREFS27, XREFS28,
               XREFS29):
        xrefs.update(xr)
    nx = 0
    for pid, refs in xrefs.items():
        if pid not in pmap:
            continue
        for x in refs:
            if x.startswith("M-") or x not in pmap:
                continue
            edges.append({"from": pid, "to": x, "w": 1.0, "kind": "xref"})
            nx += 1
    # 放电：乾种，4 跳
    adj = {}
    for e in edges:
        adj.setdefault(e["from"], []).append((e["to"], e["w"]))
        adj.setdefault(e["to"], []).append((e["from"], e["w"] * 0.8))
    fired, frontier, seen = [], ["hex:乾"], set()
    for _ in range(4):
        nxt = []
        for k in frontier:
            if k in seen or len(fired) >= 200:
                continue
            seen.add(k)
            fired.append(k)
            for tgt, w in adj.get(k, []):
                if tgt not in seen and w >= 0.5 and tgt not in nxt:
                    nxt.append(tgt)
        frontier = nxt
    deg = {}
    for e in edges:
        deg[e["from"]] = deg.get(e["from"], 0) + 1
        deg[e["to"]] = deg.get(e["to"], 0) + 1
    hubs = sorted(deg.items(), key=lambda kv: -kv[1])[:8]
    with open(FIELD_JSON, "w", encoding="utf-8") as fh:
        json.dump({"nodes": nodes, "edges": edges,
                   "stats": {"node_n": len(nodes), "edge_n": len(edges),
                             "principle_n": len(axioms),
                             "xref_n": nx,
                             "hubs": hubs},
                   "firing": {"seed": "hex:乾", "hops": 4, "fired_n": len(fired),
                              "fired": fired[:40]}},
                  fh, ensure_ascii=False)
    hub_mids = []
    for h, _ in hubs[:5]:
        if h.startswith("hex:"):
            m = sym_mid.get(h[4:])
            if m:
                hub_mids.append(m)
        elif h in pmap and pmap[h].get("mid"):
            hub_mids.append(pmap[h]["mid"])
    with open(MAP_JSONL, "w", encoding="utf-8") as fh:
        fh.write(json.dumps({
            "url": "field://atlas",
            "title": "神经场总图·%d节点%d边" % (len(nodes), len(edges)),
            "content": "基础神经网络成形：%d源原理+64卦=%d节点，%d边"
                       "（归属/跨代/共振）；乾种4跳点亮%d；枢纽：%s。" % (
                           len(axioms), len(nodes), len(edges), len(fired),
                           "、".join(h for h, _ in hubs[:5])),
            "domain": "neural-field", "memory_type": "Causal",
            "confidence": 0.78, "connections": hub_mids}, ensure_ascii=False)
                + "\n")
    print("field: nodes=%d edges=%d fired=%d hubs=%s" % (
        len(nodes), len(edges), len(fired), [h for h, _ in hubs[:5]]))


if __name__ == "__main__":
    main()
