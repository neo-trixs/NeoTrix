#!/usr/bin/env python3
"""AgentJev 教师标注（JEV 数据死锁破解器）.

从 cocoons 采样记忆 → sidecar(:8149) 出 boolean/choice 决策
→ conversations + calib_confidence 行（蒸馏：学教师的校准输出，
行为克隆，无需 gold 标签；Text-to-LoRA/SHINE 范式）。
产出 models/training/jev_labeled.jsonl，直供 LoRA stage-2
（WeightedTrainer 按 calib_confidence 加权）。

用法: python3 models/training/label_jev.py --n 30 [--out ..] [--port 8149]
"""
import argparse
import json
import os
import random
import sys
import urllib.request

SIDECAR = "http://127.0.0.1:8149/api/evaluate"

# kev 格式对齐（jaredpalmer/kev 训练口径）：criteria 与提问共用常量，
# 双发 conversations（自家 LoRA）+ kev state/questions/label（kev trainer）
REL_INSTRUCTIONS = "Is this statement reliable?"
REL_CRITERIA_TRUE = "Consistent with known facts, specific and checkable."
REL_CRITERIA_FALSE = "Contradicted, vague, or unverifiable."


def ask(port, state, questions, timeout=120):
    body = json.dumps({"state": state, "questions": questions}).encode()
    req = urllib.request.Request(
        f"http://127.0.0.1:{port}/api/evaluate",
        data=body,
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return json.load(r)
    except Exception as e:
        return {"_error": str(e)[:150]}


def load_memories(path, min_len=30, max_len=600):
    d = json.load(open(path, encoding="utf-8"))
    mems = []
    for cocoon in d.get("cocoons", {}).values():
        for m in cocoon.get("memories", []):
            c = (m.get("content") or "").strip()
            if min_len <= len(c) <= max_len and not c.startswith("[src:"):
                mems.append({"content": c, "domain": m.get("domain", "?")})
    return mems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=30)
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--port", type=int, default=8149)
    ap.add_argument("--out", default="models/training/jev_labeled.jsonl")
    ap.add_argument("--kev-out", default="models/training/jev_kev.jsonl",
                    help="kev 训练格式双发（state/questions/label），默认同目录")
    ap.add_argument("--cocoon", default=os.path.expanduser("~/.neotrix/crystal_core/cocoons.json"))
    args = ap.parse_args()
    random.seed(args.seed)

    mems = load_memories(args.cocoon)
    print(f"[label] pool={len(mems)}", flush=True)
    if len(mems) < 4:
        print("[label] pool too small", flush=True)
        sys.exit(1)
    picks = random.sample(mems, min(args.n, len(mems)))

    rows = []
    kev_rows = []
    for i, m in enumerate(picks):
        other = random.choice(mems)
        if other is m:
            other = random.choice(mems)
        state = m["content"][:800]
        # 防位置偏置：我方随机放 A/B
        ours_first = random.random() < 0.5
        opt_a = state[:400] if ours_first else other["content"][:400]
        opt_b = other["content"][:400] if ours_first else state[:400]
        ours_key = "A" if ours_first else "B"
        qs = [
            {
                "id": "rel",
                "type": "boolean",
                "question": REL_INSTRUCTIONS,
                "criteria": {
                    "true": REL_CRITERIA_TRUE,
                    "false": REL_CRITERIA_FALSE,
                },
            },
            {
                "id": "pick",
                "type": "choice",
                "question": f"Which statement is more reliable for {m['domain']}?",
                "options": {"A": opt_a, "B": opt_b},
            },
        ]
        resp = ask(args.port, state, qs)
        if "_error" in resp:
            print(f"[{i}] ERR {resp['_error']}", flush=True)
            continue
        try:
            ans = {a["id"]: a for a in resp["results"][0]["answers"]}
        except (KeyError, IndexError, TypeError):
            print(f"[{i}] bad shape", flush=True)
            continue
        b = ans.get("rel", {})
        bp = float(b.get("probability", 0.5))
        bv = "成立" if b.get("value", bp > 0.5) else "存疑"
        rows.append({
            "conversations": [
                {"role": "user", "content": f"陈述：{state}\n该陈述可靠吗？"},
                {"role": "assistant", "content": f"{bv}（confidence: {bp:.2f}）"},
            ],
            "calib_confidence": round(bp if b.get("value", bp > 0.5) else 1.0 - bp, 3),
            "source": "agentjev-distill",
            "domain": m["domain"],
        })
        ch = ans.get("pick", {})
        dist = ch.get("distribution", {}) or {}
        pa = float(dist.get("A", 0.5))
        winner = ch.get("value", "A" if pa >= 0.5 else "B")
        # 我方被选中的概率（位置无关）
        p_ours = pa if ours_key == "A" else 1.0 - pa
        verdict_bool = bool(b.get("value", bp > 0.5))
        kev_rows.append({
            "state": state,
            "questions": {
                "rel": {
                    "type": "noul",
                    "instructions": REL_INSTRUCTIONS,
                    "criteria": {"true": REL_CRITERIA_TRUE, "false": REL_CRITERIA_FALSE},
                    "label": verdict_bool,
                },
                "pick": {
                    "type": "choice",
                    "instructions": f"Which statement is more reliable for {m['domain']}?",
                    "criteria": {"A": opt_a, "B": opt_b},
                    "label": winner,
                },
            },
            "domain": m["domain"],
        })
        rows.append({
            "conversations": [
                {"role": "user", "content": f"A：{opt_a}\nB：{opt_b}\n哪个更可靠？"},
                {"role": "assistant", "content": f"选 {winner}（P={max(pa, 1.0 - pa):.2f}）"},
            ],
            "calib_confidence": round(max(pa, 1.0 - pa), 3),
            "source": "agentjev-distill",
            "domain": m["domain"],
        })
        print(f"[{i}] ok rel={bp:.2f} pick={winner} ours={ours_key} p_ours={p_ours:.2f}", flush=True)

    out = args.out if os.path.isabs(args.out) else os.path.join(os.getcwd(), args.out)
    mode = "a" if os.path.exists(out) else "w"
    with open(out, mode, encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"[label] wrote {len(rows)} rows -> {out}", flush=True)
    kev_out = args.kev_out if os.path.isabs(args.kev_out) else os.path.join(os.getcwd(), args.kev_out)
    kmode = "a" if os.path.exists(kev_out) else "w"
    with open(kev_out, kmode, encoding="utf-8") as f:
        for r in kev_rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"[label] wrote {len(kev_rows)} kev rows -> {kev_out}", flush=True)


if __name__ == "__main__":
    main()
