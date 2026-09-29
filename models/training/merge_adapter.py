#!/usr/bin/env python3
"""LoRA adapter 合并进底座（生态循环晋升件其一）.

INFUSED adapter → merge_and_unload → 独立 full 权重目录，可直接加载/ serving，
不依赖 peft。CPU 可跑（0.5B，约 1-2 分钟）。
用法: python3 models/training/merge_adapter.py --adapter models/lora-neotrix-jev-cal --base models/minimind-3-neotrix-full --out models/minimind-3-neotrix-next
产出: <out>/（config/tokenizer/safetensors）+ merge_report.json
"""
import argparse
import json
import os
import sys
import time

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

from peft import PeftModel


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--adapter", required=True)
    ap.add_argument("--base", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--smoke", action="store_true", default=True,
                    help="合并后加载并生成一句话冒烟（默认开）")
    args = ap.parse_args()
    t0 = time.time()
    base = args.base if os.path.isabs(args.base) else os.path.join(os.getcwd(), args.base)
    adapter = args.adapter if os.path.isabs(args.adapter) else os.path.join(os.getcwd(), args.adapter)
    out = args.out if os.path.isabs(args.out) else os.path.join(os.getcwd(), args.out)
    tok = AutoTokenizer.from_pretrained(base, trust_remote_code=True)
    m0 = AutoModelForCausalLM.from_pretrained(base, trust_remote_code=True, torch_dtype=torch.float32)
    n_base = sum(p.numel() for p in m0.parameters())
    m1 = PeftModel.from_pretrained(m0, adapter)
    merged = m1.merge_and_unload()
    n_merged = sum(p.numel() for p in merged.parameters())
    merged.save_pretrained(out)
    tok.save_pretrained(out)
    rep = {"base": base, "adapter": adapter, "out": out,
           "params_base": n_base, "params_merged": n_merged,
           "same_params": n_base == n_merged}
    if args.smoke:
        merged.eval()
        with torch.no_grad():
            ids = tok("晶体核心自我验证：", return_tensors="pt")["input_ids"]
            gen = merged.generate(ids, max_new_tokens=20, do_sample=False)
            text = tok.decode(gen[0], skip_special_tokens=True)
        rep["smoke"] = text[:200]
        # 冒烟判定：能加载、能生成、非空即算合并且（质量归 PPL 门，不管 here）。
        # 另报 repeat_ratio（最高频字占比，>0.6 视为复读塌缩，供人看，不挡门）。
        tail, head = text[len("晶体核心自我验证："):], text[:20]
        rep["smoke_ok"] = bool(text.strip()) and tail.strip() != head.strip()
        if tail.strip():
            from collections import Counter
            rep["repeat_ratio"] = round(max(Counter(tail).values()) / len(tail), 3)
        else:
            rep["repeat_ratio"] = 1.0
        print(f"[merge] smoke_ok={rep['smoke_ok']} :: {text[:120]}", flush=True)
    rep["elapsed"] = round(time.time() - t0, 1)
    with open(os.path.join(out, "merge_report.json"), "w", encoding="utf-8") as f:
        json.dump(rep, f, ensure_ascii=False, indent=1)
    print(json.dumps({k: v for k, v in rep.items() if k != "smoke"}, ensure_ascii=False), flush=True)
    if args.smoke and not rep["smoke_ok"]:
        sys.exit(3)


if __name__ == "__main__":
    main()
