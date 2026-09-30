# Handoff — 静默失败契约收尾 + IM outbox 毒行修复（2026-09-30）

## 1. 会话标识

- 窗口：s000（延续会话）
- 分支：`feat/capability-absorb-20260828`（HEAD `68e8d0fd`，未推送）
- 交接时间：2026-09-30

## 2. 目标（一句话）

> 把静默失败 38 条契约全部裁决闭环，并核实/修复 IM outbox 的三个历史缺陷指控。

## 3. 已完成

- [x] 契约全量裁决：`scripts/silent-failure-baseline.txt` 32 条 **0/32 OPEN**（此前 35/38 待裁决）。裁决分三类：修掉的真缺陷 / 写-only 审计轨迹且无仓内读取方 / 快照语义+读侧优雅降级；`autofixer.rs:55` 明确记为已知局限（tracker API 强制 `()` 返回）。
- [x] 修 5 处：`crystal_state.rs` 代际平移 rename 失败补 `log::error`；`experience_tree` feedback 路由表写结果直接作 bool 返回；`domain_modeling` ADR 落盘失败返回既有 `Skipped{reason}` 变体；`firewall.rs` 两处 pf 规则写失败转 Err（消掉过期规则集+成功 pfctl 的静默错误规则角落）；`cli.rs` overlay 写失败转 Err（注释承诺手动变更永不丢失）。
- [x] 修 `knowledge_assets.rs:321` 导入报告虚报（元数据写失败仍 `imported += 1`）；修 `kb_write.rs` 决策溯源丢弃；修 GraphRAG 丢边 2 处（absorber/pipeline）。
- [x] **IM outbox 毒行修复**（`crates/neotrix-neobot`）：`nt_agent.rs:393/420` 写无 channel 的 `CH_MESSAGE_NEW` 行（全仓零消费者）+ drainer 无限退避。修法：drainer 按 topic 路由，只有 `CH_CHANNEL_SEND` 有发送方；确定性坏行删+`eprintln!` 留痕；删两处毒写入。**未注册渠道仍退避重试**（测试 `outbox_drain_does_not_drop_unknown_channel` 锁定，行为不变）+ 3 个新回归测试。`neotrix-neobot --lib` **457 绿**。
- [x] unwrap 基线 7 行随迁改写（skill_tree ×5 为 +4 行均匀位移、内容一致；knowledge_assets ×2 经 `git log -S` 证伪为 477bf669 起即存在）。剩余 4 NEW 全部是他窗文件，不代改不记账。
- [x] `check-doc-claims.sh` 修空跑绿灯（此前检查 0 处却 PASS）+ 4 个抽取 bug（`:line` 后缀、CamelCase、层相对路径、`git grep -E` 的 `\b` 未定义）+ 消费者判据误报（声明≠消费、同名≠同符号）+ basename 冲突跳过。现状态：检查 1 处、跳过 2 处并明示，注入探针 FAIL/还原 PASS 已验证。
- [x] 新门 `check-silent-failure.sh`（44→38→32 条基线+棘轮，已进 CI）与 `check-feature-gates.sh`（6 feature 全绿；CI matrix 从 `--lib` 改 `--all-targets`）。
- [x] 否证记录 3 篇：`CALLGRAPH-FEASIBILITY`（HIR 文本 31KB/行→27GB 不可行；抽取器找错字段）、`CAPABILITY-GAP`（G1–G7）、doc 承诺对账门不可行（2423 误报）。

提交（本会话，均仅本地）：`bd5d25b1`（5 处修复+契约裁决+unwrap 随迁）、`68e8d0fd`（outbox 毒行）。

## 4. 正在改的文件（关键！逐个列）

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| 无 | 本会话改动已全部提交（`bd5d25b1`、`68e8d0fd`） | — |

## 5. 下一步（按优先级排序）

1. `edit_of` 真正生效（TODO.md #6）：需占位消息 + `nt_store` schema 变更。`nt_channel.rs:127` 记录准确。属 P1，有迁移风险，未动。
2. IM `/stop` worker 池（TODO.md #7）：设计在 `DESIGN-CHANNEL-DISPATCH.md` §11/§13；分支 `feat/im-stop`（worktree `.worktrees/nt-stop`，干净）可作起点，但已落后主分支，需先合并。
3. 212 死文件 / 19 层债 / 352 组重复类型：维持不动（判据不可靠，无可靠门）。
4. `.worktrees/merge-b` 的 4.7G target：`nt_worktree_gate.sh clean` 可回收，但 worktree 疑似他窗在用，先喊一声再动。

## 6. 阻塞点

- 无。门状态见 §8.3。`check-unwrap --strict` 红的 4 处全是他窗文件（apps ×2、nt_pet.rs ×2），不代改。

## 7. 给接手会话的话

- 恢复命令：先读本文件，再跑 `git status --short` / `git diff --stat` 核对。
- 禁止事项：不要碰 `apps/neobot-desktop/`、`crates/neotrix-neobot/src/nt_pet.rs`、`nt_media/`、`merge-b` worktree（均为他窗在途）；提交用 `git commit --only <文件>`，禁 `-A`。
- 风险提示：他窗正在改 `crates/neotrix-neobot/src/lib.rs`、`nt_store/mod.rs`、`nt_store_convos.rs`（约 1.4h 前 mtime）+ 新增 `nt_store_messages.rs`。我的 outbox 改动（ledger 新增方法、drainer、nt_agent）在不同文件，无冲突，但若他窗改动 ledger/dispatch/agent 三文件需先通气。
- Shell 地雷（本会话实测）：zsh 下 `echo =====` 会触发 `=cmd` 路径展开而报错；一律用引号包裹分隔符。

## 8. 收工自查（必填）

### 8.1 worktree 去向

- `sh scripts/ops/nt_worktree_gate.sh check` 输出：

```
[worktree-gate] repo=/Users/neo/Downloads/neotrix mode=check
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
/private/tmp/nt-v9 | a005db44 | HEAD | 3 | 51M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 | 4739M | 4673M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-stop | b9be70d9 | feat/im-stop | 0 | 50M | 0M | no
[worktree-gate] worktree=3 个 | 合计 4840M | target 占 4673M
[worktree-gate] 带未提交改动: 2 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ 2 个 worktree 的未提交改动**不在任何提交里**
```

- 本会话**新建**的 worktree：无（0 个）。

| worktree | 用途 | 去向 |
|---|---|---|
| `/private/tmp/nt-v9` | 非本会话创建，他窗 | 不动 |
| `.worktrees/merge-b` | 疑似他窗在用（13:08 有 git 操作史）+ 3 个脏文件 | 不动，未 prune |
| `.worktrees/nt-stop` | IM 隔离开发（干净，分支 `feat/im-stop`） | 保留不断（干净无丢失风险；分支保留供后续 #7） |

- 未手删任何 worktree 目录。

### 8.2 未提交改动的去向

本会话自己的改动**已全部提交**（`bd5d25b1`、`68e8d0fd`，`git status` 已核无己方残留）。

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `apps/neobot-desktop/**`（约 20+ 文件 M） | 他窗在途 | ☐ 非本会话改动，不动 |
| `crates/neotrix-neobot/src/{lib,nt_store/mod,nt_store_convos}.rs` M + `nt_store_messages.rs` ?? | 他窗在途（mtime 约 1.4h 前） | ☐ 非本会话改动，不动 |
| `nt_media/**`、`skills/assets/**`、`scripts/ops/nt_check*.mjs`、`results.tsv`、`Cargo.lock` 等 | 他窗在途 | ☐ 非本会话改动，不动 |
| `neotrix-core/tests/*.disabled` ?? ×2 | 未知来源 | ☐ 非本会话创建，不动 |

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：0（check 模式；⛔ 告警见上，不删）。
- 提交前跑过：`cargo test -p neotrix --lib` **12209 passed / 0 failed**；`cargo test -p neotrix-neobot --lib` **457/0**；`cargo check --all-targets` 0 error。
- 门：`check-silent-failure --strict` rc=0（0/32 OPEN）；`check-layer-deps --strict` rc=0；`check-doc-claims` rc=0（checked=1）；`check-doc-drift` 0 死链；`check-feature-gates` rc=0（6/6）；`check-gate-satisfiable` rc=0；`check-ci-refs` rc=0；`ci.yml` yaml 合法。
- 红门 1 处：`check-unwrap --strict`（4 NEW 全是他窗文件，非本会话引入，不代改不记账）。
- pre-commit 钩子：build gate ✅、doc-drift ✅（0 死链）。
