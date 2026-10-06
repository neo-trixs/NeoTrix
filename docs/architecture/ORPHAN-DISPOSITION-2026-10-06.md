# 孤儿目录处置表 —— 2026-10-06（O-1 ~ O-13）

> 判据权威：`python3 scripts/ops/nt_orphan_dir.py`（已修 `#[path]` 假阳性）。
> 能力差集：对每个孤儿，扫其 `pub struct/enum/trait`，逐个在**真实编译树**
> （排除全部孤儿目录）里找同名实现 ⇒ 得到「独有类型数」。
> 格式：M-14「限制四段式」—— 保留项 / 保留理由 / 不接线理由 / 复活触发。

## 0. 总账

| | 值 |
|---|---:|
| 孤儿目录 | **3**（O-1 归档不接线；O-7 有意归档永不接线；余 1 条待接线） |
| `.rs` 文件 | **15** |
| 代码行 | **5,744** |
| **判为「纯重复」** | **0 条** |
| 判为「独有能力」 | 2 条 |
| 判为「部分覆盖」 | 11 条 |
| 已删除 / 已接线 / 永不接线 | **1 / 10 / 1** 条 |

⚠️ **最重要的一条结论**：**13 条里没有一条是重复副本。**
⇒ 「未接线」在本仓不是「冗余」，而是「**有能力、无接线**」——
与 O-1（`agent_loop`）同一类。

## 1. O-1 `l1_action/nt_act/agent_loop`（9 文件 / 2,096 行 / 31 测试）

**保留项**：plan→execute→verify→adapt 闭环 + `Auditor` + task manager
**保留理由**：编译树内实测**无等价物**（`AgentAuditor` / `AgentPlan` /
`CrossVerification` 均 0 处；`nt_io_agent_loop` 只有 `execute`）；带 31 个测试
**不接线理由**：① 排期 §2 记录 mu 对**同类决策点**实测「2,412 块 drop **0** 个 /
吃掉 **54%** 判断输入」⇒ 无本仓数据前接线 = 复制已知高成本零收益模式；
② 该目录从未编译验证过
**复活触发**：出现本仓自有的「验证层有净收益」实测数据
详见 `O1-AGENT-LOOP-VERDICT-2026-10-06.md`

## 2. 本轮已删除：`l5_cognition/nt_consciousness`（1 文件 / 39 行）

**它是什么**：一个自称「统一意识模块 / 向后兼容 facade」的 `mod.rs`，内容全是
`pub use crate::l5_cognition::nt_consciousness::features as features;` 这类
**自引用** re-export。

**删除依据（实测，非推断）**：临时挂载后 `cargo check` 报 **3 处 `E0432`**：

```
error[E0432]: unresolved import `crate::l5_cognition::nt_consciousness::features`
  no `features` in `l5_cognition::nt_consciousness`
```

⇒ **它永远无法编译**（自引用的子路径不存在）。

**为什么必须删而不只是不接线**：它的文档写着
「**消费方应使用 `crate::l5_cognition::nt_consciousness::*`**」
⇒ 一条**会把消费者导向编译不过的路径**的死代码，比没有代码更有害。
删除后 `cargo check -p neotrix` ⇒ **RC=0**，孤儿 14 → **13**。

## 3. 其余 12 条处置表

按「体量 × 独有能力数」排序（独有能力多的排前 = 归档价值高）。

| # | 目录 | 文件/行/测试 | 独有类型 | 处置 | 理由摘要 |
|---|---|---|---:|---|---|
| O-2 | `nt_mind/cross_domain` | 5 / 2,649 / 14 | **46**/54 | **归档（不接线）** | 独有能力占比最高（85%），带测试 ⇒ 有资产价值；从无数据 |
| O-3 | `nt_mind/self_improvement` | 4 / 2,129 / **53** | 19/27 | ✅ **已接线（A54）** | 53 测试全绿。测出**单位错配真 bug**：`updated_at` 是 Unix 秒却被当回合数用（`% (current_turn+1)`，源码自称 pseudo-age）⇒ age 依赖挂钟取模，**测试可绿可红**；已引入回合域 `last_access_turn` 解耦。另 `accuracy()` 分子只含 positive ⇒ 结构上永远 ≥1.0、无法反映失败，已增 `times_reported` 作分母 |
| O-4 | `nt_world/temporal_kg` | 8 / 1,278 / **42** | 3/8 | ✅ **已接线（A53）** | 42 测试全绿。测出**真数学 bug**：PPR 把悬挂质量 `dangling_sum/n` 均摊 ⇒ 链尾 `e4` 反超种子 `e0`；已改为按 personalization 再分配（标准公式）。另 7 处 `NaiveDateTime::from_ymd_opt`（该 API 不存在）改现代 chrono |
| O-5 | `nt_core/knowledge` | 5 / 1,263 / **0→5** | 29/42 | ✅ **已接线（A51）** | 首次接线即编译通过；补 5 个冒烟测试并**测出`bfs`/`dfs` 的 `max_depth` off-by-one**（`Some(1)` 不返回直接邻居，却给它记了`distances=1.0`）⇒ 真 bug，已修并被测试锁定 |
| O-6 | `nt_core/memory` | 5 / 1,171 / 4 | 16/24 | ✅ **已接线（A51）** | 4 测试全绿。首接线暴露 **4 类真 bug**：`EmotionLabel` 只在 L6（引它即 **L5→L6 违规**）、`ConceptNode` 无 `concept` 字段 6 处、`episode.id` move 后复用 1 处、`patterns`/`principles` 借用活过 `store()` 2 处 |
| O-7 | `nt_consciousness_core/archive` | 3 / 1,082 / 19 | 13/14 | ⛔ **永不接线** | 其 `README.md` 自述：**「旧代码归档 / 归档日期 2026-09-18 / 按 FUSION-ARCHITECTURE-v4 重构为5层架构，旧代码已备份到此目录」**，且「新架构文件」列在**父目录**。⇒ 接线它等于**复活被刻意归档的死代码** |
| O-8 | `nt_act/semantic_routing` | 4 / 844 / 22 | 12/13 | ✅ **已接线（A53）** | 22 → 0 error，含**真逻辑 bug**：`context_boost` 拿上下文 **key** 比关键词（永匹配不上）⇒ 改为比 value。另 `SimilarityScore` 持 `String` 却derive `Copy`（E0204）、`BehaviorPatternType` 缺 `Eq/Hash` 却作 HashMap 键、闭包内`?` 需改 `and_then`、借用冲突 2 处 |
| O-9 | `nt_act/geo_seo` | 4 / 591 / 9 | 13/18 | ✅ **已接线（A53）** | **零错误零改动**直接编译通过，9 测试全绿 |
| O-10 | `nt_act/nt_act_dev_tools` | 4 / 582 / 4 | **5/5** | ✅ **已接线（A54）** | 全未覆盖的 `DesktopBuilder`/`BuildLadder`/`GitHook`/`DaemonMonitor`，仅 4 处导入错误，4 测试全绿 |
| O-11 | `nt_memory_knowledge_graph` | 3 / 569 / 8 | 9/10 | ✅ **已接线（A52）** | 首接线 **22 → 0 error**。测出：孤立残留 derive 块（同时作用于同一 enum ⇒ `rename_all` 重复＋6 个 `E0119`）、`use` 夹在 `#[derive]` 与 struct 之间、derive 缺 `Hash` 却用 `HashSet`、**真UTF-8 panic**（按字节切3 字节的 `→`）。另发现 `sync_to_kb` 是**活路径上的死端桩**（真实写入被注释，有调用方） |
| O-12 | `nt_feel/cognition_bridge` | 2 / 234 / 9 | 1/3 | ✅ **已接线（A52）** | 缺 `mod.rs`（只有 `config.rs`/`emotion_state.rs`）⇒ 补建后直接编译通过，**9 测试零改动全绿** |
| O-13 | `nt_core/io_skills` | 2 / 87 / 3 | **3/3** | ✅ **已接线（A52）** | 全未覆盖的 `Eli5Engine` 解释器，直接编译通过，3 测试全绿 |

## 4. 为什么「归档」而不是「删除」或「接线」

| 选项 | 判断 |
|---|---|
| **接线** | ⛔ 12 条**全部从未编译验证**（接一个 2,649 行、54 个类型的目录 ⇒ 可能几十个错误）。且 O-1 已提供反面实证：同类决策点接入 = 已知高成本零收益。 |
| **删除** | ⛔ 12 条**全部带测试**（O-5 除外，共 220+ 个测试）⇒ 删除会销毁「这层曾被设计并验证过」的资产。 |
| **归档**（推荐） | ✅ 保住测试与实现，同时**把「未接线」这个真状态显式化**。基线里逐条写明四段式 ⇒ 下个窗口接手时知道「这是裁决过的，不是遗忘的」。 |

⚠️ **归档不等于忘记**：基线里每条都带**复活触发条件**，所以「暂不接线」是**可逆决策**。

## 5. 本轮的方法论教训（三次误判，都记在这里）

| # | 误判 | 错因 |
|---|---|---|
| 1 | 「`handlers_consciousness` 漏了一行 `mod`」 | 只看父 `mod.rs` 的 `mod` 声明，**没查同名 `.rs` 用 `#[path]` 挂载** ⇒ 差点把已编译的活代码当漏接线 |
| 2 | 修孤儿门时两次引入更严重假阳性（15→26→49） | **修假阳性不能用更宽的匹配**；且 `if "mod.rs" not in filenames: continue` 是**承重**的（`crates/*/src` 无 `mod.rs`） |
| 3 | 「`nt_memory_knowledge_graph` 是 8 行桩」 | 统计用了 `ls $d/*.rs`（只看到顶层）⇒ 漏了子目录里的文件。**正确口径：`find $d -name '*.rs'`** ⇒ 实为 569 行 |
| 4 | 「`AgentLoop` 同名 ⇒ 重复，可删」 | 同名 ≠ 同一实现。实测能力差集后**推翻** |

⇒ 通则：**凡「判定某段代码重复/死/未接线」，必须先做「全文件系统 + 真实编译树」双向核对**，
     且统计口径要用 `find -name`，不能用 `ls`。