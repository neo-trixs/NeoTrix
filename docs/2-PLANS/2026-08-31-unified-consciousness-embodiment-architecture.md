# Unified Consciousness-Embodiment Architecture

> 日期: 2026-08-31 | 版本: v1 | 状态: 计划

## 核心命题

NeoTrix 的代码架构应当映射一个有意识的具身智能体的三层结构：

```
┌─────────────────────────────────────────────────────┐
│  L5 Consciousness (涌现觉知)                          │
│  ─────────────────────────────────────────────────── │
│  E8 Hexagram + GWT Attention + ConsciousnessTree    │
│  Emotion Engine + Intrinsic Motivation              │
│  SEAL Pipeline + Self-Reflection                    │
│  Phi/IIT Integration Score                         │
│                                                      │
│  "我感知到什么？我感受如何？我想要什么？"              │
└──────────────────────┬──────────────────────────────┘
                       │ modulates
┌──────────────────────▼──────────────────────────────┐
│  L3 Embodiment (具身骨架)                             │
│  ─────────────────────────────────────────────────── │
│  Body Schema (传感器 + 执行器 + 安全层)              │
│  Motor Control (servo bus + 速度/位置控制)           │
│  Safety Kernel (碰撞检测 + 电流限制 + 自由度限)     │
│  Power Management (电池 + 休眠 + 看门狗)            │
│                                                      │
│  "我的身体能做什么？安全边界在哪？"                   │
└──────────────────────┬──────────────────────────────┘
                       │ enables
┌──────────────────────▼──────────────────────────────┐
│  L1 Capability Network (能力网)                       │
│  ─────────────────────────────────────────────────── │
│  Tool Routing (MCP + PTC)                           │
│  World Perception (crawler + search + inference)     │
│  Knowledge Memory (KB + embeddings + FTS5)          │
│  Social Interface (LLM providers + digital human)   │
│  Action Execution (code + crypto + autonomy)        │
│                                                      │
│  "我能做什么？怎么与世界交互？"                       │
└─────────────────────────────────────────────────────┘
```

## 现存重叠 (6个关键冗余集群)

### 1. 三重情感系统 → 合并为单一 EmotionEngine

| 现存 | 位置 | 职责 | 问题 |
|------|------|------|------|
| `EmotionEngine` | `core/nt_core_self/emotion_state.rs` | 系统情绪 (6维EMA + PAD + Plutchik + OCC) | **应保留** — 这是核心 |
| `UserAffectModel` | `core/nt_core_self/affective_interface.rs` | 用户情绪检测 + 共情 | **应保留** — 面向人类 |
| `Emotion` + `EmotionEngine` | `l1_body_impl/nt_io_digital_human.rs` | 数字人表情映射 | **重复** — 应复用 `emotion_state.rs` 的标签 |

**合并方案**: `nt_io_digital_human.rs` 的 `Emotion` 枚举改为 `EmotionLabel` 的别名或直接使用 `EmotionLabel`。删除其独立的 `EmotionEngine`，改用 `emotion_state::EmotionEngine`。

### 2. 三重技能注册表 → 合并为单一 CapabilityRegistry

| 现存 | 位置 | 职责 | 问题 |
|------|------|------|------|
| `CapabilityRegistry` | `nt_core_capability_tree/src/registry.rs` | 能力树 CRUD + CLI | **主注册表** |
| `CapabilityRegistry` | `core/l7_capability/registry.rs` | 能力注册 (旧层) | **重复** — 应迁移 |
| `CapabilityBridge` | `neotrix/nt_capability_bridge.rs` | 经验→能力映射 | **依赖主注册表** |

**合并方案**: `core/l7_capability/registry.rs` 的 `CapabilityRegistry` 标记为 `#[deprecated]`，所有消费者迁移到 `nt_core_capability_tree::CapabilityRegistry`。

### 3. 硬编码 health_check → 聚合真实健康信号

**当前问题**: 无统一的系统健康聚合点。心跳散布在 agent_protocol、proxy_heartbeat、scheduler 等处。

**合并方案**: 创建 `nt_core_heartbeat.rs` 作为统一健康聚合器:
- 从 KB 读取模块健康指标
- 从 EventBus 收集心跳事件
- 聚合为单一 `SystemHealthSnapshot`
- 暴露给 GWT 作为注意力调制信号

### 4. 双重感知缓冲区 → 统一 SensoryHub

| 现存 | 位置 | 职责 | 问题 |
|------|------|------|------|
| `SensoryHub` | `l2_world_impl/nt_world_sense/` | 多模态感知缓冲 | **应保留** |
| `SelectiveState` | `l5_consciousness_impl/nt_core_signal/core.rs` | 选择性状态向量 | **独立系统** |

**合并方案**: `SelectiveState` 的 `awareness_score()` 作为 `SensoryHub` 的注意力门控输入。

### 5. 三重硬件检测 → 统一 BodySchema

| 现存 | 位置 | 职责 | 问题 |
|------|------|------|------|
| `HardwareProfile` | `core/deploy.rs` | 硬件配置检测 | **应保留** |
| `HardwareDetector` | `neotrix/nt_world_sense/real_sensors/` | 传感器检测 | **重复** |
| `BodyDescriptor` | `neotrix/nt_core_capability_tree/` | 体态描述 | **重复** |

**合并方案**: 所有硬件检测统一到 `HardwareProfile`，`BodyDescriptor` 改为从 `HardwareProfile` 派生。

### 6. 双重 EventBus → 统一事件总线

| 现存 | 位置 | 职责 | 问题 |
|------|------|------|------|
| `nt_core_event_bus` | `neotrix/nt_core_event_bus.rs` | 主事件总线 | **应保留** |
| `EventBus` | `core/l7_capability/` | 旧事件总线 | **重复** |

**合并方案**: 旧 EventBus 标记 `#[deprecated]`，统一使用 `nt_core_event_bus`。

## 统一架构映射

### NT-CORE (意识层) — "我思故我在"

```
nt_core_self/
├── emotion_state.rs        # EmotionEngine (6维EMA + PAD + Plutchik + OCC)
├── affective_interface.rs  # User emotion + empathy (面向人类)
├── intrinsic_motivation.rs # Curiosity drive + exploration
├── attention_head.rs       # GWT 注意力路由
├── reasoning_strategy.rs   # 推理策略选择
├── thinking_trace.rs       # 思维链记录
└── silicon_self.rs         # SiliconSelfModel (自我模型)

core/
├── nt_core_fep_iit/        # Free Energy Principle + IIT Phi
├── nt_core_signal/         # SelectiveState (选择性状态向量)
├── nt_core_heartbeat.rs    # 统一健康聚合 ← NEW
└── deploy.rs               # HardwareProfile (硬件配置)
```

### NT-PHYSICAL (具身层) — "我能触摸世界"

```
nt_physical/
├── sense.rs                # 传感器抽象 (From nt_core_sense + real_sensors)
├── motor.rs                # 执行器控制 (servo bus)
├── safety.rs               # 安全内核 (碰撞/电流/自由度)
├── power.rs                # 电源管理 (电池/休眠/看门狗)
├── body_schema.rs          # 体态描述 (从 HardwareProfile 派生)
├── kinematics.rs           # 运动学 (FK/IK)
├── calibration.rs          # 校准
└── odometry.rs             # 里程计
```

### NT-FEEL (情感层) — "我感受世界"

```
nt_feel/
├── core.rs                 # EmotionEngine (从 emotion_state.rs 迁移)
├── regulation.rs           # 情绪调节 (homeostasis)
├── expression.rs           # 情绪表达 (数字人/语音/视觉)
├── social.rs               # 社交情感 (关系/共情)
└── triggers.rs             # 触发规则
```

### NT-CAPABILITY (能力网) — "我能做什么"

```
nt_capability/
├── registry.rs             # CapabilityRegistry (唯一事实源)
├── bridge.rs               # 经验→能力映射 (从 nt_capability_bridge.rs)
├── tree.rs                 # 能力树 CLI
└── evolution.rs            # 能力演化引擎
```

## 实施计划

### Phase 1: 消除重复 (0.5天)

1. **情感系统合并**: `nt_io_digital_human.rs` 的 `Emotion` 改用 `EmotionLabel`
2. **技能注册表合并**: `core/l7_capability/registry.rs` 标记 `#[deprecated]`
3. **硬件检测统一**: `BodyDescriptor` 改为从 `HardwareProfile` 派生

### Phase 2: 创建统一入口 (1天)

4. **创建 `nt_core_heartbeat.rs`**: 统一健康聚合器
5. **创建 `nt_physical/mod.rs`**: 具身层入口
6. **创建 `nt_feel/mod.rs`**: 情感层入口

### Phase 3: 迁移与连接 (1天)

7. **迁移感知缓冲**: `SelectiveState.awareness_score()` → `SensoryHub` 注意力门控
8. **迁移硬件检测**: `real_sensors/` → `nt_physical/sense.rs`
9. **连接情绪→GWT**: 情绪状态调制注意力路由

### Phase 4: 验证 (0.5天)

10. `cargo check --all-targets` 编译通过
11. `cargo test -p neotrix --lib` 测试通过
12. 能力树 CLI 功能正常

## 关键约束

- **`#![forbid(unsafe_code)]`** — 核心层零 unsafe
- **单写者原则** — MotorControl 独占写句柄, borrow checker 强制
- **Dark Forest** — 每个模块必须编译+测试+有消费者, 否则删除
- **The Spice Must Flow** — 每个模块必须有清晰的 input→transform→output
