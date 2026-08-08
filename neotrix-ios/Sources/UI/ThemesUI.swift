// ThemesUI - Telegram themes & wallpapers (Premium)
// Mirrors Telegram's ThemeController + WallpaperController

import SwiftUI

// MARK: - Theme Model

public struct AppTheme: Identifiable {
    public let id: String
    public let name: String
    public let isPremium: Bool
    public let colors: ThemeColors
}

public struct ThemeColors {
    public let primary: Color
    public let secondary: Color
    public let background: Color
    public let chatBackground: Color
    public let bubbleIncoming: Color
    public let bubbleOutgoing: Color
    public let text: Color
    public let accent: Color
}

// MARK: - Theme Data

public struct ThemeData {
    public static let themes: [AppTheme] = [
        AppTheme(id: "day", name: "Day", isPremium: false, colors: ThemeColors(
            primary: .blue, secondary: .gray, background: .white,
            chatBackground: Color(red: 0.95, green: 0.95, blue: 0.97),
            bubbleIncoming: .white, bubbleOutgoing: .blue,
            text: .black, accentText: .white
        )),
        AppTheme(id: "night", name: "Night", isPremium: false, colors: ThemeColors(
            primary: .blue, secondary: .gray, background: .black,
            chatBackground: Color(red: 0.08, green: 0.08, blue: 0.1),
            bubbleIncoming: Color(red: 0.15, green: 0.15, blue: 0.17),
            bubbleOutgoing: .blue, text: .white, accentText: .white
        )),
        AppTheme(id: "ocean", name: "Ocean", isPremium: true, colors: ThemeColors(
            primary: .cyan, secondary: .teal, background: Color(red: 0.02, green: 0.1, blue: 0.2),
            chatBackground: Color(red: 0.03, green: 0.12, blue: 0.22),
            bubbleIncoming: Color(red: 0.05, green: 0.15, blue: 0.25),
            bubbleOutgoing: .cyan, text: .white, accentText: .black
        )),
        AppTheme(id: "sunset", name: "Sunset", isPremium: true, colors: ThemeColors(
            primary: .orange, secondary: .pink, background: Color(red: 0.2, green: 0.05, blue: 0.1),
            chatBackground: Color(red: 0.22, green: 0.06, blue: 0.12),
            bubbleIncoming: Color(red: 0.25, green: 0.08, blue: 0.14),
            bubbleOutgoing: .orange, text: .white, accentText: .white
        )),
        AppTheme(id: "forest", name: "Forest", isPremium: true, colors: ThemeColors(
            primary: .green, secondary: .mint, background: Color(red: 0.02, green: 0.15, blue: 0.08),
            chatBackground: Color(red: 0.03, green: 0.17, blue: 0.1),
            bubbleIncoming: Color(red: 0.05, green: 0.2, blue: 0.12),
            bubbleOutgoing: .green, text: .white, accentText: .white
        )),
        AppTheme(id: "midnight", name: "Midnight", isPremium: true, colors: ThemeColors(
            primary: .indigo, secondary: .purple, background: Color(red: 0.05, green: 0.05, blue: 0.15),
            chatBackground: Color(red: 0.07, green: 0.07, blue: 0.17),
            bubbleIncoming: Color(red: 0.1, green: 0.1, blue: 0.2),
            bubbleOutgoing: .indigo, text: .white, accentText: .white
        )),
    ]
}

// MARK: - Wallpaper Model

public struct Wallpaper: Identifiable {
    public let id: String
    public let name: String
    public let isPremium: Bool
    public let isAnimated: Bool
    public let gradient: [Color]
}

public struct WallpaperData {
    public static let wallpapers: [Wallpaper] = [
        Wallpaper(id: "solid_blue", name: "Blue", isPremium: false, isAnimated: false, gradient: [.blue, .blue]),
        Wallpaper(id: "solid_gray", name: "Gray", isPremium: false, isAnimated: false, gradient: [.gray, .gray]),
        Wallpaper(id: "gradient_sunset", name: "Sunset", isPremium: true, isAnimated: false, gradient: [.orange, .pink, .purple]),
        Wallpaper(id: "gradient_ocean", name: "Ocean", isPremium: true, isAnimated: false, gradient: [.cyan, .blue, .indigo]),
        Wallpaper(id: "gradient_forest", name: "Forest", isPremium: true, isAnimated: false, gradient: [.green, .mint, .teal]),
        Wallpaper(id: "animated_aurora", name: "Aurora", isPremium: true, isAnimated: true, gradient: [.purple, .blue, .green]),
        Wallpaper(id: "animated_neon", name: "Neon", isPremium: true, isAnimated: true, gradient: [.pink, .purple, .cyan]),
        Wallpaper(id: "animated_rainbow", name: "Rainbow", isPremium: true, isAnimated: true, gradient: [.red, .orange, .yellow, .green, .blue, .purple]),
    ]
}

// MARK: - Theme Manager

@MainActor
public final class ThemeManager: ObservableObject {
    @Published public var currentTheme: AppTheme
    @Published public var currentWallpaper: Wallpaper?
    @Published public var isPremium = false
    
    private let core = NeoGramCore.shared
    
    public init() {
        self.currentTheme = ThemeData.themes[0]
        loadPreferences()
    }
    
    private func loadPreferences() {
        isPremium = UserDefaults.standard.bool(forKey: "is_premium")
        if let themeId = UserDefaults.standard.string(forKey: "theme_id"),
           let theme = ThemeData.themes.first(where: { $0.id == themeId }) {
            currentTheme = theme
        }
        if let wallpaperId = UserDefaults.standard.string(forKey: "wallpaper_id"),
           let wallpaper = WallpaperData.wallpapers.first(where: { $0.id == wallpaperId }) {
            currentWallpaper = wallpaper
        }
    }
    
    public func applyTheme(_ theme: AppTheme) {
        guard !theme.isPremium || isPremium else { return }
        currentTheme = theme
        UserDefaults.standard.set(theme.id, forKey: "theme_id")
    }
    
    public func applyWallpaper(_ wallpaper: Wallpaper) {
        guard !wallpaper.isPremium || isPremium else { return }
        currentWallpaper = wallpaper
        UserDefaults.standard.set(wallpaper.id, forKey: "wallpaper_id")
    }
    
    public func canUse(_ theme: AppTheme) -> Bool {
        return !theme.isPremium || isPremium
    }
    
    public func canUse(_ wallpaper: Wallpaper) -> Bool {
        return !wallpaper.isPremium || isPremium
    }
}

// MARK: - Theme Settings View

public struct ThemeSettingsView: View {
    @StateObject private var manager = ThemeManager()
    
    public var body: some View {
        List {
            Section("Themes") {
                ForEach(ThemeData.themes) { theme in
                    Button {
                        manager.applyTheme(theme)
                    } label: {
                        HStack {
                            Circle()
                                .fill(theme.colors.primary)
                                .frame(width: 24, height: 24)
                            
                            Text(theme.name)
                                .foregroundColor(.primary)
                            
                            if theme.isPremium {
                                Image(systemName: "star.fill")
                                    .font(.caption)
                                    .foregroundColor(.yellow)
                            }
                            
                            Spacer()
                            
                            if manager.currentTheme.id == theme.id {
                                Image(systemName: "checkmark")
                                    .foregroundColor(.blue)
                            } else if theme.isPremium && !manager.isPremium {
                                Image(systemName: "lock.fill")
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                    .buttonStyle(.plain)
                    .disabled(theme.isPremium && !manager.isPremium)
                }
            }
            
            Section("Wallpapers") {
                ForEach(WallpaperData.wallpapers) { wallpaper in
                    Button {
                        manager.applyWallpaper(wallpaper)
                    } label: {
                        HStack {
                            RoundedRectangle(cornerRadius: 6)
                                .fill(LinearGradient(colors: wallpaper.gradient, startPoint: .topLeading, endPoint: .bottomTrailing))
                                .frame(width: 40, height: 40)
                            
                            Text(wallpaper.name)
                                .foregroundColor(.primary)
                            
                            if wallpaper.isAnimated {
                                Image(systemName: "play.fill")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            
                            if wallpaper.isPremium {
                                Image(systemName: "star.fill")
                                    .font(.caption)
                                    .foregroundColor(.yellow)
                            }
                            
                            Spacer()
                            
                            if manager.currentWallpaper?.id == wallpaper.id {
                                Image(systemName: "checkmark")
                                    .foregroundColor(.blue)
                            } else if wallpaper.isPremium && !manager.isPremium {
                                Image(systemName: "lock.fill")
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                    .buttonStyle(.plain)
                    .disabled(wallpaper.isPremium && !manager.isPremium)
                }
            }
        }
        .navigationTitle("Themes & Wallpapers")
    }
}

// MARK: - Animated Wallpaper Preview

public struct AnimatedWallpaperView: View {
    let wallpaper: Wallpaper
    @State private var animate = false
    
    public var body: some View {
        LinearGradient(
            colors: wallpaper.gradient,
            startPoint: animate ? .topLeading : .bottomTrailing,
            endPoint: animate ? .bottomTrailing : .topLeading
        )
        .ignoresSafeArea()
        .onAppear {
            if wallpaper.isAnimated {
                withAnimation(.linear(duration: 8).repeatForever(autoreverses: true)) {
                    animate = true
                }
            }
        }
    }
}