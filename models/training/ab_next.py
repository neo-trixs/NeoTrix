#!/usr/bin/env python3
"""next vs full 离线生成 AB（晋升候选实战验收）.

PPL 门只看分布拟合；本脚本看生成质量：同 prompt 双模型贪婪解码，
记长度/复读率/截断，落盘 side-by-side 报告供人判。不自动晋升。
用法: python3 models/training/ab_next.py [--n 8]
产出: models/training/ab_next_report.md
"""
import argparse
import time

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer

BASE = "models/minimind-3-neotrix-full"
NEXT = "models/minimind-3-neotrix-next"

PROMPTS = [
    "晶体核心自我验证：",
    "该陈述可靠吗？回答成立或存疑并给置信度：Minimind 采用 LoRA 微调",
    "总结一句话：温度缩放按域重拟合的意义",
    "列出三步：如何验证一个 LoRA adapter 是否有效",
    "Is this statement reliable? Answer yes or no: LoRA rank 16 trains less than 1% of parameters",
    "翻译成中文：calibration is domain-dependent",
    "评估 sidecar 延迟的方法是",
    "解释：为什么 full 全量微调在 CPU 上性价比死刑",
]


def repeat_ratio(text):
    from collections import Counter
    t = text.strip()
    if not text or len(t) < 5:
        return 1.0
    return round(max(Counter(t).values()) / len(t), 3)


def run_one(model_dir, prompts, max_new=60):
    tok = AutoTokenizer.from_pretrained(model_dir, trust_remote_code=True)
    m = AutoModelForCausalLM.from_pretrained(model_dir, trust_remote_code=True,
                                             torch_dtype=torch.float32)
    m.eval()
    outs = []
    with torch.no_grad():
        for p in prompts:
            ids = tok(p, return_tensors="pt")["input_ids"]
            gen = m.generate(ids, max_new_tokens=max_new, do_sample=False)
            text = tok.decode(gen[0], skip_special_tokens=True)[len(p):]
            outs.append({"prompt": p, "gen": text[:300],
                         "len": len(text.strip()), "repeat": repeat_ratio(text)})
    del m
    try:
        import gc
        gc.collect()
    except Exception:
        pass
    return outs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=len(PROMPTS))
    args = ap.parse_args()
    prompts = PROMPTS[: args.n]
    t0 = time.time()
    base_outs = run_one(BASE, prompts)
    next_outs = run_one(NEXT, prompts)
    L = ["# next vs full 离线生成 AB", "",
         f"- base={BASE} next={NEXT} n={len(prompts)}",
         f"- elapsed={time.time()-t0:.0f}s（贪婪解码 max_new=60，CPU）", ""]
    for i, (b, n_) in enumerate(zip(base_outs, next_outs)):
        L += [f"## P{i+1}: {b['prompt']}", "",
              f"- full: len={b['len']} repeat={b['repeat']}",
              f"  > {b['gen'][:200]}", "",
              f"- next: len={n_['len']} repeat={n_['repeat']}",
              f"  > {n_['gen'][:200]}", ""]
    L += ["## 小结（人判）",
          "- repeat 越低越好；len 过短（<5）视为拒答/塌缩；内容相关性人看。",
          "- PPL 门（INFUSED +0.169）已过，本表只做质检，不改 verdict。"]
    with open("models/training/ab_next_report.md", "w", encoding="utf-8") as f:
        f.write("\n".join(L) + "\n")
    print("\n".join(L[:12]), flush=True)
    print("... report -> models/training/ab_next_report.md", flush=True)


if __name__ == "__main__":
    main()
