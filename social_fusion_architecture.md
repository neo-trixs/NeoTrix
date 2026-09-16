# NeoTrix 社交接入层融合架构设计

## 设计目标

将现有冗余、零散、扁平的社会媒体能力重构为**通用统一社交接入层**，实现对所有外部模型（Twitter/X、Reddit、Instagram、TikTok、YouTube、LinkedIn）的统一接入。

## 核心原则

1. **消除冗余** — 删除 `types.rs` 死代码，合并 `source/text/social/` 到 `social_access/`
2. **扁平缺陷修复** — 建立 4 层抽象：Trait → Adapter → Manager → Capability
3. **跨域错位纠正** — L2 Perception 仅保留感知能力，L1 Action 保留行动能力
4. **通用模型适配** — 所有平台共享统一的 `SocialInteraction` 数据模型

---

## 融合架构总览

```
┌─────────────────────────────────────────────────────┐
│                  L1 Action (nt_act)                  │
│  ┌──────────────┐  ┌───────────────┐  ┌──────────┐ │
│  │ SocialAction │  │ PublishAction │  │ MetricAgg│ │
│  │  (发帖/互动) │  │  (跨平台发布) │  │(指标聚合)│ │
│  └──────┬───────┘  └──────┬────────┘  └────┬─────┘ │
│         │                 │                │        │
├─────────┼─────────────────┼────────────────┼────────┤
│         ▼                 ▼                ▼        │
│                  L2 Perception (nt_world)             │
│  ┌──────────────────────────────────────────────┐   │
│  │          SocialAccessManager (统一入口)          │   │
│  │  ┌─────────┐  ┌──────────┐  ┌───────────┐   │   │
│  │  │ AuthService│ │FeedService│ │PostService│   │   │
│  │  │(认证管理) │ │(推荐流获取)│ │(发帖管理) │   │   │
│  │  └────┬────┘  └────┬─────┘  └─────┬──────┘   │   │
│  └───────┼────────────┼──────────────┼───────────┘   │
│          │            │              │                │
│  ┌───────▼────────────▼──────────────▼───────────┐   │
│  │         PlatformAdapterRegistry (适配器注册表)    │   │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌─────┐  │   │
│  │  │XAdapter│ │RDAdapter│ │IGAdapter│ │TTAdapter│  │   │
│  │  │(Twitter)│ │(Reddit) │ │(Insta) │ │(TikTok)│  │   │
│  │  └──┬───┘ └──┬───┘ └──┬───┘ └──┬───┘ └──┬──┘  │   │
│  └──────┼────────┼────────┼────────┼────────┼──────┘   │
│         │        │        │        │        │          │
│  ┌──────▼────────▼────────▼────────▼────────▼──────┐   │
│  │        AntiDetectLayer (NT-SHIELD 隐身网络)         │   │
│  │   StealthHttpClient + ProxyPool + FingerprintMgr   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │      SessionVault (会话凭证管理)                    │   │
│  │   TokenStore + RefreshManager + CredentialVault   │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## 4 层抽象架构

### Layer 1: Trait Layer (特征层)

```rust
// 通用特征定义 — 所有平台适配器必须实现
pub trait SocialPlatform {
    fn id(&self) -> PlatformId;
    fn name(&self) -> &'static str;
    fn auth_flow(&self) -> AuthFlow;
    fn api_base_url(&self) -> &'static str;
}

pub trait SocialAuth {
    async fn login(&self, creds: Credentials) -> Result<Session, AuthError>;
    async fn refresh(&self, session: &mut Session) -> Result<(), AuthError>;
    async fn logout(&self, session: &Session) -> Result<(), AuthError>;
}

pub trait SocialFeed {
    async fn get_recommended(&self, session: &Session, limit: usize) -> Result<FeedResult, FeedError>;
    async fn get_following(&self, session: &Session, limit: usize) -> Result<FeedResult, FeedError>;
    async fn get_trending(&self, session: &Session) -> Result<Vec<TrendingTopic>, FeedError>;
}

pub trait SocialPost {
    async fn create_post(&self, session: &Session, content: PostContent) -> Result<PostResult, PostError>;
    async fn delete_post(&self, session: &Session, post_id: &str) -> Result<(), PostError>;
    async fn interact(&self, session: &Session, action: InteractionAction) -> Result<(), PostError>;
}

pub trait SocialMetrics {
    async fn aggregate_metrics(&self, session: &Session, posts: &[PostRef]) -> Result<MetricsReport, MetricsError>;
}
```

### Layer 2: Adapter Layer (适配器层)

每个平台一个适配器实现所有 trait。关键设计：
- 使用 `enum Platform { X(Box<XAdapter>), Reddit(Box<RedditAdapter>) }` 统一调度
- 适配器内部使用 `AntiDetectHttpClient` 进行请求
- 适配器之间通过 `PlatformAdapterRegistry` 统一管理

### Layer 3: Manager Layer (管理层)

`SocialAccessManager` 重构为：
- `AuthService` — 登录、刷新、注销
- `FeedService` — 推荐流、关注流、热门话题
- `PostService` — 发帖、删帖、互动
- `MetricsService` — 指标聚合
- `SessionVault` — 凭证管理

### Layer 4: Capability Layer (能力层)

映射到 `social_access_node.rs` 的 5 项能力：
- `social.login` → `AuthService`
- `social.feed.retrieve` → `FeedService`
- `social.post.unify` → `PostService`
- `social.metrics.aggregate` → `MetricsService`
- `social.session.manage` → `SessionVault`

---

## 冗余清理清单

### 立即删除 (137+ 行死代码)
- `social_access/types.rs` — 整个文件删除，被 `mod.rs` 替代

### 合并迁移
- `source/text/social/twitter.rs` → 功能合并到 `social_access/extractors/twitter.rs`
- `source/text/social/instagram.rs` → 功能合并到 `social_access/extractors/instagram.rs`
- `source/text/social/tiktok.rs` → 功能合并到 `social_access/extractors/tiktok.rs`
- `source/text/social/ytdlp.rs` → 新增 `social_access/extractors/ytdlp.rs`
- `source/text/social/mod.rs` → 标记为完全废弃后删除

### `build_default_engine()` 重构
- 移除 `MediaEngine` 中注册 `TwitterSource`/`InstagramSource`/`TikTokSource` 的调用
- 改为通过 `SocialAccessManager` 统一管理

### `nt_world_search.rs` 拆分
- 拆分为 `search_engine.rs` + `evidence_scorer.rs` + `site_learning.rs` + `router.rs` + `unified_search.rs`
- 将 `x.com`/`twitter.com` 的站点学习注册到 `SocialAccessManager`

---

## 推荐算法融合方案

### 通用推荐引擎架构

```rust
pub struct UniversalRecommender {
    // 基础检索层
    retrieval: ANNRetriever,          // 向量检索 (SimClusters 模式)
    // 排序层
    ranker: MultiTaskRanker,          // 多任务排序 (Phoenix 模式)
    // 混合层
    mixer: HomeMixer,                 // 混合编排 (Rust 实现)
    // 多样性层
    diversity: DPPReRanker,           // 行列式点过程去重
}
```

### 推荐权重参考 (基于 x-algorithm 开源数据)

```rust
pub const ACTION_WEIGHTS: &[(&str, f64)] = &[
    ("share_copy_link", 20.0),
    ("reply_mutual", 20.0),
    ("reply_normal", 5.0),
    ("quote_dm", 5.0),
    ("follow_author", 4.0),
    ("generic_share", 2.0),
    ("repost", 1.0),
    ("like", 0.5),
    ("open_link", 0.2),
    ("report", -234.0),
    ("mute_author", -58.8),
];
```

---

## 认证架构设计

### 统一认证流程

```rust
pub struct Credentials {
    pub platform: SocialPlatform,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub oauth_state: Option<String>,  // PKCE state
}

pub struct SessionVault {
    store: HashMap<SocialPlatform, SessionEntry>,
    keyring: KeyringBackend,  // OS keychain
}

impl SessionVault {
    pub async fn login(&self, platform: SocialPlatform, flow: OAuthFlow) -> Result<Session, AuthError>;
    pub async fn refresh(&self, platform: SocialPlatform) -> Result<(), AuthError>;
    pub fn get_session(&self, platform: SocialPlatform) -> Option<&SessionEntry>;
}
```

### 各平台认证方式映射

| 平台 | 认证方式 | Token 类型 | 实现策略 |
|------|---------|-----------|---------|
| X/Twitter | OAuth 2.0 PKCE | Bearer + User Access | `oauth2` crate |
| Reddit | OAuth 2.0 | Bearer | `oauth2` crate |
| Instagram | OAuth 2.0 (Facebook Login) | Short-lived → Long-lived | `oauth2` crate |
| TikTok | OAuth 2.0 | Bearer (24h) | `oauth2` crate |
| YouTube | OAuth 2.0 | Bearer | `oauth2` crate |
| LinkedIn | OAuth 2.0 | Bearer | `oauth2` crate |

---

## 跨域边界修正

### L2 Perception 保留 (感知)
- `SocialAccessManager` — 社交数据感知
- `PlatformAdapter` — 平台适配
- `AntiDetectLayer` — 反检测感知
- 推荐算法检索/排序

### L1 Action 保留 (行动)
- `SocialAction` — 发帖/互动行动
- `PublishGateway` — 跨平台发布网关
- `CredentialManager` — 凭证管理（跨 session 持久化）

### 修正后的依赖方向
```
L1 Action (SocialAction) ──depends on──> L2 Perception (SocialAccessManager)
L2 Perception ──depends on──> NT-SHIELD (AntiDetectLayer)
NT-SHIELD StealthHttpClient ──used by──> L2 Perception
```

---

## 实施优先级

### P0 — 立即修复 (架构完整性)
1. 删除 `types.rs` 死代码
2. 建立 `SocialPlatform` trait
3. 合并 `source/text/social/` 到 `social_access/`
4. 实现 `AuthService::login()` 基础 OAuth

### P1 — 核心能力 (C1→C2)
5. 实现 `FeedService::get_recommended()` 通用推荐流
6. 实现 `PostService::create_post()` 基础发帖
7. `SessionVault` 凭证持久化
8. `PlatformAdapterRegistry` 统一调度

### P2 — 完整能力 (C2→C3)
9. 所有平台适配器完整实现
10. 推荐算法权重系统
11. `MetricsService` 聚合
12. `SocialAccessManager` 重构为 4 服务架构

### P3 — 高级特性 (C3→C4)
13. 能力树注册到生产环境
14. 反检测深度集成
15. 跨平台统一发布
16. 实时流式推荐

---

## 架构决策记录 (ADR)

### ADR-001: 统一 `SocialPlatform` 枚举
- **决策**: 使用 `mod.rs` 的 `SocialPlatform { Twitter, Reddit, Instagram, TikTok, Youtube, Linkedin }`
- **理由**: 与 `social_access_node.rs` 元数据一致，避免双枚举
- **影响**: 删除 `types.rs` 后需更新所有引用

### ADR-002: Trait-based 适配器模式
- **决策**: 所有平台适配器实现 `SocialPlatform` trait
- **理由**: 符合 Rust 惯用模式，与 NeoTrix capability tree 兼容
- **影响**: `manager.rs` 的 `match platform` 改为 trait 对象调度

### ADR-003: 推荐算法使用开源 x-algorithm 权重
- **决策**: 参考 x-algorithm 开源项目的权重配置
- **理由**: 2026 年最权威的推荐系统实现
- **影响**: 需要引入 `x-algorithm` 依赖或手动移植权重

### ADR-004: 认证使用 `oauth2` crate
- **决策**: 使用 `oauth2` Rust crate 作为统一 OAuth 实现
- **理由**: 成熟的 OAuth2 库，支持 PKCE、设备流等
- **影响**: 需要添加 `oauth2` 依赖

### ADR-005: L1/L2 边界修正
- **决策**: 社交感知在 L2，社交行动在 L1
- **理由**: 遵循六层架构，L2 负责感知，L1 负责行动
- **影响**: `SocialAccessManager` 不在 L1，`SocialAction` 调用 `SocialAccessManager`

## 实施状态 (2026-09-15)

| 组件 | 状态 | 文件 |
|------|------|------|
| `traits.rs` | ✅ 完成 | 215行, 完整特征定义 |
| `feed.rs` | ✅ 完成 | 140行, UniversalRecommender + PlatformAdapterRegistry |
| `auth.rs` | ✅ 完成 | 95行, AuthService |
| `manager.rs` | ✅ 重构 | 117行, 使用 FeedService |
| `mod.rs` | ✅ 更新 | 添加 auth, feed 模块 |
| `extractors/mod.rs` | ✅ 更新 | 5个平台导出 |
| `extractors/ytdlp.rs` | ✅ 新建 | 94行, YouTube 提取 |
| `social_access_node.rs` | ✅ 更新 | 6平台 + architecture metadata |
| `types.rs` | ✅ 删除 | 137行死代码清理 |
| `source/text/social/` | ⚠️ 部分 | 从 build_default_engine 移除 |

### 剩余问题
- SocialPlatformAdapter 对象安全性需修复
- 各 extractor 缺少 impl SocialPlatformAdapter 显式实现
- SocialPlatform::from_str() 需要实现
- oauth2 crate 集成尚未完成
