// NeoTrixAPIClient - 融合: 从 NeoTrix/ 死代码体系提炼的 HTTP 服务层
// 原 NeoTrix/Services/NeoTrixAPI.swift（未被 Bazel 编译）迁移至此并补全模型类型。
// 作为 LiveFeedEngine 的真实内容源（替代 mock 降级），Dark Forest: 有消费者。

import Foundation

// MARK: - API 模型类型（补全: 原 NeoTrix/ 体系引用但未定义）

public struct ChatRequest: Codable {
    public let message: String
    public init(message: String) { self.message = message }
}

public struct ChatResponse: Codable {
    public let reply: String
}

public struct VideoSubmission: Codable, Equatable {
    public let id: String
    public let title: String
    public let author: String?
    public let duration: Double?
    public let viewCount: Int64?
    public let likeCount: Int64?
    public let url: String?
    public init(id: String, title: String, author: String? = nil, duration: Double? = nil,
                viewCount: Int64? = nil, likeCount: Int64? = nil, url: String? = nil) {
        self.id = id
        self.title = title
        self.author = author
        self.duration = duration
        self.viewCount = viewCount
        self.likeCount = likeCount
        self.url = url
    }
}

public struct ScoreRequest: Codable {
    public let moments: [VideoSubmission]
    public init(moments: [VideoSubmission]) { self.moments = moments }
}

public struct MomentItem: Codable, Equatable, Identifiable {
    public let id: String
    public let title: String
    public let author: String?
    public let score: Double?
    public let reason: String?
    public let createdAt: String?
    public init(id: String, title: String, author: String? = nil, score: Double? = nil,
                reason: String? = nil, createdAt: String? = nil) {
        self.id = id
        self.title = title
        self.author = author
        self.score = score
        self.reason = reason
        self.createdAt = createdAt
    }
}

public struct FeedbackRequest: Codable {
    public let momentId: String
    public let liked: Bool
    public let keywords: [String]?
    public init(momentId: String, liked: Bool, keywords: [String]? = nil) {
        self.momentId = momentId
        self.liked = liked
        self.keywords = keywords
    }
}

public struct SocialStatus: Codable, Equatable, Identifiable {
    public let id: String
    public let platform: String
    public let isConnected: Bool
    public let username: String?
    public let followers: Int64?
    public init(id: String, platform: String, isConnected: Bool, username: String? = nil, followers: Int64? = nil) {
        self.id = id
        self.platform = platform
        self.isConnected = isConnected
        self.username = username
        self.followers = followers
    }
}

public struct SocialLoginRequest: Codable {
    public let platform: String
    public let token: String
    public let refreshToken: String?
    public init(platform: String, token: String, refreshToken: String? = nil) {
        self.platform = platform
        self.token = token
        self.refreshToken = refreshToken
    }
}

public struct FilterConfig: Codable, Equatable {
    public let filterAds: Bool
    public let filterKeywords: [String]
    public init(filterAds: Bool = true, filterKeywords: [String] = []) {
        self.filterAds = filterAds
        self.filterKeywords = filterKeywords
    }
}

// MARK: - API Client

public actor NeoTrixAPIClient {
    public static let shared = NeoTrixAPIClient()
    
    public var baseURL: String {
        UserDefaults.standard.string(forKey: "server_url") ?? "http://localhost:3000"
    }
    
    private let decoder: JSONDecoder = {
        let d = JSONDecoder()
        d.keyDecodingStrategy = .convertFromSnakeCase
        return d
    }()
    
    private let encoder: JSONEncoder = {
        let e = JSONEncoder()
        e.keyEncodingStrategy = .convertToSnakeCase
        return e
    }()
    
    public init() {}
    
    // MARK: - Chat
    
    public func chat(message: String) async throws -> String {
        let req = ChatRequest(message: message)
        let data = try await post("/api/v1/chat", body: req)
        let resp = try decoder.decode(ChatResponse.self, from: data)
        return resp.reply
    }
    
    public func chatStream(message: String) -> AsyncThrowingStream<String, Error> {
        AsyncThrowingStream { continuation in
            Task {
                do {
                    let req = ChatRequest(message: message)
                    let body = try encoder.encode(req)
                    var urlReq = URLRequest(url: URL(string: "\(baseURL)/api/v1/chat/stream")!)
                    urlReq.httpMethod = "POST"
                    urlReq.setValue("application/json", forHTTPHeaderField: "Content-Type")
                    urlReq.httpBody = body
                    
                    let (bytes, _) = try await URLSession.shared.bytes(for: urlReq)
                    for try await byte in bytes.lines {
                        if !byte.isEmpty, byte.hasPrefix("data:") {
                            let char = byte.dropFirst(5).trimmingCharacters(in: .whitespaces)
                            if !char.isEmpty {
                                continuation.yield(char)
                            }
                        }
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }
    
    // MARK: - Moments
    
    public func scoreMoments(_ moments: [VideoSubmission]) async throws -> [MomentItem] {
        let req = ScoreRequest(moments: moments)
        let data = try await post("/api/v1/moments/score", body: req)
        return try decoder.decode([MomentItem].self, from: data)
    }
    
    public func scoreMomentsStream(_ moments: [VideoSubmission]) -> AsyncThrowingStream<MomentItem, Error> {
        AsyncThrowingStream { continuation in
            Task {
                do {
                    let req = ScoreRequest(moments: moments)
                    let body = try encoder.encode(req)
                    var urlReq = URLRequest(url: URL(string: "\(baseURL)/api/v1/moments/score-stream")!)
                    urlReq.httpMethod = "POST"
                    urlReq.setValue("application/json", forHTTPHeaderField: "Content-Type")
                    urlReq.httpBody = body
                    
                    let (bytes, _) = try await URLSession.shared.bytes(for: urlReq)
                    for try await line in bytes.lines {
                        if line.hasPrefix("data:") {
                            let json = String(line.dropFirst(5)).trimmingCharacters(in: .whitespaces)
                            if let data = json.data(using: .utf8),
                               let item = try? decoder.decode(MomentItem.self, from: data) {
                                continuation.yield(item)
                            }
                        }
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }
    
    public func sendFeedback(momentId: String, liked: Bool, keywords: [String]? = nil) async throws {
        let req = FeedbackRequest(momentId: momentId, liked: liked, keywords: keywords)
        _ = try await post("/api/v1/moments/feedback", body: req)
    }
    
    // MARK: - Social
    
    public func socialStatus() async throws -> [SocialStatus] {
        let data = try await get("/api/v1/social/status")
        return try decoder.decode([SocialStatus].self, from: data)
    }
    
    public func socialLogin(platform: String, token: String, refreshToken: String? = nil) async throws {
        let req = SocialLoginRequest(platform: platform, token: token, refreshToken: refreshToken)
        _ = try await post("/api/v1/social/login", body: req)
    }
    
    // MARK: - Filter
    
    public func filterConfig() async throws -> FilterConfig {
        let data = try await get("/api/v1/filter/config")
        return try decoder.decode(FilterConfig.self, from: data)
    }
    
    // MARK: - HTTP
    
    private func get(_ path: String) async throws -> Data {
        let url = URL(string: "\(baseURL)\(path)")!
        let (data, resp) = try await URLSession.shared.data(from: url)
        guard let httpResp = resp as? HTTPURLResponse, (200...299).contains(httpResp.statusCode) else {
            throw APIError.invalidResponse
        }
        return data
    }
    
    private func post<T: Encodable>(_ path: String, body: T) async throws -> Data {
        let url = URL(string: "\(baseURL)\(path)")!
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try encoder.encode(body)
        let (data, resp) = try await URLSession.shared.data(for: req)
        guard let httpResp = resp as? HTTPURLResponse, (200...299).contains(httpResp.statusCode) else {
            throw APIError.invalidResponse
        }
        return data
    }
}

public enum APIError: Error, LocalizedError {
    case invalidResponse
    case notFound
    case serverError(String)
    
    public var errorDescription: String? {
        switch self {
        case .invalidResponse: return "Invalid server response"
        case .notFound: return "Endpoint not found"
        case .serverError(let msg): return msg
        }
    }
}
