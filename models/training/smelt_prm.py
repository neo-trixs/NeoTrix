#!/usr/bin/env python3
"""PRM800K 过程监督数据拉取（tasksource/PRM800K，3000 行 pilot）。"""
from datasets import load_dataset
import json

ds = load_dataset("tasksource/PRM800K", split="train", streaming=True)
cnt = 0
with open("models/training/hf_prm800k.jsonl", "w", encoding="utf-8") as f:
    for row in iter(ds.take(3000)):
        q = str(row.get("question", ""))
        g = str(row.get("generation", ""))
        if not q.strip() or not g.strip():
            continue
        label = str(row.get("label", ""))
        content = f"问：{q}\n过程：{g[:2200]}\n标签：{label}"[:3000]
        f.write(json.dumps({
            "url": "hf-dataset://tasksource/PRM800K",
            "title": q[:120],
            "content": content,
            "domain": "hf-prm",
        }, ensure_ascii=False) + "\n")
        cnt += 1
print(f"PRM800K: {cnt} rows", flush=True)
