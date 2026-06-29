import SwiftUI

@main
struct NeoTrixApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @State private var selectedTab = 0

    var body: some View {
        TabView(selection: $selectedTab) {
            NavigationStack {
                ChatView()
            }
            .tabItem {
                Label("Chat", systemImage: "message.fill")
            }
            .tag(0)

            NavigationStack {
                MomentView()
            }
            .tabItem {
                Label("Moments", systemImage: "rectangle.3.offgrid.bubble.left.fill")
            }
            .tag(1)

            NavigationStack {
                UnifiedStreamView()
            }
            .tabItem {
                Label("Stream", systemImage: "waveform.path.ecg")
            }
            .tag(2)

            NavigationStack {
                SettingsView()
            }
            .tabItem {
                Label("Settings", systemImage: "gearshape.fill")
            }
            .tag(3)
        }
        .tint(.blue)
    }
}
