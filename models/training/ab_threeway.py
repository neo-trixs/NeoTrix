#!/usr/bin/env python3
"""三方 head-to-head：AgentJev(:8149) vs Laya-ft(本地CPU) vs kev(历史成绩复用)。

同一 80 题（seed=13 复现 A/B 采样），同一度量（ab_kev_agentjev.score_rel/
brier/ece_bin/summarize），门控铁律：赢换门，输吸方法论。
kev 不重跑（历史 ab4：acc=0.512 brier=0.307 ece=0.160 conf=0.673；AgentJev ref：
acc=0.575 brier=0.327 ece=0.211 conf=0.725），只跑 Laya 新臂 + 复测 AgentJev 现状。
用法: python3 models/training/ab_threeway.py [--n 80] [--seed 13]
产出: models/training/ab_threeway_report.md + sessions/logs/ab5.done
"""
import json
import os
import random
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import ab_kev_agentjev as AB

N_DEFAULT = 80
SEED_DEFAULT = 13
OUT = os.path.join(HERE, "ab_threeway_report.md")


def run_laya(rows):
    import laya
    print("[laya] loading models/laya-hf typed-decisions on cpu ...", flush=True)
    agent = laya.load("models/laya-hf", device="cpu", subfolder="typed-decisions")
    out = []
    t_all = 0.0
    for i, r in enumerate(rows):
        qrel = r["questions"].get("rel")
        q = {
            "type": "noul",
            "instructions": qrel.get("instructions", "Is this statement reliable?"),
            "criteria": qrel.get("criteria", {}),
        }
        t0 = time.time()
        try:
            resp = agent.predict(r["state"], {"rel": q})
            ans = resp.get("answers", {}).get("rel", {})
            p_true = ans.get("noul")
            if p_true is None:
                raise ValueError(f"no noul in {str(ans)[:100]}")
            p_true = float(p_true)
        except Exception as e:
            out.append({"error": str(e)[:120], "i": i})
            continue
        dt = time.time() - t0
        t_all += dt
        label = bool(qrel.get("label")) if qrel else None
        row = {"i": i, "p_true": p_true, "label": label,
               "domain": r.get("domain"), "dt": dt}
        if label is not None:
            ok, conf, _ = AB.score_rel(p_true, label)
            row["correct"] = ok
            row["conf"] = conf
        out.append(row)
        if i % 10 == 0:
            print(f"[laya] {i}/{len(rows)}", flush=True)
    print(f"[laya] mean_latency={t_all / max(1, len(rows)):.2f}s", flush=True)
    return out


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=N_DEFAULT)
    ap.add_argument("--seed", type=int, default=SEED_DEFAULT)
    ap.add_argument("--port", type=int, default=8149)
    args = ap.parse_args()

    rows = [json.loads(l) for l in open(os.path.join(HERE, "jev_kev.jsonl"), encoding="utf-8")]
    random.seed(args.seed)
    random.shuffle(rows)
    rows = rows[: args.n]
    print(f"[ab3] n={len(rows)} seed={args.seed}", flush=True)

    t0 = time.time()
    laya_rows = run_laya(rows)
    s_laya = AB.summarize("laya-ft", laya_rows)
    jev_rows = AB.run_agentjev(rows, args.port)
    s_jev = AB.summarize("agentjev", jev_rows)
    # kev 历史（ab4.log / report-ab.md，不重跑省机时）
    s_kev = {"acc": 0.512, "brier": 0.307, "ece": 0.160, "conf": 0.673, "n": 80}

    def cell(s, k, fmt=".3f"):
        v = s.get(k)
        return ("n/a" if v is None else format(v, fmt)) if k != "n" else str(s.get("n", 0))

    def lat(rows):
        ds = [r.get("dt", 0) for r in rows if "dt" in r]
        return f"{sum(ds)/len(ds):.2f}s" if ds else "n/a"

    table = [
        ("model", "acc", "brier", "ece", "conf", "lat"),
        ("agentjev", cell(s_jev, "acc"), cell(s_jev, "brier"), cell(s_jev, "ece"),
         cell(s_jev, "mean_conf"), "sidecar"),
        ("laya-ft", cell(s_laya, "acc"), cell(s_laya, "brier"), cell(s_laya, "ece"),
         cell(s_laya, "mean_conf"), lat(laya_rows)),
        ("kev(hist)", f"{s_kev['acc']:.3f}", f"{s_kev['brier']:.3f}",
         f"{s_kev['ece']:.3f}", f"{s_kev['conf']:.3f}", "hist"),
    ]
    lines = ["# Three-way head-to-head（同 80 题 seed=13，同度量）", "",
             f"- time={time.strftime('%F %T')} elapsed={time.time()-t0:.0f}s",
             f"- n: jev={s_jev['n']} laya={s_laya['n']} kev(hist)=80", ""]
    for r in table:
        lines.append("| " + " | ".join(r) + " |")
    la, ja = s_laya.get("acc"), s_jev.get("acc")
    if la is None or ja is None:
        verdict = (f"某臂无成绩（laya n={s_laya.get('n', 0)} jev n={s_jev.get('n', 0)}）"
                   "→ 先修链路再定门")
        dl = 0.0
    else:
        dl = la - ja
        if dl > 0.02:
            verdict = "Laya-ft 更准 → 换门（AgentJev 降为次臂）"
        elif dl < -0.02:
            verdict = "AgentJev 更准 → 门不动，吸 Laya 方法论（RLCD/temperature重fit/router）"
        else:
            verdict = "基本持平 → 看 brier/ece/延迟再定（延迟 Laya 占优则换）"
    lines += ["", f"**结论：{verdict}** (Δacc laya-jev={dl:+.3f})", ""]
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print("\n".join(lines), flush=True)


if __name__ == "__main__":
    main()
