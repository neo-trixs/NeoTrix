import Foundation

actor NeoTrixAPI {
    static let shared = NeoTrixAPI()
    
    var baseURL: String {
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
    
    // MARK: - Chat
    
    func chat(message: String) async throws -> String {
        let req = ChatRequest(message: message)
        let data = try await post("/api/v1/chat", body: req)
        let resp = try decoder.decode(ChatResponse.self, from: data)
        return resp.reply
    }
    
    func chatStream(message: String) -> AsyncThrowingStream<String, Error> {
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
    
    func scoreMoments(_ moments: [VideoSubmission]) async throws -> [MomentItem] {
        let req = ScoreRequest(moments: moments)
        let data = try await post("/api/v1/moments/score", body: req)
        return try decoder.decode([MomentItem].self, from: data)
    }
    
    func scoreMomentsStream(_ moments: [VideoSubmission]) -> AsyncThrowingStream<MomentItem, Error> {
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
    
    func sendFeedback(momentId: String, liked: Bool, keywords: [String]? = nil) async throws {
        let req = FeedbackRequest(momentId: momentId, liked: liked, keywords: keywords)
        _ = try await post("/api/v1/moments/feedback", body: req)
    }
    
    // MARK: - Social
    
    func socialStatus() async throws -> [SocialStatus] {
        let data = try await get("/api/v1/social/status")
        return try decoder.decode([SocialStatus].self, from: data)
    }
    
    func socialLogin(platform: String, token: String, refreshToken: String? = nil) async throws {
        let req = SocialLoginRequest(platform: platform, token: token, refreshToken: refreshToken)
        _ = try await post("/api/v1/social/login", body: req)
    }
    
    // MARK: - Filter
    
    func filterConfig() async throws -> FilterConfig {
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

enum APIError: Error, LocalizedError {
    case invalidResponse
    case notFound
    case serverError(String)
    
    var errorDescription: String? {
        switch self {
        case .invalidResponse: return "Invalid server response"
        case .notFound: return "Endpoint not found"
        case .serverError(let msg): return msg
        }
    }
}
