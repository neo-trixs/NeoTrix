# 权重与训练产物台账 — 2026-09-28

> `models/` 被 `.gitignore` 的 `models/*` 整条拦着，git 保护不到。
> 本文 + `WEIGHTS-MANIFEST.sha256` 是**唯一**记录「盘上有什么、什么版本、损坏没有」的地方。
> 工具：`scripts/ops/nt_weights_manifest.sh {write|verify|backup|restore|status}`

## 为什么按两类处置，而不是「备份 4.2G」

初看问题是「4.2G 权重无任何保护」。实测把内容拆开后发现**风险与体积完全不成比例**：

| 类 | 路径 | 体积 | 丢了会怎样 | 处置 |
|---|---|---|---|---|
| **[R] 可再生** | `qwen35-4b-uncensored/` | 3.9G | 重新下载（时间 + 带宽） | **只记来源与校验和，不备份** |
| **[R] 可再生** | `minimind-3/` | 122M | 重新下载 | 同上 |
| **[I] 不可再生** | `training/` | 274M | **重跑一遍训练**，等于永久丢失 | **真备份，gzip 后 57M** |

**该保护的是那 274M，不是那 4.2G。** GGUF 已量化，再压缩近乎零收益、耗时以小时计；
而 `training/` 里是 pretrain 语料（`pretrain.jsonl` 97M / `pretrain_dedup.jsonl` 95M /
`pretrain_clean.jsonl` 51M）、路由表、4.7M 精标语料 —— 压完 4.7:1，只要 57M。

「备份 4.2G」是错配：把可再生的东西存了三遍，把不可再生的东西存在 git 之外且无记录。

## [I] 不可再生产物 —— 已备份

```
~/Downloads/Neo/weights-backup/training-20260928.tar.gz        57M
~/Downloads/Neo/weights-backup/training-20260928.tar.gz.sha256  校验和
```

55 个文件（已排除 `__pycache__` / `*.pyc`，那些是编译产物）。
往返验证：解包到临时区，55 个文件与源 `diff -q` 无差异。

还原：

```bash
bash scripts/ops/nt_weights_manifest.sh restore ~/Downloads/Neo/weights-backup/training-20260928.tar.gz
bash scripts/ops/nt_weights_manifest.sh verify
```

`restore` 会先校验备份自身 sha256，不符即拒还原。

## [R] 可再生权重 —— 来源（部分待确认）

| 文件 | SHA-256 前 12 | 上游模型（取自 GGUF `general.name`） |
|---|---|---|
| `Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-Q6_K.gguf` | 见清单 | `Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-BF16` |
| `mmproj-Qwen3.5-4B-Uncensored-HauhouCS-Aggressive-BF16.gguf` | 见清单 | 同上（多模态投影） |
| `minimind-3/model.safetensors` | 见清单 | `architectures: [Qwen3ForCausalLM]`, `model_type: qwen3` |

⚠️ **HF repo 与 revision 未确认。** GGUF 头只记了 `general.name` /
`general.basename`（`Qwen3.5`）/ `general.architecture`（`qwen35`），**没有**
记 repo URL 或 commit。文件名 `HauhauCS-Aggressive` 强烈指向某个 HF 组织，
但**我没有验证过，不写进台账当事实** —— 写错 repo 比留空更糟：照错的 repo
下回来的量化版本校验和对不上，反而更难查。

补齐方式（二选一）：

1. 查 HF 页面确认 repo + revision，填进下表并提交；
2. 重新下载一次并记录 `--revision` / commit hash。

在补齐之前：**这 3 个文件属于「知道自己丢了，但不知道从哪捡回来」的状态。**
这是当前台账最大的缺口，比备份本身更要紧。

## 校验

```bash
bash scripts/ops/nt_weights_manifest.sh verify
```

对全部 58 项算 SHA-256 并比对清单，约 11 秒（4.5G，SSD）。
变异测试已验证它非空转：篡改 `models/training/laya_temps.json` 后
`verify` 精确报出该文件「不符」，还原后归零。

## 边界（照实说）

- 备份与仓库**同盘**（`/dev/disk3s5`，可用 179Gi）。防误删、`rm -rf`、误操作，
  **不防磁盘物理故障**。要跨盘，把 `training-20260928.tar.gz`（57M）拷到
  移动硬盘或另一台机器。
- 本机**没有第二块盘**（`df` 实测所有卷都是同一 `/dev/disk3s5`），所以「自动跨盘」
  这条路在当前硬件上不存在 —— 只能靠手动拷贝。
- `models/training/` 仍在持续增长（`smelt_progress.json` 之类是活文件）。
  重新训练出成果后应重跑 `backup`，否则新产物不在备份里。
