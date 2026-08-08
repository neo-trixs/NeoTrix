// NeoTrixTheme - Unified design system for NeoGram
// 极简风格统一设计令牌：颜色/圆角/间距/字体/卡片
// CLT 兼容约束: 不使用 Color(.systemGray6) 等 UIKit 动态色，统一 Color.gray.opacity()

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
    }

    // MARK: Radius
    public enum Radius {
        public static let small: CGFloat = 10
        public static let medium: CGFloat = 14
        public static let large: CGFloat = 18
        public static let capsule: CGFloat = 999
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
        public static let body = Font.body
        public static let caption = Font.caption
        public static let caption2 = Font.caption2
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

    /// 统一胶囊按钮样式
    func neoTrixCapsule(isSelected: Bool = false) -> some View {
        self
            .padding(.horizontal, NeoTrixTheme.Spacing.md)
            .padding(.vertical, NeoTrixTheme.Spacing.xs)
            .background(isSelected ? NeoTrixTheme.Colors.accent : NeoTrixTheme.Colors.surface)
            .foregroundColor(isSelected ? .white : NeoTrixTheme.Colors.textPrimary)
            .clipShape(Capsule())
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