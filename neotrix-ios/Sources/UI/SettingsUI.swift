// SettingsUI - Telegram-style settings with Premium integration
// Mirrors Telegram's SettingsController + SettingsSections

import SwiftUI

// MARK: - Settings Model

public struct SettingsSection: Identifiable {
    public let id: String
    public let title: String
    public let items: [SettingsItem]
}

public struct SettingsItem: Identifiable {
    public let id: String
    public let title: String
    public let icon: String
    public let iconColor: Color
    public let badge: String?
    public let isPremium: Bool
}

// MARK: - Settings View Model

@MainActor
public final class SettingsViewModel: ObservableObject {
    @Published public var isPremium = false
    @Published public var username = "user"
    @Published public var phoneNumber = "+1 (555) 000-0000"
    @Published public var storageUsed = "0 MB"
    
    private let core = NeoGramCore.shared
    
    public init() {
        loadUserData()
    }
    
    private func loadUserData() {
        // Load from MTProto account info
        if let savedUsername = UserDefaults.standard.string(forKey: "username") {
            username = savedUsername
        }
        if let savedPhone = UserDefaults.standard.string(forKey: "phone") {
            phoneNumber = savedPhone
        }
        isPremium = UserDefaults.standard.bool(forKey: "is_premium")
    }
    
    public var sections: [SettingsSection] {
        [
            SettingsSection(id: "account", title: "Account", items: [
                SettingsItem(id: "profile", title: "My Profile", icon: "person.crop.circle", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "saved", title: "Saved Messages", icon: "bookmark.fill", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "recent", title: "Recent Calls", icon: "phone.fill", iconColor: .green, badge: nil, isPremium: false),
                SettingsItem(id: "devices", title: "Devices", icon: "laptopcomputer", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "chat_folders", title: "Chat Folders", icon: "folder.fill", iconColor: .blue, badge: nil, isPremium: false),
            ]),
            SettingsSection(id: "premium", title: "Premium", items: [
                SettingsItem(id: "premium", title: "NeoGram Premium", icon: "star.circle.fill", iconColor: .yellow, badge: isPremium ? "Active" : nil, isPremium: true),
                SettingsItem(id: "gifts", title: "Premium Gifts", icon: "gift.fill", iconColor: .pink, badge: nil, isPremium: true),
                SettingsItem(id: "boosts", title: "Boost Levels", icon: "bolt.fill", iconColor: .orange, badge: nil, isPremium: true),
            ]),
            SettingsSection(id: "ai", title: "NeoTrix AI", items: [
                SettingsItem(id: "ai_settings", title: "AI Settings", icon: "brain.head.profile", iconColor: .purple, badge: nil, isPremium: false),
                SettingsItem(id: "ai_models", title: "AI Models", icon: "cpu.fill", iconColor: .indigo, badge: nil, isPremium: false),
                SettingsItem(id: "ai_memory", title: "AI Memory", icon: "memorychip.fill", iconColor: .teal, badge: nil, isPremium: false),
                SettingsItem(id: "ai_editor", title: "AI Editor", icon: "wand.and.stars", iconColor: .purple, badge: nil, isPremium: false),
                SettingsItem(id: "ai_export", title: "Export to AI", icon: "square.and.arrow.up", iconColor: .indigo, badge: nil, isPremium: false),
            ]),
            SettingsSection(id: "fusion", title: "Fusion Features", items: [
                SettingsItem(id: "filters", title: "Message Filter", icon: "line.3.horizontal.decrease.circle.fill", iconColor: .orange, badge: nil, isPremium: false),
                SettingsItem(id: "privacy", title: "Privacy & Security", icon: "lock.shield.fill", iconColor: .green, badge: nil, isPremium: false),
                SettingsItem(id: "folders", title: "Smart Folders", icon: "folder.fill.badge.gearshape", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "polls", title: "Polls", icon: "chart.bar.xaxis", iconColor: .pink, badge: nil, isPremium: false),
                SettingsItem(id: "voice", title: "Voice to Text", icon: "waveform", iconColor: .teal, badge: nil, isPremium: false),
            ]),
            SettingsSection(id: "appearance", title: "Appearance", items: [
                SettingsItem(id: "theme", title: "Theme", icon: "paintpalette.fill", iconColor: .purple, badge: nil, isPremium: false),
                SettingsItem(id: "wallpapers", title: "Wallpapers", icon: "photo.on.rectangle", iconColor: .cyan, badge: nil, isPremium: false),
                SettingsItem(id: "app_icons", title: "App Icons", icon: "app.badge.fill", iconColor: .blue, badge: nil, isPremium: false),
            ]),
            SettingsSection(id: "privacy", title: "Privacy & Security", items: [
                SettingsItem(id: "privacy", title: "Privacy", icon: "lock.fill", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "passcode", title: "Passcode & Face ID", icon: "faceid", iconColor: .green, badge: nil, isPremium: false),
                SettingsItem(id: "two_step", title: "Two-Step Verification", icon: "key.fill", iconColor: .orange, badge: nil, isPremium: false),
                SettingsItem(id: "sessions", title: "Active Sessions", icon: "iphone", iconColor: .blue, badge: nil, isPremium: false),
            ]),
            SettingsSection(id: "data", title: "Data & Storage", items: [
                SettingsItem(id: "storage", title: "Storage Usage", icon: "internaldrive.fill", iconColor: .gray, badge: storageUsed, isPremium: false),
                SettingsItem(id: "data", title: "Data & Storage", icon: "chart.bar.fill", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "proxy", title: "Proxy", icon: "network", iconColor: .blue, badge: nil, isPremium: false),
            ]),
            SettingsSection(id: "language", title: "Language", items: [
                SettingsItem(id: "language", title: "Language", icon: "globe", iconColor: .blue, badge: "English", isPremium: false),
            ]),
            SettingsSection(id: "about", title: "About", items: [
                SettingsItem(id: "help", title: "Help", icon: "questionmark.circle.fill", iconColor: .blue, badge: nil, isPremium: false),
                SettingsItem(id: "about", title: "About NeoGram", icon: "info.circle.fill", iconColor: .blue, badge: "v1.0", isPremium: false),
            ]),
        ]
    }
}

// MARK: - Settings View

public struct SettingsView: View {
    @StateObject private var viewModel = SettingsViewModel()
    @State private var showPremium = false
    @State private var showProfile = false
    @State private var showFilters = false
    @State private var showPrivacy = false
    @State private var showFolders = false
    @State private var showPolls = false
    @State private var showVoiceToText = false
    @State private var showTheme = false
    @State private var showPasscode = false
    @State private var showAIEditor = false
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            List {
                // Profile header
                Section {
                    Button {
                        showProfile = true
                    } label: {
                        HStack(spacing: 16) {
                            ZStack {
                                Circle()
                                    .fill(Color.blue.gradient)
                                    .frame(width: 60, height: 60)
                                Text(String(viewModel.username.prefix(1)).uppercased())
                                    .font(.title.bold())
                                    .foregroundColor(.white)
                            }
                            
                            VStack(alignment: .leading, spacing: 4) {
                                Text(viewModel.username)
                                    .font(.title3.bold())
                                    .foregroundColor(.primary)
                                Text(viewModel.phoneNumber)
                                    .font(.subheadline)
                                    .foregroundColor(.secondary)
                            }
                            
                            Spacer()
                            
                            Image(systemName: "chevron.right")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                        .padding(.vertical, 8)
                    }
                    .buttonStyle(.plain)
                }
                
                // Sections
                ForEach(viewModel.sections) { section in
                    Section(section.title) {
                        ForEach(section.items) { item in
                            SettingsRow(item: item) {
                                handleItemTap(item)
                            }
                        }
                    }
                }
            }
            .navigationTitle("Me")
            .navigationDestination(isPresented: $showFilters) {
                FilterSettingsView()
            }
            .navigationDestination(isPresented: $showPrivacy) {
                PrivacySettingsView()
            }
            .navigationDestination(isPresented: $showFolders) {
                FolderManagementView()
            }
            .navigationDestination(isPresented: $showPolls) {
                PollView()
            }
            .navigationDestination(isPresented: $showVoiceToText) {
                VoiceToTextView()
            }
            .navigationDestination(isPresented: $showTheme) {
                ThemeSettingsView()
            }
            .navigationDestination(isPresented: $showPasscode) {
                PasscodeSettingsView()
            }
            .navigationDestination(isPresented: $showAIEditor) {
                AIEditorView(original: "") { _ in }
            }
            .sheet(isPresented: $showPremium) {
                PremiumIntroView()
            }
        }
    }
    
    private func handleItemTap(_ item: SettingsItem) {
        switch item.id {
        case "premium": showPremium = true
        case "filters": showFilters = true
        case "privacy": showPrivacy = true
        case "folders": showFolders = true
        case "polls": showPolls = true
        case "voice": showVoiceToText = true
        case "theme": showTheme = true
        case "passcode": showPasscode = true
        case "ai_editor": showAIEditor = true
        default: break
        }
    }
}

struct SettingsRow: View {
    let item: SettingsItem
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 12) {
                Image(systemName: item.icon)
                    .font(.body)
                    .foregroundColor(.white)
                    .frame(width: 28, height: 28)
                    .background(item.iconColor.gradient)
                    .clipShape(RoundedRectangle(cornerRadius: 7))
                
                Text(item.title)
                    .foregroundColor(.primary)
                
                Spacer()
                
                if let badge = item.badge {
                    Text(badge)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                if item.isPremium {
                    Image(systemName: "star.fill")
                        .font(.caption)
                        .foregroundColor(.yellow)
                }
                
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(.vertical, 2)
        }
        .buttonStyle(.plain)
    }
}