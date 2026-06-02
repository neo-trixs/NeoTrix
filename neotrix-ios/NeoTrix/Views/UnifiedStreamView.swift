import SwiftUI

struct UnifiedStreamView: View {
    @StateObject private var viewModel = UnifiedViewModel()
    @State private var showImport = false
    
    var body: some View {
        VStack(spacing: 0) {
            HStack {
                TextField("Filter...", text: $viewModel.filterKeyword)
                    .textFieldStyle(.roundedBorder)
                
                Button(viewModel.isStreaming ? "Stop" : "Stream") {
                    if viewModel.isStreaming {
                        viewModel.stopStream()
                    } else {
                        startDemoStream()
                    }
                }
                .buttonStyle(.borderedProminent)
                .tint(viewModel.isStreaming ? .red : .blue)
            }
            .padding()
            
            List {
                ForEach(viewModel.filteredEvents) { event in
                    switch event {
                    case .chat(let msg):
                        UnifiedChatRow(message: msg)
                    case .moment(let item):
                        UnifiedMomentRow(item: item)
                    case .system(let text):
                        Label(text, systemImage: "info.circle")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .listStyle(.plain)
        }
        .navigationTitle("Unified Stream")
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button("Clear") {
                    viewModel.clearEvents()
                }
            }
        }
    }
    
    private func startDemoStream() {
        viewModel.pushEvent(.system("Starting unified stream..."))
        let submissions = [
            VideoSubmission(id: "dummy1", title: "Sample Video 1", description: "A test video", platform: "youtube", url: "https://youtube.com/watch?v=test1", thumbnailUrl: nil, durationSecs: 120, viewCount: 1000, likeCount: 50, commentCount: 5, author: "TestChannel", publishedAt: nil),
            VideoSubmission(id: "dummy2", title: "Sample Video 2", description: "Another test", platform: "tiktok", url: "https://tiktok.com/@user/video/test2", thumbnailUrl: nil, durationSecs: 30, viewCount: 5000, likeCount: 200, commentCount: 10, author: "Creator", publishedAt: nil)
        ]
        viewModel.startMomentStream(submissions)
    }
}

struct UnifiedChatRow: View {
    let message: ChatMessage
    
    var body: some View {
        HStack {
            Image(systemName: message.sender == .user ? "person.circle" : "brain.head.profile")
                .foregroundColor(message.sender == .user ? .blue : .green)
            Text(message.content)
                .font(.subheadline)
                .lineLimit(2)
            Spacer()
            Text(message.timestamp, style: .time)
                .font(.caption2)
                .foregroundColor(.secondary)
        }
    }
}

struct UnifiedMomentRow: View {
    let item: MomentItem
    
    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(item.title)
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .lineLimit(1)
                Text("\(item.platform.capitalized) · \(item.author ?? "Unknown")")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
            Text(String(format: "%.2f", item.score))
                .font(.caption)
                .fontWeight(.bold)
                .foregroundColor(item.score > 0.6 ? .green : .orange)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Color(.systemGray6))
                .clipShape(Capsule())
        }
    }
}