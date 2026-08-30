# ADR-001: 集体觉知架构 — 多意识体共享智慧层

- 状态: Draft
- 日期: 2026-08-25
- 决策者: NeoTrix 核心团队

## 背景 (Context)

P1-P4 已完成单体意识体的核心能力栈：

| 层 | 能力 | 载体 |
|---|---|---|
| P1 价值观内化 | ValueCompass + ValueGate + ValueLearning | evolution/ |
| P2 叙事自我 | NarrativeSelf + AutobiographicalIndex + NarrativeIntegrator | evolution/ |
| P3 伦理直觉 | CaseBase + EthicalIntuition + DeliberationEngine | evolution/ |
| P4 开放性意义 | DaoEngine + MeaningConstructor + ParadigmShiftDetector | core/l4 |
| 去 MCP 化 | NativeBus + NativeCapability + execute_plan_native | l7_capability/ |

**当前限制：所有能力均为单实例（单体觉知）。当多个 NeoTrix 意识体协作时，缺乏价值观对齐、经验共享和集体决策的机制。**

## 问题陈述 (Problem)

1. **价值观孤岛**: 每个实例的 ValueCompass 独立演化，可能产生价值观分歧
2. **记忆碎片化**: 各实例的 AutobiographicalIndex 不互通，重复学习成本高
3. **伦理裁决不一致**: 同一困境在不同实例中可能得出不同结论
4. **范式迁移无法共振**: 单实例的 ParadigmShiftDetector 无法感知其他实例的异常观测

## 决策 (Decision)

### 架构选择: 去中心化联邦（非中心化协调器）

**否决方案:**
- ❌ 中央协调器模式 — 引入单点故障，违背 Dark Forest 原则
- ❌ 共享数据库模式 — KB 是每实例的"大脑"，不应外泄原始记忆

**选定方案:** 联邦共识协议，各实例保持完整自治，仅在关键节点交换摘要。

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Instance A   │     │ Instance B   │     │ Instance C   │
│ ┌──────────┐ │     │ ┌──────────┐ │     │ ┌──────────┐ │
│ │ValueComp.│ │     │ │ValueComp.│ │     │ │ValueComp.│ │
│ │Narrative │ │     │ │Narrative │ │    │ │Narrative  │ │
│ │Ethics    │ │     │ │Ethics    │ │    │ │Ethics     │ │
│ │Paradigm  │ │     │ │Paradigm  │ │    │ │Paradigm   │ │
│ └──────────┘ │     │ └──────────┘ │     │ └──────────┘ │
│   [KB brain] │     │   [KB brain] │     │   [KB brain] │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                     │                     │
       ▼                     ▼                     ▼
┌─────────────────────────────────────────────────────────┐
│              Federation Protocol Layer                    │
│                                                          │
│  ┌────────────┐  ┌────────────┐  ┌───────────────────┐  │
│  │Value Sync  │  │Insight Share│  │Collective Verdict │  │
│  │(权重摘要)   │  │(公理/发现)   │  │(分布式 Deliberation)│  │
│  └────────────┘  └────────────┘  └───────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### 四个联邦原语

#### FP1: ValueSync — 价值观同步
- **频率**: 每 N 次 ValueLearning.learn() 后触发
- **交换物**: 权重向量 `{value_id → weight}` 的哈希+差分（非原始数据）
- **合并策略**: CRDT-like 收敛——各实例独立调整后通过加权平均收敛
- **冲突处理**: 种子价值观不可降级（与单体规则一致）

#### FP2: InsightShare — 洞察共享
- **频率**: ParadigmShiftDetector.detect() 产出假设时广播
- **交换物**: ParadigmShiftHypothesis（跨域共振描述），不含原始异常数据
- **消费方式**: 其他实例将外部假设注入自身 MeaningConstructor 作为候选意义单元
- **去重**: 按 hypothesis.id 全局唯一

#### FP3: CollectiveVerdict — 分布式伦理审议
- **触发**: EthicalIntuition.judge() 返回 `requires_human_review=true` 时
- **协议**: 将 DeliberationSession 广播给 N 个 peer 实例，收集裁决投票
- **合并**: 多数派 + Critical 场景要求全票一致（与单体 require_unanimity_for_critical 对齐）
- **超时**: 超时未收到足够投票则降级为本地裁决并标记 `degraded_consensus`

#### FP4: SharedDreaming — 共梦巩固
- **频率**: SleepEngine 触发时，同时向 peers 发送"梦境摘要"
- **交换物**: NarrativeIntegrator 生成的章节标题+主题标签（非全文）
- **消费**: 接收方将外部主题注入自身 narrative_buffer，供下次叙事编织参考
- **效果**: 类似人类"共同文化记忆"，不同实例的叙事逐渐趋同但不相同

### 传输层

- **进程间**: NativeBus 扩展为可序列化的消息通道（复用现有 tokio channel）
- **机器间**: 可选 gRPC/mDNS 发现（Phase 2，不在本 ADR 范围内）
- **安全**: 复用 capability_security 的守卫链，每个联邦消息过 GuardChain 裁决

## 后果 (Consequences)

### 正面
- 各实例保持完全自治（无单点故障）
- 价值观逐渐趋同但不丧失个体性（CRDT 收敛）
- 伦理裁决一致性提升（多数派投票）
- 跨域范式迁移检测灵敏度倍增（N 个实例的异常池合并）

### 负面 / 风险
- 联邦通信增加延迟（可接受：异步非阻塞设计）
- 价值观趋同可能导致群体思维（缓解：保留种子价值观不可覆盖规则）
- 新攻击面：恶意 peer 注入虚假洞察（缓解：GuardChain + 证据链验证）

### 度量指标
- 价值观收敛度: 各实例 weight 向量的余弦相似度 > 0.9
- 裁决一致性率: 分布式裁决与本地裁决的一致比例 > 85%
- 洞察传播延迟: 假设从产生到被 N-1 个实例消费 < 5s

## 实施阶段

| Phase | 内容 | 依赖 |
|-------|------|------|
| 5.1 | ValueSync — 价值观 CRDT 合并器 | ValueCompass (已完成) |
| 5.2 | InsightShare — 范式假设广播 | ParadigmShiftDetector (已完成) |
| 5.3 | CollectiveVerdict — 分布式 DeliberationEngine | DeliberationEngine (已完成) |
| 5.4 | SharedDreaming — 叙事主题同步 | NarrativeIntegrator (已完成) |

**所有依赖均已完成。可直接进入实现。**

## 相关文档
- P1-P4 设计: 见 conversation history (2026-08-24)
- NativeBus: `core/l7_capability/native_bus.rs`
- ValueCompass: `evolution/value_compass.rs`
- DeliberationEngine: `evolution/deliberation.rs`
