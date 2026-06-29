# SelfEvolutionEngine — 意识体自进化引擎

## 问题诊断

~33K 行进化代码，~93% 死代码。6 个独立进化子系统互不相通：

| 子系统 | 位置 | 行数 | 状态 |
|--------|------|------|------|
| SelfEvolutionLoop | core/experience | 3,382 | ✅ 每50cycle运行，提议→执行→验证闭环 |
| MetaArchitectureEvolutionLoop | core/consciousness | 343 | ⚠️ assess() 每10cycle运行，推荐永远不执行 |
| MetaCognitiveLoop | core/meta | 781 | ⚠️ 运行但输出不驱动进化 |
| EvolutionCoordinator | core/experience | 826 | 💀 死代码 |
| NetworkEvolution | body/agent | 481 | 💀 死代码 |
| PerceptionGateway | body/agent | 319 | 💀 死代码 |
| MetaEvolutionController | mind/evolution | 538 | 💀 死代码 |
| SelfEvolutionTaskEngine | core/experience | 307 | 💀 死代码 |
| SelfEvolutionMetaLayer | core/experience | 1,235 | 💀 死代码 |
| SEAL self_iterating/ | core/neotrix | ~14,000 | 💀 死代码 |

## 架构设计

### 核心原则

1. **不写新代码，只接电线** — 所有进化原语已存在，只需 ~500 行编排代码
2. **信号统一** — 所有子系统输出归一化为 `EvolutionSignal` 枚举
3. **优先级排序** — signal.impact × signal.urgency × signal.feasibility 决定执行顺序
4. **安全门控** — 所有执行经过 SelfModifyGuard (4层，逐步激活)
5. **闭环反馈** — 每次执行结果回写到 KPI，驱动下轮进化

### 架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                      ConsciousnessIntegration                        │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                  SelfEvolutionEngine                          │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌──────┐  │   │
│  │  │SignalHub   │→ │Prioritizer │→ │SafeExecutor│→ │Distil│  │   │
│  │  │            │  │            │  │            │  │ler   │  │   │
│  │  │• MetaAssess│  │• impact×   │  │• SelfModify│  │• Skill│  │   │
│  │  │• MetaCog   │  │  urgency   │  │  Guard(4层)│  │• Step │  │   │
│  │  │• SelfEvo   │  │• feasibility│ │• BallVerify│  │• Trace│  │   │
│  │  │• BodyStats │  │• conflict   │ │• Rollback │  │       │  │   │
│  │  └────────────┘  └────────────┘  └────────────┘  └──────┘  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│           │              │              │                           │
│           ▼              ▼              ▼                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐                    │
│  │ MetaArch   │  │ SelfEvo   │  │ BodyBridge │ ← new: 50行        │
│  │ EvoLoop    │  │ Loop(活)  │  │ (新)       │                    │
│  │ (assess)   │  │           │  │ → Network  │                    │
│  │            │  │ propose→  │  │   Evolution │                    │
│  │ 推荐→执行  │  │ verify→   │  │ → Perception│                    │
│  │  (断)      │  │ deploy    │  │   Gateway   │                    │
│  └────────────┘  └────────────┘  └────────────┘                    │
└─────────────────────────────────────────────────────────────────────┘
```

### 关键接线点

| 连接 | 当前状态 | 修复方式 | 行数 |
|------|----------|----------|------|
| MetaArchEvoLoop.assess() → SelfEvolutionEngine | 推荐在 IntegratedResult 中丢弃 | SignalHub 消费 assess() 输出 | ~30 |
| MetaCogLoop → EvolutionSignal | 无 | 添加 gap→signal 桥接 | ~30 |
| SelfEvolutionEngine → Body | 无 | BodyBridge 通过 socket/tokio::mpsc 通信 | ~50 |
| Body NetworkEvolution → SelfEvolutionEngine | 无 | 添加 cycle_count 接入点 | ~30 |
| SelfEvolutionEngine → SelfModifyGuard | 4层全None | 激活 Shield 层 (safety_rules) | ~50 |
| SelfEvolutionEngine → SelfEvolutionTaskEngine | 死代码 | 作为 execute() 的路由目标 | ~40 |
| EvolutionCoordinator → 废弃 | 死代码 | 标记 #[deprecated]，逻辑并入 Engine | ~10 |
| SelfEvolutionMetaLayer → 废弃 | 死代码 | 标记 #[deprecated]，逻辑并入 Engine | ~10 |

**总计新增**: ~250 行编排代码

### 信号优先级公式

```
priority = impact × urgency × feasibility

impact ∈ [0,1]: 影响范围 (handler数 / total_handlers)
urgency ∈ [0,1]: 降级程度 (1 - current_health / baseline_health)
feasibility ∈ [0,1]: 成功率估计 (历史同类 mutation 成功率)

threshold = 0.3  # 低于此值跳过当前 cycle
```

### 安全门控激活路径

```
SelfModifyGuard.evaluate()
  ├─ Layer 1: Shield (safety_rules)  ← 立即激活: 禁止删除核心模块
  ├─ Layer 2: Swords (约束检查)      ← Wave 1: 激活 basic_rules
  ├─ Layer 3: LLM Validator          ← Wave 2: 需 LLM API key
  └─ Layer 4: Ball Verifier          ← 已激活 (每50cycle运行)
```

## 实现计划

### Phase 1 — 核心接线 (~150 行，本会话可完成)

1. **types.rs**: +1 import `EvolutionSignal`, +3 字段 (signal_hub, body_bridge, mod_guard_enabled)
2. **core.rs**: 在 cycle 50 tick 组添加 `handle_evolution_engine_tick()` 替换 `handle_self_evolution_tick()`
3. **modules_core.rs**: 实现 `SelfEvolutionEngine` 的 signal_hub → prioritize → safe_execute → distill 循环
4. **self_evolution_loop/core.rs**: 添加 `MetaArchEvoLoop.assess()` 消费到 SelfEvolutionLoop 的桥接

### Phase 2 — Body 桥接 (~80 行)

5. **Agent集成**: body Bridge 通过循环计数接入 NetworkEvolution.tick()
6. **PerceptionGateway**: 在 bridge tick 中注册 channel stats

### Phase 3 — 安全层激活 (~50 行)

7. **SelfModifyGuard**: 激活 Shield 层 basic_rules
8. **Safety规则**: 禁止删除 core handler，禁止修改 safety_gate 自身

## 成功指标

| 指标 | Phase 1 | Phase 2 | Phase 3 |
|------|---------|---------|---------|
| 闭环反馈回路 | 1→4 (MetaAssess→执行) | +2 (Body→自进化) | 全部6条回路闭合 |
| 死代码激活 | ~2K 行 | ~1K 行 | ~500 行 |
| 新代码 | +150 行 | +80 行 | +50 行 |
| Safety layer | 1/4 | 1/4 | 2/4 |
