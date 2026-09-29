#!/usr/bin/env python3
"""NeoTrix fine-tune — pretrain.jsonl 在 MiniMind-3 上微调.

用法:
    python3 models/training/finetune.py --smoke        # 冒烟：100 步验证管线
    python3 models/training/finetune.py --steps 1000   # 正式：1000 步
    python3 models/training/finetune.py --steps 5000 --out models/minimind-3-neotrix

依赖: torch + transformers (datasets 不需要，手动读 JSONL).
"""
import argparse
import json
import os
import sys

import torch
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
)


def load_jsonl_texts(path, limit=None):
    texts = []
    with open(path, encoding="utf-8") as f:
        for i, line in enumerate(f):
            if limit and i >= limit:
                break
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = obj.get("text", "")
            if t and t.strip():
                texts.append(t.strip())
    return texts


class TextDataset(torch.utils.data.Dataset):
    def __init__(self, texts, tokenizer, max_len=256):
        self.ids = []
        for t in texts:
            ids = tokenizer.encode(t, truncation=True, max_length=max_len)
            if len(ids) >= 8:  # 过短丢弃
                self.ids.append(ids)

    def __len__(self):
        return len(self.ids)

    def __getitem__(self, i):
        return {"input_ids": torch.tensor(self.ids[i], dtype=torch.long)}


def collate(batch, pad_id=0):
    # 定形 batch（D-speedup）：pad 到 8 的倍数，MPS graph cache 形状从 ~250 种压到 ≤32 种。
    # 语义不变：labels 仍 -100 屏蔽，attention_mask 照旧（50 组随机 batch 已验：定形+保号+mask 全对）。
    max_len = max(len(b["input_ids"]) for b in batch)
    max_len = ((max_len + 7) // 8) * 8
    input_ids = torch.stack([
        torch.cat([b["input_ids"], torch.full((max_len - len(b["input_ids"]),), pad_id, dtype=torch.long)])
        for b in batch
    ])
    labels = input_ids.clone()
    labels[labels == pad_id] = -100
    attn = (input_ids != pad_id).long()
    return {"input_ids": input_ids, "labels": labels, "attention_mask": attn}


class NanGuardTrainer(Trainer):
    """NaN 熔断（双层）：
    1. compute_loss：前向非有限 → 归零跳过（防毒样本污染）；
    2. training_step：反向后逐张量查梯度，非有限的清零保留有限的
       （防“前向有限、反向 NaN”毒化 Adam 状态——v2 在 epoch 0.33
       死于此：loss 5.5 有限但 grad NaN，一步进优化器即全军覆没）。
    计数器统一 skipped_batches。"""

    skipped_batches: int = 0

    def training_step(self, model, inputs, num_items_in_batch=None):
        loss = super().training_step(model, inputs, num_items_in_batch)
        bad = 0
        for p in model.parameters():
            g = p.grad
            if g is not None and not torch.isfinite(g).all().item():
                g.detach().zero_()
                bad += 1
        if bad:
            type(self).skipped_batches += 1
            print(
                f"[guard-grad] zeroed {bad} non-finite grad tensors "
                f"(total skips #{type(self).skipped_batches})",
                flush=True,
            )
        return loss

    def compute_loss(self, model, inputs, return_outputs=False, **kwargs):
        outputs = model(**inputs)
        loss = outputs.loss
        if not torch.isfinite(loss).item():
            type(self).skipped_batches += 1
            print(
                f"[guard] skip non-finite loss batch #{type(self).skipped_batches}",
                flush=True,
            )
            loss = torch.nan_to_num(loss, nan=0.0, posinf=10.0, neginf=0.0)
        return (loss, outputs) if return_outputs else loss


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--smoke", action="store_true", help="100 步冒烟")
    ap.add_argument("--steps", type=int, default=1000)
    ap.add_argument("--out", default="models/minimind-3-neotrix-smoke")
    ap.add_argument("--data", default="models/training/pretrain.jsonl")
    ap.add_argument("--limit", type=int, default=None, help="最多用多少行（冒烟默认5000）")
    ap.add_argument("--lr", type=float, default=2e-5, help="学习率（MiniMind官方口径1e-4~5e-4；NaN防护靠eps+guard）")
    ap.add_argument("--resume", default=None, help="从 checkpoint 续跑（137 OOM 后用，如 models/minimind-3-neotrix-v2/checkpoint-4900）")
    args = ap.parse_args()

    repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    model_dir = os.path.join(os.path.dirname(repo), "models", "minimind-3")
    # 兼容从 repo 根跑
    if not os.path.isdir(model_dir):
        model_dir = "models/minimind-3"
    data_path = args.data if os.path.isabs(args.data) else os.path.join(os.getcwd(), args.data)
    out_dir = args.out if os.path.isabs(args.out) else os.path.join(os.getcwd(), args.out)

    steps = 100 if args.smoke else args.steps
    limit = 5000 if (args.smoke and args.limit is None) else args.limit

    print(f"[ft] model_dir={model_dir}", flush=True)
    print(f"[ft] data={data_path} limit={limit} steps={steps} out={out_dir}", flush=True)

    tok = AutoTokenizer.from_pretrained(model_dir, trust_remote_code=True)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    model = AutoModelForCausalLM.from_pretrained(model_dir, trust_remote_code=True)

    texts = load_jsonl_texts(data_path, limit)
    print(f"[ft] loaded {len(texts)} texts", flush=True)
    ds = TextDataset(texts, tok)
    print(f"[ft] dataset {len(ds)} samples (len>=8)", flush=True)
    if len(ds) == 0:
        print("[ft] empty dataset, abort", flush=True)
        sys.exit(1)

    targs = TrainingArguments(
        output_dir=out_dir,
        max_steps=steps,
        per_device_train_batch_size=2,
        gradient_accumulation_steps=8,  # 有效 batch 16
        learning_rate=args.lr,
        adam_epsilon=1.0,  # 必需：torch 2.12.1 + MiniMind-3 自适应分母开方 bug，默认 1e-8 单步全参 NaN
        max_grad_norm=0.5,  # 显式裁剪（默认 1.0 不够）
        warmup_steps=min(50, steps // 10),
        logging_steps=10,
        # 137 OOM 会随时杀进程；14500 才存一次等于全丢（v2 死在 414/58000 且 out 空）
        save_steps=100,
        save_total_limit=3,
        load_best_model_at_end=False,
        fp16=False,  # M5 CPU 无 fp16 加速，保持 fp32 稳定
        dataloader_num_workers=0,
        report_to="none",
        remove_unused_columns=False,
    )
    trainer = NanGuardTrainer(
        model=model,
        args=targs,
        train_dataset=ds,
        data_collator=lambda b: collate(b, tok.pad_token_id or 0),
    )
    print("[ft] training...", flush=True)
    if args.resume:
        print(f"[ft] resume from {args.resume}", flush=True)
        trainer.train(resume_from_checkpoint=args.resume)
    else:
        trainer.train()
    print(f"[ft] skipped poison batches: {NanGuardTrainer.skipped_batches}", flush=True)
    print("[ft] saving...", flush=True)
    trainer.save_model(out_dir)
    tok.save_pretrained(out_dir)
    print(f"[ft] done: {out_dir}", flush=True)


if __name__ == "__main__":
    main()
