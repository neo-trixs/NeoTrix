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
    @Published public var currentStoryIndex: Int = 0
    @Published public var isViewing = false
    
    private let core = NeoGramCore.shared
    
    public init() {
        loadStories()
    }
    
    private func loadStories() {
        stories = [
            StoryItem(id: 1, authorName: "NeoTrix AI", authorInitial: "N", avatarColor: .purple, isSeen: false, isPremium: true, isCloseFriends: false, timestamp: Date(), duration: 5),
            StoryItem(id: 2, authorName: "Alice", authorInitial: "A", avatarColor: .pink, isSeen: false, isPremium: false, isCloseFriends: true, timestamp: Date().addingTimeInterval(-600), duration: 8),
            StoryItem(id: 3, authorName: "Bob", authorInitial: "B", avatarColor: .blue, isSeen: true, isPremium: false, isCloseFriends: false, timestamp: Date().addingTimeInterval(-1800), duration: 5),
            StoryItem(id: 4, authorName: "Design", authorInitial: "D", avatarColor: .orange, isSeen: false, isPremium: true, isCloseFriends: false, timestamp: Date().addingTimeInterval(-3600), duration: 15),
        ]
    }
    
    public func markAsSeen(_ storyId: Int64) {
        if let index = stories.firstIndex(where: { $0.id == storyId }) {
            stories[index].isSeen = true
        }
    }
    
    public func nextStory() {
        if currentStoryIndex < stories.count - 1 {
            currentStoryIndex += 1
        } else {
            isViewing = false
        }
    }
    
    public func previousStory() {
        if currentStoryIndex > 0 {
            currentStoryIndex -= 1
        }
    }
    
    public func captureStoryView() {
        // Premium: stealth mode - capture view without notifying
        UserDefaults.standard.set(true, forKey: "story_stealth_mode")
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
                        .stroke(story.isSeen ? AnyShapeStyle(Color.gray.opacity(0.3)) : AnyShapeStyle(gradient), lineWidth: 3)
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
        LinearGradient(
            colors: [.purple, .pink, .orange],
            startPoint: .topLeading,
            endPoint: .bottomTrailing
        )
    }
}

// MARK: - Story Viewer

public struct StoryViewerView: View {
    @Environment(\.dismiss) private var dismiss
    let stories: [StoryItem]
    @State private var currentIndex = 0
    @State private var progress: CGFloat = 0
    @State private var timer: Timer?
    @State private var isPaused = false
    
    public init(stories: [StoryItem]) {
        self.stories = stories
    }
    
    public var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()
            
            if !stories.isEmpty {
                // Story content placeholder
                VStack {
                    Spacer()
                    
                    Text("Story \(currentIndex + 1)")
                        .font(.largeTitle)
                        .foregroundColor(.white)
                    
                    Spacer()
                }
                
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
            startTimer()
        }
        .onDisappear {
            timer?.invalidate()
        }
        .onChange(of: currentIndex) { _, _ in
            startTimer()
        }
    }
    
    private func startTimer() {
        timer?.invalidate()
        progress = 0
        timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { timer in
            if !isPaused {
                progress += 0.02
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

// MARK: - Stories List

public struct StoriesListView: View {
    @StateObject private var viewModel = StoryViewModel()
    @State private var showViewer = false
    @State private var showComposer = false
    
    public init() {}
    
    public var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Stories")
                .font(.title2.bold())
                .padding(.horizontal)
            
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 16) {
                    // My story
                    Button {
                        showComposer = true
                    } label: {
                        VStack(spacing: 6) {
                            ZStack {
                                Circle()
                                    .fill(Color.gray.opacity(0.15))
                                    .frame(width: 64, height: 64)
                                
                                Image(systemName: "plus")
                                    .font(.title2)
                                    .foregroundColor(.secondary)
                            }
                            
                            Text("My Story")
                                .font(.caption)
                        }
                    }
                    .buttonStyle(.plain)
                    
                    ForEach(viewModel.stories) { story in
                        StoryRing(story: story) {
                            viewModel.currentStoryIndex = 0
                            showViewer = true
                        }
                    }
                }
                .padding(.horizontal)
            }
        }
        .padding(.vertical)
        #if os(iOS)
        .fullScreenCover(isPresented: $showViewer) {
            StoryViewerView(stories: viewModel.stories)
        }
        #else
        .sheet(isPresented: $showViewer) {
            StoryViewerView(stories: viewModel.stories)
        }
        #endif
    }
}