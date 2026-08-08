// AIEditorUI - AI Editor for message composition
// Mirrors Telegram 2026 "AI Editor" (Cocoon AI): translate, transform, fix grammar in 2 taps
// Also covers Swiftgram "Quick Formatting Panel" + Turrit "Translate before sending"

import SwiftUI
import NeoTrixFFI

// MARK: - AI Editor Models

public enum AIEditAction: String, CaseIterable, Identifiable {
    case fixGrammar = "Fix Grammar"
    case translate = "Translate"
    case rewrite = "Rewrite"
    case shorten = "Shorten"
    case formal = "Formal"
    case friendly = "Friendly"
    case zen = "Zen"
    case viking = "Viking"
    
    public var id: String { rawValue }
    
    public var icon: String {
        switch self {
        case .fixGrammar: return "checkmark.seal.fill"
        case .translate: return "character.bubble.fill"
        case .rewrite: return "wand.and.stars"
        case .shorten: return "arrow.down.right.and.arrow.up.left"
        case .formal: return "briefcase.fill"
        case .friendly: return "face.smiling.fill"
        case .zen: return "leaf.fill"
        case .viking: return "shield.lefthalf.filled"
        }
    }
    
    public var prompt: String {
        switch self {
        case .fixGrammar: return "Fix grammar and spelling errors, keep meaning: "
        case .translate: return "Translate to English: "
        case .rewrite: return "Rewrite in a different style: "
        case .shorten: return "Shorten this message: "
        case .formal: return "Rewrite in formal tone: "
        case .friendly: return "Rewrite in friendly tone: "
        case .zen: return "Rewrite in zen, calm style: "
        case .viking: return "Rewrite in bold, viking style: "
        }
    }
}

public struct AIEditResult {
    public let original: String
    public let edited: String
    public let action: AIEditAction
}

// MARK: - AI Editor Manager

@MainActor
public final class AIEditorManager: ObservableObject {
    @Published public var isProcessing = false
    @Published public var lastError: String?
    
    private let core = NeoGramCore.shared
    
    public init() {}
    
    public func edit(_ text: String, action: AIEditAction) async -> String {
        isProcessing = true
        defer { isProcessing = false }
        
        guard let e8 = core.e8Reasoning else {
            return text
        }
        
        do {
            let request = ReasoningRequest(
                query: action.prompt + text,
                context: "AI message editor",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            return response.conclusion
        } catch {
            lastError = error.localizedDescription
            return text
        }
    }
}

// MARK: - AI Editor Panel

public struct AIEditorPanel: View {
    @StateObject private var manager = AIEditorManager()
    @Binding var text: String
    let onApply: (String) -> Void
    
    public init(text: Binding<String>, onApply: @escaping (String) -> Void) {
        self._text = text
        self.onApply = onApply
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            // Action chips
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 8) {
                    ForEach(AIEditAction.allCases) { action in
                        Button {
                            Task { await runAction(action) }
                        } label: {
                            Label(action.rawValue, systemImage: action.icon)
                                .font(.caption)
                                .padding(.horizontal, 10)
                                .padding(.vertical, 6)
                                .background(NeoTrixTheme.Colors.selection)
                                .clipShape(Capsule())
                        }
                        .buttonStyle(.plain)
                        .disabled(manager.isProcessing || text.isEmpty)
                    }
                }
            }
            
            if manager.isProcessing {
                HStack(spacing: 8) {
                    ProgressView()
                        .scaleEffect(0.7)
                    Text("AI editing…")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            
            if let error = manager.lastError {
                Text(error)
                    .font(.caption)
                    .foregroundColor(NeoTrixTheme.Colors.danger)
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 8)
        .background(.bar)
    }
    
    private func runAction(_ action: AIEditAction) async {
        let original = text
        let result = await manager.edit(original, action: action)
        if result != original {
            text = result
            onApply(result)
        }
    }
}

// MARK: - AI Editor Sheet (full editor)

public struct AIEditorView: View {
    @StateObject private var manager = AIEditorManager()
    @State private var draft: String
    @State private var selectedAction: AIEditAction = .rewrite
    @State private var result: String?
    
    let original: String
    let onUse: (String) -> Void
    
    @Environment(\.dismiss) private var dismiss
    
    public init(original: String, onUse: @escaping (String) -> Void) {
        self.original = original
        self.onUse = onUse
        self._draft = State(initialValue: original)
    }
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                // Original
                VStack(alignment: .leading, spacing: 8) {
                    Text("Original")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(original)
                        .font(.body)
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(NeoTrixTheme.Colors.inputBackground)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
                
                // Action picker
                Picker("Action", selection: $selectedAction) {
                    ForEach(AIEditAction.allCases) { action in
                        Text(action.rawValue).tag(action)
                    }
                }
                .pickerStyle(.menu)
                
                // Result
                if let result {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Result")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        Text(result)
                            .font(.body)
                            .padding()
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(NeoTrixTheme.Colors.selection)
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                        
                        // 融合修复: Use 按钮应用结果（此前 onUse 回调无 UI 接线 = 死回调）
                        Button {
                            onUse(result)
                            dismiss()
                        } label: {
                            Label("Use Result", systemImage: "checkmark.circle.fill")
                                .font(.subheadline.bold())
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, 10)
                                .background(NeoTrixTheme.Colors.accent)
                                .foregroundColor(.white)
                                .clipShape(RoundedRectangle(cornerRadius: 10))
                        }
                        .buttonStyle(.plain)
                    }
                }
                
                Spacer()
            }
            .padding()
            .navigationTitle("AI Editor")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Edit") {
                        Task {
                            result = await manager.edit(draft, action: selectedAction)
                        }
                    }
                    .disabled(manager.isProcessing)
                }
                #else
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Edit") {
                        Task {
                            result = await manager.edit(draft, action: selectedAction)
                        }
                    }
                    .disabled(manager.isProcessing)
                }
                #endif
            }
        }
    }
}