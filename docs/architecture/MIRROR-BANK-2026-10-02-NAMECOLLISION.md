# 同名不同物：`KnowledgeSource` 裁决（2026-10-02）

## 结论：**不收敛**。这是**命名冲突**，不是重复副本。

## 实测数据

| 集合 | 数量 | 语义 |
|---|---|---|
| 交集 | 51 | — |
| 仅 `neotrix-types/src/core/nt_core_knowledge/types.rs` | **40** | **外部工具/项目名** |
| 仅 `neotrix-core/src/l0_substrate/nt_core_substrate_types.rs` | **14** | **认知科学理论名** |

- 低层独有 40：`LangMem`、`LettaMemory`、`HindsightMemory`、`CogneeMemory`、
  `OpenSwe`、`QwenCode`、`Maigret`、`Crush`、`ClawCode`、`LiteParse`、`SkillOpt` …
- 高层独有 14：`ActiveInference`、`GlobalWorkspaceTheory`、
  `IntegratedInformationTheory`、`PredictiveCoding`、`JEPAWorldModel`、
  `OrchOR`、`HyperAgents`、`AttentionSchema`、`VSAHyperdim` …

## 为什么不能收敛
1. 两个集合**语义正交**（工具 vs 理论），不是「一份漂移成两份」。
2. 收敛会造成二选一坏结果：
   - 把 40 个工具名灌进认知理论枚举 ⇒ **语义污染**
   - 从低层删掉 14 个理论变体 ⇒ **丢能力**
3. 低层那 40 个**不是死变体**：`core/nt_core_knowledge/sources.rs`
   有 **250 处** `KnowledgeSource::` 引用。

## 正确处置：**改名**
拆成两个语义明确的类型（如 `CognitiveTheorySource` / `ExternalToolSource`）。
⛔ 涉及 **39 个文件**，需独立批次裁决，不适合顺手做。

## 过程教训（比结论更重要）
本轮我先把它列进「5 个跨 crate 重复类型」清单，理由是
`struct/enum` 同名同字段 —— 前 3 个（`ReasoningHexagram`/`Hexagram`/`FermionState`）
确实如此，收敛成功；**于是我把「模式成立」当成了「模式普适」**。

⛔ 若机械执行，会把 40 个外部工具名灌进认知理论枚举。
拦住它的不是更小心，而是**去数了变体**。
⇒ 纪律升级：**清单上的「N 个同类」不构成授权**，
每一项都要单独验证结构；**变体/字段的集合比较比签名比较更能暴露分歧**。
（本 session 同型获救 3 次：类型契约分叉、ASCII 子串误报、`ReasoningHexagram` panic 分支。）
