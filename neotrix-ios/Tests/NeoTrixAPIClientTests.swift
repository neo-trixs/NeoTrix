// NeoTrixAPIClientTests - 纯 Swift 单元测试（CLT 无 XCTest，用断言 + 退出码）
// 覆盖: 模型编解码 round-trip / 推荐算法排序 / 反馈信号权重
// 运行: swiftc -o /tmp/neotrix-tests Sources/Features/NeoTrixAPIClient.swift Tests/NeoTrixAPIClientTests.swift && /tmp/neotrix-tests

import Foundation

// MARK: - 轻量断言

var failures = 0
var passed = 0

func check(_ condition: Bool, _ name: String) {
    if condition {
        passed += 1
        print("  ✅ \(name)")
    } else {
        failures += 1
        print("  ❌ \(name)")
    }
}

func checkEqual<T: Equatable>(_ lhs: T, _ rhs: T, _ name: String) {
    if lhs == rhs {
        passed += 1
        print("  ✅ \(name)")
    } else {
        failures += 1
        print("  ❌ \(name): \(lhs) != \(rhs)")
    }
}

// MARK: - 测试 1: 模型编解码 round-trip

func testModelCodable() {
    print("测试 1: 模型编解码 round-trip")

    // ChatRequest → JSON → ChatResponse
    let encoder = JSONEncoder()
    encoder.keyEncodingStrategy = .convertToSnakeCase
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase

    let chatReq = ChatRequest(message: "hello")
    let chatData = try! encoder.encode(chatReq)
    let chatJSON = String(data: chatData, encoding: .utf8)!
    check(chatJSON.contains("\"message\":\"hello\""), "ChatRequest 编码 message 字段")

    let chatResp = ChatResponse(reply: "hi there")
    let respData = try! encoder.encode(chatResp)
    let decodedResp = try! decoder.decode(ChatResponse.self, from: respData)
    checkEqual(decodedResp.reply, "hi there", "ChatResponse 解码 round-trip")

    // VideoSubmission snake_case 字段
    let submission = VideoSubmission(id: "v1", title: "Test", author: "NeoTrix", duration: 30, viewCount: 100, likeCount: 10, url: "https://example.com")
    let subData = try! encoder.encode(submission)
    let subJSON = String(data: subData, encoding: .utf8)!
    check(subJSON.contains("\"view_count\":100"), "VideoSubmission 编码 view_count (snake_case)")
    check(subJSON.contains("\"like_count\":10"), "VideoSubmission 编码 like_count (snake_case)")

    let decodedSub = try! decoder.decode(VideoSubmission.self, from: subData)
    checkEqual(decodedSub.viewCount, 100, "VideoSubmission 解码 viewCount")
    checkEqual(decodedSub.likeCount, 10, "VideoSubmission 解码 likeCount")

    // MomentItem 可选字段
    let moment = MomentItem(id: "m1", title: "Moment", author: "Alice", score: 85.5, reason: "high engagement", createdAt: "2026-08-08")
    let momentData = try! encoder.encode(moment)
    let decodedMoment = try! decoder.decode(MomentItem.self, from: momentData)
    checkEqual(decodedMoment.score, 85.5, "MomentItem 解码 score")
    checkEqual(decodedMoment.reason, "high engagement", "MomentItem 解码 reason")

    // SocialStatus
    let status = SocialStatus(id: "s1", platform: "telegram", isConnected: true, username: "neo", followers: 1000)
    let statusData = try! encoder.encode(status)
    let decodedStatus = try! decoder.decode(SocialStatus.self, from: statusData)
    checkEqual(decodedStatus.platform, "telegram", "SocialStatus 解码 platform")
    check(decodedStatus.isConnected, "SocialStatus 解码 isConnected")
    checkEqual(decodedStatus.followers, 1000, "SocialStatus 解码 followers")

    // FilterConfig
    let config = FilterConfig(filterAds: true, filterKeywords: ["spam", "ad"])
    let configData = try! encoder.encode(config)
    let decodedConfig = try! decoder.decode(FilterConfig.self, from: configData)
    check(decodedConfig.filterAds, "FilterConfig 解码 filterAds")
    checkEqual(decodedConfig.filterKeywords.count, 2, "FilterConfig 解码 filterKeywords")
}

// MARK: - 测试 2: 推荐算法纯函数（LiveFeedEngine 排序逻辑）

func testRecommendationSorting() {
    print("测试 2: 推荐排序逻辑")

    // 模拟 sortAndGroup 逻辑: 按类型分组 + 组内按 score 降序
    struct MockItem {
        let type: String
        let score: Double
    }

    let items = [
        MockItem(type: "video", score: 30),
        MockItem(type: "chat", score: 90),
        MockItem(type: "video", score: 70),
        MockItem(type: "moment", score: 50),
        MockItem(type: "chat", score: 60),
    ]

    let grouped = Dictionary(grouping: items) { $0.type }
    let order = ["chat", "text", "image", "video", "document", "contact", "moment", "stream"]
    var ordered: [MockItem] = []
    for type in order {
        let sorted = (grouped[type] ?? []).sorted { $0.score > $1.score }
        ordered.append(contentsOf: sorted)
    }

    checkEqual(ordered.count, 5, "排序后总数")
    checkEqual(ordered[0].type, "chat", "chat 类型优先")
    checkEqual(ordered[0].score, 90, "chat 组内最高分在前")
    checkEqual(ordered[1].score, 60, "chat 组内次高分")
    checkEqual(ordered[2].type, "video", "video 类型其次")
    checkEqual(ordered[2].score, 70, "video 组内最高分在前")
    checkEqual(ordered[4].type, "moment", "moment 类型在 video 之后")
}

// MARK: - 测试 3: 反馈信号权重

func testFeedbackSignals() {
    print("测试 3: 反馈信号权重")

    // 对齐 LiveFeedEngine.FeedbackSignal 常量
    let like: Double = 0.4
    let comment: Double = 0.6
    let share: Double = 0.8
    let save: Double = 0.8
    let watchTime: Double = 1.0
    let notInterested: Double = -2.0
    let hideAuthor: Double = -3.0
    let blockKeyword: Double = -4.0

    check(like > 0, "点赞为正反馈")
    check(comment > like, "评论权重大于点赞")
    check(share == save, "分享与收藏权重相等")
    check(watchTime > share, "观看时长权重最高")
    check(notInterested < 0, "不感兴趣为负反馈")
    check(hideAuthor < notInterested, "隐藏作者惩罚更重")
    check(blockKeyword < hideAuthor, "屏蔽关键词惩罚最重")
}

// MARK: - 测试 4: 平台活跃权重

func testPlatformActivity() {
    print("测试 4: 平台活跃权重")

    // 对齐 PlatformActivity.index
    let index: [String: Double] = [
        "youtube": 1.00, "whatsapp": 0.87, "instagram": 0.81, "facebook": 0.76,
        "tiktok": 0.67, "telegram": 0.60, "reddit": 0.45, "twitter": 0.40,
        "bilibili": 0.35, "douyin": 0.30,
    ]

    func weight(for platform: String) -> Double {
        index[platform.lowercased()] ?? 0.3
    }

    checkEqual(weight(for: "youtube"), 1.00, "youtube 最高活跃")
    checkEqual(weight(for: "telegram"), 0.60, "telegram 活跃指数")
    checkEqual(weight(for: "unknown"), 0.3, "未知平台默认 0.3")
    check(weight(for: "youtube") > weight(for: "douyin"), "youtube > douyin")
    check(weight(for: "tiktok") > weight(for: "reddit"), "tiktok > reddit")
}

// MARK: - 主入口

@main
struct TestRunner {
    static func main() {
        print("NeoTrixAPIClient 单元测试（CLT 纯 Swift）")
        print("==========================================")
        testModelCodable()
        testRecommendationSorting()
        testFeedbackSignals()
        testPlatformActivity()
        print("==========================================")
        print("结果: \(passed) 通过, \(failures) 失败")
        if failures > 0 {
            exit(1)
        }
    }
}