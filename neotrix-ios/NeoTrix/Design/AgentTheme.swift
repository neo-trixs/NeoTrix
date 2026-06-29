import SwiftUI

struct AgentTheme {
    static let primary = Color(red: 0.0, green: 0.48, blue: 1.0)
    static let secondary = Color(red: 0.4, green: 0.8, blue: 1.0)
    static let accent = Color(red: 0.2, green: 0.9, blue: 0.6)
    static let surface = Color(.systemGray6)
    static let agentBubble = Color(.systemGray5)
    static let userBubble = Color.blue
    static let positive = Color.green
    static let negative = Color.red
    static let warning = Color.orange

    static let scoreHigh = Color.green
    static let scoreMid = Color.orange
    static let scoreLow = Color.red

    static let platformColors: [String: Color] = [
        "youtube": .red,
        "tiktok": .black,
        "douyin": Color(red: 0.1, green: 0.1, blue: 0.1),
        "instagram": .purple,
        "twitter": .blue,
        "reddit": .orange,
        "bilibili": .cyan,
    ]
}

struct RoundedCard: ViewModifier {
    func body(content: Content) -> some View {
        content
            .padding()
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 12))
            .shadow(color: .black.opacity(0.05), radius: 4, y: 2)
    }
}

extension View {
    func roundedCard() -> some View {
        modifier(RoundedCard())
    }
}

struct PlatformBadge: View {
    let platform: String

    var body: some View {
        Text(platform.capitalized)
            .font(.caption2.bold())
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(AgentTheme.platformColors[platform.lowercased(), default: .gray])
            .foregroundColor(.white)
            .clipShape(Capsule())
    }
}

struct ScoreBadge: View {
    let score: Double

    var body: some View {
        Text(String(format: "%.1f", score))
            .font(.caption.bold())
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(scoreColor(score))
            .foregroundColor(.white)
            .clipShape(Capsule())
    }

    private func scoreColor(_ s: Double) -> Color {
        s >= 0.7 ? .green : s >= 0.4 ? .orange : .red
    }
}
