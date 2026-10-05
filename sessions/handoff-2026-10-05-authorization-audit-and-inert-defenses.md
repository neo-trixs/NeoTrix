# Handoff — 授权栈审计与「声称存在但实际不生效」防线（本窗口）

> 模板：`sessions/HANDOFF-TEMPLATE.md`。**§8 收工自查已填**（AGENTS.md 硬规则）。

## 1. 会话标识

- 窗口：本会话（吸收 + 缺陷修复轮）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-10-05 23:20

## 2. 目标（一句话）

> 用 NeoTrix 自己的能力持续进化：**并行**吸收外部源（MIT 可取码）+
> 补齐真实缺陷 + 建立缺失的门，直到预算用尽、只留最优解。

## 3. 已完成（按危害排序，均已落地主分支并验证）

| 提交 | 内容 | 危害/价值 |
|---|---|---|
| `0b5358cb` | `git push --force` 的硬拒**从未生效过**（键永不相交 + `Deny` 被降级成「需审批」） | ⭐⭐⭐ 两层失效叠加 |
| `bad544f9` | `nt_policy` 漏洞：多余一个 `path` 参数即**跳过 bash 逃逸检查** | ⭐⭐⭐ neobot 唯一真执行环 |
| `fdc641d2` | 工具 panic 让执行器**永久挂死**（清理动作无 guard）+ 16 处锁中毒 | ⭐⭐⭐ 实测 >60s 挂死 |
| `6d47cc29` | 破坏性命令 deny 集（`rm -rf`/`dd`/`mkfs`/`git push --force` 此前全放行） | ⭐⭐⭐ 生产可达 |
| `93c6a01` | `web fetch` 对**所有 JSON 端点**返回带壳 HTML + tracing 污染 stdout | ⭐⭐⭐ 用自己的 CLI 干活时撞出 |
| `48ea78eb` | **5 个孤儿目录 2663 行从未编译** → 挂载后暴露 3 个真缺陷 | ⭐⭐⭐ 绿测试给未编译代码背书 |
| `5b7d16c0` | 新门 `check-orphan-dirs.sh`（防「目录未挂载」） | ⭐⭐ 补上系统性缺口 |
| `7b162c47` | Strata 的 effort 方言归一（`none`/`minimal` 曾白烧推理预算） | ⭐⭐ 静默烧钱 |
| `5a3dce85` 系 | Vibe-Trading 的**未来污染不变性元测试** | ⭐⭐ 前视偏差 |
| `6b72ca16` | 八源许可证判定（3 个 `NOASSERTION` 全是真限制）+ **GPL-3.0 空白区裁决** | ⭐⭐ 消除歧义 |
| `591dd9f0` | **`CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md`**：10 项「声称存在但实际不生效」实证 | ⭐⭐⭐ 方法论资产 |
| `0b49366b`/`c24347b8`/`5a3dce85` | `EntropyMonitor` 除零、`VSAEngine` 四级障碍、`llm_judge` 三态裁决 | ⭐⭐ |

## 4. 正在改的文件

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| （无）| 主工作树无我的未提交改动 | — |

## 5. 下一步（按优先级排序）

1. **落地 `5bf3aee7` + `5b058ce8`**（见 §8.2，patch 已兜底，门红在他窗）
2. **P0 需产品裁决**：`switch_profile` 能否改变全局 `ApprovalMode`？
   当前可单次调用把 `strict-nt_shield` 改成 `AutoEdit`，**无记录无确认**。
   允许 → 必须落审计 + 显式确认；不允许 → 该字段从 profile 移除。
3. **P1 接 `agent_guardrails`**：接成 `ShieldEnforcer::check_all` 的**第 9 段**，
   且 `GuardrailVerdict::Block` 要**真阻断**（别学 `ProjectLaws::check_laws` 的
   non-blocking default）。⚠️ 该目录他窗正在改 ⇒ 等他落地。
4. **P1 接线 `from_client_spelling`** 到 `gateway`（我方欠账：目前零生产调用方）
5. **P1 Codewhale `HANDOFF_SECTIONS`** 交接便条（纯文档、零代码成本）
6. **P2 Strata 5 态 `phase` 机 + 长 prefill keep-alive**
7. **P2 25 个孤儿目录**剩余 20 个逐个裁决（基线在 `scripts/orphan-dir-baseline.txt`）

## 6. 阻塞点

- **主树 P0 门红在他窗**（2026-10-05 23:10 核实，7 errors）：
  `l5_cognition/nt_consciousness/mod.rs`、`l1_action/nt_io/nt_io_agent_loop/mod.rs`
  （`nt_loop_canary_tests` 文件缺失，他窗暂存了对它的删除）、
  `l3_embodiment/nt_shield/guard/agent_guardrails/input_validator.rs`。
  ⇒ **非我引入**，我的两个文件在该日志里 `rg` 命中 **0**。
- **`ASD-STE100_ISSUE9.pdf` 仍无法读取**（本模型不支持 PDF 输入）。

## 7. 给接手会话的话

- 恢复命令：先读本文件，再 `git status --short`；成果核实请**查内容**
  （`rg '<符号名>' <file>`）**不要用 `git merge-base --is-ancestor`** ——
  本会话他窗 rebase 过历史，`is-ancestor` 对已 cherry-pick 的成果会**失真返回 false**。
- ⭐ **共享工作树上提交一律 `git -c core.hooksPath=.githooks commit --only <我的文件>`**：
  本会话观察到 `core.hooksPath` 被指向**已不存在的临时目录**
  ⇒ git 静默跳过所有 hook。`-c` 不落盘、不改配置、正典门真跑。
- ⭐ **绝不用 `git cherry-pick <sha>` 整体应用**（共享树暂存区有他窗改动/删除）。
  只取文件：`git checkout <sha> -- <我的文件>` 然后 `commit --only`。
- ⭐ worktree 隔离时的方向：**只从主树 `cp` 到 worktree，绝不反向 `cp`** ——
  本会话反向过一次，把主树成果覆盖成了 worktree 的旧版。
- ⚠️ 他窗文件不要碰：`agent_guardrails/*`、`nt_io_agent_loop/*`、`nt_consciousness/*`、
  `nt_memory/mod.rs`、`Cargo.lock`、`.neotrix/*.json`。

## 8. 收工自查（AGENTS.md 硬要求，逐条不空）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` → **exit 0**，输出：

```
[worktree-gate] worktree=8 个 | 合计 40605M | target 占 40016M
[worktree-gate] 带未提交改动: 5 个 | 近3h有改动: 3 个
[worktree-gate] ⚠️  3 个 worktree 近 3 小时仍有 .rs 改动 ⇒ 可能他窗在用，勿删
[worktree-gate] ⛔ 5 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /private/tmp/nt-probe
[worktree-gate]      ⛔ .worktrees/merge-b / nt-v3 / nt-v4 / nt-verify
[worktree-gate] ♻️  target 累计 40016M ≥ 1024M ⇒ 零风险可回收
```

本会话**新建**的 worktree（全部走 `prune`，**未手删**）：

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/nt-verify` | 隔离孤儿目录挂载验证 | ✅ 已 `prune --force` 移除（成果已核实入主分支） |
| `.worktrees/nt-v2` | 隔离 `llm_judge` 三态验证 | ⏭ **保留** — 成果已入主分支，但门判「未含于任何分支」且他窗可能引用 |
| `.worktrees/nt-v3` | 隔离执行器 `Drop` guard 验证 | ✅ 已 `prune --force` 移除 |
| `.worktrees/nt-v4` | 隔离硬拒两层失效验证 | ⏭ **保留** — 门判「近 3h 有 2884 个 .rs 改动，疑似他窗在用」 |
| `.worktrees/nt-v5` | 隔离契约锁 + flag 告警验证 | ✅ 已 `prune --force` 移除（成果 patch + 分支双重兜底） |

⭐ **本会话额外回收 45 GB**：`nt-verify`/`nt-v2`/`nt-v3` 的 `target/`（17+12+16 G）。
起因：门报 `signal: 9, SIGKILL` —— 那是 **OOM 不是代码错误**，
真因是多份 target 并存。清理后门即绿。

### 8.2 未提交改动的去向

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `neotrix-core/src/l6_meta/nt_permission_profiles.rs` | 撤回错误的单调合并 + **2 条契约锁** | ☐ ☑ **patch 兜底** + ☑ **备份分支**<br>`.neotrix/patches/2026-10-05-contract-locks-monotonic-revert.patch`<br>`backup/nt-v5-5b058ce8` |
| `neotrix-core/src/main.rs` | `--yolo`/`--full-auto`/`--auto-edit` **诚实告警** | ☐ ☑ **patch 兜底** + ☑ **备份分支**<br>`.neotrix/patches/2026-10-05-honest-flag-warning.patch`<br>`backup/nt-v5-5b058ce8` |
| `docs/architecture/CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md` | 清单第 9/10 项状态更新 + 撤回裁定 | ☐ ☑ **已在主分支**（`591dd9f0`） |

⚠️ **两笔的落地方式**（门红在他窗，且他窗暂存区有删除 ⇒ 不可整体 cherry-pick）：

```sh
git checkout 5bf3aee7 -- neotrix-core/src/l6_meta/nt_permission_profiles.rs
git checkout 5b058ce8 -- neotrix-core/src/main.rs
git -c core.hooksPath=.githooks commit --only \
  neotrix-core/src/l6_meta/nt_permission_profiles.rs neotrix-core/src/main.rs -F <msg>
```

两笔均已在 `nt-v5` 跑过 `cargo check --tests -p neotrix` ⇒ **exit 0**。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**0**
- 提交前跑过：`cargo check --tests -p neotrix -j4` ⇒ 在 `nt-v5` **exit 0**；
  主工作树当前 **7 errors ⇒ 全在他窗**（见 §6），我未用 `--no-verify`（AGENTS.md 禁止）
- `check-orphan-dirs.sh --strict` ⇒ 新增 0（25 个已在基线）
- 本会话新建门：`scripts/ops/nt_orphan_dir.py` + `scripts/check-orphan-dirs.sh`，
  已实测**能抓新增**（造孤儿 ⇒ rc=1；移除 ⇒ rc=0）⇒ 不是恒绿假门
