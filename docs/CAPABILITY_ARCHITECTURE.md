# NeoTrix 能力架构图

## 当前状态 (混合)

```
┌─────────────────────────────────────────────────────────────────┐
│                    nt_act_trade (30,544行)                      │
├─────────────────────────────────────────────────────────────────┤
│  通用能力 (应提取)           │  外贸特有能力 (应保留)            │
├─────────────────────────────┼───────────────────────────────────┤
│  engine_traits.rs (488)     │  extractors/joinf.rs (1125)      │
│  workers/ (1396)            │  extractors/chrome_decrypt (239) │
│  orchestrator_v2.rs (1403)  │  extractors/selenium (515)       │
│  router.rs (291)            │  nt_trade_crm.rs (768)           │
│  message.rs (784)           │  nt_trade_email.rs (445)         │
│  data_pipeline.rs (561)     │  nt_trade_pipeline.rs (553)      │
│  platform_registry.rs (221) │  nt_trade_documents.rs (435)     │
│  event_bus.rs (1016)        │  nt_trade_tasks.rs (392)         │
│  process_engine.rs (1224)   │  nt_trade_dashboard.rs (458)     │
│  knowledge_base.rs (554)    │  nt_trade_supplier_mgmt.rs (270) │
│                             │  finance_compliance.rs (957)      │
│                             │  production_logistics.rs (996)    │
│                             │  trade_core.rs (931)              │
│                             │  unified_types.rs (714)           │
│                             │  orchestrator.rs (1488) [deprecated]│
├─────────────────────────────┼───────────────────────────────────┤
│  ~6,714行 (通用)            │  ~10,249行 (外贸)                │
└─────────────────────────────┴───────────────────────────────────┘
```

## 目标状态 (分离)

```
┌─────────────────────────────────────────────────────────────────┐
│                    nt_act (通用能力层)                           │
├─────────────────────────────────────────────────────────────────┤
│  engine/                    │  worker/                         │
│  ├── engine_traits.rs       │  ├── worker_trait.rs             │
│  ├── engine_registry.rs     │  ├── worker_pool.rs              │
│  ├── engine_types.rs        │  ├── worker_types.rs             │
│  ├── engine_status.rs       │  ├── extract_worker.rs           │
│  └── engine_metrics.rs      │  ├── analyze_worker.rs           │
│                             │  ├── write_worker.rs             │
│  orchestrator/              │  ├── send_worker.rs              │
│  ├── orchestrator.rs        │  └── track_worker.rs             │
│  └── router.rs              │                                  │
│                             │  message/                        │
│  pipeline/                  │  ├── message.rs                  │
│  ├── extractor_trait.rs     │  ├── envelope.rs                 │
│  ├── data_pipeline.rs       │  └── types.rs                    │
│  ├── normalizer.rs          │                                  │
│  └── platform_registry.rs   │  event/                          │
│                             │  ├── event_bus.rs                │
│  process/                   │  ├── event_types.rs              │
│  ├── process_engine.rs      │  └── handler_trait.rs            │
│  ├── process_def.rs         │                                  │
│  └── step_handler.rs        │  knowledge/                      │
│                             │  ├── kb_trait.rs                 │
│                             │  └── kb_types.rs                 │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                 nt_act_trade (外贸特有能力)                      │
├─────────────────────────────────────────────────────────────────┤
│  extractors/                                                     │
│  ├── joinf.rs (富通天下适配器)                                    │
│  ├── chrome_decrypt.rs (Chrome密码解密)                          │
│  └── selenium_automation.rs (Selenium自动化)                     │
│                                                                 │
│  engines/                                                        │
│  ├── nt_trade_crm.rs (外贸CRM)                                  │
│  ├── nt_trade_email.rs (外贸邮件)                                │
│  ├── nt_trade_pipeline.rs (销售管道)                             │
│  ├── nt_trade_documents.rs (单证管理)                            │
│  ├── nt_trade_tasks.rs (任务日历)                                │
│  ├── nt_trade_dashboard.rs (数据看板)                            │
│  ├── nt_trade_supplier_mgmt.rs (供应商评估)                      │
│  ├── finance_compliance.rs (财务合规)                            │
│  ├── production_logistics.rs (生产物流)                          │
│  └── trade_core.rs (报价谈判/风险评估)                           │
│                                                                 │
│  models/                                                        │
│  ├── unified_types.rs (外贸领域类型)                             │
│  ├── data_model.rs (数据模型)                                   │
│  └── knowledge_base.rs (外贸知识库)                             │
│                                                                 │
│  intelligence/                                                  │
│  ├── salesperson_profiling.rs (业务员画像)                       │
│  ├── writing_style.rs (写作风格分析)                             │
│  ├── trade_intelligence.rs (贸易智能)                           │
│  └── sales_coaching.rs (销售教练)                               │
└─────────────────────────────────────────────────────────────────┘
```

## 能力组装示例

```rust
// 创建外贸Agent = 组装通用能力 + 外贸能力
pub struct TradeAgent {
    // 通用能力
    orchestrator: TradeOrchestrator,      // nt_act
    worker_pool: WorkerPool,              // nt_act
    router: TradeRouter,                  // nt_act
    event_bus: TradeEventBus,             // nt_act
    pipeline: TradeDataPipeline,          // nt_act
    
    // 外贸特有能力
    crm: TradeCrmEngine,                 // nt_act_trade
    email: TradeEmailEngine,             // nt_act_trade
    intelligence: TradeIntelligence,     // nt_act_trade
    coach: SalesCoach,                   // nt_act_trade
}

impl TradeAgent {
    pub fn new() -> Self {
        // 1. 创建通用能力
        let mut registry = TradeEngineRegistry::new();
        let mut worker_pool = WorkerPool::new();
        let mut router = TradeRouter::new("default");
        
        // 2. 注册外贸引擎
        let crm = Arc::new(TradeCrmEngine::new());
        let email = Arc::new(TradeEmailEngine::new());
        registry.register(crm.clone());
        registry.register(email.clone());
        
        // 3. 注册Worker
        worker_pool.register(Arc::new(ExtractWorker::new(pipeline)));
        worker_pool.register(Arc::new(AnalyzeWorker::new()));
        worker_pool.register(Arc::new(WriteWorker::new()));
        worker_pool.register(Arc::new(SendWorker::new()));
        worker_pool.register(Arc::new(TrackWorker::new()));
        
        // 4. 配置路由
        router.add_rule(RouteRule {
            name: "extract".into(),
            condition: RouteCondition::TaskType("extract".into()),
            target_worker: "extract".into(),
            priority: 100,
        });
        
        Self {
            orchestrator: TradeOrchestrator::new(config),
            worker_pool,
            router,
            event_bus: TradeEventBus::new(),
            pipeline: TradeDataPipeline::new(),
            crm,
            email,
            intelligence: TradeIntelligence::new(),
            coach: SalesCoach::new(),
        }
    }
    
    /// 执行外贸任务
    pub async fn execute(&self, task: TradeTask) -> Result<OrchestratorResult> {
        // 通用编排器处理
        self.orchestrator.execute(task).await
    }
}
```

## 迁移检查清单

### nt_act 通用能力 (从 nt_act_trade 迁移)

- [ ] `engine_traits.rs` → `nt_act/engine/engine_traits.rs`
- [ ] `engine_registry` → `nt_act/engine/engine_registry.rs`
- [ ] `workers/` → `nt_act/worker/`
- [ ] `orchestrator_v2.rs` → `nt_act/orchestrator/orchestrator.rs`
- [ ] `router.rs` → `nt_act/orchestrator/router.rs`
- [ ] `message.rs` → `nt_act/message/message.rs`
- [ ] `data_pipeline.rs` → `nt_act/pipeline/data_pipeline.rs`
- [ ] `platform_registry.rs` → `nt_act/pipeline/platform_registry.rs`
- [ ] `event_bus.rs` → `nt_act/event/event_bus.rs`
- [ ] `process_engine.rs` → `nt_act/process/process_engine.rs`
- [ ] `knowledge_base.rs` → `nt_act/knowledge/kb_trait.rs`

### nt_act_trade 外贸特有能力 (保留)

- [ ] `extractors/joinf.rs` (保留)
- [ ] `extractors/chrome_decrypt.rs` (保留)
- [ ] `extractors/selenium_automation.rs` (保留)
- [ ] `nt_trade_crm.rs` (保留)
- [ ] `nt_trade_email.rs` (保留)
- [ ] `nt_trade_pipeline.rs` (保留)
- [ ] `nt_trade_documents.rs` (保留)
- [ ] `nt_trade_tasks.rs` (保留)
- [ ] `nt_trade_dashboard.rs` (保留)
- [ ] `nt_trade_supplier_mgmt.rs` (保留)
- [ ] `finance_compliance.rs` (保留)
- [ ] `production_logistics.rs` (保留)
- [ ] `trade_core.rs` (保留)
- [ ] `unified_types.rs` (保留)

### 清理 (迁移后删除)

- [ ] 删除 `nt_act_trade/engine_traits.rs`
- [ ] 删除 `nt_act_trade/workers/` 目录
- [ ] 删除 `nt_act_trade/orchestrator_v2.rs`
- [ ] 删除 `nt_act_trade/router.rs`
- [ ] 删除 `nt_act_trade/message.rs`
- [ ] 删除 `nt_act_trade/data_pipeline.rs`
- [ ] 删除 `nt_act_trade/platform_registry.rs`
- [ ] 删除 `nt_act_trade/event_bus.rs`
- [ ] 删除 `nt_act_trade/process_engine.rs`
- [ ] 删除 `nt_act_trade/knowledge_base.rs`

### 更新引用

- [ ] `nt_act_trade/mod.rs` 改为引用 `nt_act::*`
- [ ] 更新所有 `use nt_act_trade::*` 为 `use nt_act::*`
- [ ] 更新测试文件
