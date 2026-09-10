//! 能力接口文档
//!
//! NeoTrix统一能力接口设计文档

/// # NeoTrix 统一能力接口
///
/// ## 概述
///
/// NeoTrix统一能力接口是一个跨域调用框架，支持:
/// - 能力注册与发现
/// - 统一输入/输出
/// - 路由调度
/// - 健康监控
/// - 跨域组合
/// - 结果缓存
/// - 分布式发现
/// - 版本管理
///
/// ## 架构
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │                    ConsciousnessCapabilityIntegrator         │
/// │              (意识核心集成 + 版本管理 + 发现)               │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    OrchestrationEngine                       │
/// │        (顺序/并行/条件/循环/分支编排模式)                    │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    LoadBalancer                              │
/// │       (轮询/加权/最少连接/响应时间/资源使用率)              │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    DistributedDiscovery                      │
/// │         (本地/UDP/HTTP/gRPC/WebSocket发现)                  │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    VersionManager                            │
/// │           (语义版本/兼容性/升级/回滚/统计)                  │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    CapabilityComposer                        │
/// │           (跨域组合 + 管道执行 + 并行调用)                   │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    CapabilityCache                           │
/// │            (LRU缓存 + 过期策略 + 统计)                      │
/// ├─────────────────────────────────────────────────────────────┤
/// │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
/// │  │ NT-MIND      │  │ NT-MEMORY    │  │ NT-ACT       │      │
/// │  │ 技能结晶     │  │ 知识存储     │  │ 工具执行     │      │
/// │  │ 元认知       │  │ 知识检索     │  │ 任务编排     │      │
/// │  │ SEAL         │  │ 版本管理     │  │ 自主行动     │      │
/// │  └──────────────┘  └──────────────┘  └──────────────┘      │
/// │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
/// │  │ NT-SHIELD    │  │ NT-WORLD     │  │ NT-IO        │      │
/// │  │ 零信任网络   │  │ 网络爬虫     │  │ LLM接口      │      │
/// │  │ 安全扫描     │  │ 内容解析     │  │ CLI接口      │      │
/// │  │ 威胁检测     │  │ 知识图谱     │  │ Web服务      │      │
/// │  └──────────────┘  └──────────────┘  └──────────────┘      │
/// ├─────────────────────────────────────────────────────────────┤
/// │                    UnifiedCapability Trait                   │
/// │         (meta + health + execute + supports + reset)        │
/// └─────────────────────────────────────────────────────────────┘
/// ```
///
/// ## 核心接口
///
/// ### UnifiedCapability Trait
///
/// ```rust
/// pub trait UnifiedCapability: Send + Sync {
///     fn meta(&self) -> CapabilityMeta;      // 元数据
///     fn health(&self) -> CapabilityHealth;  // 健康状态
///     fn execute(&self, input: CapabilityInput) -> Result<CapabilityOutput, CapabilityError>;
///     fn supports(&self, input: &CapabilityInput) -> bool;
///     fn reset(&mut self);
/// }
/// ```
///
/// ### 能力输入/输出
///
/// ```rust
/// pub enum CapabilityInput {
///     Text(String),
///     Nlp(NlpInput),
///     Asset(AssetInput),
///     Network(NetworkInput),
///     Security(SecurityInput),
/// }
///
/// pub enum CapabilityOutput {
///     Text(String),
///     Nlp(NlpResult),
///     Asset(AssetResult),
///     NetworkResult(NetworkResult),
///     SecurityScan(SecurityScanResult),
/// }
/// ```
///
/// ## 使用示例
///
/// ### 基本使用
///
/// ```rust
/// use neotrix::core::nt_core_capability::*;
///
/// // 创建能力
/// let cap = create_nlp_capability();
///
/// // 执行能力
/// let input = CapabilityInput::Nlp(NlpInput {
///     task: NlpTask::Tokenize,
///     text: "测试文本".into(),
///     language: None,
/// });
/// let output = cap.execute(input)?;
/// ```
///
/// ### 路由调度
///
/// ```rust
/// use neotrix::core::nt_core_capability::*;
///
/// // 创建路由器
/// let registry = Arc::new(init_global_registry());
/// let mut router = CapabilityRouter::new(registry);
///
/// // 添加路由规则
/// router.add_rule(|input| match input {
///     CapabilityInput::Nlp(_) => Some("nt-world-nlp".into()),
///     _ => None,
/// });
///
/// // 路由请求
/// let output = router.route(input)?;
/// ```
///
/// ### 能力组合
///
/// ```rust
/// use neotrix::core::nt_core_capability::*;
///
/// // 创建组合器
/// let registry = Arc::new(init_global_registry());
/// let composer = CapabilityComposer::new(registry);
///
/// // 创建管道
/// let pipeline = CapabilityComposer::create_pipeline("asset_discovery").unwrap();
///
/// // 执行管道
/// let result = composer.execute_pipeline(&pipeline, input)?;
/// ```
///
/// ### 缓存加速
///
/// ```rust
/// use neotrix::core::nt_core_capability::*;
///
/// // 创建缓存包装器
/// let cap = create_nlp_capability();
/// let cached = CachedCapability::new(cap, CacheConfig::default());
///
/// // 带缓存执行
/// let output = cached.execute_cached(input)?;
/// ```
///
/// ### 负载均衡
///
/// ```rust
/// use neotrix::core::nt_core_capability::*;
///
/// // 创建负载均衡器
/// let mut lb = LoadBalancer::new(LoadBalanceStrategy::LeastConnections);
///
/// // 添加实例
/// let instance = CapabilityInstance {
///     id: "inst_1".into(),
///     capability_id: "cap_1".into(),
///     weight: 1,
///     current_connections: 0,
///     max_connections: 100,
///     avg_response_time_ms: 0,
///     success_rate: 1.0,
///     total_calls: 0,
///     last_called: None,
///     status: InstanceStatus::Healthy,
/// };
/// lb.add_instance(instance);
///
/// // 选择实例
/// let selected = lb.select_instance("cap_1")?;
/// ```
///
/// ### 编排引擎
///
/// ```rust
/// use neotrix::core::nt_core_capability::*;
///
/// // 创建编排引擎
/// let registry = Arc::new(CapabilityRegistry::new());
/// let mut engine = OrchestrationEngine::new(registry);
///
/// // 创建编排流程
/// let flow = OrchestrationFlow {
///     id: "flow_1".into(),
///     name: "测试流程".into(),
///     mode: OrchestrationMode::Sequential,
///     steps: vec![
///         OrchestrationStep {
///             id: "step1".into(),
///             name: "步骤1".into(),
///             capability_id: "nt-world-nlp".into(),
///             input_mapping: "passthrough".into(),
///             output_mapping: "passthrough".into(),
///             condition: None,
///             max_retries: 3,
///             timeout_ms: 5000,
///         },
///     ],
///     global_timeout_ms: 30000,
///     max_parallelism: 1,
/// };
///
/// // 执行流程
/// let context = engine.execute_flow(&flow, input)?;
/// ```
///
/// ## 配置
///
/// ### 缓存配置
///
/// ```rust
/// pub struct CacheConfig {
///     pub max_capacity: usize,        // 最大容量
///     pub default_ttl: Duration,      // 默认过期时间
///     pub enable_stats: bool,         // 启用统计
/// }
/// ```
///
/// ### 发现配置
///
/// ```rust
/// pub struct DiscoveryConfig {
///     pub protocol: DiscoveryProtocol, // 发现协议
///     pub interval: Duration,          // 发现间隔
///     pub timeout: Duration,           // 超时时间
///     pub max_nodes: usize,            // 最大节点数
///     pub enable_encryption: bool,     // 启用加密
/// }
/// ```
///
/// ## 错误处理
///
/// ```rust
/// pub enum CapabilityError {
///     UnsupportedInput(String),        // 不支持的输入
///     ExecutionFailed(String),         // 执行失败
///     Timeout(String),                 // 超时
///     InternalError(String),           // 内部错误
/// }
/// ```
