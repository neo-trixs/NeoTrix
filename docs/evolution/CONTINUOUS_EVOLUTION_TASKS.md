# NeoTrix 持续进化任务

## 任务系统架构

进化任务系统分三层：

```
SelfEvolutionEngine (活跃, 每50cycle)
  ├─ 信号收集: MetaArchEvoLoop.assess() → EvolutionSignal
  ├─ 优先级排序: impact × urgency × feasibility
  └─ 路由执行: SelfEvolutionLoop.propose_mutation()

进化路线图 (文档驱动)
  ├─ v16 Wave A: 生存接线 (830行) ← 进行中
  ├─ v16 Wave B: 经济代理 (2100行) ← 待启动
  └─ v16 Wave C: 元进化 (2100行) ← 远期

持续迭代 (本文件)
  ├─ 已关闭任务 → 经验树蒸馏
  ├─ 进行中任务 → 优先级排序
  └─ 待发现任务 → 自审计触发
```

## 进行中任务

### Phase 1 — SelfEvolutionEngine 接线 (✅ 已完成)
| 子任务 | 状态 | 文件 | 
|--------|------|------|
| EvolutionSignal + Engine struct | ✅ | self_evolution_engine.rs |
| ConsciousnessIntegration 接线 | ✅ | types.rs + core.rs + modules_core.rs |
| MetaArchEvoLoop 桥接 | ✅ | modules_core.rs:handle_evolution_engine_tick |
| Body 编译验证 | ✅ | 0 errors |

### Phase 2 — Body Bridge (⏳ 待启动)
| 子任务 | 估计行数 | 优先级 |
|--------|---------|--------|
| NetworkEvolution.tick() → cycle 50 组 | ~30 | P0 |
| PerceptionGateway.channel_stats() → engine signal | ~20 | P1 |
| Body → Core 跨 crate 通信 (tokio::mpsc) | ~50 | P1 |

### Phase 3 — 安全层激活 (⏳ 待启动)
| 子任务 | 估计行数 | 优先级 |
|--------|---------|--------|
| SelfModifyGuard Shield 层 basic_rules | ~50 | P0 |
| Safety rules: 禁止删除 core handler | ~20 | P0 |
| Safety rules: 禁止修改 safety_gate 自身 | ~20 | P0 |

### Wave 3 — 能力升级 (📋 已规划)
| Gap ID | 描述 | 优先级 |
|--------|------|--------|
| G310 | 示例学习 (Example Learning from extraction patterns) | P1 |
| G311 | RL提取 (DOM→VSA direct encoding) | P1 |
| G313 | 层次Agent (Hierarchical Agent orchestration) | P1 |
| G314 | 双系统推理 (Fast/Slow reasoning integration) | P1 |

## 每次会话的持续进化流程

```
1. 自审计 (2分钟)
   ├─ 检查 handle_consciousness_batch_sync 中是否有新死代码
   ├─ 检查 MetaArchEvoLoop.recommendations 是否有未执行项
   └─ 检查 body/NetworkEvolution 是否需要接线

2. 优先级排序 (1分钟)
   ├─ P0: 接线断裂 (EvolutionEngine new gaps)
   ├─ P1: 能力升级 (Wave 3 gaps)
   └─ P2: 经济 + 元进化

3. 执行 (本会话)
   ├─ 选最高优先级未完成任务
   ├─ 实现 + 测试
   └─ cargo check 验证

4. 蒸馏 (会话结束时)
   ├─ 更新经验树 (新分支)
   ├─ 更新 AGENTS.md 状态
   └─ 更新本文件
```

## 关键决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-06-23 | SelfEvolutionEngine 在 nt_core_experience/ | 统一进化编排层, 避免 cross-crate 依赖 |
| 2026-06-23 | EvolutionSignal 作为统一信号格式 | 3 信号源统一排序, 避免各自为政 |
| 2026-06-23 | chengfeng 被动进化 vs NeoTrix 主动进化 | 被动进化处理用户偏好, 主动进化处理能力缺口 |
