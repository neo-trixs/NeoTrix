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
    @Published public var selectedFilter: ChatListFilter = .all
    @Published public var searchText = ""
    @Published public var isSearching = false
    
    private let core = NeoGramCore.shared
    private var cancellables = Set<AnyCancellable>()
    
    public init() {
        loadChats()
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
                StoriesBar(stories: storyViewModel.stories, onAI: { showAI = true })
                
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
                    // 融合：联系人 Section（原 ContactsView 占位）
                    if viewModel.searchText.isEmpty {
                        Section {
                            ForEach(0..<3, id: \.self) { index in
                                HStack {
                                    Circle()
                                        .fill(Color.blue.opacity(0.3))
                                        .frame(width: 40, height: 40)
                                        .overlay(
                                            Text("\(index + 1)")
                                                .foregroundColor(.blue)
                                        )
                                    
                                    Text("Contact \(index + 1)")
                                }
                            }
                        } header: {
                            Text("联系人")
                        }
                        
                        // 融合：通话（原 CallsView 占位）
                        Section {
                            ForEach(0..<2, id: \.self) { index in
                                HStack {
                                    Image(systemName: index % 2 == 0 ? "phone.arrow.up.right.fill" : "phone.arrow.down.left.fill")
                                        .foregroundColor(index % 2 == 0 ? .red : .green)
                                    
                                    Text("Call \(index + 1)")
                                    
                                    Spacer()
                                    
                                    Text("Today")
                                        .font(.caption)
                                        .foregroundColor(.secondary)
                                }
                            }
                        } header: {
                            Text("最近通话")
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
                                    // Delete chat
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

struct FilterChip: View {
    let title: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.subheadline)
                .padding(.horizontal, 14)
                .padding(.vertical, 6)
                .background(isSelected ? Color.blue : Color.gray.opacity(0.15))
                .foregroundColor(isSelected ? .white : .primary)
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}

struct ChatListRow: View {
    let chat: ChatListItem
    
    var body: some View {
        HStack(spacing: 12) {
            // Avatar
            ZStack(alignment: .bottomTrailing) {
                ZStack {
                    Circle()
                        .fill(chat.avatarColor.gradient)
                        .frame(width: 52, height: 52)
                    Text(String(chat.title.prefix(1)).uppercased())
                        .font(.title2.bold())
                        .foregroundColor(.white)
                }
                
                if chat.isOnline {
                    Circle()
                        .fill(Color.green)
                        .frame(width: 14, height: 14)
                        .overlay(Circle().stroke(Color.primary.opacity(0.2), lineWidth: 2))
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
                            .foregroundColor(.blue)
                    }
                    
                    if chat.isPremium {
                        Image(systemName: "star.fill")
                            .font(.caption2)
                            .foregroundColor(.yellow)
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
                            .background(Color.blue)
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
    @State private var showViewer = false
    
    public init(stories: [StoryItem], onAI: @escaping () -> Void) {
        self.stories = stories
        self.onAI = onAI
    }
    
    public var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 16) {
                // AI 助手入口（融合: AIHub 中枢）
                Button(action: onAI) {
                    VStack(spacing: 6) {
                        ZStack {
                            Circle()
                                .fill(
                                    LinearGradient(
                                        colors: [.purple, .blue],
                                        startPoint: .topLeading,
                                        endPoint: .bottomTrailing
                                    )
                                )
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
                    // 打开故事发布器（占位）
                } label: {
                    VStack(spacing: 6) {
                        ZStack {
                            Circle()
                                .fill(Color.gray.opacity(0.15))
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
            StoryViewerView(stories: stories)
        }
        #else
        .sheet(isPresented: $showViewer) {
            StoryViewerView(stories: stories)
        }
        #endif
    }
}