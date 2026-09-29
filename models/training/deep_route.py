#!/usr/bin/env python3
"""D1 深特征路由原型：目标文本 → 深层特征 → 按域成绩 + 代价选臂.

深特征（vs 字面关键词）：
  reversibility  可逆/不可逆（答错代价：act_cost wrong=3.0 / escalate=0.5）
  determinism    单真相/开放式（单真相走高 acc 臂）
  latency_budget 快/慢（Laya 0.09s vs sidecar 秒级）
  calib_need     要不要校准概率（要 → brier 最优臂）
  domain         域（router_table.json 按域成绩）
规则：高风险不可逆 → 保守（阈值臂/升级）；单真相 + 快 → Laya；其余默认 AgentJev。
用法: python3 models/training/deep_route.py --selftest
      python3 models/training/deep_route.py --text "..." --domain "..." [--fast] [--need-calib] [--irreversible]
"""
import argparse
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = os.path.join(HERE, "router_table.json")


def features(text, domain="", fast=False, need_calib=False, irreversible=False):
    t = (text or "").lower()
    q_words = ("?", "吗", "是否", "是不是", "可靠", "reliable", "which", "哪个", "是否")
    return {
        "domain": domain or "?",
        "judgment": any(w in t for w in q_words) or True,
        "reversible": not irreversible,
        "fast": fast,
        "need_calib": need_calib,
    }


def route(text, domain="", fast=False, need_calib=False, irreversible=False, table=None):
    table = table if table is not None else json.load(open(TABLE, encoding="utf-8"))["table"]
    f = features(text, domain, fast, need_calib, irreversible)
    reasons = []
    # 高风险不可逆 → 保守臂（AgentJev，阈值门）
    if irreversible:
        return {"pick": "agentjev", "reasons": reasons + ["irreversible→conservative"], "features": f}
    d = table.get(domain)
    if d and d.get("n", 0) >= 20:
        if d["laya_acc"] - d["jev_acc"] > 0.02:
            return {"pick": "laya", "reasons": reasons + [f"domain {domain} laya+{d['laya_acc']-d['jev_acc']:.2f} n={d['n']}"], "features": f}
        if d["jev_acc"] - d["laya_acc"] > 0.02:
            return {"pick": "agentjev", "reasons": reasons + [f"domain {domain} jev+{d['jev_acc']-d['laya_acc']:.2f} n={d['n']}"], "features": f}
    # 快 + 单真相 → Laya（0.09s）
    if fast:
        return {"pick": "laya", "reasons": reasons + ["latency_budget→laya 0.09s"], "features": f}
    # 要校准 → brier 最优（laya+tempfit 0.253）
    if need_calib:
        return {"pick": "laya", "reasons": reasons + ["calib_need→brier best"], "features": f}
    return {"pick": "agentjev", "reasons": reasons + ["default gate"], "features": f}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--text", default="")
    ap.add_argument("--domain", default="")
    ap.add_argument("--fast", action="store_true")
    ap.add_argument("--need-calib", action="store_true")
    ap.add_argument("--irreversible", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    args = ap.parse_args()
    if args.selftest:
        cases = [
            (dict(text="删库吗？", irreversible=True), "agentjev"),
            (dict(text="这个快吗", fast=True), "laya"),
            (dict(text="概率准吗", need_calib=True), "laya"),
            (dict(text="一般判断"), "agentjev"),
            (dict(text="x", domain="Computing___Technology_(Standard)_"), "agentjev"),
        ]
        for kw, want in cases:
            got = route(**kw)["pick"]
            assert got == want, (kw, got, want)
        print("[selftest] OK 5/5", flush=True)
        return
    print(json.dumps(route(args.text, args.domain, args.fast, args.need_calib, args.irreversible),
                     ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
