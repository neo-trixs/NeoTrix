#!/usr/bin/env python3
"""Laya temperature 重 fit（D3 方法论吸收第一项）.

背景：laya-typed-decisions 自带 temperature 无效（load 时告警回退 0.5），
ECE 0.246 偏高。用我方 jev_kev.jsonl 600 行做 temperature scaling：
p_T = sigmoid(logit(p)/T)，网格搜 T 最小化 NLL，上报 ECE/Brier 前后对比。
用法: python3 models/training/laya_tempfit.py [--n 600]
产出: models/training/laya_temps.json + sessions/logs/laya_tempfit.done
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


def run_laya_all(rows):
    import laya
    agent = laya.load("models/laya-hf", device="cpu", subfolder="typed-decisions")
    out = []
    for i, r in enumerate(rows):
        qrel = r["questions"].get("rel")
        try:
            resp = agent.predict(r["state"], {"rel": {
                "type": "noul",
                "instructions": qrel.get("instructions", "Is this statement reliable?"),
                "criteria": qrel.get("criteria", {}),
            }})
            p = float(resp.get("answers", {}).get("rel", {}).get("noul"))
        except Exception as e:
            out.append({"error": str(e)[:100], "i": i})
            continue
        label = bool(qrel.get("label")) if qrel else None
        out.append({"i": i, "p": p, "label": label})
        if i % 100 == 0:
            print(f"[tempfit] {i}/{len(rows)}", flush=True)
    return out


def fit(rows):
    scored = [(r["p"], r["label"]) for r in rows
              if r.get("p") is not None and r.get("label") is not None]
    print(f"[tempfit] scored={len(scored)}", flush=True)

    def nll(ts):
        s = 0.0
        for p, y in scored:
            p = min(max(p, 1e-6), 1 - 1e-6)
            logit = math.log(p / (1 - p))
            q = 1 / (1 + math.exp(-logit / ts))
            q = min(max(q, 1e-6), 1 - 1e-6)
            s += -(math.log(q) if y else math.log(1 - q))
        return s / max(1, len(scored))

    def metrics(ts):
        accs, confs = [], []
        for p, y in scored:
            p = min(max(p, 1e-6), 1 - 1e-6)
            logit = math.log(p / (1 - p))
            q = 1 / (1 + math.exp(-logit / ts))
            pred = q >= 0.5
            accs.append(int(pred == bool(y)))
            confs.append(q if pred else 1 - q)
        return accs, confs

    best_t, best_n = 1.0, nll(1.0)
    t = 0.5
    while t <= 5.0:
        v = nll(t)
        if v < best_n:
            best_n, best_t = v, t
        t += 0.1
    # 精化 ±0.1
    for dt in (-0.09, -0.08, -0.07, -0.06, -0.05, -0.04, -0.03, -0.02, -0.01,
               0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09):
        v = nll(round(best_t + dt, 2))
        if v < best_n:
            best_n, best_t = v, round(best_t + dt, 2)
    a0, c0 = metrics(1.0)
    a1, c1 = metrics(best_t)
    rep = {
        "n": len(scored),
        "T_raw": {"nll": nll(1.0), "acc": sum(a0) / len(a0),
                  "brier": AB.brier(a0, c0), "ece": AB.ece_bin(a0, c0)},
        "T_fit": {"T": best_t, "nll": best_n,
                  "acc": sum(a1) / len(a1),
                  "brier": AB.brier(a1, c1), "ece": AB.ece_bin(a1, c1)},
    }
    return rep


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=600)
    args = ap.parse_args()
    rows = [json.loads(l) for l in open(os.path.join(HERE, "jev_kev.jsonl"), encoding="utf-8")]
    rows = rows[: args.n]
    t0 = time.time()
    preds = run_laya_all(rows)
    rep = fit(preds)
    rep["elapsed"] = round(time.time() - t0, 1)
    with open(os.path.join(HERE, "laya_temps.json"), "w", encoding="utf-8") as f:
        json.dump(rep, f, ensure_ascii=False, indent=1)
    print(json.dumps(rep, ensure_ascii=False, indent=1), flush=True)


if __name__ == "__main__":
    main()
