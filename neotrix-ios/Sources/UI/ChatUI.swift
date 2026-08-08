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
    /// 当前回复目标（对标 Telegram: 引用回复）
    @Published public var replyTarget: ChatMessage?
    
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
            replyTo: replyTarget?.id,
            media: []
        )
        
        messages.append(message)
        inputText = ""
        replyTarget = nil
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
    
    /// 设置回复目标（长按消息 → Reply）
    public func setReplyTarget(_ message: ChatMessage) {
        replyTarget = message
    }
    
    public func clearReplyTarget() {
        replyTarget = nil
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
    /// 被引用消息（replyTo 渲染，对标 Telegram 引用回复）
    let replyToMessage: ChatMessage?
    let onReaction: (String) -> Void
    let onLongPress: () -> Void
    let onReplyTap: (() -> Void)?
    
    public init(message: ChatMessage,
                replyToMessage: ChatMessage? = nil,
                onReaction: @escaping (String) -> Void,
                onLongPress: @escaping () -> Void,
                onReplyTap: (() -> Void)? = nil) {
        self.message = message
        self.replyToMessage = replyToMessage
        self.onReaction = onReaction
        self.onLongPress = onLongPress
        self.onReplyTap = onReplyTap
    }
    
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
                
                // 引用回复条（对标 Telegram: 被引用消息预览 + 点击跳转）
                if let replyToMessage {
                    Button(action: { onReplyTap?() }) {
                        HStack(spacing: 8) {
                            RoundedRectangle(cornerRadius: 2)
                                .fill(NeoTrixTheme.Colors.accent)
                                .frame(width: 3)
                            
                            VStack(alignment: .leading, spacing: 2) {
                                Text(senderName(replyToMessage.sender))
                                    .font(.caption.bold())
                                    .foregroundColor(NeoTrixTheme.Colors.accent)
                                Text(replyToMessage.text.isEmpty ? "📎 Attachment" : replyToMessage.text)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                                    .lineLimit(1)
                            }
                        }
                        .padding(8)
                        .background(NeoTrixTheme.Colors.surface)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                    }
                    .buttonStyle(.plain)
                }
                
                // Message content
                if !message.media.isEmpty {
                    VStack(alignment: message.sender == .user ? .trailing : .leading, spacing: 6) {
                        ForEach(Array(message.media.enumerated()), id: \.offset) { _, media in
                            MessageMediaView(media: media)
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 10)
                    .background(bubbleColor)
                    .foregroundColor(bubbleTextColor)
                    .clipShape(RoundedRectangle(cornerRadius: 18))
                }
                
                if !message.text.isEmpty {
                    Text(message.text)
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .background(bubbleColor)
                        .foregroundColor(bubbleTextColor)
                        .clipShape(RoundedRectangle(cornerRadius: 18))
                }
                
                // Reactions
                if !message.reactions.isEmpty {
                    HStack(spacing: 4) {
                        ForEach(message.reactions, id: \.emoji) { reaction in
                            Button(action: { onReaction(reaction.emoji) }) {
                                Text("\(reaction.emoji) \(reaction.count)")
                                    .font(.caption)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 2)
                                    .background(reaction.isSelected ? NeoTrixTheme.Colors.selection : Color.clear)
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
        case .user: return NeoTrixTheme.Colors.accent
        case .agent: return NeoTrixTheme.Colors.bubbleIncoming
        case .system: return NeoTrixTheme.Colors.bubbleSystem
        }
    }
    
    private var bubbleTextColor: Color {
        switch message.sender {
        case .user: return .white
        default: return .primary
        }
    }
    
    /// 引用条发送者名（对标 Telegram: "You" / agent 名 / "System"）
    private func senderName(_ sender: ChatMessage.Sender) -> String {
        switch sender {
        case .user: return "You"
        case .agent(let name): return name
        case .system: return "System"
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
                case .read: Image(systemName: "checkmark.circle.fill").foregroundColor(NeoTrixTheme.Colors.accent)
                case .failed: Image(systemName: "exclamationmark.circle.fill").foregroundColor(NeoTrixTheme.Colors.danger)
                }
            }
        }
        .font(.caption)
    }
}

// MARK: - 日期分隔条（对标 Telegram: Today/Yesterday/具体日期）

struct DateDivider: View {
    let date: Date
    
    var body: some View {
        Text(label)
            .font(.caption2)
            .foregroundColor(.secondary)
            .padding(.horizontal, 10)
            .padding(.vertical, 4)
            .background(NeoTrixTheme.Colors.surface)
            .clipShape(Capsule())
            .frame(maxWidth: .infinity)
            .padding(.vertical, 4)
    }
    
    private var label: String {
        let calendar = Calendar.current
        if calendar.isDateInToday(date) { return "Today" }
        if calendar.isDateInYesterday(date) { return "Yesterday" }
        let formatter = DateFormatter()
        formatter.dateFormat = "MMMM d"
        return formatter.string(from: date)
    }
}

// MARK: - 消息媒体渲染（对标 Telegram: 图片/文档/贴纸/位置/联系人）

struct MessageMediaView: View {
    let media: ChatMessage.MessageMedia
    
    var body: some View {
        switch media {
        case .image(let data):
            #if os(iOS)
            if let uiImage = UIImage(data: data) {
                Image(uiImage: uiImage)
                    .resizable()
                    .scaledToFit()
                    .frame(maxWidth: 220)
                    .clipShape(RoundedRectangle(cornerRadius: NeoTrixTheme.Radius.small))
            }
            #else
            Label("Image", systemImage: "photo.fill")
                .font(.subheadline)
                .padding(.vertical, 4)
            #endif
        case .video(let url):
            Label("Video", systemImage: "play.rectangle.fill")
                .font(.subheadline)
                .padding(.vertical, 4)
        case .audio(let url):
            Label("Audio", systemImage: "waveform")
                .font(.subheadline)
                .padding(.vertical, 4)
        case .document(let url, let name):
            HStack(spacing: 8) {
                Image(systemName: "doc.fill")
                    .foregroundColor(NeoTrixTheme.Colors.accent)
                Text(name)
                    .font(.subheadline)
                    .lineLimit(1)
            }
            .padding(.vertical, 4)
        case .sticker(let emoji):
            Text(emoji)
                .font(.system(size: 64))
                .padding(.vertical, 4)
        case .location(let lat, let lon):
            Label(String(format: "📍 %.4f, %.4f", lat, lon), systemImage: "mappin.and.ellipse")
                .font(.subheadline)
                .padding(.vertical, 4)
        case .contact(let name, let phone):
            HStack(spacing: 8) {
                Image(systemName: "person.crop.circle.fill")
                    .foregroundColor(NeoTrixTheme.Colors.accent)
                VStack(alignment: .leading, spacing: 2) {
                    Text(name).font(.subheadline)
                    Text(phone).font(.caption).foregroundColor(.secondary)
                }
            }
            .padding(.vertical, 4)
        }
    }
}

// MARK: - Chat View

public struct ChatView: View {
    @StateObject private var viewModel = ChatViewModel()
    @State private var showAIEditor = false
    @State private var showExport = false
    @State private var reactionTarget: UUID?
    @State private var showReactionPicker = false
    @State private var showEmojiStatus = false
    
    public init() {}
    
    public var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 0) {
                        // 日期分隔（对标 Telegram: Today/Yesterday/日期）
                        ForEach(Array(viewModel.messages.enumerated()), id: \.element.id) { index, message in
                            if index == 0 || !Calendar.current.isDate(viewModel.messages[index - 1].timestamp, inSameDayAs: message.timestamp) {
                                DateDivider(date: message.timestamp)
                            }
                            
                            // 消息连续分组（对标 Telegram: 同发送者连续消息间距 4，新组间距 12）
                            let isGrouped = index > 0
                                && viewModel.messages[index - 1].sender == message.sender
                                && Calendar.current.isDate(viewModel.messages[index - 1].timestamp, inSameDayAs: message.timestamp)
                            
                            MessageBubbleView(
                                message: message,
                                replyToMessage: message.replyTo.flatMap { replyID in
                                    viewModel.messages.first { $0.id == replyID }
                                },
                                onReaction: { emoji in viewModel.addReaction(emoji, to: message.id) },
                                onLongPress: {
                                    // 融合: 长按消息 → ReactionPicker（官方 Reactions + Premium 门控）
                                    reactionTarget = message.id
                                    showReactionPicker = true
                                },
                                onReplyTap: {
                                    // 点击引用条 → 滚动到被引用消息（对标 Telegram）
                                    if let replyID = message.replyTo {
                                        withAnimation { proxy.scrollTo(replyID, anchor: .center) }
                                    }
                                }
                            )
                            .id(message.id)
                            .padding(.top, isGrouped ? 4 : 12)
                            .contextMenu {
                                // 对标 Telegram: 长按菜单 → Reply / Copy / Reactions
                                Button {
                                    viewModel.setReplyTarget(message)
                                } label: {
                                    Label("Reply", systemImage: "arrowshape.turn.up.left")
                                }
                                Button {
                                    #if os(iOS)
                                    UIPasteboard.general.string = message.text
                                    #endif
                                } label: {
                                    Label("Copy", systemImage: "doc.on.doc")
                                }
                                Button {
                                    reactionTarget = message.id
                                    showReactionPicker = true
                                } label: {
                                    Label("Add Reaction", systemImage: "face.smiling")
                                }
                            }
                        }
                        
                        if viewModel.isStreaming {
                            HStack {
                                TypingIndicator()
                                Spacer()
                            }
                            .padding(.leading, 16)
                            .padding(.top, 8)
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
            
            // AI Editor 快捷面板（融合: 官方 AI Editor + Swiftgram Quick Formatting）
            if !viewModel.inputText.isEmpty {
                AIEditorPanel(text: $viewModel.inputText) { edited in
                    viewModel.inputText = edited
                }
            }
            
            // 回复目标条（对标 Telegram: "Replying to X" + 取消）
            if let replyTarget = viewModel.replyTarget {
                HStack(spacing: 8) {
                    Image(systemName: "arrowshape.turn.up.left.fill")
                        .font(.caption)
                        .foregroundColor(NeoTrixTheme.Colors.accent)
                    VStack(alignment: .leading, spacing: 1) {
                        Text("Replying to \(replySenderName(replyTarget))")
                            .font(.caption.bold())
                            .foregroundColor(NeoTrixTheme.Colors.accent)
                        Text(replyTarget.text.isEmpty ? "📎 Attachment" : replyTarget.text)
                            .font(.caption2)
                            .foregroundColor(.secondary)
                            .lineLimit(1)
                    }
                    Spacer()
                    Button {
                        viewModel.clearReplyTarget()
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                    }
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 6)
                .background(NeoTrixTheme.Colors.surface)
                .transition(.move(edge: .bottom).combined(with: .opacity))
            }
            
            // Input bar
            HStack(spacing: 8) {
                // 附件菜单（对标 Telegram: 照片/文件/位置/联系人）
                Menu {
                    Button {
                        viewModel.messages.append(ChatMessage(
                            id: UUID(), text: "", sender: .user, timestamp: Date(),
                            status: .sent, reactions: [], replyTo: nil,
                            media: [.sticker("😀")]
                        ))
                    } label: {
                        Label("Sticker", systemImage: "face.smiling")
                    }
                    Button {
                        viewModel.messages.append(ChatMessage(
                            id: UUID(), text: "", sender: .user, timestamp: Date(),
                            status: .sent, reactions: [], replyTo: nil,
                            media: [.location(37.7749, -122.4194)]
                        ))
                    } label: {
                        Label("Location", systemImage: "mappin.and.ellipse")
                    }
                    Button {
                        viewModel.messages.append(ChatMessage(
                            id: UUID(), text: "", sender: .user, timestamp: Date(),
                            status: .sent, reactions: [], replyTo: nil,
                            media: [.contact("Alice", "+1 (555) 0101")]
                        ))
                    } label: {
                        Label("Contact", systemImage: "person.crop.circle")
                    }
                    Button {
                        viewModel.messages.append(ChatMessage(
                            id: UUID(), text: "", sender: .user, timestamp: Date(),
                            status: .sent, reactions: [], replyTo: nil,
                            media: [.document(URL(string: "file:///tmp/report.pdf")!, "report.pdf")]
                        ))
                    } label: {
                        Label("Document", systemImage: "doc.fill")
                    }
                } label: {
                    Image(systemName: "plus.circle.fill")
                        .font(.title2)
                }
                .contextMenu {
                    // 融合: 动画 Emoji 状态（AnimatedEmoji 接线）
                    Button {
                        showEmojiStatus = true
                    } label: {
                        Label("Emoji Status", systemImage: "sparkles")
                    }
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
                            .foregroundColor(NeoTrixTheme.Colors.accentSecondary)
                    }
                }
                
                Button(action: sendMessage) {
                    Image(systemName: viewModel.isStreaming ? "stop.circle" : "arrow.up.circle.fill")
                        .font(.title2)
                }
                .disabled(viewModel.inputText.trimmingCharacters(in: .whitespaces).isEmpty && !viewModel.isStreaming)
            }
            .padding()
            .background(NeoTrixTheme.Colors.surface)
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
        .sheet(isPresented: $showReactionPicker) {
            // 融合: Reactions 工具栏（长按消息 → 选择 emoji → 应用到目标消息）
            ReactionPickerView { emoji in
                if let target = reactionTarget {
                    viewModel.addReaction(emoji, to: target)
                }
            }
        }
        .sheet(isPresented: $showEmojiStatus) {
            // 融合: 动画 Emoji 状态选择器
            EmojiStatusPickerView { emoji in
                // 选择后作为消息发送（动画 emoji 状态）
                viewModel.inputText = emoji
            }
        }
        .toolbar {
            #if os(iOS)
            ToolbarItem(placement: .navigationBarTrailing) {
                Menu {
                    Button("Summarize", action: { Task { await viewModel.summarizeChat() } })
                    Button("Export to AI", action: { showExport = true })
                    Button("Clear", role: .destructive, action: { viewModel.messages.removeAll() })
                } label: {
                    Image(systemName: "ellipsis.circle")
                }
            }
            #else
            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Button("Summarize", action: { Task { await viewModel.summarizeChat() } })
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
    
    /// 回复目标条发送者名
    private func replySenderName(_ message: ChatMessage) -> String {
        switch message.sender {
        case .user: return "You"
        case .agent(let name): return name
        case .system: return "System"
        }
    }
}

// MARK: - Typing Indicator

struct TypingIndicator: View {
    @State private var animating = false
    
    var body: some View {
        HStack(spacing: 6) {
            HStack(spacing: 4) {
                ForEach(0..<3) { i in
                    Circle()
                        .fill(NeoTrixTheme.Colors.textSecondary)
                        .frame(width: 8, height: 8)
                        .scaleEffect(animating ? 1.0 : 0.5)
                        .animation(.easeInOut(duration: 0.5).repeatForever().delay(Double(i) * 0.2), value: animating)
                }
            }
            Text("typing...")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(NeoTrixTheme.Colors.bubbleIncoming)
        .clipShape(Capsule())
        .onAppear { animating = true }
    }
}