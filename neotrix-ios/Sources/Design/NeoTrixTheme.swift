// NeoTrixTheme - Unified design system for NeoGram
// 极简风格统一设计令牌：颜色/圆角/间距/字体/卡片/渐变/状态色
// CLT 兼容约束: 不使用 Color(.systemGray6) 等 UIKit 动态色，统一 Color.gray.opacity()
// 对标: Telegram (列表/分组) + 小红书 (双列卡片) + TikTok (分类 Tab)

import SwiftUI

// MARK: - Design Tokens

public enum NeoTrixTheme {
    // MARK: Colors
    public enum Colors {
        /// 主背景（深色）
        public static let background = Color.black
        /// 表面（卡片/列表行）
        public static let surface = Color.gray.opacity(0.12)
        /// 强调色（品牌蓝）
        public static let accent = Color.blue
        /// 次要强调（AI 紫）
        public static let accentSecondary = Color.purple
        /// 成功/在线
        public static let success = Color.green
        /// 警告/未读
        public static let warning = Color.yellow
        /// 危险/删除
        public static let danger = Color.red
        /// 主文本
        public static let textPrimary = Color.primary
        /// 次要文本
        public static let textSecondary = Color.secondary
        /// 分隔线
        public static let separator = Color.gray.opacity(0.2)
        /// 输入框背景
        public static let inputBackground = Color.gray.opacity(0.15)
        /// 气泡（对方）
        public static let bubbleIncoming = Color.gray.opacity(0.25)
        /// 气泡（系统）
        public static let bubbleSystem = Color.gray.opacity(0.15)
        /// 品牌渐变（头像/启动页/AI 入口）
        public static let brandGradient = LinearGradient(
            colors: [Color.blue, Color.purple],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        /// 选中态背景（FilterChip/CategoryBar 等）
        public static let selection = accent.opacity(0.2)
        /// 未读徽标/选中填充（复用强调色）
        public static let badge = accent
        /// 在线状态点
        public static let online = success
        /// Premium 星标
        public static let premium = warning
        /// 验证标记
        public static let verified = accent
        /// Toast 背景
        public static let toastBackground = Color.black.opacity(0.8)
        /// 占位图底
        public static let placeholder = Color.gray.opacity(0.1)
    }

    // MARK: Gradient（可复用的命名渐变）
    public enum Gradients {
        /// 品牌渐变（头像/启动页/AI 入口）
        public static let brand = Colors.brandGradient
        /// 故事环渐变（StoryRing 未读状态）
        public static let story = LinearGradient(
            colors: [Color.orange, Color.pink, Color.purple],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
        /// 危险渐变（删除/退出）
        public static let danger = LinearGradient(
            colors: [Color.red, Color.orange],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
    }

    // MARK: Radius
    public enum Radius {
        public static let xs: CGFloat = 6
        public static let small: CGFloat = 10
        public static let medium: CGFloat = 14
        public static let large: CGFloat = 18
        public static let capsule: CGFloat = 999
        /// 细分档位（补全: 4/8/12/16 高频值，此前代码大量硬编码导致不一致）
        public static let tiny: CGFloat = 4
        public static let s8: CGFloat = 8
        public static let s12: CGFloat = 12
        public static let s16: CGFloat = 16
    }

    // MARK: Spacing
    public enum Spacing {
        public static let xs: CGFloat = 4
        public static let sm: CGFloat = 8
        public static let md: CGFloat = 12
        public static let lg: CGFloat = 16
        public static let xl: CGFloat = 24
    }

    // MARK: Fonts
    public enum Fonts {
        public static let title = Font.title2.bold()
        public static let headline = Font.headline
        public static let subheadline = Font.subheadline
        public static let body = Font.body
        public static let caption = Font.caption
        public static let caption2 = Font.caption2
    }

    // MARK: Shadows
    public enum Shadows {
        /// 卡片柔和阴影
        public static let card = Color.black.opacity(0.05)
        /// Toast 阴影
        public static let toast = Color.black.opacity(0.2)
    }

    // MARK: 平台品牌色（LiveFeed 平台徽标统一映射）
    public enum PlatformColors {
        public static func color(for platform: String) -> Color {
            switch platform.lowercased() {
            case "youtube": return .red
            case "tiktok", "douyin": return .black
            case "instagram": return .purple
            case "twitter", "x": return .blue
            case "reddit": return .orange
            case "telegram": return .cyan
            case "bilibili": return .pink
            case "whatsapp", "wechat": return .green
            default: return .gray
            }
        }
    }

    // MARK: 内容类型色（LiveFeed 类型角标统一映射）
    public enum TypeColors {
        public static func color(for type: String) -> Color {
            switch type.lowercased() {
            case "moment": return .pink      // 社交状态流
            case "stream": return .cyan      // 实时事件流
            case "video": return .red
            case "image": return .orange
            case "chat": return .blue
            case "contact": return .green
            case "document": return .indigo
            default: return .purple
            }
        }
    }

    // MARK: 评分三档色（LiveCard 评分统一映射）
    public enum ScoreColors {
        public static func color(for score: Double) -> Color {
            score >= 70 ? .green : score >= 40 ? .orange : .red
        }
    }
}

// MARK: - Card Modifier (LiveCard 标准)

/// 统一卡片语言：圆角 + 柔和阴影 + 可选类型角标
public struct NeoTrixCard: ViewModifier {
    var cornerRadius: CGFloat
    var padding: CGFloat

    public init(cornerRadius: CGFloat = NeoTrixTheme.Radius.medium,
                padding: CGFloat = NeoTrixTheme.Spacing.md) {
        self.cornerRadius = cornerRadius
        self.padding = padding
    }

    public func body(content: Content) -> some View {
        content
            .padding(padding)
            .background(NeoTrixTheme.Colors.surface)
            .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
    }
}

public extension View {
    /// 统一卡片样式
    func neoTrixCard(cornerRadius: CGFloat = NeoTrixTheme.Radius.medium,
                     padding: CGFloat = NeoTrixTheme.Spacing.md) -> some View {
        modifier(NeoTrixCard(cornerRadius: cornerRadius, padding: padding))
    }

    /// 统一胶囊按钮样式（选中态走设计系统 accent）
    func neoTrixCapsule(isSelected: Bool = false) -> some View {
        self
            .padding(.horizontal, NeoTrixTheme.Spacing.md)
            .padding(.vertical, NeoTrixTheme.Spacing.xs)
            .background(isSelected ? NeoTrixTheme.Colors.accent : NeoTrixTheme.Colors.surface)
            .foregroundColor(isSelected ? .white : NeoTrixTheme.Colors.textPrimary)
            .clipShape(Capsule())
    }

    /// 统一未读徽标样式
    func neoTrixUnreadBadge(_ count: Int) -> some View {
        Text("\(count)")
            .font(NeoTrixTheme.Fonts.caption.bold())
            .foregroundColor(.white)
            .frame(minWidth: 20, minHeight: 20)
            .background(NeoTrixTheme.Colors.badge)
            .clipShape(Circle())
    }

    /// 统一在线状态点（底部右下角 + 描边）
    func neoTrixOnlineDot(size: CGFloat = 14) -> some View {
        Circle()
            .fill(NeoTrixTheme.Colors.online)
            .frame(width: size, height: size)
            .overlay(Circle().stroke(NeoTrixTheme.Colors.textPrimary.opacity(0.2), lineWidth: 2))
    }

    /// 统一 Toast 提示条
    func neoTrixToast(_ message: String) -> some View {
        Text(message)
            .font(NeoTrixTheme.Fonts.caption)
            .padding(.horizontal, NeoTrixTheme.Spacing.lg)
            .padding(.vertical, NeoTrixTheme.Spacing.sm)
            .background(NeoTrixTheme.Colors.toastBackground)
            .foregroundColor(.white)
            .clipShape(Capsule())
            .shadow(color: NeoTrixTheme.Shadows.toast, radius: 8, y: 2)
    }
}

// MARK: - Type Badge

/// 卡片类型角标（LiveFeed 类型 / Moments / Stream 统一语言）
public struct NeoTrixTypeBadge: View {
    let title: String
    let color: Color

    public init(title: String, color: Color = NeoTrixTheme.Colors.accentSecondary) {
        self.title = title
        self.color = color
    }

    public var body: some View {
        Text(title)
            .font(NeoTrixTheme.Fonts.caption2.bold())
            .padding(.horizontal, NeoTrixTheme.Spacing.sm)
            .padding(.vertical, 2)
            .background(color.opacity(0.25))
            .foregroundColor(color)
            .clipShape(Capsule())
    }
}

// MARK: - Section Header

/// 统一分组标题
public struct NeoTrixSectionHeader: View {
    let title: String

    public init(_ title: String) {
        self.title = title
    }

    public var body: some View {
        Text(title)
            .font(NeoTrixTheme.Fonts.headline)
            .foregroundColor(NeoTrixTheme.Colors.textSecondary)
            .padding(.horizontal, NeoTrixTheme.Spacing.lg)
            .padding(.top, NeoTrixTheme.Spacing.md)
    }
}

// MARK: - Avatar

/// 统一头像（首字母 + 品牌渐变）
public struct NeoTrixAvatar: View {
    let title: String
    let size: CGFloat
    let gradient: LinearGradient

    public init(title: String, size: CGFloat = 52,
                gradient: LinearGradient = NeoTrixTheme.Gradients.brand) {
        self.title = title
        self.size = size
        self.gradient = gradient
    }

    public var body: some View {
        ZStack {
            Circle()
                .fill(gradient)
                .frame(width: size, height: size)
            Text(String(title.prefix(1)).uppercased())
                .font(.system(size: size * 0.42, weight: .bold))
                .foregroundColor(.white)
        }
    }
}

// MARK: - Empty State

/// 统一空状态（图标 + 主文案 + 次文案）
public struct NeoTrixEmptyState: View {
    let icon: String
    let title: String
    let message: String

    public init(icon: String, title: String, message: String) {
        self.icon = icon
        self.title = title
        self.message = message
    }

    public var body: some View {
        VStack(spacing: NeoTrixTheme.Spacing.md) {
            Image(systemName: icon)
                .font(.system(size: 48))
                .foregroundColor(NeoTrixTheme.Colors.textSecondary)
            Text(title)
                .font(NeoTrixTheme.Fonts.headline)
            Text(message)
                .font(NeoTrixTheme.Fonts.caption)
                .foregroundColor(NeoTrixTheme.Colors.textSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
