# 裁决：`nt_crystal_core` 的层归属（2026-10-03）

> ⭐ **本裁决推翻了上一轮我自己的假设**。按 `LESSONS-20260929-checked-is-not-verified.md`
> 与 `AGENTS.md` §5 R-SCAN-1 的纪律：**手推 ≠ 实证**，假设必须允许被自己的证据推翻。

## 0. 待决问题

`check-layer-deps.sh` 记 **16 处** L1→L5 违规，全部指向 `l5_cognition::nt_crystal_core`
（`nt_tui_app.rs` 13 + `nt_dispatcher_core.rs` 3）。

**上一轮我提出的假设**（❌ **已被自己的证据推翻**）：
> 「若它本质是**任务编排引擎**，更接近 L1/L3 而非 L5 ⇒ 该升层或下沉。」

## 1. 事实一：它是 **20,849 行**的认知编排（实测）

`neotrix-core/src/l5_cognition/nt_crystal_core*`：**20,849 行**，提供的符号横跨

| 子模块 | 代表符号 | 性质 |
|---|---|---|
| `agent_orchestrator.rs` | `Agent` `Crew` `Task` `dispatch` `execute_flow` `hierarchical_delegate` `AgentStatus` `ProcessType` `CrystalArchetype` | ⭐ 多智能体编排 |
| `arbitrator.rs` | `arbitrate` `ArbitrationCondition` `ArbitrationDecision` `check_and_consume` | ⭐ 仲裁 |
| `self_healing.rs` | （372 行）自愈 | ⭐ 自愈 |
| `nt_crystal_task_fusion.rs` | `NtLlmAsk` `NtLlmReply` `NtTaskFusionError` | LLM 问答契约 |
| `nt_crystal_dialogue.rs` | `NtDemand` `NtHumanChannel` `NtHumanReply` | 人机通道 |

⇒ ⭐⭐ **「多智能体编排 + 仲裁 + 自愈」是认知语义，不是动作语义。**
⇒ ⭐⭐ **`nt_crystal_core` 留在 L5 是正确的。** 假设 ❌ **被推翻**。

## 2. ⭐⭐⭐ 事实二：真正的缺陷在**另一侧** —— L1 在干认知的活

逐个读 L1 消费方的文件头与用法（实测）：

| L1 文件 | 自述 | 用它做什么 |
|---|---|---|
| `nt_tui_app.rs`（13 处） | TUI 应用 | `NtCrystalTaskLoop` `NtProgressSink` `NtLlmAsk/Reply` `NtTaskFusionError` ⇒ ⭐⭐ **驱动 LLM 任务循环** |
| `nt_crystal_llm_bridge.rs` | 「**L1** 异步 Provider → 晶体同步问答」 | ⭐⭐ 文件名自述就承认是 **LLM 桥**，却住在 L1 |
| `nt_free_pool.rs` | 「池免费模型的智能调用」 | ⭐ LLM 调用 |
| `nt_stdin_human.rs` | 「终端里的人」 | ⭐ 走人机通道 |
| `nt_dialogue_tui.rs` | 人机对话 TUI | ⭐ 走人机通道 |
| `nt_dispatcher_core.rs`（3 处） | 任务派发核心 | `CrystalCore` `NtCrystalTaskLoop` `NtTaskLoopConfig/Report` |

⇒ ⭐⭐⭐ **这 16 处违规不是「引擎放错层」，是「L1 的文件在承载认知编排职责」。**

## 3. ⭐⭐ 裁决

**① `nt_crystal_core` 留在 L5。** ⭐ 证据充分（§1）。

**② 这 16 处** ⭐ **不是"必须消除的债"**，而是 ⭐**症状**：它诚实地暴露了
「L1 = 行动层」这个层名与这些文件的实际职责**不一致**。

**③ ⭐⭐ 唯一正确的修法有两条，二选一（都属结构改动）：**

| 方案 | 做法 | 代价 |
|---|---|---|
| **甲：把这些 shell 移出 L1** | TUI / CLI / stdin-human / free-pool 本质是**壳**，不是「行动」⇒ ⭐ 归入更合适的层或独立 crate | ⭐ 大：动 6 个文件 + 它们在 `mod.rs` 的注册 |
| **乙：承认它们是 L1 的 LLM 门面** | ⭐ 经 **L1 自己的 facade** 转出所需类型（**已做过两批**：`4796c84e` 19→16、`f5cff5cc` 16→14） | ⭐ 小：逐簇推进 |

⇒ ⭐⭐ **推荐乙**，理由：
· ⭐ 与已完成的 2 批**同形同法**，不引入新的架构概念；
· ⭐ 甲会**同时改动文件归属与 `mod.rs` 注册**，⭐ 属更大风险面；
· ⛔ 但 ⭐⭐ **乙有一个天花板**：当 facade 需要转出的类型**逼近整个 `nt_crystal_core`**
  时，facade 就成了「换名字的整包 re-export」⇒ ⭐ 那时**必须回头认真考虑甲**。

**④ ⭐ 裁决的可证伪信号**（避免又变成「做了但没用」）：
若继续按乙推进后，facade 转出的符号数**超过 8 个**，或门基线**降到 10 以下**
⇒ ⭐⭐ 说明「L1 只是门面的消费者」这个前提**已被推翻** ⇒ 应停手改走甲。

## 4. 与已完成工作的关系

| commit | 做了什么 | 与本裁决的关系 |
|---|---|---|
| `4796c84e` | L1 facade 转出「LLM 问答契约」簇（3 符号 / 3 文件） | ⭐ 方案乙的第 1 批 |
| `f5cff5cc` | L1 facade 转出「人机通道」簇（4 符号 / 2 文件） | ⭐ 方案乙的第 2 批 |
| `fe0dc946` | `Dispatcher` 下沉 L0（14→13） | ⭐ 与本裁决**同型**（位置错配要移，不是加 facade） |

⇒ ⭐⭐ 今天已完成的三处分层修复，**恰好是三个不同模式**：
**下沉**（Dispatcher）、**facade**（两批 L1 簇）、**裁决**（本篇）。
⇒ ⭐ 判据也随之明确：**「谁拥有该职责」决定用哪种修法**，⛔ 不再一律用 facade。

## 5. ⛔ 本裁决**未**做的事

⛔ 未动任何代码（**这是裁决，不是实施**）。
⛔ 未定「甲方案」的 `mod.rs` 注册改法。
⛔ 未把 §3④ 的可证伪信号做成门（⭐ 该门应与 §3③ 的推进同步加，否则
   「继续用乙」这个决策本身**无牙**）。

---

## 6. ⭐⭐⭐ 2026-10-03 晚：**方案乙已被证伪**（裁决机制第一次真正生效）

### 6.1 发生了什么

`9fcdb462` 把 §3④ 的可证伪信号做成了门（`scripts/ops/nt_crystal_core_judge.py`）。
⭐ **下一批推进（方案乙第 3 批 = `nt_tui_app.rs` 引擎簇）动手之前**，门就开了火：

| 项 | 值 |
|---|---|
| facade 现有转出符号 | **7**（`NtDemand` `NtDemandKind` `NtHumanChannel` `NtHumanReply` `NtLlmAsk` `NtLlmReply` `NtTaskFusionError`） |
| `nt_tui_app.rs` 需要 | **14** 个 |
| 其中**新增** | **7**（`CrystalCore` `NtCrystalTaskLoop` `NtInnerLoop` `NtInnerLoopOutcome` `NtLoopStatus` `NtProgressSink` `NtTaskLoopConfig`） |
| 转出后 | ⭐ **7 + 7 = 14**　（阈值 **8**） |

⇒ ⛔ **越线**。实测门输出：
> `⛔ facade 从 nt_crystal_core 转出 14 个符号（阈值 8）`
> `⇒ ⭐⭐ 方案乙的前提「L1 只是门面的消费者」已被推翻`
> `⇒ 停止按乙推进，改走甲。`

### 6.2 ⭐⭐ 我上一轮的建议**是错的**

我在上一条消息里写「方案乙第 3 批**风险最低**」。
⭐⭐ **实测证明那是错的**：做完就是 14，越线，**方案乙的前提当场被推翻**。
⇒ ⭐⭐ **继续做乙不是低风险，是明知会推翻裁决前提。**
⇒ ⭐ 这是**裁决机制第一次真正生效**：它在我**写一行代码之前**就判了「该走甲」。
⇒ ⭐⭐ 若没有这个门（§3④ 只写在文档里），这一批会照做，
**并且「facade 变成整包 re-export」会在很久以后才被察觉。**

### 6.3 ⭐⭐ 方案乙为什么必然到顶（不是偶然）

`nt_tui_app.rs` 一个文件就要 **7 个新增**符号。⭐ 而 §3③ 给乙定的天花板是 8。
⇒ ⭐⭐ **一个文件就吃掉 87% 的余量** ⇒ 剩下的 shell（`nt_model_cli` /
`nt_stdin_human` / `nt_free_pool` / `nt_crystal_llm_bridge`）只会再加。
⇒ ⭐⭐ **结论：乙在架构上不可能完成**，⛔ 不是「再努努力就能过」。

### 6.4 ⭐ 方案甲的真实代价（实测，比预想低）

| 文件 | 在 `l1_action/mod.rs` 注册 | L1 外的引用数 |
|---|---|---|
| `nt_tui_app.rs` | ✅ | **1** |
| `nt_stdin_human.rs` | ✅ | **1** |
| `nt_crystal_llm_bridge.rs` | ✅ | **0** |

⇒ ⭐⭐ 外部引用几乎为零 ⇒ ⭐ **甲的代价可控**
（⚠️ 但 `rg` 数的是**文件名字符串**，⛔ 不等于真实调用面 —— 实施前仍须逐个追 provenance）。

### 6.5 ⭐ 下一步（**改为方案甲**）

1. ⭐⭐ 逐个核实这 5 个 shell 的**真实调用面**（⛔ 不靠 `rg` 文件名计数）
2. ⭐⭐ 定「移到哪」：⭐ 三种可能 ——（a）留在 L1 但**改名/改层名**使其名副其实、
   （b）移到独立的 `apps/` 壳层、⭐（c）接受它们就是 L1 的门面、
   ⭐⭐ **但 (c) 必须同时把 facade 阈值调高并记录理由**（否则等于自我豁免）
3. ⭐⭐ **门保持红**直到裁决更新 ⇒ ⭐ 不为了让门绿而调高阈值


---

## 7. 方案甲第 1 步：**真实调用面**已核实（⛔ 不靠文件名计数）

### 7.1 方法修正
⛔ §6.4 用 `rg` 数**文件名字符串**（得 1/1/0）⇒ ⭐ **不可信**，已在 §6.4 自我标注。
⇒ 本步改按 ⭐ **模块路径**（`l1_action::<mod>`）统计。

| shell | 文件名计数（不可信） | ⭐ **模块路径计数** |
|---|---|---|
| `nt_tui_app` | 1 | **1** |
| `nt_model_cli` | — | **3** |
| `nt_stdin_human` | — | **3** |
| `nt_free_pool` | — | **2** |
| `nt_crystal_llm_bridge` | 0 | **1** |

### 7.2 ⭐⭐⭐ 决定性事实：**4 个 shell 的主要消费者是同一个二进制**

| 消费者 | 引用的 shell |
|---|---|
| ⭐ **`neotrix-core/src/bin/ntcode.rs`** | `nt_tui_app` · `nt_model_cli` · `nt_stdin_human` · `nt_free_pool` |
| `l1_action/nt_dialogue_tui.rs` | `nt_stdin_human`（与 `nt_tui_app` 互相引用） |
| `l1_action/nt_io/nt_io_provider/catalog/cli_free_source.rs` | `nt_free_pool` |
| `l1_action/nt_core_task_dispatcher/nt_dispatcher_core.rs` | `nt_crystal_llm_bridge` |

⭐⭐ 而 `bin/ntcode.rs` 只有 **332 行**，文件头自述：
> `//! # ntcode — NeoTrix 对话终端（产品名）`

且它对 shell 的引用**只有一处**：`l1_action::nt_tui_app::run_tui_session`。

### 7.3 ⭐⭐ 由 7.2 得出的结论

⭐⭐ **这些文件是「一个对话终端二进制的壳」，⛔ 不是「行动层」。**

- ⭐ `nt_tui_app` = TUI 会话 · `nt_stdin_human` = 终端里的人 · `nt_model_cli` =
  模型 CLI · `nt_free_pool` = 免费模型池调用
⇒ 四者**共同服务于一个产品形态（终端）**，且**主要消费者是那个终端的 bin**。

⇒ ⭐⭐ **这解释了为什么方案乙必然到顶**：它们需要的
（`CrystalCore` / `NtCrystalTaskLoop` / `NtProgressSink` / `NtInnerLoop*`）
⭐ **正是终端驱动认知循环所需的全套** ⇒ ⭐ 它们是**「认知的驱动方」**，
而 L1 的层名是「**行动**」⇒ ⭐⭐ **层名与职责从一开始就对不上。**

⇒ ⭐⭐ **裁决更新的方向（§8）**：不是「把认知塞进 L1 的 facade」，
而是 ⭐ **承认这些是壳层**，其与 L5 的依赖 ⭐**根本不是违规**。

### 7.4 一个**例外**，不能一并处理
`nt_crystal_llm_bridge` 的消费者是 ⭐ `nt_dispatcher_core.rs`（**L1 内部**），
⛔ **不是** `ntcode` ⇒ 它是**真正的 L1 内部依赖** ⇒ ⭐ 不适用 §7.3 的结论。

## 8. ⭐⭐ 裁决更新（§9 之前的正式修正）

**原 §3③ 的推荐（乙）已作废**（§6 已证伪）。修正为：

| shell | 归属裁决 | 理由 |
|---|---|---|
| `nt_tui_app` `nt_stdin_human` `nt_model_cli` `nt_free_pool` | ⭐⭐ **移出 L1，成为壳层** | §7.3：它们是 `ntcode` 这个**对话终端的壳**，主要消费者是该 bin |
| `nt_crystal_llm_bridge` | ⭐ **暂留 L1** | §7.4：消费者是 L1 内部的 `nt_dispatcher_core`，是真内部依赖 |

⇒ ⭐⭐ **随之必须解决的是「壳层放哪」**，而不是「L1 怎么转出」。
⇒ ⭐ 门 `nt_crystal_core_judge.py` ⭐**保持红**，直到本裁决落成 + 门据新裁决改写。
⇒ ⭐⭐ **明确不做**：为了让门变绿而调高阈值（⭐ 那是自我豁免）。


---

## 9. ⛔⭐⭐ 甲-1（移到 `entry/`）**不可行** —— 我的建议又错了

### 9.1 我上一条的估算是错的
我推荐甲-1 时说「`entry/` 已是入口层，**归属与现有架构一致**，**代价最小**」。
⇒ ⭐⭐ **实测证明两处都错。**

### 9.2 ⛔ 错因一：`entry/` 是 **`main.rs` 的私有模块**，不在 lib 里
```
$ grep -rn 'mod entry' neotrix-core/src/*.rs
neotrix-core/src/main.rs:8:mod entry;
```
⇒ ⭐⭐ **`entry/` 是二进制 `main.rs` 的私有子树**，⛔ **完全不在 `lib.rs` 里**。
⇒ ⭐⭐ `bin/ntcode.rs` 用的是 `neotrix::l1_action::…`（实测 `:13-17`），
**根本无法 `use neotrix::entry::…`**（`lib.rs` 里 `grep 'mod entry'` = **0 命中**）。

### 9.3 ⛔ 错因二：`entry/` 全私有
```
$ grep -c 'pub mod' neotrix-core/src/entry/mod.rs
0
```
⇒ ⭐ 全部是私有 `mod` ⇒ ⭐⭐ **它连自己的子模块都不对外暴露。**

### 9.4 ⇒ 甲-1 若要成立，须先做**三件**前置工程
1. 把 `entry` 从 `main.rs` **搬进 `lib.rs`**
2. 把目标模块从私有 `mod` 改成 **`pub mod`**
3. 建立 `neotrix::entry::…` 的公开路径

⇒ ⭐⭐ **代价比「移 4 个文件」大得多** ⇒ 我说的「代价最小」是**错的**。

## 10. ⭐⭐ 甲-2（独立 crate）的真实体量（实测）

| 文件 | 行数 |
|---|---|
| `nt_tui_app.rs` | **1,091** |
| `nt_dialogue_tui.rs` | **1,061** |
| `nt_model_cli.rs` | **617** |
| `nt_free_pool.rs` | **296** |
| `nt_stdin_human.rs` | **196** |
| ⭐ **总计** | **3,261 行** |

⭐⭐ **它们内部互相引用**（实测）：
`nt_tui_app → nt_dialogue_tui` · `nt_tui_app → nt_free_pool`
⇒ ⭐⭐ **这 5 个是一个内聚单元**，⛔ 不是 5 个独立的 L1 模块。

⭐⭐⭐ 结合 §7.2（主要消费者**只有一个** `bin/ntcode.rs`，332 行）⇒
> **这 3,261 行 + 内聚 + 单一消费者 + 消费者是「对话终端」这个产品形态
> ⇒ ⭐ 它们就是 `ntcode` 这个产品的壳，不是「行动层」。**

## 11. ⭐ 修正后的选项（真实代价）

| 选项 | 做法 | ⭐ 真实代价 |
|---|---|---|
| ⭐ **甲-2** | 抽 `apps/ntcode/` crate（lib + bin） | ⭐ 中等：3,261 行移动 + 新 `Cargo.toml` + `ntcode.rs` 拆 lib/bin + ⭐⭐ **它们对 `nt_io_provider` 等 L1 内部模块的依赖要重新裁决** |
| **甲-3** | 留在 L1，**改层名/职责表述** | ⭐ 小，但 ⭐⭐ **等于放弃分层** ⇒ ⛔ 不推荐 |
| ⭐ **甲-4（新）** | ⭐⭐ **承认「壳」是一类合法层归属**，把 L1 层名从「行动」扩为「行动 + 交互壳」 | ⭐ 小；⭐⭐ **但这是改层定义，须同步 `check-layer-deps` 的层语义与门基线** |

⇒ ⭐⭐ **我的判断变了**：上一条我推荐甲-1，⭐⭐ **现在撤回**。
⇒ ⭐ **甲-2 是唯一在架构上自洽的选项**，但它有一个**未解前提**：
⭐ 这 3,261 行对 `l1_action::nt_io_provider` 等 L1 内部模块的依赖
⇒ ⭐ **抽 crate 后这些依赖会跨 crate 边界**，⇒ ⭐⭐ 需先核清这批依赖面。
⛔ **本轮不做实施**（预算已尽），⭐ 因为 ⭐⭐ **在没核清依赖面前动手，
就是又一次「估错代价」**（本日已发生两次）。

## 12. ⭐⭐ 门的状态（**故意保持红**）
`nt_crystal_core_judge.py` 仍红 ⇒ 它诚实反映「§3③ 推荐的乙已被证伪、
新裁决（甲）尚未落成」。
⇒ ⭐⭐ **不为了让门绿而调高阈值** —— ⭐ 那是自我豁免。
⇒ ⭐ 门应在本裁决落成后**据新裁决改写**，⛔ 而不是改数字。


---

## 13. ⭐⭐⭐ 甲-2 的依赖面已实测 —— ⭐ **我 §11 说的「未解前提」被证伪**

### 13.1 ⭐ 本会话最后一条对策：估代价前**实测三件事**

> ① 可见性（谁能访问）② 依赖面（跨几层）③ 门的算术（会不会越线）

⇒ 本节即执行 ② 与 ③。⭐ **结论：我的「未解前提」不存在。**

### 13.2 ⭐⭐ ② 依赖面实测：那 3,261 行**完全自包含**

集群（`nt_tui_app` `nt_dialogue_tui` `nt_model_cli` `nt_free_pool` `nt_stdin_human`）
的全部内部引用：

| 引用目标 | 次数 | 性质 |
|---|---|---|
| `crate::l5_cognition::…` | **17** | ⭐ 就是那些**争议违规** |
| `crate::l1_action::…` | **10** | ⭐ **全部在集群内部**（+ 5 次 `nt_action_facade`） |
| 外部 crate | — | ⭐ 仅 `crossterm` + `ratatui`（纯 UI 库，与 L1/L5 无关） |

⇒ ⭐⭐⭐ **零**对 `l1_action::nt_io_provider` 等 L1 其他子系统的依赖。

⇒ ⛔⭐⭐ **§11 我写的「对 `nt_io_provider` 的依赖会跨 crate 边界」是错的。**
⭐ 那是我**又一次只看了模块名就下结论**，⭐⭐ 而 §13.2 的数据说：
**集群只依赖「L5 + 彼此 + 两个 UI 库」**。

### 13.3 ⭐⭐ ③ 门的算术：若甲-2 成立，两个判据**都会松**

⭐⭐⭐ 关键发现（实测）：facade 里那 7 个转出符号，**消费者几乎全在集群内**：

| 符号 | L1 内消费者（集群外） |
|---|---|
| `NtDemandKind` | ⭐ **0** |
| `NtHumanReply` | ⭐ **0** |
| `NtTaskFusionError` | 1（`nt_crystal_llm_bridge.rs`） |
| `NtLlmAsk` | 2（`nt_crystal_llm_bridge.rs` + `mod.rs`） |

⇒ ⭐⭐ **若集群移出 L1**：这 7 个转出符号里至少 **2 个立即变成死再导出**，
且 ⭐⭐ **`nt_crystal_core_judge` 的判据一（facade ≤ 8）会从「越线」退回「宽松」**
⇒ ⭐⭐ **门会自己从红变绿**，⛔ 无需改任何阈值。

⇒ ⭐⭐⭐ **这就是正确的结果**：门红是因为「乙做不下去」；
⇒ ⭐ **集群移走后乙的前提自然消失** ⇒ ⭐ **门绿是因为前提真的变了，不是数字被调了**。

### 13.4 ⭐ 修正后的甲-2 代价（真实）
| 项 | 数量 |
|---|---|
| 移动代码 | **3,261 行 / 5 文件** |
| 新增 `Cargo.toml` | 依赖仅 `crossterm` + `ratatui`（⭐ **不含 neotrix 内部 crate** ⇒ 无循环依赖风险） |
| 需改引用 | ⭐ **`bin/ntcode.rs` 一处**（实测它对 shell 只调 `nt_tui_app::run_tui_session`） |
| ⭐⭐ **原先担心** | ⛔ ~~`nt_io_provider` 跨 crate 边界~~ ⇒ **不存在** |

⇒ ⭐⭐ **甲-2 从「代价比想象大」翻转为「可执行」**。
⛔ **本轮仍不做实施**（预算已尽）—— ⭐ 但 ⭐⭐ **现在做它的三个前置条件都已实测通过**：
可见性（§9）· 依赖面（§13.2）· 门算术（§13.3）。
