#!/usr/bin/env python3
"""LoRA adapter 验收：base vs base+adapter 在同分布上的 PPL 对比.
adapter 若真学到行为，目标分布 PPL 应显著低于 base。
用法: python3 models/training/eval_lora.py --adapter models/lora-neotrix-jev [--data ..] [--n 50]
"""
import argparse
import json
import os
import random

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

try:
    from peft import PeftModel

    HAS_PEFT = True
except ImportError:
    HAS_PEFT = False


def row_text(r):
    parts = []
    for turn in r.get("conversations", []):
        c = turn.get("content", "")
        if c:
            parts.append(f"### {turn.get('role', '?')}:\n{c}")
    return "\n".join(parts)


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
    ap.add_argument("--adapter", required=True)
    ap.add_argument("--base", default="models/minimind-3")
    ap.add_argument("--data", default="models/training/jev_labeled.jsonl")
    ap.add_argument("--n", type=int, default=50)
    ap.add_argument("--seed", type=int, default=11)
    args = ap.parse_args()
    if not HAS_PEFT:
        print("need peft", flush=True)
        raise SystemExit(1)

    base = args.base if os.path.isabs(args.base) else os.path.join(os.getcwd(), args.base)
    adapter = args.adapter if os.path.isabs(args.adapter) else os.path.join(os.getcwd(), args.adapter)
    data = args.data if os.path.isabs(args.data) else os.path.join(os.getcwd(), args.data)

    random.seed(args.seed)
    rows = [json.loads(l) for l in open(data, encoding="utf-8")]
    texts = [row_text(r) for r in random.sample(rows, min(args.n, len(rows)))]
    print(f"texts={len(texts)}", flush=True)

    tok = AutoTokenizer.from_pretrained(base, trust_remote_code=True)
    m0 = AutoModelForCausalLM.from_pretrained(base, trust_remote_code=True)
    l0 = ppl(m0, tok, texts)
    print(f"base PPL-loss: {l0:.3f}", flush=True)
    del m0

    m1 = AutoModelForCausalLM.from_pretrained(base, trust_remote_code=True)
    m1 = PeftModel.from_pretrained(m1, adapter)
    m1 = m1.merge_and_unload()
    l1 = ppl(m1, tok, texts)
    print(f"base+adapter PPL-loss: {l1:.3f}", flush=True)
    print(f"delta: {l0 - l1:+.3f} ({'INFUSED' if l1 < l0 - 0.05 else 'FLAT'})", flush=True)


if __name__ == "__main__":
    main()
