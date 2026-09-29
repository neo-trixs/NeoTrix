#!/usr/bin/env python3
"""全量微调验收：base vs 全量模型在同分布上的 PPL-loss 对比.
用法: python3 models/training/eval_full.py --model models/minimind-3-neotrix-v2 [--data ..] [--n 100]
delta<-0.05 判 INFUSED（学到东西），否则 FLAT。
"""
import argparse
import json
import os
import random

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer


def ppl(model, tok, texts, max_len=256):
    model.eval()
    tot, n = 0.0, 0
    with torch.no_grad():
        for t in texts:
            ids = tok.encode(t, truncation=True, max_length=max_len)
            if len(ids) < 8:
                continue
            inp = torch.tensor([ids])
            tot += model(input_ids=inp, labels=inp).loss.item()
            n += 1
    return tot / max(1, n)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True, help="全量微调输出目录")
    ap.add_argument("--base", default="models/minimind-3")
    ap.add_argument("--data", default="models/training/pretrain_clean.jsonl")
    ap.add_argument("--n", type=int, default=100)
    ap.add_argument("--seed", type=int, default=11)
    args = ap.parse_args()

    base = args.base if os.path.isabs(args.base) else os.path.join(os.getcwd(), args.base)
    model = args.model if os.path.isabs(args.model) else os.path.join(os.getcwd(), args.model)
    data = args.data if os.path.isabs(args.data) else os.path.join(os.getcwd(), args.data)

    random.seed(args.seed)
    rows = []
    for line in open(data, encoding="utf-8"):
        line = line.strip()
        if not line:
            continue
        try:
            t = json.loads(line).get("text", "")
        except json.JSONDecodeError:
            continue
        if t and t.strip():
            rows.append(t.strip())
    texts = random.sample(rows, min(args.n, len(rows)))
    print(f"texts={len(texts)}", flush=True)

    tok = AutoTokenizer.from_pretrained(base, trust_remote_code=True)
    m0 = AutoModelForCausalLM.from_pretrained(base, trust_remote_code=True)
    l0 = ppl(m0, tok, texts)
    print(f"base PPL-loss: {l0:.3f}", flush=True)
    del m0

    m1 = AutoModelForCausalLM.from_pretrained(model, trust_remote_code=True)
    l1 = ppl(m1, tok, texts)
    print(f"full PPL-loss: {l1:.3f}", flush=True)
    print(f"delta: {l0 - l1:+.3f} ({'INFUSED' if l1 < l0 - 0.05 else 'FLAT'})", flush=True)


if __name__ == "__main__":
    main()
