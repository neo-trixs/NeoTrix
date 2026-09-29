#!/usr/bin/env python3
"""NeoTrix LoRA stage-2 — 缺陷 #7 修复（灾难遗忘防护）.

MiniMind 官方路线：LoRA rank-16 只训 <1% 参数，基模权重冻结，
通用能力不丢，域行为叠加上去。本脚本在 full-ft 产出（或基模）上
叠 JEV/SFT adapter。

用法:
    python3 models/training/lora_finetune.py --smoke   # 20 步冒烟（pretrain 行包成对话）
    python3 models/training/lora_finetune.py --data models/training/jev_sft.jsonl --steps 500

数据格式：{"conversations": [{"role":..,"content":..}, ...]}（与 sft.jsonl 同构）。
为简练起见用全序列 LM loss（user 段不 mask，stage-2 adapter 容量小可接受）。
数值教训继承 finetune.py：adam_epsilon=1.0 + NanGuard 熔断。
"""
import argparse
import json
import os
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, TrainerCallback, TrainingArguments

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from finetune import NanGuardTrainer, collate  # noqa: E402

from peft import LoraConfig, TaskType, get_peft_model  # noqa: E402


class FinchCallback(TrainerCallback):
    """FINCH（2605.20005 穷人版）：遗忘上界 ∝ lr × √batch_loss，
    故 lr_t = base × sqrt(ref / ema_loss)，钳 [lo, hi]。
    高loss batch 降速（防遗忘），收敛后放开；目标函数本身不动，
    难样本照常学（与 token 屏蔽法不同）。ema 防抖，lag-1 步应用。
    用法：cb = FinchCallback(); <trainer 构造后> cb.bind(trainer); trainer.add_callback(cb)
    """

    def __init__(self, ref=7.0, lo=0.2, hi=1.5, beta=0.9):
        self.ref = ref
        self.lo = lo
        self.hi = hi
        self.beta = beta
        self.ema = None
        self.trainer = None
        self.base_lrs = None

    def bind(self, trainer):
        self.trainer = trainer
        try:
            self.base_lrs = [g["lr"] for g in trainer.optimizer.param_groups]
        except Exception:
            self.base_lrs = None

    def factor_for(self, batch_loss):
        if self.ema is None:
            self.ema = batch_loss
        else:
            self.ema = self.beta * self.ema + (1.0 - self.beta) * batch_loss
        if self.ema <= 0:
            return 1.0
        f = (self.ref / self.ema) ** 0.5
        return max(self.lo, min(self.hi, f))

    def on_step_end(self, args, state, control, **kwargs):
        if self.trainer is None or self.base_lrs is None:
            return
        last_loss = getattr(self.trainer, "_last_batch_loss", None)
        if last_loss is None:
            return
        try:
            f = self.factor_for(float(last_loss))
            for g, base in zip(self.trainer.optimizer.param_groups, self.base_lrs):
                g["lr"] = base * f
        except Exception:
            pass


class WeightedDataset(torch.utils.data.Dataset):
    """(文本, 置信度) 对；置信度缺省 1.0（pretrain 行/无标签行行为不变）.

    mask_user=True 时 pairs 为 [([{"role","content"}...], w)] 结构化输入，
    user 轮 token 在 labels 置 -100（D-speedup §6-3；默认关，行为零变化）。"""

    def __init__(self, pairs, tokenizer, max_len=256, mask_user=False):
        self.ids = []
        self.ws = []
        self.label_list = [] if mask_user else None
        for item, w in pairs:
            if mask_user:
                ids, labs = encode_conversation_masked(tokenizer, item, max_len)
            else:
                ids = tokenizer.encode(item, truncation=True, max_length=max_len)
                labs = None
            if len(ids) >= 8:
                self.ids.append(ids)
                self.ws.append(float(w))
                if mask_user:
                    self.label_list.append(labs)

    def __len__(self):
        return len(self.ids)

    def __getitem__(self, i):
        out = {
            "input_ids": torch.tensor(self.ids[i], dtype=torch.long),
            "w": torch.tensor(self.ws[i], dtype=torch.float32),
        }
        if self.label_list is not None:
            out["labels"] = torch.tensor(self.label_list[i], dtype=torch.long)
        return out


def encode_conversation_masked(tokenizer, conv, max_len=256):
    """逐轮编码：assistant/action/decision 轮训，user/observe 轮在 labels 置 -100。
    截断砍尾（与 tokenizer.encode(truncation=True) 保头口径一致；本仓数据中位 88 字符，
    极少触发）。返回 (ids, labels)，等长。"""
    ids, train = [], []
    for turn in conv:
        r = (turn.get("role") or "?").strip().lower()
        c = turn.get("content") or ""
        if not c:
            continue
        piece = tokenizer.encode(f"### {r}:\n{c}", add_special_tokens=False)
        ids.extend(piece)
        train.extend([r not in ("user", "observe", "?")] * len(piece))
    ids, train = ids[:max_len], train[:max_len]
    labels = [t if k else -100 for t, k in zip(ids, train)]
    return ids, labels


def weighted_collate(batch, pad_id=0):
    if "labels" in batch[0]:
        # mask_user 路：labels 预计算（含 -100），只 pad
        max_len = max(len(b["input_ids"]) for b in batch)
        out = {
            "input_ids": torch.stack([
                torch.cat([b["input_ids"],
                           torch.full((max_len - len(b["input_ids"]),), pad_id,
                                      dtype=torch.long)])
                for b in batch]),
            "labels": torch.stack([
                torch.cat([b["labels"],
                           torch.full((max_len - len(b["labels"]),), -100,
                                      dtype=torch.long)])
                for b in batch]),
        }
        out["attention_mask"] = (out["input_ids"] != pad_id).long()
    else:
        out = collate(
            [{"input_ids": b["input_ids"]} for b in batch], pad_id=pad_id
        )
    out["w"] = torch.stack([b["w"] for b in batch])
    return out


class WeightedTrainer(NanGuardTrainer):
    """不确定性门控可塑性（2609.18842 §3.3 穷人版）：
    样本 loss × calib_confidence 再平均——高置信样本多学、低置信少学，
    近似“在后验不确定处花可塑性、在确信处保护”的精度加权。
    非有限时归零跳过（与 NanGuard 同计数器），Adam 状态永不污染。"""

    def compute_loss(self, model, inputs, return_outputs=False, **kwargs):
        import torch.nn.functional as F

        w = inputs.pop("w", None)
        outputs = model(
            input_ids=inputs["input_ids"],
            attention_mask=inputs.get("attention_mask"),
        )
        logits = outputs.logits
        labels = inputs["labels"]
        sh_l = logits[..., :-1, :].contiguous()
        sh_y = labels[..., 1:].contiguous()
        per_tok = F.cross_entropy(
            sh_l.view(-1, sh_l.size(-1)), sh_y.view(-1),
            reduction="none", ignore_index=-100,
        ).view(sh_y.size(0), -1)
        mask = (sh_y != -100).float()
        seq_loss = (per_tok * mask).sum(1) / mask.sum(1).clamp_min(1)
        if w is not None:
            w = w.to(seq_loss.device).float().clamp(0.0, 1.0)
            loss = (seq_loss * w).sum() / w.sum().clamp_min(1e-9)
        else:
            loss = seq_loss.mean()
        if not torch.isfinite(loss).item():
            NanGuardTrainer.skipped_batches += 1
            print(
                f"[guard] skip non-finite weighted batch "
                f"#{NanGuardTrainer.skipped_batches}",
                flush=True,
            )
            loss = torch.nan_to_num(loss, nan=0.0, posinf=10.0, neginf=0.0)
        # FINCH 用：记录本 micro-batch 标量 loss（callback lag-1 步调 LR）
        try:
            self._last_batch_loss = float(loss.detach())
        except Exception:
            pass
        return (loss, outputs) if return_outputs else loss


def load_conversation_texts(path, limit=None):
    """返回 [(text, confidence)]；confidence 取 calib_confidence/confidence
    字段（nt_jev_calibration 产出行自带），缺省 1.0。"""
    pairs = []
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
            conv = obj.get("conversations")
            if isinstance(conv, list) and conv:
                parts = []
                for turn in conv:
                    r = turn.get("role", "?")
                    c = turn.get("content", "")
                    if c:
                        parts.append(f"### {r}:\n{c}")
                if parts:
                    w = obj.get("calib_confidence", obj.get("confidence", 1.0))
                    try:
                        w = float(w)
                    except (TypeError, ValueError):
                        w = 1.0
                    pairs.append(("\n".join(parts), w))
            elif obj.get("text"):
                # pretrain 行冒烟用：包成单轮对话
                pairs.append((f"### user:\n继续\n### assistant:\n{obj['text']}", 1.0))
    return pairs


def load_conversation_structs(path, limit=None):
    """结构化版：返回 [([{"role","content"}...], w)]，供 mask_user 路用。"""
    out = []
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
            conv = obj.get("conversations")
            if isinstance(conv, list) and conv:
                turns = [{"role": t.get("role", "?"), "content": t.get("content", "")}
                         for t in conv if t.get("content")]
                if turns:
                    w = obj.get("calib_confidence", obj.get("confidence", 1.0))
                    try:
                        w = float(w)
                    except (TypeError, ValueError):
                        w = 1.0
                    out.append((turns, w))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--smoke", action="store_true")
    ap.add_argument("--steps", type=int, default=500)
    ap.add_argument("--rank", type=int, default=16)
    ap.add_argument("--base", default=None, help="默认 full-ft 产出（有则用）否则基模")
    ap.add_argument("--data", default="models/training/pretrain.jsonl")
    ap.add_argument("--out", default="models/lora-neotrix-jev")
    ap.add_argument("--limit", type=int, default=None)
    ap.add_argument("--finch", action="store_true", default=True,
                    help="开 FINCH 自适应 LR（默认开；--no-finch 关）")
    ap.add_argument("--no-finch", dest="finch", action="store_false")
    ap.add_argument("--mask-user", action="store_true",
                    help="user/observe 轮 labels 置 -100（D-speedup §6-3；默认关零变化）")
    args = ap.parse_args()

    steps = 20 if args.smoke else args.steps
    limit = 500 if (args.smoke and args.limit is None) else args.limit
    base = args.base
    if base is None:
        cand = "models/minimind-3-neotrix-full"
        base = cand if os.path.isdir(cand) else "models/minimind-3"
    out = args.out if os.path.isabs(args.out) else os.path.join(os.getcwd(), args.out)
    data = args.data if os.path.isabs(args.data) else os.path.join(os.getcwd(), args.data)

    print(f"[lora] base={base} data={data} limit={limit} rank={args.rank} steps={steps}", flush=True)
    tok = AutoTokenizer.from_pretrained(base, trust_remote_code=True)
    if tok.pad_token is None:
        tok.pad_token = tok.eos_token
    model = AutoModelForCausalLM.from_pretrained(base, trust_remote_code=True)

    # MiniMind 官方口径：rank-16 注入 attention QKVO + MLP（本 repo 为 Qwen3 系命名）
    cfg = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=args.rank,
        lora_alpha=args.rank * 2,
        lora_dropout=0.05,
        bias="none",
        target_modules=[
            "q_proj", "k_proj", "v_proj", "o_proj",
            "gate_proj", "up_proj", "down_proj",
        ],
    )
    model = get_peft_model(model, cfg)
    model.print_trainable_parameters()

    pairs = load_conversation_texts(data, limit)
    print(f"[lora] loaded {len(pairs)} pairs (mask_user={args.mask_user})", flush=True)
    if args.mask_user:
        structs = load_conversation_structs(data, limit)
        ds = WeightedDataset(structs, tok, mask_user=True)
    else:
        ds = WeightedDataset(pairs, tok)
    print(f"[lora] dataset {len(ds)} samples", flush=True)
    if len(ds) == 0:
        print("[lora] empty dataset, abort", flush=True)
        sys.exit(1)

    targs = TrainingArguments(
        output_dir=out,
        max_steps=steps,
        per_device_train_batch_size=2,
        gradient_accumulation_steps=8,
        learning_rate=1e-4,  # LoRA 官方口径（adapter 从零训，可比全参激进）
        adam_epsilon=1.0,  # 同 finetune 教训
        max_grad_norm=0.5,
        warmup_steps=min(20, steps // 10),
        logging_steps=5 if args.smoke else 10,
        save_steps=max(10, steps // 2),
        save_total_limit=1,
        fp16=False,
        dataloader_num_workers=0,
        report_to="none",
        remove_unused_columns=False,
    )
    tr = WeightedTrainer(
        model=model,
        args=targs,
        train_dataset=ds,
        data_collator=lambda b: weighted_collate(b, tok.pad_token_id or 0),
    )
    if args.finch:
        finch_cb = FinchCallback()
        finch_cb.bind(tr)
        tr.add_callback(finch_cb)
        print("[lora] FINCH on (ref=7.0, clamp [0.2,1.5])", flush=True)
    print("[lora] training...", flush=True)
    tr.train()
    print(f"[lora] skipped poison batches: {NanGuardTrainer.skipped_batches}", flush=True)
    model.save_pretrained(out)
    tok.save_pretrained(out)
    print(f"[lora] done: {out}", flush=True)


if __name__ == "__main__":
    main()
