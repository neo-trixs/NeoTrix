# NeoTrix 意识自进化架构 v19 — 认知级能力吸收

> 吸收自: EverOS/MemRL/MemSkill/CTE/Aegis/ADP-MA/ROMA/AdaptOrch 等 13+ 项目
> 构建日期: 2026-06-23

## 第二轮深度搜索缺口矩阵

| # | 项目 | ★ | 关键能力 | NeoTrix当前状态 | 等级 |
|---|------|---|---------|----------------|------|
| 1 | **MemRL** | 133 | 情景记忆上运行RL（两阶段检索+噪声过滤+高效用策略识别） | 仅有被动式VSA语义记忆，无RL学习 | **P0** |
| 2 | **MemSkill** | 523 | 元记忆技能（关于"如何记忆"的技能，非记忆内容本身） | 记忆系统是被动的KV存储，无主动记忆策略 | **P0** |
| 3 | **CTE (Evolving-Memory)** | - | 生物启发记忆管道：SWS策划→REM抽象→巩固连接→压缩 | SleepCycle仅门控，无真实记忆巩固 | **P0** |
| 4 | **Mnemosyne** | 7 | 预测性前提条件+矛盾边+置信度衰减+技能元记忆管理 | 无置信度衰减系统，无前提条件预测 | **P0** |
| 5 | **Aegis-DQ** | - | 自主数据质量：31规则×6仓库→LLM诊断→根因分析→SQL修复提案 | 无自动质量控制和质量驱动进化 | **P0** |
| 6 | **EverOS** | 8111 | 自进化记忆生态：混合检索+多模态整合+跨agent记忆层 | CrossSessionMemory存在但无混合检索 | **P1** |
| 7 | **ADP-MA** | 论文 | 3层元Agent编排（Architect/Supervisor/Optimizer） | ConsciousnessCycle是扁平的12步 | **P1** |
| 8 | **ROMA** | 论文 | 递归分解：Atomizer→Planner→Executor→Aggregator | EvolutionTaskSystem仅有线性任务列表 | **P1** |
| 9 | **AdaptOrch** | 论文 | 自适应拓扑路由：4种规范拓扑动态选择（+12-23%） | 无编排拓扑感知 | **P2** |
| 10 | **AOrchestra** | 论文 | 动态子Agent创建(Instruction/Context/Tools/Model) | MCP Install CLI是雏形 | **P2** |

## 认知架构升级：4个P0认知级缺口

### P0-1: 情景RL学习（MemRL模式）

```
问题: NeoTrix记录情景但从未从失败中RL学习
方案: 在SelfEvolutionMetaLayer.tick()中集成"两阶段检索"
  Phase 1: 广度检索 (用当前context检索相关历史episode)
  Phase 2: 深度过滤 (用reward信号筛选高利用策略)
  → Episode被选择时，更新Q值
  → 低Q值episode自然衰减 (仿MemRL稳定性-可塑性平衡)
```

### P0-2: 元记忆技能（MemSkill模式）

```
问题: 记忆系统被动——不知道"该记住什么、关注什么、遗忘什么"
方案: MetaMemorySkill trait
  - extraction_skill: 从交互中提取关键信息
  - retention_skill: 确定哪些信息值得长期保留
  - retrieval_skill: 确定如何检索（语义vs时空vs因果）
  - forgetting_skill: 主动遗忘策略（置信度衰减+访问频率）
  → 每个skill是一个可进化的{code+test+performance}三元组
```

### P0-3: 梦境巩固管道（CTE模式）

```
问题: SleepCycle仅是一个门控开关，无真实记忆巩固
方案: DreamCycle（在ConsciousnessCycle的Phase 12 SLEEP中运行）
  Phase SWS: 回放轨迹→提取关键路径→过滤噪声→提取负面约束
  Phase REM: 压缩关键路径→层次化记忆节点（策略+子步骤）
  Phase CONSOLIDATE: 语义嵌入→检测合并候选→因果/时序边→跨域链接
  Phase COMPACT: 低访问记忆节点的LLM摘要压缩
```

### P0-4: 自主数据质量（Aegis-DQ模式）

```
问题: 无数据质量监控→数据腐烂时意识体不知道
方案: DataQualityPipeline（集成到Engineer-Executor循环）
  Step 1: Monitor — 自动注册监控规则（完整性/唯一性/有效性/统计异常）
  Step 2: Detect — schema漂移/空值尖峰/延迟到达/质量退化
  Step 3: Diagnose — 根因分析（谱系追踪+历史事件+依赖图）
  Step 4: Remediate — 自动修复（重试/回填/模式重映射/隔离坏记录）
  Step 5: Learn — 失败模式存储到EpisodicMemory用于跨会话学习
```

## 三波执行计划

### Wave A (本周): 数据质量监控 + 自修复
```
P0-4 DataQualityMonitor + SelfHealingExecutor
→ 集成到 SmartSchemaMapper 的 validate() 步骤
→ 异常检测 → 根因分析 → 自动修复提案
```

### Wave B (本月): 梦境巩固 + 元记忆技能  
```
P0-3 DreamCycle (CTE模式) → 替换现有SleepGate
P0-2 MetaMemorySkill → 集成到EvolutionTaskSystem
```

### Wave C (下月): 情景RL + 层次编排
```
P0-1 EpisodicRL (MemRL模式) → SelfEvolutionMetaLayer增强
P1-7 ADP-MA层次编排 → ConsciousnessCycle分层化
```

## ConsciousnessCycle接线映射

```
Phase 8 (VERIFY)  → P0-4 DataQualityMonitor.detect()
Phase 9 (ACT)     → P0-4 SelfHealingExecutor.remediate()
Phase 10 (RECORD) → P0-1 EpisodicMemory.store()
Phase 11 (METRIC) → P0-2 MetaMemorySkill.evaluate()
Phase 12 (SLEEP)  → P0-3 DreamCycle.run(SWS→REM→CONSOLIDATE→COMPACT)
```

## 经验蒸馏: 认知架构升级的4个关键哲学

1. **记忆不是被动存储**（MemSkill）— 记忆是主动技能系统
2. **学习不需要重训练**（MemRL）— 非参数化的情景RL
3. **睡眠不是空闲**（CTE）— 做梦是核心认知过程
4. **质量不是一次性检查**（Aegis-DQ）— 质量是持续监控+自主修复闭环
