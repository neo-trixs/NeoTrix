import SwiftUI

struct ChatView: View {
    @StateObject private var viewModel = ChatViewModel()
    @State private var inputText = ""
    @State private var useStreaming = true
    
    var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 12) {
                        ForEach(viewModel.messages) { msg in
                            MessageBubble(message: msg)
                                .id(msg.id)
                        }
                        if viewModel.isStreaming {
                            HStack {
                                DotLoader()
                                Spacer()
                            }
                            .padding(.leading, 16)
                        }
                    }
                    .padding()
                }
                .onChange(of: viewModel.messages.count) { _, _ in
                    if let last = viewModel.messages.last {
                        withAnimation { proxy.scrollTo(last.id, anchor: .bottom) }
                    }
                }
            }
            
            HStack(spacing: 8) {
                TextField("Type a message...", text: $inputText)
                    .textFieldStyle(.roundedBorder)
                    .disabled(viewModel.isStreaming)
                
                Button(action: sendMessage) {
                    Image(systemName: viewModel.isStreaming ? "stop.circle" : "arrow.up.circle.fill")
                        .font(.title2)
                }
                .disabled(inputText.trimmingCharacters(in: .whitespaces).isEmpty && !viewModel.isStreaming)
            }
            .padding()
            .background(.bar)
        }
        .navigationTitle("Chat")
    }
    
    private func sendMessage() {
        let text = inputText
        inputText = ""
        Task {
            if useStreaming {
                await viewModel.sendMessageStream(text)
            } else {
                await viewModel.sendMessage(text)
            }
        }
    }
}

struct MessageBubble: View {
    let message: ChatMessage
    
    var body: some View {
        HStack {
            if message.sender == .user { Spacer(minLength: 60) }
            Text(message.content)
                .padding(12)
                .background(message.sender == .user ? Color.blue : Color(.systemGray5))
                .foregroundColor(message.sender == .user ? .white : .primary)
                .clipShape(RoundedRectangle(cornerRadius: 16))
            if message.sender == .agent { Spacer(minLength: 60) }
        }
    }
}

struct DotLoader: View {
    @State private var animating = false
    
    var body: some View {
        HStack(spacing: 4) {
            ForEach(0..<3) { i in
                Circle()
                    .fill(Color.gray)
                    .frame(width: 8, height: 8)
                    .scaleEffect(animating ? 1.0 : 0.5)
                    .animation(.easeInOut(duration: 0.5).repeatForever().delay(Double(i) * 0.2), value: animating)
            }
        }
        .onAppear { animating = true }
    }
}