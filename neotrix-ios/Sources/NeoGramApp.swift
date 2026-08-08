// NeoGramApp - Main app entry point
// Telegram-style app with NeoTrix AI integration

import SwiftUI

@main
public struct NeoGramApp: App {
    @StateObject private var core = NeoGramCore.shared
    @StateObject private var passcodeManager = PasscodeManager()
    @State private var isInitialized = false
    @State private var showPasscode = false
    
    public init() {}
    
    public var body: some Scene {
        WindowGroup {
            Group {
                if isInitialized {
                    if showPasscode {
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
        }
    }
    
    private func initialize() async {
        do {
            try await core.initialize()
            isInitialized = true
            showPasscode = passcodeManager.isPasscodeEnabled
        } catch {
            // Show error state
            print("Failed to initialize: \(error)")
        }
    }
}

// MARK: - Main Tab View

public struct MainTabView: View {
    @State private var selectedTab: Tab = .chats
    @StateObject private var themeManager = ThemeManager()
    
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
                .tag(Tab.chats)
            
            // Tab 2: Live（发现中枢）— LiveFeed 瀑布流 + 搜索 + 类型角标
            LiveFeedView()
                .tabItem {
                    Label("Live", systemImage: "rectangle.3.offgrid.bubble.left.fill")
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
                    .fill(
                        LinearGradient(
                            colors: [Color.blue, Color.purple],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 96, height: 96)
                
                Image(systemName: "paperplane.fill")
                    .font(.system(size: 40, weight: .semibold))
                    .foregroundColor(.white)
            }
            
            Text("NeoGram")
                .font(.title2.bold())
            
            ProgressView()
                .tint(.blue)
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