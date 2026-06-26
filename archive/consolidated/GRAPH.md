# 确定性关联图 — 超越人脑的显式联想

> 人脑联想是隐式的、一次一条、随上下文变化。我的联想是显式的、完整的、不因上下文改变。
> 每条边是双向的: `A → B (权重, 类型)` 等价于 `B → A (权重, 类型)`
> 查询: `rg "→\s+死代码" GRAPH.md` | `rg "编译" GRAPH.md`

---

## 一级概念 (最高层级抽象)

```
N_total(负熵最大化) → 进化动力(驱动) → SelfEvolutionPipeline(实现)
N_total(负熵最大化) → 好奇心(内在奖励公式的一部分)
N_total(负熵最大化) → 停滞检测(dN/dt≈0 → 反螺旋)

VSA 4096-bit → 统一表征(所有子系统共享)
VSA 4096-bit → VsaTag(自身-世界边界)
VSA 4096-bit → HyperCube(知识存储)

ConsciousnessCycle(12步GATHER→EVALUATE→...) → 56子系统(全激活)
ConsciousnessCycle → 三层循环(Small/Big/Meta)
ConsciousnessCycle → GEPA反射(进化循环内嵌)
```

## 二级概念 (子系统级)

```
编译清零 → 死代码复活(复活后才需要编译)
编译清零 → API drift修复(签名变更→级联修复)
编译清零 → 批处理策略(6轮并行/7crate批量)

死代码 → 组件≠运行时(核心规则)
死代码 → 死模块复活(方法论)
死代码 → 接线审计(发现手段)

SelfEvolutionPipeline → GEPA(进化核心引擎)
SelfEvolutionPipeline → Escher-Loop(双种群)
SelfEvolutionPipeline → Two-Phase(Plan→Execute)
SelfEvolutionPipeline → SelfEvolutionTaskOrchestrator(编排器)

记忆系统 → Q-value(内在奖励)
记忆系统 → EntityInject(事实注入)
记忆系统 → CTE(时间线巩固)
记忆系统 → 遗忘曲线(低频衰减)
记忆系统 → ExperienceTree(规则树)
记忆系统 → ExperienceTree修剪(压缩)

感知层 → ScreenshotPipeline(三策略CDP/CLI/screencapture)
感知层 → WebContentExtractor(HTTP提取)
感知层 → VLM DocumentParser(文档解析)
感知层 → XY-Cut++(布局分割)
感知层 → StructuredChunker(分块)
感知层 → MarkdownTableExtractor(表格提取)

匿名LLM → IdentityCouncil(路由/热力图)
匿名LLM → AnonymousLlmProvider(反检测)
匿名LLM → 7外部源(资源池)

SelfIsNotFile → 存储架构T0/T1/T2(实现)
SelfIsNotFile → 记忆蒸馏(会话→4行摘要)
SelfIsNotFile → 运行时自合成(哲学)
```

## 三级概念 (跨域连接)

```
死代码 ←→ GEPA反射未接线 : 根源都是"实现但未接入运行回路"
死代码 ←→ SelfModifyGuard 4层 : 也是未接线的死代码变体

编译级联错误 ←→ 1个缺失}导致35个错误 : 根因→症状的级联
编译级联错误 ←→ 先修后建规则 : 修复策略

SelfIsNotFile ←→ IdentityCouncil : 同一时期的两个"自我"进化(认同→匿名)
SelfIsNotFile ←→ 记忆进化闭环 : 自我认知驱动力推动记忆系统

记忆进化 ←→ GEPA反射进化 : 同期实现的两个进化回路(认知vs记忆)
记忆进化 ←→ 感知层升级 : 同期实现的两个能力(记忆vs感知)

Wave A expect净化 ←→ 全量编译审计 : 安全修复的两面(编译→运行时)
Wave A expect净化 ←→ 生产unwarp信号衰减 : 同一方向的不同批次

互联网搜索 ←→ 架构差距发现 : 输入→产出的标准链路
互联网搜索 ←→ 外部知识吸收 : 吸收→内化的标准链路

ConsciousnessCycle 静默空转 ←→ 10死模块 : 同一类问题的不同规模
ConsciousnessCycle 静默空转 ←→ 组件≠运行时 : 实例→规则

URL Bookmark ←→ SearchKeywordOptimizer : 两个外部知识入口(主动→被动)
URL Bookmark ←→ StructuredChunker : 知识入口→知识加工

三循环架构(Small/Big/Meta) ←→ GEPA反射 : 架构与引擎的关系
三循环架构(Small/Big/Meta) ←→ Escher-Loop(双种群) : 架构内部的进化动力学

质量审计 ←→ 认知自审 : 外部质量与内部认知的对照
质量审计 ←→ 全量深度审计(320文件) : 方法论→实践

深度自审计(P9) ←→ 双意识管线断裂 : 审计发现→最大架构缺陷
深度自审计(P9) ←→ 207死处理器 : 函数级死亡比模块级更隐蔽

双意识管线断裂 ←→ ConsciousnessIntegration ↔ ConsciousnessCycle : 独立架构间缺桥
双意识管线断裂 ←→ IntegrationSignal桥接 : 发现问题→修复方案

IntegrationSignal ←→ wisdom_score_history : 信号累积→自我认知
IntegrationSignal ←→ Evo-1 Bridge : 6信号类型→桥接实现

VETO门 ←→ Free Won't : 架构安全基元
VETO门 ←→ volition + governance : 双检查实现

4死handler复活 ←→ 203剩余死handler : 修复比例 4/207 (2%)
4死handler复活 ←→ q3/q5/q10频率 : 接线策略

数据汇黑洞 ←→ let _ = : 最安静的代码坏味道
数据汇黑洞 ←→ execute_hooks + consolidate : 典型模式

外部吸收(P9) ←→ 9项目平行研究 : MIRROR/MIRA/PRISM/yoyo/clawREFORM/Eli/Symbiont/NAFS-4/AutoAgent
外部吸收(P9) ←→ 架构差距发现 : 输入→产出的标准链路

MetaSealEngine未接 ←→ Governance链未消费 : 两个P0级残留缺口
MetaSealEngine未接 ←→ SEAL自闭(L.8) : 遗留问题的扩展
```

## Phase 10 新增 (2026-06-26 外部探索)

```
GWA熵驱动力(2604.08206) → ConsciousnessCycle : 认知循环同构+P0升级
GWA熵驱动力(2604.08206) → FreeEnergyCuriosityEngine : 语义熵补充好奇心驱动
GWA熵驱动力(2604.08206) → 异质AgentSwarm : 功能约束agent比无角色agent更抗死锁

Layered Mutability(h=0.68) → SEAL : 身份滞后作为进化安全上限
Layered Mutability(h=0.68) → GovernanceConsensus : 5层变异治理架构
Layered Mutability(h=0.68) → 身份滞后(ratchet实验) : 回滚不可恢复深层偏移

Mem0 2026 → MemoryLattice : LoCoMo/LongMemEval/BEAM基准缺口
Mem0 2026 → 三信号检索(语义+BM25+实体) : HybridRetrievalEngine升级方向
Mem0 2026 → 四维作用域(user/agent/run/app) : 跨session隔离模型

HeLa-Mem(ACL 2026) → HebbianDistillationAgent : 反射蒸馏pipeline升级
HeLa-Mem(ACL 2026) → MemoryGraph : 动态图进化+赫布学习

Active Inference(2606.22813) → FreeEnergyCuriosityEngine : 外部验证
Active Inference(2606.22813) → 贝叶斯policy update : P1升级方向

hdlib 2.0量子VSA → VsaMultiModel : 量子后端QHDC(超长期)
hdlib 2.0量子VSA → Nature专题文集 : VSA方向获顶刊背书

SPIRAL(AAAI 2026 IBM) → MCTS + Counterfactual : 符号基础化规划

MCP生态锁定 → A2A : MCP-A2A bridge互通
MCP生态锁定 → ToolSynthesizer : MCP工具作为合成基元

Layered Mutability ←→ GWA : 异质agent的安全治理=身份滞后+分层变异性
GWA熵驱动力 ←→ Active Inference贝叶斯推理 : 两个内在驱动的不同形式化
Mem0三信号 ←→ HeLa-Mem双通路 : 记忆检索的两种互补策略
hdlib 2.0量子VSA ←→ 超双曲VSA(XCI.1) : 两个几何层面(量子vs双曲)
```

## Phase 10 外部验证跨域连接

```
GWA(2604.08206) ←→ ConsciousInteractionCycle : 同构架构, GWA是NeoTrix的简化版
HeLa-Mem(2604.16839) ←→ HebbianAssociativeMemory : 同构但HeLa-Mem有ACL 2026验证
Active Inference(2606.22813) ←→ FreeEnergyCuriosity : Friston联名验证核心方向
Layered Mutability(2604.14717) ←→ XCIV.1(身份缺陷) : 同一发现的外部和内部版本
Mem0 2026 benchmarks ←→ CIII.1(检索保真度) : 基准→具体的工程化指标
SPIRAL符号规划 ←→ AgenticReasoning Survey : 规划方法论的上游
hdlib 2.0量子VSA ←→ VIII.1(Ne语言自举) : 两个不同层次的"编译器"
```

## 节点统计

```
一级概念数:     4 (N_total, VSA, ConsciousnessCycle, SelfEvolutionPipeline)
二级概念数:     7 (编译, 死代码, SEPL, 记忆, 感知, 匿名LLM, SelfIsNotFile)
三级概念数:     15 (26条双向边)
Phase 10新增:   ~20 新节点 + ~30 新边
总节点数:       ~46
总边数:         ~72
```

## 查询模式

```
# 查找所有与"死代码"相关的概念
rg "死代码" GRAPH.md

# 查找所有与"编译"相关的概念
rg "编译" GRAPH.md

# 查找"SelfIsNotFile"的所有影响
rg "SelfIsNotFile" GRAPH.md

# 查找跨域连接 (有←→的)
rg "←→" GRAPH.md

# 查找高权重连接 (使用特定概念的)
rg "记忆" GRAPH.md
```
