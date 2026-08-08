// ReactionsUI - Telegram message reactions (Premium feature)
// Mirrors Telegram's ReactionPicker + reactions management

import SwiftUI

// MARK: - Reaction Model

public struct MessageReaction: Identifiable {
    public let id: String
    public let emoji: String
    public let isPremium: Bool
    public let animationType: AnimationType
    public var isSelected: Bool = false
    public var count: Int = 0
    
    public enum AnimationType {
        case staticImage
        case animated
        case fullScreen
        case multiLayer
    }
}

// MARK: - Reaction Data

public struct ReactionData {
    public static let freeReactions: [String] = [
        "👍", "👎", "❤️", "🔥", "🥰", "👏", "😁", "🤔", "🤯", "😱", "🤬", "😢", "🎉", "🤩", "🤮", "💩", "🙏", "👌", "🕊", "🤡", "🥱", "🥴", "😍", "🐳", "❤️‍🔥", "🌚", "🌭", "💯", "🤣", "⚡", "🍌", "🏆", "💔", "🤨", "😐", "🍓", "🍾", "💋", "🖕", "😈", "😴", "😭", "🤓", "👻", "👨‍💻", "👀", "🎃", "🙈", "😇", "😨", "🤝", "✍️", "🤗", "🫡", "🎅", "🎄", "☃️", "💅", "🤪", "🗿", "🆒", "💘", "🙉", "🦄", "😘", "💊", "🙊", "😎", "👾", "🤷‍♂️", "🤷", "🤷‍♀️", "😡"
    ]
    
    public static let premiumReactions: [String] = [
        "😀", "😃", "😄", "😁", "😆", "😅", "😂", "🤣", "😊", "😇", "🙂", "🙃", "😉", "😌", "😍", "🥰", "😘", "😗", "😙", "😚", "😋", "😛", "😝", "😜", "🤪", "🤨", "🧐", "🤓", "😎", "🥸", "🤩", "🥳", "😏", "😒", "😞", "😔", "😟", "😕", "🙁", "☹️", "😣", "😖", "😫", "😩", "🥺", "😢", "😭", "😤", "😠", "😡", "🤬", "🤯", "😳", "🥵", "🥶", "😱", "😨", "😰", "😥", "😓", "🤗", "🤔", "🫢", "🫣", "🤭", "🤫", "🫠", "🤥", "😶", "🫥", "😐", "😑", "😬", "🙄", "😯", "😦", "😧", "😮", "😲", "🥱", "😴", "🤤", "😪", "😵", "🫤", "🤐", "🥴", "🤒", "🤕", "🤑", "🤠", "😈", "👿", "🤡", "💩", "👻", "💀", "☠️", "👽", "👾", "🤖", "🎃", "😺", "😸", "😹", "😻", "😼", "😽", "🙀", "😿", "😾", "👋", "🤚", "🖐", "✋", "🖖", "👌", "🤌", "🤏", "✌️", "🤞", "🫰", "🤟", "🤘", "🤙", "👈", "👉", "👆", "🖕", "👇", "☝️", "🫵", "👍", "👎", "✊", "👊", "🤛", "🤜", "👏", "🙌", "🫶", "🤲", "🤝", "🙏", "✍️", "💅", "🤳", "💪", "🦾", "🦿", "🦵", "🦶", "👂", "🦻", "👃", "🧠", "🫀", "🫁", "🦷", "🦴", "👀", "👁", "👅", "👄"
    ]
    
    public static var all: [MessageReaction] {
        let free = freeReactions.map { MessageReaction(id: $0, emoji: $0, isPremium: false, animationType: .staticImage) }
        let premium = premiumReactions.map { MessageReaction(id: $0, emoji: $0, isPremium: true, animationType: .animated) }
        return free + premium
    }
}

// MARK: - Reaction Manager

@MainActor
public final class ReactionManager: ObservableObject {
    @Published public var favoriteReactions: [String] = ["👍", "❤️", "🔥", "🥰", "👏"]
    @Published public var isPremium = false
    
    private let core = NeoGramCore.shared
    
    public init() {
        loadPreferences()
    }
    
    private func loadPreferences() {
        isPremium = UserDefaults.standard.bool(forKey: "is_premium")
        if let saved = UserDefaults.standard.array(forKey: "favorite_reactions") as? [String] {
            favoriteReactions = saved
        }
    }
    
    public func canUse(_ reaction: MessageReaction) -> Bool {
        return !reaction.isPremium || isPremium
    }
    
    public func setFavorite(_ emoji: String) {
        if !favoriteReactions.contains(emoji) {
            favoriteReactions.append(emoji)
            UserDefaults.standard.set(favoriteReactions, forKey: "favorite_reactions")
        }
    }
    
    public func removeFavorite(_ emoji: String) {
        favoriteReactions.removeAll { $0 == emoji }
        UserDefaults.standard.set(favoriteReactions, forKey: "favorite_reactions")
    }
}

// MARK: - Reaction Picker

public struct ReactionPickerView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var manager = ReactionManager()
    let onSelect: (String) -> Void
    
    @State private var searchText = ""
    @State private var selectedTab: Int = 0
    @State private var fullScreenEmoji: String?
    
    public init(onSelect: @escaping (String) -> Void) {
        self.onSelect = onSelect
    }
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                // Tab selector
                Picker("", selection: $selectedTab) {
                    Text("All").tag(0)
                    Text("Recent").tag(1)
                    Text("Premium").tag(2)
                }
                .pickerStyle(.segmented)
                .padding()
                
                // Search
                if !manager.isPremium {
                    SearchField(text: $searchText, placeholder: "Search emoji")
                        .padding(.horizontal)
                }
                
                // Reaction grid
                ScrollView {
                    LazyVGrid(columns: Array(repeating: GridItem(.flexible()), count: 6), spacing: 12) {
                        let reactions = filteredReactions
                        ForEach(reactions) { reaction in
                            Button {
                                if manager.canUse(reaction) {
                                    // 融合: 全屏动画效果（AnimatedEmoji 接线）
                                    fullScreenEmoji = reaction.emoji
                                    onSelect(reaction.emoji)
                                    dismiss()
                                }
                            } label: {
                                ZStack(alignment: .topTrailing) {
                                    Text(reaction.emoji)
                                        .font(.system(size: 28))
                                    
                                    if reaction.isPremium && !manager.isPremium {
                                        Image(systemName: "star.fill")
                                            .font(.system(size: 10))
                                            .foregroundColor(NeoTrixTheme.Colors.premium)
                                            .offset(x: 6, y: -6)
                                    }
                                }
                                .padding(8)
.background(NeoTrixTheme.Colors.inputBackground)
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                            }
                            .buttonStyle(.plain)
                            .disabled(reaction.isPremium && !manager.isPremium)
                        }
                    }
                    .padding()
                }
            }
            .navigationTitle("Reactions")
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
        .overlay {
            if let emoji = fullScreenEmoji {
                FullScreenEmojiEffect(emoji: emoji)
                    .allowsHitTesting(false)
                    .onAppear {
                        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
                            fullScreenEmoji = nil
                        }
                    }
            }
        }
    }
    
    private var filteredReactions: [MessageReaction] {
        var result = ReactionData.all
        
        if !searchText.isEmpty {
            result = result.filter { $0.emoji.contains(searchText) }
        }
        
        switch selectedTab {
        case 0:
            break
        case 1:
            result = result.filter { manager.favoriteReactions.contains($0.emoji) }
        case 2:
            result = result.filter { $0.isPremium }
        default:
            break
        }
        
        return result
    }
}

struct SearchField: View {
    @Binding var text: String
    let placeholder: String
    
    var body: some View {
        HStack {
            Image(systemName: "magnifyingglass")
                .foregroundColor(.secondary)
            TextField(placeholder, text: $text)
                .textFieldStyle(.plain)
            
            if !text.isEmpty {
                Button {
                    text = ""
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(10)
        .background(NeoTrixTheme.Colors.inputBackground)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}

// MARK: - Reaction Bar (attached to messages)
// 注: ReactionBarView 已删除（Dark Forest）— 消息 reactions 显示由 ChatUI.MessageBubbleView
// 内联实现（ChatMessage.Reaction 模型），此组件用不同模型（MessageReaction）无法复用且无消费者。