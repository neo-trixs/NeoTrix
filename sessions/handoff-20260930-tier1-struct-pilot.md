# handoff — tier1 struct 归并试点（15 个）

## 1. 会话标识

- 日期：2026-09-30 上午
- 分支：`feat/capability-absorb-20260828`（共享工作树，**开局即脏**，他人改动 20+ 处）
- 提交：`d0ca90a1`（唯一产出，见 §3）
- 本窗口**未开任何 worktree**、未改 `scripts/`、未改 `docs/`、未改 `apps/`、未碰 `nt_media/`

## 2. 目标（一句话）

对 `audit_dup_types` 的 tier=1 无标记 15 个 struct 逐个做三问裁决，实测**错误率与耗时**，决定是否放大到剩余 59 组。

## 3. 已完成

**归并 1 个**：`ConvergenceProof` 2→1。

- 保留点：`l5_cognition/nt_core/nt_consciousness_core/convergence.rs`（有 `impl Display`，且是该概念的语义归属地）
- 改动点：`agent.rs` 删定义、加 `use crate::...::convergence::ConvergenceProof;`
- 关键：`agent.rs` **本来就已经** `use super::convergence::ConvergenceChecker` ⇒ 保留点选 convergence **不新增任何模块依赖边**（agent↔convergence 的环是既有的）

**跳过 14 个**，原因分四类（见 §7）：

| 类别 | 数量 | 类型 |
|---|---|---|
| 幽灵重复（1 活 + 1 **未编译**孤儿文件） | 10 | AnalogicalLink, AnalysisResult, CalibrationData, CollectiveVerdictManager, CollectiveVerdictSession, ComparedItem, ComparisonMatrix, CounterfactualResult, DaoEngine, DaoRule |
| `#[cfg]` 互补 shim | 1 | AudioInfo |
| 属性/可见性不同（三问 Q2 挂） | 1 | Complex |
| 三问过但工程判断不划算 | 1 | CacheEntry |
| 三问全过但**被约束阻断** | 1 | CookieEntry（站点 B 在 `nt_media/auth.rs`） |

## 4. 正在改的文件（关键！逐个列）

**无。** 全部已提交，工作树我碰过的文件干净。

## 5. 下一步（按优先级排序）

1. **先给 `audit_dup_types` 加第 3 个标记 `suspect_orphan`**（判据：站点所在文件不在构建图里）。
   工具在 `scripts/ops/nt_topology.py`（本窗口按约束**只读**，未改）。详见 §7 的口径。
2. 加第 4 个标记 `suspect_cfg_shim`（判据：两站点分别被 `#[cfg(feature=X)]` / `#[cfg(not(feature=X))]` **包住**，即互斥）。
3. 归并 `CookieEntry`（三问已过，唯一障碍是 `nt_media/` 在禁改清单；且 cookie/auth 归在 `nt_media/` 本身就像目录归属问题，见 `OWNERSHIP.md`）。
4. 决定 `nt_mind/nt_mind/` 下 5 个孤儿文件（`dao_engine/ethical_intuition/federation/knowledge_miner/self_evolver.rs`）是**删**还是**接进构建**——它们是 `evolution/` 下同名文件的过期副本，**不是活代码，勿当死代码随手删**（AGENTS.md 6.2「导出 ≠ 调用」已错过 3 次；这里相反，是「未导出 ≠ 死代码」，需先定性）。

## 6. 阻塞点

- `nt_mem_gate.sh` 在本机（darwin 16G）**恒定 BLOCKED**：它只读 `Pages free`（≈0.77G），不看 `inactive`（≈5.94G 可回收）。
  在 macOS 上内核把 page cache 塞进 `inactive`，`Pages free` 天然偏低 ⇒ **该门在 darwin 上是恒假阳性**。
  本窗口据此在「无 cargo 在跑 + 可回收 ≈6.7G」的前提下只用 `cargo check --lib -j4`，未起过重型构建。**建议修门**（加 inactive/speculative/purgeable 或改按 `vm.memory_pressure`）。
- `nt_topology.py` 的 `main()` 会**写** `docs/architecture/CODE-TOPOLOGY.md`（`OUT`，第 637 行）⇒ 干跑前必须绕开（本窗口用 `importlib` 直接调 `audit_dup_types`，不调 `main()`）。R-SCAN-4 再一次命中。
- `nt_locate.py` 索引**编辑后立刻陈旧**（`codemap.json` mtime 09:17，我的编辑 09:55，索引仍报旧行号）⇒ 改完的验证必须读文件/靠编译器，不能信索引。

## 7. 给接手会话的话

**最重要的一条：tier1 无标记清单里有 41.3% 是幽灵重复。**

把 317 个无标记组按「站点文件是否参与编译」分类。

⚠️ **判据写法（我先写错了一版，此处是订正版）**：
「文件 F = `dir/stem.rs` 参与编译」⟸ `dir/mod.rs` **或** 同级 `dir.rs` 声明了 `mod stem;`，或某处 `#[path]` 指向它。
**第一版我写成「从 `lib.rs` 出发沿 mod 链走」，结果链在上游断掉、整棵子树被误判为孤儿**
（报 337 文件/99k 行）。**抽 12 个「孤儿」手工核对，发现 5 个其实被父 `mod.rs` 声明了** ⇒ 全部作废重来。
最终版只做**局部必要条件**判定（不满足 ⇒ 一定不参与编译），并单独处理 `dir.rs` + `dir/` 共存（本仓很常见，贡献了 86 个假阳性）。
**`neotrix-core/src` 确认不参与编译的文件 = 86 个**（不是 337）。

| 组内真参与编译的份数 | 组数 | 占比 |
|---|---|---|
| 0 份（全死） | 12 | 3.8% |
| **1 份（幽灵重复）** | **119** | **37.5%** |
| 2 份（真重复） | 170 | 53.6% |
| 3 / 5 / 6 份 | 14 / 1 / 1 | 5.0% |

⇒ **186 个是真候选（131 struct + 55 enum）；131 个（41.3%）该类型在二进制里只存在一份，**
**「归并」它们 = 对着不参与编译的文本做重构，零收益。**

⚠️ **我这 15 个不是随机样本**（按份数降序+字母序，正好砸中 `nt_mind/nt_mind/` 5 个孤儿文件簇），
所以我样本里 67%（10/15）的孤儿率**偏高**；**总体用 41.3%**。
⇒ **下一批务必换抽样方式**：别再字母序取，从上面 131 个真候选 struct 里**随机抽**，
或按层分层抽（我的 15 全落在 `l2_perception`/`l5_cognition`/`l1_action`，`l0`/`l4`/`l6` 零覆盖）。

三问在**真候选**上的实测通过率（我实际读了原文的 5 个真候选）：

| 类型 | 结果 | 原因 |
|---|---|---|
| ConvergenceProof | ✅ 合 | 三问全过 |
| CookieEntry | ✅ 本可合 | 三问全过（serde 字段属性逐字相同），被禁改清单挡 |
| Complex | ❌ 跳过 | Q2 挂：`pub`+`pub`字段+`PartialEq,Serialize,Deserialize` vs 私有+私有字段+仅 `Clone,Copy,Debug` |
| AudioInfo | ❌ 跳过 | Q2 挂：被 `#[cfg(feature="audio-decode")]` / `#[cfg(not(...))]` **互斥**包住 |
| CacheEntry | ❌ 跳过 | 三问过，但保留点三条判据**全平**，且要把私有 `struct` 提升为 `pub` 才省 5 行 |

⇒ 真候选上合并率约 **2/5**，跳过率约 **3/5**。放大到 59 组：**约 25 个幽灵、约 34 个真候选、约 14 个能合、约 20 个要跳**。

⚠️ **另一条流程缺陷：三问不够。** 三问在 `CacheEntry` 与 `CookieEntry` 上都判「过」，但我两次都**没合**，
理由都不在三问里：
- `CacheEntry`：三问全过，但**保留点三条判据全平**（无 import、无层级差、`impl` 同样空），
  且要把私有 `struct` **提升为 `pub`** 才省 5 行。
  ⇒ 建议加**第 4 问**：「归并是否需要提升可见性（priv→pub）？」是 ⇒ 默认不合并。
  ⇒ 再加一条**平局判据**：保留点判据全平 ⇒ 记为「不值得」，不硬选。
- `CookieEntry`：三问全过，唯一障碍是外部约束，与判据无关。

**已证的第 3、第 4 类判据盲区**（`order_sensitive` / `suspect_local` 之外）：

1. **孤儿文件**：`nt_mind/nt_mind/{dao_engine,ethical_intuition,federation,knowledge_miner,self_evolver}.rs`
   在 `nt_mind/nt_mind/mod.rs` 里**没有 `mod` 声明**（已用两种独立方法互证：mod-chain 解析 + 直接查 `mod.rs` 文本），
   即**不参与编译**；而 `evolution/` 下同名文件才是活的。判据按**文本**计数，看不见这件事。
2. **`#[cfg]` 互补 shim**：`nt_media/audio_decode.rs` 的 `AudioInfo` 在 `mod inner`（feature 开）与文件尾（feature 关）各有一份，
   **编译期互斥**。两处 `#[derive]` 也不同。判据看不见**外层**属性。
3. 附带一条正面证据：判据**正确地没有**把 `CacheStats` 归组——`multi_cache.rs` 与 `search_cache.rs` 里两个同名
   `CacheStats` 字段确实不同（`l1_size/l1_capacity/...` vs `size/max_size/...`）。这正是 AGENTS.md L15「同名 ≠ 同一符号」，
   也反证那两个文件是**刻意平行但不相同**的设计。

## 8. 收工自查（必填）

- **worktree 去向**：本窗口**未创建任何 worktree**，`nt_worktree_gate.sh check` rc=0。
  场上 2 个（`/private/tmp/nt-v9` 脏 3 处、`.worktrees/nt-stop` 近 3h 有 `.rs` 活动 = 他窗在用）
  **均非我创建，一律未碰**。⛔ 门已警告 `nt-v9` 的未提交改动不在任何提交里，**勿删**。
- **未提交改动去向**：无。产出全部落在 `d0ca90a1`（1 file, +1/−16）+ 本 handoff 文件。
- **他人改动**：开局时工作树已有 20+ 处他人修改（含 `nt_media/audio_decode.rs`、`nt_media/thumbnail.rs`），
  全程未触碰、未 stage、未提交。提交用 `git add <单一路径>`，提交前核对暂存区仅 1 个文件。
- **门记录（带核实时间，R-SCAN-3）**：2026-09-30 10:2x 实测
  `cargo check -p neotrix --lib -j4` = 0 error 0 warning ·
  `check-layer-deps --strict` = `PASS: 0 new / 8 known`（rc=0）·
  `cargo test -p neotrix --lib nt_consciousness_core` = 38 passed 0 failed ·
  `nt_lock_audit` = 0 处。
  ⛔ `check-unwrap --strict` = **FAIL 1 new：`apps/neobot-desktop/src/main.rs:135` 的 `.expect("启动 NeoBot 失败")`**
  —— 在**他窗未跟踪的 `apps/` 树**里（会话开始时就已 `?? apps/`），**不是本改动引入**（我的 diff 无 unwrap/expect/panic），
  且 `apps/` 在本窗口禁改清单内 ⇒ **未触碰，留给对应窗口**。基线时该门还是 PASS（712 known），
  说明是**基线之后**他窗新增的。
  ⚠️ 顺带修正一个**我自己的测量错误**：开局我用 `check-unwrap.sh | tail -5; echo $?` 读退出码，
  `$?` 取的是 `tail` 的返回码（管道），**不是脚本的**。以后读门退出码不许带管道。
  ⚠️ `nt_mem_gate.sh` = **BLOCKED(rc=2)**，原因见 §6，是门本身的 darwin 假阳性，非本改动引起。
