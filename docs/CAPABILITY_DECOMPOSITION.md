# 能力拆解分析：通用能力 vs 外贸特有能力

## 核心原则

**Agent = 通用能力的组装，不是重复构建**

```
┌─────────────────────────────────────────────────────────────┐
│                    NT-ACT (通用能力层)                        │
│  编排器 | Worker池 | 路由器 | 消息协议 | 事件总线 | 数据管道    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 NT-ACT-TRADE (外贸特有能力)                   │
│  富通天下适配器 | CRM引擎 | 邮件引擎 | 智能分析 | 销售教练      │
└─────────────────────────────────────────────────────────────┘
```

## 1. 通用能力 (应提取到 nt_core / nt_act)

### 1.1 编排系统 (Orchestration)

| 能力 | 当前位置 | 建议位置 | 说明 |
|------|----------|----------|------|
| `TradeEngine` trait | `nt_act_trade/engine_traits.rs` | `nt_act/engine/engine_traits.rs` | 所有Engine的通用契约 |
| `TradeEngineRegistry` | `nt_act_trade/engine_traits.rs` | `nt_act/engine/engine_registry.rs` | Engine动态注册表 |
| `WorkerType` enum | `nt_act_trade/workers/mod.rs` | `nt_act/worker/worker_types.rs` | 工作类型分类 |
| `TradeWorker` trait | `nt_act_trade/workers/mod.rs` | `nt_act/worker/worker_trait.rs` | Worker通用契约 |
| `WorkerPool` | `nt_act_trade/workers/mod.rs` | `nt_act/worker/worker_pool.rs` | Worker池管理 |
| `WorkerTask` / `WorkerResult` | `nt_act_trade/workers/mod.rs` | `nt_act/worker/worker_types.rs` | 任务输入输出 |
| `TradeOrchestrator` | `nt_act_trade/orchestrator_v2.rs` | `nt_act/orchestrator/orchestrator.rs` | 编排器核心 |
| `TradeRouter` | `nt_act_trade/router.rs` | `nt_act/orchestrator/router.rs` | 动态路由 |
| `TradeMessage` | `nt_act_trade/message.rs` | `nt_act/message/message.rs` | 消息协议 |
| `Envelope` | `nt_act_trade/message.rs` | `nt_act/message/envelope.rs` | 消息信封 |

### 1.2 数据管道 (Data Pipeline)

| 能力 | 当前位置 | 建议位置 | 说明 |
|------|----------|----------|------|
| `ExternalPlatformExtractor` trait | `nt_act_trade/extractors/mod.rs` | `nt_act/pipeline/extractor_trait.rs` | 平台提取器通用契约 |
| `TradeDataPipeline` | `nt_act_trade/data_pipeline.rs` | `nt_act/pipeline/data_pipeline.rs` | 数据管道编排 |
| `DataNormalizer` | `nt_act_trade/data_pipeline.rs` | `nt_act/pipeline/normalizer.rs` | 数据标准化 |
| `PlatformRegistry` | `nt_act_trade/platform_registry.rs` | `nt_act/pipeline/platform_registry.rs` | 平台注册表 |
| `ExtractConfig` / `EmailConfig` | `nt_act_trade/extractors/mod.rs` | `nt_act/pipeline/config.rs` | 配置类型 |
| `SyncResult` / `ExtractionResult` | `nt_act_trade/extractors/mod.rs` | `nt_act/pipeline/result_types.rs` | 结果类型 |

### 1.3 事件系统 (Event System)

| 能力 | 当前位置 | 建议位置 | 说明 |
|------|----------|----------|------|
| `TradeEventBus` | `nt_act_trade/event_bus.rs` | `nt_act/event/event_bus.rs` | 事件总线 |
| `TradeEvent` | `nt_act_trade/event_bus.rs` | `nt_act/event/event_types.rs` | 事件类型 |
| `EventType` | `nt_act_trade/event_bus.rs` | `nt_act/event/event_types.rs` | 事件分类 |
| `TradeEventHandler` trait | `nt_act_trade/event_bus.rs` | `nt_act/event/handler_trait.rs` | 处理器契约 |

### 1.4 流程引擎 (Process Engine)

| 能力 | 当前位置 | 建议位置 | 说明 |
|------|----------|----------|------|
| `ProcessEngine` | `nt_act_trade/process_engine.rs` | `nt_act/process/process_engine.rs` | 流程引擎 |
| `ProcessDefinition` | `nt_act_trade/process_engine.rs` | `nt_act/process/process_def.rs` | 流程定义 |
| `ProcessInstance` | `nt_act_trade/process_engine.rs` | `nt_act/process/instance.rs` | 流程实例 |
| `StepHandler` trait | `nt_act_trade/process_engine.rs` | `nt_act/process/step_handler.rs` | 步骤处理器 |

### 1.5 知识库接口 (Knowledge Base)

| 能力 | 当前位置 | 建议位置 | 说明 |
|------|----------|----------|------|
| `KnowledgeBase` trait | `nt_act_trade/knowledge_base.rs` | `nt_act/knowledge/kb_trait.rs` | 知识库通用接口 |
| `KnowledgeResult` | `nt_act_trade/knowledge_base.rs` | `nt_act/knowledge/kb_types.rs` | 查询结果 |
| `PriceQuery` / `PriceResult` | `nt_act_trade/knowledge_base.rs` | `nt_act/knowledge/price_types.rs` | 价格查询 |

### 1.6 通用类型 (Common Types)

| 类型 | 当前位置 | 建议位置 | 说明 |
|------|----------|----------|------|
| `EngineType` | `nt_act_trade/engine_traits.rs` | `nt_act/engine/engine_types.rs` | Engine分类 |
| `EngineStatus` | `nt_act_trade/engine_traits.rs` | `nt_act/engine/engine_status.rs` | 健康状态 |
| `EngineMetrics` | `nt_act_trade/engine_traits.rs` | `nt_act/engine/engine_metrics.rs` | 性能指标 |
| `MessagePriority` | `nt_act_trade/message.rs` | `nt_act/message/priority.rs` | 消息优先级 |
| `TaskStatus` | `nt_act_trade/message.rs` | `nt_act/message/task_status.rs` | 任务状态 |

## 2. 外贸特有能力 (应保留在 nt_act_trade)

### 2.1 平台适配器 (Platform Adapters)

| 能力 | 文件 | 说明 |
|------|------|------|
| `JoinfExtractor` | `extractors/joinf.rs` | 富通天下平台适配器 |
| `ChromeDecryptor` | `extractors/chrome_decrypt.rs` | Chrome密码解密 |
| `SeleniumSession` | `extractors/selenium_automation.rs` | Selenium自动化 |
| 富通天下API端点 | `joinf.rs` | `/rapi/d/customers`, `/rapi/b/emails` 等 |

### 2.2 外贸业务引擎 (Trade Business Engines)

| 能力 | 文件 | 说明 |
|------|------|------|
| `TradeCrmEngine` | `nt_trade_crm.rs` | 外贸CRM |
| `TradeEmailEngine` | `nt_trade_email.rs` | 外贸邮件 |
| `TradePipelineEngine` | `nt_trade_pipeline.rs` | 销售管道 |
| `TradeDocumentEngine` | `nt_trade_documents.rs` | 单证管理 |
| `TradeTaskEngine` | `nt_trade_tasks.rs` | 任务日历 |
| `TradeDashboardEngine` | `nt_trade_dashboard.rs` | 数据看板 |
| `SupplierMgmtEngine` | `nt_trade_supplier_mgmt.rs` | 供应商评估 |
| `FinanceEngine` | `finance_compliance.rs` | 财务合规 |
| `ProductionEngine` | `production_logistics.rs` | 生产跟踪 |
| `LogisticsEngine` | `production_logistics.rs` | 物流管理 |

### 2.3 外贸领域模型 (Trade Domain Models)

| 类型 | 文件 | 说明 |
|------|------|------|
| `Customer` | `unified_types.rs` | 外贸客户 |
| `Supplier` | `unified_types.rs` | 供应商 |
| `Inquiry` | `unified_types.rs` | 询盘 |
| `Quote` | `unified_types.rs` | 报价 |
| `Order` | `unified_types.rs` | 订单 |
| `Grade` | `unified_types.rs` | 客户等级 |
| `Channel` | `unified_types.rs` | 客户来源渠道 |
| `TradeTerms` | `unified_types.rs` | 贸易条款 |

### 2.4 外贸业务流程 (Trade Business Logic)

| 能力 | 文件 | 说明 |
|------|------|------|
| `NegotiationEngine` | `trade_core.rs` | 报价谈判算法 |
| `RiskAssessor` | `trade_core.rs` | 风险评估 |
| `StateMachine` | `trade_core.rs` | 订单状态机 |
| `QuoteGenerator` | `quote_negotiation.rs` | 报价生成 |
| `ContractParser` | `contract_parser.rs` | 合同解析 |
| `TemplateDetector` | `template_detector.rs` | 模板检测 |

### 2.5 外贸智能分析 (Trade Intelligence)

| 能力 | 文件 | 说明 |
|------|------|------|
| `TradeIntelligence` | `l5_cognition/nt_core/trade_intelligence.rs` | 贸易智能分析 |
| `SalespersonProfiler` | `l4_emotion/nt_feel/salesperson_profiling.rs` | 业务员画像 |
| `WritingStyleAnalyzer` | `l4_emotion/nt_feel/writing_style.rs` | 写作风格分析 |
| `SalesCoach` | `l5_cognition/nt_mind/sales_coaching.rs` | 销售教练 |

## 3. 重构路线图

### Phase 1: 提取通用能力到 nt_act (Week 1-2)

```
neotrix-core/src/l1_action/nt_act/
├── engine/
│   ├── mod.rs
│   ├── engine_traits.rs      (从 nt_act_trade 迁移)
│   ├── engine_registry.rs    (从 nt_act_trade 迁移)
│   ├── engine_types.rs       (从 nt_act_trade 迁移)
│   ├── engine_status.rs      (从 nt_act_trade 迁移)
│   └── engine_metrics.rs     (从 nt_act_trade 迁移)
├── worker/
│   ├── mod.rs
│   ├── worker_trait.rs       (从 nt_act_trade 迁移)
│   ├── worker_pool.rs        (从 nt_act_trade 迁移)
│   ├── worker_types.rs       (从 nt_act_trade 迁移)
│   ├── extract_worker.rs     (从 nt_act_trade 迁移)
│   ├── analyze_worker.rs     (从 nt_act_trade 迁移)
│   ├── write_worker.rs       (从 nt_act_trade 迁移)
│   ├── send_worker.rs        (从 nt_act_trade 迁移)
│   └── track_worker.rs       (从 nt_act_trade 迁移)
├── orchestrator/
│   ├── mod.rs
│   ├── orchestrator.rs       (从 nt_act_trade 迁移)
│   └── router.rs             (从 nt_act_trade 迁移)
├── message/
│   ├── mod.rs
│   ├── message.rs            (从 nt_act_trade 迁移)
│   ├── envelope.rs           (从 nt_act_trade 迁移)
│   └── types.rs              (从 nt_act_trade 迁移)
├── pipeline/
│   ├── mod.rs
│   ├── extractor_trait.rs    (从 nt_act_trade 迁移)
│   ├── data_pipeline.rs      (从 nt_act_trade 迁移)
│   ├── normalizer.rs         (从 nt_act_trade 迁移)
│   └── platform_registry.rs  (从 nt_act_trade 迁移)
├── event/
│   ├── mod.rs
│   ├── event_bus.rs          (从 nt_act_trade 迁移)
│   ├── event_types.rs        (从 nt_act_trade 迁移)
│   └── handler_trait.rs      (从 nt_act_trade 迁移)
├── process/
│   ├── mod.rs
│   ├── process_engine.rs     (从 nt_act_trade 迁移)
│   ├── process_def.rs        (从 nt_act_trade 迁移)
│   └── step_handler.rs       (从 nt_act_trade 迁移)
└── knowledge/
    ├── mod.rs
    ├── kb_trait.rs            (从 nt_act_trade 迁移)
    └── kb_types.rs            (从 nt_act_trade 迁移)
```

### Phase 2: nt_act_trade 引用通用能力 (Week 3)

```rust
// nt_act_trade/mod.rs 改为:
pub use nt_act::engine::{TradeEngine, TradeEngineRegistry, EngineType, EngineStatus, EngineMetrics};
pub use nt_act::worker::{TradeWorker, WorkerPool, WorkerType, WorkerTask, WorkerResult};
pub use nt_act::orchestrator::{TradeOrchestrator, TradeRouter};
pub use nt_act::message::{TradeMessage, Envelope, MessagePriority};
pub use nt_act::pipeline::{ExternalPlatformExtractor, TradeDataPipeline, DataNormalizer};
pub use nt_act::event::{TradeEventBus, TradeEvent, EventType};
pub use nt_act::process::{ProcessEngine, ProcessDefinition};

// 外贸特有能力保留
pub mod extractors;  // JoinfExtractor 等
pub mod nt_trade_crm;
pub mod nt_trade_email;
// ... 其他外贸引擎
```

### Phase 3: 清理 nt_act_trade 中的通用代码 (Week 4)

删除已迁移的文件：
- `engine_traits.rs`
- `workers/` (整个目录)
- `orchestrator_v2.rs`
- `router.rs`
- `message.rs`
- `data_pipeline.rs`
- `platform_registry.rs`
- `event_bus.rs`
- `process_engine.rs`
- `knowledge_base.rs`

## 4. 收益分析

### 4.1 代码复用

| 收益 | 说明 |
|------|------|
| 避免重复实现 | 其他Agent（如CRM Agent、电商Agent）可直接复用编排/Worker/管道 |
| 统一接口 | 所有Agent使用相同的Engine/Worker trait |
| 降低维护成本 | 通用能力只需维护一份 |

### 4.2 架构清晰

| 收益 | 说明 |
|------|------|
| 职责分离 | nt_act = 通用能力，nt_act_trade = 外贸业务 |
| 依赖方向清晰 | trade 依赖 act，不反向依赖 |
| 测试独立 | 通用能力可独立测试 |

### 4.3 扩展性

| 收益 | 说明 |
|------|------|
| 新Agent快速构建 | 只需实现业务逻辑，复用通用能力 |
| 跨域组合 | 外贸+物流+金融可组合使用同一套编排系统 |
| 插件化 | 平台适配器可动态加载 |

## 5. 当前问题

### 5.1 代码重复

| 问题 | 位置 | 影响 |
|------|------|------|
| Worker trait 定义 | `nt_act_trade/workers/mod.rs` | 其他Agent无法复用 |
| Orchestrator 实现 | `nt_act_trade/orchestrator_v2.rs` | 其他Agent无法复用 |
| Message 协议 | `nt_act_trade/message.rs` | 其他Agent无法复用 |
| Event bus | `nt_act_trade/event_bus.rs` | 其他Agent无法复用 |

### 5.2 依赖混乱

| 问题 | 位置 | 影响 |
|------|------|------|
| trade 依赖 act 但 act 没有通用编排 | 循环依赖风险 | 架构不清晰 |
| 外贸类型定义在 unified_types.rs | 其他模块难以引用 | 类型孤岛 |

### 5.3 测试隔离

| 问题 | 位置 | 影响 |
|------|------|------|
| 测试在 nt_act_trade/tests/ | 无法测试通用能力 | 测试覆盖不完整 |

## 6. 行动清单

### 立即行动 (P0)

- [ ] 创建 `nt_act/engine/` 模块
- [ ] 创建 `nt_act/worker/` 模块
- [ ] 创建 `nt_act/orchestrator/` 模块
- [ ] 创建 `nt_act/message/` 模块
- [ ] 创建 `nt_act/pipeline/` 模块

### 短期行动 (P1)

- [ ] nt_act_trade 引用通用能力
- [ ] 删除 nt_act_trade 中的通用代码
- [ ] 更新所有测试

### 中期行动 (P2)

- [ ] 为其他Agent创建示例
- [ ] 文档化通用能力 API
- [ ] 性能基准测试

## 7. 总结

**当前状态**: 通用能力与外贸能力混合在 nt_act_trade 中

**目标状态**: 
- nt_act = 通用能力（可复用）
- nt_act_trade = 外贸特有能力（组装通用能力）

**核心收益**: Agent = 通用能力的组装，不是重复构建
