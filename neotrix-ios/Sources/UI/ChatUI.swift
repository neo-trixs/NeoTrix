// ChatUI - Telegram-style chat interface
// Mirrors Telegram's ChatController + ChatControllerNode + Message rendering

import SwiftUI
import Combine
import NeoTrixFFI

// MARK: - Message Model

public struct ChatMessage: Identifiable, Equatable {
    public let id: UUID
    public let text: String
    public let sender: Sender
    public let timestamp: Date
    public var status: MessageStatus
    public var reactions: [Reaction]
    public var replyTo: UUID?
    public var media: [MessageMedia]
    
    public enum Sender: Equatable {
        case user
        case agent(String) // agent name
        case system
    }
    
    public enum MessageStatus: Equatable {
        case sending
        case sent
        case delivered
        case read
        case failed(Error)
        
        public static func == (lhs: MessageStatus, rhs: MessageStatus) -> Bool {
            switch (lhs, rhs) {
            case (.sending, .sending), (.sent, .sent), (.delivered, .delivered), (.read, .read):
                return true
            case (.failed(let lhsError), .failed(let rhsError)):
                return lhsError.localizedDescription == rhsError.localizedDescription
            default:
                return false
            }
        }
    }
    
    public struct Reaction: Equatable {
        public let emoji: String
        public var count: Int
        public var isSelected: Bool
    }
    
    public enum MessageMedia: Equatable {
        case image(Data)
        case video(URL)
        case audio(URL)
        case document(URL, String)
        case sticker(String)
        case location(Double, Double)
        case contact(String, String)
    }
}

// MARK: - Chat View Model

@MainActor
public final class ChatViewModel: ObservableObject {
    @Published public var messages: [ChatMessage] = []
    @Published public var isStreaming = false
    @Published public var inputText = ""
    @Published public var showAttachmentMenu = false
    @Published public var selectedStickerPack: String?
    
    private let core = NeoGramCore.shared
    private var cancellables = Set<AnyCancellable>()
    
    public init() {
        setupAIIntegration()
    }
    
    private func setupAIIntegration() {
        // Connect to NeoTrix AI for smart replies, summarization
        core.gwtAttention?.registerModule(name: "ChatUI", interestKeywords: ["message", "reply", "summary"])
    }
    
    public func sendMessage(_ text: String) async {
        let message = ChatMessage(
            id: UUID(),
            text: text,
            sender: .user,
            timestamp: Date(),
            status: .sending,
            reactions: [],
            replyTo: nil,
            media: []
        )
        
        messages.append(message)
        inputText = ""
        isStreaming = true
        
        // Send via MTProto
        do {
            try await sendViaMTProto(message)
            await updateMessageStatus(message.id, status: .sent)
        } catch {
            await updateMessageStatus(message.id, status: .failed(error))
        }
        
        // Get AI response
        await generateAIResponse(for: text)
    }
    
    private func sendViaMTProto(_ message: ChatMessage) async throws {
        // Send via MTProtoManager
    }
    
    private func updateMessageStatus(_ id: UUID, status: ChatMessage.MessageStatus) async {
        if let index = messages.firstIndex(where: { $0.id == id }) {
            messages[index].status = status
        }
    }
    
    private func generateAIResponse(for text: String) async {
        guard let e8 = core.e8Reasoning else {
            await addAgentMessage("AI not available")
            return
        }
        
        do {
            let request = ReasoningRequest(
                query: text,
                context: "Chat conversation",
                maxDepth: 3,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            
            await addAgentMessage(response.conclusion)
        } catch {
            await addAgentMessage("Error: \(error.localizedDescription)")
        }
    }
    
    private func addAgentMessage(_ text: String) async {
        let message = ChatMessage(
            id: UUID(),
            text: text,
            sender: .agent("NeoTrix"),
            timestamp: Date(),
            status: .delivered,
            reactions: [],
            replyTo: nil,
            media: []
        )
        messages.append(message)
        isStreaming = false
    }
    
    public func addReaction(_ emoji: String, to messageId: UUID) {
        if let index = messages.firstIndex(where: { $0.id == messageId }) {
            if let reactionIndex = messages[index].reactions.firstIndex(where: { $0.emoji == emoji }) {
                messages[index].reactions[reactionIndex].count += 1
                messages[index].reactions[reactionIndex].isSelected = true
            } else {
                messages[index].reactions.append(ChatMessage.Reaction(emoji: emoji, count: 1, isSelected: true))
            }
        }
    }
    
    /// AI Summarize（融合: 官方 Summarize + Export to LLM → NeoTrix KB）
    public func summarizeChat() async {
        guard !messages.isEmpty else { return }
        guard let e8 = core.e8Reasoning else {
            await addAgentMessage("AI not available")
            return
        }
        
        let transcript = messages
            .prefix(50)
            .map { message -> String in
                let label: String
                switch message.sender {
                case .user: label = "User"
                case .agent(let name): label = name
                case .system: label = "System"
                }
                return "\(label): \(message.text)"
            }
            .joined(separator: "\n")
        
        do {
            let request = ReasoningRequest(
                query: "Summarize this conversation in 3 bullet points:\n\(transcript)",
                context: "Chat summarization",
                maxDepth: 2,
                useConsciousness: true
            )
            let response = try e8.reason(request: request)
            await addAgentMessage("📋 对话摘要\n\(response.conclusion)")
        } catch {
            await addAgentMessage("Error: \(error.localizedDescription)")
        }
    }
    
    public func toggleStickerPicker() {
        showAttachmentMenu.toggle()
    }
}

// MARK: - Message Bubble View

public struct MessageBubbleView: View {
    let message: ChatMessage
    let onReaction: (String) -> Void
    let onLongPress: () -> Void
    
    public var body: some View {
        HStack(alignment: .bottom, spacing: 8) {
            if message.sender == .user {
                Spacer(minLength: 50)
            }
            
            VStack(alignment: message.sender == .user ? .trailing : .leading, spacing: 4) {
                // Sender name for agent/system
                if case .agent(let name) = message.sender {
                    Text(name)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .padding(.horizontal, 12)
                }
                
                // Message content
                Text(message.text)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(bubbleColor)
                    .foregroundColor(bubbleTextColor)
                    .clipShape(RoundedRectangle(cornerRadius: 18))
                
                // Reactions
                if !message.reactions.isEmpty {
                    HStack(spacing: 4) {
                        ForEach(message.reactions, id: \.emoji) { reaction in
                            Button(action: { onReaction(reaction.emoji) }) {
                                Text("\(reaction.emoji) \(reaction.count)")
                                    .font(.caption)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 2)
                                    .background(reaction.isSelected ? Color.blue.opacity(0.2) : Color.clear)
                                    .clipShape(Capsule())
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(.horizontal, 4)
                }
                
                // Status
                HStack(spacing: 4) {
                    Text(message.timestamp, style: .time)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                    
                    if message.sender == .user {
                        statusIcon
                    }
                }
                .padding(.horizontal, 4)
            }
            
            if message.sender != .user {
                Spacer(minLength: 50)
            }
        }
        .onLongPressGesture(perform: onLongPress)
    }
    
    private var bubbleColor: Color {
        switch message.sender {
        case .user: return .blue
        case .agent: return Color.gray.opacity(0.25)
        case .system: return Color.gray.opacity(0.15)
        }
    }
    
    private var bubbleTextColor: Color {
        switch message.sender {
        case .user: return .white
        default: return .primary
        }
    }
    
    private var statusIcon: some View {
        Group {
            // Ghost Mode: hide read receipts (Nicegram fusion)
            if PrivacyEngine.shared.settings.ghostMode && message.status == .read {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(.secondary)
            } else {
                switch message.status {
                case .sending: ProgressView().scaleEffect(0.6)
                case .sent: Image(systemName: "checkmark").foregroundColor(.secondary)
                case .delivered: Image(systemName: "checkmark.circle.fill").foregroundColor(.secondary)
                case .read: Image(systemName: "checkmark.circle.fill").foregroundColor(.blue)
                case .failed: Image(systemName: "exclamationmark.circle.fill").foregroundColor(.red)
                }
            }
        }
        .font(.caption)
    }
}

// MARK: - Chat View

public struct ChatView: View {
    @StateObject private var viewModel = ChatViewModel()
    @State private var scrollProxy: ScrollViewProxy?
    @State private var showAIEditor = false
    @State private var showExport = false
    
    public init() {}
    
    public var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 12) {
                        ForEach(viewModel.messages) { message in
                            MessageBubbleView(
                                message: message,
                                onReaction: { emoji in viewModel.addReaction(emoji, to: message.id) },
                                onLongPress: { /* show context menu */ }
                            )
                            .id(message.id)
                        }
                        
                        if viewModel.isStreaming {
                            HStack {
                                TypingIndicator()
                                Spacer()
                            }
                            .padding(.leading, 16)
                        }
                    }
                    .padding()
                }
                .onChange(of: viewModel.messages.count) { _, _ in
                    if let last = viewModel.messages.last {
                        withAnimation { proxy.scrollTo(last.id, anchor: .bottom) }
                    }
                }
            }
            
            // Input bar
            HStack(spacing: 8) {
                Button(action: { viewModel.toggleStickerPicker() }) {
                    Image(systemName: "face.smiling")
                        .font(.title2)
                }
                
                TextField("Message", text: $viewModel.inputText, axis: .vertical)
                    .textFieldStyle(.roundedBorder)
                    .lineLimit(1...5)
                    .disabled(viewModel.isStreaming)
                
                // AI Editor button (Fusion: official AI Editor + Swiftgram formatting)
                if !viewModel.inputText.isEmpty {
                    Button {
                        showAIEditor = true
                    } label: {
                        Image(systemName: "wand.and.stars")
                            .font(.title2)
                            .foregroundColor(.purple)
                    }
                }
                
                Button(action: sendMessage) {
                    Image(systemName: viewModel.isStreaming ? "stop.circle" : "arrow.up.circle.fill")
                        .font(.title2)
                }
                .disabled(viewModel.inputText.trimmingCharacters(in: .whitespaces).isEmpty && !viewModel.isStreaming)
            }
            .padding()
            .background(.bar)
        }
        .navigationTitle("Chat")
        .sheet(isPresented: $showAIEditor) {
            AIEditorView(original: viewModel.inputText) { edited in
                viewModel.inputText = edited
            }
        }
        .sheet(isPresented: $showExport) {
            ExportView(messages: viewModel.messages, chatTitle: "Chat")
        }
        .toolbar {
            #if os(iOS)
            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button("Summarize", action: { /* summarize chat */ })
                    Button("Export to AI", action: { showExport = true })
                    Button("Clear", role: .destructive, action: { viewModel.messages.removeAll() })
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
            #else
            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Button("Summarize", action: { /* summarize chat */ })
                    Button("Export to AI", action: { showExport = true })
                    Button("Clear", role: .destructive, action: { viewModel.messages.removeAll() })
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
            #endif
        }
    }
    
    private func sendMessage() {
        let text = viewModel.inputText
        viewModel.inputText = ""
        Task { await viewModel.sendMessage(text) }
    }
}

// MARK: - Typing Indicator

struct TypingIndicator: View {
    @State private var animating = false
    
    var body: some View {
        HStack(spacing: 4) {
            ForEach(0..<3) { i in
                Circle()
                    .fill(Color.gray)
                    .frame(width: 8, height: 8)
                    .scaleEffect(animating ? 1.0 : 0.5)
                    .animation(.easeInOut(duration: 0.5).repeatForever().delay(Double(i) * 0.2), value: animating)
            }
        }
        .onAppear { animating = true }
    }
}