// FilterEngine - 消息过滤引擎 (Fusion Architecture)
// 融合: Swiftgram Message Filter + Turrit 关键词屏蔽 + Nicegram Ghost Mode
// 独有: AI 语义过滤 (E8) + 关键词规则 + 频道广告过滤 + 指定用户屏蔽

import Foundation
import Combine
import SwiftUI

// MARK: - Filter Models

public struct FilterRule: Identifiable, Equatable, Codable {
    public let id: UUID
    public var keyword: String
    public var isEnabled: Bool
    public var scope: FilterScope
    
    public enum FilterScope: String, CaseIterable, Codable {
        case all = "All Chats"
        case channels = "Channels Only"
        case groups = "Groups Only"
        case dms = "Direct Messages"
    }
    
    public init(id: UUID = UUID(), keyword: String, isEnabled: Bool = true, scope: FilterScope = .all) {
        self.id = id
        self.keyword = keyword
        self.isEnabled = isEnabled
        self.scope = scope
    }
}

public struct BlockedUser: Identifiable, Equatable, Codable {
    public let id: Int64
    public let name: String
    /// 头像色板索引（持久化索引而非 Color，保证重启动后颜色稳定）
    public let colorIndex: Int
    
    public init(id: Int64, name: String, colorIndex: Int = 0) {
        self.id = id
        self.name = name
        self.colorIndex = colorIndex
    }
    
    /// 头像色（从 FilterEngine 色板映射）
    public var avatarColor: Color {
        FilterEngine.avatarPalette[colorIndex % FilterEngine.avatarPalette.count]
    }
}

// MARK: - Filter Engine

@MainActor
public final class FilterEngine: ObservableObject {
    @Published public var rules: [FilterRule] = []
    @Published public var blockedUsers: [BlockedUser] = []
    @Published public var filterChannelAds = true {
        didSet { saveRules() }
    }
    @Published public var filterSpam = true {
        didSet { saveRules() }
    }
    @Published public var useAISemanticFilter = true {
        didSet { saveRules() }
    }
    
    /// 屏蔽用户头像色板（colorIndex 持久化索引映射，避免 Color 不可 Codable）
    fileprivate static let avatarPalette: [Color] = [.pink, .blue, .orange, .green, .purple, .teal]
    
    private static let blockedUsersKey = "filter_blocked_users"
    private static let filterChannelAdsKey = "filter_channel_ads"
    private static let filterSpamKey = "filter_spam"
    private static let useAISemanticFilterKey = "filter_ai_semantic"
    
    private let aiHub = AIHub.shared
    
    public init() {
        loadRules()
    }
    
    private func loadRules() {
        // Load from UserDefaults
        if let data = UserDefaults.standard.data(forKey: "filter_rules"),
           let decoded = try? JSONDecoder().decode([FilterRule].self, from: data) {
            rules = decoded
        } else {
            // Default rules
            rules = [
                FilterRule(keyword: "spam", scope: .channels),
                FilterRule(keyword: "promo", scope: .channels),
                FilterRule(keyword: "discount", scope: .channels),
                FilterRule(keyword: "click here", scope: .all),
            ]
        }
        
        // 开关持久化（未设置过时保持默认值 true）
        if UserDefaults.standard.object(forKey: Self.filterChannelAdsKey) != nil {
            filterChannelAds = UserDefaults.standard.bool(forKey: Self.filterChannelAdsKey)
        }
        if UserDefaults.standard.object(forKey: Self.filterSpamKey) != nil {
            filterSpam = UserDefaults.standard.bool(forKey: Self.filterSpamKey)
        }
        if UserDefaults.standard.object(forKey: Self.useAISemanticFilterKey) != nil {
            useAISemanticFilter = UserDefaults.standard.bool(forKey: Self.useAISemanticFilterKey)
        }
        
        // 屏蔽用户
        if let data = UserDefaults.standard.data(forKey: Self.blockedUsersKey),
           let decoded = try? JSONDecoder().decode([BlockedUser].self, from: data) {
            blockedUsers = decoded
        }
    }
    
    private func saveRules() {
        if let data = try? JSONEncoder().encode(rules) {
            UserDefaults.standard.set(data, forKey: "filter_rules")
        }
        UserDefaults.standard.set(filterChannelAds, forKey: Self.filterChannelAdsKey)
        UserDefaults.standard.set(filterSpam, forKey: Self.filterSpamKey)
        UserDefaults.standard.set(useAISemanticFilter, forKey: Self.useAISemanticFilterKey)
    }
    
    private func saveBlockedUsers() {
        if let data = try? JSONEncoder().encode(blockedUsers) {
            UserDefaults.standard.set(data, forKey: Self.blockedUsersKey)
        }
    }
    
    public func addRule(keyword: String, scope: FilterRule.FilterScope = .all) {
        rules.append(FilterRule(keyword: keyword, scope: scope))
        saveRules()
    }
    
    public func removeRule(_ rule: FilterRule) {
        rules.removeAll { $0.id == rule.id }
        saveRules()
    }
    
    public func toggleRule(_ rule: FilterRule) {
        if let index = rules.firstIndex(where: { $0.id == rule.id }) {
            rules[index].isEnabled.toggle()
            saveRules()
        }
    }
    
    public func blockUser(_ user: BlockedUser) {
        blockedUsers.append(user)
        saveBlockedUsers()
    }
    
    /// 按用户名屏蔽（屏蔽列表页 "Add User" 入口）
    public func blockUser(name: String) {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        let id = Int64(Date().timeIntervalSince1970 * 1000)  // 单调递增，避免与 mock 冲突
        blockedUsers.append(BlockedUser(id: id, name: trimmed, colorIndex: blockedUsers.count % Self.avatarPalette.count))
        saveBlockedUsers()
    }
    
    public func unblockUser(_ user: BlockedUser) {
        blockedUsers.removeAll { $0.id == user.id }
        saveBlockedUsers()
    }
    
    // MARK: - Filtering
    
    public func shouldFilter(_ text: String, isChannel: Bool = false, isGroup: Bool = false, isDM: Bool = false) async -> Bool {
        // 1. Keyword rules
        for rule in rules where rule.isEnabled {
            let scopeMatches = switch rule.scope {
            case .all: true
            case .channels: isChannel
            case .groups: isGroup
            case .dms: isDM
            }
            if scopeMatches && text.lowercased().contains(rule.keyword.lowercased()) {
                return true
            }
        }
        
        // 2. Channel ads
        if filterChannelAds && isChannel && looksLikeAd(text) {
            return true
        }
        
        // 3. AI semantic filter (E8)
        if useAISemanticFilter && filterSpam {
            let result = await aiHub.process(AIHubRequest(text: text, type: .filter))
            if result.text.lowercased() == "spam" && result.confidence > 0.6 {
                return true
            }
        }
        
        return false
    }
    
    private func looksLikeAd(_ text: String) -> Bool {
        let adPatterns = ["% off", "buy now", "limited time", "act fast", "free trial", "subscribe"]
        let matches = adPatterns.filter { text.lowercased().contains($0) }.count
        return matches >= 2
    }
}

// MARK: - Filter Settings View

public struct FilterSettingsView: View {
    @StateObject private var engine = FilterEngine()
    @State private var newKeyword = ""
    @State private var newBlockedUserName = ""
    @State private var showBlockUserAlert = false
    
    public init() {}
    
    public var body: some View {
        Form {
            Section("AI Semantic Filter") {
                Toggle("AI spam detection", isOn: $engine.useAISemanticFilter)
                Toggle("Filter channel ads", isOn: $engine.filterChannelAds)
                Toggle("Filter spam", isOn: $engine.filterSpam)
            }
            
            Section("Keyword Rules") {
                HStack {
                    TextField("Add keyword…", text: $newKeyword)
                    Button("Add") {
                        let keyword = newKeyword
                        newKeyword = ""
                        engine.addRule(keyword: keyword)
                    }
                    .disabled(newKeyword.trimmingCharacters(in: .whitespaces).isEmpty)
                }
                
                ForEach(engine.rules) { rule in
                    HStack {
                        Button {
                            engine.toggleRule(rule)
                        } label: {
                            Image(systemName: rule.isEnabled ? "checkmark.circle.fill" : "circle")
                                .foregroundColor(rule.isEnabled ? NeoTrixTheme.Colors.accent : .secondary)
                        }
                        .buttonStyle(.plain)
                        
                        Text(rule.keyword)
                        
                        Spacer()
                        
                        Text(rule.scope.rawValue)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                    .swipeActions {
                        Button(role: .destructive) {
                            engine.removeRule(rule)
                        } label: {
                            Label("Delete", systemImage: "trash")
                        }
                    }
                }
            }
            
            Section("Blocked Users") {
                Button {
                    showBlockUserAlert = true
                } label: {
                    Label("Add User", systemImage: "person.badge.plus")
                }
                if engine.blockedUsers.isEmpty {
                    Text("No blocked users")
                        .foregroundColor(.secondary)
                }
                ForEach(engine.blockedUsers) { user in
                    HStack {
                        Circle()
                            .fill(user.avatarColor)
                            .frame(width: 32, height: 32)
                            .overlay(Text(String(user.name.prefix(1))).foregroundColor(.white))
                        Text(user.name)
                        Spacer()
                        Button("Unblock") {
                            engine.unblockUser(user)
                        }
                        .font(.caption)
                    }
                }
            }
        }
        .navigationTitle("Message Filter")
        .alert("Block User", isPresented: $showBlockUserAlert) {
            TextField("Username", text: $newBlockedUserName)
            Button("Block") {
                engine.blockUser(name: newBlockedUserName)
                newBlockedUserName = ""
            }
            Button("Cancel", role: .cancel) {}
        }
    }
}