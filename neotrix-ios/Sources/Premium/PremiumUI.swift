// PremiumUI - Telegram Premium features implementation
// Mirrors Telegram's PremiumUI module

import SwiftUI
import StoreKit

// MARK: - Premium Status

public enum PremiumTier: String, CaseIterable, Identifiable {
    case free = "Free"
    case monthly = "Monthly"
    case yearly = "Yearly"
    case biannual = "Biannual"
    
    public var id: String { rawValue }
    
    /// 层级排序（修复 P0: 此前 isFeatureEnabled 用 rawValue 字典序比较，
    /// "Biannual" < "Monthly"（B<M）→ 两年订阅者打不开 Monthly 档功能；"Yearly" >= "Biannual"（Y>B）→ 年付错误解锁两年档）
    public var rank: Int {
        switch self {
        case .free: return 0
        case .monthly: return 1
        case .yearly: return 2
        case .biannual: return 3
        }
    }
    
    public var displayName: String {
        switch self {
        case .free: return "Free"
        case .monthly: return "Monthly"
        case .yearly: return "Yearly (Save 20%)"
        case .biannual: return "2 Years (Save 40%)"
        }
    }
    
    public var price: String {
        switch self {
        case .free: return "Free"
        case .monthly: return "$4.99/mo"
        case .yearly: return "$39.99/yr"
        case .biannual: return "$59.99/2yr"
        }
    }
}

public struct PremiumFeatures {
    public static let all: [PremiumFeature] = [
        PremiumFeature(id: "double_limits", name: "Double Limits", description: "2x channels, folders, pins, saved messages", icon: "2.square.fill", tier: .monthly),
        PremiumFeature(id: "no_ads", name: "No Ads", description: "Ad-free experience in channels and chats", icon: "nosign", tier: .monthly),
        PremiumFeature(id: "voice_to_text", name: "Voice-to-Text", description: "Convert voice messages to text", icon: "waveform", tier: .monthly),
        PremiumFeature(id: "peer_colors", name: "Peer Colors", description: "Custom chat colors and themes", icon: "paintpalette", tier: .monthly),
        PremiumFeature(id: "animated_emoji", name: "Animated Emoji", description: "Full-screen animated emoji reactions", icon: "face.smiling", tier: .monthly),
        PremiumFeature(id: "reactions", name: "Custom Reactions", description: "Unlimited custom emoji reactions", icon: "heart.fill", tier: .monthly),
        PremiumFeature(id: "emoji_status", name: "Emoji Status", description: "Set animated emoji next to your name", icon: "person.circle", tier: .monthly),
        PremiumFeature(id: "app_icons", name: "App Icons", description: "Custom app icons for home screen", icon: "app.badge", tier: .monthly),
        PremiumFeature(id: "chat_folders", name: "Chat Folders", description: "Unlimited folders with auto-sorting", icon: "folder.fill", tier: .monthly),
        PremiumFeature(id: "translation", name: "Translation", description: "Instant message translation", icon: "character.bubble", tier: .monthly),
        PremiumFeature(id: "wallpapers", name: "Animated Wallpapers", description: "Dynamic chat backgrounds", icon: "photo", tier: .monthly),
        PremiumFeature(id: "saved_tags", name: "Saved Tags", description: "Organize saved messages with tags", icon: "tag.fill", tier: .monthly),
        PremiumFeature(id: "channel_boost", name: "Channel Boost", description: "Boost channels for perks", icon: "bolt.fill", tier: .yearly),
        PremiumFeature(id: "forum_topic_icon", name: "Forum Topic Icons", description: "Custom icons for forum topics", icon: "number.circle", tier: .yearly),
        PremiumFeature(id: "paid_messages", name: "Paid Messages", description: "Receive paid messages from non-contacts", icon: "dollarsign.circle", tier: .yearly),
        PremiumFeature(id: "pm_noforwards", name: "No Forwards", description: "Prevent forwarding of your messages", icon: "arrow.turn.up.left", tier: .yearly),
        PremiumFeature(id: "ai_compose", name: "AI Compose", description: "NeoTrix AI-powered message composition", icon: "brain", tier: .yearly),
        PremiumFeature(id: "stories", name: "Stories", description: "Post and view stories", icon: "circle.dotted", tier: .monthly),
        PremiumFeature(id: "story_quality", name: "Story Quality", description: "High-quality story uploads", icon: "photo.badge.plus", tier: .yearly),
        PremiumFeature(id: "story_links", name: "Story Links", description: "Add links to stories", icon: "link", tier: .yearly),
        PremiumFeature(id: "permanent_views", name: "Permanent Views History", description: "See who viewed your stories forever", icon: "eye.fill", tier: .yearly),
    ]
}

public struct PremiumFeature: Identifiable {
    public let id: String
    public let name: String
    public let description: String
    public let icon: String
    public let tier: PremiumTier
}

// MARK: - Premium Manager

@MainActor
public final class PremiumManager: ObservableObject {
    @Published public var currentTier: PremiumTier = .free
    @Published public var isPremium: Bool = false
    @Published public var features: [PremiumFeature] = PremiumFeatures.all
    @Published public var purchaseInProgress = false
    
    private let core = NeoGramCore.shared
    
    public init() {
        loadPremiumStatus()
    }
    
    private func loadPremiumStatus() {
        // Load from UserDefaults or receipt validation
        if let tierRaw = UserDefaults.standard.string(forKey: "premium_tier"),
           let tier = PremiumTier(rawValue: tierRaw) {
            currentTier = tier
            isPremium = tier != .free
        }
    }
    
    public func purchase(_ tier: PremiumTier) async throws {
        purchaseInProgress = true
        defer { purchaseInProgress = false }
        
        // Implement StoreKit 2 purchase flow
        // This is a simplified version
        try await Task.sleep(nanoseconds: 1_000_000_000)
        
        currentTier = tier
        isPremium = tier != .free
        UserDefaults.standard.set(tier.rawValue, forKey: "premium_tier")
    }
    
    public func restorePurchases() async throws {
        // 真实实现: StoreKit 2 AppTransaction.refresh() + 收据校验
        // 当前: 模拟恢复（The Spice Must Flow: 有结果反馈，非空操作）
        purchaseInProgress = true
        defer { purchaseInProgress = false }
        
        try await Task.sleep(nanoseconds: 800_000_000)
        
        // 若本地已有 premium_tier 记录则恢复，否则保持 free
        if let tierRaw = UserDefaults.standard.string(forKey: "premium_tier"),
           let tier = PremiumTier(rawValue: tierRaw), tier != .free {
            currentTier = tier
            isPremium = true
        }
    }
    
    public func isFeatureEnabled(_ featureId: String) -> Bool {
        guard let feature = features.first(where: { $0.id == featureId }) else { return false }
        // 修复 P0: 用 rank 语义比较（此前 rawValue 字典序 → tier 比较失效）
        return isPremium && currentTier.rank >= feature.tier.rank
    }
}

// MARK: - Premium Intro Screen

public struct PremiumIntroView: View {
    @StateObject private var manager = PremiumManager()
    @Environment(\.dismiss) private var dismiss
    @State private var showLimits = false
    @State private var showGifts = false
    @State private var showBoosts = false
    
    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Hero
                    VStack(spacing: 16) {
                        Image(systemName: "star.circle.fill")
                            .font(.system(size: 80))
                            .foregroundStyle(NeoTrixTheme.Colors.premium.gradient)
                        
                        Text("NeoGram Premium")
                            .font(.largeTitle.bold())
                        
                        Text("Unlock the full power of AI-enhanced messaging")
                            .font(.body)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.top, 40)
                    
                    // Feature list
                    VStack(spacing: 12) {
                        ForEach(manager.features) { feature in
                            PremiumFeatureRow(feature: feature, isEnabled: manager.isFeatureEnabled(feature.id))
                        }
                    }
                    .padding(.horizontal)
                    
                    // Tier selection
                    VStack(spacing: 12) {
                        ForEach(PremiumTier.allCases.filter { $0 != .free }) { tier in
                            PremiumTierButton(
                                tier: tier,
                                isSelected: manager.currentTier == tier,
                                isPurchasing: manager.purchaseInProgress
                            ) {
                                Task { try? await manager.purchase(tier) }
                            }
                        }
                    }
                    .padding(.horizontal)
                    
                    // Restore
                    Button("Restore Purchases") {
                        Task { try? await manager.restorePurchases() }
                    }
                    .font(.footnote)
                    .foregroundColor(.secondary)
                    .padding(.top, 8)
                    
                    // 修复: 死视图接线（PremiumLimitsView/GiftsView/BoostLevelsView 此前无任何调用者）
                    // 对标 Telegram Premium 设置页: Limits / Gifts / Boost 入口
                    VStack(spacing: 0) {
                        Divider()
                        Button {
                            showLimits = true
                        } label: {
                            HStack {
                                Label("Limits", systemImage: "gauge.with.dots.needle.50percent")
                                Spacer()
                                Image(systemName: "chevron.right")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.vertical, 12)
                        }
                        .buttonStyle(.plain)
                        
                        Divider()
                        Button {
                            showGifts = true
                        } label: {
                            HStack {
                                Label("Premium Gifts", systemImage: "gift.fill")
                                Spacer()
                                Image(systemName: "chevron.right")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.vertical, 12)
                        }
                        .buttonStyle(.plain)
                        
                        Divider()
                        Button {
                            showBoosts = true
                        } label: {
                            HStack {
                                Label("Boost Levels", systemImage: "bolt.fill")
                                Spacer()
                                Image(systemName: "chevron.right")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.vertical, 12)
                        }
                        .buttonStyle(.plain)
                    }
                    .padding(.horizontal)
                    .background(NeoTrixTheme.Colors.surface)
                    .clipShape(RoundedRectangle(cornerRadius: 12))
                    .padding(.horizontal)
                    
                    Spacer(minLength: 40)
                }
            }
            .navigationTitle("Premium")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") { dismiss() }
                }
                #else
                ToolbarItem(placement: .primaryAction) {
                    Button("Done") { dismiss() }
                }
                #endif
            }
            .navigationDestination(isPresented: $showLimits) {
                PremiumLimitsView()
            }
            .navigationDestination(isPresented: $showGifts) {
                PremiumGiftsView()
            }
            .navigationDestination(isPresented: $showBoosts) {
                BoostLevelsView()
            }
        }
    }
}

struct PremiumFeatureRow: View {
    let feature: PremiumFeature
    let isEnabled: Bool
    
    var body: some View {
        HStack(spacing: 16) {
            Image(systemName: feature.icon)
                .font(.title2)
                .foregroundColor(isEnabled ? NeoTrixTheme.Colors.accent : .secondary)
                .frame(width: 40)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(feature.name)
                    .font(.headline)
                Text(feature.description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            if isEnabled {
                Image(systemName: "checkmark.circle.fill")
                    .foregroundColor(NeoTrixTheme.Colors.success)
            } else {
                Image(systemName: "lock.fill")
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .background(NeoTrixTheme.Colors.surface)
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct PremiumTierButton: View {
    let tier: PremiumTier
    let isSelected: Bool
    var isPurchasing: Bool = false
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(tier.displayName)
                        .font(.headline)
                    Text(tier.price)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                if isPurchasing {
                    // 修复: 购买 loading 态（此前 purchaseInProgress 无 UI 消费，点击后无反馈）
                    ProgressView()
                } else if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(NeoTrixTheme.Colors.accent)
                }
            }
            .padding()
            .background(isSelected ? NeoTrixTheme.Colors.selection : NeoTrixTheme.Colors.surface)
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(isSelected ? NeoTrixTheme.Colors.accent : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
        .disabled(isPurchasing)
    }
}

// MARK: - Premium Limits Screen

public struct PremiumLimitsView: View {
    @StateObject private var manager = PremiumManager()
    
    public var body: some View {
        List {
            Section("Your Limits") {
                LimitRow(name: "Channels", current: 500, premium: 1000, unit: "channels")
                LimitRow(name: "Folders", current: 10, premium: 20, unit: "folders")
                LimitRow(name: "Pinned Chats", current: 5, premium: 10, unit: "chats")
                LimitRow(name: "Saved Messages", current: 100, premium: 200, unit: "items")
                LimitRow(name: "File Upload", current: 2, premium: 4, unit: "GB")
            }
            
            Section("Premium Benefits") {
                ForEach(manager.features.filter { manager.isFeatureEnabled($0.id) }) { feature in
                    HStack {
                        Image(systemName: feature.icon)
                            .foregroundColor(NeoTrixTheme.Colors.accent)
                        Text(feature.name)
                        Spacer()
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundColor(NeoTrixTheme.Colors.success)
                    }
                }
            }
        }
        .navigationTitle("Limits")
    }
}

struct LimitRow: View {
    let name: String
    let current: Double
    let premium: Double
    let unit: String
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(name)
                .font(.headline)
            
            HStack {
                VStack(alignment: .leading) {
                    Text("Free: \(format(current)) \(unit)")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                    Text("Premium: \(format(premium)) \(unit)")
                        .font(.subheadline)
                        .foregroundColor(NeoTrixTheme.Colors.accent)
                }
                
                Spacer()
                
                ProgressView(value: current, total: premium)
                    .frame(width: 100)
            }
        }
        .padding(.vertical, 4)
    }
    
    private func format(_ value: Double) -> String {
        value.truncatingRemainder(dividingBy: 1) == 0
            ? String(Int(value))
            : String(format: "%.1f", value)
    }
}

// MARK: - Premium Gifts

public struct PremiumGiftsView: View {
    @StateObject private var manager = PremiumManager()
    @State private var giftMessage: String?
    
    public var body: some View {
        List {
            Section("Send Premium Gift") {
                ForEach(PremiumTier.allCases.filter { $0 != .free }) { tier in
                    HStack {
                        VStack(alignment: .leading) {
                            Text(tier.displayName)
                            Text(tier.price)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        Spacer()
                        Button("Gift") {
                            // 修复: 此前空操作死按钮 → 确认反馈（真实 gifting 需联系人选择）
                            giftMessage = tier.displayName
                        }
                        .buttonStyle(.borderedProminent)
                    }
                }
            }
            
            Section("Received Gifts") {
                Text("No gifts received yet")
                    .foregroundColor(.secondary)
            }
        }
        .navigationTitle("Premium Gifts")
        .alert("Gift \(giftMessage ?? "")", isPresented: Binding(
            get: { giftMessage != nil },
            set: { if !$0 { giftMessage = nil } }
        )) {
            Button("OK", role: .cancel) { giftMessage = nil }
        } message: {
            Text("Select a contact to send this gift. Contact picker coming soon.")
        }
    }
}

// MARK: - Boost Levels

public struct BoostLevelsView: View {
    public var body: some View {
        List {
            Section("Channel Boost Levels") {
                ForEach(1...10, id: \.self) { level in
                    HStack {
                        Text("Level \(level)")
                        Spacer()
                        Text("\(level * 10)% perks")
                            .foregroundColor(.secondary)
                    }
                }
            }
            
            Section("Your Boosts") {
                Text("No active boosts")
                    .foregroundColor(.secondary)
            }
        }
        .navigationTitle("Boost Levels")
    }
}