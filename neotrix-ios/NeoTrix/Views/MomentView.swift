import SwiftUI

struct MomentView: View {
    @StateObject private var viewModel = MomentViewModel()
    @State private var showLoginSheet = false
    @State private var selectedPlatform: String?
    @State private var platformFilter: Set<String> = []
    
    let platforms = ["youtube", "tiktok", "douyin", "instagram", "twitter", "reddit", "bilibili"]
    
    var filteredMoments: [MomentItem] {
        guard !platformFilter.isEmpty else { return viewModel.moments }
        return viewModel.moments.filter { platformFilter.contains($0.platform) }
    }
    
    var body: some View {
        VStack(spacing: 0) {
            socialStatusBar
            
            platformFilterBar
            
            if viewModel.isLoading {
                Spacer()
                ProgressView("Loading moments...")
                Spacer()
            } else if filteredMoments.isEmpty {
                Spacer()
                VStack(spacing: 12) {
                    Image(systemName: "rectangle.3.offgrid.bubble.left")
                        .font(.system(size: 48))
                        .foregroundColor(.secondary)
                    Text("No moments yet")
                        .font(.headline)
                    Text("Import content from social media to see your curated feed")
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal)
                }
                Spacer()
            } else {
                List {
                    ForEach(filteredMoments) { item in
                        MomentCardView(item: item)
                            .swipeActions(edge: .trailing) {
                                Button(role: .destructive) {
                                    Task { await viewModel.submitFeedback(itemId: item.id, liked: false) }
                                } label: {
                                    Label("Dismiss", systemImage: "hand.thumbsdown")
                                }
                            }
                            .swipeActions(edge: .leading) {
                                Button {
                                    Task { await viewModel.submitFeedback(itemId: item.id, liked: true) }
                                } label: {
                                    Label("Like", systemImage: "hand.thumbsup")
                                }
                                .tint(.green)
                            }
                    }
                }
                .listStyle(.plain)
                .refreshable {
                    await viewModel.loadSocialStatus()
                }
            }
        }
        .navigationTitle("Moments")
        .sheet($showLoginSheet) {
            WebLoginView(platform: selectedPlatform ?? "youtube")
        }
        .task {
            await viewModel.loadSocialStatus()
            await viewModel.loadFilterConfig()
        }
    }
    
    private var socialStatusBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(viewModel.socialStatuses, id: \.platform) { status in
                    Button {
                        selectedPlatform = status.platform
                        if status.loggedIn {
                            platformFilter.toggle(status.platform)
                        } else {
                            showLoginSheet = true
                        }
                    } label: {
                        HStack(spacing: 4) {
                            Circle()
                                .fill(status.loggedIn ? Color.green : Color.red)
                                .frame(width: 8, height: 8)
                            Text(status.platform.capitalized)
                                .font(.caption)
                                .foregroundColor(.primary)
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(platformFilter.contains(status.platform) ? Color.blue.opacity(0.2) : Color(.systemGray6))
                        .clipShape(Capsule())
                    }
                }
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
    }
    
    private var platformFilterBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 6) {
                ForEach(platforms, id: \.self) { p in
                    Button {
                        if platformFilter.contains(p) { platformFilter.remove(p) }
                        else { platformFilter.insert(p) }
                    } label: {
                        Text(p.capitalized)
                            .font(.caption2)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(platformFilter.isEmpty || platformFilter.contains(p) ? Color.blue : Color(.systemGray5))
                            .foregroundColor(platformFilter.isEmpty || platformFilter.contains(p) ? .white : .secondary)
                            .clipShape(Capsule())
                    }
                }
            }
            .padding(.horizontal)
            .padding(.bottom, 8)
        }
    }
}

struct MomentCardView: View {
    let item: MomentItem
    
    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                if let thumb = item.thumbnail, let url = URL(string: thumb) {
                    AsyncImage(url: url) { phase in
                        switch phase {
                        case .success(let img):
                            img.resizable().aspectRatio(contentMode: .fill)
                        default:
                            Color(.systemGray5)
                        }
                    }
                } else {
                    Color(.systemGray5)
                }
            }
            .frame(width: 100, height: 70)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            
            VStack(alignment: .leading, spacing: 4) {
                Text(item.title)
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .lineLimit(2)
                
                HStack(spacing: 4) {
                    Text(item.platform.capitalized)
                        .font(.caption2)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(platformColor(item.platform))
                        .foregroundColor(.white)
                        .clipShape(Capsule())
                    
                    if let author = item.author {
                        Text(author)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                
                HStack(spacing: 8) {
                    if let views = item.viewCount {
                        Label(formatCount(views), systemImage: "eye")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    if let likes = item.likeCount {
                        Label(formatCount(likes), systemImage: "heart")
                            .font(.caption2)
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                    Text(String(format: "%.1f", item.score))
                        .font(.caption)
                        .fontWeight(.bold)
                        .foregroundColor(scoreColor(item.score))
                }
            }
        }
        .padding(.vertical, 4)
    }
    
    private func platformColor(_ p: String) -> Color {
        switch p {
        case "youtube": return .red
        case "tiktok", "douyin": return .black
        case "instagram": return .purple
        case "twitter": return .blue
        case "reddit": return .orange
        case "bilibili": return .cyan
        default: return .gray
        }
    }
    
    private func scoreColor(_ s: Double) -> Color {
        s >= 0.7 ? .green : s >= 0.4 ? .orange : .red
    }
    
    private func formatCount(_ n: Int64) -> String {
        if n >= 1_000_000 { return String(format: "%.1fM", Double(n) / 1_000_000) }
        if n >= 1_000 { return String(format: "%.1fK", Double(n) / 1_000) }
        return "\(n)"
    }
}

extension Set<String> {
    mutating func toggle(_ element: String) {
        if contains(element) { remove(element) }
        else { insert(element) }
    }
}