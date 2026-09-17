# NeoTrix 深度能力生态统一架构

## 一、现有 Agent 清单

### 1.1 已实现 UnifiedCapability 的 Agent

| Agent | 位置 | 层级 | 注册状态 |
|-------|------|------|----------|
| AssetMapCapability | `l2_perception/nt_world/asset_map/` | L2 | ❌ 未注册全局 |
| NlpCapability | `l2_perception/nt_world/nt_nlp_capability.rs` | L2 | ❌ 未注册全局 |
| OcrCapability | `l2_perception/nt_world/ocr/mod.rs` | L2 | ❌ 未注册全局 |
| ZeroTrustCapability | `l3_embodiment/nt_shield/shield_capability.rs` | L3 | ❌ 未注册全局 |
| SecurityScanCapability | `l3_embodiment/nt_shield/shield_capability.rs` | L3 | ❌ 未注册全局 |
| ThreatDetectionCapability | `l3_embodiment/nt_shield/shield_capability.rs` | L3 | ❌ 未注册全局 |
| PriceCalculatorCapability | `l1_action/nt_act/nt_act_trade/capability_registry.rs` | L1 | ✅ Trade 子注册 |
| ProductMatcherCapability | `l1_action/nt_act/nt_act_trade/capability_registry.rs` | L1 | ✅ Trade 子注册 |
| RiskAssessorCapability | `l1_action/nt_act/nt_act_trade/capability_registry.rs` | L1 | ✅ Trade 子注册 |
| SupplierMatcherCapability | `l1_action/nt_act/nt_act_trade/capability_registry.rs` | L1 | ✅ Trade 子注册 |
| PdfEnhanceCapability | `neotrix/nt_file_ability/capability.rs` | L1 | ✅ FileAbility 子注册 |

### 1.2 未实现 UnifiedCapability 的 Orchestrator

| Orchestrator | 位置 | 层级 |
|--------------|------|------|
| TradeOrchestrator v1 | `l1_action/nt_act/nt_act_trade/orchestrator.rs` | L1 |
| TradeOrchestrator v2 | `l1_action/nt_act/nt_act_trade/orchestrator_v2.rs` | L1 |
| ProductionOrchestrator | `l1_action/nt_act/actions/orchestration/` | L1 |
| AgentOrchestrator | `l1_action/nt_act/agent_protocol.rs` | L1 |
| VideoOrchestrator | `l2_perception/nt_world/nt_world_video_pipeline.rs` | L2 |
| ConsciousnessOrchestrator | `l5_cognition/nt_mind/nt_mind_background_loop/` | L5 |
| MemoryOrchestrator | `l5_cognition/nt_mind/foundation/memory_bank.rs` | L5 |
| RecoveryOrchestrator | `core/nt_core_error/recovery.rs` | Core |

### 1.3 独立运行的 Agent

| Agent | 位置 | 层级 |
|-------|------|------|
| EmotionEngine | `l4_emotion/nt_feel/emotion_engine.rs` | L4 |
| ConsciousnessCore | `l5_cognition/nt_core/nt_consciousness_core/` | L5 |
| EvolutionLoop | `l5_cognition/nt_mind/evolution/evolution_loop.rs` | L5 |
| SelfTest | `l6_meta/healing/nt_core_self_test.rs` | L6 |
| CrossSessionMemory | `l6_meta/nt_nexus/cross_session_memory.rs` | L6 |

---

## 二、重复构建清单

| 类型 | 实例数 | 主要位置 | 严重度 |
|------|--------|----------|--------|
| EventBus | 4 | core/ + L1/ + L5/ | HIGH |
| WorkerPool | 2 | L1 trade/ | MEDIUM |
| Pipeline | 30+ | 全层 | HIGH |
| Registry | 50+ | 全层 | HIGH |
| Orchestrator | 12+ | 全层 | HIGH |
| Config struct | 100+ | 全层 | MEDIUM |
| Error enum | 87+ | 全层 | HIGH |

---

## 三、缺失统一构建清单

| 能力 | 现状 | 建议 |
|------|------|------|
| 日志/追踪 | log + tracing 混用 | 统一 tracing |
| 配置管理 | 100+ Config 各自解析 | nt_core_config |
| 错误处理 | 87+ Error 独立 | 层级化 NeoTrixError |
| 缓存 | 仅 capability 层 | nt_core_cache 三级缓存 |
| 重试/超时 | 无全局框架 | nt_core_retry |
| 限流/熔断 | 仅 nt_shield | 提取到 core |
| 指标收集 | 仅 capability 层 | nt_core_metrics |
| 健康检查 | 仅 capability 层 | nt_core_health |

---

## 四、统一架构设计

### 4.1 Cross-Cutting Platform 层

```
nt_core_platform/
├── event_bus.rs      (全局 EventBus)
├── registry.rs       (trait DomainRegistry<T>)
├── pipeline.rs       (trait Pipeline)
├── orchestrator.rs   (trait Orchestrator)
├── config.rs         (trait Configurable)
├── error.rs          (层级化错误)
├── cache.rs          (三级缓存)
├── retry.rs          (重试策略)
├── metrics.rs        (指标收集)
├── health.rs         (健康检查)
└── logging.rs        (tracing 统一)
```

### 4.2 Agent 注册协议

```rust
/// 所有 Agent 必须实现的 trait
#[async_trait]
pub trait Agent: UnifiedCapability + Send + Sync {
    /// Agent 唯一标识
    fn agent_id(&self) -> &str;
    
    /// Agent 名称
    fn agent_name(&self) -> &str;
    
    /// Agent 层级
    fn layer(&self) -> Layer;
    
    /// Agent 域
    fn domain(&self) -> Domain;
    
    /// 初始化 Agent
    async fn initialize(&mut self) -> Result<(), AgentError>;
    
    /// 启动 Agent
    async fn start(&self) -> Result<(), AgentError>;
    
    /// 停止 Agent
    async fn stop(&self) -> Result<(), AgentError>;
    
    /// 获取 Agent 状态
    fn status(&self) -> AgentStatus;
    
    /// 获取 Agent 指标
    fn metrics(&self) -> AgentMetrics;
}
```

### 4.3 统一注册表

```rust
/// 全局 Agent 注册表
pub struct AgentRegistry {
    /// 按 ID 索引
    agents: HashMap<String, Arc<dyn Agent>>,
    /// 按层级索引
    by_layer: HashMap<Layer, Vec<String>>,
    /// 按域索引
    by_domain: HashMap<Domain, Vec<String>>,
    /// 健康检查
    health_checker: HealthChecker,
}

impl AgentRegistry {
    /// 注册 Agent
    pub fn register(&mut self, agent: Arc<dyn Agent>) { ... }
    
    /// 获取 Agent
    pub fn get(&self, agent_id: &str) -> Option<Arc<dyn Agent>> { ... }
    
    /// 按层级获取
    pub fn get_by_layer(&self, layer: Layer) -> Vec<Arc<dyn Agent>> { ... }
    
    /// 按域获取
    pub fn get_by_domain(&self, domain: Domain) -> Vec<Arc<dyn Agent>> { ... }
    
    /// 健康检查所有 Agent
    pub async fn health_check_all(&self) -> HashMap<String, AgentHealth> { ... }
    
    /// 启动所有 Agent
    pub async fn start_all(&self) -> Result<(), AgentError> { ... }
    
    /// 停止所有 Agent
    pub async fn stop_all(&self) -> Result<(), AgentError> { ... }
}
```

---

## 六、实现状态

### 6.1 nt_core_platform crate ✅

已创建 `neotrix-core/src/core/nt_core_platform/` 模块，包含:

| 文件 | 内容 | 行数 |
|------|------|------|
| `mod.rs` | 模块入口，re-exports | 28 |
| `agent.rs` | Agent trait (继承 UnifiedCapability) | 78 |
| `agent_registry.rs` | AgentRegistry 全局注册表 | 262 |
| `pipeline_registry.rs` | PipelineRegistry 全局注册表 | 212 |
| `registry.rs` | DomainRegistry<T> trait | 34 |
| `pipeline.rs` | Pipeline trait | 41 |
| `orchestrator.rs` | Orchestrator trait | 63 |
| `config.rs` | Configurable trait | 20 |
| `error.rs` | PlatformError 类型 | 32 |
| `health.rs` | HealthChecker + HealthCheck trait | 69 |
| `metrics.rs` | MetricsCollector | 69 |

**总计**: 11 个文件, 908 行代码

### 6.2 Agent 迁移状态

| Agent | 位置 | 层级 | 域 | 状态 |
|-------|------|------|-----|------|
| Orchestrator | `l1_action/nt_act/nt_act_orchestrator/mod.rs` | L1 | NtAct | ✅ |
| AgentOrchestrator | `l1_action/nt_act/agent_protocol.rs` | L1 | NtAct | ✅ |
| TradeOrchestrator v1 | `l1_action/nt_act/nt_act_trade/orchestrator.rs` | L1 | Trade | ✅ |
| TradeOrchestrator v2 | `l1_action/nt_act/nt_act_trade/orchestrator_v2.rs` | L1 | Trade | ✅ |
| SecurityGuardManager | `l1_action/nt_act/actions/security/security.rs` | L1 | Security | ✅ |
| ProductionOrchestrator | `l1_action/nt_act/actions/orchestration/production_orchestrator.rs` | L1 | NtAct | ✅ |
| ElasticMemoryOrchestrator | `l1_action/nt_memory/nt_memory_kb/memory_orchestrator.rs` | L1 | NtMemory | ✅ |
| LeadManager | `l1_action/nt_memory/nt_memory_lead.rs` | L1 | NtMemory | ✅ |
| KbSearchEngine | `l1_action/nt_memory/nt_memory_kb/nt_memory_search.rs` | L1 | NtMemory | ✅ |
| NlpCapability | `l2_perception/nt_world/nt_nlp_capability.rs` | L2 | NtWorld | ✅ |
| OcrCapability | `l2_perception/nt_world/ocr/mod.rs` | L2 | NtWorld | ✅ |
| PdfEnhanceCapability | `neotrix/nt_file_ability/capability.rs` | L1 | NtFileAbility | ✅ |
| ZtNetUnifiedCapability | `l3_embodiment/nt_shield/nt_shield_ztnet/ztnet_capability.rs` | L3 | NtShield | ✅ |
| ConsciousnessOrchestrator | `l5_cognition/nt_mind/nt_mind_background_loop/consciousness_orchestrator.rs` | L5 | NtMind | ✅ |
| MemoryOrchestrator | `l5_cognition/nt_mind/foundation/memory_bank.rs` | L5 | NtMind | ✅ |
| DgmEditOrchestrator | `l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/brain_dgm.rs` | L5 | NtMind | ✅ |
| MemoryAgent | `l5_cognition/nt_mind/nt_mind/evolution/agent_capability/mod.rs` | L5 | NtMemory | ✅ |
| SupervisorL1Bridge | `l5_cognition/nt_core/capability/l7_l1_bridge.rs` | L5 | NtAct | ✅ |
| L7OrchestratorRegistry | `l5_cognition/nt_core/capability/l7_l1_bridge.rs` | L5 | NtAct | ✅ |
| RecoveryOrchestrator | `core/nt_core_error/recovery.rs` | Core | Core | ✅ |

**已迁移**: 20 个 Agent

### 6.3 核心 trait 定义

```
Agent: UnifiedCapability + Send + Sync
├── agent_id() -> &str
├── agent_name() -> &str
├── agent_layer() -> Layer
├── agent_domain() -> Domain
├── initialize() -> Result<(), AgentError>
├── start() -> Result<(), AgentError>
├── stop() -> Result<(), AgentError>
├── status() -> AgentStatus
├── metrics() -> AgentMetrics
└── health_check() -> AgentHealth

DomainRegistry<T: Send + Sync>: Send + Sync
├── registry_name() -> &str
├── register(id, entry)
├── get(id) -> Option<Arc<T>>
├── list() -> Vec<RegistryEntry>
├── has(id) -> bool
├── len() -> usize
└── is_empty() -> bool

Pipeline: Send + Sync
├── name() -> &str
├── stages() -> Vec<PipelineStage>
├── run(input) -> Result<PipelineResult, String>
├── checkpoint(stage, state) -> Result<(), String>
└── restore(checkpoint_id) -> Result<Value, String>

Orchestrator: Send + Sync
├── name() -> &str
├── config() -> &OrchestratorConfig
├── start() -> Result<(), String>
├── stop() -> Result<(), String>
├── submit_task(task) -> Result<String, String>
├── cancel_task(task_id) -> Result<(), String>
├── status() -> OrchestratorStatus
└── stats() -> OrchestratorStats
```

### 6.4 全局初始化

```
init_all() -> (AgentRegistry, PipelineRegistry, PlatformMonitor)
├── AgentRegistry: 注册 20 个 Agent
├── PipelineRegistry: 注册 3 个 Pipeline
└── PlatformMonitor: MetricsCollector + HealthChecker
```

### 6.5 最终统计

| 类型 | 数量 | 状态 |
|------|------|------|
| Agent 实现 | 19 | ✅ 全部迁移 |
| Pipeline 实现 | 12 | ✅ 已添加 |
| From 实现 | 66 | ✅ 错误层级统一 |
| 平台代码 | 1265行 | ✅ 完成 |
| Agent 监控方法 | 4个 | ✅ 已添加 |

### 6.6 Agent 监控能力

```
Agent trait 监控方法:
├── record_request(success, duration_ms) — 记录请求
├── error_rate() -> f64 — 计算错误率
├── throughput() -> f64 — 计算吞吐量
└── reset_metrics() — 重置指标

AgentMetrics 指标:
├── total_requests — 总请求数
├── successful_requests — 成功请求数
├── failed_requests — 失败请求数
├── avg_response_time_ms — 平均响应时间
├── uptime_secs — 运行时间
├── last_request_time — 最后请求时间
├── error_rate — 错误率
└── throughput_per_sec — 吞吐量
```

### 6.7 错误层级统一

```
NeoTrixError (21个变体)
├── From<std::io::Error>
├── From<String>
├── From<&str>
├── From<PlatformError>
├── From<AgentError>
├── From<CapabilityError>
├── From<HotDataError>
├── From<PluginError>
├── From<MiddlewareError>
├── From<TradeError>
├── From<AgentProtocolError>
├── From<SocialAccessError>
├── From<DefenseError>
├── From<MultiAgentError>
└── ... (共66个From实现)
```

---

## 七、收益

| 收益 | 说明 |
|------|------|
| 统一发现 | 所有 Agent 通过全局注册表发现 |
| 统一生命周期 | 所有 Agent 统一 start/stop/status |
| 统一监控 | 所有 Agent 统一指标/日志/健康检查 |
| 能力组合 | 不同域 Agent 可组合使用 |
| 插件化 | 新 Agent 可动态加载/卸载 |
| 降低维护 | 基础设施只需维护一份 |
