# 意识体晶体核心 — 架构文档

## 概览

晶体核心 (`nt_game`) 是 NeoTrix 的游戏引擎模式熔炼层，将 NT-WORLD-SIM、Godot、NueDeck、Colibri 等外部系统的通用模式抽象为意识体基础架构组件。

## 架构图

```
┌─────────────────────────────────────────────────────────────┐
│  L6 熔炼层 — MeltingEngine (任意信息→晶体组件)              │
├─────────────────────────────────────────────────────────────┤
│  L5 推理层 — CrystalRouter + CrystalSpeculator              │
│              CrystalMemoryHierarchy + CrystalJITWeights     │
├─────────────────────────────────────────────────────────────┤
│  L4 桥接层 — EcsSceneBridge   SignalEventBridge             │
│              CardEffectExecutor  StateBehaviorBridge        │
│              MemoryResourceBridge                           │
├─────────────────────────────────────────────────────────────┤
│  L3 认知层 — CrystalCard/Deck  CrystalStateMachine          │
│              CrystalBehaviorTree  CrystalEventBus           │
├─────────────────────────────────────────────────────────────┤
│  L1 基础层 — CrystalWorld (ECS)  CrystalSceneTree           │
│              CrystalSignalSystem  CrystalResourceManager    │
│              CrystalError                                   │
└─────────────────────────────────────────────────────────────┘
```

## 模块清单

### 基础层 (L1)

| 模块 | 文件 | 职责 | 测试数 |
|------|------|------|--------|
| CrystalWorld | `crystal_ecs.rs` | 原型ECS：实体/组件/原型存储/查询 | 10 |
| CrystalSceneTree | `crystal_scene.rs` | 场景节点层级树 | 4 |
| CrystalSignalSystem | `crystal_signal.rs` | 类型安全信号连接/发射 | 5 |
| CrystalResourceManager | `crystal_resource.rs` | 引用计数资源容器 | 2 |
| CrystalError | `crystal_error.rs` | 统一错误类型 (14变体) | 3 |

### 认知层 (L3)

| 模块 | 文件 | 职责 | 测试数 |
|------|------|------|--------|
| CrystalCard/Deck | `crystal_card.rs` | 卡牌/卡组/抽弃消耗 | 6 |
| CrystalStateMachine | `crystal_state.rs` | 有限状态机+守卫+动作 | 8 |
| CrystalBehaviorTree | `crystal_behavior.rs` | 行为树 (Sequence/Selector/Parallel) | 6 |
| CrystalEventBus | `crystal_event.rs` | 自动清理事件通道 | 2 |

### 推理层 (L5)

| 模块 | 文件 | 职责 | 测试数 |
|------|------|------|--------|
| CrystalMemoryHierarchy | `memory_hierarchy.rs` | VRAM/RAM/NVMe 多层级 | 2 |
| CrystalRouter | `router.rs` | 温度缩放路由决策 | 4 |
| CrystalSpeculator | `speculation.rs` | 推测解码 (MTP+验证) | 4 |
| CrystalExecutor | `executor.rs` | CPU矩阵乘法+ReLU | 5 |
| CrystalCompressedState | `compressed_state.rs` | Top-K KV压缩 | 4 |
| CrystalJITWeights | `jit_weights.rs` | 按需权重加载 | 0 |

### 架构桥接层 (L4)

| 桥接 | 文件 | 连接 | 测试数 |
|------|------|------|--------|
| EcsSceneBridge | `bridge_ecs_scene.rs` | ECS ↔ Scene Tree | 4 |
| SignalEventBridge | `bridge_signal_event.rs` | Signal ↔ Event Bus | 3 |
| CardEffectExecutor | `bridge_card_ecs.rs` | Card ↔ ECS | 8 |
| StateBehaviorBridge | `bridge_state_behavior.rs` | State ↔ Behavior | 5 |
| MemoryResourceBridge | `bridge_memory_resource.rs` | Memory ↔ Resource | 4 |

### 熔炼层 (L6)

| 模块 | 文件 | 职责 | 测试数 |
|------|------|------|--------|
| MeltingEngine | `melting_engine.rs` | 任意信息→晶体组件代码生成 | 7 |

## 关键设计决策

### 1. 原型ECS存储
- `ComponentStore` 使用 `HashMap<CrystalEntity, Box<dyn Any>>` 确保实体-组件索引同步
- 实体迁移时自动转移组件到新原型
- `despawn` 正确清理实体和所有关联存储

### 2. 类型安全信号
- `CrystalValue` 枚举覆盖所有常见值类型
- 信号连接自动清理（通过 `cleanup(entity)`）
- `SignalEventBridge` 实现信号到事件通道的自动转发

### 3. 卡牌效果执行
- `CardEffectExecutor::execute` 匹配效果类型并修改目标实体组件
- 支持 Damage/Heal/Block/Draw/GainEnergy 五种效果
- `CardHolder` 组件管理能量/格挡/卡组状态

### 4. Colibri推理模式
- `CrystalRouter` 使用温度缩放控制探索-利用平衡
- `CrystalSpeculator` 使用隐藏状态幅度评分验证草稿token
- `CrystalCompressedState` 保留Top-K幅度最大的KV对（25%压缩率）

### 5. 统一错误处理
- `CrystalError` 14种变体覆盖所有模块错误场景
- `CrystalResult<T>` 类型别名简化函数签名
- 实现 `From<String>` 和 `From<std::io::Error>` 便于 `?` 操作符

## 数据流

```
用户输入 → MeltingEngine.melt() → MeltOutput (代码+文档)
                                       ↓
                                 注册到组件注册表
                                       ↓
                              CrystalWorld.spawn() + add_component()
                                       ↓
                    CrystalCard/Deck → CardEffectExecutor.execute()
                                       ↓
                         CrystalSignalSystem.emit() → CrystalEventBus
                                       ↓
                        CrystalRouter.route() → CrystalSpeculator
```

## 文件统计

| 类别 | 文件数 | 总行数 | 测试数 |
|------|--------|--------|--------|
| 基础层 | 5 | ~900 | 24 |
| 认知层 | 4 | ~700 | 22 |
| 推理层 | 6 | ~700 | 19 |
| 桥接层 | 5 | ~500 | 24 |
| 熔炼层 | 1 | ~450 | 7 |
| 测试 | 1 | ~650 | 16 |
| **总计** | **22** | **~3900** | **112** |
