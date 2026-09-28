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

## [R] 可再生权重 —— 来源已核实（2026-09-28 逐字节验证）

| 文件 | 上游 repo | revision | 验证结果 |
|---|---|---|---|
| `Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-Q6_K.gguf` | `HauhauCS/Qwen3.5-4B-Uncensored-HauhauCS-Aggressive` | `c09cdbcdb1fefad6d335809d445621b5f5ba0c6e` | **SHA-256 与上游 LFS oid 逐字节一致** |
| `mmproj-Qwen3.5-4B-Uncensored-HauhauCS-Aggressive-BF16.gguf` | 同上 | 同上 | **SHA-256 与上游 LFS oid 逐字节一致** |
| `minimind-3/model.safetensors` | `jingyaogong/minimind-3` | `f92512d4cd6142fa9acc0d6022375049a8974bf6` | **SHA-256 与上游 LFS oid 逐字节一致** |
| `minimind-3/config.json` | 同上 | 同上 | **git blob sha1 逐字节一致** |
| `minimind-3/tokenizer.json` | 同上 | 同上 | **git blob sha1 逐字节一致** |

复算：

```bash
bash scripts/ops/nt_weights_manifest.sh provenance
```

### 两种校验口径，不可混用

HF 对不同类型的文件给的 oid 是**两种不同的东西**：

| 文件类型 | 上游给的 oid | 本地怎么算 | 本仓标记 |
|---|---|---|---|
| LFS（`.gguf` / `.safetensors`） | `lfs.oid` = **内容 SHA-256** | `shasum -a 256` | `[lfs]` |
| 非 LFS（`.json`） | git **blob sha1** = `sha1("blob <字节数>\0" + content)` | `git hash-object` | `[blob]` |

混用会**永远不符** —— 而"永远不符"看起来很像"文件坏了"，会把人引向完全错误的方向
（重新下载、白查磁盘）。脚本按记录里的算法标记分派，不靠猜。

`git hash-object` 走的是仓库自己的对象格式，**即使 `models/` 被 gitignore 也照样能算**
（它只算不写对象库，不需要文件被跟踪）。

### 顺带补上的一个更基础的漏

`config.json` / `tokenizer.json` 此前**根本没进清单**，不只是"只比了尺寸"。
原 `write` 的 `[R]` 收集用的是权重扩展名白名单
（`.gguf|.safetensors|.bin|.pt|.pth`），而这两个文件是 `.json` —— 于是被漏在外面：
**丢了不会被 `verify` 发现，也不在台账里**。而 minimind-3 没有 config/tokenizer
根本加载不了，等于半个模型不可用。

已改为**收集 `models/` 下除 `training/` 外的全部文件**（排除 `.DS_Store` 与
`__pycache__`）。清单 58 → **60 项，双向零缺口**（盘上 60 / 登记 60）。

## 校验


### 为什么必须逐字节比对，不能只比尺寸

这一条是本次差点踩进去的坑。`minimind-3` 在 HF 上**被大量镜像**（`meet447/minimind`、
`Alrightlone/…`、`narvalsmiths/…` …），且 `config.json` 完全相同
（`Qwen3ForCausalLM` / `vocab 6400` / `hidden 768` / `8 layers`）—— 靠 config 匹配
**无法区分镜像**。

更要命的是尺寸：搜索首先命中的是 5 个月前的历史 commit `2943d18`，它的
`model.safetensors` 是

```
size  127834168          ← 与本地完全相同
sha256 177130464c7d…     ← 与本地不同
```

**尺寸一分不差，内容不同。** 只比尺寸会把错误来源记成事实，而照错来源重下的文件
校验和对不上，反而比"不知道来源"更难查。

同一 repo 的 `main` 才是对的：其 LFS oid `3adf69402b5d22e6…` 与本地一致 ——
文件在历史上被重传过。**结论：HF 上 LFS oid 就是内容的 SHA-256**，拿它当基准，
比对就变成确定性的，无需信任任何文件名或目录结构。



### 顺带确认的两件事

- **架构交叉验证**：上游 API 报 `architecture: qwen35`，与盘上 GGUF 头的
  `general.architecture: qwen35` 一致。
- **`--reasoning off` 有据**：该 repo 的 chat template 里确有
  `{%- if enable_thinking is defined and enable_thinking is false %}` 分支，
  即关思考要走显式开关 —— 这正是 `LOCAL-LLAMA-2026-09-28.md` 记的那条硬要求。
  上游 `context_length: 262144`（262K）。

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
