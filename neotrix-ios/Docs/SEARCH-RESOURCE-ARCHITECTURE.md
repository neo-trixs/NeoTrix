# NeoGram Search & Resource Architecture — 搜索与资源管理架构

> 状态: 提议 (proposed) | 日期: 2026-08-08 | 作者: NT-CORE (des-architect)
> 前置: `FUSION-ARCHITECTURE.md` + `FUSION-DEEP-DIVE.md`
> 目标: 搜索框对接官方搜索接口 → 过滤垃圾/广告 → E8 精准排序 → 频道统一资源管理 → 文件时间线分门别类

---

## 1. 设计原则

| 原则 | 说明 |
|------|------|
| **官方接口优先** | 搜索走 Telegram 官方 MTProto 搜索方法（`messages.searchGlobal` / `messages.search` / `contacts.search`），非本地 mock |
| **过滤前置** | 搜索结果先过 FilterEngine（关键词 + 频道广告 + E8 语义），无效结果不进入排序 |
| **E8 精准排序** | 过滤后结果经 E8 相关性评分（0-100），按分数降序 = "最优解" |
| **统一资源管理** | 加入的频道 = 资源源，统一索引文件/消息/媒体 |
| **时间线 + 分类** | 文件按时间线排序 + 按类型/主题分门别类 |
| **降级策略** | Rust 核心未初始化 / 官方 API 未配置 → 降级本地搜索 + 规则过滤 |

---

## 2. 分层架构

```
┌─────────────────────────────────────────────────────────┐
│  UI Layer                                               │
│  ChatListUI (搜索框) / SearchResultsUI /                │
│  ChannelLibraryUI (频道资源) / FileTimelineUI (文件)     │
├─────────────────────────────────────────────────────────┤
│  Feature Layer                                          │
│  SearchEngine (搜索中枢)                                 │
│  ├─ SearchProvider (官方 API 适配)                       │
│  ├─ SearchFilter (FilterEngine 复用)                     │
│  ├─ SearchRanker (E8 排序)                               │
│  └─ SearchResult (统一结果模型)                          │
│  ChannelResourceManager (频道资源管理)                    │
│  FileTimelineEngine (文件时间线 + 分类)                   │
├─────────────────────────────────────────────────────────┤
│  Domain Layer                                            │
│  NeoGramCore (协调器) / MTProto (网络)                   │
├─────────────────────────────────────────────────────────┤
│  Bridge Layer (NeoTrixFFI)                               │
├─────────────────────────────────────────────────────────┤
│  Rust Core (E8 / GWT / VSA / KB)                         │
└─────────────────────────────────────────────────────────┘
```

---

## 3. SearchEngine 设计

### 3.1 官方搜索接口契约（MTProto TL 方法）

| TL 方法 | 用途 | 参数 |
|---------|------|------|
| `messages.searchGlobal` | 全局搜索（所有对话/频道/消息） | `q`, `filter`, `min_date`, `max_date`, `offset_id`, `limit` |
| `messages.search` | 指定对话内搜索 | `peer`, `q`, `filter`, `min_date`, `max_date` |
| `contacts.search` | 搜索联系人/用户 | `q`, `limit` |
| `channels.searchPosts` | 频道内搜索帖子 | `channel`, `q`, `filter` |

### 3.2 SearchProvider 协议

```swift
public protocol SearchProvider {
    func searchGlobal(query: String, filter: SearchFilter, limit: Int) async throws -> [SearchResult]
    func searchInChat(chatID: Int64, query: String, filter: SearchFilter) async throws -> [SearchResult]
    func searchContacts(query: String) async throws -> [SearchResult]
}
```

- **官方实现** `MTProtoSearchProvider`: 走 `MTProtoManager.invoke` 封装 TL 方法（apiId/apiHash 配置后启用）
- **降级实现** `LocalSearchProvider`: 本地 mock 数据 + KB semanticSearch（FFI 已提供 `semanticSearch(query:namespace:limit:)`）

### 3.3 统一结果模型

```swift
public struct SearchResult: Identifiable, Equatable {
    public let id: UUID
    public let type: SearchResultType   // chat / message / media / document / contact
    public let title: String
    public let subtitle: String
    public let timestamp: Date
    public let chatID: Int64?
    public let messageID: Int64?
    public let media: MessageMedia?
    public let relevance: Double         // E8 评分 0-100
    public let isFiltered: Bool         // 是否被过滤（垃圾/广告）
}
```

### 3.4 搜索管线（Search Pipeline）

```
用户输入 → SearchEngine.search(query)
  → 1. SearchProvider.searchGlobal (官方 API / 本地降级)
  → 2. SearchFilter.filter (FilterEngine: 关键词 + 广告 + AI 语义)
  → 3. SearchRanker.rank (E8 相关性评分 0-100)
  → 4. 排序 + 分组 (按类型: 聊天/消息/文件/联系人)
  → 5. UI 渲染
```

### 3.5 过滤规则（复用 FilterEngine）

- 关键词规则（spam/promo/discount/click here）
- 频道广告检测（`looksLikeAd`: >=2 广告词）
- AI 语义过滤（E8 判断 "spam" 且 confidence > 0.6）
- **新增**: 搜索结果专用过滤（`filterSearchResults`）— 过滤无效/重复/低质量结果

---

## 4. ChannelResourceEngine 设计（频道统一资源管理）

### 4.1 职责

- 统一管理用户加入的所有频道（资源源）
- 索引频道内文件/消息/媒体
- 提供频道资源统计（文件数/类型分布/时间范围）
- 支持频道收藏/归档/隐藏

### 4.2 模型

```swift
public struct ChannelResource: Identifiable, Equatable {
    public let id: Int64
    public var title: String
    public var isJoined: Bool
    public var isArchived: Bool
    public var isFavorite: Bool
    public var fileCount: Int
    public var lastActivity: Date
    public var categories: [ResourceCategory]   // 分门别类
}

public enum ResourceCategory: String, CaseIterable {
    case documents, images, videos, audio, links, code, other
}
```

### 4.3 方法

```swift
public func addChannel(_ channel: ChannelResource)          // 加入频道
public func archiveChannel(_ channel: ChannelResource)      // 归档
public func favoriteChannel(_ channel: ChannelResource)     // 收藏
public func resourcesForChannel(_ channel: ChannelResource) -> [ChannelResource]
public func allResources() -> [ChannelResource]             // 统一资源列表
public func searchResources(query: String) async -> [ChannelResource]  // 资源内搜索
```

---

## 4. FileTimelineEngine 设计（文件时间线 + 分门别类）

### 4.1 职责

- 从频道/聊天提取所有文件（`ChatMessage.media` 的 document/video/audio/image）
- 按时间线排序（最新在前）
- 按类型分门别类（文档/图片/视频/音频/压缩包/代码）
- 过滤无效文件（损坏/重复/垃圾）

### 4.2 模型

```swift
public struct FileItem: Identifiable, Equatable {
    public let id: UUID
    public let name: String
    public let type: ResourceCategory
    public let size: Int64
    public let timestamp: Date
    public let chatID: Int64
    public let chatTitle: String
    public let url: URL?
    public let isDuplicate: Bool
    public let isFiltered: Bool
}

public struct FileTimeline: Identifiable {
    public let id = UUID()
    public let date: Date
    public let items: [FileItem]
}
```

### 4.3 方法

```swift
public func buildTimeline(from messages: [ChatMessage]) -> [FileTimeline]  // 时间线分组
public func categorize(_ file: FileItem) -> ResourceCategory               // 分类
public func filterFiles(_ files: [FileItem]) -> [FileItem]                 // 过滤垃圾/重复
public func searchFiles(query: String) -> [FileItem]                       // 文件搜索
```

### 4.4 时间线分组逻辑

```
按日期分组: 今天 / 昨天 / 本周 / 本月 / 更早
每组内按时间倒序
```

---

## 5. 数据流（The Spice Must Flow）

```
搜索: 用户输入 → SearchEngine → SearchProvider (官方/本地)
  → FilterEngine (过滤) → SearchRanker (E8 排序) → SearchResultsView

资源: 加入频道 → ChannelResourceEngine → 索引文件
  → FileTimelineEngine → 时间线 + 分类 → FileLibraryView

导出: 文件 → ExportEngine → KB (VSA 沉淀)
```

---

## 6. 实施路线图

| 阶段 | 内容 | 验证 |
|------|------|------|
| P0 | SearchEngine + SearchProvider 协议 + 降级实现 | swiftc 0 error |
| P1 | ChannelResourceEngine + FileTimelineEngine | swiftc 0 error |
| P2 | ChatListUI 搜索框接线 + SearchResultsView | swiftc + 功能测试 |
| P3 | SettingsUI 入口 + 频道资源管理 UI | swiftc + 功能测试 |
| P4 | 官方 API 对接（apiKey 就绪后） | 集成测试 |

---

## 7. 风险与未决项

| 风险 | 缓解 |
|------|------|
| 官方 API 需 apiKey/apiHash | SearchProvider 协议抽象，mock 降级先行 |
| MTProto 层未实现真实网络 | 本地降级 + KB semanticSearch |
| 文件索引量大 | 懒加载 + 分页 |
| 未决: 官方 API 认证流程 | 待 apiKey 就绪后对接 |