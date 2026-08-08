// ChatListUI - Telegram-style chat list with folders
// Mirrors Telegram's ChatListController + ChatListFilters

import SwiftUI
import Combine

// MARK: - Chat List Models

public struct ChatListItem: Identifiable {
    public let id: Int64
    public let title: String
    public let lastMessage: String
    public let timestamp: Date
    public let unreadCount: Int
    public let isPinned: Bool
    public let isMuted: Bool
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
    @Published public var chats: [ChatList] = []
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
            ChatList(id: 1, title: "NeoTrix AI", lastMessage: "Ready to help!", timestamp: Date(), unreadCount: 2, isPinned: true, isMuted: false, isOnline: true, avatarColor: .purple, isPremium: true, isVerified: true),
            ChatList(id: 2, title: "Family Group", lastMessage: "Mom: Dinner at 7?", timestamp: Date().addingTimeInterval(-300), unreadCount: 0, isPinned: true, isMuted: false, isOnline: false, avatarColor: .green, isPremium: false, isVerified: false),
            ChatList(id: 3, title: "Work", lastMessage: "John: Meeting moved to 3pm", timestamp: Date().addingTimeInterval(-1800), unreadCount: 5, isPinned: false, isMuted: true, isOnline: true, avatarColor: .blue, isPremium: false, isVerified: false),
            ChatList(id: 4, title: "Tech News", lastMessage: "New AI model released", timestamp: Date().addingTimeInterval(-3600), unreadCount: 0, isPinned: false, isMuted: false, isOnline: false, avatarColor: .orange, isPremium: true, isVerified: true),
            ChatList(id: 5, title: "Design Team", lastMessage: "Alice: Updated the mockups", timestamp: Date().addingTimeInterval(-7200), unreadCount: 1, isPinned: false, isMuted: false, isOnline: false, avatarColor: .pink, isPremium: false, isVerified: false),
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
    
    public var filteredChats: [ChatList] {
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
    @State private var selectedChat: ChatList?
    @State private var showSettings = false
    @State private var showPremium = false
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
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
            }
            .sheet(isPresented: $showSettings) {
                SettingsView()
            }
            .sheet(isPresented: $showPremium) {
                PremiumIntroView()
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
                .background(isSelected ? Color.blue : Color(.systemGray6))
                .foregroundColor(isSelected ? .white : .primary)
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}

struct ChatListRow: View {
    let chat: ChatList
    
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
                        .overlay(Circle().stroke(Color(.systemBackground), lineWidth: 2))
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