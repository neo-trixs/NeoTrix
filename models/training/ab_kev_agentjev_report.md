# kev-0.8B ↔ AgentJev A/B

- n=80 seed=13 time=2026-09-23 16:38:42 (kev) / 2026-09-23 (AgentJev ref)
- 评测集：`models/training/jev_kev.jsonl` / `jev_labeled.jsonl` 中带 rel label 的行，seed=13 shuffle 取 n=80
- kev 侧：`HF_HUB_OFFLINE=1 ... ab_kev_agentjev.py --n 80 --skip-jev`（单进程，MPS，T=2.41，EXIT:0，log `sessions/logs/ab4.log`）
- AgentJev 侧：先前同 seed 跑通（HTTP :8149，n=80，errors=0），ref 见 `sessions/logs/ab_jev_ref.txt`

## 结果

| 模型 | acc | brier | ece | conf | errors |
|---|---:|---:|---:|---:|---:|
| **AgentJev-0.6B** | **0.575** | 0.327 | 0.211 | 0.725 | 0/80 |
| **kev-0.8B** | 0.512 | **0.307** | **0.160** | 0.673 | 0/80 |

- **AgentJev-0.6B**: acc=0.575 brier=0.327 ece=0.211 conf=0.725 errors=0/80
- **kev-0.8B**: acc=0.512 brier=0.307 ece=0.160 conf=0.673 errors=0/80
- pairwise: skipped（本轮 `--skip-jev`，仅汇总指标对比；per-row 对齐可在下一轮双跑补齐）

## 结论：AgentJev 更准 (Δacc=-0.063)

- 准确率：AgentJev +6.3pp（0.575 vs 0.512），|Δ|>0.02 → **AgentJev 胜**
- 校准：kev 的 Brier/ECE 更好（0.307/0.160 vs 0.327/0.211），置信度更保守（0.673 vs 0.725）
- 判读：决策头 gate 按 **acc** 取 **AgentJev-0.6B** 继续挂 `:8149`；kev-0.8B 作为 **校准更稳的次级/融合候选**，不单独换门
- 后续可选：双模型同批 per-row 对齐算 flip-rate；或 kev 只读 calibrator（温度/T 再扫）

## 运行备注

- 无 flash-linear-attention 时 hybrid Qwen3.5 前向 ~7–17s/条；`--device` 已加，**默认 cpu**
- 双跑曾互杀 report；本轮单进程 + `ab4.done` 收口
- 产物：本文件；ref `sessions/logs/ab_jev_ref.txt`；raw `sessions/logs/ab4.log` / 早前 jev log
