#!/usr/bin/env python3
"""AgentJev 按域 Platt 分桶拟合（RLCD 方法论吸收第二项）.

背景：Laya 教训——temperature 必须按域重 fit，不可照搬（T=5.45 vs 对方 1.98）。
我方 Rust 侧已按 arXiv 2608.05064 用 Platt（斜率+偏置）替代 temperature
（`nt_jev_calibration.rs` PlattParams::fit，坐标下降最小化 NLL）。
本脚本产出按域 Platt 参数，供 Rust 侧分桶加载：
  - 跑臂：复用 ab_kev_agentjev.run_agentjev（sidecar :8149，需健康）
  - 数据：jev_kev.jsonl 全量 1130 行（全带 label + domain）
  - 规则：n>=20 的域才信（门控铁律同 router_table.json），否则回退 global
  - 拟合：逐行镜像 Rust PlattParams::fit（init a=1,b=0；step_a=0.5,step_b=0.25；
    a>=0.05；logit 截断 ±6；p 截断 1e-6；NLL 截断 1e-9）
用法: python3 models/training/jev_platts.py [--n 1130] [--port 8149]
产出: models/training/jev_calib_rows.jsonl + models/training/jev_platts.json
      + sessions/logs/jev_platts.done（门控脚本写）
"""
import argparse
import json
import math
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import ab_kev_agentjev as AB

N_FLOOR = 20
ITERS = 200


def platt_apply(p, a, b):
    pc = min(max(p, 1e-6), 1.0 - 1e-6)
    logit = max(min(math.log(pc / (1.0 - pc)), 6.0), -6.0)
    z = a * logit + b
    return 1.0 / (1.0 + math.exp(-z))


def platt_nll(items, a, b):
    s = 0.0
    for p, y in items:
        q = min(max(platt_apply(p, a, b), 1e-9), 1.0 - 1e-9)
        s += -math.log(q) if y else -math.log(1.0 - q)
    return s / max(1, len(items))


def platt_fit(items, iters=ITERS):
    """逐行镜像 Rust PlattParams::fit."""
    if not items:
        return {"a": 1.0, "b": 0.0, "nll": float("inf")}
    ba, bb = 1.0, 0.0
    best = platt_nll(items, ba, bb)
    sa, sb = 0.5, 0.25
    for _ in range(max(1, iters)):
        improved = False
        for da, db in ((sa, 0.0), (-sa, 0.0), (0.0, sb), (0.0, -sb)):
            ca, cb = max(ba + da, 0.05), bb + db
            v = platt_nll(items, ca, cb)
            if v < best:
                ba, bb, best = ca, cb, v
                improved = True
        if not improved:
            sa *= 0.5
            sb *= 0.5
        if sa < 1e-4 and sb < 1e-4:
            break
    return {"a": ba, "b": bb, "nll": best}


def metrics(items, a=1.0, b=0.0):
    accs, confs = [], []
    for p, y in items:
        q = platt_apply(p, a, b)
        pred = q >= 0.5
        accs.append(int(pred == bool(y)))
        confs.append(q if pred else 1.0 - q)
    return {
        "n": len(items),
        "acc": sum(accs) / len(accs),
        "brier": AB.brier(accs, confs),
        "ece": AB.ece_bin(accs, confs),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=1130)
    ap.add_argument("--port", type=int, default=8149)
    args = ap.parse_args()
    rows = [json.loads(l) for l in open(os.path.join(HERE, "jev_kev.jsonl"), encoding="utf-8")]
    rows = rows[: args.n]
    t0 = time.time()
    got = AB.run_agentjev(rows, args.port)
    with open(os.path.join(HERE, "jev_calib_rows.jsonl"), "w", encoding="utf-8") as f:
        for r in got:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    scored = [(r["p_true"], bool(r["label"])) for r in got
              if r.get("p_true") is not None and r.get("label") is not None]
    by_dom = {}
    for r in got:
        if r.get("p_true") is None or r.get("label") is None:
            continue
        by_dom.setdefault(r.get("domain") or "unknown", []).append(
            (r["p_true"], bool(r["label"])))
    rep = {"n_scored": len(scored), "global": {}, "buckets": {}, "fallback": []}
    g = platt_fit(scored)
    rep["global"] = {"a": g["a"], "b": g["b"], "raw": metrics(scored),
                     "fit": metrics(scored, g["a"], g["b"])}
    for dom, items in sorted(by_dom.items()):
        if len(items) < N_FLOOR:
            rep["fallback"].append({"domain": dom, "n": len(items)})
            continue
        f = platt_fit(items)
        rep["buckets"][dom] = {"a": f["a"], "b": f["b"], "n": len(items),
                               "raw": metrics(items), "fit": metrics(items, f["a"], f["b"])}
    rep["elapsed"] = round(time.time() - t0, 1)
    with open(os.path.join(HERE, "jev_platts.json"), "w", encoding="utf-8") as f:
        json.dump(rep, f, ensure_ascii=False, indent=1)
    print(json.dumps({k: (v if k != "buckets" else f"{len(v)} domains")
                      for k, v in rep.items()}, ensure_ascii=False, indent=1), flush=True)


if __name__ == "__main__":
    main()
