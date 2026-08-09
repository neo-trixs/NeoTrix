// SettingsUI - Telegram-style settings with Premium integration
// 对标: Telegram SettingsController — Profile 头部 + 逻辑分组（Account/Privacy/Data/Appearance/AI/About）
// 原则: 每个可点项必须有目标视图或明确禁用态（Dark Forest: 无死按钮）

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
    public let isDisabled: Bool   // true = Coming Soon 禁用态（有明确视觉反馈）
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
        // 统一 premium 事实源为 premium_tier（与 PremiumManager 一致）
        if let tierRaw = UserDefaults.standard.string(forKey: "premium_tier") {
            isPremium = tierRaw != "free"
        }
    }
    
    /// 分组对标 Telegram 层级：Account → NeoTrix AI → Appearance → Privacy → Data → About
    public var sections: [SettingsSection] {
        [
            SettingsSection(id: "account", title: "Account", items: [
                SettingsItem(id: "profile", title: "My Profile", icon: "person.crop.circle", iconColor: .blue, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "saved", title: "Saved Messages", icon: "bookmark.fill", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "devices", title: "Devices", icon: "laptopcomputer", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "chat_folders", title: "Chat Folders", icon: "folder.fill", iconColor: .blue, badge: nil, isPremium: false, isDisabled: false),
            ]),
            SettingsSection(id: "ai", title: "NeoTrix AI", items: [
                SettingsItem(id: "ai_editor", title: "AI Editor", icon: "wand.and.stars", iconColor: .purple, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "ai_settings", title: "AI Settings", icon: "brain.head.profile", iconColor: .purple, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "ai_models", title: "AI Models", icon: "cpu.fill", iconColor: .indigo, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "ai_memory", title: "AI Memory", icon: "memorychip.fill", iconColor: .teal, badge: nil, isPremium: false, isDisabled: true),
            ]),
            SettingsSection(id: "appearance", title: "Appearance", items: [
                SettingsItem(id: "theme", title: "Theme", icon: "paintpalette.fill", iconColor: .purple, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "wallpapers", title: "Wallpapers", icon: "photo.on.rectangle", iconColor: .cyan, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "app_icons", title: "App Icons", icon: "app.badge.fill", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
            ]),
            SettingsSection(id: "privacy", title: "Privacy & Security", items: [
                SettingsItem(id: "passcode", title: "Passcode & Face ID", icon: "faceid", iconColor: .green, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "privacy", title: "Privacy", icon: "lock.fill", iconColor: .blue, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "two_step", title: "Two-Step Verification", icon: "key.fill", iconColor: .orange, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "sessions", title: "Active Sessions", icon: "iphone", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
            ]),
            SettingsSection(id: "content", title: "Content & Filters", items: [
                SettingsItem(id: "filters", title: "Message Filter", icon: "line.3.horizontal.decrease.circle.fill", iconColor: .orange, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "folders", title: "Smart Folders", icon: "folder.fill.badge.gearshape", iconColor: .blue, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "polls", title: "Polls", icon: "chart.bar.xaxis", iconColor: .pink, badge: nil, isPremium: false, isDisabled: false),
                SettingsItem(id: "voice", title: "Voice to Text", icon: "waveform", iconColor: .teal, badge: nil, isPremium: false, isDisabled: false),
            ]),
            SettingsSection(id: "premium", title: "Premium", items: [
                SettingsItem(id: "premium", title: "NeoGram Premium", icon: "star.circle.fill", iconColor: .yellow, badge: isPremium ? "Active" : nil, isPremium: true, isDisabled: false),
                SettingsItem(id: "gifts", title: "Premium Gifts", icon: "gift.fill", iconColor: .pink, badge: nil, isPremium: true, isDisabled: true),
                SettingsItem(id: "boosts", title: "Boost Levels", icon: "bolt.fill", iconColor: .orange, badge: nil, isPremium: true, isDisabled: true),
            ]),
            SettingsSection(id: "data", title: "Data & Storage", items: [
                SettingsItem(id: "storage", title: "Storage Usage", icon: "internaldrive.fill", iconColor: .gray, badge: storageUsed, isPremium: false, isDisabled: true),
                SettingsItem(id: "data", title: "Data & Storage", icon: "chart.bar.fill", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "proxy", title: "Proxy", icon: "network", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
            ]),
            SettingsSection(id: "about", title: "About", items: [
                SettingsItem(id: "language", title: "Language", icon: "globe", iconColor: .blue, badge: "English", isPremium: false, isDisabled: true),
                SettingsItem(id: "help", title: "Help", icon: "questionmark.circle.fill", iconColor: .blue, badge: nil, isPremium: false, isDisabled: true),
                SettingsItem(id: "about", title: "About NeoGram", icon: "info.circle.fill", iconColor: .blue, badge: "v1.0", isPremium: false, isDisabled: true),
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
                // Profile header（对标 Telegram: 头像 + 姓名 + 号码 → 个人页）
                Section {
                    Button {
                        showProfile = true
                    } label: {
                        HStack(spacing: 16) {
                            NeoTrixAvatar(title: viewModel.username, size: 60)
                            
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
            .navigationDestination(isPresented: $showProfile) {
                ProfileView(viewModel: viewModel)
            }
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
                // AI 编辑器（传入当前对话上下文；空原文 = 新建草稿）
                AIEditorView(original: "") { _ in }
            }
            .sheet(isPresented: $showPremium) {
                PremiumIntroView()
            }
        }
    }
    
    private func handleItemTap(_ item: SettingsItem) {
        // Coming Soon 项: 不响应（禁用态已有视觉反馈）
        guard !item.isDisabled else { return }
        switch item.id {
        case "premium": showPremium = true
        case "filters": showFilters = true
        case "privacy": showPrivacy = true
        case "folders": showFolders = true
        case "chat_folders": showFolders = true
        case "polls": showPolls = true
        case "voice": showVoiceToText = true
        case "theme": showTheme = true
        case "wallpapers": showTheme = true
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
                    .background(item.isDisabled
                        ? AnyShapeStyle(NeoTrixTheme.Colors.surface)
                        : AnyShapeStyle(item.iconColor.gradient))
                    .clipShape(RoundedRectangle(cornerRadius: 7))
                    .opacity(item.isDisabled ? 0.6 : 1)
                
                Text(item.title)
                    .foregroundColor(.primary)
                    .opacity(item.isDisabled ? 0.6 : 1)
                
                Spacer()
                
                if let badge = item.badge {
                    Text(badge)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                if item.isPremium {
                    Image(systemName: "star.fill")
                        .font(.caption)
                        .foregroundColor(NeoTrixTheme.Colors.premium)
                }
                
                if item.isDisabled {
                    Text("Soon")
                        .font(.caption2)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(NeoTrixTheme.Colors.surface)
                        .foregroundColor(.secondary)
                        .clipShape(Capsule())
                } else {
                    Image(systemName: "chevron.right")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            .padding(.vertical, 2)
        }
        .buttonStyle(.plain)
        .disabled(item.isDisabled)
    }
}

// MARK: - Profile 页（对标 Telegram: 头像 + 姓名/号码编辑 + About + 共享媒体入口）

struct ProfileView: View {
    @ObservedObject var viewModel: SettingsViewModel
    @State private var about = ""
    @State private var isEditingAbout = false
    @State private var showAvatarMenu = false
    
    var body: some View {
        List {
            // 头部: 大号头像 + 姓名 + 号码
            Section {
                HStack(spacing: 16) {
                    NeoTrixAvatar(title: viewModel.username, size: 72)
                    
                    VStack(alignment: .leading, spacing: 4) {
                        Text(viewModel.username)
                            .font(.title2.bold())
                            .foregroundColor(.primary)
                        Text(viewModel.phoneNumber)
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                        if !about.isEmpty {
                            Text(about)
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    
                    Spacer()
                    
                    Button {
                        // 头像编辑（对标 Telegram: 拍照/选图/删除照片）
                        showAvatarMenu = true
                    } label: {
                        Image(systemName: "camera.fill")
                            .font(.subheadline)
                            .foregroundColor(NeoTrixTheme.Colors.accent)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.vertical, 8)
            }
            
            // About（对标 Telegram: 编辑简介）
            Section("Info") {
                Button {
                    isEditingAbout = true
                } label: {
                    HStack {
                        Text("About")
                            .foregroundColor(.primary)
                        Spacer()
                        Text(about.isEmpty ? "Add bio" : about)
                            .foregroundColor(about.isEmpty ? .secondary : .primary)
                        Image(systemName: "chevron.right")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }
            
            // 共享媒体/链接（对标 Telegram: Shared Media / Shared Links）
            Section {
                HStack {
                    Label("Shared Media", systemImage: "photo.on.rectangle")
                        .foregroundColor(.primary)
                    Spacer()
                    disabledBadge
                }
                HStack {
                    Label("Shared Links", systemImage: "link")
                        .foregroundColor(.primary)
                    Spacer()
                    disabledBadge
                }
            }
            
            // 操作（对标 Telegram: 通知/通话/星标）
            Section {
                HStack {
                    Label("Notifications", systemImage: "bell")
                        .foregroundColor(.primary)
                    Spacer()
                    disabledBadge
                }
                HStack {
                    Label("Start Secret Chat", systemImage: "lock.fill")
                        .foregroundColor(.primary)
                    Spacer()
                    disabledBadge
                }
            }
        }
        .navigationTitle("Profile")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
        #if os(iOS)
        .alert("Edit About", isPresented: $isEditingAbout) {
            TextField("About", text: $about)
            Button("Save") {
                // 修复: 此前 Save 空操作，About 永不保存
                UserDefaults.standard.set(about, forKey: "profile_about")
            }
            Button("Cancel", role: .cancel) {}
        }
        #else
        .sheet(isPresented: $isEditingAbout) {
            VStack(spacing: 16) {
                Text("Edit About")
                    .font(.headline)
                TextField("About", text: $about)
                    .textFieldStyle(.roundedBorder)
                    .padding(.horizontal)
                Button("Save") {
                    UserDefaults.standard.set(about, forKey: "profile_about")
                    isEditingAbout = false
                }
                .buttonStyle(.borderedProminent)
            }
            .padding()
            .frame(width: 300)
        }
        #endif
        .confirmationDialog("Change Avatar", isPresented: $showAvatarMenu, titleVisibility: .visible) {
            Button("Take Photo") {}
            Button("Choose from Library") {}
            Button("Remove Photo", role: .destructive) {}
            Button("Cancel", role: .cancel) {}
        }
        .onAppear {
            // 加载已保存的 About（对标 Telegram: 简介持久化）
            if let saved = UserDefaults.standard.string(forKey: "profile_about") {
                about = saved
            }
        }
    }
    
    /// 禁用态徽标（对标 Telegram: 未实现项置灰 + "Soon"）
    private var disabledBadge: some View {
        Text("Soon")
            .font(.caption2)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(NeoTrixTheme.Colors.surface)
            .foregroundColor(.secondary)
            .clipShape(Capsule())
    }
}
