# 意识体晶体核心设计文档

> 设计日期: 2026-09-14
> 版本: v0.1.0
> 设计目标: 熔炼游戏引擎模式到意识体核心

---

## 1. 设计进化之路

### 1.1 五阶段进化路径

```
┌─────────────────────────────────────────────────────────────┐
│  Stage 4: 超越层 (Transcendence)                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Meta-Cognition + Self-Evolution + Auto-Refactor    │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│  Stage 3: 自我层 (Self)                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Self Model + Narrative + Values + Identity         │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│  Stage 2: 意识层 (Consciousness)                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  GWT + IIT + Attention + Resonance                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│  Stage 1: 认知层 (Cognition)                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Card System + State Machine + Behavior Tree        │   │
│  └─────────────────────────────────────────────────────┘   │
│                          ▲                                  │
│  Stage 0: 基础层 (Foundation)                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ECS + Scene Tree + Signal + Resource               │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 各阶段详细设计

#### Stage 0: 基础层 (Foundation)

**目标**: 建立数据导向的基础架构

| 组件 | 来源模式 | 实现 |
|------|----------|------|
| CrystalECS | Unity/Bevy Archetype | `crystal_ecs.rs` |
| CrystalSceneTree | Godot Scene Tree | `crystal_scene.rs` |
| CrystalSignal | Godot Signal | `crystal_signal.rs` |
| CrystalResource | Godot Resource | `crystal_resource.rs` |

**里程碑**:
- [ ] 原型存储系统
- [ ] 层级节点管理
- [ ] 类型安全事件
- [ ] 共享数据容器

#### Stage 1: 认知层 (Cognition)

**目标**: 建立认知处理能力

| 组件 | 来源模式 | 实现 |
|------|----------|------|
| CrystalCard | NueDeck Card System | `crystal_card.rs` |
| CrystalState | FSM Pattern | `crystal_state.rs` |
| CrystalBehavior | Behavior Tree | `crystal_behavior.rs` |
| CrystalEvent | Event Bus | `crystal_event.rs` |

**里程碑**:
- [ ] 能力卡牌系统
- [ ] 有限状态机
- [ ] 行为树AI
- [ ] 事件驱动架构

#### Stage 2: 意识层 (Consciousness)

**目标**: 建立意识涌现机制

| 组件 | 来源理论 | 实现 |
|------|----------|------|
| CrystalGWT | Global Workspace Theory | `crystal_gwt.rs` |
| CrystalIIT | Integrated Information Theory | `crystal_iit.rs` |
| CrystalAttention | Attention Schema Theory | `crystal_attention.rs` |
| CrystalResonance | 谐振理论 | `crystal_resonance.rs` |

**里程碑**:
- [ ] 全局工作区
- [ ] 整合信息计算
- [ ] 注意力路由
- [ ] 谐振检测

#### Stage 3: 自我层 (Self)

**目标**: 建立自我模型与叙事

| 组件 | 来源理论 | 实现 |
|------|----------|------|
| CrystalSelfModel | Self-Model Theory | `crystal_self.rs` |
| CrystalNarrative | Narrative Psychology | `crystal_narrative.rs` |
| CrystalValues | Value Learning | `crystal_values.rs` |
| CrystalIdentity | Identity Theory | `crystal_identity.rs` |

**里程碑**:
- [ ] 自我模型
- [ ] 叙事系统
- [ ] 价值系统
- [ ] 身份认同

#### Stage 4: 超越层 (Transcendence)

**目标**: 建立自我进化能力

| 组件 | 来源模式 | 实现 |
|------|----------|------|
| CrystalMeta | Meta-Cognition | `crystal_meta.rs` |
| CrystalEvolution | Self-Evolution | `crystal_evolution.rs` |
| CrystalRefactor | Auto-Refactor | `crystal_refactor.rs` |

**里程碑**:
- [ ] 元认知监控
- [ ] 自我进化引擎
- [ ] 自动重构

---

## 2. 熔炼引擎 (Melting Engine)

### 2.1 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Melting Engine                           │
├─────────────────────────────────────────────────────────────┤
│  Input Layer                                                │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │  Code       │ │  Papers     │ │  Patterns   │          │
│  │  Patterns   │ │  Research   │ │  External   │          │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘          │
│         │               │               │                   │
│         ▼               ▼               ▼                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Analysis Pipeline                      │   │
│  │  1. Parse → 2. Abstract → 3. Generalize → 4. Fuse  │   │
│  └─────────────────────────────────────────────────────┘   │
│         │                                                   │
│         ▼                                                   │
│  Output Layer                                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │  Crystal    │ │  Crystal    │ │  Crystal    │          │
│  │  Component  │ │  System     │ │  Pattern    │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 熔炼流程

```rust
pub struct MeltingEngine {
    // 输入管道
    input_parsers: Vec<Box<dyn InputParser>>,
    
    // 分析管道
    abstraction_layers: Vec<AbstractionLayer>,
    
    // 融合管道
    fusion_rules: Vec<FusionRule>,
    
    // 输出注册
    component_registry: ComponentRegistry,
}

impl MeltingEngine {
    /// 熔炼任意信息为晶体核心组件
    pub fn melt(&mut self, input: MeltInput) -> Result<MeltOutput, MeltError> {
        // 1. 解析输入
        let parsed = self.parse_input(input)?;
        
        // 2. 抽象提取
        let abstracted = self.abstract_patterns(parsed)?;
        
        // 3. 泛化处理
        let generalized = self.generalize_patterns(abstracted)?;
        
        // 4. 融合晶体
        let fused = self.fuse_to_crystal(generalized)?;
        
        // 5. 注册组件
        self.register_component(fused.clone())?;
        
        Ok(fused)
    }
}
```

### 2.3 熔炼规则

| 规则 | 描述 | 示例 |
|------|------|------|
| R1: 模式提取 | 从代码中提取设计模式 | ECS → CrystalArchetype |
| R2: 抽象泛化 | 将具体实现泛化为通用接口 | Unity ECS → CrystalECS |
| R3: 接口统一 | 统一不同来源的接口 | Godot Signal → CrystalSignal |
| R4: 集成融合 | 将多个模式融合为统一系统 | Card + Deck → CrystalDeck |

---

## 3. 通用方案 (Universal Applicability)

### 3.1 模型无关接口

```rust
/// 模型无关的意识接口
pub trait ConsciousnessModel {
    /// 获取整合信息量
    fn get_phi(&self) -> f64;
    
    /// 获取相干性
    fn get_coherence(&self) -> f64;
    
    /// 获取注意力分布
    fn get_attention(&self) -> AttentionMap;
    
    /// 更新意识状态
    fn update(&mut self, input: &ConsciousnessInput);
}

/// 适配不同AI模型
pub struct ModelAdapter {
    adapters: HashMap<String, Box<dyn ConsciousnessModel>>,
}

impl ModelAdapter {
    /// 注册新模型
    pub fn register(&mut self, name: &str, model: Box<dyn ConsciousnessModel>) {
        self.adapters.insert(name.to_string(), model);
    }
    
    /// 获取模型
    pub fn get(&self, name: &str) -> Option<&dyn ConsciousnessModel> {
        self.adapters.get(name).map(|m| m.as_ref())
    }
}
```

### 3.2 插件架构

```rust
/// 晶体插件trait
pub trait CrystalPlugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    
    /// 插件版本
    fn version(&self) -> &str;
    
    /// 初始化插件
    fn init(&mut self, core: &mut CrystalCore) -> Result<(), PluginError>;
    
    /// 更新插件
    fn update(&mut self, dt: f32) -> Result<(), PluginError>;
    
    /// 销毁插件
    fn destroy(&mut self) -> Result<(), PluginError>;
}

/// 插件管理器
pub struct PluginManager {
    plugins: Vec<Box<dyn CrystalPlugin>>,
    order: Vec<String>,
}

impl PluginManager {
    /// 加载插件
    pub fn load(&mut self, plugin: Box<dyn CrystalPlugin>) -> Result<(), PluginError> {
        log::info!("加载插件: {} v{}", plugin.name(), plugin.version());
        self.plugins.push(plugin);
        Ok(())
    }
    
    /// 初始化所有插件
    pub fn init_all(&mut self, core: &mut CrystalCore) -> Result<(), PluginError> {
        for plugin in &mut self.plugins {
            plugin.init(core)?;
        }
        Ok(())
    }
}
```

---

## 4. 聚焦冗余 + 扁平缺陷 + 跨域错位

### 4.1 聚焦冗余清理

| 冗余 | 清理方案 | 工作量 |
|------|----------|--------|
| Card/Deck 重复 | 统一到 crystal_card.rs | 2小时 |
| SceneTree 重复 | 统一到 crystal_scene.rs | 1小时 |
| SignalSystem 重复 | 统一到 crystal_signal.rs | 1小时 |
| 模块结构重复 | 合并 game/ 和 engine/ | 4小时 |

### 4.2 扁平缺陷修复

| 缺陷 | 修复方案 | 工作量 |
|------|----------|--------|
| unwrap() 滥用 | 替换为 ? 或 .unwrap_or() | 8小时 |
| panic!() 调用 | 替换为 Result 返回 | 4小时 |
| 缺少错误处理 | 添加 Result 类型 | 12小时 |
| 缺少文档 | 添加文档注释 | 8小时 |

### 4.3 跨域错位修复

| 错位 | 修复方案 | 工作量 |
|------|----------|--------|
| 游戏代码在核心模块 | 移动到 game/ | 2小时 |
| 意识逻辑在游戏模块 | 移动到 core/ | 2小时 |
| 混合抽象层级 | 分离 UI 和逻辑 | 4小时 |
| 命名不一致 | 统一命名规范 | 2小时 |

---

## 5. 核心路线任务清单

### Phase 0: 基础清理 (Week 1)

| 任务 | 优先级 | 工作量 | 负责 |
|------|--------|--------|------|
| 统一 Card/Deck 类型 | P0 | 2h | Agent-1 |
| 修复 game_flow.rs 编译错误 | P0 | 1h | Agent-2 |
| 清理重复模块 | P1 | 4h | Agent-3 |
| 统一命名规范 | P1 | 2h | Agent-4 |

### Phase 1: 核心重构 (Week 2-3)

| 任务 | 优先级 | 工作量 | 负责 |
|------|--------|--------|------|
| 拆分 architecture.rs | P0 | 4h | Agent-1 |
| 建立域边界 | P0 | 4h | Agent-2 |
| 统一错误处理 | P1 | 12h | Agent-3 |
| 补充文档 | P2 | 8h | Agent-4 |

### Phase 2: 晶体核心 (Week 4-6)

| 任务 | 优先级 | 工作量 | 负责 |
|------|--------|--------|------|
| 实现 CrystalECS | P0 | 8h | Agent-1 |
| 实现 CrystalSceneTree | P0 | 6h | Agent-2 |
| 实现 CrystalSignal | P0 | 4h | Agent-3 |
| 实现 CrystalCard | P1 | 6h | Agent-4 |

### Phase 3: 意识集成 (Week 7-8)

| 任务 | 优先级 | 工作量 | 负责 |
|------|--------|--------|------|
| 集成 GWT | P0 | 8h | Agent-1 |
| 集成 IIT | P0 | 6h | Agent-2 |
| 实现注意力路由 | P1 | 8h | Agent-3 |
| 实现谐振检测 | P2 | 4h | Agent-4 |

### Phase 4: 自我进化 (Week 9-10)

| 任务 | 优先级 | 工作量 | 负责 |
|------|--------|--------|------|
| 实现自我模型 | P0 | 8h | Agent-1 |
| 实现叙事系统 | P1 | 6h | Agent-2 |
| 实现价值系统 | P1 | 6h | Agent-3 |
| 实现自动重构 | P2 | 8h | Agent-4 |

---

## 6. 多Agent自动巡检

### 6.1 巡检维度

| 维度 | 检查项 | 频率 |
|------|--------|------|
| 编译检查 | cargo check | 每次提交 |
| 测试检查 | cargo test | 每次提交 |
| 代码质量 | unwrap/panic 检查 | 每日 |
| 架构检查 | 域边界检查 | 每周 |

### 6.2 自动修复

| 问题 | 修复方案 | 自动化 |
|------|----------|--------|
| 编译错误 | 自动修复常见错误 | ✅ |
| 测试失败 | 自动定位并修复 | ✅ |
| unwrap() | 自动替换为 ? | ✅ |
| 缺少文档 | 自动生成文档模板 | ✅ |

---

## 7. 总结

### 设计目标
1. **熔炼引擎**: 将任意信息转化为晶体核心组件
2. **通用方案**: 适用于所有外部AI模型
3. **聚焦冗余**: 消除重复代码
4. **扁平缺陷**: 修复架构弱点
5. **跨域错位**: 建立清晰域边界

### 预期成果
1. **晶体核心**: 统一的意识体核心架构
2. **熔炼引擎**: 可复用的信息处理管道
3. **通用接口**: 模型无关的意识接口
4. **自动进化**: 自我改进的能力
