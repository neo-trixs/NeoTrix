#!/usr/bin/env python3
"""NeoTrix Colab 版微调（Google 免费 T4 GPU）.

在 Colab 里按顺序跑三个 cell 即可（见底部 COLAB_CELLS 说明）。
与本地 finetune.py 同源策略：adam_epsilon=1.0（防自适应分母 bug）+
NanGuard 熔断 + FINCH 可选；GPU 开 fp16 + 大 batch。

数据：把 pretrain_clean.jsonl 传到 Colab（Drive 挂载或直接上传 51MB）。
权重：直读 HF jingyaogong/minimind-3，无需上传。
产出：Drive /content/drive/MyDrive/neotrix-out/（防掉线丢）。
"""
import argparse
import json
import os
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, Trainer, TrainingArguments


class NanGuardTrainer(Trainer):
    skipped_batches = 0

    def compute_loss(self, model, inputs, return_outputs=False, **kwargs):
        outputs = model(**inputs)
        loss = outputs.loss
        if not torch.isfinite(loss).item():
            type(self).skipped_batches += 1
            print(f"[guard] skip #{type(self).skipped_batches}", flush=True)
            loss = torch.nan_to_num(loss, nan=0.0, posinf=10.0, neginf=0.0)
        return (loss, outputs) if return_outputs else loss


class TextDataset(torch.utils.data.Dataset):
    def __init__(self, texts, tokenizer, max_len=256):
        self.ids = []
        for t in texts:
            ids = tokenizer.encode(t, truncation=True, max_length=max_len)
            if len(ids) >= 8:
                self.ids.append(ids)

    def __len__(self):
        return len(self.ids)

    def __getitem__(self, i):
        return {"input_ids": torch.tensor(self.ids[i], dtype=torch.long)}


def collate(batch, pad_id=0):
    m = max(len(b["input_ids"]) for b in batch)
    ii = torch.stack([
        torch.cat([b["input_ids"], torch.full((m - len(b["input_ids"]),), pad_id, dtype=torch.long)])
        for b in batch
    ])
    lb = ii.clone()
    lb[lb == pad_id] = -100
    return {"input_ids": ii, "labels": lb, "attention_mask": (ii != pad_id).long()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="jingyaogong/minimind-3")
    ap.add_argument("--data", required=True, help="pretrain_clean.jsonl 路径")
    ap.add_argument("--out", required=True, help="输出目录（Drive 路径防掉线）")
    ap.add_argument("--steps", type=int, default=58000)
    ap.add_argument("--lr", type=float, default=1e-4)
    ap.add_argument("--bs", type=int, default=8, help="per-device batch（T4 16GB 可设 8-16）")
    ap.add_argument("--accum", type=int, default=4, help="8x4=32 有效 batch")
    ap.add_argument("--limit", type=int, default=None)
    args = ap.parse_args()

    use_bf16 = torch.cuda.is_available() and torch.cuda.is_bf16_supported()
    print(f"[colab] cuda={torch.cuda.is_available()} bf16={use_bf16}", flush=True)
    tok = AutoTokenizer.from_pretrained(args.model, trust_remote_code=True)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    model = AutoModelForCausalLM.from_pretrained(
        args.model, trust_remote_code=True,
        torch_dtype=torch.bfloat16 if use_bf16 else torch.float32,
    )
    texts = []
    with open(args.data, encoding="utf-8") as f:
        for i, line in enumerate(f):
            if args.limit and i >= args.limit:
                break
            try:
                t = json.loads(line).get("text", "")
            except json.JSONDecodeError:
                continue
            if t and t.strip():
                texts.append(t.strip())
    print(f"[colab] texts={len(texts)}", flush=True)
    ds = TextDataset(texts, tok)
    print(f"[colab] dataset={len(ds)}", flush=True)

    targs = TrainingArguments(
        output_dir=args.out,
        max_steps=args.steps,
        per_device_train_batch_size=args.bs,
        gradient_accumulation_steps=args.accum,
        learning_rate=args.lr,
        adam_epsilon=1.0,
        max_grad_norm=0.5,
        warmup_steps=500,
        logging_steps=50,
        save_steps=5000,
        save_total_limit=2,
        bf16=use_bf16,
        fp16=not use_bf16 and torch.cuda.is_available(),
        dataloader_num_workers=2,
        report_to="none",
        remove_unused_columns=False,
    )
    tr = NanGuardTrainer(
        model=model, args=targs, train_dataset=ds,
        data_collator=lambda b: collate(b, tok.pad_token_id or 0),
    )
    tr.train()
    print(f"[colab] skipped={NanGuardTrainer.skipped_batches}", flush=True)
    tr.save_model(args.out)
    tok.save_pretrained(args.out)
    print(f"[colab] done: {args.out}", flush=True)


COLAB_CELLS = r"""
# Cell 1 — 环境（T4 GPU：运行时→更改运行时类型→T4）
!pip install -q transformers accelerate
from google.colab import drive
drive.mount('/content/drive')

# Cell 2 — 传脚本+数据（本地上传 colab_finetune.py 和 pretrain_clean.jsonl 到 /content/）
# 或: !curl -sL <raw-url> -o colab_finetune.py

# Cell 3 — 开训（冒烟先行：--steps 200 --limit 2000，通了再全量）
!python3 /content/colab_finetune.py --steps 200 --limit 2000 \
  --data /content/pretrain_clean.jsonl --out /content/drive/MyDrive/neotrix-smoke
# 全量：--steps 58000（2 epochs，T4 约 2-4h；掉线从 checkpoint 续跑见 README）
"""

if __name__ == "__main__":
    if "--cells" in sys.argv:
        print(COLAB_CELLS)
    else:
        main()
