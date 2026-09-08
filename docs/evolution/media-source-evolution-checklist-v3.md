# NeoTrix 媒体源进化清单 v3.0 — 深度集成与智能化

> **版本**: v3.0 | **日期**: 2026-09-08
> **前置**: Phase 1-16 全部完成 (30+ 新增文件)
> **目标**: 从独立模块 → 深度集成 + 实时能力 + 智能化

---

## 一、当前状态 (v2.0 ✅)

| 能力 | 状态 | 文件 |
|------|------|------|
| 29 源适配器 | ✅ | `audio/` `video/` `image/` `document/` `book/` `social/` |
| Feed 聚合 (RadarRSS) | ✅ | `feed/` (9 文件) |
| 歌词增强 | ✅ | `lyrics/timed.rs` `format.rs` `search.rs` |
| 视频增强 | ✅ | `video/metadata.rs` `stream.rs` `offline.rs` |
| 搜索增强 | ✅ | `engine/ranking.rs` `suggest.rs` `federated.rs` |
| 健康监控 | ✅ | `engine/health.rs` `ratelimit.rs` |
| 社交增强 | ✅ | `social/twitter_enhanced.rs` `telegram.rs` `rss_bridge.rs` |
| AI 集成 | ✅ | `ai/summarizer.rs` `sentiment.rs` `ner.rs` |
| 性能优化 | ✅ | `engine/pool.rs` `cache.rs` `concurrency.rs` |
| 插件系统 | ✅ | `plugin/mod.rs` `loader.rs` `musicfree_compat.rs` |
| 知识图谱 | ✅ | `graph/entity_graph.rs` `timeline.rs` `backlinks.rs` |
| CLI 命令 | ✅ | `cli/feed.rs` `lyrics.rs` |

---

## 二、Phase 17: 跨模块集成 (Cross-Module Integration)

### 17.1 MediaSource ↔ NT-MEMORY KB 桥接
**文件**: `integration/kb_bridge.rs` (新建)

```
┌─────────────────────────────────────────────────────────────┐
│                    MediaSource ←→ NT-MEMORY                 │
├─────────────────────────────────────────────────────────────┤
│  搜索结果 → KB 实体    │  KB 实体 → 相关媒体               │
│  歌词 → KB embeddings  │  实体关系 → 搜索优化               │
│  Feed 条目 → KB 节点   │  KB 图谱 → 推荐                   │
└─────────────────────────────────────────────────────────────┘
```

```rust
pub struct MediaKBBridge {
    kb: Arc<KnowledgeBase>,
    graph: EntityGraph,
    backlinks: BacklinkStore,
}

impl MediaKBBridge {
    /// 搜索结果 → KB 实体 (自动提取 + 写入)
    pub async fn index_search_results(&self, results: &[MediaItem]) -> Result<usize>;
    
    /// KB 实体 → 相关媒体 (反向查询)
    pub async fn find_related_media(&self, entity: &str, limit: usize) -> Vec<MediaItem>;
    
    /// Feed 条目 → KB 节点 + 实体提取
    pub async fn index_feed_entries(&self, entries: &[FeedEntry]) -> Result<usize>;
    
    /// 从 KB 图谱生成推荐
    pub async fn recommend(&self, media_id: &str, limit: usize) -> Vec<MediaItem>;
}
```

- [ ] 搜索结果自动写入 KB (实体 + 关系)
- [ ] KB 实体反向查询相关媒体
- [ ] Feed 条目自动索引
- [ ] 基于图谱的推荐

### 17.2 MediaSource ↔ NT-MIND 进化反馈
**文件**: `integration/evolution_feedback.rs` (新建)

```rust
pub struct MediaEvolutionFeedback {
    mind: Arc<dyn EvolutionEngine>,
}

impl MediaEvolutionFeedback {
    /// 源健康度 → 进化信号
    pub async fn report_source_health(&self, health: &SourceHealth) -> Result<()>;
    
    /// 搜索质量 → 技能树反馈
    pub async fn report_search_quality(&self, query: &str, results: &[MediaItem], clicks: &[String]) -> Result<()>;
    
    /// 用户偏好 → 个性化进化
    pub async fn learn_preferences(&self, user_id: &str, interactions: &[Interaction]) -> Result<()>;
}
```

- [ ] 源健康度写入进化信号
- [ ] 搜索点击反馈训练排名
- [ ] 用户偏好学习

### 17.3 MediaSource ↔ NT-SHIELD 安全审计
**文件**: `integration/security_audit.rs` (新建)

```rust
pub struct MediaSecurityAudit {
    shield: Arc<dyn SecurityModule>,
}

impl MediaSecurityAudit {
    /// API Key 安全扫描
    pub async fn scan_api_keys(&self) -> Vec<SecurityFinding>;
    
    /// URL 安全检查 (恶意链接检测)
    pub async fn check_url_safety(&self, url: &str) -> SafetyResult;
    
    /// 内容安全扫描 (NSFW/暴力/仇恨)
    pub async fn scan_content_safety(&self, content: &str) -> SafetyResult;
}
```

- [ ] API Key 硬编码扫描
- [ ] URL 安全检查
- [ ] 内容安全过滤

---

## 三、Phase 18: 实时能力 (Real-Time Features)

### 18.1 WebSocket 推送
**文件**: `realtime/ws_server.rs` (新建)

```rust
pub struct MediaWsServer {
    connections: RwLock<HashMap<String, WsConnection>>,
    broadcast: broadcast::Sender<WsMessage>,
}

pub enum WsMessage {
    SearchUpdate { query: String, results: Vec<MediaItem> },
    HealthAlert { source: String, status: HealthStatus },
    FeedUpdate { feed_url: String, new_entries: usize },
    PlaybackEvent { event: PlaybackEvent },
    LyricSync { position_ms: u64, line: TimedLine },
}

impl MediaWsServer {
    pub async fn start(&self, addr: &str) -> Result<()>;
    pub async fn broadcast(&self, msg: WsMessage) -> Result<()>;
    pub async fn subscribe(&self, client_id: &str) -> broadcast::Receiver<WsMessage>;
}
```

- [ ] WebSocket 服务器 (tokio-tungstenite)
- [ ] 消息广播
- [ ] 客户端订阅/取消订阅
- [ ] 心跳保活

### 18.2 Server-Sent Events (SSE)
**文件**: `realtime/sse_server.rs` (新建)

```rust
pub struct MediaSseServer {
    clients: RwLock<HashMap<String, mpsc::Sender<SseEvent>>>,
}

pub enum SseEvent {
    SearchComplete { query: String, count: usize },
    SourceHealthChanged { source: String, status: String },
    FeedNewEntries { feed_url: String, count: usize },
    DailyBriefingReady { date: String },
}

impl MediaSseServer {
    pub async fn start(&self, addr: &str) -> Result<()>;
    pub fn client_stream(&self, client_id: &str) -> SseStream;
    pub async fn emit(&self, event: SseEvent) -> Result<()>;
}
```

- [ ] SSE 服务器 (axum)
- [ ] 事件流
- [ ] 客户端管理

### 18.3 直播流监控
**文件**: `realtime/live_monitor.rs` (新建)

```rust
pub struct LiveStreamMonitor {
    streams: HashMap<String, LiveStream>,
}

pub struct LiveStream {
    pub url: String,
    pub platform: String,
    pub status: LiveStatus,
    pub viewer_count: Option<u64>,
    pub started_at: Option<i64>,
}

impl LiveStreamMonitor {
    pub async fn add_stream(&mut self, url: &str) -> Result<()>;
    pub async fn check_status(&self, stream_id: &str) -> LiveStatus;
    pub async fn notify_start(&self, stream_id: &str) -> Result<()>;
    pub async fn get_chat(&self, stream_id: &str) -> Vec<ChatMessage>;
}
```

- [ ] 直播状态检测
- [ ] 开播通知
- [ ] 弹幕/聊天抓取

---

## 四、Phase 19: 数据管道 (Data Pipeline)

### 19.1 ETL 管道
**文件**: `pipeline/etl.rs` (新建)

```rust
pub struct MediaETL {
    extractors: Vec<Box<dyn Extractor>>,
    transformers: Vec<Box<dyn Transformer>>,
    loaders: Vec<Box<dyn Loader>>,
}

pub trait Extractor: Send + Sync {
    async fn extract(&self) -> Result<Vec<RawData>>;
}

pub trait Transformer: Send + Sync {
    fn transform(&self, data: Vec<RawData>) -> Result<Vec<TransformedData>>;
}

pub trait Loader: Send + Sync {
    async fn load(&self, data: Vec<TransformedData>) -> Result<usize>;
}

impl MediaETL {
    pub async fn run_pipeline(&self) -> Result<PipelineResult>;
}
```

- [ ] 可插拔提取器/转换器/加载器
- [ ] 管道编排
- [ ] 错误重试 + 死信队列

### 19.2 数据质量检查
**文件**: `pipeline/quality.rs` (新建)

```rust
pub struct DataQualityChecker {
    rules: Vec<QualityRule>,
}

pub enum QualityRule {
    SchemaValidation { schema: JsonSchema },
    CompletenessCheck { required_fields: Vec<String> },
    FreshnessCheck { max_age_hours: u64 },
    DeduplicationCheck { key_fields: Vec<String> },
    RangeCheck { field: String, min: f64, max: f64 },
}

impl DataQualityChecker {
    pub fn check(&self, data: &[MediaItem]) -> QualityReport;
}
```

- [ ] 数据质量规则引擎
- [ ] 质量报告生成
- [ ] 自动修复建议

### 19.3 数据血缘追踪
**文件**: `pipeline/lineage.rs` (新建)

```rust
pub struct DataLineage {
    nodes: Vec<LineageNode>,
    edges: Vec<LineageEdge>,
}

pub struct LineageNode {
    pub id: String,
    pub kind: NodeKind,  // Source, Transform, Load
    pub name: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, String>,
}

impl DataLineage {
    pub fn record_extract(&mut self, source: &str, count: usize);
    pub fn record_transform(&mut self, transform: &str, input: usize, output: usize);
    pub fn record_load(&mut self, target: &str, count: usize);
    pub fn to_mermaid(&self) -> String;
    pub fn to_dot(&self) -> String;
}
```

- [ ] 血缘节点/边记录
- [ ] Mermaid/DOT 可视化
- [ ] 影响分析

---

## 五、Phase 20: 高级缓存 (Advanced Caching)

### 20.1 多级缓存
**文件**: `cache/multi_level.rs` (新建)

```rust
pub struct MultiLevelCache {
    l1_memory: LruCache<String, CacheEntry>,    // 热数据 (1000 entries)
    l2_disk: DiskCache,                          // 温数据 (10GB)
    l3_network: Option<RedisCache>,              // 冷数据 (可选)
}

impl MultiLevelCache {
    pub async fn get(&self, key: &str) -> Option<CacheEntry>;
    pub async fn insert(&self, key: &str, entry: CacheEntry);
    pub async fn evict(&self, strategy: EvictionStrategy);
    pub fn stats(&self) -> CacheStats;
}

pub enum EvictionStrategy {
    Lru,
    Lfu,
    Ttl,
    SizeBased,
}
```

- [ ] 三级缓存架构
- [ ] 多种淘汰策略
- [ ] 缓存命中率统计

### 20.2 缓存预热
**文件**: `cache/warmup.rs` (新建)

```rust
pub struct CacheWarmer {
    popular_queries: Vec<String>,
    popular_feeds: Vec<String>,
}

impl CacheWarmer {
    pub async fn warm_search_cache(&self, engine: &MediaEngine);
    pub async fn warm_feed_cache(&self, feed_engine: &FeedEngine);
    pub async fn predict_and_warm(&self, user_id: &str);
}
```

- [ ] 热门查询预热
- [ ] 热门 Feed 预热
- [ ] 预测性预热

### 20.3 缓存一致性
**文件**: `cache/consistency.rs` (新建)

```rust
pub struct CacheConsistency {
    invalidation_rules: Vec<InvalidationRule>,
}

pub enum InvalidationRule {
    TimeBased { ttl_secs: u64 },
    EventBased { event_type: String },
    VersionBased { version_field: String },
}

impl CacheConsistency {
    pub fn register_rule(&mut self, rule: InvalidationRule);
    pub async fn invalidate(&self, key: &str, reason: &str);
    pub async fn sync(&self) -> Result<SyncResult>;
}
```

- [ ] 失效规则引擎
- [ ] 事件驱动失效
- [ ] 缓存同步

---

## 六、Phase 21: 高级 AI (Advanced AI)

### 21.1 嵌入向量搜索
**文件**: `ai/embedding_search.rs` (新建)

```rust
pub struct EmbeddingSearch {
    index: HnswIndex,
    model: Box<dyn EmbeddingModel>,
}

impl EmbeddingSearch {
    pub async fn index_media(&self, items: &[MediaItem]) -> Result<usize>;
    pub async fn semantic_search(&self, query: &str, limit: usize) -> Vec<ScoredItem>;
    pub async fn find_similar(&self, media_id: &str, limit: usize) -> Vec<MediaItem>;
}
```

- [ ] 嵌入向量索引 (HNSW)
- [ ] 语义搜索
- [ ] 相似媒体发现

### 21.2 RAG 增强搜索
**文件**: `ai/rag_search.rs` (新建)

```rust
pub struct RagSearch {
    retriever: Box<dyn Retriever>,
    generator: Box<dyn Generator>,
   reranker: Box<dyn Reranker>,
}

impl RagSearch {
    pub async fn search(&self, query: &str) -> RagResult {
        // 1. 检索相关文档
        let docs = self.retriever.retrieve(query).await?;
        // 2. 生成回答
        let answer = self.generator.generate(query, &docs).await?;
        // 3. 重排序
        let ranked = self.reranker.rerank(query, &docs).await?;
        Ok(RagResult { answer, sources: ranked })
    }
}
```

- [ ] 检索增强生成
- [ ] 多阶段检索
- [ ] 引用溯源

### 21.3 多模态理解
**文件**: `ai/multimodal.rs` (新建)

```rust
pub struct MultimodalUnderstanding {
    image_model: Box<dyn ImageModel>,
    audio_model: Box<dyn AudioModel>,
    video_model: Box<dyn VideoModel>,
}

impl MultimodalUnderstanding {
    pub async fn understand_image(&self, url: &str) -> ImageAnalysis;
    pub async fn understand_audio(&self, url: &str) -> AudioAnalysis;
    pub async fn understand_video(&self, url: &str) -> VideoAnalysis;
    pub async fn cross_modal_search(&self, query: &str) -> Vec<MediaItem>;
}
```

- [ ] 图像理解 (描述/标签/OCR)
- [ ] 音频理解 (语音/音乐分类)
- [ ] 视频理解 (场景/动作/摘要)
- [ ] 跨模态搜索

---

## 七、Phase 22: API 网关 (API Gateway)

### 22.1 RESTful API
**文件**: `api/rest.rs` (新建)

```rust
pub struct MediaRestApi {
    engine: Arc<MediaEngine>,
    auth: Arc<AuthMiddleware>,
}

impl MediaRestApi {
    pub fn routes(&self) -> Router {
        Router::new()
            .route("/api/v1/search", get(self.search))
            .route("/api/v1/play/:id", get(self.play_url))
            .route("/api/v1/lyrics/:id", get(self.lyrics))
            .route("/api/v1/feeds", get(self.list_feeds).post(self.add_feed))
            .route("/api/v1/feeds/:url/fetch", get(self.fetch_feed))
            .route("/api/v1/sources", get(self.sources))
            .route("/api/v1/health", get(self.health))
            .route("/api/v1/briefing", get(self.daily_briefing))
    }
}
```

- [ ] RESTful API (axum)
- [ ] JWT 认证
- [ ] 速率限制
- [ ] API 文档 (OpenAPI)

### 22.2 GraphQL API
**文件**: `api/graphql.rs` (新建)

```rust
pub struct MediaGraphQL {
    schema: Schema<QueryRoot, MutationRoot, EmptySubscription>,
}

#[derive(GraphQLObject)]
pub struct MediaItemGql {
    pub id: String,
    pub title: String,
    pub source: String,
    pub media_type: String,
    pub url: String,
}

impl QueryRoot {
    async fn search(&self, query: String, source: Option<String>) -> Vec<MediaItemGql>;
    async fn feed(&self, url: String) -> FeedEntryGql;
    async fn health(&self) -> Vec<SourceHealthGql>;
}
```

- [ ] GraphQL schema (juniper)
- [ ] 查询/变更/订阅
- [ ] DataLoader 批量加载

### 22.3 gRPC API
**文件**: `api/grpc.rs` (新建)

```protobuf
service MediaService {
    rpc Search(SearchRequest) returns (SearchResponse);
    rpc GetPlayUrl(PlayUrlRequest) returns (PlayUrlResponse);
    rpc GetLyrics(LyricsRequest) returns (LyricsResponse);
    rpc StreamUpdates(StreamRequest) returns (stream MediaEvent);
}
```

- [ ] gRPC 服务定义
- [ ] tonic 实现
- [ ] 双向流

---

## 八、Phase 23: 测试基础设施 (Testing Infrastructure)

### 23.1 集成测试
**文件**: `tests/integration/mod.rs` (新建)

```rust
#[tokio::test]
async fn test_search_fallback() {
    let engine = build_test_engine();
    let result = engine.search("周杰伦").await;
    assert!(!result.items.is_empty());
}

#[tokio::test]
async fn test_feed_aggregation() {
    let alerter = FeedAlerter::new();
    let state = alerter.build_alert_state().await.unwrap();
    assert!(state.total_entries > 0);
}
```

- [ ] 源集成测试 (带 mock)
- [ ] Feed 聚合测试
- [ ] 搜索排名测试

### 23.2 性能基准测试
**文件**: `tests/benchmark/mod.rs` (新建)

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_search(c: &mut Criterion) {
    c.bench_function("search_10_sources", |b| {
        b.iter(|| runtime.block_on(search_10_sources()))
    });
}

fn bench_dedup(c: &mut Criterion) {
    c.bench_function("dedup_1000_entries", |b| {
        b.iter(|| dedup_1000_entries())
    });
}
```

- [ ] 搜索性能基准
- [ ] 去重性能基准
- [ ] 缓存命中率基准

### 23.3 模糊测试
**文件**: `tests/fuzz/mod.rs` (新建)

```rust
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = FeedParser::parse_rss(s);
        let _ = TextUtils::tokenize(s);
        let _ = DedupEngine::normalize_url(s);
    }
});
```

- [ ] Feed 解析模糊测试
- [ ] 文本处理模糊测试
- [ ] URL 归一化模糊测试

---

## 九、Phase 24: 可观测性 (Observability)

### 24.1 指标收集
**文件**: `observability/metrics.rs` (新建)

```rust
pub struct MediaMetrics {
    search_counter: IntCounter,
    search_latency: Histogram,
    source_health_gauge: IntGauge,
    cache_hit_ratio: Gauge,
    feed_update_counter: IntCounter,
}

impl MediaMetrics {
    pub fn record_search(&self, source: &str, latency_ms: u64, success: bool);
    pub fn record_source_health(&self, source: &str, health: f64);
    pub fn record_cache_hit(&self, hit: bool);
    pub fn record_feed_update(&self, feed_url: &str, count: usize);
}
```

- [ ] Prometheus 指标
- [ ] 自定义指标
- [ ] Grafana 仪表盘

### 24.2 分布式追踪
**文件**: `observability/tracing.rs` (新建)

```rust
pub struct MediaTracer {
    tracer: Tracer,
}

impl MediaTracer {
    pub fn trace_search(&self, query: &str) -> Span;
    pub fn trace_source_call(&self, source: &str) -> Span;
    pub fn trace_feed_fetch(&self, url: &str) -> Span;
}
```

- [ ] OpenTelemetry 追踪
- [ ] Span 创建
- [ ] 追踪导出

### 24.3 结构化日志
**文件**: `observability/logging.rs` (新建)

```rust
pub struct MediaLogger {
    logger: Logger,
}

impl MediaLogger {
    pub fn log_search(&self, query: &str, results: usize, latency_ms: u64);
    pub fn log_source_error(&self, source: &str, error: &MediaError);
    pub fn log_feed_update(&self, url: &str, new_entries: usize);
}
```

- [ ] 结构化日志 (tracing)
- [ ] 日志级别控制
- [ ] 日志聚合

---

## 十、Phase 25: 文档与示例 (Documentation)

### 25.1 API 文档
**文件**: `docs/api/openapi.yaml` (新建)

```yaml
openapi: 3.0.3
info:
  title: NeoTrix Media API
  version: 1.0.0
paths:
  /api/v1/search:
    get:
      summary: Search media
      parameters:
        - name: q
          in: query
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Search results
```

- [ ] OpenAPI 规范
- [ ] API 文档生成
- [ ] 示例请求/响应

### 25.2 架构文档
**文件**: `docs/architecture/media-source.md` (新建)

```markdown
# Media Source Architecture

## 组件图

┌─────────────┐
│   MediaApi  │
└──────┬──────┘
       │
┌──────▼──────┐
│ MediaEngine │
└──────┬──────┘
       │
┌──────▼──────┐
│  Source Pool │
└──────┬──────┘
       │
┌──────▼──────┐
│  Individual │
│   Sources   │
└─────────────┘
```

- [ ] 架构概览
- [ ] 数据流图
- [ ] 部署指南

### 25.3 使用示例
**文件**: `examples/search.rs` (新建)

```rust
use neotrix::media_source::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = media_api();
    
    // 搜索音乐
    let results = api.search("周杰伦 晴天", Some("audio"), 1).await?;
    println!("Found {} results", results.items.len());
    
    // 获取播放 URL
    if let Some(item) = results.items.first() {
        let play_url = api.play_url(item, Quality::Flac).await?;
        println!("Play: {}", play_url.url);
    }
    
    // 获取歌词
    if let Some(item) = results.items.first() {
        let lyrics = api.lyric(item).await?;
        println!("Lyrics: {}", lyrics.content);
    }
    
    Ok(())
}
```

- [ ] 搜索示例
- [ ] Feed 聚合示例
- [ ] 歌词同步示例

---

## 十一、Phase 26: 安全加固 (Security Hardening)

### 26.1 输入验证
**文件**: `security/input_validation.rs` (新建)

```rust
pub struct InputValidator;

impl InputValidator {
    pub fn validate_search_query(query: &str) -> Result<String, ValidationError>;
    pub fn validate_url(url: &str) -> Result<String, ValidationError>;
    pub fn validate_feed_url(url: &str) -> Result<String, ValidationError>;
    pub fn sanitize_html(html: &str) -> String;
}
```

- [ ] 查询注入防护
- [ ] URL 验证
- [ ] HTML 清理

### 26.2 速率限制
**文件**: `security/rate_limit.rs` (新建)

```rust
pub struct ApiRateLimiter {
    global: TokenBucket,
    per_user: HashMap<String, TokenBucket>,
    per_ip: HashMap<String, TokenBucket>,
}

impl ApiRateLimiter {
    pub async fn check(&self, user_id: &str, ip: &str) -> Result<(), RateLimitError>;
    pub fn configure(&mut self, config: RateLimitConfig);
}
```

- [ ] 全局限制
- [ ] 每用户限制
- [ ] 每 IP 限制

### 26.3 审计日志
**文件**: `security/audit.rs` (新建)

```rust
pub struct AuditLogger {
    log_file: File,
}

impl AuditLogger {
    pub fn log_search(&self, user_id: &str, query: &str, results: usize);
    pub fn log_play(&self, user_id: &str, media_id: &str);
    pub fn log_admin_action(&self, admin_id: &str, action: &str);
}
```

- [ ] 操作审计
- [ ] 异常检测
- [ ] 合规报告

---

## 十二、执行优先级

### P0 (核心)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 17 跨模块集成 | **Critical** | 5 天 | Phase 1-16 |
| Phase 21 高级 AI | **Critical** | 6 天 | Phase 12 |
| Phase 23 测试基础设施 | **Critical** | 4 天 | Phase 1-16 |

### P1 (功能)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 18 实时能力 | **High** | 5 天 | 无 |
| Phase 22 API 网关 | **High** | 6 天 | Phase 17 |
| Phase 24 可观测性 | **High** | 3 天 | 无 |

### P2 (增强)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 19 数据管道 | **Medium** | 5 天 | Phase 17 |
| Phase 20 高级缓存 | **Medium** | 4 天 | Phase 15 |
| Phase 25 文档示例 | **Medium** | 3 天 | Phase 22 |

### P3 (安全)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 26 安全加固 | **Low** | 4 天 | Phase 22 |

---

## 十三、验收标准

### Phase 17 跨模块集成
- [ ] 搜索结果自动写入 KB
- [ ] KB 实体反向查询正常
- [ ] Feed 条目自动索引
- [ ] 推荐结果相关性 > 0.6

### Phase 18 实时能力
- [ ] WebSocket 连接数 > 1000
- [ ] 消息延迟 < 100ms
- [ ] SSE 事件流正常
- [ ] 直播状态检测准确率 > 90%

### Phase 19 数据管道
- [ ] ETL 管道吞吐 > 1000 items/s
- [ ] 数据质量检查覆盖 > 90%
- [ ] 血缘追踪可视化正常

### Phase 20 高级缓存
- [ ] 多级缓存命中率 > 85%
- [ ] 缓存预热减少冷启动 > 50%
- [ ] 缓存一致性延迟 < 5s

### Phase 21 高级 AI
- [ ] 语义搜索准确率 > 0.7
- [ ] RAG 回答相关性 > 0.8
- [ ] 多模态理解延迟 < 2s

### Phase 22 API 网关
- [ ] RESTful API 响应 < 200ms
- [ ] GraphQL 查询延迟 < 300ms
- [ ] gRPC 流延迟 < 50ms

### Phase 23 测试基础设施
- [ ] 集成测试覆盖率 > 80%
- [ ] 性能基准测试通过
- [ ] 模糊测试发现 0 panic

### Phase 24 可观测性
- [ ] Prometheus 指标 100% 覆盖
- [ ] 分布式追踪正常
- [ ] 结构化日志 100% 覆盖

### Phase 25 文档示例
- [ ] API 文档 100% 覆盖
- [ ] 架构文档完整
- [ ] 示例代码可运行

### Phase 26 安全加固
- [ ] 输入验证 100% 覆盖
- [ ] 速率限制正常
- [ ] 审计日志完整

---

## 十四、文件结构预览

```
nt_world_media_source/
├── integration/
│   ├── mod.rs
│   ├── kb_bridge.rs           # [NEW] KB 桥接
│   ├── evolution_feedback.rs  # [NEW] 进化反馈
│   └── security_audit.rs      # [NEW] 安全审计
├── realtime/
│   ├── mod.rs
│   ├── ws_server.rs           # [NEW] WebSocket
│   ├── sse_server.rs          # [NEW] SSE
│   └── live_monitor.rs        # [NEW] 直播监控
├── pipeline/
│   ├── mod.rs
│   ├── etl.rs                 # [NEW] ETL 管道
│   ├── quality.rs             # [NEW] 数据质量
│   └── lineage.rs             # [NEW] 数据血缘
├── cache/
│   ├── mod.rs
│   ├── multi_level.rs         # [NEW] 多级缓存
│   ├── warmup.rs              # [NEW] 缓存预热
│   └── consistency.rs         # [NEW] 缓存一致性
├── api/
│   ├── mod.rs
│   ├── rest.rs                # [NEW] RESTful API
│   ├── graphql.rs             # [NEW] GraphQL
│   └── grpc.rs                # [NEW] gRPC
├── observability/
│   ├── mod.rs
│   ├── metrics.rs             # [NEW] 指标
│   ├── tracing.rs             # [NEW] 追踪
│   └── logging.rs             # [NEW] 日志
├── security/
│   ├── mod.rs
│   ├── input_validation.rs    # [NEW] 输入验证
│   ├── rate_limit.rs          # [NEW] 速率限制
│   └── audit.rs               # [NEW] 审计日志
├── ai/
│   ├── embedding_search.rs    # [NEW] 嵌入搜索
│   ├── rag_search.rs          # [NEW] RAG 搜索
│   └── multimodal.rs          # [NEW] 多模态
├── tests/
│   ├── integration/mod.rs     # [NEW] 集成测试
│   ├── benchmark/mod.rs       # [NEW] 性能基准
│   └── fuzz/mod.rs            # [NEW] 模糊测试
├── docs/
│   ├── api/openapi.yaml       # [NEW] API 文档
│   └── architecture/media-source.md  # [NEW] 架构文档
└── examples/
    ├── search.rs              # [NEW] 搜索示例
    ├── feed.rs                # [NEW] Feed 示例
    └── lyrics.rs              # [NEW] 歌词示例
```

---

## 十五、技术债清理

| 项目 | 优先级 | 说明 |
|------|--------|------|
| 统一错误类型 | High | `MediaError` → `nt_core_error` 统一 |
| API Key 外部化 | High | 环境变量 / KB 配置 |
| 类型导出规范化 | Medium | 统一 `pub use` 模式 |
| 文档补全 | Medium | 每个 pub fn 加 `///` |
| 测试覆盖率 | High | 目标 > 80% |
