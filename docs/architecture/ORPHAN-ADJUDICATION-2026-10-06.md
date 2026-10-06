# 孤儿目录逐条裁决 —— 2026-10-06

> 触发：`FULL-AUDIT-2026-10-06.md` §2「未接线：63 个 `.rs` 不在编译树上」。
> 判据权威：`python3 scripts/ops/nt_orphan_dir.py`（**不另造扫描器**）。
> 棘轮：`scripts/orphan-dir-baseline.txt`。

## 0. ⚠️ 结论与预期相反：**这不是垃圾，是 16,264 行带测试的活代码挂在编译树外**

| 度量 | 值 |
|---|---:|
| 孤儿目录（门报） | 15（**其中 1 个是 `#[path]` 假阳性** ⇒ 真实 **14**） |
| `.rs` 文件（门报） | 63（**真实 57**） |
| 代码行 | **16,264** |
| **内含测试** | **204 个 `#[test]`** |
| 文档引用（`docs/`+`sessions/`） | 最高 120 处（`nt_core/memory`） |

⇒ 我原报告里「若其中有活路径，则『绿』是**假绿**」的担心**已被证实**：
**11 个目录含测试**（有人维护过），`knowledge` 与 `memory` 分别有 60 / 120 处
文档引用。**这些代码的测试从未被执行过** —— 因为它们不在编译树上。

## 1. 基线陈旧：25 条里 **10 条已失效**（已收缩）

原基线 25 条，当前真孤儿只有 15 个 ⇒ **10 条的裁决前提已不成立**
（那些目录已被挂载进编译树，基线没收缩）：

| 已挂载（不再孤儿） | 层 |
|---|---|
| `nt_world/crawl/dom_extractor` | L2 |
| `nt_world/crawl/ordered_backend_router` | L2 |
| `nt_shield/guard/agent_guardrails` | L3 |
| `nt_core_gwt/cost_ladder` · `decision_layer` · `evidence_gating` | L5 |
| `nt_mind/dual_track` · `jit_harness` | L5 |
| `nt_meta_cleanup` · `dream_replay` | L6 |

⇒ **基线已棘轮收缩 25 → 15**，`check-orphan-dirs.sh --strict` 仍 RC=0。

⚠️ 这本身暴露一个门缺陷：**该门只报「新增」，从不报「基线陈旧」** ——
所以 10 条失效项可以无限期留在基线里而无人察觉。
（`check-silent-failure.sh` 有「baseline stale」提示，本门**没有** ⇒ 建议对齐。）

## 2. 逐条定性（15 条，按证据分三类）

判据：**含测试 + 有文档引用** ⇒ 活代码；**极小且无测试** ⇒ 疑为桩/占位。

| # | 目录 | 文件 | 行 | 测试 | 文档 | 定性 |
|---|---|---:|---:|---:|---:|---|
| 1 | `l1_action/nt_act/agent_loop` | 9 | 2,096 | **31** | 20 | ⚠️ **最大嫌疑**。AGENTS.md §6.2 把它记为「死引擎，已把有效部分迁入 neobot」，但**这 9 个文件仍在原地且带 31 个测试** ⇒ 裁决前提需重验：是残留副本，还是迁移后忘了删？ |
| 2 | `l5_cognition/nt_mind/cross_domain` | 5 | **2,649** | 14 | 7 | ⚠️ **最大体量**，带测试 ⇒ 活代码未接线 |
| 4 | `nt_mind/self_improvement` | 4 | 2,129 | **44** | 10 | ⚠️ 测试密度最高（44/2129 行）⇒ 明显被维护过 |
| 5 | `l2_perception/nt_world/temporal_kg` | 8 | 1,278 | **42** | 4 | ⚠️ 测试密度最高之一 |
| 6 | `l5_cognition/nt_core/knowledge` | 5 | 1,263 | 0 | **60** | ⚠️ 无测试但 60 处文档引用 ⇒ 被文档当作既有能力引用，**却不可用** |
| 7 | `l5_cognition/nt_core/memory` | 5 | 1,171 | 4 | **120** | ⚠️ 同上，文档引用最多 |
| 8 | `l5_cognition/nt_core/nt_consciousness_core/archive` | 3 | 1,082 | 19 | 50 | 名为 `archive` ⇒ 可能是**有意归档**，需确认是否保留 |
| 9 | `l1_action/nt_act/semantic_routing` | 4 | 844 | 22 | 2 | 活代码 |
| 10 | `l1_action/nt_act/geo_seo` | 4 | 591 | 9 | 4 | 活代码 |
| 11 | `l1_action/nt_act/nt_act_dev_tools` | 4 | 582 | 4 | 3 | 活代码 |
| 12 | `l4_emotion/nt_feel/cognition_bridge` | 2 | 234 | 9 | 1 | 活代码 |
| 13 | `l5_cognition/nt_core/io_skills` | 2 | 87 | 3 | 3 | 活代码（小） |
| 14 | `l5_cognition/nt_consciousness` | 1 | 39 | 0 | 18 | · 极小，**疑为兼容桩**（18 处文档引用却只有 39 行） |
| 15 | `l4_emotion/nt_memory/nt_memory_knowledge_graph` | 1 | **8** | 0 | 3 | · **8 行** ⇒ 几乎确定是占位/转发桩 |

## 3. ⛔ 三条不能由我单方裁决的发现

| # | 发现 | 为什么卡住 |
|---|---|---|
| **O-1** | **`agent_loop` 9 文件 / 2,096 行 / 31 测试**与 AGENTS.md「已迁入 neobot」的记载**冲突** | 迁移可能只完成一半（neobot 侧已接线、这侧忘删）。⇒ 需比对两侧是否重复实现，再决定「删」还是「接」。**这正是排期 A3 那类「前提已变」的情况。** |
| **O-2** | **`knowledge` / `memory` 被文档引用 60 / 120 次，却不可执行** | 文档把它们当既有能力引用 ⇒ 要么接线，要么改文档。**改文档 = 承认能力不存在**，属产品决策。 |
| **O-3** | **`handlers_consciousness` 不在编译树上 ⇒ 我本日对它的 D1 修复未生效** | 我在 `4247b260` 修了它 2 处静默失败，但那些代码**根本不参与编译** ⇒ 修复是「正确的死代码修改」。⇒ 须与O-1 一并裁决：接线 or 迁移或删除。 |

## 3.1 ⛔ O-3 的完整撤销记录（★ 我的判断错误全过程）

1. 我判定「`handlers_consciousness` 未挂载 ⇒ 漏了一行 `mod`」；
2. 我新建了 `handlers_consciousness/mod.rs` 并在父 `mod.rs` 加 `mod handlers_consciousness;`；
3. 编译器立刻报 `E0761: file for module 'handlers_consciousness' found at **both**
   handlers_consciousness.rs and handlers_consciousness/mod.rs`；
4. 复查发现**早就存在** `handlers_consciousness.rs`（12 行），内容正是
   `#[path = "handlers_consciousness/nt_audit.rs"] pub mod nt_audit;` ×6
   ⇒ **代码从来都在编译树里**；
5. 已完全回退（删我建的 `mod.rs` + `git checkout` 还原父 `mod.rs`），
   `cargo check -p neotrix` 恢复 **RC=0**。

⇒ **裁决层面的修正**：15 个「孤儿」中 **1 个是假阳性**，
真实孤儿为 **14 个 / 57 文件**（不是 15 / 63）。

⚠️ **门缺陷确认**：`nt_orphan_dir.py` **不解析 `#[path]` 属性**
（AGENTS.md §4.2 已把它列为该门已知假阳性来源，但门本身至今**没有任何提示**）
⇒ 假阳性会长期混在基线里，让「未接线」的判断失真。
⇒ 已把该假阳性从基线剔除，并建议给门加「`#[path]` 挂载」识别。

⚠️ **方法论教训**：判「某目录是否编译」**不能只看父 `mod.rs` 的 `mod` 声明** ——
必须同时查① 同名 `.rs` 文件 ② 全仓 `#[path = "…/该目录/…"` 引用。
我漏了第②步，代价是「差点把已编译的活代码当成漏接线」。

## 4. 建议的处置顺序（按「解除假绿收益 ÷ 风险」）

1. **先裁决 `agent_loop`（O-1）** —— 它是唯一一个「文档已判死、代码却带 31 个测试在原地」的矛盾点，其它 14 条的处置都依赖这个前提。
2. **再裁决 `handlers_consciousness`（O-3）** —— 同上，且牵连我本日的 D1 修复是否有效。
3. **`knowledge` / `memory`（O-2）** —— 接线 vs 改文档，属产品决策。
4. **剩下 11 条活代码** —— 逐条「接入 or 删除」，**不接受继续挂基线**：
   基线的语义是「已裁决」，不是「已放弃」。
5. **给 `nt_orphan_dir.py` 加「基线陈旧」提示**（§1）—— 纯门改进，无裁决成本。

## 5. 本轮已落地的部分（无裁决成本的那部分）

- [x] **基线棘轮收缩 25 → 15**（10 条失效项清出），`check-orphan-dirs --strict` 仍 RC=0
- [x] 逐条定性表（§2）含文件数/行数/测试数/文档引用数四个**实测**维度
- [x] 标出 3 条**不可自裁**的发现（§3）

⚠️ **未落地**：15 条的实际处置（接入/删除）。全部依赖 §3 的三个裁决前提。