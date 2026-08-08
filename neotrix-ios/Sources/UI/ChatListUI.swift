// ChatListUI - Telegram-style chat list with folders
// Mirrors Telegram's ChatListController + ChatListFilters

import SwiftUI
import Combine

// MARK: - Chat List Models

public struct ChatListItem: Identifiable, Hashable {
    public let id: Int64
    public let title: String
    public let lastMessage: String
    public let timestamp: Date
    public var unreadCount: Int
    public var isPinned: Bool
    public var isMuted: Bool
    public let isOnline: Bool
    public let avatarColor: Color
    public let isPremium: Bool
    public let isVerified: Bool
}

/// 联系人（融合原 ContactsView 占位 → 真实模型）
public struct ContactItem: Identifiable {
    public let id: Int64
    public let name: String
    public let phone: String
    public let avatarColor: Color
    public let isOnline: Bool
    public let isPremium: Bool
}

/// 通话记录（融合原 CallsView 占位 → 真实模型）
public struct CallRecord: Identifiable {
    public enum Direction { case incoming, outgoing, missed }
    public let id: Int64
    public let name: String
    public let direction: Direction
    public let timestamp: Date
    public let duration: Int  // seconds
}

public enum ChatListFilter: String, CaseIterable {
    case all = "All"
    case unread = "Unread"
    case personal = "Personal"
    case groups = "Groups"
    case channels = "Channels"
    case bots = "Bots"
    case favorites = "Favorites"
}

// MARK: - Chat List View Model

@MainActor
public final class ChatListViewModel: ObservableObject {
    @Published public var chats: [ChatListItem] = []
    @Published public var contacts: [ContactItem] = []
    @Published public var calls: [CallRecord] = []
    @Published public var selectedFilter: ChatListFilter = .all
    @Published public var searchText = ""
    @Published public var isSearching = false
    
    private let core = NeoGramCore.shared
    private var cancellables = Set<AnyCancellable>()
    
    public init() {
        loadChats()
        loadContacts()
        loadCalls()
        setupSearch()
    }
    
    private func loadChats() {
        // Load from MTProto dialogs
        chats = [
            ChatListItem(id: 1, title: "NeoTrix AI", lastMessage: "Ready to help!", timestamp: Date(), unreadCount: 2, isPinned: true, isMuted: false, isOnline: true, avatarColor: .purple, isPremium: true, isVerified: true),
            ChatListItem(id: 2, title: "Family Group", lastMessage: "Mom: Dinner at 7?", timestamp: Date().addingTimeInterval(-300), unreadCount: 0, isPinned: true, isMuted: false, isOnline: false, avatarColor: .green, isPremium: false, isVerified: false),
            ChatListItem(id: 3, title: "Work", lastMessage: "John: Meeting moved to 3pm", timestamp: Date().addingTimeInterval(-1800), unreadCount: 5, isPinned: false, isMuted: true, isOnline: true, avatarColor: .blue, isPremium: false, isVerified: false),
            ChatListItem(id: 4, title: "Tech News", lastMessage: "New AI model released", timestamp: Date().addingTimeInterval(-3600), unreadCount: 0, isPinned: false, isMuted: false, isOnline: false, avatarColor: .orange, isPremium: true, isVerified: true),
            ChatListItem(id: 5, title: "Design Team", lastMessage: "Alice: Updated the mockups", timestamp: Date().addingTimeInterval(-7200), unreadCount: 1, isPinned: false, isMuted: false, isOnline: false, avatarColor: .pink, isPremium: false, isVerified: false),
        ]
    }
    
    private func loadContacts() {
        // 真实联系人模型（融合原 ContactsView 占位）
        contacts = [
            ContactItem(id: 101, name: "Alice", phone: "+1 (555) 0101", avatarColor: .pink, isOnline: true, isPremium: false),
            ContactItem(id: 102, name: "Bob", phone: "+1 (555) 0102", avatarColor: .blue, isOnline: false, isPremium: false),
            ContactItem(id: 103, name: "Charlie", phone: "+1 (555) 0103", avatarColor: .orange, isOnline: true, isPremium: true),
            ContactItem(id: 104, name: "Diana", phone: "+1 (555) 0104", avatarColor: .green, isOnline: false, isPremium: false),
        ]
    }
    
    private func loadCalls() {
        // 真实通话记录（融合原 CallsView 占位）
        let now = Date()
        calls = [
            CallRecord(id: 201, name: "Alice", direction: .incoming, timestamp: now.addingTimeInterval(-600), duration: 186),
            CallRecord(id: 202, name: "Work", direction: .missed, timestamp: now.addingTimeInterval(-3600), duration: 0),
            CallRecord(id: 203, name: "Diana", direction: .outgoing, timestamp: now.addingTimeInterval(-7200), duration: 92),
            CallRecord(id: 204, name: "Family Group", direction: .incoming, timestamp: now.addingTimeInterval(-10800), duration: 540),
        ]
    }
    
    private func setupSearch() {
        $searchText
            .debounce(for: .milliseconds(300), scheduler: RunLoop.main)
            .sink { [weak self] text in
                self?.isSearching = !text.isEmpty
            }
            .store(in: &cancellables)
    }
    
    public var filteredChats: [ChatListItem] {
        var result = chats
        
        // Filter by search
        if !searchText.isEmpty {
            result = result.filter { $0.title.localizedCaseInsensitiveContains(searchText) }
        }
        
        // Filter by category
        switch selectedFilter {
        case .all:
            break
        case .unread:
            result = result.filter { $0.unreadCount > 0 }
        case .personal:
            result = result.filter { !$0.title.contains("Group") && !$0.title.contains("News") }
        case .groups:
            result = result.filter { $0.title.contains("Group") || $0.title.contains("Team") }
        case .channels:
            result = result.filter { $0.title.contains("News") }
        case .bots:
            result = result.filter { $0.isPremium }
        case .favorites:
            result = result.filter { $0.isPinned }
        }
        
        // Sort: pinned first, then by timestamp
        return result.sorted { lhs, rhs in
            if lhs.isPinned != rhs.isPinned { return lhs.isPinned }
            return lhs.timestamp > rhs.timestamp
        }
    }
    
    public func markAsRead(_ chatId: Int64) {
        if let index = chats.firstIndex(where: { $0.id == chatId }) {
            chats[index].unreadCount = 0
        }
    }
    
    public func toggleMute(_ chatId: Int64) {
        if let index = chats.firstIndex(where: { $0.id == chatId }) {
            chats[index].isMuted.toggle()
        }
    }
    
    public func togglePin(_ chatId: Int64) {
        if let index = chats.firstIndex(where: { $0.id == chatId }) {
            chats[index].isPinned.toggle()
        }
    }
}

// MARK: - Chat List View

public struct ChatListView: View {
    @StateObject private var viewModel = ChatListViewModel()
    @StateObject private var storyViewModel = StoryViewModel()
    @State private var selectedChat: ChatListItem?
    @State private var showSettings = false
    @State private var showPremium = false
    @State private var showAI = false
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Stories 环（融合: 顶部故事 + AI 助手入口）
                StoriesBar(stories: storyViewModel.stories,
                           onAI: { showAI = true },
                           onPublish: { caption, isCloseFriends in
                               storyViewModel.publishStory(caption: caption, isCloseFriends: isCloseFriends)
                           },
                           onSeen: { id in storyViewModel.markAsSeen(id) })
                
                // Filter bar
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(ChatListFilter.allCases, id: \.self) { filter in
                            FilterChip(
                                title: filter.rawValue,
                                isSelected: viewModel.selectedFilter == filter
                            ) {
                                withAnimation { viewModel.selectedFilter = filter }
                            }
                        }
                    }
                    .padding(.horizontal)
                    .padding(.vertical, 8)
                }
                
                // Chat list
                List {
                    // 融合：联系人 Section（真实模型，替代 Contact N 占位）
                    if viewModel.searchText.isEmpty {
                        Section {
                            ForEach(viewModel.contacts) { contact in
                                HStack(spacing: 12) {
                                    ZStack(alignment: .bottomTrailing) {
                                        Circle()
                                            .fill(contact.avatarColor.gradient)
                                            .frame(width: 40, height: 40)
                                            .overlay(
                                                Text(String(contact.name.prefix(1)).uppercased())
                                                    .font(.subheadline.bold())
                                                    .foregroundColor(.white)
                                            )
                                        
                                        if contact.isOnline {
                                            Circle()
                                                .fill(NeoTrixTheme.Colors.online)
                                                .frame(width: 10, height: 10)
                                                .overlay(Circle().stroke(NeoTrixTheme.Colors.background, lineWidth: 1.5))
                                        }
                                    }
                                    
                                    VStack(alignment: .leading, spacing: 2) {
                                        HStack(spacing: 4) {
                                            Text(contact.name)
                                                .font(.body)
                                            if contact.isPremium {
                                                Image(systemName: "star.fill")
                                                    .font(.caption2)
                                                    .foregroundColor(NeoTrixTheme.Colors.premium)
                                            }
                                        }
                                        Text(contact.phone)
                                            .font(.caption)
                                            .foregroundColor(.secondary)
                                    }
                                }
                                .padding(.vertical, 2)
                            }
                        } header: {
                            Text("Contacts")
                        }
                        
                        // 融合：通话记录（真实模型，替代 Call N 占位）
                        Section {
                            ForEach(viewModel.calls) { call in
                                HStack(spacing: 12) {
                                    Image(systemName: iconForCall(call.direction))
                                        .font(.body)
                                        .foregroundColor(colorForCall(call.direction))
                                        .frame(width: 24)
                                    
                                    Text(call.name)
                                    
                                    Spacer()
                                    
                                    Text(callDurationText(call))
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                                .padding(.vertical, 2)
                            }
                        } header: {
                            Text("Recent Calls")
                        }
                    }
                    
                    ForEach(viewModel.filteredChats) { chat in
                        ChatListRow(chat: chat)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                selectedChat = chat
                            }
                            .contextMenu {
                                Button {
                                    viewModel.markAsRead(chat.id)
                                } label: {
                                    Label("Mark as Read", systemImage: "checkmark.circle")
                                }
                                
                                Button {
                                    viewModel.toggleMute(chat.id)
                                } label: {
                                    Label(chat.isMuted ? "Unmute" : "Mute", systemImage: chat.isMuted ? "speaker.wave.2" : "bell.slash")
                                }
                                
                                Button {
                                    viewModel.togglePin(chat.id)
                                } label: {
                                    Label(chat.isPinned ? "Unpin" : "Pin", systemImage: chat.isPinned ? "pin.slash" : "pin")
                                }
                                
                                Divider()
                                
                                Button(role: .destructive) {
                                    withAnimation {
                                        viewModel.chats.removeAll { $0.id == chat.id }
                                    }
                                } label: {
                                    Label("Delete", systemImage: "trash")
                                }
                            }
                    }
                }
                .listStyle(.plain)
                .searchable(text: $viewModel.searchText, prompt: "Search")
            }
            .navigationTitle("Chats")
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarLeading) {
                    Button {
                        showSettings = true
                    } label: {
                        Image(systemName: "gearshape.fill")
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    HStack(spacing: 16) {
                        Button {
                            showPremium = true
                        } label: {
                            Image(systemName: "star.circle.fill")
                                .foregroundColor(.yellow)
                        }
                        
                        Button {
                            // New chat
                        } label: {
                            Image(systemName: "square.and.pencil")
                        }
                    }
                }
                #else
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        showPremium = true
                    } label: {
                        Image(systemName: "star.circle.fill")
                            .foregroundColor(.yellow)
                    }
                }
                #endif
            }
            .sheet(isPresented: $showSettings) {
                SettingsView()
            }
            .sheet(isPresented: $showPremium) {
                PremiumIntroView()
            }
            .sheet(isPresented: $showAI) {
                // AI 助手入口（融合: AIHub 中枢）
                NavigationStack {
                    ChatView()
                        .navigationTitle("AI 助手")
                }
            }
            .navigationDestination(item: $selectedChat) { chat in
                ChatView()
                    .navigationTitle(chat.title)
            }
        }
    }
}

// MARK: - 通话方向辅助（真实通话记录渲染）

private func iconForCall(_ direction: CallRecord.Direction) -> String {
    switch direction {
    case .incoming: return "phone.arrow.down.left.fill"
    case .outgoing: return "phone.arrow.up.right.fill"
    case .missed: return "phone.arrow.up.right.fill"
    }
}

private func colorForCall(_ direction: CallRecord.Direction) -> Color {
    switch direction {
    case .incoming: return NeoTrixTheme.Colors.success
    case .outgoing: return NeoTrixTheme.Colors.accent
    case .missed: return NeoTrixTheme.Colors.danger
    }
}

private func callDurationText(_ call: CallRecord) -> String {
    if case .missed = call.direction { return "Missed" }
    if call.duration >= 60 {
        return "\(call.duration / 60)m \(call.duration % 60)s"
    }
    return "\(call.duration)s"
}

struct FilterChip: View {
    let title: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.subheadline)
                .neoTrixCapsule(isSelected: isSelected)
        }
        .buttonStyle(.plain)
    }
}

struct ChatListRow: View {
    let chat: ChatListItem
    
    var body: some View {
        HStack(spacing: 12) {
            // Avatar（设计系统: 头像 + 在线点）
            ZStack(alignment: .bottomTrailing) {
                NeoTrixAvatar(title: chat.title, size: 52,
                              gradient: LinearGradient(colors: [chat.avatarColor, chat.avatarColor.opacity(0.7)],
                                                       startPoint: .topLeading, endPoint: .bottomTrailing))
                
                if chat.isOnline {
                    NeoTrixTheme.Colors.online
                        .frame(width: 14, height: 14)
                        .clipShape(Circle())
                        .overlay(Circle().stroke(NeoTrixTheme.Colors.textPrimary.opacity(0.2), lineWidth: 2))
                }
            }
            
            // Content
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(chat.title)
                        .font(.headline)
                        .lineLimit(1)
                    
                    if chat.isVerified {
                        Image(systemName: "checkmark.seal.fill")
                            .font(.caption)
                            .foregroundColor(NeoTrixTheme.Colors.verified)
                    }
                    
                    if chat.isPremium {
                        Image(systemName: "star.fill")
                            .font(.caption2)
                            .foregroundColor(NeoTrixTheme.Colors.premium)
                    }
                    
                    Spacer()
                    
                    Text(chat.timestamp, style: .time)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                HStack {
                    Text(chat.lastMessage)
                        .font(.subheadline)
                        .foregroundColor(chat.unreadCount > 0 ? .primary : .secondary)
                        .lineLimit(1)
                    
                    Spacer()
                    
                    if chat.isMuted {
                        Image(systemName: "bell.slash.fill")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    
                    if chat.unreadCount > 0 {
                        Text("\(chat.unreadCount)")
                            .font(.caption.bold())
                            .foregroundColor(.white)
                            .frame(minWidth: 20, minHeight: 20)
                            .background(NeoTrixTheme.Colors.badge)
                            .clipShape(Circle())
                    }
                }
            }
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Stories Bar（融合: 顶部故事环 + AI 助手入口）

/// Chats Tab 顶部横向 Stories 环，末尾固定 AI 助手入口
public struct StoriesBar: View {
    let stories: [StoryItem]
    let onAI: () -> Void
    let onPublish: (String, Bool) -> Void
    let onSeen: (Int64) -> Void
    @State private var showViewer = false
    @State private var showComposer = false
    
    public init(stories: [StoryItem],
                onAI: @escaping () -> Void,
                onPublish: @escaping (String, Bool) -> Void = { _, _ in },
                onSeen: @escaping (Int64) -> Void = { _ in }) {
        self.stories = stories
        self.onAI = onAI
        self.onPublish = onPublish
        self.onSeen = onSeen
    }
    
    public var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 16) {
                // AI 助手入口（融合: AIHub 中枢）
                Button(action: onAI) {
                    VStack(spacing: 6) {
                        ZStack {
                            Circle()
                                .fill(NeoTrixTheme.Gradients.brand)
                                .frame(width: 64, height: 64)
                            
                            Image(systemName: "wand.and.stars")
                                .font(.title2)
                                .foregroundColor(.white)
                        }
                        
                        Text("AI 助手")
                            .font(.caption)
                    }
                }
                .buttonStyle(.plain)
                
                // My story
                Button {
                    showComposer = true
                } label: {
                    VStack(spacing: 6) {
                        ZStack {
                            Circle()
                                .fill(NeoTrixTheme.Colors.placeholder)
                                .frame(width: 64, height: 64)
                            
                            Image(systemName: "plus")
                                .font(.title2)
                                .foregroundColor(.secondary)
                        }
                        
                        Text("My Story")
                            .font(.caption)
                    }
                }
                .buttonStyle(.plain)
                
                ForEach(stories) { story in
                    StoryRing(story: story) {
                        showViewer = true
                    }
                }
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
        #if os(iOS)
        .fullScreenCover(isPresented: $showViewer) {
            StoryViewerView(stories: stories, onSeen: onSeen)
        }
        #else
        .sheet(isPresented: $showViewer) {
            StoryViewerView(stories: stories, onSeen: onSeen)
        }
        #endif
        .sheet(isPresented: $showComposer) {
            StoryComposerView { caption, isCloseFriends in
                onPublish(caption, isCloseFriends)
            }
        }
    }
}