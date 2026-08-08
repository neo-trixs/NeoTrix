// LiveFeedEngine - Live 统一中枢 (Live Feed Architecture)
// 融合: 主流社交媒体资讯聚合 + 官方搜索接口 + 过滤 + 自研推荐算法
// 管线: 内容源 → 活跃排名加权 → 过滤(垃圾/广告) → E8 评分 → 双列瀑布流
// 交互: 点赞 / 不感兴趣 / 分享 / 隐藏作者 / 屏蔽关键词

import Foundation
import Combine
import SwiftUI
import NeoTrixFFI

// MARK: - 统一内容模型 (搜索 + 推送统一)

public enum FeedItemType: String, CaseIterable, Equatable {
    case text
    case image
    case video
    case document
    case chat
    case contact
}

public struct LiveFeedItem: Identifiable, Equatable {
    public let id: UUID
    public let platform: String          // youtube / telegram / reddit / instagram / tiktok ...
    public let type: FeedItemType
    public let title: String
    public let subtitle: String
    public let author: String?
    public let timestamp: Date
    public let thumbnailURL: URL?
    public let mediaURL: URL?
    public let platformID: String?
    public let engagement: EngagementStats
    public let score: Double             // 推荐评分 0-100
    public let isFiltered: Bool          // 是否被过滤（垃圾/广告）
    public let isLiked: Bool
    public let isHidden: Bool
    
    public init(id: UUID = UUID(), platform: String, type: FeedItemType, title: String,
                subtitle: String, author: String? = nil, timestamp: Date,
                thumbnailURL: URL? = nil, mediaURL: URL? = nil,
                platformID: String? = nil, engagement: EngagementStats = EngagementStats(),
                score: Double = 0, isFiltered: Bool = false, isLiked: Bool = false, isHidden: Bool = false) {
        self.id = id
        self.platform = platform
        self.type = type
        self.title = title
        self.subtitle = subtitle
        self.author = author
        self.timestamp = timestamp
        self.thumbnailURL = thumbnailURL
        self.mediaURL = mediaURL
        self.platformID = platformID
        self.engagement = engagement
        self.score = score
        self.isFiltered = isFiltered
        self.isLiked = isLiked
        self.isHidden = isHidden
    }
}

public struct EngagementStats: Equatable {
    public var views: Int64
    public var likes: Int64
    public var comments: Int64
    public var shares: Int64
    public var saves: Int64
    
    public init(views: Int64 = 0, likes: Int64 = 0, comments: Int64 = 0, shares: Int64 = 0, saves: Int64 = 0) {
        self.views = views
        self.likes = likes
        self.comments = comments
        self.shares = shares
        self.saves = saves
    }
}

// MARK: - 内容源协议

public protocol ContentSourceProvider {
    var platformName: String { get }
    var activityWeight: Double { get }   // 全球活跃用户指数 (SimilarWeb)
    func fetchLatest(limit: Int, category: FeedCategory?) async throws -> [LiveFeedItem]
    func search(query: String, filter: FeedFilter, limit: Int) async throws -> [LiveFeedItem]
}

// MARK: - 分类 (顶部横向 Tab)

public enum FeedCategory: String, CaseIterable, Identifiable {
    case all = "All"
    case trending = "Trending"
    case tech = "Tech"
    case news = "News"
    case entertainment = "Entertainment"
    case education = "Education"
    case gaming = "Gaming"
    case live = "Live"
    
    public var id: String { rawValue }
}

public enum FeedFilter: String, CaseIterable {
    case all
    case chats
    case messages
    case media
    case documents
    case contacts
}

// MARK: - 全球活跃排名 (SimilarWeb 2026 指数)

public enum PlatformActivity {
    public static let index: [String: Double] = [
        "youtube": 1.00,
        "whatsapp": 0.87,
        "instagram": 0.81,
        "facebook": 0.76,
        "tiktok": 0.67,
        "telegram": 0.60,   // API 开放度加成
        "reddit": 0.45,
        "twitter": 0.40,
        "bilibili": 0.35,
        "douyin": 0.30,
    ]
    
    public static func weight(for platform: String) -> Double {
        index[platform.lowercased()] ?? 0.3
    }
}

// MARK: - 推荐算法 (自研 Value Model)

public struct RecommendationWeights {
    public var platform: Double = 0.20    // 活跃排名
    public var engagement: Double = 0.35  // 互动信号
    public var recency: Double = 0.20     // 时间衰减
    public var affinity: Double = 0.15    // 用户兴趣 (E8)
    public var negative: Double = 0.10    // 负反馈
    
    public static let `default` = RecommendationWeights()
}

// MARK: - 反馈信号权重 (参考小红书 CES + Meta Value Model)

public enum FeedbackSignal {
    public static let like: Double = 0.4
    public static let comment: Double = 0.6
    public static let share: Double = 0.8
    public static let save: Double = 0.8
    public static let watchTime: Double = 1.0
    public static let notInterested: Double = -2.0
    public static let hideAuthor: Double = -3.0
    public static let blockKeyword: Double = -4.0
}

// MARK: - Live Feed Engine

@MainActor
public final class LiveFeedEngine: ObservableObject {
    public static let shared = LiveFeedEngine()
    
    // 状态
    @Published public var items: [LiveFeedItem] = []
    @Published public var searchQuery = ""
    @Published public var selectedCategory: FeedCategory = .all
    @Published public var isSearching = false
    @Published public var isRefreshing = false
    @Published public var lastError: String?
    @Published public var hiddenAuthors: Set<String> = []
    @Published public var blockedKeywords: [String] = []
    @Published public var likedIDs: Set<UUID> = []
    
    // 配置
    @Published public var useAIFilter = true
    @Published public var filterAds = true
    @Published public var exploreRatio = 0.2   // Exploit 80% / Explore 20%
    
    private let filterEngine = FilterEngine()
    private let core = NeoGramCore.shared
    private var userAffinity: [String: Double] = [:]   // 关键词 → 兴趣分
    private var hiddenItems: Set<UUID> = []
    private var filterAuthors: Set<String> = []
    
    private var providers: [ContentSourceProvider] = []
    
    public init() {
        setupProviders()
        loadUserState()
    }
    
    private func setupProviders() {
        providers = [
            TelegramContentProvider(),
            YouTubeContentProvider(),
            RedditContentProvider(),
        ]
    }
    
    // MARK: - 统一管线 (搜索 + 推送融合)
    
    /// 统一入口：搜索框有输入 → 搜索模式；否则 → 推荐模式
    public func refresh() async {
        if searchQuery.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            await loadFeed()
        } else {
            await search(searchQuery)
        }
    }
    
    /// 推荐模式：内容源聚合 → 过滤 → 排序 → 瀑布流
    public func loadFeed() async {
        isRefreshing = true
        defer { isRefreshing = false }
        
        var collected: [LiveFeedItem] = []
        
        // 1. 并行拉取各内容源
        await withTaskGroup(of: [LiveFeedItem].self) { group in
            for provider in providers {
                group.addTask {
                    do {
                        return try await provider.fetchLatest(limit: 30, category: self.selectedCategory)
                    } catch {
                        return []
                    }
                }
            }
            for await batch in group {
                collected.append(contentsOf: batch)
            }
        }
        
        // 2. 过滤（垃圾/广告/隐藏作者/屏蔽关键词）
        var filtered: [LiveFeedItem] = []
        for item in collected {
            if !(await shouldFilter(item)) {
                filtered.append(item)
            }
        }
        
        // 3. 推荐排序（Value Model + E8）
        let ranked = await rank(filtered, query: nil)
        
        // 4. 多样性控制（同作者降权 + 探索配额）
        let diversified = applyDiversity(ranked)
        
        items = diversified
    }
    
    /// 搜索模式：官方搜索接口 → 过滤 → 精准排序
    public func search(_ query: String) async {
        guard !query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            await loadFeed()
            return
        }
        
        isSearching = true
        defer { isSearching = false }
        
        var collected: [LiveFeedItem] = []
        
        // 1. 各内容源搜索（官方接口 / 本地降级）
        await withTaskGroup(of: [LiveFeedItem].self) { group in
            for provider in providers {
                group.addTask {
                    do {
                        return try await provider.search(query: query, filter: .all, limit: 50)
                    } catch {
                        return []
                    }
                }
            }
            
            for await batch in group {
                collected.append(contentsOf: batch)
            }
        }
        
        // 2. 过滤（垃圾/广告）
        var filtered: [LiveFeedItem] = []
        for item in collected {
            if !(await shouldFilter(item)) {
                filtered.append(item)
            }
        }
        
        // 3. E8 精准排序（相关性）
        let ranked = await rank(filtered, query: query)
        
        // 4. 按类型分组
        items = sortAndGroup(ranked)
    }
    
    // MARK: - 过滤管线
    
    private func shouldFilter(_ item: LiveFeedItem) async -> Bool {
        if item.isFiltered { return true }
        if hiddenItems.contains(item.id) { return true }
        if let author = item.author, hiddenAuthors.contains(author) { return true }
        if let author = item.author, filterAuthors.contains(author) { return true }
        
        // 屏蔽关键词
        let text = "\(item.title) \(item.subtitle)"
        for keyword in blockedKeywords where text.lowercased().contains(keyword.lowercased()) {
            return true
        }
        
        // 广告检测
        if filterAds {
            let adPatterns = ["% off", "buy now", "limited time", "act fast", "free trial", "subscribe", "promo", "discount", "click here"]
            let matches = adPatterns.filter { text.lowercased().contains($0) }.count
            if matches >= 1 { return true }
        }
        
        // AI 语义过滤
        if useAIFilter {
            let shouldFilterResult = await filterEngine.shouldFilter(text, isChannel: item.type == .chat, isGroup: false, isDM: false)
            if shouldFilterResult { return true }
        }
        
        return false
    }
    
    // MARK: - 推荐排序 (Value Model)
    
    private func rank(_ items: [LiveFeedItem], query: String?) async -> [LiveFeedItem] {
        let weights = RecommendationWeights.default
        let e8 = core.e8Reasoning
        // 预取 MainActor 隔离状态（闭包内不可直接访问）
        let likedSet = likedIDs
        let hiddenSet = hiddenAuthors
        let affinityMap = userAffinity
        
        return await withTaskGroup(of: (Int, LiveFeedItem).self) { group in
            for (index, item) in items.enumerated() {
                group.addTask {
                    var score = 0.0
                    
                    // 1. 平台活跃权重
                    score += weights.platform * PlatformActivity.weight(for: item.platform) * 100
                    
                    // 2. 互动信号 (CES 加权)
                    let engagement = item.engagement
                    let ces = Double(engagement.likes) * FeedbackSignal.like
                        + Double(engagement.comments) * FeedbackSignal.comment
                        + Double(engagement.shares) * FeedbackSignal.share
                        + Double(engagement.saves) * FeedbackSignal.save
                    score += weights.engagement * min(ces, 100)
                    
                    // 3. 时间衰减 (越新越高)
                    let ageHours = Date().timeIntervalSince(item.timestamp) / 3600
                    let recency = max(0, 100 - ageHours * 2)
                    score += weights.recency * recency
                    
                    // 4. 用户兴趣 (E8 或关键词匹配)
                    var affinity = 0.0
                    if let e8 = e8, let query = query {
                        do {
                            let request = ReasoningRequest(
                                query: "Rate relevance of '\(item.title)' to '\(query)' from 0 to 100.",
                                context: "Feed relevance",
                                maxDepth: 1,
                                useConsciousness: false
                            )
                            let response = try e8.reason(request: request)
                            if let last = response.reasoningChain.last,
                               let num = Double(last.components(separatedBy: CharacterSet.decimalDigits.inverted).joined()) {
                                affinity = num
                            }
                        } catch {
                            affinity = keywordAffinity(item, affinityMap: affinityMap)
                        }
                    } else {
                        affinity = keywordAffinity(item, affinityMap: affinityMap)
                    }
                    score += weights.affinity * affinity
                    
                    // 5. 负反馈
                    if likedSet.contains(item.id) { score += 10 }
                    if let author = item.author, hiddenSet.contains(author) { score -= 30 }
                    
                    let ranked = LiveFeedItem(
                        platform: item.platform, type: item.type, title: item.title,
                        subtitle: item.subtitle, author: item.author, timestamp: item.timestamp,
                        thumbnailURL: item.thumbnailURL, mediaURL: item.mediaURL,
                        platformID: item.platformID, engagement: item.engagement,
                        score: min(max(score, 0), 100),
                        isFiltered: item.isFiltered, isLiked: item.isLiked, isHidden: item.isHidden
                    )
                    return (index, ranked)
                }
            }
            
            var ranked: [(Int, LiveFeedItem)] = []
            for await item in group {
                ranked.append(item)
            }
            return ranked.sorted { $0.0 < $1.0 }.map { $0.1 }
        }
    }
    
    // MARK: - 多样性控制
    
    private func applyDiversity(_ items: [LiveFeedItem]) -> [LiveFeedItem] {
        var result: [LiveFeedItem] = []
        
        for item in items {
            // 同作者降权（最多 2 条）
            let author = item.author ?? ""
            let authorCount = result.filter { $0.author == author }.count
            if authorCount >= 2 { continue }
            
            // Explore 20%: 保留低分项以探索
            if item.score >= 40 {
                result.append(item)
            } else if Double.random(in: 0...1) < exploreRatio {
                result.append(item)
            }
        }
        return result
    }
    
    private func sortAndGroup(_ items: [LiveFeedItem]) -> [LiveFeedItem] {
        let grouped = Dictionary(grouping: items) { $0.type }
        var ordered: [LiveFeedItem] = []
        for type in [FeedItemType.chat, .text, .image, .video, .document, .contact] {
            let sorted = (grouped[type] ?? []).sorted { $0.score > $1.score }
            ordered.append(contentsOf: sorted)
        }
        return ordered
    }
    
    // MARK: - 用户交互 (点赞/不感兴趣/分享)
    
    public func like(_ item: LiveFeedItem) {
        if likedIDs.contains(item.id) {
            likedIDs.remove(item.id)
        } else {
            likedIDs.insert(item.id)
            recordAffinity(item, delta: FeedbackSignal.like)
        }
    }
    
    public func share(_ item: LiveFeedItem) {
        recordAffinity(item, delta: FeedbackSignal.share)
    }
    
    public func notInterested(_ item: LiveFeedItem) {
        hiddenItems.insert(item.id)
        recordAffinity(item, delta: FeedbackSignal.notInterested)
    }
    
    public func hideAuthor(_ author: String) {
        hiddenAuthors.insert(author)
        items.removeAll { $0.author == author }
        saveUserState()
    }
    
    public func blockKeyword(_ keyword: String) {
        guard !keyword.isEmpty else { return }
        blockedKeywords.append(keyword)
        items.removeAll { "\($0.title) \($0.subtitle)".lowercased().contains(keyword.lowercased()) }
        saveUserState()
    }
    
    private func recordAffinity(_ item: LiveFeedItem, delta: Double) {
        let keywords = (item.title + " " + item.subtitle)
            .split(separator: " ")
            .map(String.init)
            .filter { $0.count > 3 }
        for kw in keywords.prefix(5) {
            userAffinity[kw, default: 50] += delta
        }
    }
    
    // MARK: - 持久化
    
    private func loadUserState() {
        if let data = UserDefaults.standard.data(forKey: "live_hidden_authors"),
           let decoded = try? JSONDecoder().decode([String].self, from: data) {
            hiddenAuthors = Set(decoded)
        }
        if let data = UserDefaults.standard.data(forKey: "live_blocked_keywords"),
           let decoded = try? JSONDecoder().decode([String].self, from: data) {
            blockedKeywords = decoded
        }
    }
    
    private func saveUserState() {
        if let data = try? JSONEncoder().encode(Array(hiddenAuthors)) {
            UserDefaults.standard.set(data, forKey: "live_hidden_authors")
        }
        if let data = try? JSONEncoder().encode(blockedKeywords) {
            UserDefaults.standard.set(data, forKey: "live_blocked_keywords")
        }
    }
}

// MARK: - 全局辅助函数 (非隔离)

private func keywordAffinity(_ item: LiveFeedItem, affinityMap: [String: Double]) -> Double {
    guard !affinityMap.isEmpty else { return 50 }
    let text = "\(item.title) \(item.subtitle)".lowercased()
    var total = 0.0
    for (keyword, score) in affinityMap where text.contains(keyword.lowercased()) {
        total += score
    }
    return min(total, 100)
}

// MARK: - 内容源实现

/// Telegram 内容源 (官方 MTProto / Bot API, 开放度最高)
@MainActor
public final class TelegramContentProvider: ContentSourceProvider {
    public var platformName: String { "telegram" }
    public var activityWeight: Double { PlatformActivity.weight(for: "telegram") }
    
    public init() {}
    
    public func fetchLatest(limit: Int, category: FeedCategory?) async throws -> [LiveFeedItem] {
        // 真实实现: MTProto channels.getChannels + messages.getHistory
        // 当前: 本地 mock 降级
        return Self.mockItems(limit: limit, category: category)
    }
    
    public func search(query: String, filter: FeedFilter, limit: Int) async throws -> [LiveFeedItem] {
        // 真实实现: messages.searchGlobal
        // 当前: 本地 mock 降级
        return Self.mockItems(limit: limit, category: nil).filter {
            $0.title.lowercased().contains(query.lowercased()) || $0.subtitle.lowercased().contains(query.lowercased())
        }
    }
    
    static func mockItems(limit: Int, category: FeedCategory?) -> [LiveFeedItem] {
        let now = Date()
        let templates: [(String, String, FeedItemType, FeedCategory)] = [
            ("E8 Hexagram 推理引擎升级", "NeoTrix 核心推理引擎完成 64 卦象重构", .text, .tech),
            ("SwiftUI 6 新特性深度解析", "Liquid Glass 设计语言全面解读", .video, .tech),
            ("Rust 零成本抽象实战", "所有权系统在大型项目中的应用", .text, .education),
            ("今日全球科技新闻", "AI 芯片竞争白热化", .image, .news),
            ("Telegram 2026 新功能", "Communities 多频道聚合上线", .video, .news),
            ("独立开发者访谈", "如何用 AI 提升 10 倍效率", .video, .entertainment),
            ("VSA 向量符号架构入门", "高维向量计算基础", .text, .education),
            ("GWT 注意力路由实践", "全局工作空间理论工程化", .text, .tech),
        ]
        
        var result: [LiveFeedItem] = []
        for (i, item) in templates.enumerated() where category == nil || category == .all || item.3 == category {
            result.append(LiveFeedItem(
                platform: "telegram",
                type: item.2,
                title: item.0,
                subtitle: item.1,
                author: "NeoTrix Channel",
                timestamp: now.addingTimeInterval(-Double(i) * 3600),
                engagement: EngagementStats(
                    views: Int64(1000 + i * 500),
                    likes: Int64(100 + i * 50),
                    comments: Int64(20 + i * 5),
                    shares: Int64(10 + i * 3),
                    saves: Int64(30 + i * 8)
                )
            ))
            if result.count >= limit { break }
        }
        return result
    }
}

/// YouTube 内容源 (RSS, 原生支持)
@MainActor
public final class YouTubeContentProvider: ContentSourceProvider {
    public var platformName: String { "youtube" }
    public var activityWeight: Double { PlatformActivity.weight(for: "youtube") }
    
    public init() {}
    
    public func fetchLatest(limit: Int, category: FeedCategory?) async throws -> [LiveFeedItem] {
        // 真实实现: YouTube Data API / RSS
        return []
    }
    
    public func search(query: String, filter: FeedFilter, limit: Int) async throws -> [LiveFeedItem] {
        return []
    }
}

/// Reddit 内容源 (RSS)
@MainActor
public final class RedditContentProvider: ContentSourceProvider {
    public var platformName: String { "reddit" }
    public var activityWeight: Double { PlatformActivity.weight(for: "reddit") }
    
    public init() {}
    
    public func fetchLatest(limit: Int, category: FeedCategory?) async throws -> [LiveFeedItem] {
        return []
    }
    
    public func search(query: String, filter: FeedFilter, limit: Int) async throws -> [LiveFeedItem] {
        return []
    }
}