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
| `96056d65` | **方案 A**（用户裁决）：切档改审批模式必须「显式确认 + 留痕」+ **关闭匿名放宽路径** + 6 条锁 | ⭐⭐⭐ 产品语义落地 |
| `591dd9f0` | **`CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md`**：10 项「声称存在但实际不生效」实证 | ⭐⭐⭐ 方法论资产 |
| `0b49366b`/`c24347b8`/`5a3dce85` | `EntropyMonitor` 除零、`VSAEngine` 四级障碍、`llm_judge` 三态裁决 | ⭐⭐ |

## 4. 正在改的文件

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| （无）| 主工作树无我的未提交改动 | — |

## 5. 下一步（按优先级排序）

1. **落地 `5bf3aee7` + `5b058ce8`**（见 §8.2，patch 已兜底，门红在他窗）
2. ~~**P0 需产品裁决**~~ ⇒ ✅ **已裁决并实现**（用户选 A，落地于 `96056d65`）：
   · 新增 `plan_profile_switch`（纯查询）⇒ 调用方先看副作用再决定
   · 新增 `switch_profile_with_audit(name, actor)`，空 actor 被拒
   · **匿名 `switch_profile` 在会改模式的档位上直接 `Err`**
     ⇒「能在无 actor 情况下放宽审批的路径」必须不存在
   · ⏭ **待办**：接 CLI（`plan` → 若 `loosens_approval` 则提示确认 → 执行）；
     当前 `switch_profile` 仍**零生产调用方**，故接线前还需先有 CLI 入口。
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

---

## 📌 补记（2026-10-06 06:50 收尾）

### 晚间落地的 3 笔
| 提交 | 内容 |
|---|---|
| `d68744d3` | 方案 A 的实现主体（`plan` / `switch_profile_with_audit` / 匿名入口关闭） |
| `96056d65` | 补齐 6 条反向锁 + **测试串行化** + 改写 2 处「钉死旧行为」的既有测试 |
| 本次收尾 | 交接文档更新 + worktree 收尾 |

### ⭐⭐⭐ 方案 A 落地过程中抓到的**第三类并发病**
前两类我已遇到过（2026-09-29「暂存区与提交不原子」、他窗 WIP 打断门），
这第三类是**测试之间**的：

**`nt_permission_profiles` 的 13+ 处测试共用两个全局单例**
（`global_profile_manager` / `global_approval`），
而 cargo test **默认多线程** ⇒ 同模块测试互相抢状态
⇒ 表现为**间歇性失败且失败行号漂移**。

**⭐ 关键取证**：我新写的锁④「`plan` 不得改动全局审批模式」首跑就红。
我用**独立探针**（单线程连打 before/after）验证：
```
PROBE after setup mode=Suggest active=nt_shield
PROBE after plan  mode=Suggest      ← plan 确实是纯的
```
⇒ **真因是测试抢全局态，不是被测代码有副作用。**
若不做这一步，我会去「修」一个**正确的** `plan`（R-SCAN-1 的又一次）。

**修法**：本仓无 `serial_test` dev-dependency ⇒ 用标准库 `Mutex` 手写串行化
（`TEST_GLOBAL_STATE`，中毒取内值 —— 守卫本身不含状态，中毒只意味着
另一个测试 panic 过，不该连带阻断）。
**实证必要性**：去掉互斥锁后连跑两次 ⇒ 第一次 **FAILED(18/1)**、第二次 **ok(19/0)**。

### ⭐ 顺带改写两处「钉死旧行为」的既有测试
它们断言的正是方案 A 要消除的行为：
1. `test_global_state_integration` 里的 `switch_profile("developer").is_ok()`
2. 专门测「override 会生效」的那一段
改写后**保留了「功能未被削掉」**：匿名 ⇒ `Err` + 模式不变；
带 actor ⇒ 成功 + **override 依然生效**（`AutoEdit`）。

### 门状态（2026-10-06 06:49 核实）
· `cargo check --tests -p neotrix` ⇒ **exit 0**
· `cargo test -p neotrix --lib nt_permission_profiles` ⇒ **19 passed / 0 failed**
· `cargo test -p neotrix --lib` ⇒ **13358 passed / 1 failed**
  ⛔ 唯一失败是**他窗文件** `l0_substrate/nt_core_platform/mod_orphan.rs`
  （他提交 `64913227`「补上孤儿目录模块盲区」引入），
  **与本会话改动无交集**（`96056d65` 只改 1 个文件）。

### worktree 最终状态
| worktree | 去向 |
|---|---|
| `.worktrees/nt-verify` / `nt-v3` / `nt-v5` / `nt-v7` | ✅ 已 `prune --force` 移除（成果均已入主分支或 patch 兜底） |
| `.worktrees/nt-v2` | ⏭ 保留（门判 HEAD 未含于任何分支；成果已入主分支） |
| `.worktrees/nt-v4` | ⏭ 保留（门判「脏 + 疑似他窗在用」） |
| `.worktrees/merge-b` | ⏭ 保留（**他窗**的 worktree，非我创建） |

⭐ 备份分支（成果的第二重保险，HEAD 被 rebase 后 `is-ancestor` 会失真）：
`backup/nt-verify-01ec33b9` · `backup/nt-v3-26744a78` · `backup/nt-v4-307499b9`
· `backup/nt-v5-5b058ce8` · `backup/nt-v7-940b157a`

### 本轮新增的 patch 兜底
· `.neotrix/patches/2026-10-05-profile-switch-confirm-planA.patch`
· `.neotrix/patches/2026-10-06-planA-mutex-and-lock-rewrite.patch`
· `.neotrix/patches/2026-10-05-contract-locks-monotonic-revert.patch`
· `.neotrix/patches/2026-10-05-honest-flag-warning.patch`

---

# 追加：2026-10-06 授权/沙箱双轴收敛 + profile CLI 接线

用户裁决：① `--sandbox` 走**方案 A**；② 审批模式收敛成**一个 flag**。

## 本轮三笔提交（均在 `feat/capability-absorb-20260828`，未推送）
| 提交 | 内容 |
|---|---|
| `2169b633` | 审批/沙箱双轴收敛 + 「启用即断言接线」测试（7 条 + 沙箱 8 条） |
| `145c33d7` | `profile` 子命令接线（方案 A 从库函数变成命令行可达）+ 授权判据反向锁 6 条 |
| `98395ab7` | 更新 `CLAIMED-BUT-NOT-ENFORCED` 第 9/10 项（含「未闭环」的诚实标注） |

## 修掉的两个真实缺陷（都是「静默失效」族）
1. **`--sandbox` 未知值静默兜底成 `Disabled`** ⇒ `--sandbox danger-full-access`
   不报错、静默变成「不设限」，**语义与意图相反**。改为 `Err` + 退出码 2。
2. **`ApprovalMode::from_str` 返回 `Option`** ⇒ 档位里
   `approval_mode_override` 拼错一个字母 ⇒ 覆盖被**无声忽略**。
   注意它「看起来更严」（停在 Suggest）⇒ 实则用户要的自动批准没生效且无人知道。
   改为 `Result`。

## ⭐ 可复用的判据
- **静默失效只允许朝严格方向回落**。「未知 → Disabled」是朝宽松方向 ⇒ 直接违反。
- **安全边界必须做成纯函数**。`authorize_profile_use` 决定「要不要问一句」，
  内联在 I/O 里就**测不到** ⇒ 改松了没人知道。
- **穷举优于逐格**。授权真值表逐格测容易漏一格，而漏的那格恰好是
  「CI 里静默放宽审批」⇒ 用循环穷举全 8 格，钉住「恰好一格 Refuse」。

## ⚠️ 夹具坑（下一个人会踩）
macOS `script -q /dev/null cmd` **不把管道输入送进子进程 pty**（`^D` 早于输入到达）
⇒ 测出「输入 `y` 也被取消」的**假阳性**。
按 R-SCAN-1 改用 python `pty.fork()` 精确驱动后确认**代码无缺陷**。
**不修正它就会去「修」正确的代码** —— 错误的测试结论比没有记录更危险。

## 8. 收工自查

### 8.1 worktree 去向
```
[worktree-gate] repo=/Users/neo/Downloads/neotrix mode=check
------------------------------------------------------------
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
--------------------------------------------------------------------------
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 | 66M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-v2 | 6b57fe08 | HEAD | 0 | 81M | 0M | no
[worktree-gate] ℹ️  **主树**：13 处未提交 | target 239869M（**只报告，不影响退出码**）
[worktree-gate]    ⛔ 主树未提交改动**不在任何提交里**（AGENTS.md §1 收工义务）
[worktree-gate]    提交：git add <显式路径> && git commit --only <同一批>（⛔ 共享 index 下禁 -A）
------------------------------------------------------------
[worktree-gate] worktree=2 个 | 合计 147M | target 占 0M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
```
本会话**新建**的 worktree：**无**（全程在主工作树 + `--only` 定点提交）。

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/merge-b` | **非本会话创建**（他窗） | ⛔ 含未提交改动（含 1 处 staged 删除 `proxy_daemon_wrapper.rs`）⇒ **不得 prune、不得手删**，移交他窗处理 |
| `.worktrees/nt-v2` | 非本会话创建 | 无未提交改动，本会话未触碰 |

### 8.2 未提交改动的去向
本会话触碰的 6 个文件（`main.rs`、`nt_sandbox.rs`、`nt_core_approval.rs`、
`nt_permission_profiles.rs`、`nt_approval.rs`、`CLAIMED-BUT-NOT-ENFORCED-*.md`）
**全部已进入上述 3 笔提交**，`git status --porcelain -- <这些文件>` 输出为空。
⛔ 主工作树仍有**他窗 WIP**（非本会话产生），按共享 index 事故纪律**未触碰、未提交**。

### 8.3 用户侧副作用
测试动过 `~/.neotrix/profiles.toml`（`switch_profile_with_audit` 会落盘）
⇒ 已备份 `/tmp/profiles.bak.toml` 并**逐字节复原**（`diff` 干净，`active = "nt_shield"`）。

## ⏭ 接手者的下一个动作
1. ⛔ **第 9 项仍未闭环**：`require_approval` 的两个消费者
   （`ShieldEnforcer` 悬空 + `turn_stream_with_approval` 零调用方）
   仍不在生产链上 ⇒ 「flag 落地 ⇒ 被工具执行消费」那一段仍然断。
   **不要因为 flag 已收敛就把这一项划掉。**
2. `EffortTier::from_client_spelling` 仍**无生产调用方**（仅测试）⇒ 待接 gateway。

---

# 追加：2026-10-06 五源吸收（uber/ADR 为主）+ 三处缺陷修复

用户提交 5 源并要求「吸收，补齐 neotrix 缺陷」。**先过 LICENSE 前置门**：
MangoDisk 与 openhuman 均 **GPL-3.0 ⇒ 不取码**；Jev-Mem / Codewhale 为 MIT（只取设计）；
**uber/ADR 为 Apache-2.0 ⇒ 可取码**，是本轮主要来源。

## 本轮四笔提交（`feat/capability-absorb-20260828`，未推送）
| 提交 | 内容 |
|---|---|
| `408d23c1` | ⭐ 审计不再抄录文件内容（含密钥）+ 修 `action_verdict` 丢 AutoEdit 白名单 |
| `9fa2af7b` | `--sandbox` 接到活对象 + **如实记录**两处 sandbox 闸语义漂移（不擅自统一） |
| `39f37509` | `init_sandbox` 同时推进 `global_shield()` |
| `2714681a` | 五源吸收判定文档 + 台账 525 条 + 第 9 项升级为**实测结论** |

## ⭐ 修掉的三个真实缺陷
1. **审计抄录文件内容**：`describe_action` 把 `content_preview`/`diff` 原文（≤60 字符）
   拼进 `description`，而它被 `ApprovalAuditEntry` **快照进审计轨迹**
   ⇒ 写 `.env` 时那 60 字符**就是密钥**。改为 **`<N> chars, sha256:<12hex>`** 指纹
   （源：uber/ADR `run_manifest` 明确 *file contents are not stored*）。
2. **`action_verdict` 丢 AutoEdit 文件类白名单**：它是 `ActionSandbox` 的硬拒判据，
   却把同一动作判成与 `require_approval` **不同**的答案 ⇒ AutoEdit 下所有文件写全被当 Ask。
   改为委托 `require_approval`（判据唯一真源）+ 穷举 3 档 × 6 动作的一致性锁。
   ⚠️ 该函数此前**零测试**。
3. **`--sandbox` 完全无效**：`init_sandbox` 只写 `global_sandbox()`，
   而实测该单例**模块外零读者**。改为同时推进 `global_shield()` 的活对象。

## ⚠️ 本轮「刻意没做」的一件事（需要产品裁决）
`check_all` 与 `check_cli_command` 的 sandbox 闸语义**不一致**
（前者拦一切、后者只拦写），且前者被既有测试**显式断言**（*"should block even reads"*）。
我一度改成「只拦写」，**随后撤回**，三条实测理由：
1. 对活路径**零效果**（唯一调用方 `seal_iterate` 本就登记为 `irreversible`）
2. 默认档**根本走不到** sandbox 段（`SecurityGuard` 第 1 段先短路）
3. 改它 = 单方面翻转被测试钉住的安全语义；「read-only 该不该拦读」是**产品判断**
⇒ 代码里已写入完整裁决依据 + 7 条按**实测**（非我期望）写的锁。

## ⭐⭐ 环境事故：他窗 `cargo clean` 两次抽走主 `target/`
症状：`can't find crate for hashbrown` / `extern location for libc does not exist` /
`failed to write … .fingerprint/… No such file or directory`。
⇒ **这些不是代码错误。** 应对：改用私有 `CARGO_TARGET_DIR=/tmp/nt-target-private`
（11G，他窗再也删不到）。⚠️ 拷贝热缓存失败（拷贝期间目录正被删，只拿到 170M），
冷构建约 3 分钟可接受。
**下一个人若在共享工作树编译报上述错误，先 `ps aux | rg cargo clean` 查他窗，不要
去「修」代码。**

## 三条可复用判据
1. **审计/日志记摘要不记内容**（源 uber/ADR）
2. **遮蔽误伤的代价 = 审批失效**，比漏遮更难发现 ⇒ 遮蔽器必须配反向测试
   （`KEYBOARD=1` 被误遮就是这么抓到的）
3. **解析/输出失败必须朝严格方向**（escalate/`Err`），不得回落成放行

## 8. 收工自查

### 8.1 worktree 去向
```
[worktree-gate] repo=/Users/neo/Downloads/neotrix mode=check
------------------------------------------------------------
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
--------------------------------------------------------------------------
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 | 66M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-v2 | 6b57fe08 | HEAD | 0 | 81M | 0M | no
[worktree-gate] ℹ️  **主树**：14 处未提交 | target 14017M（**只报告，不影响退出码**）
[worktree-gate]    ⛔ 主树未提交改动**不在任何提交里**（AGENTS.md §1 收工义务）
[worktree-gate]    提交：git add <显式路径> && git commit --only <同一批>（⛔ 共享 index 下禁 -A）
------------------------------------------------------------
[worktree-gate] worktree=2 个 | 合计 147M | target 占 0M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
```
本会话**新建** worktree：**无**。

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/merge-b` | **非本会话创建**（他窗） | ⛔ 含未提交改动 ⇒ **不得 prune、不得手删**，移交他窗 |
| `.worktrees/nt-v2` | 非本会话创建 | 无未提交改动，本会话未触碰 |

### 8.2 未提交改动的去向
本会话触碰的文件（`nt_approval.rs`、`nt_shield_enforcer.rs`、`nt_sandbox.rs`、
`repos.csv`、`CLAIMED-BUT-NOT-ENFORCED-*.md`、新增吸收文档、handoff）
**全部已进入上述 4+1 笔提交**，`git status --porcelain -- <这些文件>` 为空，
暂存区亦为空。
⛔ 主工作树仍有**他窗 WIP**（`agent_guardrails/input_validator.rs` 等），未触碰未提交。

### 8.3 用户侧副作用
本轮**未触碰**用户配置（`~/.neotrix/profiles.toml` 未改动）。
新增 `.neotrix/patches/2026-10-06-verdict-and-sandbox-gate.patch`（改动中途的兜底，
最终已入提交，可留作对照）。

## ⏭ 接手者的下一个动作
1. **需要产品裁决**：`read-only` sandbox 该不该拦读（两处语义漂移，代码里已备好依据）。
2. **第 9 项仍未闭环**：`--approval-mode` 的 `Ask` 档在**生产工具执行链**上无绑定点。
   三条硬约束与三条收口路径见 `CLAIMED-BUT-NOT-ENFORCED-2026-10-05.md` 第 9 项。
   ⛔ **不要因为本轮闭环了两条就把本项划掉。**
