#!/usr/bin/env python3
"""Teacher-A 蒸馏目标按域重校准（RLCD proper 落地：目标侧）.

背景：lora_finetune 用 Teacher-A 自判 confidence 作样本权重，但自判未经校准
（域差实锤：zim 过锐 a=2.087 vs Agriculture 贴地 a=0.05）。本脚本用
jev_platts.json（全量 1130 拟合，n>=20 成桶）对 jev_labeled.jsonl 逐条做
Platt 校准（与 Rust PlattParams::apply 逐行同式），产出校准版 SFT 数据。
未命中域桶 → global 回退。原值保留备查，零破坏。
用法: python3 models/training/recalibrate_labels.py
产出: models/training/jev_labeled_cal.jsonl（+calib_raw/calib_domain/a/b 字段）
"""
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))


def platt_apply(p, a, b):
    pc = min(max(p, 1e-6), 1.0 - 1e-6)
    logit = max(min(math.log(pc / (1.0 - pc)), 6.0), -6.0)
    return 1.0 / (1.0 + math.exp(-(a * logit + b)))


def main():
    platts = json.load(open(os.path.join(HERE, "jev_platts.json"), encoding="utf-8"))
    buckets = platts["buckets"]
    ga, gb = platts["global"]["a"], platts["global"]["b"]
    n_hit, n_fallback, n = 0, 0, 0
    out_path = os.path.join(HERE, "jev_labeled_cal.jsonl")
    with open(os.path.join(HERE, "jev_labeled.jsonl"), encoding="utf-8") as fin, \
            open(out_path, "w", encoding="utf-8") as fout:
        for line in fin:
            d = json.loads(line)
            n += 1
            raw = float(d.get("calib_confidence", 0.5))
            dom = d.get("domain") or "unknown"
            if dom in buckets:
                a, b = buckets[dom]["a"], buckets[dom]["b"]
                n_hit += 1
            else:
                a, b = ga, gb
                n_fallback += 1
            d["calib_raw"] = raw
            d["calib_domain"] = dom
            d["calib_a"] = a
            d["calib_b"] = b
            d["calib_confidence"] = platt_apply(raw, a, b)
            fout.write(json.dumps(d, ensure_ascii=False) + "\n")
    print(json.dumps({"n": n, "bucket_hit": n_hit, "global_fallback": n_fallback,
                      "out": "jev_labeled_cal.jsonl"}, ensure_ascii=False), flush=True)


if __name__ == "__main__":
    main()
