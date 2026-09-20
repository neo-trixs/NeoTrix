# 高质量数据源集成设计方案

## 1. 概述

本方案旨在为 NeoTrix 项目集成高质量的外部数据源，覆盖学术研究、技术趋势、行业报告和开源项目等多个维度，实现每日自动获取外部世界的最新信息。

## 2. 数据源清单

### 2.1 学术研究数据源

| 数据源 | API 类型 | 更新频率 | 覆盖领域 |
|--------|----------|----------|----------|
| arXiv | REST API | 每日 | AI/ML/NLP/CS/SE 等 |
| Semantic Scholar | REST API | 每日 | 全学科 |
| PubMed | E-utilities API | 每日 | 生物医学 |

### 2.2 技术趋势数据源

| 数据源 | API 类型 | 更新频率 | 覆盖领域 |
|--------|----------|----------|----------|
| Hacker News | Firebase API | 实时 | 技术社区热点 |
| GitHub Trending | 网页爬虫 | 每日 | 热门开源项目 |
| Product Hunt | GraphQL API | 每日 | 新产品发布 |
| TrendShift | REST API | 每日 | GitHub 项目趋势 |

### 2.3 科技新闻数据源

| 数据源 | API 类型 | 更新频率 | 覆盖领域 |
|--------|----------|----------|----------|
| TechCrunch | RSS | 每日 | 科技创业 |
| TheVerge | RSS | 每日 | 消费科技 |
| ArsTechnica | RSS | 每日 | 深度科技分析 |

## 3. 架构设计

### 3.1 模块结构

```
neotrix-core/src/l2_perception/nt_world/data_source/
├── mod.rs                    # 数据源模块入口
├── nt_world_arxiv.rs         # arXiv 学术论文
├── nt_world_semantic_scholar.rs  # Semantic Scholar
├── nt_world_pubmed.rs        # PubMed 生物医学
├── nt_world_hackernews.rs    # Hacker News
├── nt_world_github_trending.rs   # GitHub Trending
├── nt_world_producthunt.rs   # Product Hunt
├── nt_world_trendshift.rs    # TrendShift
└── nt_world_tech_rss.rs      # 科技新闻 RSS 聚合
```

### 3.2 统一数据模型

```rust
/// 高质量数据源统一记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecord {
    pub id: String,
    pub source: String,           // 数据源标识
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub authors: Vec<String>,
    pub published_at: Option< chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
    pub metrics: Option<RecordMetrics>,
    pub metadata: serde_json::Value,
}

/// 记录度量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetrics {
    pub citations: Option<u64>,
    pub stars: Option<u64>,
    pub forks: Option<u64>,
    pub upvotes: Option<u64>,
    pub impact_score: Option<f64>,
}
```

### 3.3 统一 Fetcher 接口

```rust
/// 高质量数据源 Fetcher trait
#[async_trait]
pub trait QualityFetcher: Send + Sync {
    /// 数据源名称
    fn source_name(&self) -> &str;
    
    /// 获取最新记录
    async fn fetch_latest(&self, limit: usize) -> Result<Vec<QualityRecord>, FetchError>;
    
    /// 搜索记录
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<QualityRecord>, FetchError>;
    
    /// 获取记录详情
    async fn get_detail(&self, id: &str) -> Result<QualityRecord, FetchError>;
}
```

## 4. 实现细节

### 4.1 arXiv 学术论文

```rust
// 端点: http://export.arxiv.org/api/query
// 参数: search_query, start, max_results, sortBy, sortOrder
// 支持领域: cs.AI, cs.LG, cs.CL, cs.SE, cs.CV, cs.NE 等
```

### 4.2 Semantic Scholar

```rust
// 端点: https://api.semanticscholar.org/graph/v1
// /paper/search?query=...&limit=...&fields=...
// 提供引用网络、影响力分析、开放引用数据
```

### 4.3 PubMed

```rust
// 端点: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/
// esearch.fcgi (搜索) + efetch.fcgi (获取详情)
// 支持 MeSH 术语和布尔查询
```

### 4.4 Hacker News

```rust
// 端点: https://hacker-news.firebaseio.com/v0/
// /topstories.json, /item/{id}.json
// 实时获取技术社区热点
```

### 4.5 GitHub Trending

```rust
// 端点: https://github.com/trending (网页爬虫)
// 解析每日/每周热门项目
// 提取 stars、forks、语言、描述等
```

### 4.6 Product Hunt

```rust
// 端点: https://api.producthunt.com/v2/api/graphql
// GraphQL 查询，需要 API token
// 获取新产品发布和讨论
```

### 4.7 TrendShift

```rust
// 端点: https://api.trendshift.io/
// 追踪 GitHub 项目增长趋势
// 识别新兴热门项目
```

### 4.8 科技新闻 RSS

```rust
// RSS 源:
// - TechCrunch: https://techcrunch.com/feed/
// - TheVerge: https://www.theverge.com/rss/index.xml
// - ArsTechnica: https://feeds.arstechnica.com/arstechnica/index
// 使用 rss crate 解析
```

## 5. 数据流设计

```
外部数据源 → Fetcher 层 → 标准化 → 存储层 → 知识库
   ↓              ↓           ↓         ↓
 arXiv         QualityRecord  ↓    nt_memory_kb
 Semantic Scholar    ↓        ↓         ↓
 PubMed             ↓        ↓         ↓
 Hacker News        ↓        ↓         ↓
 GitHub Trending    ↓        ↓         ↓
 Product Hunt       ↓        ↓         ↓
 Tech RSS           ↓        ↓         ↓
```

## 6. 配置管理

```rust
/// 高质量数据源配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySourceConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: String,
    pub fetch_interval_hours: u64,
    pub max_records_per_fetch: usize,
    pub rate_limit_per_minute: u64,
}
```

## 7. 测试策略

- **单元测试**: 使用 fixture 数据，无网络依赖
- **集成测试**: 可选真实 API 调用（需网络）
- **Mock 服务**: 本地 HTTP 服务器模拟 API 响应

## 8. 实现计划

### Phase 1: 核心数据源 (Week 1)
- [x] 设计文档
- [ ] arXiv 学术论文
- [ ] Semantic Scholar
- [ ] Hacker News

### Phase 2: 扩展数据源 (Week 2)
- [ ] PubMed 生物医学
- [ ] GitHub Trending
- [ ] Product Hunt

### Phase 3: 聚合与优化 (Week 3)
- [ ] 科技新闻 RSS 聚合
- [ ] TrendShift 集成
- [ ] 统一查询接口

### Phase 4: 测试与文档 (Week 4)
- [ ] 完整测试覆盖
- [ ] 使用文档
- [ ] 性能优化

## 9. 预期收益

- **学术前沿**: 每日追踪 AI/ML/NLP 领域最新论文
- **技术趋势**: 实时了解技术社区热点和新兴项目
- **行业动态**: 追踪科技新闻和产品发布
- **知识积累**: 自动构建高质量知识库，支持 RAG 检索
