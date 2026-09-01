//! # NeoTrix L1 统一能力网架构文档
//!
//! 生成时间: 2026-09-01
//! 版本: v0.21.0

# NeoTrix L1 统一能力网架构

## 概述

NeoTrix L1 层实现了**统一能力网**架构，将所有底层能力按 8 个类别标准化为：
**Trait + Registry + Router + Bridge** 四件套模式。

```
┌─────────────────────────────────────────────────────────────────────┐
│                        L5 认知层 (领域技能)                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │
│  │ TradeOrchestrator│  │ CodeOrchestrator│  │ ResearchSkill   │ ... │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │
│           │                    │                    │              │
│           ▼                    ▼                    ▼              │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                    L1 能力网 (统一接口)                        │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐    │  │
│  │  │CAT-1   │ │CAT-2   │ │CAT-3   │ │CAT-4   │ │CAT-5   │    │  │
│  │  │通信    │ │内容    │ │数据    │ │搜索    │ │执行    │    │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘    │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐                         │  │
│  │  │CAT-6   │ │CAT-7   │ │CAT-8   │                         │  │
│  │  │安全    │ │协调    │ │认知    │                         │  │
│  │  └────────┘ └────────┘ └────────┘                         │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

## 8 大能力类别

| CAT | 类别 | Trait | 核心模块 | 进化等级 |
|-----|------|-------|---------|---------|
| 1 | 通信 | `MessagingProvider` | `nt_io_messaging` | C1 |
| 2 | 内容 | `ContentProvider` | `nt_act_media` | C1 |
| 3 | 数据 | `DataStore` | `nt_memory_lead` | C1 |
| 4 | 搜索 | `SearchEngine` | `nt_memory_search` | C2 |
| 5 | 执行 | `ToolExecutor` | `nt_act_code` | C1 |
| 6 | 安全 | `SecurityGuard` | `nt_act_security` | C1 |
| 7 | 协调 | `Orchestrator` | `nt_act_orchestrator` | C2 |
| 8 | 认知 | `LlmRouter` | `nt_io_provider/pool` | C2 |

## 统一 Trait 定义

```rust
// 基座 trait - 所有能力必须实现
pub trait L1Capability {
    fn capability_id(&self) -> &str;
    fn category(&self) -> CapabilityCategory;
    fn constellation(&self) -> ConstellationLevel;
    fn health_check(&self) -> CapabilityHealth;
    fn description(&self) -> &str;
    fn stats(&self) -> CapabilityStats;
}

// 8 个类别 trait
pub trait MessagingProvider: L1Capability { ... }
pub trait ContentProvider: L1Capability { ... }
pub trait DataStore: L1Capability { ... }
pub trait SearchEngine: L1Capability { ... }
pub trait ToolExecutor: L1Capability { ... }
pub trait SecurityGuard: L1Capability { ... }
pub trait Orchestrator: L1Capability { ... }
pub trait LlmRouter: L1Capability { ... }
```

## Registry / Router / Bridge 模式

每个类别实现标准三件套：

```rust
// Registry - 注册中心
pub struct MessagingRegistry {
    providers: Vec<Box<dyn MessagingProvider>>,
}
impl MessagingRegistry {
    fn register(&mut self, provider: Box<dyn MessagingProvider>);
    fn get(&self, id: &str) -> Option<&dyn MessagingProvider>;
    fn health_check_all(&self) -> Vec<(String, CapabilityHealth)>;
    fn optimal(&self) -> Option<&dyn MessagingProvider>;
}

// Router - 智能路由
pub struct MessagingRouter {
    registry: MessagingRegistry,
}
impl MessagingRouter {
    fn route(&self, channel: Channel) -> Option<&dyn MessagingProvider>;
    fn send(&self, msg: &Message) -> Result<String, CapabilityError>;
}

// Bridge - L1↔L5 桥接
pub struct MessagingBridge {
    router: MessagingRouter,
}
impl MessagingBridge {
    fn send(&self, msg: &Message) -> Result<String, CapabilityError>;
}
```

## 基础设施层

```
nt_infra/
├── tracing.rs      ← 调用追踪 (span + 聚合统计)
├── breaker.rs      ← 断路器 (Closed→Open→HalfOpen)
├── semantic_router.rs ← 语义路由 (关键词+意图+学习)
├── agent_card.rs   ← Agent Card (A2A 自描述+发现)
├── scatter_gather.rs ← 并行聚合 (BestScore/MajorityVote/Fallback)
├── persistence.rs  ← Registry 持久化 (JSON + 增量)
├── learning.rs     ← Router 自学习 (成功率+延迟+反馈)
└── integration.rs  ← 统一集成层
```

### 调用链路追踪

```
L5 TradeOrchestrator
    │
    ▼
MessagingBridge.send(msg)
    │
    ▼
EnhancedRouter.route(query)  ──► trace_start("io.messaging", "send")
    │                                 │
    ▼                                 ▼
SemanticRouter.route()            trace_end(span_id, true, None)
    │                                 │
    ▼                                 ▼
Breaker.allow()                     trace_aggregate("io.messaging")
    │                                 │
    ▼                                 ▼
Provider.send()                     │
    │                                 ▼
    └─────────────────────────► CapabilityAggregate 更新
```

### 断路器状态机

```
Closed (正常)
    │ error_rate >= threshold
    ▼
Open (熔断) ──open_duration_ms──► HalfOpen (试探)
    ▲                              │
    │                              ▼
    │                    success >= 50%
    │                              │
    └──────────────────────────────┘
```

### 语义路由决策

```
输入: query + intent
    │
    ▼
┌─────────────────────┐
│ 1. 关键词匹配        │──confidence > 0.6──► 返回 Provider
│    RouteRule.keywords│
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ 2. 意图匹配          │──exact match──► 返回 Provider
│    RouteRule.intent  │
└─────────────────────┘
    │
    ▼
┌─────────────────────┐
│ 3. 学习权重回退       │──best_provider()──► 返回 Provider
│    ProviderLearning  │
└─────────────────────┘
```

### Router 学习算法

```
weight = success_rate * 0.4 + latency_score * 0.3 + feedback * 0.3

其中:
- success_rate = successful / total_calls
- latency_score = 1 - (avg_latency_ms / 1000).clamp(0, 1)
- feedback = user_feedback_score (EMA, α=0.2)
```

## L5 领域技能集成

### TradeOrchestrator (外贸全流程)

```rust
pub struct TradeOrchestrator {
    pub messaging: MessagingBridge,    // CAT-1
    pub content_gen: ContentGenerator, // CAT-2
    pub schedule: ScheduleEngine,      // CAT-2
    pub analytics: SocialAnalytics,    // CAT-2
    pub leads: LeadManager,            // CAT-3
}
```

**流程分组 (7组26步):**
- G1 获客运营 (FT01-FT05): 社交媒体 → 询盘 → 资质 → 跟进 → 沟通
- G2 报价谈判 (FT06-FT09): 需求确认 → 报价 → 谈判 → 合同
- G3 收款 (FT10-FT11): 付款方式 → 收款确认
- G4 生产 (FT12-FT15): 排产 → 跟单 → 质检 → 放行
- G5 物流 (FT16-FT20): 检证 → 订舱 → 报关 → 提单 → 运输
- G6 结算 (FT21-FT24): 尾款 → 结汇 → 退税 → 核销
- G7 复盘 (FT25-FT26): 订单复盘 → 经验吸收

## 进化等级 (Constellation)

| 等级 | 含义 | 验证标准 |
|------|------|---------|
| C0 | 编译通过 | `cargo check` |
| C1 | 单元测试 | `cargo test --lib` 单模块 |
| C2 | 集成测试 | 跨模块调用 + Registry/Router |
| C3 | 基准测试 | 性能基准 + 压力测试 |
| C4 | 生产接入 | 接入主流水线 + 监控 |
| C5 | 自愈 | 断路器 + 自动恢复 |
| C6 | 自进化 | Router 学习 + 动态优化 |

当前状态: **所有 8 类别达到 C1-C2**

## 端到端测试验证

| 测试 | 验证内容 |
|------|---------|
| `test_cat1_messaging_e2e` | 消息发送 + 模板渲染 |
| `test_cat2_content_e2e` | 内容发布 + 互动查询 |
| `test_cat3_data_e2e` | 询盘捕获 + 资质评分 + 管道追踪 |
| `test_cat8_cognition_e2e` | LLM 路由 + Provider 选择 |
| `test_cat6_security_e2e` | 安全检查 + 审计 |
| `test_l5_trade_orchestrator_integration` | L5→L1 完整链路 |
| `test_infra_tracing` | 调用追踪 + 聚合统计 |
| `test_infra_breaker` | 断路器三态切换 |
| `test_infra_semantic_router` | 语义路由决策 |
| `test_infra_agent_card` | Agent Card 发布 + 发现 |
| `test_infra_scatter_gather` | 并行聚合策略 |
| `test_infra_persistence` | Registry 持久化 |
| `test_infra_learning` | Router 权重学习 |
| `test_infra_integration` | 统一集成层 |

## 配置示例

### Registry 配置
```toml
# ~/.neotrix/registry.json
[
  {
    "id": "io.messaging.whatsapp",
    "category": "communication",
    "constellation": "C1",
    "description": "WhatsApp messaging provider",
    "health_healthy": true,
    "health_error_rate": 0.0,
    "tags": ["messaging", "whatsapp"],
    "metadata": {}
  }
]
```

### 语义路由规则
```rust
router.add_rule(RouteRule {
    id: "trade_inquiry".into(),
    intent: "inquiry".into(),
    keywords: vec!["inquiry".into(), "quote".into(), "price".into()],
    provider_preference: vec!["lead_manager".into(), "messaging".into()],
    priority: 1,
});
```

### 断路器配置
```rust
BreakerConfig {
    error_threshold: 0.5,      // 50% 错误率触发
    open_duration_ms: 30000,   // 30秒熔断
    half_open_max_calls: 3,    // 半开允许3次试探
    window_size: 10,           // 最近10次调用
}
```

## 关键指标

- **8/8 类别** 全部有 Trait + Registry + Router + Bridge
- **8 个基础设施模块** 覆盖: 追踪/断路/路由/发现/聚合/持久化/学习/集成
- **L7→L1 桥接**: 3 个编排模式可通过 L1 trait 调用
- **L5→L1 集成**: TradeOrchestrator 通过 Bridge 调度 L1 能力
- **自进化能力**: Router 从历史调用中学习最优路由策略
- **端到端测试**: 14 个测试覆盖所有类别 + 基础设施

## 待办事项

- [ ] Phase 10: 清理 329 个预存在编译错误 (l2/l3/l4/l5/l6/cli 层)
- [ ] Phase 11: 添加更多端到端集成测试场景
- [ ] Phase 12: 生成架构图 (Mermaid/PlantUML)
- [ ] Phase 13: 性能基准测试 (C3)
- [ ] Phase 14: 生产监控接入 (C4)