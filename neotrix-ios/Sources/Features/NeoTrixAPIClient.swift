// NeoTrixAPIClient - 融合: 从 NeoTrix/ 死代码体系提炼的 HTTP 服务层
// 对接 NeoTrix Rust 服务真实端点（nt_io_web/server.rs）：
//   - POST /api/brain/reason        {"prompt"} → {"output","success"}    LLM 推理
//   - POST /api/agent/reason-stream  {"prompt"} → SSE {"token","done"}   流式推理
//   - GET  /api/brain/stats          服务健康/能力探测
// Dark Forest: 消费面 = LiveFeedEngine.NeoTrixAPIProvider（社交状态探测 + 搜索过滤配置）。
// 注: scoreMoments/sendFeedback/socialLogin 原指向 /api/v1/* 不存在的端点，已删除（服务端无此能力）。

import Foundation

// MARK: - API 模型类型（与服务端契约对齐）

/// /api/brain/reason 响应 — 服务端返回 {"output": "...", "success": true/false}
public struct ReasonResponse: Codable {
    public let output: String
    public let success: Bool?
    public init(output: String, success: Bool? = nil) {
        self.output = output
        self.success = success
    }
}

/// 社交平台连接状态（LiveFeed 内容源探测用；本地 mock + 服务可用性门控）
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

/// 搜索过滤配置（LiveFeed 搜索时取关键词）
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
    
    // MARK: - LLM 推理（对接 /api/brain/reason）
    
    public func reason(prompt: String) async throws -> String {
        let body: [String: String] = ["prompt": prompt]
        let data = try await post("/api/brain/reason", body: body)
        let resp = try decoder.decode(ReasonResponse.self, from: data)
        return resp.output
    }
    
    // MARK: - 流式推理（对接 /api/agent/reason-stream）
    
    public func reasonStream(prompt: String) -> AsyncThrowingStream<String, Error> {
        AsyncThrowingStream { continuation in
            Task {
                do {
                    let body: [String: String] = ["prompt": prompt]
                    var urlReq = URLRequest(url: URL(string: "\(baseURL)/api/agent/reason-stream")!)
                    urlReq.httpMethod = "POST"
                    urlReq.setValue("application/json", forHTTPHeaderField: "Content-Type")
                    urlReq.httpBody = try encoder.encode(body)
                    
                    let (bytes, _) = try await URLSession.shared.bytes(for: urlReq)
                    for try await line in bytes.lines {
                        guard line.hasPrefix("data:") else { continue }
                        let json = String(line.dropFirst(5)).trimmingCharacters(in: .whitespaces)
                        guard let data = json.data(using: .utf8),
                              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { continue }
                        if let token = obj["token"] as? String {
                            continuation.yield(token)
                        } else if let error = obj["error"] as? String {
                            continuation.finish(throwing: APIError.serverError(error))
                            return
                        }
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
        }
    }
    
    // MARK: - 服务可用性探测（对接 /api/brain/stats）
    
    /// 探测 NeoTrix 服务是否在线。在线时返回本地社交状态（标记 NeoTrix 已连接），
    /// 离线抛错 → LiveFeedEngine 降级 mock（The Spice Must Flow: 无断流）。
    public func socialStatus() async throws -> [SocialStatus] {
        let _ = try await get("/api/brain/stats")
        return [
            SocialStatus(id: "neotrix-server", platform: "neotrix", isConnected: true, username: "neotrix", followers: 0)
        ]
    }
    
    // MARK: - 搜索过滤配置
    
    /// 服务在线时返回默认过滤配置（服务端暂无独立配置端点，使用内置默认值）。
    public func filterConfig() async throws -> FilterConfig {
        let _ = try await get("/api/brain/stats")
        return FilterConfig(filterAds: true, filterKeywords: ["spam", "ad"])
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
    case serverError(String)
    
    public var errorDescription: String? {
        switch self {
        case .invalidResponse:
            return "无效的服务响应"
        case .serverError(let message):
            return "服务错误: \(message)"
        }
    }
}
