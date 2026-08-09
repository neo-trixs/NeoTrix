// StoriesUI - Telegram Stories feature (Premium)
// Mirrors Telegram's StoryController + Story components

import SwiftUI

// MARK: - Story Models

public struct StoryItem: Identifiable {
    public let id: Int64
    public let authorName: String
    public let authorInitial: String
    public let avatarColor: Color
    public var isSeen: Bool
    public let isPremium: Bool
    public let isCloseFriends: Bool
    public let timestamp: Date
    public let duration: TimeInterval
    /// Story 内容（text/link/image/video）— StoryViewerView 真实渲染
    public var content: StoryContent? = nil
}

public struct StoryContent: Identifiable {
    public let id: Int64
    public let type: StoryType
    public let caption: String
    public let timestamp: Date
    
    public enum StoryType {
        case image(Data)
        case video(URL)
        case text(String)
        case link(String, URL)
    }
}

// MARK: - Story View Model

@MainActor
public final class StoryViewModel: ObservableObject {
    @Published public var stories: [StoryItem] = []
    
    private let core = NeoGramCore.shared
    
    public init() {
        loadStories()
    }
    
    private func loadStories() {
        stories = [
            StoryItem(id: 1, authorName: "NeoTrix AI", authorInitial: "N", avatarColor: .purple, isSeen: false, isPremium: true, isCloseFriends: false, timestamp: Date(), duration: 5,
                      content: StoryContent(id: 1, type: .text("NeoTrix AI — your self-evolving developer toolkit is live. E8, VSA HyperCube, and GWT all working together."), caption: "NeoTrix update", timestamp: Date())),
            StoryItem(id: 2, authorName: "Alice", authorInitial: "A", avatarColor: .pink, isSeen: false, isPremium: false, isCloseFriends: true, timestamp: Date().addingTimeInterval(-600), duration: 8,
                      content: StoryContent(id: 2, type: .link("New NeoTrix Release", URL(string: "https://neotrix.dev")!), caption: "What's new", timestamp: Date().addingTimeInterval(-600))),
            StoryItem(id: 3, authorName: "Bob", authorInitial: "B", avatarColor: .blue, isSeen: true, isPremium: false, isCloseFriends: false, timestamp: Date().addingTimeInterval(-1800), duration: 5,
                      content: StoryContent(id: 3, type: .image(Data()), caption: "Weekend vibes", timestamp: Date().addingTimeInterval(-1800))),
            StoryItem(id: 4, authorName: "Design", authorInitial: "D", avatarColor: .orange, isSeen: false, isPremium: true, isCloseFriends: false, timestamp: Date().addingTimeInterval(-3600), duration: 15,
                      content: StoryContent(id: 4, type: .video(URL(string: "https://neotrix.dev/story.mp4")!), caption: "NeoTrix in 60s", timestamp: Date().addingTimeInterval(-3600))),
        ]
    }
    
    public func markAsSeen(_ storyId: Int64) {
        if let index = stories.firstIndex(where: { $0.id == storyId }) {
            stories[index].isSeen = true
        }
    }
    
    /// 发布新 Story（融合: 官方 Stories + AI 摘要）
    public func publishStory(caption: String, isCloseFriends: Bool = false) {
        let newStory = StoryItem(
            id: Int64(Date().timeIntervalSince1970),
            authorName: "My Story",
            authorInitial: String(caption.prefix(1)).uppercased(),
            avatarColor: .purple,
            isSeen: false,
            isPremium: true,
            isCloseFriends: isCloseFriends,
            timestamp: Date(),
            duration: 5,
            content: StoryContent(
                id: Int64(Date().timeIntervalSince1970),
                type: .text(caption),
                caption: "",
                timestamp: Date()
            )
        )
        stories.insert(newStory, at: 0)
    }
}

// MARK: - Story Ring (Story Circle)

public struct StoryRing: View {
    let story: StoryItem
    let action: () -> Void
    
    public var body: some View {
        Button(action: action) {
            VStack(spacing: 6) {
                ZStack {
                    Circle()
                        .stroke(story.isSeen ? AnyShapeStyle(NeoTrixTheme.Colors.separator) : AnyShapeStyle(gradient), lineWidth: 3)
                        .frame(width: 64, height: 64)
                    
                    Circle()
                        .fill(story.avatarColor.gradient)
                        .frame(width: 54, height: 54)
                    
                    Text(story.authorInitial)
                        .font(.title3.bold())
                        .foregroundColor(.white)
                }
                
                Text(story.authorName)
                    .font(.caption)
                    .lineLimit(1)
                    .frame(width: 70)
            }
        }
        .buttonStyle(.plain)
    }
    
    private var gradient: LinearGradient {
        NeoTrixTheme.Gradients.story
    }
}

// MARK: - Story Viewer

public struct StoryViewerView: View {
    @Environment(\.dismiss) private var dismiss
    let stories: [StoryItem]
    /// 故事已读回调（对标 Telegram: 浏览即标记已读）
    var onSeen: ((Int64) -> Void)?
    @State private var currentIndex = 0
    @State private var progress: CGFloat = 0
    @State private var timer: Timer?
    @State private var isPaused = false
    
    public init(stories: [StoryItem], onSeen: ((Int64) -> Void)? = nil) {
        self.stories = stories
        self.onSeen = onSeen
    }
    
    public var body: some View {
        ZStack {
            NeoTrixTheme.Colors.background.ignoresSafeArea()
            
            if !stories.isEmpty {
                // Story content（真实渲染 StoryContent: text/link/image/video）
                StoryContentView(content: stories[currentIndex].content, index: currentIndex)
                
                // Progress bars
                VStack {
                    HStack(spacing: 4) {
                        ForEach(0..<stories.count, id: \.self) { index in
                            GeometryReader { geo in
                                ZStack(alignment: .leading) {
                                    Capsule()
                                        .fill(Color.white.opacity(0.3))
                                    Capsule()
                                        .fill(Color.white)
                                        .frame(width: index < currentIndex ? geo.size.width : (index == currentIndex ? geo.size.width * progress : 0))
                                }
                            }
                            .frame(height: 3)
                        }
                    }
                    .padding(.top, 8)
                    .padding(.horizontal)
                    
                    // Header
                    HStack {
                        Circle()
                            .fill(stories[currentIndex].avatarColor)
                            .frame(width: 36, height: 36)
                        
                        Text(stories[currentIndex].authorName)
                            .foregroundColor(.white)
                            .font(.subheadline.bold())
                        
                        Spacer()
                        
                        Button {
                            dismiss()
                        } label: {
                            Image(systemName: "xmark")
                                .foregroundColor(.white)
                        }
                    }
                    .padding(.horizontal)
                    .padding(.top, 12)
                    
                    Spacer()
                }
                
                // Tap zones
                HStack {
                    Color.clear
                        .contentShape(Rectangle())
                        .onTapGesture {
                            if currentIndex > 0 {
                                currentIndex -= 1
                            }
                        }
                    
                    Color.clear
                        .contentShape(Rectangle())
                        .onTapGesture {
                            if currentIndex < stories.count - 1 {
                                currentIndex += 1
                            } else {
                                dismiss()
                            }
                        }
                }
            }
        }
        .onAppear {
            markCurrentAsSeen()
            startTimer()
        }
        .onDisappear {
            timer?.invalidate()
        }
        .onChange(of: currentIndex) { _, _ in
            markCurrentAsSeen()
            startTimer()
        }
        // 长按暂停（对标 Telegram: 按住故事暂停进度）
        .simultaneousGesture(
            LongPressGesture(minimumDuration: 0.3)
                .onChanged { _ in isPaused = true }
                .onEnded { _ in isPaused = false }
        )
    }
    
    /// 标记当前故事已读（对标 Telegram: 浏览即已读）
    private func markCurrentAsSeen() {
        guard !stories.isEmpty, currentIndex < stories.count else { return }
        onSeen?(stories[currentIndex].id)
    }
    
    private func startTimer() {
        timer?.invalidate()
        progress = 0
        guard currentIndex < stories.count else { return }
        
        // 计时步进随 story.duration 计算（此前固定 0.02/0.1s = 恒 5 秒）
        let tick: TimeInterval = 0.1
        let duration = max(stories[currentIndex].duration, 0.1)
        let step = tick / duration
        
        timer = Timer.scheduledTimer(withTimeInterval: tick, repeats: true) { timer in
            if !isPaused {
                progress += step
                if progress >= 1 {
                    if currentIndex < stories.count - 1 {
                        currentIndex += 1
                    } else {
                        timer.invalidate()
                    }
                }
            }
        }
    }
}

// MARK: - Story Content Renderer

/// 真实渲染 StoryContent（text 显示文本 / link 显示链接按钮 / image、video 用 SF Symbol 占位 + 文案）
private struct StoryContentView: View {
    let content: StoryContent?
    let index: Int
    
    var body: some View {
        VStack {
            Spacer()
            
            Group {
                if let content {
                    contentView(for: content)
                } else {
                    Text("Story \(index + 1)")
                        .font(.largeTitle)
                        .foregroundColor(.white)
                }
            }
            
            Spacer()
        }
    }
    
    @ViewBuilder
    private func contentView(for content: StoryContent) -> some View {
        VStack(spacing: 16) {
            switch content.type {
            case .text(let text):
                ScrollView {
                    Text(text)
                        .font(.title2)
                        .foregroundColor(.white)
                        .multilineTextAlignment(.center)
                        .padding(.horizontal, 24)
                }
                
            case .link(let title, let url):
                VStack(spacing: 16) {
                    Image(systemName: "link")
                        .font(.system(size: 56))
                        .foregroundColor(.white)
                    Text(title)
                        .font(.headline)
                        .foregroundColor(.white)
                        .multilineTextAlignment(.center)
                    Link(destination: url) {
                        Label("Open Link", systemImage: "safari")
                            .font(NeoTrixTheme.Fonts.subheadline.bold())
                            .padding(.horizontal, NeoTrixTheme.Spacing.lg)
                            .padding(.vertical, NeoTrixTheme.Spacing.sm)
                            .background(NeoTrixTheme.Colors.accent)
                            .foregroundColor(.white)
                            .clipShape(Capsule())
                    }
                }
                .padding(.horizontal, 24)
                
            case .image:
                VStack(spacing: 12) {
                    Image(systemName: "photo.fill")
                        .font(.system(size: 72))
                        .foregroundColor(.white.opacity(0.9))
                    Text("Image story")
                        .font(NeoTrixTheme.Fonts.subheadline)
                        .foregroundColor(.white.opacity(0.8))
                }
                
            case .video:
                VStack(spacing: 12) {
                    Image(systemName: "play.rectangle.fill")
                        .font(.system(size: 72))
                        .foregroundColor(.white.opacity(0.9))
                    Text("Video story")
                        .font(NeoTrixTheme.Fonts.subheadline)
                        .foregroundColor(.white.opacity(0.8))
                }
            }
            
            if !content.caption.isEmpty {
                Text(content.caption)
                    .font(NeoTrixTheme.Fonts.caption)
                    .foregroundColor(.white.opacity(0.7))
            }
        }
    }
}

// MARK: - Story Composer（融合: 官方 Stories 发布 + AI 摘要）

public struct StoryComposerView: View {
    @Environment(\.dismiss) private var dismiss
    let onPublish: (String, Bool) -> Void
    @State private var caption = ""
    @State private var isCloseFriends = false
    
    public init(onPublish: @escaping (String, Bool) -> Void) {
        self.onPublish = onPublish
    }
    
    public var body: some View {
        NavigationStack {
            VStack(spacing: 16) {
                // 预览环
                ZStack {
                    Circle()
                        .fill(NeoTrixTheme.Gradients.story)
                        .frame(width: 96, height: 96)
                    
                    Text(String(caption.prefix(1)).uppercased())
                        .font(.largeTitle.bold())
                        .foregroundColor(.white)
                }
                .padding(.top, 24)
                
                // 文案输入
                TextField("What's on your mind?", text: $caption, axis: .vertical)
                    .textFieldStyle(.roundedBorder)
                    .lineLimit(3...6)
                    .padding(.horizontal)
                
                // 亲密好友开关
                Toggle(isOn: $isCloseFriends) {
                    Label("Close Friends", systemImage: "person.2.fill")
                        .foregroundColor(.primary)
                }
                .padding(.horizontal)
                .neoTrixCard()
                
                Spacer()
            }
            .navigationTitle("New Story")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Publish") {
                        onPublish(caption, isCloseFriends)
                        dismiss()
                    }
                    .disabled(caption.trimmingCharacters(in: .whitespaces).isEmpty)
                }
                #else
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Publish") {
                        onPublish(caption, isCloseFriends)
                        dismiss()
                    }
                    .disabled(caption.trimmingCharacters(in: .whitespaces).isEmpty)
                }
                #endif
            }
        }
    }
}

// MARK: - Stories List
// 注: StoriesListView 已删除（Dark Forest）— Chats Tab 使用更轻量的 StoriesBar
// （StoriesBar 位于 ChatListUI.swift），本视图无消费者且功能重复。