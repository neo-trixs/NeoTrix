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

// MARK: - Emoji Effect Style

/// 全屏动画效果的统一渲染引擎风格（bounce/shake/pulse 有独立实现，见 FullScreenEmojiEffect）
public enum EmojiEffectStyle {
    /// 静态图：居中展示大 emoji，无粒子
    case none
    /// 粒子雨：随机撒落（默认）
    case confetti
    /// 烟花：自中心向外辐射
    case firework
    /// 弹跳：粒子自底部向上弹
    case bounce
    /// 摇晃：单 emoji 水平抖动
    case shake
    /// 脉冲：单 emoji 缩放脉动
    case pulse
}

// MARK: - Full-Screen Emoji Effect

public struct FullScreenEmojiEffect: View {
    let emoji: String
    var style: EmojiEffectStyle = .confetti
    @State private var particles: [Particle] = []
    @State private var isAnimating = false
    
    public init(emoji: String, style: EmojiEffectStyle = .confetti) {
        self.emoji = emoji
        self.style = style
    }
    
    public var body: some View {
        GeometryReader { geo in
            ZStack {
                NeoTrixTheme.Colors.toastBackground.ignoresSafeArea()
                
                switch style {
                case .none:
                    centerEmoji
                    
                case .pulse:
                    centerEmoji
                        .scaleEffect(isAnimating ? 1.35 : 0.75)
                        .opacity(isAnimating ? 1 : 0.5)
                        .animation(.easeInOut(duration: 0.55).repeatForever(autoreverses: true), value: isAnimating)
                    
                case .shake:
                    centerEmoji
                        .offset(x: isAnimating ? 14 : -14)
                        .animation(.easeInOut(duration: 0.12).repeatForever(autoreverses: true), value: isAnimating)
                    
                case .confetti, .firework, .bounce:
                    ForEach(particles) { particle in
                        Text(emoji)
                            .font(.system(size: particle.size))
                            .position(isAnimating ? particle.target : particle.position)
                            .opacity(particle.opacity)
                            .scaleEffect(isAnimating ? particle.scale : 0.1)
                            .animation(.easeOut(duration: particle.duration), value: isAnimating)
                    }
                }
            }
            .onAppear {
                generateParticles(in: geo.size)
                isAnimating = true
            }
        }
    }
    
    private var centerEmoji: some View {
        Text(emoji)
            .font(.system(size: 110))
    }
    
    private func generateParticles(in size: CGSize) {
        let count: Int
        switch style {
        case .none, .pulse, .shake: count = 0
        case .firework: count = 24
        case .bounce: count = 8
        case .confetti: count = 20
        }
        
        let center = CGPoint(x: size.width / 2, y: size.height / 2)
        particles = (0..<count).map { index in
            let start: CGPoint
            let target: CGPoint
            switch style {
            case .firework:
                // 自中心向外辐射
                let angle = Double(index) / Double(max(count, 1)) * .pi * 2
                let radius = CGFloat.random(in: 60...max(size.width, size.height) * 0.45)
                start = center
                target = CGPoint(x: center.x + cos(angle) * radius, y: center.y + sin(angle) * radius)
            case .bounce:
                // 自底部向上弹
                let baseX = CGFloat.random(in: 0...max(size.width, 1))
                start = CGPoint(x: baseX, y: size.height * 0.95)
                target = CGPoint(x: baseX, y: start.y - CGFloat.random(in: 50...130))
            default:
                // 随机撒落（保持原有行为）
                let point = CGPoint(x: CGFloat.random(in: 0...max(size.width, 1)), y: CGFloat.random(in: 0...max(size.height, 1)))
                start = point
                target = point
            }
            return Particle(
                id: index,
                position: start,
                target: target,
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
        let target: CGPoint
        let size: CGFloat
        let duration: Double
        let opacity: Double
        let scale: CGFloat
    }
}

// MARK: - Effect → EmojiEffectStyle 映射
// 说明: AnimatedEmoji.Effect 9 变体统一映射到渲染引擎风格（此前全部走粒子、无区分）

public extension AnimatedEmoji.Effect {
    /// 映射到统一渲染引擎（消费全部 9 个变体: bounce/shake/pulse 各有独立实现）
    var emojiEffectStyle: EmojiEffectStyle {
        switch self {
        case .none: return .none
        case .bounce: return .bounce
        case .explode: return .firework
        case .confetti: return .confetti
        case .firework: return .firework
        case .heart: return .pulse
        case .star: return .confetti
        case .rainbow: return .confetti
        case .shake: return .shake
        case .pulse: return .pulse
        }
    }
}

// MARK: - Emoji Status Picker

public struct EmojiStatusPickerView: View {
    @Environment(\.dismiss) private var dismiss
    let onSelect: (String) -> Void
    
    @State private var selectedEmoji: String?
    /// Premium 门控（对标 ReactionPickerView: 统一 premium_tier 事实源 + 锁标 + 禁用）
    @State private var isPremium = false
    
    public init(onSelect: @escaping (String) -> Void) {
        self.onSelect = onSelect
    }
    
    public var body: some View {
        NavigationStack {
            ScrollView {
                LazyVGrid(columns: Array(repeating: GridItem(.flexible()), count: 6), spacing: 12) {
                    ForEach(AnimatedEmojiData.all) { emoji in
                        Button {
                            guard isPremium || !emoji.isPremium else { return }
                            selectedEmoji = emoji.emoji
                            onSelect(emoji.emoji)
                            dismiss()
                        } label: {
                            ZStack(alignment: .topTrailing) {
                                Text(emoji.emoji)
                                    .font(.system(size: 32))
                                
                                if emoji.isPremium && !isPremium {
                                    Image(systemName: "star.fill")
                                        .font(.system(size: 10))
                                        .foregroundColor(NeoTrixTheme.Colors.premium)
                                        .offset(x: 6, y: -6)
                                }
                            }
                            .padding(8)
                            .background(NeoTrixTheme.Colors.inputBackground)
                            .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                        .buttonStyle(.plain)
                        .disabled(emoji.isPremium && !isPremium)
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
        .onAppear {
            // 统一 premium 事实源（与 ReactionManager/PremiumManager 一致）
            if let tierRaw = UserDefaults.standard.string(forKey: "premium_tier") {
                isPremium = PremiumTier(rawValue: tierRaw) != .free
            }
        }
    }
}