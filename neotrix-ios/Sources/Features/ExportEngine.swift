// ExportEngine - 导出到 AI/KB (Fusion Architecture)
// 融合: Nicegram Export to LLM + NeoTrix KB 知识库
// 独有: 导出到 NeoTrix KB (VSA 向量沉淀) + AI 总结 + 关键决策提取

import Foundation
import Combine
import SwiftUI
import NeoTrixFFI

// MARK: - Export Models

public struct ExportResult {
    public let summary: String
    public let keyPoints: [String]
    public let decisions: [String]
    public let storedInKB: Bool
}

// MARK: - Export Engine

@MainActor
public final class ExportEngine: ObservableObject {
    @Published public var isExporting = false
    @Published public var lastExport: ExportResult?
    @Published public var exportHistory: [ExportRecord] = []
    
    private let core = NeoGramCore.shared
    private let aiHub = AIHub.shared
    
    public struct ExportRecord: Identifiable {
        public let id = UUID()
        public let date: Date
        public let chatTitle: String
        public let messageCount: Int
        public let summary: String
    }
    
    public init() {}
    
    // MARK: - Export to AI
    
    public func exportChat(messages: [ChatMessage], chatTitle: String) async -> ExportResult {
        isExporting = true
        defer { isExporting = false }
        
        let texts = messages.map { $0.text }
        let joined = texts.joined(separator: "\n")
        
        // 1. AI summary
        let summaryResult = await aiHub.process(AIHubRequest(text: joined, type: .summarize))
        
        // 2. Key decisions extraction
        let decisions = await extractKeyDecisions(from: joined)
        
        // 3. Store in KB (VSA embedding)
        let stored = storeInKB(message: joined, summary: summaryResult.text, chatTitle: chatTitle)
        
        let result = ExportResult(
            summary: summaryResult.text,
            keyPoints: summaryResult.suggestions,
            decisions: decisions,
            storedInKB: stored
        )
        
        lastExport = result
        exportHistory.append(ExportRecord(
            date: Date(),
            chatTitle: chatTitle,
            messageCount: messages.count,
            summary: summaryResult.text
        ))
        
        return result
    }
    
    private func extractKeyDecisions(from text: String) async -> [String] {
        guard let e8 = core.e8Reasoning else { return [] }
        
        do {
            let request = ReasoningRequest(
                query: "Extract key decisions from this conversation, one per line:\n\(text)",
                context: "Decision extraction",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            return response.reasoningChain
        } catch {
            return []
        }
    }
    
    private func storeInKB(message: String, summary: String, chatTitle: String) -> Bool {
        // Store conversation summary into NeoTrix KB via VSA embedding
        guard let vsa = core.vsaHyperCube else { return false }
        
        do {
            let vector = vsa.randomVector(label: "\(chatTitle)_\(Date().timeIntervalSince1970)")
            return vsa.store(label: summary, vector: vector)
        } catch {
            return false
        }
    }
}

// MARK: - Export View

public struct ExportView: View {
    @StateObject private var engine = ExportEngine()
    let messages: [ChatMessage]
    let chatTitle: String
    
    @Environment(\.dismiss) private var dismiss
    
    public init(messages: [ChatMessage], chatTitle: String) {
        self.messages = messages
        self.chatTitle = chatTitle
    }
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                if engine.isExporting {
                    ProgressView("Analyzing conversation…")
                        .padding()
                } else if let result = engine.lastExport {
                    ScrollView {
                        VStack(alignment: .leading, spacing: 16) {
                            // Summary
                            VStack(alignment: .leading, spacing: 8) {
                                Label("AI Summary", systemImage: "text.alignleft")
                                    .font(.headline)
                                Text(result.summary)
                                    .font(.body)
                                    .padding()
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .background(NeoTrixTheme.Colors.selection)
                                    .clipShape(RoundedRectangle(cornerRadius: 12))
                            }
                            
                            // Key points
                            if !result.keyPoints.isEmpty {
                                VStack(alignment: .leading, spacing: 8) {
                                    Label("Key Points", systemImage: "list.bullet")
                                        .font(.headline)
                                    ForEach(Array(result.keyPoints.enumerated()), id: \.offset) { index, point in
                                        HStack(alignment: .top, spacing: 8) {
                                            Text("\(index + 1).")
                                                .font(.subheadline.bold())
                                            Text(point)
                                                .font(.subheadline)
                                        }
                                    }
                                }
                            }
                            
                            // Decisions
                            if !result.decisions.isEmpty {
                                VStack(alignment: .leading, spacing: 8) {
                                    Label("Key Decisions", systemImage: "checkmark.seal.fill")
                                        .font(.headline)
                                    ForEach(result.decisions, id: \.self) { decision in
                                        HStack(alignment: .top, spacing: 8) {
                                            Image(systemName: "checkmark.circle.fill")
                                                .foregroundColor(NeoTrixTheme.Colors.success)
                                            Text(decision)
                                                .font(.subheadline)
                                        }
                                    }
                                }
                            }
                            
                            // KB status
                            HStack {
                                Image(systemName: result.storedInKB ? "checkmark.circle.fill" : "xmark.circle.fill")
                                    .foregroundColor(result.storedInKB ? NeoTrixTheme.Colors.success : NeoTrixTheme.Colors.danger)
                                Text(result.storedInKB ? "Stored in NeoTrix KB" : "KB unavailable — summary only")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                        .padding()
                    }
                } else {
                    VStack(spacing: 12) {
                        Image(systemName: "square.and.arrow.up")
                            .font(.system(size: 48))
                            .foregroundColor(NeoTrixTheme.Colors.accent)
                        Text("Export \(messages.count) messages to AI")
                            .font(.headline)
                        Text("Get a summary, key points, and decisions. Optionally stored in NeoTrix KB for future recall.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal)
                    }
                    .padding()
                }
                
                Spacer()
            }
            .navigationTitle("Export to AI")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    if engine.lastExport == nil && !engine.isExporting {
                        Button("Export") {
                            Task { await engine.exportChat(messages: messages, chatTitle: chatTitle) }
                        }
                    }
                }
                #else
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    if engine.lastExport == nil && !engine.isExporting {
                        Button("Export") {
                            Task { await engine.exportChat(messages: messages, chatTitle: chatTitle) }
                        }
                    }
                }
                #endif
            }
        }
    }
}