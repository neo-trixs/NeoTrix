// AnimatedEmojiUI - Telegram animated emoji (Premium feature)
// Mirrors Telegram's AnimatedEmojiNode + emoji effects

import SwiftUI

// MARK: - Animated Emoji Model

public struct AnimatedEmoji: Identifiable {
    public let id: String
    public let emoji: String
    public let effect: Effect
    public let isPremium: Bool
    
    public enum Effect {
        case none
        case bounce
        case explode
        case confetti
        case firework
        case heart
        case star
        case rainbow
        case shake
        case pulse
    }
}

// MARK: - Animated Emoji Data

public struct AnimatedEmojiData {
    public static let premiumEmojis: [AnimatedEmoji] = [
        AnimatedEmoji(id: "fire", emoji: "🔥", effect: .firework, isPremium: true),
        AnimatedEmoji(id: "heart", emoji: "❤️", effect: .heart, isPremium: true),
        AnimatedEmoji(id: "star", emoji: "⭐️", effect: .star, isPremium: true),
        AnimatedEmoji(id: "rainbow", emoji: "🌈", effect: .rainbow, isPremium: true),
        AnimatedEmoji(id: "party", emoji: "🎉", effect: .confetti, isPremium: true),
        AnimatedEmoji(id: "clap", emoji: "👏", effect: .shake, isPremium: true),
        AnimatedEmoji(id: "love", emoji: "😍", effect: .pulse, isPremium: true),
        AnimatedEmoji(id: "cool", emoji: "😎", effect: .none, isPremium: true),
        AnimatedEmoji(id: "thinking", emoji: "🤔", effect: .none, isPremium: true),
        AnimatedEmoji(id: "wow", emoji: "😮", effect: .none, isPremium: true),
    ]
    
    public static let freeEmojis: [AnimatedEmoji] = [
        AnimatedEmoji(id: "smile", emoji: "😀", effect: .none, isPremium: false),
        AnimatedEmoji(id: "laugh", emoji: "😂", effect: .none, isPremium: false),
        AnimatedEmoji(id: "wink", emoji: "😉", effect: .none, isPremium: false),
        AnimatedEmoji(id: "ok", emoji: "👍", effect: .none, isPremium: false),
    ]
    
    public static var all: [AnimatedEmoji] {
        freeEmojis + premiumEmojis
    }
}

// MARK: - Animated Emoji View

public struct AnimatedEmojiView: View {
    let emoji: AnimatedEmoji
    @State private var animate = false
    
    public init(emoji: AnimatedEmoji) {
        self.emoji = emoji
    }
    
    public var body: some View {
        Text(emoji.emoji)
            .font(.system(size: 60))
            .scaleEffect(scale)
            .rotationEffect(rotation)
            .offset(y: offset)
            .onAppear {
                animate = true
            }
    }
    
    private var scale: CGFloat {
        guard animate else { return 1.0 }
        switch emoji.effect {
        case .pulse, .heart:
            return animate ? 1.2 : 1.0
        case .shake:
            return 1.0
        default:
            return 1.0
        }
    }
    
    private var rotation: Angle {
        guard animate else { return .zero }
        switch emoji.effect {
        case .shake:
            return animate ? .degrees(10) : .degrees(-10)
        default:
            return .zero
        }
    }
    
    private var offset: CGFloat {
        guard animate else { return 0 }
        switch emoji.effect {
        case .confetti, .firework:
            return animate ? -20 : 0
        default:
            return 0
        }
    }
}

// MARK: - Full-Screen Emoji Effect

public struct FullScreenEmojiEffect: View {
    let emoji: String
    @State private var particles: [Particle] = []
    @State private var isAnimating = false
    
    public init(emoji: String) {
        self.emoji = emoji
    }
    
    public var body: some View {
        ZStack {
            Color.black.opacity(0.3).ignoresSafeArea()
            
            ForEach(particles) { particle in
                Text(emoji)
                    .font(.system(size: particle.size))
                    .position(particle.position)
                    .opacity(particle.opacity)
                    .scaleEffect(isAnimating ? particle.scale : 0.1)
                    .animation(.easeOut(duration: particle.duration), value: isAnimating)
            }
        }
        .onAppear {
            generateParticles()
            isAnimating = true
        }
    }
    
    private func generateParticles() {
        particles = (0..<20).map { index in
            Particle(
                id: index,
                position: CGPoint(x: CGFloat.random(in: 0...400), y: CGFloat.random(in: 0...800)),
                size: CGFloat.random(in: 20...60),
                duration: Double.random(in: 0.5...1.5),
                opacity: Double.random(in: 0.6...1.0),
                scale: CGFloat.random(in: 0.8...1.2)
            )
        }
    }
    
    struct Particle: Identifiable {
        let id: Int
        let position: CGPoint
        let size: CGFloat
        let duration: Double
        let opacity: Double
        let scale: CGFloat
    }
}

// MARK: - Emoji Status

public struct EmojiStatusView: View {
    let emoji: String
    @State private var isAnimating = false
    
    public init(emoji: String) {
        self.emoji = emoji
    }
    
    public var body: some View {
        Text(emoji)
            .font(.system(size: 20))
            .scaleEffect(isAnimating ? 1.2 : 1.0)
            .onAppear {
                withAnimation(.easeInOut(duration: 0.5).repeatForever(autoreverses: true)) {
                    isAnimating = true
                }
            }
    }
}

// MARK: - Emoji Status Picker

public struct EmojiStatusPickerView: View {
    @Environment(\.dismiss) private var dismiss
    let onSelect: (String) -> Void
    
    @State private var selectedEmoji: String?
    
    public init(onSelect: @escaping (String) -> Void) {
        self.onSelect = onSelect
    }
    
    public var body: some View {
        NavigationStack {
            ScrollView {
                LazyVGrid(columns: Array(repeating: GridItem(.flexible()), count: 6), spacing: 12) {
                    ForEach(AnimatedEmojiData.all) { emoji in
                        Button {
                            selectedEmoji = emoji.emoji
                            onSelect(emoji.emoji)
                            dismiss()
                        } label: {
                            Text(emoji.emoji)
                                .font(.system(size: 32))
                                .padding(8)
                                .background(Color.gray.opacity(0.15))
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding()
            }
            .navigationTitle("Emoji Status")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Cancel") { dismiss() }
                }
                #else
                ToolbarItem(placement: .primaryAction) {
                    Button("Cancel") { dismiss() }
                }
                #endif
            }
        }
    }
}