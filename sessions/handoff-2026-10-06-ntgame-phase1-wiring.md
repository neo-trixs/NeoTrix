# 交接：NT-GAME 阶段 1 接线 + 合约补全（D1–D6）

## 1. 会话标识

- 窗口：space-bunny（primary dev agent）
- 分支：`feat/capability-absorb-20260828`
- 交接时间：2026-10-06

## 2. 目标（一句话）

> 把 `nt_game` 收敛成「自我进化的整套游戏构建能力」：先接线、再补合约缺陷、然后才删冗余。

**顺序不可颠倒** —— 审计原本判「删 10,684 行死抽象」，实测证明那是错的：那些代码不是死代码，是**给缺失合约打的补丁**，直接删会把补丁连同它掩盖的能力一起删掉。

## 3. 已完成

### 3.1 阶段 1：活路径接线（删两份重复实现）

- [x] `handlers_game.rs`（409 → 162 行）：删除自带的 `GameTickReport` + `_GameTrainingDaemon` + `_GameDaemonState` + `run_tictactoe`/`run_2048`/`run_hex_crucible` + `rng_step`，改调 `nt_game::evolution::GameEvolutionLoop`。**活路径** `run.rs:903` 每 300s 触发的 `handle_game_training()` 现在真正跑到 `nt_game` 上。
- [x] `evolution.rs`：删除 `trait AutoGame` + `AutoTicTacToe`/`Auto2048`/`AutoHexCrucible`（约 530 行自含副本），`create_game()` 改返回 `Box<dyn NtGameEnv>` 指向**正式游戏**。
- [x] 3 条旧测试改打正式游戏 + 新增 6 条 handler 测试（`test_games_are_the_canonical_ones` 是「副本已消失」的判别力证据）。

### 3.2 合约补全（实测驱动，非推演）

删副本会掩盖能力缺失，必须同时补合约。全部有实测数据支撑：

| # | 缺陷 | 实测证据 | 修法 |
|---|---|---|---|
| **D1** | `NtGameEnv` 无终局出口 ⇒ 胜率恒 0 | 探针 `win_rate ≡ 0.000`、`scores={0:0.0,1:0.0}`；三局**内部早有** `winner`（`HexTicTacToe.winner` / `HexCrucibleState.winner` / `determine_winner()`）只是没出口 | 新增 `GameOutcome` + `outcome()`，三局各实现。**修后** `c=0: win_rate=0.900`、`c=2: 0.333` |
| **D2** | `HexTicTacToe`/`Game2048` 未定义 phi，吃默认 `0.0`，把缺口洗成数据 | `phi_avg ≡ 0.0000` 且无法区分「真是 0」与「没定义」 | `phi_contribution()` 改 `-> Option<f64>`（`None` = 未定义）+ 两局补实现（棋盘 hexagram 多样性 / score÷2048） |
| **D3** | `health = win_rate*0.6 + phi*0.4` 结构退化 | 零和对称局胜率≈0.5；2048 记分局胜率无意义 | **保留公式**、加事实注释（改它需先修 D5，否则只是换噪声）。未发明新公式 |
| **D4** | `hex_crucible.rs:457` 下溢 panic | 实测 `attempt to subtract with overflow`，`constellation≥2` 必踩。此前是死代码所以从未触发 | 差值只算一次 `saturating_sub`，两处共用 |
| **D6** | 两套回合预算互相矛盾 ⇒ c=4/5 永不到终局 | `for_constellation` 给 20/30/40/50/60/**80**，外层硬帽 `max_turns=50` ⇒ 实测 `turns≡50.0`、`30/30` 全 draw、`win_rate≡0` | 新增 `turn_budget()`，有效预算 `= max(安全帽, 本局预算)`。**修后** `c=4: turns=60.0, win_rate=0.400`；`c=5: turns=80.0, win_rate=0.400` |

- [x] `GameEvolutionState` + `GameTickReport` 新增单调 `total_ticks`/`tick`（渲染层调研的「响应带单调序号」纪律同源）。

### 3.3 回归护栏（防复发，3 条永久测试）

`test_every_constellation_reaches_a_verdicts` / `test_high_constellation_budget_exceeds_flat_cap` / `test_all_games_declare_phi_or_honestly_none` —— 用**不变量**而非具体数值（钉死随机胜率会逼后人调种子造假）。

### 3.4 验证

- `cargo check -p neotrix --lib`：**CLEAR**
- `cargo test -p neotrix --lib game`：**458 passed / 0 failed**
- `cargo test -p neotrix --lib` 全量：**13399 passed / 2 failed / 38 ignored** —— 2 条失败在 `l0_substrate/nt_judge.rs::m4_ledger_tests`，是**他窗未提交 WIP**（`git status` 显示 `M`，最近提交 `1e6602a6` 是 EVO 工作），与本会话无关。
- `python3 scripts/ops/nt_lock_audit.py neotrix-core/src`：**可疑 0 处，RC=0**

## 4. 正在改的文件

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/env.rs` | 新增 `GameOutcome`；`outcome()`/`turn_budget()` 默认 `None`；`phi_contribution()` 改 `Option<f64>` | ✅ 是（已验证） |
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/evolution.rs` | 删 `Auto*`；`create_game` 指向正式游戏；切到 `outcome()`；修 phi 分母；3 条新测试 | ✅ 是 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/hex_crucible.rs` | **D4** 下溢修复；实现 `outcome()`/`turn_budget()`/`phi→Some` | ✅ 是 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/builtin/hex_tictactoe.rs` | 实现 `outcome()`/`phi_contribution()` | ✅ 是 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/builtin/game_2048.rs` | 实现 `outcome()`/`phi_contribution()` | ✅ 是 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind_background_loop/handlers_game.rs` | 接线 + 6 条测试 | ✅ 是 |

⚠️ 6 个文件是**一个原子变更**（trait 签名改动互相咬合），必须同一提交，不能拆。

## 5. 下一步（按优先级）

1. **【最高】补跑 feature 门并如实记录**（见 §6 阻塞点）：`bash scripts/check-feature-gates.sh --quick`。本会话改了 trait 签名，**早先那次 PASS 已失效**。
2. **【最高】D5：假策略**（唯一剩余的真实缺陷）。`evolution.rs::tick()` 的动作选择仍是
   ```rust
   let idx = ((ep_seed.wrapping_add(steps as u64)) % actions.len() as u64) as usize;
   ```
   注释自称 `simulating a policy network` —— **对种子取模**，不学习/不记忆/不依状态 ⇒ 自对弈产出**纯噪声** ⇒ `win_rate`/`avg_reward` 无统计意义，`advance_threshold` 无法被真实跨过 ⇒ **constellation 进阶仍不会真正发生**。
   ⇒ 仓内已有真机器：`nt_game/play/self_play_loop.rs`（`SelfPlayLoop`/`SelfPlayConfig`/`TrainingMetrics`）＋ `advantage.rs`/`grpo_adapter.rs`/`adaptive.rs`。收敛计划本就判其「保留」，但**目前同样未被本循环调用**。接线即可让 D3 的公式有可解释基础。
3. **阶段 2：删冗余**（必须在 1、2 之后）。D1–D6 已证「审计判的死代码有一部分是补丁」，故删除清单需**重新审计**：先跑 `rg` 逐块确认零消费方，再用测试数预测证伪（原估 449→152，现基线 458）。
4. **阶段 3：渲染契约**。新增 `nt_wire.rs`（结构化场景图 + 单调 `rev` + 资产 ID + 网格坐标 + 保留 `text_state`），复用 `l1_action/nt_io/nt_io_web/server.rs:207` 已有的 axum ws handler；扩展 `mcp.rs::observe()`（现返回 `String`）。选型见 `NT-GAME-CONVERGENCE-2026-10-06.md`（React+DOM／Phaser4／R3F；不引入 Bevy/macroquad）。

## 6. 阻塞点

- **🔴 feature 门未闭合（必须接手者补）**：本会话早期在**改动之前**跑过 `check-feature-gates.sh --quick` = PASS（6 个非默认 feature 全通）。随后改了 `NtGameEnv` trait 签名与新增 `GameOutcome`，重跑时被用户中止。⇒ **当前提交未经验证 feature 门覆盖**，不得引用早先那次 PASS 作为依据（R-SCAN-3：门记录必须带核实时间戳且对应被测状态）。
- 无其它阻塞。共享树当前不稳定来源为 `nt_judge.rs`（他窗 WIP），会随其提交自愈。

## 7. 给接手会话的话

- 恢复命令：先读本文件，再 `git status --short` / `git diff --stat` 核对。
- **禁止**跑 `cargo check --all-targets` 或 `--test` 全量构建（16G 机必爆 swap）；只用 `cargo check -p neotrix --lib` 与 `cargo test -p neotrix --lib`，且经 `bash scripts/ops/nt_build_lock.sh --strict --timeout <秒> -- <cmd>` 串行。
- **提交必须 `git commit --only <6 个路径>`**（共享 index 下「核对暂存区」与「提交」不原子）；新文件先 `git add --intent-to-add`。
- **禁止在共享树 `git commit --amend`**（本会话早前误碰他窗 `6440106`，树哈希证明是空操作并已恢复）。
- 他窗在改的文件（**勿碰**）：`neotrix-core/src/l0_substrate/nt_judge.rs`、`.neotrix/capability_registry.json`、`.neotrix/capability_overrides.json`。

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出：

```
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 1 个
[worktree-gate] ⚠️  1 个 worktree 近 3 小时仍有 .rs 改动 ⇒ 可能他窗在用，勿删
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**：
[worktree-gate]      ⛔ /Users/neo/Downloads/neotrix/.worktrees/merge-b
[worktree-gate]    删它们必须先 patch 兜底（R-DISK-5）：sh scripts/ops/nt_worktree_gate.sh prune
[worktree-gate] ♻️  target 累计 3480M ≥ 1024M ⇒ 零风险可回收：sh scripts/ops/nt_worktree_gate.sh clean
RC=0
```

- 本会话**未新建**任何 worktree。

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/merge-b` | **他窗的**，非本会话 | ⛔ 勿动（近 3h 仍有 .rs 改动）。未提交改动须由该窗自行 commit/patch |

### 8.2 未提交改动的去向

本会话改动 = 6 个 `.rs`（见 §4）+ 本交接文档 + 收敛文档的 D1–D6 追加：

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `nt_game/env.rs` | 补 `GameOutcome`/`outcome()`/`turn_budget()`，phi 改 `Option` | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `nt_game/evolution.rs` | 删 `Auto*`，接线，3 条回归测试 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `nt_game/hex_crucible.rs` | D4 下溢修复 + 3 方法实现 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `nt_game/builtin/hex_tictactoe.rs` | `outcome()`/`phi` 实现 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `nt_game/builtin/game_2048.rs` | `outcome()`/`phi` 实现 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `nt_mind_background_loop/handlers_game.rs` | 接线 + 6 条测试 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `sessions/handoff-2026-10-06-ntgame-phase1-wiring.md` | 本文件 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |
| `docs/architecture/NT-GAME-CONVERGENCE-2026-10-06.md` | 追加 D1–D6 裁决 | ☑ `git commit --only` 已提交（`3fdd1a0f`） |

> 他窗改动（`nt_judge.rs`、两个 `.neotrix/*.json`、未跟踪的 patch/db）**不在上表**，本会话未碰，保持其原状。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**0**
- 提交前是否跑过 `cargo check -p neotrix --lib` / `cargo test -p neotrix --lib`：☑ **是**
  - `check --lib` CLEAR；`--lib game` 458/0；全量 13399 passed / 2 failed（他窗）/ 38 ignored
- `nt_lock_audit.py`：**0 处，RC=0**
- 🔴 `check-feature-gates.sh`：**改动前 PASS；改动后重跑被中止 ⇒ 当前状态未验证**（见 §6）

## 9. 经验（供后续窗口，勿只读最新）

1. **「删冗余」是最容易把 bug 删进正确代码的一类裁决。** 本会话原计划删 530 行 `Auto*`，但 `AutoGame` 比 `NtGameEnv` 多两个方法（`reward(player)`/`board_hexagrams()`）—— 那不是随手重写，是**给缺失合约打的补丁**。判据：`trait B` 比 `trait A` 多方法 ⇒ B 的存在有原因，先查原因再删。
2. **接线会立刻暴露被死代码掩盖的 bug。** D4（`hex_crucible.rs:457` 下溢 panic）此前从未触发，正因为活路径走的是副本。⇒ 「死代码里没有 bug」是选择偏差，不是安全证明。
3. **`0.0` 默认值会把「未定义」洗成「真值 0」。** `phi_contribution() -> f64` 默认 `0.0` 让两个未实现的游戏看起来像 phi 真的是 0。⇒ 缺失信号应返回 `Option`，让缺口可表达。
4. **一次性探针 > 纯推演，但必须换成永久回归。** 四个缺陷全部由探针打出（`win_rate≡0.000`、`phi≡0.0000`、panic、`turns≡50.0`）；随后把探针换成 3 条**不变量**测试。注意断言要分层：`HexCrucible::compute_phi()` 按设计不归一（实测 1.08），强加 `[0,1]` 断言会逼后人篡改游戏语义 —— 这条误判是被我自己的测试当场抓出来的。
5. **两处重复的「上限」是隐蔽 bug 温床。** D6 里游戏预算（60/80）与外层安全帽（50）都是「回合上限」的**双重权威**，静默截断高星位。⇒ 引入外层上限前先问「游戏自己管不管」。
