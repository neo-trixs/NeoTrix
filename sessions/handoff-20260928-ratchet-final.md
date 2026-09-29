# Handoff — 2026-09-28 分层门棘轮收口（101 → 8）

> 分支 `fix/bitemporal-and-layer-ratchet`，17 commits，**未合入主干**。
> 接手前先读 `docs/architecture/LESSONS-2026-09-28-ledger-rot-and-consumer-audit.md`
> （L1–L19，本轮 15 条经验）与 `DECISIONS-2026-09-28.md` §P0–P3。

## 1. 本轮产出（全部已落盘 + 验证绿）

| 项 | 结果 |
|---|---|
| 分层门棘轮 | **101 → 8**（清 93 条），`PASS 0 new`、RC=0、只向下 |
| L1–L5 | **真跨层引用全清**（含第二棵树 `neotrix/`） |
| 默认门 | `cargo check --tests` exit=0；`cargo test --lib --test-threads=4` → **12154 passed / 0 failed / 39 ignored** |
| ios-bridge 门 | `cargo check -p neotrix --features ios-bridge` → **exit=0**（顺带修掉一个预存 E0433） |
| B-1 真双时间 | `nodes` 零 schema 变更实现，两个 `#[ignore]` 已摘 |

## 2. ⛔ 接手必知的三条硬约束

1. **改动 `neotrix/` 树必须跑 `--features ios-bridge`**。该树整体受
   `#[cfg(feature="ios-bridge")]` 门控，默认 `check`/`test` **编不到它** ——
   `seal_pipeline.rs` 的 E0433 就是这么长期隐身的。
2. **本仓无 upstream**，分支全在本地。合入前按 L8：**必须复核门数字**，
   基线类文件的 `Auto-merging` 会静默降级。
3. **P0 门红先判红因归属**（L9）：`git ls-tree HEAD` / `git grep HEAD` 查 HEAD 侧。
   完好的话红因是他窗**未提交 WIP**，别去修，改用 `.worktrees/` 独立车道（L7-b）。

## 3. 剩余 8 条：全部已定性，**不要**再尝试改道

| 类别 | 条数 | 原因 |
|---|---|---|
| l0 无对应真实现 | 4 条 / 11 处 | VSA（l2 真实现）、metacalib（l5）、training_cycle（l5）、BranchKind（l5） |
| l6 本地真实现 | 1 处 | `register_absorbed_modules`（`nt_core_self_test_integration.rs:9`） |
| 字符串/参数字面量 | 3 条 | 剥离后 0 命中（L14） |

明细见 `DECISIONS-2026-09-28.md`「剩余 8 条」表。
**删基线会让 CI `FAIL: 8 new`** —— 只能留，不能删。

## 4. 待你拍板

| 项 | 状态 |
|---|---|
| **合入时机** | 17 commits 未合主干；主干已推进到 `c606a35e` |
| **B-2 Noise IK** | ⛔ 仍卡**官方测试向量**，我无解。**不接受手写"看起来能跑"的 crypto** |
| 门加字符串剥离过滤 | 独立待办。⚠️ 不能用 `-v '"[^"]*lX[^"]*"'`（会连带滤掉同行真引用，`l7_l1_bridge.rs:201-202` 即「字符串 + 真实路径」同行） |
| `nt_file_ability/capability.rs` 本地重复类型 | 它有一套**独立复制的** `Layer`/`Domain`/`CapabilityMeta`（与 l0 副本不共源），是潜在的类型分裂点，未动 |

## 5. 收工自查（§8）

- **worktree 去向**：`.worktrees/ratchet`（50M，target 0M，工作树**干净**），
  分支 `fix/bitemporal-and-layer-ratchet` @ `6956a5c7`。
  按门报告「近 3h 有 .rs 活动」，**保留不删**（有 17 个未合入 commit）。
  回收时**必须走 `prune`**，禁止手删。
- **未提交改动去向**：**无**。工作树 `git status --porcelain` 为空。
  唯一在途的 `seal_pipeline.rs` 已提交为 `6956a5c7`。
- **他窗 WIP**：主树 `nt_core_capability_tree::CapabilityRegistry` 断链 4+ 处
  属**他窗未提交重构**（L9 原则，全程未碰）。
- **经验已落 KB**：`neotrix-experience absorb`，cycle `2026-09-28-ratchet`，
  15 条分支 + 8 条 route，`route-verify` 0 ghost。
- **文档**：LESSONS（L1–L19）、DECISIONS（P0–P3）、AGENTS.md 门记录（8 known）
  均已同步，无陈旧数字残留。

## 6. 一句话

> 门从 101 降到 8 靠的是**逐批验证 + 核到最终定义处**，
> 靠的不是改注释、不是给门豁免打补丁、也不是并行构建。
> 剩下 8 条的正确动作是**别动**。
