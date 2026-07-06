import Foundation

@MainActor
class ChatViewModel: ObservableObject {
    @Published var messages: [ChatMessage] = []
    @Published var isStreaming = false
    @Published var errorMessage: String?

    init() {
        messages.append(ChatMessage(content: "Hello, I'm NeoTrix Agent. How can I help you explore social media today?", sender: .agent))
    }

    func sendMessage(_ text: String) async {
        let trimmed = text.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        messages.append(ChatMessage(content: trimmed, sender: .user))
        isStreaming = true
        errorMessage = nil
        do {
            let reply = try await NeoTrixAPI.shared.chat(message: trimmed)
            messages.append(ChatMessage(content: reply, sender: .agent))
        } catch {
            errorMessage = error.localizedDescription
            messages.append(ChatMessage(content: "Error: \(error.localizedDescription)", sender: .agent))
        }
        isStreaming = false
    }

    func sendMessageStream(_ text: String) async {
        let trimmed = text.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        messages.append(ChatMessage(content: trimmed, sender: .user))
        isStreaming = true
        errorMessage = nil

        let agentMsg = ChatMessage(content: "", sender: .agent)
        messages.append(agentMsg)
        let idx = messages.count - 1

        do {
            for try await char in await NeoTrixAPI.shared.chatStream(message: trimmed) {
                messages[idx].content += String(char)
            }
        } catch {
            errorMessage = error.localizedDescription
            messages[idx].content = "Error: \(error.localizedDescription)"
        }
        isStreaming = false
    }
}
