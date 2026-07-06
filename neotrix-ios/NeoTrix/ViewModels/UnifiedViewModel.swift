import Foundation

enum UnifiedEvent: Identifiable {
    case chat(ChatMessage)
    case moment(MomentItem)
    case system(String)

    var id: String {
        switch self {
        case .chat(let m): return "chat-\(m.id)"
        case .moment(let m): return "moment-\(m.id)"
        case .system(let s): return "sys-\(s)"
        }
    }
}

@MainActor
class UnifiedViewModel: ObservableObject {
    @Published var events: [UnifiedEvent] = []
    @Published var isStreaming = false
    @Published var filterKeyword: String = ""

    private var streamTaskId: UUID?

    func pushEvent(_ event: UnifiedEvent) {
        events.append(event)
        if events.count > 200 {
            events = Array(events.suffix(150))
        }
    }

    func startMomentStream(_ submissions: [VideoSubmission]) {
        isStreaming = true
        streamTaskId = StreamingService.shared.connectToMomentStream(moments: submissions) { [weak self] item in
            self?.pushEvent(.moment(item))
        }
    }

    func stopStream() {
        if let id = streamTaskId {
            StreamingService.shared.cancel(id)
            streamTaskId = nil
        }
        isStreaming = false
    }

    func clearEvents() {
        events.removeAll()
    }

    var filteredEvents: [UnifiedEvent] {
        guard !filterKeyword.isEmpty else { return events }
        return events.filter { event in
            switch event {
            case .chat(let m):
                return m.content.localizedCaseInsensitiveContains(filterKeyword)
            case .moment(let m):
                return m.title.localizedCaseInsensitiveContains(filterKeyword) ||
                       (m.author?.localizedCaseInsensitiveContains(filterKeyword) ?? false)
            case .system(let s):
                return s.localizedCaseInsensitiveContains(filterKeyword)
            }
        }
    }
}
