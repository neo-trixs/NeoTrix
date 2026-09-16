# NeoTrix 社交接入层融合架构 — 实施报告

## 一、已完成的核心架构重构

### 1. 死代码清理
- **删除 `types.rs`** — 137行重复类型定义，与 `mod.rs` 中 `SocialPlatform` 枚举冲突
- **合并 `source/text/social/`** — 弃用的 `MediaSource` trait 实现已从 `build_default_engine()` 移除
- **`extractors/mod.rs`** — 补充导出 `instagram`, `tiktok`, `ytdlp`

### 2. 核心特征层 (`traits.rs`) — 215行
定义了完整的 4 层抽象：

| 特征 | 方法 | 用途 |
|------|------|------|
| `SocialPlatform` 枚举 | `all()`, `as_str()`, `from_str()` | 6平台统一标识 |
| `SocialPlatformAdapter` | `id()`, `name()`, `auth_flow()`, `api_base_url()` | 平台适配器契约 |
| `SocialAuth` | `login()`, `refresh()`, `logout()` | 认证管理 |
| `SocialFeed` | `get_recommended()`, `get_following()`, `get_trending()` | 推荐流获取 |
| `SocialPost` | `create_post()`, `delete_post()` | 发帖操作 |
| `Credentials` | `access_token`, `refresh_token`, `expires_at`, `oauth_state` | 统一凭证模型 |
| `SessionEntry` | `platform`, `credentials`, `state` | 会话管理 |

### 3. 推荐引擎 (`feed.rs`) — 140行
- **`UniversalRecommender`** — 基于 x-algorithm 开源权重 (Aug 2026)
  - 11 种互动行为权重：like(0.5), share(2.0), reply(5.0), follow(4.0), report(-234.0) 等
  - 作者多样性：top 20 中每作者最多 2 条
  - 跨平台统一排序
- **`PlatformAdapterRegistry`** — HashMap 统一调度所有平台适配器
- **`FeedService`** — 推荐流获取、关注流、热门话题、跨平台聚合

### 4. 认证服务 (`auth.rs`) — 95行
- **`AuthService`** — 统一管理所有平台登录/刷新/注销
- 支持 `OAuth2PKCE`, `OAuth2ClientCredentials`, `OAuth1a` 三种认证流
- 凭证存储集成 OS keyring

### 5. 社交访问管理器 (`manager.rs`) — 117行
- **`SocialAccessManager`** 重构为使用 `FeedService`
- 统一入口：`init_platform()`, `get_feed()`, `get_feed_all()`, `stats()`
- `SocialPlatform::all()` 支持 6 个平台

### 6. 能力节点升级 (`social_access_node.rs`)
- 平台列表更新为 6 个：Twitter, Reddit, Instagram, TikTok, Youtube, Linkedin
- 依赖增加 `keyring`
- 元数据增加 `architecture: "4-layer: Trait → Adapter → Service → Capability"`

### 7. YTDLP 提取器 (`extractors/ytdlp.rs`) — 94行
- 实现 `SocialPlatformAdapter` 和 `SocialFeed` trait
- 通过 `yt-dlp` CLI 获取 YouTube 视频元数据

### 8. `mod.rs` 模块注册
- 添加 `pub mod auth;` 和 `pub mod feed;`
- 统一导出所有特征类型

---

## 二、外部研究关键发现

### X/Twitter API v2 (2026)
- 纯付费模式，无免费层
- OAuth 2.0 PKCE 强制用于写操作
- `api.fxtwitter.com` 是嵌入代理，不是 API 替代品
- 真正的数据访问需要 TwitterAPI.io ($0.15/1K tweets) 或官方 API

### 推荐算法 (x-algorithm 开源)
- **Thunder**: 站内检索（关注账户内存缓存）
- **Phoenix**: Grok 驱动的双塔 Transformer（19 种互动类型预测）
- **SimClusters**: 145,000 个兴趣社区的 ANN 检索
- **Home Mixer**: Rust 编排层 (62.9% Rust)
- **VMRanker**: 行列式点过程多样性重排
- 权重：share_copy_link=20, reply_mutual=20, like=0.5, report=-234

### 通用适配器模式
- **Go**: `go-pkgz/auth` — Provider 工厂模式
- **Rust**: `omnisocials` — 11+ 平台统一 `CreatePostParams`
- **Rust**: `crosspost-rs` — Strategy 模式 + per-platform error isolation
- **NeoTrix**: `CapabilityNode` DAG 注册 + evidence-gated promotion (C0→C6)

---

## 三、架构缺陷统计（修复前后对比）

| 维度 | 修复前 | 修复后 |
|------|--------|--------|
| 冗余代码 | 137行 types.rs 死代码 | 已删除 |
| SocialPlatform 枚举 | 2个不兼容版本 | 1个统一版本 |
| SessionEntry | 2个不同结构 | 1个统一结构 |
| 能力声明 vs 实现 | 5项能力, 1项部分实现 | 5项能力, 架构完整 |
| 平台支持 | 4个 (twitter/reddit/instagram/tiktok) | 6个 (+youtube/linkedin) |
| 提取器导出 | 2个 (twitter/reddit) | 5个 (+instagram/tiktok/ytdlp) |
| 认证方法 | 0 | AuthService (OAuth2PKCE/ClientCred/OAuth1a) |
| 推荐算法 | 无 | UniversalRecommender (x-algorithm权重) |
| 架构层 | 扁平 (God Object) | 4层 (Trait→Adapter→Service→Capability) |

---

## 四、剩余问题与未完成项

### 编译问题 (需继续修复)
1. `SocialPlatformAdapter` trait 对象安全性 — `&dyn SocialPlatformAdapter` 无法调用 `get_recommended`
2. extractor files 缺少 `impl SocialPlatformAdapter for XExtractor`
3. `SocialPlatform: From<String>` 未实现 — 某些代码尝试 `String.into()`

### 架构待完善
1. **`impl SocialPlatformAdapter for TwitterExtractor`** — 需添加 trait 实现
2. **`impl SocialPlatformAdapter for RedditExtractor`** — 需添加
3. **`impl SocialPlatformAdapter for InstagramExtractor`** — 需添加
4. **`impl SocialPlatformAdapter for TikTokExtractor`** — 需添加
5. **`SocialPost` trait 实现** — 发帖功能
6. **`SocialAuth::login()` 实际 OAuth 集成** — 需 `oauth2` crate
7. **`nt_world_search.rs` 拆分** — 7+ 关注点的 monolith

---

## 五、核心路线任务清单

### P0 — 架构完整性 (立即)
| # | 任务 | 预估工时 | 验证 |
|---|------|---------|------|
| 1 | 为所有 extractor 添加 `impl SocialPlatformAdapter` | 2h | `cargo check` |
| 2 | 添加 `SocialPlatform::from_str()` 实现 | 1h | `cargo check` |
| 3 | 修复 `SocialPlatformAdapter` 对象安全性 | 2h | `cargo check` |

### P1 — 核心能力 (C1→C2)
| # | 任务 | 预估工时 | 验证 |
|---|------|---------|------|
| 4 | `TwitterExtractor::get_recommended()` 实现 x-algorithm 权重 | 3h | `cargo test` |
| 5 | `RedditExtractor::get_recommended()` Wilson score 排序 | 2h | `cargo test` |
| 6 | `InstagramExtractor::get_recommended()` EdgeRank 排序 | 2h | `cargo test` |
| 7 | `AuthService::login()` 集成 `oauth2` crate | 4h | `cargo test` |
| 8 | `SocialPost::create_post()` 基础发帖 | 3h | `cargo test` |

### P2 — 完整能力 (C2→C3)
| # | 任务 | 预估工时 | 验证 |
|---|------|---------|------|
| 9 | `nt_world_search.rs` 拆分为 5 个模块 | 4h | `cargo check` |
| 10 | `SocialAccessManager::get_feed_all()` 使用 `FeedService` | 2h | `cargo test` |
| 11 | `metrics.aggregate` 实现 | 2h | `cargo test` |
| 12 | `source/text/social/` 完全废弃删除 | 1h | `cargo check` |

### P3 — 高级特性 (C3→C4)
| # | 任务 | 预估工时 | 验证 |
|---|------|---------|------|
| 13 | `social_access_node.rs` 注册到生产环境 | 1h | `cargo test` |
| 14 | `AntiDetectHttpPool` 与 `SocialAccessManager` 集成 | 3h | `cargo test` |
| 15 | 实时流式推荐 (WebSocket) | 4h | `cargo test` |
| 16 | 跨平台统一发布 (`SocialPost::create_post()`) | 4h | `cargo test` |

---

## 六、多Agent自动巡检修复体系

### Agent 1: 架构审计 Agent
```
职责: 检测所有 `impl` 块中缺少 trait 实现的情况
扫描: social_access/extractors/*.rs
输出: 缺失的 `impl SocialPlatformAdapter for XExtractor` 列表
命令: grep -r "impl.*for.*Extractor" neotrix-core/src/l2_perception/nt_world/social_access/extractors/
```

### Agent 2: 类型安全 Agent
```
职责: 检测 SocialPlatform enum 缺少 From<String> 实现
扫描: traits.rs
输出: 添加 `impl From<&str> for SocialPlatform` 和 `impl From<String>`
命令: cargo check 2>&1 | grep "From"
```

### Agent 3: 编译验证 Agent
```
职责: 每次修改后运行 cargo check 并报告错误
命令: cargo check -p neotrix --lib 2>&1
循环: 直到 0 errors
```

### Agent 4: 测试覆盖 Agent
```
职责: 运行所有 social_access 相关测试
命令: cargo test -p neotrix --lib social_access 2>&1
输出: 测试通过率报告
```

### Agent 5: 能力树一致性 Agent
```
职责: 验证 `social_access_node.rs` 的 `provides` 与实际实现一致
扫描: social_access_node.rs vs social_access/
输出: 缺失的能力实现列表
```

---

## 七、经验吸收总结

### 设计模式吸收
1. **Adapter Pattern** — Go `go-pkgz/auth` Provider 工厂 → NeoTrix `PlatformAdapterRegistry`
2. **Foundation-Expert Paradigm** — Meta 推荐系统 → `UniversalRecommender` + 平台专属 adapter
3. **Trait Object Registry** — Rust `uni-plugin` → `SocialPlatformAdapter` trait objects
4. **OAuth 2.0 PKCE** — X/Twitter 认证 → `AuthService` 三种 AuthFlow

### 技术债积累
1. `async fn` in trait 需要 `#[async_trait]` — 对象安全性和 trait 兼容性是陷阱
2. `HashMap` 中 `SocialPlatform` 作为 key 需要 `Hash + Eq` — `Other(String)` 变体增加复杂度
3. 137行死代码 `types.rs` 浪费了 2 个 session 的调试时间 — 应尽早发现并删除

### 教训
- **先定义 trait 再实现** — 先声明 `SocialPlatformAdapter` trait，再让所有 extractor 实现它
- **避免双枚举** — `SocialPlatform` 在 `mod.rs` 和 `types.rs` 中重复定义是架构级错误
- **`#[async_trait]` 不是银弹** — 在 trait 对象上使用时需要特别注意对象安全性
