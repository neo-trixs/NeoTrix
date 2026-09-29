#!/usr/bin/env python3
"""kev-0.8B ↔ AgentJev-0.6B 同题 A/B：域问题上谁的分布更贴教师标签.

数据：models/training/jev_kev.jsonl（state/questions/label，600 行）。
- AgentJev：sidecar HTTP :8149（boolean/choice → 分布）
- kev：本地 Checkpoint 推理（noul/choice → 分布）

指标：per-question accuracy、Brier、ECE（分箱）、mean conf、flip 意见率。
用法:
  python3 models/training/ab_kev_agentjev.py [--n 100] [--port 8149] [--skip-kev] [--skip-jev]
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
KEV_RUN = os.path.join(HERE, "..", "..", "models", "kev-08b")
DATA = os.path.join(HERE, "jev_kev.jsonl")
OUT = os.path.join(HERE, "ab_kev_agentjev_report.md")


def http_json(url, payload, timeout=30):
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return json.load(r)


def parse_jev_bool(answers):
    """AgentJev boolean answer → (p_true, hard_label)。"""
    for a in answers:
        if a.get("type") == "boolean" or "distribution" in a or "probability" in a:
            dist = a.get("distribution") or a.get("probabilities") or {}
            # 上游格式：{"TRUE": p, "FALSE": p} 或 candidates
            if not dist and "candidates" in a:
                dist = {c["text"]: c["p"] for c in a["candidates"]}
            # normalize keys
            p_true = None
            for k, v in dist.items():
                if str(k).upper() in ("TRUE", "T", "YES", "成立"):
                    p_true = float(v)
            if p_true is None and len(dist) == 2:
                # take first as TRUE by convention of sorted keys
                keys = sorted(dist.keys())
                p_true = float(dist[keys[0]])
            if p_true is None:
                continue
            return p_true, ("true" if p_true >= 0.5 else "false")
    return None, None


def parse_jev_choice(answers):
    for a in answers:
        if a.get("type") == "choice" or "options" in a or "distribution" in a:
            dist = a.get("distribution") or a.get("probabilities") or {}
            if not dist and "candidates" in a:
                dist = {c["text"]: c["p"] for c in a["candidates"]}
            if dist:
                best = max(dist, key=dist.get)
                return dist, best
    return {}, None


def ece_bin(acc, conf, n_bins=10):
    if not acc:
        return float("nan")
    bins = [[] for _ in range(n_bins)]
    for a, c in zip(acc, conf):
        b = min(n_bins - 1, int(c * n_bins))
        bins[b].append((a, c))
    ece = 0.0
    for b in bins:
        if not b:
            continue
        ma = sum(x[0] for x in b) / len(b)
        mc = sum(x[1] for x in b) / len(b)
        ece += len(b) / len(acc) * abs(ma - mc)
    return ece


def brier(acc, conf):
    # binary: (p - y)^2 where conf is p of predicted class...
    # use p of true class when available: acc=1 → conf, acc=0 → 1-conf
    return sum((c if a else 1 - c) ** 2 for a, c in zip(acc, conf)) / max(1, len(acc))


def score_rel(pred_p_true, label_true):
    """returns (correct, conf_of_predicted)"""
    pred = pred_p_true >= 0.5
    conf = pred_p_true if pred else 1 - pred_p_true
    return int(pred == bool(label_true)), conf, pred_p_true


def run_agentjev(rows, port, timeout=30):
    base = f"http://127.0.0.1:{port}"
    out = []
    for i, r in enumerate(rows):
        state = r["state"]
        qrel = r["questions"].get("rel")
        payload = {
            "state": state,
            "questions": [
                {
                    "id": "rel",
                    "type": "boolean",
                    "question": qrel.get("instructions", "Is this statement reliable?"),
                    "criteria": qrel.get("criteria", {}),
                }
            ],
        }
        # choice 若有
        qpick = r["questions"].get("pick")
        if qpick and qpick.get("criteria"):
            payload["questions"].append(
                {
                    "id": "pick",
                    "type": "choice",
                    "question": qpick.get("instructions", ""),
                    "options": qpick.get("criteria", {}),
                }
            )
        try:
            resp = http_json(f"{base}/api/evaluate", payload, timeout=timeout)
        except Exception as e:
            out.append({"error": str(e)[:120], "i": i})
            continue
        answers = []
        for res in resp.get("results", []):
            answers.extend(res.get("answers", []))
        p_true, hard = parse_jev_bool(answers)
        label = bool(qrel.get("label")) if qrel else None
        row = {"i": i, "p_true": p_true, "label": label, "domain": r.get("domain")}
        if p_true is not None and label is not None:
            ok, conf, _ = score_rel(p_true, label)
            row["correct"] = ok
            row["conf"] = conf
        out.append(row)
        if i % 25 == 0:
            print(f"[jev] {i}/{len(rows)}", flush=True)
    return out


def run_kev(rows, device=None):
    """LocalPredictor via kev.checkpoint（adapter+head）."""
    sys.path.insert(0, os.path.abspath(os.path.join(HERE, "..", "..", "thirdparty", "kev")))
    from kev.checkpoint import Checkpoint, LoadOptions
    from kev.predictors import LocalPredictor
    from kev.device import default_device

    dev = device or default_device()
    print(f"[kev] loading on {dev} ...", flush=True)
    ck = Checkpoint(os.path.abspath(KEV_RUN))
    opts = LoadOptions.from_env()
    pred = LocalPredictor(os.path.abspath(KEV_RUN), dev, opts)
    print(f"[kev] loaded T={pred.temperature:.2f}", flush=True)

    out = []
    for i, r in enumerate(rows):
        record = {"state": r["state"], "questions": {}}
        # materialize 契约：每题必须有 label + src（kev.data.materialize）
        for qid, q in r["questions"].items():
            qq = dict(q)
            qq.setdefault("src", "jev_kev")
            if "label" not in qq:
                continue
            # choice label 必须是 criteria 键名
            if qq.get("type") == "choice" and isinstance(qq.get("label"), bool):
                keys = list((qq.get("criteria") or {}).keys())
                if keys:
                    qq["label"] = keys[0]
            record["questions"][qid] = qq
        if not record["questions"]:
            out.append({"error": "no questions", "i": i})
            continue
        try:
            res = pred(record)
        except Exception as e:
            out.append({"error": f"{type(e).__name__}: {str(e)[:100]}", "i": i})
            continue
        probs = res.get("probabilities", {})
        rel = probs.get("rel", {})
        # noul keys = ["false", "true"]
        p_true = None
        for k, v in rel.items():
            if str(k).lower() == "true":
                p_true = float(v)
        if p_true is None and len(rel) == 2:
            # 兜底：按 ["false","true"] 排序不可靠，取字典序第一为 false 惯例
            keys = sorted(rel.keys())
            p_true = float(rel[keys[-1]]) if keys[-1].lower() == "true" else float(rel[keys[0]])
        label = bool(r["questions"].get("rel", {}).get("label"))
        row = {"i": i, "p_true": p_true, "label": label, "domain": r.get("domain")}
        if p_true is not None:
            ok, conf, _ = score_rel(p_true, label)
            row["correct"] = ok
            row["conf"] = conf
        out.append(row)
        if i % 25 == 0:
            print(f"[kev] {i}/{len(rows)}", flush=True)
    return out


def summarize(name, rows):
    scored = [r for r in rows if r.get("correct") is not None]
    if not scored:
        return {"name": name, "n": 0}
    accs = [r["correct"] for r in scored]
    confs = [r.get("conf", 0.5) for r in scored]
    return {
        "name": name,
        "n": len(scored),
        "acc": sum(accs) / len(accs),
        "brier": brier(accs, confs),
        "ece": ece_bin(accs, confs),
        "mean_conf": sum(confs) / len(confs),
        "errors": sum(1 for r in rows if "error" in r),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=100)
    ap.add_argument("--port", type=int, default=8149)
    ap.add_argument("--skip-kev", action="store_true")
    ap.add_argument("--skip-jev", action="store_true")
    ap.add_argument("--seed", type=int, default=13)
    ap.add_argument("--device", default="cpu", help="kev 推理设备：cpu 稳（默认），mps 慢但并行可慢毒")
    args = ap.parse_args()

    import random

    random.seed(args.seed)
    rows = [json.loads(l) for l in open(DATA, encoding="utf-8") if l.strip()]
    # 只评有 rel label 的
    rows = [r for r in rows if r.get("questions", {}).get("rel", {}).get("label") is not None]
    random.shuffle(rows)
    rows = rows[: args.n]
    print(f"[ab] n={len(rows)}", flush=True)

    jev = [] if args.skip_jev else run_agentjev(rows, args.port)
    kev = [] if args.skip_kev else run_kev(rows, device=args.device)

    s_jev = summarize("AgentJev-0.6B", jev)
    s_kev = summarize("kev-0.8B", kev)

    # pairwise agreement on overlapping scored
    agree = both = flip = 0
    for a, b in zip(jev, kev):
        if a.get("correct") is None or b.get("correct") is None:
            continue
        if a.get("p_true") is None or b.get("p_true") is None:
            continue
        both += 1
        pa = a["p_true"] >= 0.5
        pb = b["p_true"] >= 0.5
        if pa == pb:
            agree += 1
        else:
            flip += 1

    lines = [
        "# kev-0.8B ↔ AgentJev A/B",
        f"- n={len(rows)} seed={args.seed} time={time.strftime('%F %T')}",
        "",
    ]
    for s in (s_jev, s_kev):
        if s["n"] == 0:
            lines.append(f"- **{s['name']}**: skipped/error")
        else:
            lines.append(
                f"- **{s['name']}**: acc={s['acc']:.3f} brier={s['brier']:.3f} "
                f"ece={s['ece']:.3f} conf={s['mean_conf']:.3f} errors={s['errors']}/{s['n']+s['errors']}"
            )
    if both:
        lines.append(f"- pairwise: agree={agree}/{both} flip={flip} ({100*flip/max(1,both):.1f}% 意见分歧)")
    lines.append("")
    verdict = ""
    if s_jev["n"] and s_kev["n"]:
        da = s_kev["acc"] - s_jev["acc"]
        verdict = "kev 更准" if da > 0.02 else ("AgentJev 更准" if da < -0.02 else "基本持平")
        lines.append(f"**结论：{verdict}** (Δacc={da:+.3f})")
    lines.append("")
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print("\n".join(lines), flush=True)
    print(f"[ab] report -> {OUT}", flush=True)


if __name__ == "__main__":
    main()
