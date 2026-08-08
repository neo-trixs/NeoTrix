// LiveFeedUI - Live 标签页 (Live Feed Architecture)
// 对标: 小红书双列瀑布流 + TikTok LIVE 分类 Tab + Instagram 网格
// 交互: 点赞 / 分享 / 不感兴趣 (长按) / 隐藏作者 / 屏蔽关键词
// 搜索: 顶部搜索框 → LiveFeedEngine.search (官方接口 + 过滤 + E8 排序)

import SwiftUI

// MARK: - Live Feed View

public struct LiveFeedView: View {
    @StateObject private var engine = LiveFeedEngine.shared
    @State private var pendingKeyword = ""
    @State private var shareItem: LiveFeedItem?
    @State private var showBlockedAlert = false
    @State private var toastMessage: String?
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                categoryBar
                
                if engine.isSearching {
                    Spacer()
                    ProgressView("Searching...")
                    Spacer()
                } else if engine.items.isEmpty {
                    emptyState
                } else {
                    waterfallGrid
                }
            }
            .navigationTitle("Live")
            .searchable(text: $engine.searchQuery, prompt: "Search all resources")
            .onSubmit(of: .search) {
                Task { await engine.refresh() }
            }
            .onChange(of: engine.searchQuery) { _, newValue in
                // 300ms debounce 搜索
                Task {
                    try? await Task.sleep(nanoseconds: 300_000_000)
                    if engine.searchQuery == newValue {
                        await engine.refresh()
                    }
                }
            }
            .refreshable {
                await engine.refresh()
            }
            .task {
                if engine.items.isEmpty {
                    await engine.loadFeed()
                }
            }
            .alert("Block Keyword", isPresented: $showBlockedAlert) {
                TextField("Keyword", text: $pendingKeyword)
                Button("Block") { engine.blockKeyword(pendingKeyword) }
                Button("Cancel", role: .cancel) {}
            }
            .sheet(item: $shareItem) { item in
                #if os(iOS)
                ShareSheet(items: [item.title, item.subtitle, item.sourceURL?.absoluteString ?? ""].compactMap { $0 })
                #else
                Text("Share: \(item.title)")
                #endif
            }
            .overlay(alignment: .bottom) {
                if toastMessage != nil {
                    toastView
                }
            }
        }
    }
    
    // MARK: - 分类 Tab (TikTok LIVE 2026 模式)
    
    private var categoryBar: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(FeedCategory.allCases) { category in
                    Button {
                        engine.selectedCategory = category
                        Task { await engine.loadFeed() }
                    } label: {
                        Text(category.rawValue)
                            .font(.caption.weight(engine.selectedCategory == category ? .bold : .regular))
                            .neoTrixCapsule(isSelected: engine.selectedCategory == category)
                    }
                }
            }
            .padding(.horizontal)
            .padding(.vertical, 8)
        }
    }
    
    // MARK: - 双列瀑布流 (小红书模式)
    
    private var waterfallGrid: some View {
        ScrollView {
            LazyVGrid(columns: [GridItem(.flexible(), spacing: 12), GridItem(.flexible(), spacing: 12)], spacing: 12) {
                ForEach(engine.items) { item in
                    LiveCardView(item: item, engine: engine)
                        .contextMenu {
                            Button {
                                engine.like(item)
                            } label: {
                                Label(engine.likedIDs.contains(item.id) ? "Unlike" : "Like", systemImage: engine.likedIDs.contains(item.id) ? "heart.fill" : "heart")
                            }
                            Button {
                                shareItem = item
                            } label: {
                                Label("Share", systemImage: "square.and.arrow.up")
                            }
                            Divider()
                            Button(role: .destructive) {
                                engine.notInterested(item)
                                showToastMessage("Marked as not interested")
                            } label: {
                                Label("Not Interested", systemImage: "hand.thumbsdown")
                            }
                            if let author = item.author {
                                Button(role: .destructive) {
                                    engine.hideAuthor(author)
                                    showToastMessage("Hidden \(author)")
                                } label: {
                                    Label("Hide \(author)", systemImage: "eye.slash")
                                }
                            }
                            Button(role: .destructive) {
                                pendingKeyword = item.title.split(separator: " ").first.map(String.init) ?? ""
                                showBlockedAlert = true
                            } label: {
                                Label("Block Keyword", systemImage: "nosign")
                            }
                        }
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
        }
    }
    
    // MARK: - 空状态
    
    private var emptyState: some View {
        NeoTrixEmptyState(
            icon: "rectangle.3.offgrid.bubble.left",
            title: "No content yet",
            message: "Pull to refresh or search across all resources"
        )
    }
    
    // MARK: - Toast
    
    private func showToastMessage(_ message: String) {
        toastMessage = message
        Task {
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            toastMessage = nil
        }
    }
    
    private var toastView: some View {
        Text(toastMessage ?? "")
            .neoTrixToast(toastMessage ?? "")
            .padding(.top, 8)
            .transition(.move(edge: .top).combined(with: .opacity))
    }
}

// MARK: - Live Card View

public struct LiveCardView: View {
    let item: LiveFeedItem
    @ObservedObject var engine: LiveFeedEngine
    
    public init(item: LiveFeedItem, engine: LiveFeedEngine) {
        self.item = item
        self.engine = engine
    }
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            // 缩略图 / 类型图标
            ZStack {
                if let url = item.thumbnailURL {
                    AsyncImage(url: url) { phase in
                        switch phase {
                        case .success(let img):
                            img.resizable().aspectRatio(contentMode: .fill)
                        default:
                            placeholder
                        }
                    }
                } else {
                    placeholder
                }
                
                // 类型角标
                VStack {
                    HStack {
                        Spacer()
                        typeBadge
                    }
                    Spacer()
                }
                .padding(6)
            }
            .frame(height: 120)
            .clipShape(RoundedRectangle(cornerRadius: 10))
            
            // 标题
            Text(item.title)
                .font(.subheadline)
                .fontWeight(.medium)
                .lineLimit(2)
                .multilineTextAlignment(.leading)
            
            // 平台 + 作者
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
                        .lineLimit(1)
                }
            }
            
            // 互动统计 + 评分
            HStack(spacing: 8) {
                Label(formatCount(item.engagement.likes), systemImage: "heart")
                    .font(.caption2)
                    .foregroundColor(.secondary)
                Label(formatCount(item.engagement.views), systemImage: "eye")
                    .font(.caption2)
                    .foregroundColor(.secondary)
                Spacer()
                Text(String(format: "%.0f", item.score))
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(scoreColor(item.score))
            }
            
            // 操作栏 (点赞/分享)
            HStack(spacing: 16) {
                Button {
                    withAnimation(.spring(response: 0.3, dampingFraction: 0.6)) {
                        engine.like(item)
                    }
                } label: {
                    Image(systemName: engine.likedIDs.contains(item.id) ? "heart.fill" : "heart")
                        .foregroundColor(engine.likedIDs.contains(item.id) ? .red : .secondary)
                }
                .buttonStyle(.plain)
                
                Button {
                    engine.share(item)
                } label: {
                    Image(systemName: "square.and.arrow.up")
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                
                Spacer()
                
                Button {
                    engine.notInterested(item)
                } label: {
                    Image(systemName: "hand.thumbsdown")
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            .font(.system(size: 14))
        }
        .padding(10)
        .background(NeoTrixTheme.Colors.surface)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .shadow(color: NeoTrixTheme.Shadows.card, radius: 4, x: 0, y: 2)
    }
    
    private var placeholder: some View {
        ZStack {
            NeoTrixTheme.Colors.placeholder
            Image(systemName: typeIcon(item.type))
                .font(.system(size: 32))
                .foregroundColor(.gray)
        }
    }
    
    private var typeBadge: some View {
        // 统一设计系统: NeoTrixTypeBadge（Moments/Stream 融合类型角标）
        NeoTrixTypeBadge(title: item.type.rawValue.capitalized, color: typeBadgeColor(item.type))
    }
    
    private func typeBadgeColor(_ type: FeedItemType) -> Color {
        NeoTrixTheme.TypeColors.color(for: type.rawValue)
    }
    
    private func typeIcon(_ type: FeedItemType) -> String {
        switch type {
        case .text: return "text.alignleft"
        case .image: return "photo"
        case .video: return "play.rectangle"
        case .document: return "doc"
        case .chat: return "bubble.left"
        case .contact: return "person"
        case .moment: return "sparkles"
        case .stream: return "dot.radiowaves.left.and.right"
        }
    }
    
    private func platformColor(_ p: String) -> Color {
        NeoTrixTheme.PlatformColors.color(for: p)
    }
    
    private func scoreColor(_ s: Double) -> Color {
        NeoTrixTheme.ScoreColors.color(for: s)
    }
    
    private func formatCount(_ n: Int64) -> String {
        if n >= 1_000_000 { return String(format: "%.1fM", Double(n) / 1_000_000) }
        if n >= 1_000 { return String(format: "%.1fK", Double(n) / 1_000) }
        return "\(n)"
    }
}

// MARK: - Share Sheet

#if os(iOS)
import UIKit

public struct ShareSheet: UIViewControllerRepresentable {
    let items: [String]
    
    public init(items: [String]) {
        self.items = items
    }
    
    public func makeUIViewController(context: Context) -> UIActivityViewController {
        UIActivityViewController(activityItems: items, applicationActivities: nil)
    }
    
    public func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}
#endif