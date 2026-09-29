# D3 轨迹蒸馏 — 设计（2026-09-24，只设计 + Teacher-A 冒烟）

> 上游：`docs/plans/2026-09-24-human-inspired-refinement.md` §D3（M3，权重 0.25）。
> 停全量后唯一的炼丹口；ftv4（ftv6 续跑中）权重当底座。

## 1. 现状定点（均已读）

| 件 | 位置 | 状态 |
|---|------|------|
| LoRA stage-2 | `models/training/lora_finetune.py`（rank-16，全序列 LM loss、user 段不 mask、FINCH 防遗忘） | ✅ 有；缺分段加权 |
| 决策蒸馏 | `models/training/label_jev.py`（cocoons→sidecar /api/evaluate→`jev_labeled.jsonl`，WeightedTrainer 按 calib_confidence 加权） | ✅ 有；单步决策，非轨迹 |
| 验收 | `models/training/eval_lora.py`（base vs base+adapter PPL） | ✅ 有 |
| Teacher-A | sidecar :8149 `/api/evaluate`（单步 boolean/choice + 置信度；本窗已 HEALTHY） | ✅ 活；无轨迹端点 |
| Teacher-C | `nt_game/play/`（Trajectory/steps/reward、GRPO adapter、role baselines） | ✅ 结构有；未导出 SFT |
| NLL 选样器 | 无（grep `nll/NLL/per_sample` 零命中） | ❌ 缺 |
| 录轨迹器 | 无 | ❌ 缺 |

## 2. 轨迹 schema（新增，JSONL）

```json
{"segments": [{"role": "reason|action|observe|decision", "content": ".."}],
 "source": "teacher-a|teacher-b|teacher-c", "task": "..", "success": true}
```

转 conversations 时 role 映射：reason→assistant（think 段），action/decision→assistant，
observe→user；段标签保留供加权。

## 3. 三路 teacher

- **A（今晚可跑）**：sidecar 单步决策作 decision 段冷启动：`label_jev.py --n 500` → 500 条。
  冒烟（--n 30）本窗已验通。
  - [x] 2026-09-24 11:55 全量跑完（`label_jev --n 500` EXIT 干净，`jev_labeled.jsonl` 1660 行全含 conversations，kev 双发 500 行）
- **NLL 选样器** `nll_select.py` 原型已就绪（排序+去重+配额单测过；`score_with_student` 真模型接入等内存窗）
- **LoRA mask** `--mask-user` 已落地（见 speedup §6-3）
- **MLX 底座** `models/qwen3-06b-mlx`（bf16，1.19GB）预取完成，D3 首版米已下锅
- **B（待录）**：大模型 reason-act-observe 全轨迹——新增 `record_teacher.py`：
  调外部大模型 API（或回放 MetaAgentShell 派单日志）按 schema 落盘，目标 500 条。
- **C（内生）**：NT-PLAY 高 reward episode 导出（self-distill，R1 式）——`buffer.rs` 加
  `export_jsonl(min_reward)`，首批 200 条。

## 4. 缺口二件（实现等内存窗口）

1. **`nll_select.py`**：student=ftv4 底座对候选算 NLL，保留高 NLL 难样本 + 内容去重，
   500+500+200 → ~600 精选。
2. **分段加权**：`lora_finetune.py` 加 `segment_weights{reason:1.0, action:1.5, decision:1.5,
   observe:0.3}`（mask 仍保持 user 段不 mask 的 stage-2 口径，权重只作用于 assistant 段内
   token），冒烟 `--smoke` 先过。

## 5. 验证

- LoRA 首版出炉 → `eval_lora.py` PPL 不涨 → **D5 验收集涨分**（refinement 原话）。
- 回归：full-ft 通用能力（FINCH _clause：难样本照常学，lr 自适应防遗忘）。

## 6. 工时序

A 路 500 条（30min，sidecar 空闲时）→ NLL 选样器（1h）→ LoRA 首版 500 步（~1h，需内存窗）
→ D5 验涨 → B/C 扩量。
