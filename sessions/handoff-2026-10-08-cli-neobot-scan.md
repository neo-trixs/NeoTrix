# 会话交接：CLI 扫描修复 + Captain_Who 吸收（2026-10-08 下午）

> 窗口：neotrix CLI / neobot 完善 + 外部吸收落账。
> 所有代码改动**未提交**（共享 index，禁 `-A`）；patch 兜底已落。
> 改动文件 mtime/diff 均属本会话（13:09–13:40），最后相关 commit 为 `19d18ef9` / `bbeabfb8` / `5d19d81b`。

## 1. 已完成（本窗口）

| 项 | 内容 | 验证 |
|---|---|---|
| N-1 | `neotrix completions` EPIPE panic：改为先生成到 `Vec<u8>` 再写 | `completions bash \| head` 无 panic；`bash -n` 过 |
| N-2 | `neotrix status --json` 纯 JSON 直出 | stdout 纯 JSON，rc=0 |
| B-1/B-2 | `nt_cli` 协议写端：`format_result_envelope`/`append_cli_result`（camelCase `sideEffects`）; `neobot -p` 已写回执 | envelope 文件 camelCase 字段、`normalize_side_effects` 兼容双键 |
| B-3 | `neobot completions <shell>` 5 shell + EPIPE 容错 | zsh `#compdef neobot`、bash `bash -n` 过 |
| B-4 | `neobot ledger --json` / `quota --json` | 有效 JSON、三态措辞各不相同 |
| B-5 | neobot bin 单测 0 ⇒ 9 条 | `cargo test -p neotrix-neobot --bin neobot` = 9 passed |
| Captain_Who 吸收 | 段0–4 全部执行：信号初筛（Apache-2.0/20★/活）、零克隆取源、熔炼成束、四字段矩阵、KB 节点落账 | `neotrix-experience absorb-node` inserted #0；`ABSORPTION-CAPTAIN-WHO-2026-10-08.md` |
| OPEN-DEFECTS | #6/#7 判定为已修、总账对账、clippy N4 登记 | 本文 §3 复核 |
| TODO | N7 任务清单、N6.6 来源指针、N-round 未提交行订正 | 本文 §2 |

## 2. 基线

- `cargo test -p neotrix-neobot --lib`：**609 passed**（606+3 新）
- `cargo test -p neotrix-neobot --bin neobot`：**9 passed**
- `cargo test -p neotrix --bin neotrix`：**41 passed**
- `nt_lock_audit`（neotrix-core/src 与 crates/neotrix-neobot/src）：0 可疑
- `cargo clippy --no-deps -p neotrix-neobot --all-targets`：0 error / 本轮文件 0 命中
- `nt_worktree_gate.sh check`：RC=4（见 §8.1）

## 3. 重要事实订正（已同步进台账）

- OPEN-DEFECTS **#7**：core `distill_output` 死循环已由 `bbeabfb8` 修复（现场重读 `nt_loop_step.rs:156+`，有 `guard` 收敛 + `TAIL_MARK` 兜底）。
- OPEN-DEFECTS **#6**：`maybe_compact_context` 的摘要消耗已由 `record_agent_cost("agent-loop-compaction", …)`（`nt_loop_step.rs:613`）记账，`record_degraded`（:560）另列。
- TODO **1134 行**「本轮所有改动均未提交」已过期：N-round 已由 `5d19d81b`/`19d18ef9`/`bbeabfb8`/nt_store 全量入库。
- `Cargo.lock` 的 `+tokio` 非本会话编辑来源：某 crate 的 Cargo.toml 已含 tokio 但 HEAD lock 陈旧，cargo 自动回写（随 `--only` 或故意 cherry-pick 时一并带走即可，不单列）。

## 4. Captain_Who → neobot 的吸收结论

设计判词（全部**路线图降级**，未改 `.rs`）：

1. `nt_channel_serve` 有界 worker 池 + observer-only 子会话 + `outcome_unknown` 语义位（ledger 加列）；
2. 持久 **active compaction head**（`messages` 加 `compaction_head_seq`，幂等 ALTER）；
3. 不做项：Provider wire 进 application 层、Usage 树层重聚合、未知副作用自动重放。

依据见 `docs/architecture/ABSORPTION-CAPTAIN-WHO-2026-10-08.md`。

## 5. 已知验证缺口（如实记录）

- **neotrix bin 单元 clippy 不可取证**：`neotrix` lib 自带 43161 个 pedantic-denied 基线 error（`lib.rs:22 deny(warnings)` + `warn(pedantic)`），`--bin neotrix` 单元在 lib 失败后不跑 lint ⇒ 「我的 neotrix 改动 clippy 0 新增」**只是 grep 无命中**，不是 clippy 通过。neobot 侧可证实 0 新增。
- Makefile:284 / CI `ci.yml:618` 同口径 ⇒ **clippy 当前不能当绿门**；需要先分 crate 收 pedantic deny。

## 6. 完整剩余任务清单

### 可立即做（零阻塞）
- neotrix 只读子命令 `--json` 矩阵（仅 `status` 已有）；每子命令加 snapshot 行断言。
- `nt_channel_serve` 有界 worker 池 + 子会话 observer-only 投影（吸收 P1-1）。
- ledger 加 `outcome_unknown INTEGER DEFAULT 0` + 恢复路径置位（吸收 P1-2）。
- `messages` 加 `compaction_head_seq` + `maybe_compact_context` 写摘要时更新（吸收 P1-3）。
- `neotrix-types` / lib pedantic deny 分级收口（clippy 可用作门的前提）。

### 需主人裁决
- N6.3 网关依赖口径（axum 0.8 vs 手写 HTTP/1.1）；开工前先跑 `check-feature-gates.sh`。
- P1-3 `capability_invoke` 执行端口 A/B/C。
- N6.1 P1 provider 额度探针（需 owner 指定 provider + 口径）。

### 阻塞 / 未核实
- OPEN-DEFECTS：#3（裁决）、#10（所有者）、#11/#12（需先复现）、#13（定位污染源，需二分测试序）、N1（集成测试禁用原因）、N3（装饰星号，CI 层拦）。
- ROADMAP-REDUNDANCY 的 T0/T1/T2（见 `docs/architecture/ROADMAP-REDUNDANCY-FLAT-MISALIGN-2026-10-07.md` 收尾表）。

## 7. 下一步建议

1. 如需提交本会话改动：`git add neotrix-core/src/main.rs neotrix-core/src/entry/status.rs crates/neotrix-neobot/src/bin/neobot.rs crates/neotrix-neobot/src/nt_cli.rs crates/neotrix-neobot/Cargo.toml Cargo.lock && git commit --only <同一批>`（共享 index，⛔ 禁 `-A`）。
2. 下一窗口开工「P1-1 有界池 + outcome_unknown + compaction head」三联小 PR。
3. clippy 分级收口后再跑 `make clippy`（目前必红）。

## 8. 收工自查（2026-10-08 起**必填**）

### 8.1 worktree 去向

```
[worktree-gate] repo=/Users/neo/Downloads/neotrix mode=check
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
/Users/neo/Downloads/neotrix/.worktrees/evo | d524e278 | HEAD | 0 | 3562M | 3480M | no
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 | 66M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-v2 | 6b57fe08 | HEAD | 0 | 81M | 0M | no
[worktree-gate] ℹ️  主树：38 处未提交 | target 102834M（只报告，不影响退出码）
[worktree-gate] worktree=3 个 | 合计 3709M | target 占 3480M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
RC=4
```

- 本会话**新建 worktree**：无。
- `merge-b` 的 3 处未提交改动**非本会话** ⇒ 未删、未动（其 patch 兜底需由其属主窗口决定）。

### 8.2 未提交改动去向

| 文件 | 改动 | 去向 |
|---|---|---|
| `neotrix-core/src/main.rs` | `status` 子命令加 `--json` | ☐ 提交（建议 `--only`）☐ patch：`.neotrix/patches/2026-10-08-cli-neobot-scan-fixes.patch` ☐ 明确弃用：否 |
| `neotrix-core/src/entry/status.rs` | `show_status(json)` + completions EPIPE 容错 | 同上 |
| `crates/neotrix-neobot/src/bin/neobot.rs` | `-p`、`completions`、ledger/quota `--json`、9 单测 | 同上 |
| `crates/neotrix-neobot/src/nt_cli.rs` | `format_result_envelope`/`append_cli_result` | 同上 |
| `crates/neotrix-neobot/Cargo.toml` | `clap_complete = "4"` | 同上 |
| `Cargo.lock` | `clap_complete` 系 + 预存 `tokio` 陈旧回写 | 同上 |
| `TODO.md`、`sessions/OPEN-DEFECTS.md`、`ABSORPTION-CAPTAIN-WHO-2026-10-08.md`、本文件 | 文档本轮改动 | 建议与代码同批 `--only` |

> 「留给下一个 agent」不算合法去向。上面六条代码**等待用户明确指令才提交**；patch 兜底已落。
