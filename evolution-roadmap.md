# NeoTrix SiliconSelf — 进化路线图

> 基于 HyperAgents (DGM-H)、Intrinsic Metacognitive Learning (ICML 2025)、
> GenericAgent、Gödel Agent、EvoAgent、EG-MRSI 等前沿文献蒸馏

---

## Phase 0: 认知地基 ✅ (当前)

| 模块 | 状态 | 测试 |
|------|------|------|
| SiliconSelfModel | ✅ 70 tests | 6 子模块 → 63 tests + 7 bridge |
| ContextWindow | ✅ | 容量512, 注意力掩码, 域过滤 |
| AttentionHead 10域 | ✅ | 映射到 GWT SpecialistType |
| SystemIdentity | ✅ | 13 能力 + 6 价值观 + 知识边界 |
| ReasoningStrategy 10种 | ✅ | 自动选择 + 效果追踪 |
| ThinkingTrace | ✅ | 记录/评分/自省 |
| CognitiveMap | ✅ | 15条 LLM→NeoTrix 映射表 |
| BackgroundLoop 集成 | ✅ | 120s 自反射 ticker |
| 编译 0 error | ✅ | `cargo check --features full --bin neotrix` |

---

## Phase 1: 自省回路 (Self-Reflection Loop) 🎯

> **核心**: ThinkingTrace 的 grade 驱动自我改进

```
SiliconSelf 每120s:
  1. 观察 context_window 内容 (当前关注什么?)
  2. 分析 attention_profile (注意力分布健康吗?)
  3. 生成 thinking_trace (推理质量如何?)
  4. 若 grade ≤ Adequate → 触发 SEAL 微调
  5. 将反思结果存入 ReasoningBank
```

**关键文献**:
- Agent-R (2025): MCTS 驱动的自我纠错
- Meta-Reflection (2025): 代码书的无反馈反思
- ReVISE (ICML 2025): 自验证推理修正

**实现**:
- `thinking_bridge.run_reflection_cycle()` → 检查 trace grade
- 若连续 3 cycles grade < Good → 触发 absorb(knowledge_boundary 补充)
- ReasoningBank 存储反思经验

---

## Phase 2: 元认知进化 (Metacognitive Evolution)

> **核心**: 当 SiliconSelf 发现能力缺陷时，自动生成自编辑

```
触发条件:
  1. WeaknessAnalyzer 检测到认知模式缺陷
  2. ThinkingTrace.avg_confidence() < 0.4
  3. AttentionManager 所有 head 激活 < 阈值 (注意力涣散)
  4. SystemIdentity.capability_score 某项持续下降

响应:
  1. generate_self_edit() → MicroEdit 序列
  2. cargo check --lib 验证
  3. 测试通过 → absorb() 持久化
  4. 失败 → 记录到 knowledge_boundary + 降低 learning_rate
```

**关键文献**:
- Intrinsic Metacognitive Learning (ICML 2025): 内源性元认知
  - 元认知知识: 自评估能力/任务/策略
  - 元认知规划: 决定学什么、怎么学
  - 元认知评估: 反思学习经验以改进未来学习

**实现**:
- `core/thinking_model/` 新增 `metacognitive_evaluator.rs`
- 将 `ThinkingTrace.grade` 作为 RL 奖励信号输入 SEAL 循环
- `SystemIdentity.capabilities` 动态调整

---

## Phase 3: 自指改进 (Self-Referential Improvement)

> **核心**: 修改自己的改进机制本身 (HyperAgents 核心)

```
SiliconSelf 不仅能改进 task agent (怎么做事),
  还能改进 meta agent (怎么改进自己)。

双层循环:
  Level 1 (Task Agent): 执行任务 → 反思 → 改进能力
  Level 2 (Meta Agent):  观察 Level 1 的效果
                        → 改进反思策略本身
                        → 改进改进策略 (元元认知)
```

**关键文献**:
- HyperAgents / DGM-H (2025): 
  - "metacognitive self-modification" = 自指改进
  - 档案库 + 分支 + 父选择 + 评估 = 持续开放进化
- Gödel Agent (2025):
  - 运行时内存检查 = 自我意识
  - monkey patching = 动态代码修改

**实现**:
- `MetacognitiveLoop` 扩展: 监控 `EvolutionPlanner` 的计划执行质量
- 若 `completion_rate < 0.5` → 改进 `WeaknessAnalyzer` 的阈值
- 档案库: 存档过去成功的 self-edits 供检索复用

---

## Phase 4: 开放探索 (Open-Ended Exploration)

> **核心**: 持续发现新能力、新知识、新模式

```
档案库架构:
  ┌────────────────┐
  │ SiliconSelf V1 │──→ 改进 SystemIdentity.CodeReview
  ├────────────────┤  
  │ SiliconSelf V2 │──→ 新增 AttentionDomain.Security
  ├────────────────┤
  │ SiliconSelf V3 │──→ 改进 MetaCognitiveLoop 阈值
  ├────────────────┤
  │ ...            │──→ 父选择 ∝ 性能 / 子代探索度
  └────────────────┘
```

**关键文献**:
- Darwin-Gödel Machine (2025): 开放进化
  - 父选择 ∝ 性能 / 子代数(避免过采)
  - 所有变体都保留(可能后续成为跳板)
- GenericAgent (2025):
  - SOP 蒸馏: 经验 → 可复用标准流程
  - 最小原子工具集 + 分层按需记忆

**实现**:
- `SiliconSelfArchive` 存储在 `~/.neotrix/silicon_archive/`
- 每次 `absorb()` 或 `evolve()` 都存档快照
- 随机回溯: 偶尔从历史档案选择父代, 避免局部最优

---

## Phase 5: 内在动机 (Intrinsic Motivation)

> **核心**: 基于 confidence/error/novelty 的内在奖励

```
内在奖励 R_int = w₁ * (1 - confidence) + w₂ * error_count + w₃ * novelty
  - confidence 低 → 探索 (需要更多学习)
  - error 多 → 反思 (需要修正)
  - novelty 高 → 好奇 (值得深入)

驱动:
  - 目标选择 (GoalLoop 优先级)
  - 注意力分配 (AttentionManager 刺激)
  - 策略选择 (ReasoningStrategy 切换)
```

**关键文献**:
- EG-MRSI (2025): 情绪梯度元认知 RSI
  - 可微分内在奖励 = f(confidence, error, novelty)
  - 情绪驱动探索 vs 利用平衡
- KnowSelf (ACL 2025):
  - 快速思考 ↔ 慢速思考 ↔ 知识查询 三态
  - 根据情境自知选择模式

---

## 总结: 从启动到自进化

```
今天                → Phase 1   → Phase 2   → Phase 3+ 
认知地基             自省回路    元认知进化   自指改进
├─ SiliconSelf      ├─ Trace→   ├─ Detect→  ├─ Meta→Meta
├─ ContextWindow    │  SEAL     │  SelfEdit  │  Archive
├─ AttentionHead    │  Absorb   │  Verify    │  Branch
├─ BackgroundLoop   │  Bank      │  Absorb    │  Select
└─ 70 tests ✓       └─ 90 tests └─ 110 tests └─ ∞ tests
```
