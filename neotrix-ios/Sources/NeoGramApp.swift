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
    
    public enum Tab {
        case chats
        case contacts
        case calls
        case settings
    }
    
    public var body: some View {
        TabView(selection: $selectedTab) {
            ChatListView()
                .tabItem {
                    Label("Chats", systemImage: "bubble.left.and.bubble.right.fill")
                }
                .tag(Tab.chats)
            
            ContactsView()
                .tabItem {
                    Label("Contacts", systemImage: "person.2.fill")
                }
                .tag(Tab.contacts)
            
            CallsView()
                .tabItem {
                    Label("Calls", systemImage: "phone.fill")
                }
                .tag(Tab.calls)
            
            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gearshape.fill")
                }
                .tag(Tab.settings)
        }
    }
}

// MARK: - Placeholder Views

public struct ContactsView: View {
    public init() {}
    
    public var body: some View {
        NavigationStack {
            List {
                Section("Contacts") {
                    ForEach(0..<10, id: \.self) { index in
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
                }
            }
            .navigationTitle("Contacts")
        }
    }
}

public struct CallsView: View {
    public init() {}
    
    public var body: some View {
        NavigationStack {
            List {
                Section("Recent") {
                    ForEach(0..<5, id: \.self) { index in
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
                }
            }
            .navigationTitle("Calls")
        }
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