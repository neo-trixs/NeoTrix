# 架构扩散分析报告：重复与缺失

## 1. 重复构建清单

### 1.1 高严重度 (HIGH)

| 类型 | 位置1 | 位置2 | 问题 | 建议 |
|------|-------|-------|------|------|
| **WorkerPool** | `workers/mod.rs` (trait-object) | `orchestrator_v2.rs` (async callback) | 同一子模块内两套 WorkerPool | 统一为单一 WorkerPool trait |
| **WorkerType** | `workers/mod.rs` (Extract/Analyze/Write/Send/Track) | `orchestrator_v2.rs` (Inquiry/Quotation/Contract...) | 同名不同语义 | 分层: DomainWorkerType + TaskWorkerType |
| **WorkerResult** | `workers/mod.rs` | `orchestrator_v2.rs` | 同名结构体字段不同 | 统一为单一 WorkerResult |
| **EventBus** | `nt_core_event_bus.rs` (全局) | `nt_act_trade/event_bus.rs` (TradeEventBus) | 重复事件总线 | trade 事件通过全局 EventBus + EventType 过滤 |
| **CapabilityRegistry** | `nt_core_capability/mod.rs` | `core/l7_capability/registry.rs` | 全局级别两个注册表 | 统一为单一 CapabilityRegistry |

### 1.2 中严重度 (MEDIUM)

| 类型 | 位置1 | 位置2 | 问题 | 建议 |
|------|-------|-------|------|------|
| **TradeOrchestrator** | `orchestrator.rs` (v1, sync) | `orchestrator_v2.rs` (v2, async) | v1 deprecated 但未删除 | 加速迁移，删除 v1 |
| **Message 协议** | `message.rs` (TradeMessage) | `orchestrator_v2.rs` (TradeMessage) | 重复定义 | 统一引用 message.rs |
| **PerformanceMetrics** | `unified_types.rs` | `data_model.rs` | SSOT 已声明但有残留 | 清理 data_model 残留 |
| **RiskLevel** | `trade_core.rs` | `full_cycle.rs` / `production_logistics.rs` | 已标注但未执行清理 | 执行 REDUNDANCY_REPORT 清理 |
| **Orchestrator** | `nt_act/orchestrator.rs` | `nt_act_trade/orchestrator.rs` | 两个域级实现 | 明确职责边界 |

## 2. 缺失统一构建清单

### 2.1 基础设施能力

| 能力 | 全局实现 | nt_act_trade 实现 | 缺失 | 建议 |
|------|---------|-------------------|------|------|
| **日志/追踪** | `log` crate | 无 | 结构化日志 | 引入 tracing，关键路径添加 span |
| **配置管理** | `NeoTrixConfig` | 每个 Engine 各自 struct | 统一配置框架 | 引入 Config trait + TOML/YAML 加载 |
| **错误处理** | `NeoTrixError` + `L1Error` | 多个独立 Error enum | 统一错误分类 | 定义 `TradeError` 枚举 + From 转换 |
| **缓存** | `nt_core_cache.rs` | 无 | 缓存层 | 引入 LRU 缓存用于产品/价格查询 |
| **重试** | `nt_core_error/recovery.rs` | 手写 retry loop | 通用 RetryPolicy | 提取 RetryPolicy (backoff/jitter) |
| **超时** | 无 | `timeout_secs` 字段 | 统一超时配置 | 统一 TimeoutConfig，所有 async 强制超时 |
| **限流** | `nt_infra_breaker.rs` | `rate_limit` 字段 (空壳) | 限流实现 | 实现 token-bucket 或 sliding-window |
| **熔断** | `circuit_breaker.rs` | 注释提到但未实现 | 熔断集成 | 集成全局 CircuitBreaker |
| **指标收集** | 无 | `EngineMetrics` (局部) | 全局 metrics facade | 接入 Prometheus/OpenTelemetry |
| **健康检查** | `nt_core_heartbeat.rs` | `health_check` trait | 注册到全局 | 注册到 HeartbeatAggregator |

### 2.2 业务基础设施

| 能力 | 是否有统一实现 | 问题 | 建议 |
|------|-------------|------|------|
| **Platform Adapter** | `ExternalPlatformExtractor` trait | 仅有 JoinfExtractor 一个实现 | 定义标准 adapter 接口 |
| **Data Normalizer** | `DataNormalizer` | 仅在 data_pipeline 中 | 提取为通用能力 |
| **Template Engine** | 无 | Email/Document 各自模板 | 统一模板引擎 |

## 3. 跨层依赖分析

| 模块 | L2-L6 依赖 | 状态 |
|------|-----------|------|
| `nt_act_trade/` | 无 | ✅ 合规 |
| `nt_act/` | 无 | ✅ 合规 |

**结论**: 架构分层合规，无向上依赖。

## 4. 能力网 vs 流程引擎

```
capabilities/ (纯计算能力)
    │
    ├── price_calculator     (无状态)
    ├── product_matcher      (无状态)
    ├── risk_assessor        (无状态)
    └── supplier_matcher     (无状态)
    
process_engine.rs (流程编排)
    │
    └── ProcessDefinition → ProcessInstance → StepHandler
    
orchestrator_v2.rs (异步任务调度)
    │
    └── TradeRouter → WorkerPool → TaskTracker
```

**问题**: 三者无统一注册/发现机制

**建议**: 形成三层统一架构
```
Capability (计算) → Process (编排) → Worker (执行)
       │                  │                │
       └──────── CapabilityRegistry ───────┘
```

## 5. Top 5 修复优先级

### P0: WorkerPool/WorkerType/WorkerResult 统一
- 位置: `workers/mod.rs` + `orchestrator_v2.rs`
- 影响: 同一子模块内类型混乱
- 方案: 统一为单一类型系统

### P1: EventBus 统一
- 位置: `nt_core_event_bus.rs` + `nt_act_trade/event_bus.rs`
- 影响: 全局事件总线重复
- 方案: trade 事件通过全局 EventBus + EventType 过滤

### P2: TradeError 统一错误枚举
- 位置: 多个独立 Error enum
- 影响: 错误处理不一致
- 方案: 定义 `TradeError` 枚举 + From 转换

### P3: Rate Limit / Circuit Breaker 实现
- 位置: `platform_registry.rs` (空壳) + `orchestrator_v2.rs` (注释)
- 影响: 外部调用无保护
- 方案: 集成全局 CircuitBreaker + 实现限流器

### P4: CapabilityRegistry 统一注册
- 位置: `capabilities/` + `process_engine.rs` + `orchestrator_v2.rs`
- 影响: 能力无法统一发现
- 方案: 统一 CapabilityRegistry 协议

## 6. 文件清单

### 需要统一的文件
```
workers/mod.rs + orchestrator_v2.rs → 统一 Worker 类型
event_bus.rs → 集成到全局 EventBus
KnowledgeBaseError + JoinfError + ... → 统一为 TradeError
platform_registry.rs → 实现限流
orchestrator_v2.rs → 实现熔断
```

### 需要新增的文件
```
nt_act/error.rs          → 统一错误处理
nt_act/retry.rs          → 通用重试策略
nt_act/timeout.rs        → 统一超时配置
nt_act/circuit_breaker.rs → 熔断器
nt_act/rate_limiter.rs   → 限流器
```

### 需要删除的文件
```
orchestrator.rs (v1)     → 已 deprecated
orchestrator_compat.rs   → v1 兼容层
```
