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
