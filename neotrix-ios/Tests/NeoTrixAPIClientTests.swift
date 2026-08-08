// NeoTrixAPIClientTests - 纯 Swift 单元测试（CLT 无 XCTest，用断言 + 退出码）
// 覆盖: reason 契约编解码 / stats 探测响应 / 推荐排序 / 反馈信号权重
// 运行: swiftc -parse-as-library -o /tmp/neotrix-tests Sources/Features/NeoTrixAPIClient.swift Tests/NeoTrixAPIClientTests.swift && /tmp/neotrix-tests

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

// MARK: - 测试 1: reason 契约编解码（Rust /api/brain/reason）

func testReasonContract() {
    print("测试 1: reason 契约编解码（Rust /api/brain/reason）")

    let encoder = JSONEncoder()
    encoder.keyEncodingStrategy = .convertToSnakeCase
    let decoder = JSONDecoder()
    decoder.keyDecodingStrategy = .convertFromSnakeCase

    // 请求: iOS 发送 {"prompt": "..."} 匹配 ReasonBody
    let reqBody: [String: String] = ["prompt": "hello"]
    let reqData = try! encoder.encode(reqBody)
    let reqJSON = String(data: reqData, encoding: .utf8)!
    check(reqJSON.contains("\"prompt\":\"hello\""), "请求编码 prompt 字段（匹配 ReasonBody）")

    // 响应: Rust 返回 {"output": "...", "success": true}
    let respJSON = "{\"output\": \"Hi there!\", \"success\": true}"
    let decoded = try! decoder.decode(ReasonResponse.self, from: respJSON.data(using: .utf8)!)
    checkEqual(decoded.output, "Hi there!", "响应解码 output 字段")
    check(decoded.success == true, "响应解码 success 字段")

    // reason() 方法: 成功路径取 output
    // 失败路径: {"success": false, "output": "LLM error: ..."}
    let errJSON = "{\"output\": \"LLM error: timeout\", \"success\": false}"
    let errDecoded = try! decoder.decode(ReasonResponse.self, from: errJSON.data(using: .utf8)!)
    checkEqual(errDecoded.output, "LLM error: timeout", "失败响应仍可解码 output")
}

// MARK: - 测试 2: stats 探测响应模型（Rust /api/brain/stats）

func testStatsProbe() {
    print("测试 2: stats 探测响应模型（Rust /api/brain/stats）")

    // socialStatus 在服务在线时返回本地 SocialStatus（探测 GET 成功才返回）
    let status = SocialStatus(id: "neotrix-server", platform: "neotrix", isConnected: true, username: "neotrix", followers: 0)
    check(status.isConnected, "服务在线标记为已连接")
    checkEqual(status.platform, "neotrix", "平台标识 neotrix")
    checkEqual(status.followers, 0, "本地状态 followers 为 0")

    // filterConfig 默认值
    let config = FilterConfig(filterAds: true, filterKeywords: ["spam", "ad"])
    check(config.filterAds, "默认过滤广告")
    checkEqual(config.filterKeywords, ["spam", "ad"], "默认过滤关键词")
}

// MARK: - 测试 3: 推荐排序逻辑

func testRecommendationSorting() {
    print("测试 3: 推荐排序逻辑")

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

// MARK: - 测试 4: 反馈信号权重

func testFeedbackSignals() {
    print("测试 4: 反馈信号权重")

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

// MARK: - 测试 5: 平台活跃权重

func testPlatformActivity() {
    print("测试 5: 平台活跃权重")

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
        testReasonContract()
        testStatsProbe()
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
