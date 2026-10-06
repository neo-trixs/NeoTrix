# 孤儿目录处置表 —— 2026-10-06（O-1 ~ O-13）

> 判据权威：`python3 scripts/ops/nt_orphan_dir.py`（已修 `#[path]` 假阳性）。
> 能力差集：对每个孤儿，扫其 `pub struct/enum/trait`，逐个在**真实编译树**
> （排除全部孤儿目录）里找同名实现 ⇒ 得到「独有类型数」。
> 格式：M-14「限制四段式」—— 保留项 / 保留理由 / 不接线理由 / 复活触发。

## 0. 总账

| | 值 |
|---|---:|
| 孤儿目录 | **11**（O-1 单列裁决；O-5/O-6 已于A51 接线） |
| `.rs` 文件 | **42** |
| 代码行 | **9,586** |
| **判为「纯重复」** | **0 条** |
| 判为「独有能力」 | 2 条 |
| 判为「部分覆盖」 | 11 条 |
| 已删除 / 已接线 | **1 条**（`nt_consciousness`，实测无法编译）/ **2 条**（A51） |

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
| O-3 | `nt_mind/self_improvement` | 4 / 2,129 / **44** | 19/27 | **归档** | 测试密度最高（44 / 2,129 行）⇒ 明显被维护过 |
| O-4 | `nt_world/temporal_kg` | 8 / 1,278 / **42** | 3/8 | **归档** | 覆盖率最高（5/8）⇒ 若接线，**先迁 3 个独有类型**即可，其余可弃 |
| O-5 | `nt_core/knowledge` | 5 / 1,263 / **0→5** | 29/42 | ✅ **已接线（A51）** | 首次接线即编译通过；补 5 个冒烟测试并**测出`bfs`/`dfs` 的 `max_depth` off-by-one**（`Some(1)` 不返回直接邻居，却给它记了`distances=1.0`）⇒ 真 bug，已修并被测试锁定 |
| O-6 | `nt_core/memory` | 5 / 1,171 / 4 | 16/24 | ✅ **已接线（A51）** | 4 测试全绿。首接线暴露 **4 类真 bug**：`EmotionLabel` 只在 L6（引它即 **L5→L6 违规**）、`ConceptNode` 无 `concept` 字段 6 处、`episode.id` move 后复用 1 处、`patterns`/`principles` 借用活过 `store()` 2 处 |
| O-7 | `nt_consciousness_core/archive` | 3 / 1,082 / 19 | 13/14 | **归档（名已自述）** | 目录名即 `archive` ⇒ 可能是有意归档；需确认保留策略 |
| O-8 | `nt_act/semantic_routing` | 4 / 844 / 22 | 12/13 | **归档** | 覆盖率 1/13 ⇒ 近乎全新能力 |
| O-9 | `nt_act/geo_seo` | 4 / 591 / 9 | 13/18 | **归档** | 独有 `KeywordRanking`/`GeoRegion` 等 SEO 域能力 |
| O-10 | `nt_act/nt_act_dev_tools` | 4 / 582 / 4 | **5/5** | **归档** | **全未覆盖** ⇒ 独有：`DesktopBuilder`/`BuildLadder`/`GitHook`/`DaemonMonitor` |
| O-11 | `nt_memory_knowledge_graph` | 3 / 569 / 8 | 9/10 | **归档** | ⚠️ 曾被我误判为「8 行桩」⇒ 实为 **569 行**（`mod.rs` + 2 个子模块），教训：统计须用 `find -name '*.rs'` 而非 `ls *.rs` |
| O-12 | `nt_feel/cognition_bridge` | 2 / 234 / 9 | 1/3 | **归档** | 仅 1 个独有类型（`CouplingConfig`）⇒ 若接线成本极低，可优先评估 |
| O-13 | `nt_core/io_skills` | 2 / 87 / 3 | **3/3** | **归档** | **全未覆盖**：独有 `Eli5Engine`/`Eli5Explainer`/`Eli5SelfTest` |

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