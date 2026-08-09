// NeoGramApp - Main app entry point
// Telegram-style app with NeoTrix AI integration

import SwiftUI

@main
public struct NeoGramApp: App {
    @StateObject private var core = NeoGramCore.shared
    @StateObject private var passcodeManager = PasscodeManager()
    @State private var isInitialized = false
    @State private var initFailed = false
    
    public init() {}
    
    public var body: some Scene {
        WindowGroup {
            Group {
                if initFailed {
                    // 初始化失败错误态（对标主流产品: 可重试，不卡死转圈）
                    InitErrorView { retry in
                        Task { await initialize() }
                    }
                } else if isInitialized {
                    if passcodeManager.isLocked {
                        PasscodeLockView()
                    } else {
                        MainTabView()
                    }
                } else {
                    LaunchView()
                        .task {
                            await initialize()
                        }
                }
            }
            .preferredColorScheme(.dark)
            .environmentObject(passcodeManager)
        }
    }
    
    private func initialize() async {
        do {
            try await core.initialize()
            isInitialized = true
            // Passcode 已启用 → 启动即锁定（对标 Telegram: 冷启动要求输入）
            if passcodeManager.isPasscodeEnabled {
                passcodeManager.lock()
            }
        } catch {
            // 错误态：展示重试（修复: 此前仅 print 且卡死在 LaunchView 转圈）
            print("Failed to initialize: \(error)")
            initFailed = true
        }
    }
}

// MARK: - 初始化错误态（修复: 此前初始化失败永久卡 LaunchView）

struct InitErrorView: View {
    let onRetry: () -> Void
    
    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "exclamationmark.triangle.fill")
                .font(.system(size: 48))
                .foregroundColor(NeoTrixTheme.Colors.danger)
            
            Text("Failed to Initialize")
                .font(.headline)
            
            Text("The NeoTrix core could not be started. Check your network and try again.")
                .font(.caption)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
            
            Button(action: onRetry) {
                Text("Retry")
                    .font(.subheadline.bold())
                    .padding(.horizontal, 32)
                    .padding(.vertical, 10)
                    .background(NeoTrixTheme.Colors.accent)
                    .foregroundColor(.white)
                    .clipShape(Capsule())
            }
            .buttonStyle(.plain)
        }
        .preferredColorScheme(.dark)
    }
}

// MARK: - Main Tab View

public struct MainTabView: View {
    @State private var selectedTab: Tab = .chats
    @StateObject private var themeManager = ThemeManager()
    @StateObject private var chatListVM = ChatListViewModel()
    
    public enum Tab {
        case chats
        case live
        case me
    }
    
    public var body: some View {
        TabView(selection: $selectedTab) {
            // Tab 1: Chats（对话中枢）— Stories 环 + 聊天列表 + 联系人/通话融合 + AI 入口
            ChatListView()
                .tabItem {
                    Label("Chats", systemImage: "bubble.left.and.bubble.right.fill")
                }
                .badge(chatListVM.chats.reduce(0) { $0 + $1.unreadCount })
                .tag(Tab.chats)
            
            // Tab 2: Live（发现中枢）— LiveFeed 瀑布流 + 搜索 + 类型角标
            LiveFeedView()
                .tabItem {
                    Label("Live", systemImage: "sparkles.rectangle.stack.fill")
                }
                .tag(Tab.live)
            
            // Tab 3: Me（个人中枢）— 设置 + Premium + 主题 + 隐私 + AI 入口
            SettingsView()
                .tabItem {
                    Label("Me", systemImage: "person.crop.circle.fill")
                }
                .tag(Tab.me)
        }
        // 统一设计系统：极简风格 + 主题联动（Premium 主题切换 → Tab 强调色）
        .tint(themeManager.currentTheme.colors.primary)
        .environmentObject(themeManager)
    }
}

// MARK: - Launch View

public struct LaunchView: View {
    public init() {}
    
    public var body: some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .fill(NeoTrixTheme.Gradients.brand)
                    .frame(width: 96, height: 96)
                
                Image(systemName: "paperplane.fill")
                    .font(.system(size: 40, weight: .semibold))
                    .foregroundColor(.white)
            }
            
            Text("NeoGram")
                .font(.title2.bold())
            
            ProgressView()
                .tint(NeoTrixTheme.Colors.accent)
        }
        .preferredColorScheme(.dark)
    }
}

// MARK: - Content View

public struct ContentView: View {
    public init() {}
    
    public var body: some View {
        MainTabView()
    }
}