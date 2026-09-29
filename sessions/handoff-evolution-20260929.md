# 收尾交接 —— 进化实验接线 + L1 备件清理（2026-09-29）

## 一句话

进化实验判决链从「两套实现 + 零消费者」变成「唯一实现 + CI 在跑」；
L1 三个零消费者备件已删（-1218 行）；**遗留 1 个他窗引入的测试失败未解**。

## 落地清单（全部已提交）

| 提交 | 内容 |
|---|---|
| `2f11389f` | 统一判决：删 Python 版，Rust bin 成唯一真源 |
| `c95675ee` | bin 注入 `ExperimentRunner`（重复/统计/账本归位） |
| `ea827bed` | `min_effect` 由 `1/CASES.len()` 导出（曾被硬编码架空） |
| `fbee77fd` | 提交纪律：必须用 `git commit --only`（含为什么门防不了） |
| `44c9f5f4` | 进化账本门接进 **CI**（我上轮说"接线完成"是假的） |
| `7148fbc5` | `strictest_verdict` 最严否决（修 `SecurityRouter` fail-open） |
| `f75f80b7` | 补回被吞的 87 行 + 门禁化「重复吞内容」 |
| `3aec9cae` | 删 `security.rs`（448 行零消费者） |
| `2e0e8255` | 删 `tool_registry.rs`（770 行零消费者） |

新增：`scripts/check-evolution-ledger.sh`（账本活性门，4 场景双向实测）

## ⛔ 唯一遗留：1 个测试失败，**他窗引入，我未处理**

`neotrix::nt_crystal_core::nt_jev_calibration::tests::test_decide_calibrated_identity_matches_plain`

**逐提交二分取证**（干净 worktree，非工作区推测）：

| 提交 | 结果 |
|---|---|
| `2e0e8255`（我的删除） | **PASS** |
| `ce794ff7` | **PASS** |
| `f78b56a0 fix(crystal): keywords() 对中文近乎失明` | **FAIL** ← 根因 |
| `6332ea67`（他窗 CJK 八副本收敛） | FAIL |
| HEAD | FAIL |

⇒ 引入者是 `f78b56a0`（`consciousness.rs` +163 行）。
该测试用中文记忆（`"火焰 燃烧 释放 热量"`），CJK token 化改变后
`decide_calibrated` 的校准路径行为变了。
⇒ 他窗当时**只验了自己的 6 个测试**，未跑全量，故未发现。
⇒ 后续 `6332ea67` / `044a9dd2` 做了 is_cjk 收敛，**但该测试仍红**。

**为什么我不管**：
① 不属我的改动；② 擅自改他的代码 = 把他窗 WIP 变成我的错；
③ 把它记进 baseline = 把问题变成谎言（`check-test-baseline.sh` 头注明确禁止）。

**两条修法（属他窗决定）**：
- 修 CJK token 化，让断言恢复
- 或确认新行为正确后，更新该断言

⇒ 影响面：`check-test-baseline.sh --strict` 因此为红，会卡提交。
⇒ 修好后 baseline 保持 0 字节即自动转绿（**不要往 baseline 里记账**）。

## 三个已删除的 L1 备件（为何该删）

| 文件 | 行数 | 判据 |
|---|---|---|
| `nt_act/tool_registry.rs` | 770 | `mod.rs` **从未注册**；grep 的 28 个消费者全是**同名不同型** |
| `nt_act/actions/security/security.rs` | 448 | 外部消费者 0、tests/ 引用 0 |
| ~~`nt_act/tool_contract/mod.rs`~~ | 95 | ⛔ **恢复** — 被 `tests/phase_integration_tests.rs` 8 处引用 |

⛔ **保住的**（有活路径，删了会打断）：
`actions/security/sandbox.rs`（`ActionSandbox`，L5 26 消费者）
`actions/security/disk_guard.rs`（`RiskLevel`，6 消费者）

## 本轮踩到的三个判别陷阱（留给下一个人）

1. **符号零消费者 ≠ 模块零消费者**
   我删 `nt_action_facade.rs`（"`ActionFacade` 零消费者"）⇒
   `nt_media/playback.rs:6` 依赖其 `source::types::MediaItem`。编译当场报 `E0583`。
   ⇒ 删**模块**前查 `mod <name>` 注册 + `use ...<name>::`。

2. **同名不同型**（`DIR-AUDIT` 记的病）
   grep `ToolRegistry` 报 28 个消费者，实际全是 `l5 nt_core_gate::ToolRegistry`
   与 `l2 crawl::ToolRegistry`。⇒ 判据是「`mod.rs` 有没有注册」，不是「grep 到几个」。

3. **漏查 `tests/` 层**
   我删 `tool_contract` 只查了 `src/`，被 pre-commit 编译门当场抓住。
   ⇒ 删任何东西前，判据必须含 `neotrix-core/tests/` 与 `crates/*/tests/`。

## 共享工作树纪律（今天实测三次）

- 提交用 `git commit --only`（核对与提交**不原子**，我两次被并发插入/覆盖）
- 提交后**立即核行数**：`wc -l` 下降 > 5 行就停下来查（我今天吞过 62 + 87 行）
- 删除需 `DELETION-INTENT:`；`--only` 场景下门读 `COMMIT_EDITMSG`，
  `-F` 不写该文件 ⇒ 解法 `-F` + 预写 `COMMIT_EDITMSG` + `-c core.editor=true`
- 恢复文件用 `git checkout HEAD~N -- <path>`（从对象库），
  **不要** `git show > <path>`（经工作区，共享树下不可靠）

## 未做的（需产品判断，不擅自决定）

- **`ActionFacade` / `AsyncToolExecutor` 仍是备件**（登记为 `N-12b`）：
  `ActionFacadeConfig` 全仓唯一构造点在 `test_facade()` 里 ⇒ 生产侧无组装点。
  ⛔ 不是"再接一次"能解决，缺的是**接线机制**。
- **L5 `experiment.rs` 决策**：`design_ab_test` 与 L6 功能重复，已删 314 行；
  剩余是否补接属产品判断。
- **`crates/neotrix-neobot` 的安全门**（`nt_policy.rs::evaluate_policy`）
  **已完整接线**（`nt_agent.rs:590` `gate()` → 先审计后执行）⇒ 不需要动。

## 验证基线（收工时）

- `cargo check --tests -p neotrix` → **0 error**
- `cargo test -p neotrix --lib` → **12207 passed / 1 failed**（那 1 个见上）
- `scripts/check-evolution-ledger.sh` → 0
- `scripts/check-test-baseline.sh --strict` → 1（因那个遗留失败）
- 临时 worktree 已全部清理
