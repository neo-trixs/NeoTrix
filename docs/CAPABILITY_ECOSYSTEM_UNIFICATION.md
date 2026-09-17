# 能力生态统一分析

## 问题: 三套能力系统并存

```
┌─────────────────────────────────────────────────────────────────┐
│                    全域能力生态 (L5 Cognition)                   │
│  nt_core_capability/                                            │
│  ├── UnifiedCapability trait                                    │
│  ├── CapabilityRegistry                                        │
│  ├── CapabilityRouter                                          │
│  ├── discovery / factory / loadbalancer / monitor              │
│  └── versioning / security / performance                       │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼ (无连接)
┌─────────────────────────────────────────────────────────────────┐
│                    L1 Action 层基础设施                          │
│  nt_act/actions/core/                                          │
│  ├── action_cache (ActionCache)                                │
│  ├── circuit_breaker (CircuitBreaker)                          │
│  ├── eventbus (EventBus)                                       │
│  ├── rate_limiter (RateLimiter)                                │
│  └── workflow (Workflow)                                       │
│                                                                 │
│  nt_act/communication/  (新提取)                                │
│  nt_act/geo_seo/        (新提取)                                │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼ (无连接)
┌─────────────────────────────────────────────────────────────────┐
│                    Trade 模块能力                               │
│  nt_act_trade/                                                  │
│  ├── capability_registry.rs (TradeCapability trait)            │
│  ├── engine_traits.rs (TradeEngine trait)                      │
│  ├── workers/ (TaskWorkerType)                                 │
│  ├── rate_limiter.rs (RateLimiter)     ← 重复!                │
│  ├── circuit_breaker.rs (CircuitBreaker) ← 重复!              │
│  └── error.rs (TradeError)             ← 未连接全局错误        │
└─────────────────────────────────────────────────────────────────┘
```

## 发现的问题

### 1. 三套 Capability trait 无继承关系

| Trait | 位置 | 用途 |
|-------|------|------|
| `UnifiedCapability` | `nt_core_capability/mod.rs` | 全域能力注册 |
| `TradeCapability` | `nt_act_trade/capability_registry.rs` | 贸易能力注册 |
| (无) | `nt_act/actions/core/` | 基础设施能力 |

**问题**: `TradeCapability` 未继承 `UnifiedCapability`，两套注册表独立运行。

### 2. 基础设施重复实现

| 能力 | 全局/L1 | Trade | 重复 |
|------|---------|-------|------|
| RateLimiter | `nt_act/actions/core/nt_act_rate_limiter.rs` | `nt_act_trade/rate_limiter.rs` | ✅ 重复 |
| CircuitBreaker | `nt_act/actions/core/nt_act_circuit_breaker.rs` | `nt_act_trade/circuit_breaker.rs` | ✅ 重复 |
| EventBus | `nt_act/actions/core/nt_act_eventbus.rs` | `nt_act_trade/event_bus.rs` | ✅ 重复 |
| ActionCache | `nt_act/actions/core/nt_act_cache.rs` | 无 | ❌ Trade未复用 |

### 3. 跨层依赖不一致

```
nt_act_trade/full_cycle.rs       → use nt_core_capability_tree::...
nt_act_trade/finance_compliance.rs → use nt_core_capability_tree::...
nt_act_trade/production_logistics.rs → use nt_core_capability_tree::...
nt_act_trade/quote_negotiation.rs  → use nt_core_capability_tree::...

但:
nt_act_trade/capability_registry.rs → 独立实现 (未引用 nt_core_capability)
nt_act_trade/engine_traits.rs      → 独立实现 (未引用 nt_core_capability)
```

### 4. 缺失的连接

| 连接 | 状态 | 说明 |
|------|------|------|
| TradeCapability → UnifiedCapability | ❌ 未实现 | 两套 trait 无继承 |
| TradeCapabilityRegistry → CapabilityRegistry | ❌ 未实现 | 两个注册表独立 |
| TradeEngine → UnifiedCapability | ❌ 未实现 | Engine 未注册到全局 |
| RateLimiter (trade) → RateLimiter (L1) | ❌ 未实现 | 重复实现 |
| CircuitBreaker (trade) → CircuitBreaker (L1) | ❌ 未实现 | 重复实现 |
| TradeError → NeoTrixError | ❌ 未实现 | 错误类型未连接 |

## 统一方案

### 方案 A: 分层继承 (推荐)

```
UnifiedCapability (全局 trait)
    │
    ├── TradeCapability (extends UnifiedCapability)
    │       ├── PriceCalculatorCapability
    │       ├── ProductMatcherCapability
    │       └── ...
    │
    └── 其他域 Capability (extends UnifiedCapability)
            ├── ChatCapability
            └── ...
```

**修改**:
1. `TradeCapability` 继承 `UnifiedCapability`
2. `TradeCapabilityRegistry` 包装全局 `CapabilityRegistry`
3. Trade 的 RateLimiter/CircuitBurner 使用 L1 的实现
4. TradeError 实现 `From<NeoTrixError>`

### 方案 B: 统一注册表

```
CapabilityRegistry (全局唯一)
    │
    ├── 注册 UnifiedCapability (全局能力)
    ├── 注册 TradeCapability (贸易能力)
    └── 注册其他域能力
```

**修改**:
1. 删除 `TradeCapabilityRegistry`，改用全局 `CapabilityRegistry`
2. `TradeCapability` 实现 `UnifiedCapability` trait
3. 所有能力统一注册到全局注册表

## 推荐执行顺序

1. **P0**: 让 `TradeCapability` 继承 `UnifiedCapability`
2. **P1**: 删除 Trade 的重复 RateLimiter/CircuitBreaker，使用 L1 实现
3. **P2**: `TradeCapabilityRegistry` 包装全局 `CapabilityRegistry`
4. **P3**: `TradeError` 实现 `From<NeoTrixError>`
5. **P4**: 所有 Trade Engine 注册到全局 CapabilityRegistry

## 收益

| 收益 | 说明 |
|------|------|
| 消除重复 | RateLimiter/CircuitBurner 只有一份 |
| 统一发现 | 所有能力通过全局注册表发现 |
| 跨域能力组合 | 贸易+物流+金融可组合使用同一套能力 |
| 降低维护成本 | 基础设施只需维护一份 |
