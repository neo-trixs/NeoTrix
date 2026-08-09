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
    /// 会话类型（对标 Telegram: peer 类型，替代脆弱的标题 contains 匹配）
    public let kind: ChatKind
    
    public enum ChatKind: Hashable {
        case personal
        case group
        case channel
        case bot
    }
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

/// New chat 导航目标（toolbar → contacts 选择 → 进入会话）
private struct NewChatDestination: Identifiable {
    let id = UUID()
    let title: String
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
    
    // MARK: - 持久化 Keys（chats 用户状态覆盖 mock，首次启动以 mock 播种）
    private static let mutedKey = "chatlist_muted_ids"
    private static let pinnedKey = "chatlist_pinned_ids"
    private static let readKey = "chatlist_read_ids"
    private static let deletedKey = "chatlist_deleted_ids"
    
    private let core = NeoGramCore.shared
    private var cancellables = Set<AnyCancellable>()
    
    public init() {
        loadChats()
        loadContacts()
        loadCalls()
        setupSearch()
    }
    
    // MARK: - 持久化辅助
    
    private func persistedIDs(for key: String) -> Set<Int64> {
        guard let data = UserDefaults.standard.data(forKey: key),
              let decoded = try? JSONDecoder().decode([Int64].self, from: data) else {
            return []
        }
        return Set(decoded)
    }
    
    private func persist(_ ids: Set<Int64>, for key: String) {
        if let data = try? JSONEncoder().encode(ids.sorted()) {
            UserDefaults.standard.set(data, forKey: key)
        }
    }
    
    private func loadChats() {
        let mock = Self.mockChats
        let defaults = UserDefaults.standard
        
        // 首次启动：以 mock 为准播种持久化状态（保证演示数据存活且后续可覆盖）
        let seedMuted = Set(mock.filter(\.isMuted).map(\.id))
        let seedPinned = Set(mock.filter(\.isPinned).map(\.id))
        let seedRead = Set(mock.filter { $0.unreadCount == 0 }.map(\.id))
        if defaults.data(forKey: Self.mutedKey) == nil { persist(seedMuted, for: Self.mutedKey) }
        if defaults.data(forKey: Self.pinnedKey) == nil { persist(seedPinned, for: Self.pinnedKey) }
        if defaults.data(forKey: Self.readKey) == nil { persist(seedRead, for: Self.readKey) }
        
        // 应用持久化覆盖 mock（集合为权威状态：不在集合内即未 mute/未 pin）
        let muted = persistedIDs(for: Self.mutedKey)
        let pinned = persistedIDs(for: Self.pinnedKey)
        let read = persistedIDs(for: Self.readKey)
        let deleted = persistedIDs(for: Self.deletedKey)
        
        chats = mock
            .filter { !deleted.contains($0.id) }
            .map { chat in
                var c = chat
                c.isMuted = muted.contains(c.id)
                c.isPinned = pinned.contains(c.id)
                if read.contains(c.id) { c.unreadCount = 0 }
                return c
            }
    }
    
    private static var mockChats: [ChatListItem] {
        [
            ChatListItem(id: 1, title: "NeoTrix AI", lastMessage: "Ready to help!", timestamp: Date(), unreadCount: 2, isPinned: true, isMuted: false, isOnline: true, avatarColor: .purple, isPremium: true, isVerified: true, kind: .bot),
            ChatListItem(id: 2, title: "Family Group", lastMessage: "Mom: Dinner at 7?", timestamp: Date().addingTimeInterval(-300), unreadCount: 0, isPinned: true, isMuted: false, isOnline: false, avatarColor: .green, isPremium: false, isVerified: false, kind: .group),
            ChatListItem(id: 3, title: "Work", lastMessage: "John: Meeting moved to 3pm", timestamp: Date().addingTimeInterval(-1800), unreadCount: 5, isPinned: false, isMuted: true, isOnline: true, avatarColor: .blue, isPremium: false, isVerified: false, kind: .personal),
            ChatListItem(id: 4, title: "Tech News", lastMessage: "New AI model released", timestamp: Date().addingTimeInterval(-3600), unreadCount: 0, isPinned: false, isMuted: false, isOnline: false, avatarColor: .orange, isPremium: true, isVerified: true, kind: .channel),
            ChatListItem(id: 5, title: "Design Team", lastMessage: "Alice: Updated the mockups", timestamp: Date().addingTimeInterval(-7200), unreadCount: 1, isPinned: false, isMuted: false, isOnline: false, avatarColor: .pink, isPremium: false, isVerified: false, kind: .group),
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
        
        // Filter by category（对标 Telegram: 按 peer 类型过滤，替代标题 contains 脆弱匹配）
        switch selectedFilter {
        case .all:
            break
        case .unread:
            result = result.filter { $0.unreadCount > 0 }
        case .personal:
            result = result.filter { $0.kind == .personal }
        case .groups:
            result = result.filter { $0.kind == .group }
        case .channels:
            result = result.filter { $0.kind == .channel }
        case .bots:
            result = result.filter { $0.kind == .bot }
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
        guard let index = chats.firstIndex(where: { $0.id == chatId }) else { return }
        chats[index].unreadCount = 0
        var read = persistedIDs(for: Self.readKey)
        read.insert(chatId)
        persist(read, for: Self.readKey)
    }
    
    public func toggleMute(_ chatId: Int64) {
        guard let index = chats.firstIndex(where: { $0.id == chatId }) else { return }
        chats[index].isMuted.toggle()
        var muted = persistedIDs(for: Self.mutedKey)
        if chats[index].isMuted { muted.insert(chatId) } else { muted.remove(chatId) }
        persist(muted, for: Self.mutedKey)
    }
    
    public func togglePin(_ chatId: Int64) {
        guard let index = chats.firstIndex(where: { $0.id == chatId }) else { return }
        chats[index].isPinned.toggle()
        var pinned = persistedIDs(for: Self.pinnedKey)
        if chats[index].isPinned { pinned.insert(chatId) } else { pinned.remove(chatId) }
        persist(pinned, for: Self.pinnedKey)
    }
    
    public func deleteChat(_ chatId: Int64) {
        chats.removeAll { $0.id == chatId }
        var deleted = persistedIDs(for: Self.deletedKey)
        deleted.insert(chatId)
        persist(deleted, for: Self.deletedKey)
        // 清理其余状态集（已删除会话不再参与排序/过滤）
        var muted = persistedIDs(for: Self.mutedKey); muted.remove(chatId); persist(muted, for: Self.mutedKey)
        var pinned = persistedIDs(for: Self.pinnedKey); pinned.remove(chatId); persist(pinned, for: Self.pinnedKey)
        var read = persistedIDs(for: Self.readKey); read.remove(chatId); persist(read, for: Self.readKey)
    }
}

// MARK: - Chat List View

public struct ChatListView: View {
    @StateObject private var viewModel = ChatListViewModel()
    @StateObject private var storyViewModel = StoryViewModel()
    @StateObject private var folderEngine = FolderEngine()
    @State private var selectedChat: ChatListItem?
    @State private var showSettings = false
    @State private var showPremium = false
    @State private var showAI = false
    @State private var showNewChat = false
    @State private var newChatDestination: NewChatDestination?
    
    public init() {}
    
    /// 当前显示的会话：选中文件夹时按文件夹过滤，否则走分类过滤（FolderEngine 接线）
    private var displayedChats: [ChatListItem] {
        guard let folderID = folderEngine.selectedFolderID,
              let folder = folderEngine.folders.first(where: { $0.id == folderID }) else {
            return viewModel.filteredChats
        }
        return folderEngine.chatsForFolder(folder, allChats: viewModel.filteredChats)
    }
    
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
                
                // Filter bar（分类 Tabs + 文件夹 Tabs，对标 Telegram: filters + folders）
                VStack(spacing: 0) {
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 8) {
                            ForEach(ChatListFilter.allCases, id: \.self) { filter in
                                FilterChip(
                                    title: filter.rawValue,
                                    isSelected: viewModel.selectedFilter == filter && folderEngine.selectedFolderID == nil
                                ) {
                                    withAnimation {
                                        viewModel.selectedFilter = filter
                                        if folderEngine.selectedFolderID != nil {
                                            folderEngine.deselectFolder()
                                        }
                                    }
                                }
                            }
                        }
                        .padding(.horizontal)
                        .padding(.vertical, 8)
                    }
                    
                    if !folderEngine.visibleFolders.isEmpty {
                        ScrollView(.horizontal, showsIndicators: false) {
                            HStack(spacing: 8) {
                                ForEach(folderEngine.visibleFolders) { folder in
                                    FilterChip(
                                        title: folder.name,
                                        isSelected: folderEngine.selectedFolderID == folder.id
                                    ) {
                                        withAnimation {
                                            if folderEngine.selectedFolderID == folder.id {
                                                folderEngine.deselectFolder()
                                            } else {
                                                folderEngine.selectFolder(folder)
                                            }
                                        }
                                    }
                                }
                            }
                            .padding(.horizontal)
                            .padding(.bottom, 8)
                        }
                    }
                }
                
                // Chat list
                List {
                    // 搜索无匹配 → 空结果态
                    if !viewModel.searchText.isEmpty && displayedChats.isEmpty {
                        Section {
                            NeoTrixEmptyState(
                                icon: "magnifyingglass",
                                title: "No Results",
                                message: "No chats match \"\(viewModel.searchText)\""
                            )
                            .frame(maxWidth: .infinity, minHeight: 220)
                            .listRowBackground(Color.clear)
                            .listRowSeparator(.hidden)
                        }
                    } else {
                        // 融合：联系人 Section（真实模型，替代 Contact N 占位）
                        if viewModel.searchText.isEmpty {
                            Section {
                                ForEach(viewModel.contacts) { contact in
                                    HStack(spacing: 12) {
                                        ZStack(alignment: .bottomTrailing) {
                                            NeoTrixAvatar(
                                                title: contact.name,
                                                size: 40,
                                                gradient: LinearGradient(colors: [contact.avatarColor, contact.avatarColor.opacity(0.7)],
                                                                         startPoint: .topLeading, endPoint: .bottomTrailing)
                                            )
                                            
                                            if contact.isOnline {
                                                neoTrixOnlineDot(size: 10)
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
                        
                        ForEach(displayedChats) { chat in
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
                                            viewModel.deleteChat(chat.id)
                                        }
                                    } label: {
                                        Label("Delete", systemImage: "trash")
                                    }
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
                                .foregroundColor(NeoTrixTheme.Colors.premium)
                        }
                        
                        Button {
                            showNewChat = true
                        } label: {
                            Image(systemName: "square.and.pencil")
                        }
                        .confirmationDialog("New Chat", isPresented: $showNewChat, titleVisibility: .visible) {
                            ForEach(viewModel.contacts) { contact in
                                Button(contact.name) {
                                    newChatDestination = NewChatDestination(title: contact.name)
                                }
                            }
                            Button("Cancel", role: .cancel) {}
                        }
                    }
                }
                #else
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        showPremium = true
                    } label: {
                        Image(systemName: "star.circle.fill")
                            .foregroundColor(NeoTrixTheme.Colors.premium)
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
                    ChatView(title: "AI 助手")
                }
            }
            .navigationDestination(item: $selectedChat) { chat in
                ChatView(title: chat.title)
            }
            .navigationDestination(item: $newChatDestination) { destination in
                ChatView(title: destination.title)
            }
        }
    }
}

// MARK: - 通话方向辅助（真实通话记录渲染）

private func iconForCall(_ direction: CallRecord.Direction) -> String {
    switch direction {
    case .incoming: return "phone.arrow.down.left.fill"
    case .outgoing: return "phone.arrow.up.right.fill"
    case .missed: return "phone.arrow.down.left.fill"
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

/// 会话时间戳（对标 Telegram: 今天显示时间 / 昨天显示 "Yesterday" / 更早显示日期）
private func chatTimestampText(_ date: Date) -> String {
    let calendar = Calendar.current
    if calendar.isDateInToday(date) {
        return date.formatted(date: .omitted, time: .shortened)
    }
    if calendar.isDateInYesterday(date) { return "Yesterday" }
    return date.formatted(date: .abbreviated, time: .omitted)
}

struct FilterChip: View {
    let title: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(NeoTrixTheme.Fonts.subheadline)
                .neoTrixCapsule(isSelected: isSelected)
        }
        .buttonStyle(.plain)
    }
}

struct ChatListRow: View {
    let chat: ChatListItem
    
    var body: some View {
        HStack(spacing: 12) {
            // Avatar（设计系统: 头像 + 在线点 neoTrixOnlineDot）
            ZStack(alignment: .bottomTrailing) {
                NeoTrixAvatar(title: chat.title, size: 52,
                              gradient: LinearGradient(colors: [chat.avatarColor, chat.avatarColor.opacity(0.7)],
                                                       startPoint: .topLeading, endPoint: .bottomTrailing))
                
                if chat.isOnline {
                    neoTrixOnlineDot(size: 14)
                        .padding(1)
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
                    
                    Text(chatTimestampText(chat.timestamp))
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
                        neoTrixUnreadBadge(chat.unreadCount)
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