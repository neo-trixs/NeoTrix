// NeoTrixAIIntegration - NeoTrix AI features in NeoGram
// Wraps the Rust FFI bridge for AI-powered messaging

import Foundation
import Combine
import SwiftUI
import NeoTrixFFI

// MARK: - AI Feature Models

public struct AICompose {
    public let text: String
    public let confidence: Double
    public let suggestions: [String]
}

public struct AISummary {
    public let summary: String
    public let keyPoints: [String]
    public let sentiment: Sentiment
    
    public enum Sentiment: String {
        case positive = "Positive"
        case neutral = "Neutral"
        case negative = "Negative"
    }
}

public struct AITranslation {
    public let original: String
    public let translated: String
    public let sourceLanguage: String
    public let targetLanguage: String
}

// MARK: - AI Integration Manager

@MainActor
public final class NeoTrixAIManager: ObservableObject {
    @Published public var isEnabled = true
    @Published public var isProcessing = false
    @Published public var lastError: String?
    
    private let core = NeoGramCore.shared
    private var cancellables = Set<AnyCancellable>()
    
    public init() {
        setupCore()
    }
    
    private func setupCore() {
        // Register AI module with GWT attention routing
        core.gwtAttention?.registerModule(
            name: "NeoTrixAI",
            interestKeywords: ["compose", "translate", "summarize", "reply"]
        )
    }
    
    // MARK: - AI Compose
    
    public func composeMessage(prompt: String, style: String = "friendly") async -> AICompose {
        isProcessing = true
        defer { isProcessing = false }
        
        guard let e8 = core.e8Reasoning else {
            return AICompose(text: prompt, confidence: 0, suggestions: [])
        }
        
        do {
            let request = ReasoningRequest(
                query: "Compose a \(style) message: \(prompt)",
                context: "Message composition",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            
            return AICompose(
                text: response.conclusion,
                confidence: Double(response.confidence),
                suggestions: response.reasoningChain
            )
        } catch {
            lastError = error.localizedDescription
            return AICompose(text: prompt, confidence: 0, suggestions: [])
        }
    }
    
    // MARK: - AI Summary
    
    public func summarize(messages: [String]) async -> AISummary {
        isProcessing = true
        defer { isProcessing = false }
        
        guard let e8 = core.e8Reasoning else {
            return AISummary(summary: "AI not available", keyPoints: [], sentiment: .neutral)
        }
        
        let joined = messages.joined(separator: "\n")
        
        do {
            let request = ReasoningRequest(
                query: "Summarize this conversation:\n\(joined)",
                context: "Chat summary",
                maxDepth: 3,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            
            return AISummary(
                summary: response.conclusion,
                keyPoints: response.reasoningChain,
                sentiment: detectSentiment(response.conclusion)
            )
        } catch {
            lastError = error.localizedDescription
            return AISummary(summary: "Unable to summarize", keyPoints: [], sentiment: .neutral)
        }
    }
    
    // MARK: - AI Translation
    
    public func translate(text: String, to targetLanguage: String) async -> AITranslation {
        guard let e8 = core.e8Reasoning else {
            return AITranslation(original: text, translated: text, sourceLanguage: "unknown", targetLanguage: targetLanguage)
        }
        
        do {
            let request = ReasoningRequest(
                query: "Translate to \(targetLanguage): \(text)",
                context: "Translation",
                maxDepth: 1,
                useConsciousness: false
            )
            let response = try e8.reason(request: request)
            
            return AITranslation(
                original: text,
                translated: response.conclusion,
                sourceLanguage: "auto",
                targetLanguage: targetLanguage
            )
        } catch {
            lastError = error.localizedDescription
            return AITranslation(original: text, translated: text, sourceLanguage: "auto", targetLanguage: targetLanguage)
        }
    }
    
    // MARK: - Smart Reply
    
    public func smartReply(to message: String) async -> String? {
        guard let e8 = core.e8Reasoning else { return nil }
        
        do {
            let request = ReasoningRequest(
                query: "Generate a smart reply to: \(message)",
                context: "Smart reply",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            return response.conclusion
        } catch {
            return nil
        }
    }
    
    // MARK: - AI Memory
    
    public func remember(_ fact: String) {
        var facts = UserDefaults.standard.array(forKey: "ai_memory_facts") as? [String] ?? []
        facts.append(fact)
        UserDefaults.standard.set(facts, forKey: "ai_memory_facts")
    }
    
    public func recall(query: String) -> String? {
        let facts = UserDefaults.standard.array(forKey: "ai_memory_facts") as? [String] ?? []
        return facts.first { $0.localizedCaseInsensitiveContains(query) }
    }
    
    private var e8: E8ReasoningImpl? {
        core.e8Reasoning
    }
    
    private func detectSentiment(_ text: String) -> AISummary.Sentiment {
        let positive = ["good", "great", "love", "happy", "excellent", "amazing", "wonderful", "best"]
        let negative = ["bad", "terrible", "hate", "awful", "worst", "horrible", "sad", "angry"]
        let lower = text.lowercased()
        
        let positiveCount = positive.filter { lower.contains($0) }.count
        let negativeCount = negative.filter { lower.contains($0) }.count
        
        if positiveCount > negativeCount { return .positive }
        if negativeCount > positiveCount { return .negative }
        return .neutral
    }
}

// MARK: - AI Compose UI

public struct AIComposeView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var aiManager = NeoTrixAIManager()
    
    @State private var prompt = ""
    @State private var style = "friendly"
    @State private var result: AICompose?
    @State private var isGenerating = false
    
    let onUse: (String) -> Void
    
    public init(onUse: @escaping (String) -> Void) {
        self.onUse = onUse
    }
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                // Prompt input
                TextField("Describe what to write...", text: $prompt, axis: .vertical)
                    .textFieldStyle(.roundedBorder)
                    .lineLimit(3...6)
                    .padding(.horizontal)
                
                // Style selector
                Picker("Style", selection: $style) {
                    Text("Friendly").tag("friendly")
                    Text("Professional").tag("professional")
                    Text("Casual").tag("casual")
                    Text("Formal").tag("formal")
                }
                .pickerStyle(.segmented)
                .padding(.horizontal)
                
                // Generate button
                Button {
                    generate()
                } label: {
                    if isGenerating {
                        ProgressView()
                    } else {
                        Label("Generate", systemImage: "wand.and.stars")
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(prompt.isEmpty || isGenerating)
                
                // Result
                if let result {
                    VStack(alignment: .leading, spacing: 12) {
                        Text(result.text)
                            .padding()
                            .background(NeoTrixTheme.Colors.selection)
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                        
                        // Suggestions
                        if !result.suggestions.isEmpty {
                            Text("Suggestions")
                                .font(.headline)
                            
                            ForEach(result.suggestions, id: \.self) { suggestion in
                                Button {
                                    onUse(suggestion)
                                    dismiss()
                                } label: {
                                    Text(suggestion)
                                        .padding()
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .background(NeoTrixTheme.Colors.selection)
                                        .clipShape(RoundedRectangle(cornerRadius: 8))
                                }
                                .buttonStyle(.plain)
                            }
                        }
                        
                        Button("Use This") {
                            onUse(result.text)
                            dismiss()
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    .padding(.horizontal)
                }
                
                Spacer()
            }
            .navigationTitle("AI Compose")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Cancel") { dismiss() }
                }
                #else
                ToolbarItem(placement: .primaryAction) {
                    Button("Cancel") { dismiss() }
                }
                #endif
            }
        }
    }
    
    private func generate() {
        isGenerating = true
        Task {
            result = await aiManager.composeMessage(prompt: prompt, style: style)
            isGenerating = false
        }
    }
}

// MARK: - AI Summary UI

public struct AISummaryView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var manager = NeoTrixAIManager()
    
    let messages: [String]
    @State private var summary: AISummary?
    
    public init(messages: [String]) {
        self.messages = messages
    }
    
    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    if let summary {
                        // Sentiment badge
                        HStack {
                            Text(summary.sentiment.rawValue)
                                .font(.caption.bold())
                                .padding(.horizontal, 10)
                                .padding(.vertical, 4)
                                .background(sentimentColor)
                                .clipShape(Capsule())
                            Spacer()
                        }
                        
                        // Summary text
                        Text(summary.summary)
                            .font(.body)
                            .padding()
                            .background(NeoTrixTheme.Colors.selection)
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                        
                        // Key points
                        if !summary.keyPoints.isEmpty {
                            Text("Key Points")
                                .font(.headline)
                            
                            ForEach(Array(summary.keyPoints.enumerated()), id: \.offset) { index, point in
                                HStack(alignment: .top, spacing: 8) {
                                    Text("\(index + 1).")
                                        .font(.subheadline.bold())
                                    Text(point)
                                        .font(.subheadline)
                                }
                                .padding(.vertical, 4)
                            }
                        }
                    } else {
                        ProgressView("Generating summary...")
                            .frame(maxWidth: .infinity)
                            .padding(.top, 100)
                    }
                }
                .padding()
            }
            .navigationTitle("AI Summary")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") { dismiss() }
                }
                #else
                ToolbarItem(placement: .primaryAction) {
                    Button("Done") { dismiss() }
                }
                #endif
            }
            .task {
                summary = await manager.summarize(messages: messages)
            }
        }
    }
    
    private var sentimentColor: Color {
        switch summary?.sentiment {
        case .positive: return NeoTrixTheme.Colors.success.opacity(0.2)
        case .negative: return NeoTrixTheme.Colors.danger.opacity(0.2)
        default: return NeoTrixTheme.Colors.selection
        }
    }
}