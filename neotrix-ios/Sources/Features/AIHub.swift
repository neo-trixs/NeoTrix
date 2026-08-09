// AIHub - AI 中枢 (Fusion Architecture ADR-001)
// 统一路由所有 AI 请求（编辑/翻译/总结/过滤/推荐），通过 GWT 注意力路由分发
// 到 E8/VSA/ConsciousnessTree。Rust 核心未初始化时降级为本地规则。

import Foundation
import Combine
import NeoTrixFFI

// MARK: - AI Request

public struct AIHubRequest {
    public let text: String
    public let type: AIRequestType
    
    public init(text: String, type: AIRequestType) {
        self.text = text
        self.type = type
    }
}

public enum AIRequestType {
    case edit(AIEditAction)
    case translate(String)      // target language
    case summarize
    case filter
    case recommend
    case style
    case classify
}

// MARK: - AI Result

public struct AIHubResult {
    public let text: String
    public let confidence: Double
    public let source: AISource
    public let suggestions: [String]
    
    public enum AISource {
        case e8Reasoning
        case vsaRecall
        case consciousnessTree
        case localRule   // degraded fallback
    }
}

// MARK: - AI Hub

@MainActor
public final class AIHub: ObservableObject {
    public static let shared = AIHub()
    
    @Published public var isEnabled = true
    @Published public var isProcessing = false
    @Published public var lastError: String?
    
    private let core = NeoGramCore.shared
    private var cancellables = Set<AnyCancellable>()
    
    public init() {
        registerWithGWT()
    }
    
    private func registerWithGWT() {
        // GWT attention routing: AIHub listens for all AI-related broadcasts
        core.gwtAttention?.registerModule(
            name: "AIHub",
            interestKeywords: ["edit", "translate", "summarize", "filter", "recommend", "compose", "reply"]
        )
    }
    
    // MARK: - Unified AI Dispatch
    
    public func process(_ request: AIHubRequest) async -> AIHubResult {
        isProcessing = true
        defer { isProcessing = false }
        
        switch request.type {
        case .edit(let action):
            return await routeEdit(request.text, action: action)
        case .translate(let target):
            return await routeTranslate(request.text, target: target)
        case .summarize:
            return await routeSummarize(request.text)
        case .filter:
            return await routeFilter(request.text)
        case .recommend:
            return await routeRecommend(request.text)
        case .style:
            return await routeStyle(request.text)
        case .classify:
            return await routeClassify(request.text)
        }
    }
    
    // MARK: - Route: Edit
    
    private func routeEdit(_ text: String, action: AIEditAction) async -> AIHubResult {
        guard let e8 = core.e8Reasoning else {
            return degradedResult("AI not available", original: text)
        }
        
        do {
            let request = ReasoningRequest(
                query: action.prompt + text,
                context: "AI message editor",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            return AIHubResult(
                text: response.conclusion,
                confidence: Double(response.confidence),
                source: .e8Reasoning,
                suggestions: response.reasoningChain
            )
        } catch {
            lastError = error.localizedDescription
            return degradedResult(error.localizedDescription, original: text)
        }
    }
    
    // MARK: - Route: Translate
    
    private func routeTranslate(_ text: String, target: String) async -> AIHubResult {
        guard let e8 = core.e8Reasoning else {
            return degradedResult("Translation unavailable", original: text)
        }
        
        do {
            let request = ReasoningRequest(
                query: "Translate to \(target): \(text)",
                context: "Message translation",
                maxDepth: 2,
                useConsciousness: false
            )
            let response = try e8.reason(request: request)
            return AIHubResult(
                text: response.conclusion,
                confidence: Double(response.confidence),
                source: .e8Reasoning,
                suggestions: []
            )
        } catch {
            return degradedResult("Translation unavailable", original: text)
        }
    }
    
    // MARK: - Route: Summarize
    
    private func routeSummarize(_ text: String) async -> AIHubResult {
        guard let e8 = core.e8Reasoning else {
            return degradedResult("Summarization unavailable", original: text)
        }
        
        do {
            let request = ReasoningRequest(
                query: "Summarize concisely:\n\(text)",
                context: "Conversation summary",
                maxDepth: 3,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            return AIHubResult(
                text: response.conclusion,
                confidence: Double(response.confidence),
                source: .consciousnessTree,
                suggestions: response.reasoningChain
            )
        } catch {
            return degradedResult("Summarization unavailable", original: text)
        }
    }
    
    // MARK: - Route: Filter (AI semantic filter)
    
    private func routeFilter(_ text: String) async -> AIHubResult {
        // Semantic spam/ad classification via E8
        guard let e8 = core.e8Reasoning else {
            // Local rule fallback: keyword-based
            let spamKeywords = ["spam", "ad", "promo", "discount", "free money", "click here"]
            let isSpam = spamKeywords.contains { text.lowercased().contains($0) }
            return AIHubResult(
                text: isSpam ? "spam" : "clean",
                confidence: isSpam ? 0.8 : 0.5,
                source: .localRule,
                suggestions: []
            )
        }
        
        do {
            let request = ReasoningRequest(
                query: "Classify this message as spam or clean. Reply 'spam' or 'clean':\n\(text)",
                context: "Message filtering",
                maxDepth: 1,
                useConsciousness: false
            )
            let response = try e8.reason(request: request)
            let classification = response.conclusion.lowercased()
            let isSpam = classification.contains("spam")
            return AIHubResult(
                text: isSpam ? "spam" : "clean",
                confidence: Double(response.confidence),
                source: .e8Reasoning,
                suggestions: []
            )
        } catch {
            return degradedResult("clean", original: text)
        }
    }
    
    // MARK: - Route: Recommend
    
    private func routeRecommend(_ context: String) async -> AIHubResult {
        // VSA recall for related content recommendations
        guard let vsa = core.vsaHyperCube else {
            return degradedResult("Recommendation unavailable", original: context)
        }
        
        do {
            // Retrieve related vectors via VSA batch operation
            let queryVector = vsa.randomVector(label: context)
            let ops = [
                VsaOperation(
                    opType: "retrieve",
                    vectors: [queryVector],
                    parameters: ["top_k": "5"]
                )
            ]
            let results = try vsa.batchOperation(ops: ops)
            let suggestions = results.map { "\($0.similarityScores.first ?? 0)" }
            return AIHubResult(
                text: suggestions.joined(separator: "\n"),
                confidence: 0.7,
                source: .vsaRecall,
                suggestions: suggestions
            )
        } catch {
            return degradedResult("Recommendation unavailable", original: context)
        }
    }
    
    // MARK: - Route: Style
    
    private func routeStyle(_ text: String) async -> AIHubResult {
        guard let e8 = core.e8Reasoning else {
            return degradedResult("Style unavailable", original: text)
        }
        
        do {
            let request = ReasoningRequest(
                query: "Apply custom style: \(text)",
                context: "AI style transformation",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            return AIHubResult(
                text: response.conclusion,
                confidence: Double(response.confidence),
                source: .e8Reasoning,
                suggestions: []
            )
        } catch {
            return degradedResult("Style unavailable", original: text)
        }
    }
    
    // MARK: - Route: Classify (importance/priority)
    
    private func routeClassify(_ text: String) async -> AIHubResult {
        guard let e8 = core.e8Reasoning else {
            // Local fallback: keyword importance
            let importantKeywords = ["urgent", "asap", "important", "紧急", "重要", "deadline"]
            let isImportant = importantKeywords.contains { text.lowercased().contains($0) }
            return AIHubResult(
                text: isImportant ? "important" : "normal",
                confidence: isImportant ? 0.7 : 0.4,
                source: .localRule,
                suggestions: []
            )
        }
        
        do {
            let request = ReasoningRequest(
                query: "Classify message importance as 'important' or 'normal':\n\(text)",
                context: "Smart notification routing",
                maxDepth: 1,
                useConsciousness: false
            )
            let response = try e8.reason(request: request)
            let classification = response.conclusion.lowercased()
            let isImportant = classification.contains("important")
            return AIHubResult(
                text: isImportant ? "important" : "normal",
                confidence: Double(response.confidence),
                source: .e8Reasoning,
                suggestions: []
            )
        } catch {
            return degradedResult("normal", original: text)
        }
    }
    
    // MARK: - Degradation
    
    private func degradedResult(_ text: String, original: String) -> AIHubResult {
        return AIHubResult(
            text: text,
            confidence: 0,
            source: .localRule,
            suggestions: []
        )
    }
}