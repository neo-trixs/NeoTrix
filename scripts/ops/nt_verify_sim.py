#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""nt_verify_sim — 前提选择器 + verify() 的 Python 预演台（零构建，用于门开前定量）.

## 为什么需要它

Rust 侧 `nt_premise_selector.rs`（14 测试）与 `verify` 的 D8/D9 修复（11 测试）
**在内存门关闭期间无法编译**。而其中两件事**不能靠"编译通过"来判断对错**：

1. **选择器在真实 6.4 万条语料上到底选不选得出东西。**
   合成夹具绿 ≠ 真实语料可用。若真实语料里合格锚点太少 → 每轮降级
   → `proposed == 0` 静默停摆，而**编译与单测都不会报**。

2. **D8/D9 改了分数分布，`GOLD_FLOOR = 0.7` 需要重标定。**
   0.7 是在**旧分布**（novelty=fresh/unique、bridge=常数 0.2）下标的。
   修复后若绝大多数链分数掉到 0.7 以下 → 判分器会"全拒"，
   表现为 `chosen == 0`，看起来像"推理变难了"，实际是**阈值失配**。
   这是必须**先量**才知道的事，不能等编译完才发现。

故本脚本把两者**按源码逐行忠实移植**到 Python，在**活库真实数据**上跑，
给出 RFC §4 要求的四项指标。**它是预测器，不是验收器** —— 真值仍以
Rust 活库测试为准（本脚本与其数字对不上就说明有一边移植错了）。

## 移植保真约定

常量与 Rust 一一对应，见各节。**任何一侧改了都要同步**：
  DEFAULT_PER_ROUND_CAP = 3   # 2026-09-28 实测由 5 降 3：搭便车率 46.8%→34.5%      CANONICAL_ANCHOR_IN_DEGREE = 8
  MEGA_HUB_MAX_IN_DEGREE = 400   DEFAULT_ANCHOR_WINDOW = 4
  ZERO_OVERLAP_DAMPEN = 0.7      BRIDGE_MAX = 0.2      BRIDGE_SCAN_CAP = 32
  GOLD_FLOOR = 0.7               （n_jev_calibration.rs:22）

用法:
  python3 scripts/ops/nt_verify_sim.py
  python3 scripts/ops/nt_verify_sim.py --selftest
  python3 scripts/ops/nt_verify_sim.py --json
"""
import argparse
import collections
import json
import os
import re
import sys

COCOONS = os.path.expanduser("~/.neotrix/crystal_core/cocoons.json")
# ---- 与 Rust 逐字对应的常量 ----
DEFAULT_PER_ROUND_CAP = 3   # 2026-09-28 实测由 5 降 3：搭便车率 46.8%→34.5%
CANONICAL_ANCHOR_IN_DEGREE = 8
MEGA_HUB_MAX_IN_DEGREE = 400
DEFAULT_ANCHOR_WINDOW = 4
ZERO_OVERLAP_DAMPEN = 0.7
BRIDGE_MAX = 0.2
NOVELTY_GROUND_PEAK = 0.78125   # 0.5*novelty+0.5*coverage 的理论峰值 @ t=0.375
BASE_WEIGHT = 0.8               # base 份额，其余留给 bridge（和恰为 1.0）
BRIDGE_SCAN_CAP = 32
GOLD_FLOOR = 0.7
KEYWORDS_SPLIT = re.compile(r"[\s，。、；：？！…—·,. ;:?!()（）「」『』\"'【】《》]")


def keywords(text):
    """对齐 CrystalConsciousness::keywords：切分 + 去掉长度 1 的词 + 停用词。"""
    return [w for w in KEYWORDS_SPLIT.split(text or "") if len(w) > 1]


class Graph(object):
    def __init__(self, mem):
        self.mem = mem
        self.out = {}
        inn = collections.defaultdict(list)
        for i, m in mem.items():
            tg = sorted({t for t in m["connections"] if t in mem})
            if tg:
                self.out[i] = tg
                for t in tg:
                    inn[t].append(i)
        self.inn = {k: sorted(v) for k, v in inn.items()}

    def in_deg(self, i):
        return len(self.inn.get(i, []))

    def out_deg(self, i):
        return len(self.out.get(i, []))

    def is_anchor(self, i):
        d = self.in_deg(i)
        return CANONICAL_ANCHOR_IN_DEGREE <= d <= MEGA_HUB_MAX_IN_DEGREE

    def anchors(self, domain=None):
        out = []
        for i in self.mem:
            if not self.is_anchor(i):
                continue
            if domain is not None and self.mem[i]["domain"] != domain:
                continue
            out.append((self.in_deg(i), i))
        out.sort(key=lambda t: (t[0], t[1]))   # in-degree 升序 = 特异度降序
        return [i for _d, i in out]

    def facet_cluster(self, anchor, cap):
        """对齐 Adjacency::facet_cluster：域分桶 → 域内按「谁指向我」的入邻域
        升序（专属优先）→ 域名排序 → 每域取 1 的轮转。"""
        buckets = collections.OrderedDict()
        for pid in self.inn.get(anchor, []):
            d = self.mem[pid]["domain"]
            buckets.setdefault(d, []).append(pid)
        for d in buckets:
            buckets[d].sort(key=lambda p: (self.in_deg(p), p))
        order = sorted(buckets.keys())
        picked = []
        rnd = 0
        while True:
            before = len(picked)
            for d in order:
                lst = buckets[d]
                if rnd < len(lst):
                    picked.append(lst[rnd])
            if len(picked) == before:
                break
            rnd += 1
            if cap and len(picked) >= cap:
                break
        if cap and len(picked) > cap:
            picked = picked[:cap]
        cross = len(order) >= 2
        return picked, cross


def verify_new(g, mem, conclusion_id, premise_ids):
    """D8/D9 修复后的 verify()（移植自 nt_awaken_loop.rs）。"""
    c = mem.get(conclusion_id)
    if c is None:
        return None
    prem = [mem[p] for p in premise_ids if p in mem]
    if not prem:
        return None
    premise_keys = set()
    for m in prem:
        premise_keys.update(keywords(m["content"]))
    concl_keys = keywords(c["content"])
    seen, fresh, unique = set(), 0, 0
    for k in concl_keys:
        if k in seen:
            continue
        seen.add(k)
        unique += 1
        if k not in premise_keys:
            fresh += 1
    shared = unique - fresh
    t = 0.0 if unique == 0 else fresh / float(unique)
    novelty = 4.0 * t * (1.0 - t)                      # D8
    coverage = 1.0 - t                                 # A: 词汇接地率（非 confidence 均值）
    domains = {m["domain"] for m in prem}
    bridge = 0.0
    if len(domains) >= 2:
        linked = _pairwise_linked(g, prem)
        per_shared = sum(
            1 for m in prem if set(keywords(m["content"])) & set(concl_keys))
        bridges_all = per_shared == len(prem)
        if linked and bridges_all:
            score = 1.0
        elif linked or bridges_all:
            score = 0.5
        else:
            score = 0.0
        bridge = score * BRIDGE_MAX                     # D9
    base = (0.5 * novelty + 0.5 * coverage) / NOVELTY_GROUND_PEAK * BASE_WEIGHT
    total = base + bridge
    if shared == 0 and bridge == 0.0:
        total *= ZERO_OVERLAP_DAMPEN
    return {"novelty": novelty, "coverage": coverage, "bridge": bridge,
            "total": max(0.0, min(1.0, total)), "domains": len(domains)}


def verify_old(g, mem, conclusion_id, premise_ids):
    """旧版 verify()（novelty=fresh/unique, bridge=常数）——用于对比分布漂移。"""
    c = mem.get(conclusion_id)
    if c is None:
        return None
    prem = [mem[p] for p in premise_ids if p in mem]
    if not prem:
        return None
    premise_keys = set()
    for m in prem:
        premise_keys.update(keywords(m["content"]))
    concl_keys = keywords(c["content"])
    seen, fresh, unique = set(), 0, 0
    for k in concl_keys:
        if k in seen:
            continue
        seen.add(k)
        unique += 1
        if k not in premise_keys:
            fresh += 1
    shared = unique - fresh
    # 旧口径：novelty = t（奖励编造）、grounding = 前提 confidence 均量、
    # bridge = 常数 0.2、上界 1.2 → clamp 截顶。此函数**故意保留原样**，
    # 作为「修复前 vs 修复后」分布漂移的对照基线。
    novelty = 0.0 if unique == 0 else fresh / float(unique)
    grounding = sum(m["confidence"] for m in prem) / float(len(prem))
    domains = {m["domain"] for m in prem}
    bridge = BRIDGE_MAX if len(domains) >= 2 else 0.0
    total = 0.5 * novelty + 0.5 * grounding + bridge
    if shared == 0 and bridge == 0.0:
        total *= ZERO_OVERLAP_DAMPEN
    return {"novelty": novelty, "grounding": grounding, "bridge": bridge,
            "total": max(0.0, min(1.0, total)), "domains": len(domains)}


SNIPPET_LEN = 60          # consciousness.rs SNIPPET_LEN


def _snippet(s):
    return s if len(s) <= SNIPPET_LEN else s[:SNIPPET_LEN] + "…"


def _sim_conclusion(mem, sel):
    """按 consciousness.rs 的**真实模板**合成结论（deduce / cross_domain_reason）。

    为什么必须用真模板：早先版本用"前提关键词拼结论"，那使 t≡0（结论里每个词
    都来自前提），于是 D8 的效果在测量里**完全不可见** —— 测量工具有缺陷时，
    它给出的"新旧一致"是假象。真实模板含固定中文框架词（"跨域融合"／"由…可得"
    ／"成立"），这些词**不在**英文医学前提里 → t 落在真实区间。
    """
    doms = sorted({mem[p]["domain"] for p in sel})
    ev = " ＋ ".join(_snippet(mem[p]["content"]) for p in sel)
    if len(doms) >= 2:
        # cross_domain_reason
        shared = _shared_terms([mem[p]["content"] for p in sel])
        shared_txt = "、".join(shared[:5]) if shared else "待发现的交叉点"
        return "跨域融合(%s)：由%s交汇，得到域交叉新知——%s" % (
            " × ".join(doms), ev, shared_txt)
    shared = _shared_terms([mem[p]["content"] for p in sel])
    shared_txt = "、".join(shared[:5]) if shared else "共同前提"
    return "演绎(%s)：由%s可得，%s成立" % (doms[0] if doms else "general", ev, shared_txt)


def _shared_terms(contents):
    """对齐 shared_terms：按词频排序取前 5（此处只用于形状对齐）。"""
    freq = collections.Counter()
    for c in contents:
        for w in set(keywords(c)):
            freq[w] += 1
    return [w for w, _n in sorted(freq.items(), key=lambda kv: (-kv[1], kv[0]))]


def _pairwise_linked(g, prem):
    if len(prem) < 2:
        return False
    nbrs = []
    for m in prem:
        v = sorted({c for c in m["connections"] if c in g.mem})
        nbrs.append(set(v[:BRIDGE_SCAN_CAP]))
    ids = [m["id"] for m in prem]
    for i in range(len(prem)):
        for j in range(i + 1, len(prem)):
            if ids[j] in nbrs[i] or ids[i] in nbrs[j]:
                return True
            if nbrs[i] & nbrs[j]:
                return True
    return False


def selftest():
    # 关键词切分：长度 1 丢弃（对齐 Rust `chars().count() > 1`）
    assert keywords("a bb c") == ["bb"], keywords("a bb c")
    assert keywords("测试 内容") == ["测试", "内容"]
    # D8：纯编造 → 0；半有据 → 1；纯复述 → 0
    assert abs(4 * 1.0 * (1 - 1.0)) < 1e-12
    assert abs(4 * 0.5 * 0.5 - 1.0) < 1e-12
    assert abs(4 * 0.0 * 1.0) < 1e-12
    # facet_cluster 的域多样性：小夹具
    mem = {
        "A": {"id": "A", "domain": "d0", "content": "anchor", "connections": [],
              "confidence": 0.8},
    }
    for k, d in [("P1", "d1"), ("P2", "d1"), ("D1", "d2"), ("S1", "d3")]:
        mem[k] = {"id": k, "domain": d, "content": k, "connections": ["A"],
                  "confidence": 0.8}
    g = Graph(mem)
    assert g.in_deg("A") == 4 and g.is_anchor("A") is False, "入度 4 < 8 不算锚点"
    mem["P1"]["content"] = "alpha beta"
    mem["P2"]["content"] = "alpha beta"
    sel, cross = g.facet_cluster("A", 0)
    assert len(sel) == 4 and cross is True, (sel, cross)
    # 前 3 条应分属 3 个不同域（域多样性贪心：每域先取 1）
    first3 = {mem[p]["domain"] for p in sel[:3]}
    assert len(first3) == 3, "前 3 条应覆盖 3 个域，实得 %s" % first3
    # 确定性：重复调用结果一致
    assert g.facet_cluster("A", 0)[0] == sel
    print("selftest ok: keywords/D8-curve/anchor-gate/facet-cluster")
    return 0


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cocoons", default=COCOONS)
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--sample", type=int, default=300, help="抽多少个锚点做选前提+判分")
    args = ap.parse_args()
    if args.selftest:
        return selftest()

    d = json.load(open(args.cocoons, encoding="utf-8"))
    mem = {}
    for c in d["cocoons"].values():
        for m in c["memories"]:
            mem.setdefault(m["id"], m)
    g = Graph(mem)
    n = len(mem)
    anchors = g.anchors()
    total_in = sum(1 for i in mem if g.in_deg(i) >= CANONICAL_ANCHOR_IN_DEGREE)
    mega = [i for i in mem if g.in_deg(i) > MEGA_HUB_MAX_IN_DEGREE]
    isolated = sum(1 for i in mem if g.in_deg(i) == 0 and g.out_deg(i) == 0)
    leaf = sum(1 for i in mem if g.in_deg(i) == 0 and g.out_deg(i) > 0)

    print("=== 前提选择器 · 活库预演 ===")
    print("  记忆=%d  合格锚点=%d (入度∈[%d,%d])  入度>=8 的节点=%d  超大枢纽=%d"
          % (n, len(anchors), CANONICAL_ANCHOR_IN_DEGREE, MEGA_HUB_MAX_IN_DEGREE,
             total_in, len(mega)))
    print("  孤立=%d  纯叶子=%d" % (isolated, leaf))
    if not anchors:
        print("  >>> FAIL: 真实语料里没有合格锚点 → 每轮降级、proposed==0 静默停摆")
        return 1

    # 抽锚点跑 facet_cluster
    step = max(1, len(anchors) // max(args.sample, 1))
    picks = anchors[::step][:args.sample]
    sizes, cross_n, dom_span = [], 0, []
    for a in picks:
        sel, cross = g.facet_cluster(a, DEFAULT_PER_ROUND_CAP)
        sizes.append(len(sel))
        if cross:
            cross_n += 1
        dom_span.append(len({mem[p]["domain"] for p in sel}))
    print("  抽样 %d 个锚点：前提数 min/中位/max = %d/%d/%d"
          % (len(picks), min(sizes), sorted(sizes)[len(sizes) // 2], max(sizes)))
    print("  跨域(>=2 域)比例 = %.1f%%   域跨度中位=%d"
          % (100.0 * cross_n / len(picks), sorted(dom_span)[len(dom_span) // 2]))

    # 叶子 LeafWalk 可达性：叶子沿 out 走一跳能否落到合格锚点
    lw_hit = lw_tot = 0
    for i in list(mem)[:20000]:
        if g.in_deg(i) == 0 and g.out_deg(i) > 0:
            lw_tot += 1
            for t in g.out.get(i, []):
                if g.is_anchor(t):
                    lw_hit += 1
                    break
    if lw_tot:
        print("  叶子 LeafWalk：%d/%d (%.0f%%) 能一跳到合格锚点"
              % (lw_hit, lw_tot, 100.0 * lw_hit / lw_tot))

    # ── D8/D9 分布量测：用真实 facet 簇当前提，合成一个"结论"代理 ──
    # 真实 tick 里结论由 reason() 模板生成；这里用「簇内跨域拼接」近似，
    # 目的是量**分数分布**与 GOLD_FLOOR 的关系，而非复现 reason()。
    new_tot, old_tot, passed_new, passed_old = [], [], 0, 0
    n_chain = 0
    for a in picks:
        sel, _c = g.facet_cluster(a, DEFAULT_PER_ROUND_CAP)
        if len(sel) < 2:
            continue
        n_chain += 1
        concl = _sim_conclusion(mem, sel)
        cid = "SIM-%s" % a
        mem[cid] = {"id": cid, "domain": mem[a]["domain"], "content": concl,
                    "connections": [], "confidence": 0.7}
        sn = verify_new(g, mem, cid, sel)
        so = verify_old(g, mem, cid, sel)
        if sn:
            new_tot.append(sn["total"])
            old_tot.append(so["total"])
            if sn["total"] >= GOLD_FLOOR:
                passed_new += 1
            if so["total"] >= GOLD_FLOOR:
                passed_old += 1
        del mem[cid]
    print()
    print("=== D8/D9 分数分布（%d 条模拟链）===" % n_chain)
    if new_tot:
        def q(v, p):
            s = sorted(v)
            return s[min(len(s) - 1, int(len(s) * p))]
        print("  新版 total: min=%.3f p25=%.3f 中位=%.3f p75=%.3f max=%.3f"
              % (min(new_tot), q(new_tot, .25), q(new_tot, .5), q(new_tot, .75), max(new_tot)))
        print("  旧版 total: min=%.3f p25=%.3f 中位=%.3f p75=%.3f max=%.3f"
              % (min(old_tot), q(old_tot, .25), q(old_tot, .5), q(old_tot, .75), max(old_tot)))
        pn = 100.0 * passed_new / len(new_tot)
        po = 100.0 * passed_old / len(old_tot)
        print("  过 GOLD_FLOOR=%.2f：新版 %.1f%%   旧版 %.1f%%" % (GOLD_FLOOR, pn, po))
        # 判据必须**两端**都查：全通过（无判别力）与全拒（阈值过高）同样是失配。
        import statistics
        sd = statistics.pstdev(new_tot)
        sat = 100.0 * sum(1 for t in new_tot if t >= 0.999) / len(new_tot)
        print("  判别力：标准差=%.4f   饱和(=1.0)占比=%.1f%%" % (sd, sat))
        bad = []
        if pn < 5:
            bad.append("通过率过低(%.1f%%)→ 阈值过高" % pn)
        if pn > 95:
            bad.append("通过率过高(%.1f%%)→ 门无判别力" % pn)
        if sat > 10:
            bad.append("饱和 %.1f%% → clamp 仍在截顶，排序信息丢失" % sat)
        if sd < 0.05:
            bad.append("标准差 %.4f 过小 → 分量近乎常量" % sd)
        verdict = "OK（四项判据全过）" if not bad else ">>> 失配: " + "；".join(bad)
        print("  VERDICT: %s" % verdict)
    if args.json:
        print(json.dumps({"memories": n, "anchors": len(anchors),
                          "sampled": len(picks), "chains": n_chain,
                          "new_total_p50": (sorted(new_tot)[len(new_tot) // 2]
                                            if new_tot else None),
                          "pass_new_pct": (100.0 * passed_new / len(new_tot)
                                            if new_tot else None),
                          "pass_old_pct": (100.0 * passed_old / len(old_tot)
                                            if old_tot else None)}, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main() or 0)
