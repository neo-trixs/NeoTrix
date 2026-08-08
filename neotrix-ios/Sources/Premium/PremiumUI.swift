// PremiumUI - Telegram Premium features implementation
// Mirrors Telegram's PremiumUI module

import SwiftUI
import StoreKit

// MARK: - Premium Status

public enum PremiumTier: String, CaseIterable {
    case free = "Free"
    case monthly = "Monthly"
    case yearly = "Yearly"
    case biannual = "Biannual"
    
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
        // Implement receipt validation
    }
    
    public func isFeatureEnabled(_ featureId: String) -> Bool {
        guard let feature = features.first(where: { $0.id == featureId }) else { return false }
        return isPremium && currentTier.rawValue >= feature.tier.rawValue
    }
}

// MARK: - Premium Intro Screen

public struct PremiumIntroView: View {
    @StateObject private var manager = PremiumManager()
    @Environment(\.dismiss) private var dismiss
    
    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Hero
                    VStack(spacing: 16) {
                        Image(systemName: "star.circle.fill")
                            .font(.system(size: 80))
                            .foregroundStyle(.yellow.gradient)
                        
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
                            PremiumTierButton(tier: tier, isSelected: manager.currentTier == tier) {
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
                    
                    Spacer(minLength: 40)
                }
            }
            .navigationTitle("Premium")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") { dismiss() }
                }
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
                .foregroundColor(isEnabled ? .blue : .secondary)
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
                    .foregroundColor(.green)
            } else {
                Image(systemName: "lock.fill")
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .background(Color(.systemGray6))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

struct PremiumTierButton: View {
    let tier: PremiumTier
    let isSelected: Bool
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
                
                if isSelected {
                    Image(systemName: "checkmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(.blue)
                }
            }
            .padding()
            .background(isSelected ? Color.blue.opacity(0.1) : Color(.systemGray6))
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .overlay(
                RoundedRectangle(cornerRadius: 12)
                    .stroke(isSelected ? Color.blue : Color.clear, lineWidth: 2)
            )
        }
        .buttonStyle(.plain)
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
                LimitRow(name: "File Upload", current: "2 GB", premium: "4 GB", unit: "")
            }
            
            Section("Premium Benefits") {
                ForEach(manager.features.filter { manager.isFeatureEnabled($0.id) }) { feature in
                    HStack {
                        Image(systemName: feature.icon)
                            .foregroundColor(.blue)
                        Text(feature.name)
                        Spacer()
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundColor(.green)
                    }
                }
            }
        }
        .navigationTitle("Limits")
    }
}

struct LimitRow: View {
    let name: String
    let current: Any
    let premium: Any
    let unit: String
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(name)
                .font(.headline)
            
            HStack {
                VStack(alignment: .leading) {
                    Text("Free: \(current) \(unit)")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                    Text("Premium: \(premium) \(unit)")
                        .font(.subheadline)
                        .foregroundColor(.blue)
                }
                
                Spacer()
                
                ProgressView(value: Double("\(current)".replacingOccurrences(of: " GB", with: "")) ?? 0, 
                           total: Double("\(premium)".replacingOccurrences(of: " GB", with: "")) ?? 1)
                    .frame(width: 100)
            }
        }
        .padding(.vertical, 4)
    }
}

// MARK: - Premium Gifts

public struct PremiumGiftsView: View {
    @StateObject private var manager = PremiumManager()
    
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
                            // Implement gifting
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