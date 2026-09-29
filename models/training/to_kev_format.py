#!/usr/bin/env python3
"""jev_labeled.jsonl（conversations）→ jev_kev.jsonl（kev state/questions/label）.

幂等：已有 id 不重写。用于把已标注 600 行对齐 jaredpalmer/kev 训练口径，
不必重跑 sidecar 标注。
用法: python3 models/training/to_kev_format.py
"""
import json
import os
import re

REL_INSTRUCTIONS = "Is this statement reliable?"
REL_CRITERIA_TRUE = "Consistent with known facts, specific and checkable."
REL_CRITERIA_FALSE = "Contradicted, vague, or unverifiable."

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "jev_labeled.jsonl")
DST = os.path.join(HERE, "jev_kev.jsonl")


def parse_reliability(convs):
    """从 '成立（confidence: 0.61）' / '不成立…' 提取 bool + conf。"""
    for t in convs:
        if t.get("role") != "assistant":
            continue
        c = t.get("content", "")
        conf = None
        m = re.search(r"confidence[:：]\s*([0-9.]+)", c)
        if m:
            conf = float(m.group(1))
        yes = "成立" in c and "不成立" not in c
        if conf is None:
            return yes, 0.5
        return yes, conf
    return None, None


def parse_statement(convs):
    for t in convs:
        if t.get("role") == "user":
            c = t.get("content", "")
            m = re.search(r"陈述：(.+?)(?:\n|该陈述|$)", c, re.S)
            if m:
                return m.group(1).strip()[:800]
            return c.split("\n")[0][:800]
    return ""


def parse_choice_row(r):
    """choice 行：conversations 里 pick 段（如有）。目前 600 行以 rel 为主，
    pick 缺失时用 calib_confidence>0.5 视作 'A'（我方陈述）。"""
    convs = r.get("conversations", [])
    for t in convs:
        if t.get("role") == "user" and "哪条" in t.get("content", ""):
            m = re.search(r"甲：(.+?)\n乙：(.+?)\n", t["content"], re.S)
            if m:
                return m.group(1).strip()[:400], m.group(2).strip()[:400]
    return None, None


def main():
    done = set()
    if os.path.exists(DST):
        for line in open(DST, encoding="utf-8"):
            try:
                done.add(json.loads(line).get("id", ""))
            except Exception:
                pass

    n_new = 0
    with open(DST, "a", encoding="utf-8") as out:
        for i, line in enumerate(open(SRC, encoding="utf-8")):
            rid = f"lab-{i}"
            if rid in done:
                continue
            r = json.loads(line)
            convs = r.get("conversations", [])
            state = parse_statement(convs)
            yes, conf = parse_reliability(convs)
            if yes is None:
                continue
            a, b = parse_choice_row(r)
            questions = {
                "rel": {
                    "type": "noul",
                    "instructions": REL_INSTRUCTIONS,
                    "criteria": {"true": REL_CRITERIA_TRUE, "false": REL_CRITERIA_FALSE},
                    "label": yes,
                }
            }
            if a and b:
                questions["pick"] = {
                    "type": "choice",
                    "instructions": f"Which statement is more reliable for {r.get('domain', '?')}?",
                    "criteria": {"A": a, "B": b},
                    "label": "A",
                }
            out.write(json.dumps({
                "id": rid,
                "state": state,
                "questions": questions,
                "domain": r.get("domain", "?"),
                "calib_confidence": conf,
            }, ensure_ascii=False) + "\n")
            n_new += 1

    total = sum(1 for _ in open(DST, encoding="utf-8"))
    print(f"[to_kev] +{n_new} rows, total {total} -> {DST}", flush=True)


if __name__ == "__main__":
    main()
