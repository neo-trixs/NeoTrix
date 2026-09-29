#!/usr/bin/env python3
"""NeoTrix 训练数据质量过滤（P4.3 缺陷修复：Quality > Quantity）.

输入 pretrain.jsonl（477K），产出 pretrain_clean.jsonl：
1. HTML 实体解码（&#x27; → '，&amp; → &，1312 行受影响）
2. 精确去重（md5，~11.8K 行）
3. 长度过滤（<10 字符太短无信息；>2000 字符多为拼接垃圾）
4. 空白归一（多余空行/首尾空白）

只读源文件、写新文件，现跑训练不受影响（已全载入内存）。
用法：python3 models/training/clean_data.py [--in ..] [--out ..]
"""
import argparse
import hashlib
import html
import json
import os
import re

WS_RE = re.compile(r"[ \t\xa0]{2,}|\n{3,}")


def clean_text(t: str) -> str:
    t = html.unescape(t)  # &#x27; &amp; &lt; 等
    t = WS_RE.sub(" ", t).strip()
    return t


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", default="models/training/pretrain.jsonl")
    ap.add_argument("--out", dest="out", default="models/training/pretrain_clean.jsonl")
    ap.add_argument("--min-len", type=int, default=10)
    ap.add_argument("--max-len", type=int, default=2000)
    args = ap.parse_args()

    inp = args.inp if os.path.isabs(args.inp) else os.path.join(os.getcwd(), args.inp)
    out = args.out if os.path.isabs(args.out) else os.path.join(os.getcwd(), args.out)

    seen = set()
    stats = {"rows": 0, "kept": 0, "empty": 0, "dup": 0, "short": 0, "long": 0, "html_fixed": 0}
    with open(inp, encoding="utf-8") as fin, open(out, "w", encoding="utf-8") as fout:
        for line in fin:
            stats["rows"] += 1
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            raw = obj.get("text", "")
            if "&#" in raw or "&amp;" in raw or "&lt;" in raw:
                stats["html_fixed"] += 1
            t = clean_text(raw)
            if not t:
                stats["empty"] += 1
                continue
            if len(t) < args.min_len:
                stats["short"] += 1
                continue
            if len(t) > args.max_len:
                stats["long"] += 1
                continue
            dg = hashlib.md5(t.encode()).hexdigest()
            if dg in seen:
                stats["dup"] += 1
                continue
            seen.add(dg)
            fout.write(json.dumps({"text": t}, ensure_ascii=False) + "\n")
            stats["kept"] += 1
            if stats["rows"] % 100000 == 0:
                print(f"[clean] {stats['rows']} rows, kept {stats['kept']}", flush=True)
    print(f"[clean] done: {stats}", flush=True)


if __name__ == "__main__":
    main()
