# C6 进化循环升级计划 (基于学术研究)

## 研究基础

### 关键论文
1. **"A Survey of Self-Evolving Agents" (TMLR 2026)**
   - 三维度框架: What to evolve, When to evolve, How to evolve
   - 进化机制: Model, Memory, Tools, Architecture
   - 来源: arXiv:2507.21046

2. **"Self-Evolving AI Agents — The New Paradigm of 2026"**
   - MAPE-K 架构: Monitor, Analyze, Plan, Execute, Knowledge
   - 闭环反馈: 快照→蒸馏→落盘→反馈

3. **Awesome-Self-Evolving-Agents (XMUDeepLIT)**
   - 402 stars, 系统性综述
   - 模型-环境共进化

### GitHub 参考项目
| 项目 | Stars | 核心机制 |
|------|-------|----------|
| self-evolving-software | 3 | MAPE-K 双平面架构 |
| GenericAgent | 4.3K | 技能结晶化 |
| Evolver | 4.7K | 基因组进化协议 |
| Neo.mjs | - | MX 循环 (Model Experience) |

## C6 进化循环定义

### 四阶段闭环
```
┌─────────────────────────────────────────────────────────────┐
│                    C6 进化循环                              │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  快照    │→ │  蒸馏    │→ │  落盘    │→ │  反馈    │   │
│  │ Snapshot │  │  Distill │  │  Persist │  │ Feedback │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│       ↑                                        │           │
│       └────────────────────────────────────────┘           │
│                    持续进化闭环                             │
└─────────────────────────────────────────────────────────────┘
```

### 各阶段要求

#### 1. 快照 (Snapshot)
- 收集模块健康状态 (health score)
- 收集性能指标 (latency, throughput, error_rate)
- 收集使用统计 (invocation_count, success_rate)
- 输出: `EvolutionSnapshot`

#### 2. 蒸馏 (Distill)
- 分析快照数据, 识别模式
- 生成洞察 (patterns) 和建议 (recommendations)
- 对标: MAPE-K 的 Analyze 阶段
- 输出: `EvolutionInsight`

#### 3. 落盘 (Persist)
- 持久化蒸馏结果到 KB
- 更新模块元数据
- 记录历史轨迹
- 对标: MAPE-K 的 Knowledge 阶段

#### 4. 反馈 (Feedback)
- 根据洞察采取行动 (Monitor/Optimize/Refactor/Upgrade)
- 触发进化升级 (constellation promotion)
- 对标: MAPE-K 的 Execute 阶段
- 输出: `EvolutionAction`

## 升级验证清单

### Phase 1: 基础能力 (C0→C1)
- [ ] 模块可编译 (cargo check 0 errors)
- [ ] 实现 SelfTest trait
- [ ] 至少 3 个单元测试
- [ ] 测试全部通过

### Phase 2: 集成能力 (C1→C2)
- [ ] 集成测试通过
- [ ] 生产接线完成
- [ ] 跨模块契约验证

### Phase 3: 性能基准 (C2→C3)
- [ ] 建立性能基准
- [ ] 无性能回归
- [ ] 基准可度量可对比

### Phase 4: 管线集成 (C3→C4)
- [ ] 接入 SEAL 主流水线
- [ ] 被生产路径消费
- [ ] 输出可被下游使用

### Phase 5: 自愈能力 (C4→C5)
- [ ] 错误自动重试
- [ ] 格式降级
- [ ] 性能降级
- [ ] 进度回调

### Phase 6: 进化循环 (C5→C6) ← 最终目标
- [ ] 实现 EvolutionCapable trait
- [ ] 实现 snapshot() 方法
- [ ] 实现 distill() 方法
- [ ] 实现 persist() 方法
- [ ] 实现 feedback() 方法
- [ ] 注册到 EvolutionLoopManager
- [ ] 通过 C6 自检

## 试点模块: file_adapter

### 当前状态
- Constellation: C4 (已升级到 C5)
- SelfTest: T1+T2+T3
- 测试: 8 单元 + 1 基准

### 升级到 C6 的步骤
1. 实现 EvolutionCapable trait
2. 添加快照/蒸馏/落盘/反馈实现
3. 添加进化循环测试
4. 注册到 EvolutionLoopManager
5. 验证通过

## 升级优先级

### 高优先级 (立即升级)
1. NT-IO 域: file_adapter, batch_processor, output_formatter
2. NT-CORE 域: 核心基础模块

### 中优先级 (1-2 周)
1. NT-MIND 域: 进化循环相关模块
2. NT-MEMORY 域: 知识存储模块

### 低优先级 (2-4 周)
1. 其他域的模块
2. 辅助性模块
