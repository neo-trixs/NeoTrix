#!/usr/bin/env python3
"""双臂按域路由表（D3 router 吸收）.

AgentJev(:8149) vs Laya-ft+tempfit 全量 600 行，同度量，按 domain 选优：
acc 优先，持平（|Δ|<0.02）看 brier。产出 models/training/router_table.json。
用法: python3 models/training/ab_router.py
"""
import json
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import ab_kev_agentjev as AB


def run_laya(rows, temp=5.0):
    import math
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
            p = min(max(p, 1e-6), 1 - 1e-6)
            logit = math.log(p / (1 - p))
            q = 1 / (1 + math.exp(-logit / temp))
        except Exception as e:
            out.append({"error": str(e)[:100], "i": i})
            continue
        label = bool(qrel.get("label")) if qrel else None
        row = {"i": i, "p_true": q, "label": label, "domain": r.get("domain")}
        if label is not None:
            ok, conf, _ = AB.score_rel(q, label)
            row["correct"] = ok
            row["conf"] = conf
        out.append(row)
        if i % 100 == 0:
            print(f"[router-laya] {i}/{len(rows)}", flush=True)
    return out


def main():
    rows = [json.loads(l) for l in open(os.path.join(HERE, "jev_kev.jsonl"), encoding="utf-8")]
    print(f"[router] n={len(rows)}", flush=True)
    t0 = time.time()
    laya_rows = run_laya(rows)
    jev_rows = AB.run_agentjev(rows, 8149)
    doms = sorted({r.get("domain", "?") for r in rows})
    table = {}
    for d in doms:
        lj = [r for r in laya_rows if r.get("domain") == d and r.get("correct") is not None]
        jj = [r for r in jev_rows if r.get("domain") == d and r.get("correct") is not None]
        if not lj or not jj:
            continue
        la = sum(r["correct"] for r in lj) / len(lj)
        ja = sum(r["correct"] for r in jj) / len(jj)
        lb = AB.brier([r["correct"] for r in lj], [r["conf"] for r in lj])
        jb = AB.brier([r["correct"] for r in jj], [r["conf"] for r in jj])
        if la - ja > 0.02:
            pick = "laya"
        elif ja - la > 0.02:
            pick = "agentjev"
        else:
            pick = "laya" if lb < jb else "agentjev"
        table[d] = {"pick": pick, "n": len(lj),
                    "laya_acc": round(la, 3), "jev_acc": round(ja, 3),
                    "laya_brier": round(lb, 3), "jev_brier": round(jb, 3)}
    out = {"time": time.strftime("%F %T"), "elapsed": round(time.time() - t0, 1),
           "default": "agentjev", "table": table}
    with open(os.path.join(HERE, "router_table.json"), "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
    wins = {}
    for v in table.values():
        wins[v["pick"]] = wins.get(v["pick"], 0) + 1
    print(f"[router] domains={len(table)} wins={wins} elapsed={out['elapsed']}s", flush=True)
    for d, v in table.items():
        print(f"  {d[:40]:40s} -> {v['pick']} (laya {v['laya_acc']:.2f} vs jev {v['jev_acc']:.2f}, n={v['n']})",
              flush=True)


if __name__ == "__main__":
    main()
