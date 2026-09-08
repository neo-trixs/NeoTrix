# NeoTrix 媒体源迭代进化清单

> **版本**: v2.0 | **日期**: 2026-09-08
> **目标**: 从 29 源 + Feed 聚合 → 全能媒体能力网

---

## 一、当前状态 (v1.0 ✅)

| 能力 | 状态 | 文件 |
|------|------|------|
| 29 源适配器 | ✅ | `audio/` `video/` `image/` `document/` `book/` `social/` `lyrics/` |
| 8 级音质体系 | ✅ | `types.rs` Quality enum |
| 多源聚合引擎 | ✅ | `engine.rs` MediaEngine |
| Feed 聚合 (RadarRSS) | ✅ | `feed/` (9 文件) |
| 播放控制器 | ✅ | `playback.rs` |
| Now Playing | ✅ | `now_playing.rs` |
| KB 持久化 | ✅ | `resource_store.rs` |
| LX 脚本适配 | ✅ | `lx_script.rs` |

---

## 二、Phase 7: 歌词增强 (Lyrics Enhance)

### 7.1 时间轴歌词同步
**文件**: `lyrics/timed.rs` (新建)

```
┌─────────────────────────────────────────────────────┐
│  LRC 解析 → 时间轴排序 → 逐行同步 → 滚动高亮      │
├─────────────────────────────────────────────────────┤
│  [00:12.34] 第一行歌词                              │
│  [00:15.67] 第二行歌词                              │
│  [00:18.90] 第三行歌词                              │
└─────────────────────────────────────────────────────┘
```

- [ ] LRC 格式解析 (标准 + 扩展标签 `[ar:`, `[ti:`, `[by:`)
- [ ] 纯净时间戳提取 `[mm:ss.xx]`
- [ ] 逐行时间轴排序
- [ ] `sync(position_ms) -> (current_line, progress)`
- [ ] 支持翻译歌词双行显示
- [ ] 支持 Ruby 注音 (日文/粤语)

### 7.2 多格式歌词支持
**文件**: `lyrics/format.rs` (新建)

| 格式 | 来源 | 解析器 |
|------|------|--------|
| LRC | LRCLib, 网易云 | `parse_lrc()` |
| QRC | QQ 音乐 | `parse_qrc()` |
| KRC | 酷狗 | `parse_krc()` |
| TTML | Apple Music | `parse_ttml()` |
| JSON | Spotify | `parse_spotify_json()` |

- [ ] 统一 `TimedLine` 结构
- [ ] 格式自动检测
- [ ] 降级链: QRC/KRC → LRC → 纯文本

### 7.3 歌词搜索引擎
**文件**: `lyrics/search.rs` (新建)

```rust
pub struct LyricsSearcher {
    providers: Vec<Box<dyn LyricsProvider>>,
    cache: HashMap<String, CachedLyrics>,
}

impl LyricsSearcher {
    /// 多源并发搜索 + 聚合
    pub async fn search(&self, query: &str) -> Vec<LyricsResult>;
    
    /// 按质量排序 (时间轴完整度 > 来源权威性 > 匹配度)
    pub fn rank(results: Vec<LyricsResult>) -> Vec<LyricsResult>;
}
```

- [ ] 3 源并发 (LRCLib + Genius + NetEase)
- [ ] 匹配算法: 标题+艺术家 fuzzy match
- [ ] 质量评分: 时间轴覆盖率 + 行数 + 来源权重
- [ ] 缓存策略: 24h TTL, 歌曲 ID 为 key

---

## 三、Phase 8: 视频能力增强 (Video Enhance)

### 8.1 视频信息提取
**文件**: `video/metadata.rs` (新建)

```rust
pub struct VideoMetadata {
    pub duration_ms: u64,
    pub resolution: (u32, u32),
    pub fps: f64,
    pub codec: String,
    pub bitrate: u64,
    pub chapters: Vec<Chapter>,
}
```

- [ ] 从 yt-dlp 提取完整元数据
- [ ] 章节信息解析
- [ ] 字幕轨道列表

### 8.2 视频流选择
**文件**: `video/stream.rs` (新建)

```rust
pub struct StreamSelector {
    pub fn select_best(video: &VideoMetadata, quality: Quality) -> Stream;
    pub fn select_audio(video: &VideoMetadata, lang: &str) -> Stream;
    pub fn merge(video: Stream, audio: Stream) -> MuxCommand;
}
```

- [ ] 画质降级链: 4K → 1080p → 720p → 480p
- [ ] 音频流选择 (语言偏好)
- [ ] 字幕嵌入命令生成

### 8.3 离线缓存
**文件**: `video/offline.rs` (新建)

```rust
pub struct OfflineCache {
    dir: PathBuf,
    max_size_gb: f64,
}

impl OfflineCache {
    pub async fn download(&self, url: &str, quality: Quality) -> PathBuf;
    pub async fn get(&self, id: &str) -> Option<PathBuf>;
    pub async fn cleanup(&self);  // LRU 淘汰
}
```

- [ ] 分块下载 (tokio::io::copy)
- [ ] 进度回调
- [ ] LRU 淘汰策略
- [ ] 存储配额管理

---

## 四、Phase 9: 搜索增强 (Search Enhance)

### 9.1 搜索排名优化
**文件**: `engine/ranking.rs` (新建)

```rust
pub struct SearchRanker {
    /// BM25 文本相关性
    pub fn bm25_score(query: &str, doc: &MediaItem) -> f64;
    
    /// 来源权威性
    pub fn source_authority(source: &str) -> f64;
    
    /// 时效性衰减
    pub fn freshness_score(published: i64) -> f64;
    
    /// 综合排名
    pub fn rank(query: &str, items: Vec<MediaItem>) -> Vec<ScoredItem>;
}
```

- [ ] BM25 实现 (k1=1.5, b=0.75)
- [ ] 来源权重表 (官方 API > 爬虫 > 缓存)
- [ ] 时效性衰减 (24h 半衰期)
- [ ] 多维融合评分

### 9.2 搜索建议
**文件**: `engine/suggest.rs` (新建)

```rust
pub struct SearchSuggester {
    history: Vec<String>,
    trending: HashMap<String, u64>,
}

impl SearchSuggester {
    /// 前缀匹配 + 热度排序
    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<String>;
    
    /// 纠错 (编辑距离 <= 2)
    pub fn correct(&self, query: &str) -> Option<String>;
}
```

- [ ] Trie 前缀树
- [ ] 搜索历史统计
- [ ] 热词趋势追踪
- [ ] 拼写纠错 (Levenshtein)

### 9.3 联合搜索
**文件**: `engine/federated.rs` (新建)

```rust
pub struct FederatedSearch {
    /// 并发搜索多个源
    pub async fn search(&self, query: &str) -> Vec<MediaItem>;
    
    /// 去重 + 合并
    pub fn merge(results: Vec<Vec<MediaItem>>) -> Vec<MediaItem>;
}
```

- [ ] 并发 fan-out (tokio::join!)
- [ ] 结果去重 (URL + 标题 hash)
- [ ] 超时降级 (单源超时不阻塞)
- [ ] 负载均衡 (轮询 + 健康度)

---

## 五、Phase 10: 源健康监控 (Source Health)

### 10.1 健康检查器
**文件**: `engine/health.rs` (新建)

```rust
pub struct SourceHealth {
    pub source: String,
    pub status: HealthStatus,       // Healthy / Degraded / Down
    pub latency_ms: u64,
    pub success_rate: f64,          // 0.0 ~ 1.0
    pub last_check: i64,
    pub error_count: u64,
}

pub struct HealthChecker {
    sources: HashMap<String, SourceHealth>,
    check_interval_secs: u64,
}

impl HealthChecker {
    /// 单源探测
    pub async fn probe(&self, source: &str) -> SourceHealth;
    
    /// 全源扫描
    pub async fn scan_all(&self) -> Vec<SourceHealth>;
    
    /// 自动降级 (连续 3 次失败 → 标记 Down)
    pub fn auto_degrade(&mut self);
}
```

- [ ] 轻量探测 (搜索 "test" + 超时 5s)
- [ ] 成功率滑动窗口 (100 次采样)
- [ ] 自动降级/恢复
- [ ] 健康度报告

### 10.2 速率限制器
**文件**: `engine/ratelimit.rs` (新建)

```rust
pub struct RateLimiter {
    limits: HashMap<String, TokenBucket>,
}

impl RateLimiter {
    /// 获取令牌 (阻塞等待)
    pub async fn acquire(&self, source: &str);
    
    /// 尝试获取 (非阻塞)
    pub fn try_acquire(&self, source: &str) -> bool;
}
```

- [ ] 令牌桶算法
- [ ] 每源独立限制
- [ ] 429 响应自动退避
- [ ] 指数退避重试

---

## 六、Phase 11: 社交媒体增强 (Social Enhance)

### 11.1 Twitter/X 增强
**文件**: `social/twitter_enhanced.rs` (新建)

```rust
pub struct TwitterEnhanced {
    auth_token: Option<String>,
    guest_token: Option<String>,
}

impl TwitterEnhanced {
    /// Cookie-based 认证
    pub async fn authenticate(&mut self, cookie: &str) -> Result<()>;
    
    /// 搜索推文
    pub async fn search(&self, query: &str) -> Vec<Tweet>;
    
    /// 获取用户时间线
    pub async fn timeline(&self, user: &str) -> Vec<Tweet>;
}
```

- [ ] Guest token 获取
- [ ] Cookie 认证
- [ ] 推文搜索 API
- [ ] 用户时间线

### 11.2 Telegram 频道监控
**文件**: `social/telegram.rs` (新建)

```rust
pub struct TelegramSource {
    pub async fn get_messages(channel: &str, limit: u32) -> Vec<Message>;
    pub async fn search(channel: &str, query: &str) -> Vec<Message>;
}
```

- [ ] 公开频道消息获取
- [ ] 媒体文件提取
- [ ] 消息搜索

### 11.3 RSS 社交媒体桥接
**文件**: `social/rss_bridge.rs` (新建)

```rust
/// 将社交媒体转化为 RSS feed
pub struct SocialRssBridge {
    pub fn twitter_to_rss(user: &str) -> String;
    pub fn telegram_to_rss(channel: &str) -> String;
    pub fn github_to_rss(repo: &str) -> String;
}
```

- [ ] Twitter → RSS (via rsshub)
- [ ] Telegram → RSS (via tgram)
- [ ] GitHub → RSS (releases/commits)

---

## 七、Phase 12: AI 能力集成 (AI Integration)

### 12.1 内容摘要
**文件**: `ai/summarizer.rs` (新建)

```rust
pub struct ContentSummarizer {
    llm: Arc<dyn LlmProvider>,
}

impl ContentSummarizer {
    /// 单篇摘要
    pub async fn summarize(&self, content: &str) -> String;
    
    /// 批量摘要 (每日 briefing)
    pub async fn daily_briefing(&self, entries: &[FeedEntry]) -> String;
    
    /// 要点提取
    pub async fn extract_key_points(&self, content: &str) -> Vec<String>;
}
```

- [ ] 调用 nt_io LLM 接口
- [ ] 300 字摘要模板
- [ ] 每日 briefing 生成
- [ ] 关键点提取

### 12.2 情感分析
**文件**: `ai/sentiment.rs` (新建)

```rust
pub struct SentimentAnalyzer {
    /// 关键词规则 + LLM 辅助
    pub fn analyze(&self, text: &str) -> Sentiment;
}

pub enum Sentiment { Positive, Negative, Neutral, Mixed }
```

- [ ] 关键词词典 (正面/负面)
- [ ] LLM 辅助判断
- [ ] 新闻情感倾向

### 12.3 实体识别
**文件**: `ai/ner.rs` (新建)

```rust
pub struct EntityRecognizer {
    /// 识别实体: 人名/公司/产品/地点
    pub fn recognize(&self, text: &str) -> Vec<Entity>;
}

pub struct Entity {
    pub text: String,
    pub kind: EntityKind,  // Person, Company, Product, Location
    pub confidence: f64,
}
```

- [ ] 正则 + 词典基础识别
- [ ] LLM 辅助消歧
- [ ] 实体链接到 KB

---

## 八、Phase 13: 插件系统 (Plugin System)

### 13.1 插件 trait
**文件**: `plugin/mod.rs` (新建)

```rust
#[async_trait]
pub trait MediaPlugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    
    /// 版本
    fn version(&self) -> &str;
    
    /// 支持的媒体类型
    fn supported_types(&self) -> Vec<MediaType>;
    
    /// 搜索
    async fn search(&self, query: &str) -> Result<Vec<MediaItem>, MediaError>;
    
    /// 获取播放 URL
    async fn get_play_url(&self, id: &str, quality: Quality) -> Result<String, MediaError>;
}
```

### 13.2 插件加载器
**文件**: `plugin/loader.rs` (新建)

```rust
pub struct PluginLoader {
    plugins_dir: PathBuf,
    loaded: HashMap<String, Box<dyn MediaPlugin>>,
}

impl PluginLoader {
    /// 从目录加载插件
    pub fn load_all(&mut self) -> Result<()>;
    
    /// 动态加载单个插件
    pub fn load(&mut self, path: &Path) -> Result<Box<dyn MediaPlugin>>;
}
```

- [ ] 目录扫描
- [ ] 插件清单解析 (manifest.json)
- [ ] 依赖检查
- [ ] 热加载

### 13.3 MusicFree 兼容层
**文件**: `plugin/musicfree_compat.rs` (新建)

```rust
/// 兼容 MusicFree 插件格式
pub struct MusicFreeAdapter {
    script: Box<dyn ScriptRuntime>,  // quickjs/boa
}

impl MediaPlugin for MusicFreeAdapter {
    /// 执行 JS 插件搜索
    async fn search(&self, query: &str) -> Result<Vec<MediaItem>, MediaError> {
        // 调用插件的 search() 函数
    }
}
```

- [ ] JS 运行时集成 (boa-engine)
- [ ] 插件 API 桥接
- [ ] 安全沙箱

---

## 九、Phase 14: 知识图谱 (Knowledge Graph)

### 14.1 实体关系图
**文件**: `graph/entity_graph.rs` (新建)

```rust
pub struct EntityGraph {
    entities: HashMap<String, Entity>,
    relations: Vec<Relation>,
}

impl EntityGraph {
    /// 添加实体
    pub fn add_entity(&mut self, entity: Entity);
    
    /// 添加关系
    pub fn add_relation(&mut self, from: &str, to: &str, kind: RelationKind);
    
    /// 查询实体关系
    pub fn query(&self, entity: &str) -> Vec<(Entity, RelationKind, Entity)>;
    
    /// 导出为 Mermaid 图
    pub fn to_mermaid(&self) -> String;
}
```

### 14.2 事件时间线
**文件**: `graph/timeline.rs` (新建)

```rust
pub struct EventTimeline {
    events: Vec<Event>,
}

impl EventTimeline {
    /// 按时间排序
    pub fn chronological(&self) -> Vec<&Event>;
    
    /// 按主题聚类
    pub fn by_topic(&self) -> HashMap<String, Vec<&Event>>;
    
    /// 生成时间线 Markdown
    pub fn to_markdown(&self) -> String;
}
```

### 14.3 KB 双向链接
**文件**: `graph/backlinks.rs` (新建)

```rust
/// 媒体资源 ↔ KB 实体双向链接
pub struct MediaKBLinker {
    kb: Arc<KnowledgeBase>,
}

impl MediaKBLinker {
    /// 从媒体提取实体 → 写入 KB
    pub async fn extract_to_kb(&self, item: &MediaItem) -> Result<()>;
    
    /// 从 KB 实体 → 反向查找相关媒体
    pub async fn find_related_media(&self, entity: &str) -> Vec<MediaItem>;
}
```

---

## 十、Phase 15: 性能优化 (Performance)

### 15.1 连接池
**文件**: `engine/pool.rs` (新建)

```rust
pub struct ConnectionPool {
    clients: HashMap<String, reqwest::Client>,
    max_idle: usize,
}
```

- [ ] reqwest Client 复用
- [ ] HTTP/2 优先
- [ ] 连接超时配置

### 15.2 响应缓存
**文件**: `engine/cache.rs` (新建)

```rust
pub struct ResponseCache {
    store: HashMap<String, CachedResponse>,
    max_entries: usize,
    ttl_secs: u64,
}

impl ResponseCache {
    pub fn get(&self, key: &str) -> Option<CachedResponse>;
    pub fn insert(&mut self, key: &str, resp: CachedResponse);
    pub fn cleanup(&mut self);  // 过期清理
}
```

- [ ] LRU 缓存
- [ ] TTL 过期
- [ ] 内存限制 (100MB)

### 15.3 并发控制
**文件**: `engine/concurrency.rs` (新建)

```rust
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl ConcurrencyLimiter {
    /// 限制并发请求数
    pub async fn run<F, R>(&self, f: F) -> R
    where F: Future<Output = R> + Send;
}
```

- [ ] Semaphore 并发限制
- [ ] 每源独立限制
- [ ] 全局上限

---

## 十一、Phase 16: CLI 命令扩展 (CLI Commands)

### 16.1 feed 命令组
**文件**: `cli/feed.rs` (新建)

```bash
# Feed 管理
nt feed add <url> --category tech
nt feed remove <url>
nt feed list
nt feed enable <url>
nt feed disable <url>

# Feed 操作
nt feed fetch                    # 拉取所有
nt feed fetch <url>              # 拉取单个
nt feed search <query>           # 搜索条目
nt feed top                      # Top 10 高分条目
nt feed briefing                 # 每日摘要

# OPML
nt feed import <file.opml>
nt feed export > feeds.opml
```

### 16.2 search 增强
**文件**: `cli/search.rs` (修改)

```bash
# 搜索增强
nt search <query> --source netease --quality flac
nt search <query> --sort relevance|freshness|rating
nt search <query> --format json|table|markdown
nt search <query> --limit 20
```

### 16.3 lyrics 命令
**文件**: `cli/lyrics.rs` (新建)

```bash
# 歌词操作
nt lyrics search "歌名" --artist "艺术家"
nt lyrics fetch <song_id>
nt lyrics sync <file.lrc> --offset 1000
nt lyrics export <song_id> --format lrc|srt|txt
```

---

## 十二、执行优先级

### P0 (核心体验)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 7 歌词增强 | **Critical** | 3 天 | 无 |
| Phase 9 搜索增强 | **Critical** | 4 天 | 无 |
| Phase 10 源健康监控 | **Critical** | 2 天 | 无 |

### P1 (功能扩展)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 8 视频增强 | **High** | 3 天 | 无 |
| Phase 11 社交增强 | **High** | 4 天 | Phase 10 |
| Phase 15 性能优化 | **High** | 3 天 | 无 |

### P2 (智能化)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 12 AI 集成 | **Medium** | 5 天 | Phase 9 |
| Phase 14 知识图谱 | **Medium** | 4 天 | Phase 12 |

### P3 (生态)
| Phase | 优先级 | 预估工时 | 依赖 |
|-------|--------|----------|------|
| Phase 13 插件系统 | **Low** | 6 天 | 无 |
| Phase 16 CLI 扩展 | **Low** | 2 天 | Phase 7-12 |

---

## 十三、验收标准

### Phase 7 歌词增强 ✅
- [x] LRC 解析 + 时间轴同步正确
- [x] 多格式降级链正常 (LRC/QRC/KRC/TTML)
- [x] 搜索结果按质量排序

### Phase 8 视频增强 ✅
- [x] 视频元数据提取完整 (ffprobe)
- [x] 离线缓存 LRU 淘汰正常
- [x] 流选择降级链正确 (4K→1080p→720p)

### Phase 9 搜索增强 ✅
- [x] BM25 排名优于简单字符串匹配
- [x] 搜索建议 Trie 前缀树
- [x] 联合搜索超时降级正常

### Phase 10 源健康监控 ✅
- [x] 健康探测 5s 超时
- [x] 自动降级/恢复正常
- [x] 速率限制令牌桶正确

### Phase 11 社交增强 ✅
- [x] Twitter/X Cookie 认证 + 搜索
- [x] Telegram 频道监控
- [x] RSS 桥接 (Twitter/YouTube/Reddit)

### Phase 12 AI 集成 ✅
- [x] 内容摘要 + 每日 briefing
- [x] 情感分析 (26 关键词规则)
- [x] 实体识别 (组织/技术/产品)

### Phase 15 性能优化 ✅
- [x] 连接池 (per-host 复用)
- [x] 响应缓存 (LRU + TTL)
- [x] 并发控制 (Semaphore)

---

## 十四、文件结构预览

```
nt_world_media_source/
├── audio/          # 11 音频源
├── video/          # 3 视频源
├── image/          # 4 图片源
├── document/       # 2 文档源
├── book/           # 2 图书源
├── lyrics/         # 3 歌词源
│   ├── mod.rs
│   ├── timed.rs    # [NEW] 时间轴歌词
│   ├── format.rs   # [NEW] 多格式解析
│   └── search.rs   # [NEW] 歌词搜索
├── social/         # 4 社交源
│   ├── mod.rs
│   ├── twitter_enhanced.rs  # [NEW]
│   ├── telegram.rs          # [NEW]
│   └── rss_bridge.rs        # [NEW]
├── feed/           # Feed 聚合 (RadarRSS)
├── engine/
│   ├── mod.rs
│   ├── ranking.rs   # [NEW] BM25 排名
│   ├── suggest.rs   # [NEW] 搜索建议
│   ├── federated.rs # [NEW] 联合搜索
│   ├── health.rs    # [NEW] 健康监控
│   ├── ratelimit.rs # [NEW] 速率限制
│   ├── pool.rs      # [NEW] 连接池
│   ├── cache.rs     # [NEW] 响应缓存
│   └── concurrency.rs # [NEW] 并发控制
├── ai/
│   ├── mod.rs
│   ├── summarizer.rs # [NEW] 内容摘要
│   ├── sentiment.rs  # [NEW] 情感分析
│   └── ner.rs        # [NEW] 实体识别
├── plugin/
│   ├── mod.rs
│   ├── loader.rs     # [NEW] 插件加载
│   └── musicfree_compat.rs # [NEW] MusicFree 兼容
├── graph/
│   ├── mod.rs
│   ├── entity_graph.rs # [NEW] 实体关系图
│   ├── timeline.rs     # [NEW] 事件时间线
│   └── backlinks.rs    # [NEW] KB 双向链接
├── cli/
│   ├── feed.rs      # [NEW] feed 命令
│   ├── search.rs    # [MODIFY] search 增强
│   └── lyrics.rs    # [NEW] lyrics 命令
├── mod.rs
├── types.rs
├── engine.rs
├── api.rs
├── resource_store.rs
├── lx_script.rs
├── playback.rs
├── now_playing.rs
└── tests.rs
```

---

## 十五、技术债清理

| 项目 | 优先级 | 说明 |
|------|--------|------|
| 移除 `#[allow(dead_code)]` | Medium | 逐个审查并移除 |
| 统一错误类型 | High | `MediaError` → `nt_core_error` 统一 |
| 移除硬编码 API Key | High | 改为环境变量 / KB 配置 |
| 文档补全 | Low | 每个 pub fn 加 `///` 注释 |

---

## 十六、执行状态 (2026-09-08)

| Phase | 状态 | 新增文件 | 说明 |
|-------|------|----------|------|
| Phase 1-6 | ✅ | `feed/` (9 文件) | RSS 聚合 + RadarRSS 核心 |
| Phase 7 | ✅ | `lyrics/timed.rs`, `format.rs`, `search.rs` | 歌词增强 |
| Phase 8 | ✅ | `video/metadata.rs`, `stream.rs`, `offline.rs` | 视频增强 |
| Phase 9 | ✅ | `engine/ranking.rs`, `suggest.rs`, `federated.rs` | 搜索增强 |
| Phase 10 | ✅ | `engine/health.rs`, `ratelimit.rs` | 健康监控 |
| Phase 11 | ✅ | `social/twitter_enhanced.rs`, `telegram.rs`, `rss_bridge.rs` | 社交增强 |
| Phase 12 | ✅ | `ai/summarizer.rs`, `sentiment.rs`, `ner.rs` | AI 集成 |
| Phase 15 | ✅ | `engine/pool.rs`, `cache.rs`, `concurrency.rs` | 性能优化 |

### 新增文件汇总 (27 个)

```
lyrics/timed.rs, format.rs, search.rs
video/metadata.rs, stream.rs, offline.rs
engine/ranking.rs, suggest.rs, federated.rs, health.rs, ratelimit.rs, pool.rs, cache.rs, concurrency.rs
ai/summarizer.rs, sentiment.rs, ner.rs
social/twitter_enhanced.rs, telegram.rs, rss_bridge.rs
feed/parser.rs, engine.rs, registry.rs, text_utils.rs, features.rs, dedup.rs, scoring.rs, alerter.rs
```

### 编译状态

- `nt_world_media_source/` 全部新模块: **零错误** ✅
- 其他模块: 存在预存错误 (energy_core, traits 等)，与媒体源无关
