import Foundation

actor StreamingService {
    static let shared = StreamingService()
    
    private var activeTasks: [UUID: Task<Void, Never>] = [:]
    
    func connectToChatStream(message: String, onChar: @escaping (String) -> Void) -> UUID {
        let id = UUID()
        let task = Task {
            do {
                for try await char in await NeoTrixAPI.shared.chatStream(message: message) {
                    await MainActor.run { onChar(char) }
                }
            } catch {
                await MainActor.run { onChar("[error: \(error.localizedDescription)]") }
            }
        }
        activeTasks[id] = task
        return id
    }
    
    func connectToMomentStream(moments: [VideoSubmission], onItem: @escaping (MomentItem) -> Void) -> UUID {
        let id = UUID()
        let task = Task {
            do {
                for try await item in await NeoTrixAPI.shared.scoreMomentsStream(moments) {
                    await MainActor.run { onItem(item) }
                }
            } catch {}
        }
        activeTasks[id] = task
        return id
    }
    
    func cancel(_ id: UUID) {
        activeTasks[id]?.cancel()
        activeTasks.removeValue(forKey: id)
    }
    
    func cancelAll() {
        for (_, task) in activeTasks {
            task.cancel()
        }
        activeTasks.removeAll()
    }
}
