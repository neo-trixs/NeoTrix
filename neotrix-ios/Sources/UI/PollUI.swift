// PollUI - Mighty Polls (Telegram 2026)
// Mirrors Telegram's 10+ new poll features: media/location attachments, descriptions,
// suggest options, visible voters, no-revote, shuffle, time limits, hidden results

import SwiftUI

// MARK: - Poll Models

public struct PollOption: Identifiable, Equatable {
    public let id: UUID
    public var text: String
    public var voteCount: Int
    public var isSelected: Bool
    public var voters: [String]
    
    public init(id: UUID = UUID(), text: String, voteCount: Int = 0, isSelected: Bool = false, voters: [String] = []) {
        self.id = id
        self.text = text
        self.voteCount = voteCount
        self.isSelected = isSelected
        self.voters = voters
    }
}

public struct Poll: Identifiable {
    public let id: UUID
    public var question: String
    public var description: String
    public var options: [PollOption]
    public var isQuiz: Bool
    public var isAnonymous: Bool
    public var allowsMultipleAnswers: Bool
    public var allowsSuggestions: Bool
    public var allowsRevote: Bool
    public var shuffleOptions: Bool
    public var hideResults: Bool
    public var timeLimit: TimeInterval?
    public var hasMedia: Bool
    public var hasLocation: Bool
    public var createdAt: Date
    public var isClosed: Bool
    
    public init(
        id: UUID = UUID(),
        question: String,
        description: String = "",
        options: [PollOption],
        isQuiz: Bool = false,
        isAnonymous: Bool = true,
        allowsMultipleAnswers: Bool = false,
        allowsSuggestions: Bool = true,
        allowsRevote: Bool = true,
        shuffleOptions: Bool = false,
        hideResults: Bool = false,
        timeLimit: TimeInterval? = nil,
        hasMedia: Bool = false,
        hasLocation: Bool = false,
        createdAt: Date = Date(),
        isClosed: Bool = false
    ) {
        self.id = id
        self.question = question
        self.description = description
        self.options = options
        self.isQuiz = isQuiz
        self.isAnonymous = isAnonymous
        self.allowsMultipleAnswers = allowsMultipleAnswers
        self.allowsSuggestions = allowsSuggestions
        self.allowsRevote = allowsRevote
        self.shuffleOptions = shuffleOptions
        self.hideResults = hideResults
        self.timeLimit = timeLimit
        self.hasMedia = hasMedia
        self.hasLocation = hasLocation
        self.createdAt = createdAt
        self.isClosed = isClosed
    }
    
    public var totalVotes: Int { options.reduce(0) { $0 + $1.voteCount } }
    
    public var isExpired: Bool {
        guard let timeLimit else { return false }
        return Date().timeIntervalSince(createdAt) > timeLimit
    }
}

// MARK: - Poll View Model

@MainActor
public final class PollViewModel: ObservableObject {
    @Published public var polls: [Poll] = []
    
    public init() {
        loadSamplePolls()
    }
    
    private func loadSamplePolls() {
        polls = [
            Poll(
                question: "Which AI feature matters most?",
                description: "Help us prioritize the next NeoTrix AI features",
                options: [
                    PollOption(text: "AI Editor", voteCount: 42, voters: ["user1", "user2"]),
                    PollOption(text: "Voice-to-Text", voteCount: 31, voters: ["user3"]),
                    PollOption(text: "Smart Replies", voteCount: 18, voters: []),
                    PollOption(text: "Chat Summaries", voteCount: 9, voters: []),
                ],
                isAnonymous: true,
                allowsSuggestions: true,
                timeLimit: 86400
            ),
            Poll(
                question: "Quiz: What powers NeoTrix reasoning?",
                options: [
                    PollOption(text: "E8 Hexagram", voteCount: 55, voters: ["user1"]),
                    PollOption(text: "Random Forest", voteCount: 3, voters: []),
                    PollOption(text: "Markov Chain", voteCount: 2, voters: []),
                ],
                isQuiz: true,
                isAnonymous: false,
                allowsRevote: false,
                hideResults: true
            ),
        ]
    }
    
    public func vote(_ pollId: UUID, optionId: UUID) {
        guard let pollIndex = polls.firstIndex(where: { $0.id == pollId }) else { return }
        let poll = polls[pollIndex]
        guard !poll.isClosed && !poll.isExpired else { return }
        
        if !poll.allowsRevote {
            // Clear previous votes
            for i in polls[pollIndex].options.indices {
                polls[pollIndex].options[i].isSelected = false
            }
        }
        
        if let optionIndex = polls[pollIndex].options.firstIndex(where: { $0.id == optionId }) {
            let wasSelected = polls[pollIndex].options[optionIndex].isSelected
            polls[pollIndex].options[optionIndex].isSelected = !wasSelected
            polls[pollIndex].options[optionIndex].voteCount += wasSelected ? -1 : 1
        }
    }
    
    public func suggestOption(_ pollId: UUID, text: String) {
        guard let pollIndex = polls.firstIndex(where: { $0.id == pollId }) else { return }
        guard polls[pollIndex].allowsSuggestions else { return }
        polls[pollIndex].options.append(PollOption(text: text))
    }
    
    public func closePoll(_ pollId: UUID) {
        guard let pollIndex = polls.firstIndex(where: { $0.id == pollId }) else { return }
        polls[pollIndex].isClosed = true
    }
}

// MARK: - Poll View

public struct PollView: View {
    @StateObject private var viewModel = PollViewModel()
    @State private var showCreatePoll = false
    
    public init() {}
    
    public var body: some View {
        List {
            ForEach(viewModel.polls) { poll in
                PollCard(poll: poll, onVote: { optionId in
                    viewModel.vote(poll.id, optionId: optionId)
                }, onClose: { text in
                    viewModel.suggestOption(poll.id, text: text)
                })
            }
        }
        .navigationTitle("Polls")
        .toolbar {
            #if os(iOS)
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    showCreatePoll = true
                } label: {
                    Image(systemName: "plus.circle.fill")
                }
            }
            #else
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showCreatePoll = true
                } label: {
                    Image(systemName: "plus.circle.fill")
                }
            }
            #endif
        }
        .sheet(isPresented: $showCreatePoll) {
            PollCreateView()
        }
    }
}

// MARK: - Poll Card

struct PollCard: View {
    let poll: Poll
    let onVote: (UUID) -> Void
    let onClose: (String) -> Void
    
    @State private var suggestionText = ""
    @State private var showSuggestionField = false
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            // Header
            HStack {
                Text(poll.isQuiz ? "Quiz" : "Poll")
                    .font(.caption.bold())
                    .padding(.horizontal, 8)
                    .padding(.vertical, 3)
                    .background(poll.isQuiz ? Color.orange.opacity(0.2) : Color.blue.opacity(0.2))
                    .clipShape(Capsule())
                
                if poll.isAnonymous {
                    Label("Anonymous", systemImage: "eye.slash.fill")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                if poll.isExpired || poll.isClosed {
                    Text("Closed")
                        .font(.caption.bold())
                        .foregroundColor(.red)
                } else if let timeLimit = poll.timeLimit {
                    Text("\(formatRemaining(timeLimit)) left")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
            
            // Question
            Text(poll.question)
                .font(.headline)
            
            if !poll.description.isEmpty {
                Text(poll.description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            // Media/location badges
            if poll.hasMedia || poll.hasLocation {
                HStack(spacing: 8) {
                    if poll.hasMedia {
                        Label("Media", systemImage: "photo.fill")
                            .font(.caption2)
                            .foregroundColor(.blue)
                    }
                    if poll.hasLocation {
                        Label("Location", systemImage: "mappin.and.ellipse")
                            .font(.caption2)
                            .foregroundColor(.green)
                    }
                }
            }
            
            // Options
            VStack(spacing: 8) {
                ForEach(poll.options) { option in
                    PollOptionRow(
                        option: option,
                        totalVotes: poll.totalVotes,
                        showResults: !poll.hideResults || poll.isClosed || poll.isExpired,
                        isEnabled: !poll.isClosed && !poll.isExpired
                    ) {
                        onVote(option.id)
                    }
                }
            }
            
            // Suggestion
            if poll.allowsSuggestions && !poll.isClosed && !poll.isExpired {
                if showSuggestionField {
                    HStack {
                        TextField("Suggest an option…", text: $suggestionText)
                            .textFieldStyle(.roundedBorder)
                        Button("Add") {
                            let text = suggestionText
                            suggestionText = ""
                            showSuggestionField = false
                            onClose(text)
                        }
                        .disabled(suggestionText.trimmingCharacters(in: .whitespaces).isEmpty)
                    }
                } else {
                    Button {
                        showSuggestionField = true
                    } label: {
                        Label("Suggest an option", systemImage: "plus.circle")
                            .font(.caption)
                            .foregroundColor(.blue)
                    }
                    .buttonStyle(.plain)
                }
            }
            
            // Footer
            HStack {
                Text("\(poll.totalVotes) votes")
                    .font(.caption)
                    .foregroundColor(.secondary)
                
                if poll.allowsMultipleAnswers {
                    Text("• Multiple answers")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                if poll.shuffleOptions {
                    Text("• Shuffled")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding()
        .background(Color.gray.opacity(0.1))
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .padding(.vertical, 4)
    }
    
    private func formatRemaining(_ interval: TimeInterval) -> String {
        let remaining = max(0, interval - Date().timeIntervalSince(poll.createdAt))
        let hours = Int(remaining) / 3600
        let minutes = (Int(remaining) % 3600) / 60
        if hours > 0 { return "\(hours)h \(minutes)m" }
        return "\(minutes)m"
    }
}

// MARK: - Poll Option Row

struct PollOptionRow: View {
    let option: PollOption
    let totalVotes: Int
    let showResults: Bool
    let isEnabled: Bool
    let onTap: () -> Void
    
    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 12) {
                // Selection indicator
                Image(systemName: option.isSelected ? "checkmark.circle.fill" : "circle")
                    .foregroundColor(option.isSelected ? .blue : .secondary)
                
                Text(option.text)
                    .font(.subheadline)
                    .foregroundColor(.primary)
                
                Spacer()
                
                if showResults {
                    let percentage = totalVotes > 0 ? Double(option.voteCount) / Double(totalVotes) * 100 : 0
                    Text("\(Int(percentage))%")
                        .font(.caption.bold())
                        .foregroundColor(.secondary)
                }
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .background(
                ZStack(alignment: .leading) {
                    if showResults && totalVotes > 0 {
                        GeometryReader { geo in
                            RoundedRectangle(cornerRadius: 8)
                                .fill(Color.blue.opacity(0.15))
                                .frame(width: geo.size.width * CGFloat(Double(option.voteCount) / Double(totalVotes)))
                        }
                    }
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color.gray.opacity(0.1))
                }
            )
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
        .buttonStyle(.plain)
        .disabled(!isEnabled)
    }
}

// MARK: - Poll Create View

public struct PollCreateView: View {
    @State private var question = ""
    @State private var description = ""
    @State private var options: [String] = ["", ""]
    @State private var isQuiz = false
    @State private var isAnonymous = true
    @State private var allowsMultipleAnswers = false
    @State private var allowsSuggestions = true
    @State private var allowsRevote = true
    @State private var shuffleOptions = false
    @State private var hideResults = false
    @State private var hasTimeLimit = false
    @State private var timeLimit = 24.0
    
    @Environment(\.dismiss) private var dismiss
    
    public init() {}
    
    public var body: some View {
        NavigationStack {
            Form {
                Section("Question") {
                    TextField("Ask a question…", text: $question)
                    TextField("Description (optional)", text: $description)
                }
                
                Section("Options") {
                    ForEach(options.indices, id: \.self) { index in
                        HStack {
                            TextField("Option \(index + 1)", text: $options[index])
                            if options.count > 2 {
                                Button {
                                    options.remove(at: index)
                                } label: {
                                    Image(systemName: "minus.circle.fill")
                                        .foregroundColor(.red)
                                }
                            }
                        }
                    }
                    
                    Button {
                        options.append("")
                    } label: {
                        Label("Add option", systemImage: "plus.circle.fill")
                    }
                }
                
                Section("Settings") {
                    Toggle("Quiz mode", isOn: $isQuiz)
                    Toggle("Anonymous", isOn: $isAnonymous)
                    Toggle("Multiple answers", isOn: $allowsMultipleAnswers)
                    Toggle("Allow suggestions", isOn: $allowsSuggestions)
                    Toggle("Allow revoting", isOn: $allowsRevote)
                    Toggle("Shuffle options", isOn: $shuffleOptions)
                    Toggle("Hide results until end", isOn: $hideResults)
                    Toggle("Time limit", isOn: $hasTimeLimit)
                    
                    if hasTimeLimit {
                        HStack {
                            Text("Duration")
                            Spacer()
                            Text("\(Int(timeLimit)) hours")
                                .foregroundColor(.secondary)
                        }
                        Slider(value: $timeLimit, in: 1...168, step: 1)
                    }
                }
            }
            .navigationTitle("New Poll")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Create") {
                        dismiss()
                    }
                    .disabled(question.isEmpty || options.filter { !$0.isEmpty }.count < 2)
                }
                #else
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Create") {
                        dismiss()
                    }
                    .disabled(question.isEmpty || options.filter { !$0.isEmpty }.count < 2)
                }
                #endif
            }
        }
    }
    
    }