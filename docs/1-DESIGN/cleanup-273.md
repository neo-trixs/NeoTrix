# 跨域引用分析

**执行时间**: 2026-09-11  
**分析范围**: `neotrix-core/src/{l1_action,l2_perception,l3_embodiment,l4_emotion,l5_cognition,l6_meta}`  
**过滤条件**: 排除 test、facade 文件

## 跨域引用矩阵

| 引用方向 | 数量 |
|---------|------|
| l5_cognition→l1_action | 4 |
| l5_cognition→l2_perception | 1 |
| l5_cognition→l6_meta | 2 |

## 发现

**总跨域引用**: 7 处

### 按源域分析

| 源域 | 引用目标 | 数量 | 风险等级 |
|------|---------|------|---------|
| l5_cognition | l1_action | 4 | ⚠️ 中 |
| l5_cognition | l2_perception | 1 | ✅ 低 |
| l5_cognition | l6_meta | 2 | ✅ 低 |

### 详细分析

#### l5_cognition→l1_action (4处)
- **位置**: `nt_core`, `nt_mind` 模块
- **引用内容**: 可能引用 nt_act 的工具/动作定义
- **风险**: 认知层依赖行动层，违反分层原则
- **建议**: 需要确认是否为合理依赖（如 GWT 路由需要知道可用工具）

#### l5_cognition→l2_perception (1处)
- **位置**: `nt_core` 模块
- **引用内容**: 可能引用 nt_world/nt_sense 的感知数据
- **风险**: 认知层依赖感知层，违反分层原则
- **建议**: 需要确认是否为合理依赖（如 ConsciousnessTree 需要感知输入）

#### l5_cognition→l6_meta (2处)
- **位置**: `nt_core`, `nt_mind` 模块
- **引用内容**: 可能引用 nt_meta 的元认知协调
- **风险**: 认知层依赖元认知层，方向合理（L5→L6）
- **建议**: ✅ 这是合理的向上依赖，符合六层架构设计

## 架构合规性检查

### 分层依赖规则
- ✅ **L6 Meta-Cognition**: 可依赖所有层（元认知协调）
- ⚠️ **L5 Cognition**: 应仅依赖 L4 及以下，但当前引用了 L1/L2
- ✅ **L4 Emotion**: 无跨域引用
- ✅ **L3 Embodiment**: 无跨域引用
- ✅ **L2 Perception**: 无跨域引用
- ✅ **L1 Action**: 无跨域引用

### 违规项
1. **l5_cognition→l1_action** (4处): 认知层不应直接依赖行动层
2. **l5_cognition→l2_perception** (1处): 认知层不应直接依赖感知层

## 修复建议

### 方案 A: 通过 trait 抽象（推荐）
```rust
// 在 l5_cognition 中定义抽象 trait
pub trait ActionProvider {
    fn execute(&self, action: &str) -> Result<()>;
}

// l1_action 实现该 trait
impl ActionProvider for NtAct { ... }
```

### 方案 B: 通过事件总线解耦
```rust
// l5_cognition 发布事件
EventBus::publish(CognitionEvent::ActionRequested { ... });

// l1_action 订阅事件
EventBus::subscribe::<ActionRequested>(handle_action);
```

### 方案 C: 检查是否为误报
部分引用可能是：
- 类型定义（如共享枚举）
- 测试辅助代码（已被过滤但可能遗漏）
- facade 模块的重新导出

## 下一步行动

1. **详细审查**: 检查具体引用位置，确认是否为合理依赖
2. **应用修复**: 选择方案 A/B/C 进行修复
3. **重新验证**: 修复后重新运行跨域引用检查
4. **更新文档**: 将修复结果更新到本文件

---

**状态**: 待审查  
**负责人**: NT-CORE / NT-MIND