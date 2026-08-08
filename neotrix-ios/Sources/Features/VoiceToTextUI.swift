// VoiceToTextUI - 语音转文字 (Fusion Architecture)
// 融合: Swiftgram 免费 Voice-to-Text + Turrit 语音输入 + Whisper
// 独有: AI 语音总结 (要点提取) + 免费 (非 Premium 限制)

import SwiftUI
import AVFoundation
import NeoTrixFFI

// MARK: - Voice To Text Manager

@MainActor
public final class VoiceToTextManager: ObservableObject {
    @Published public var isRecording = false
    @Published public var isTranscribing = false
    @Published public var transcript = ""
    @Published public var summary: String?
    @Published public var lastError: String?
    @Published public var recordingDuration: TimeInterval = 0
    
    private let aiHub = AIHub.shared
    private var audioRecorder: AVAudioRecorder?
    private var timer: Timer?
    private var audioURL: URL?
    
    public init() {}
    
    // MARK: - Recording
    
    public func startRecording() {
        #if os(iOS)
        let session = AVAudioSession.sharedInstance()
        do {
            try session.setCategory(.playAndRecord, mode: .default)
            try session.setActive(true)
            
            let url = FileManager.default.temporaryDirectory.appendingPathComponent("voice_\(UUID().uuidString).m4a")
            let settings: [String: Any] = [
                AVFormatIDKey: Int(kAudioFormatMPEG4AAC),
                AVSampleRateKey: 44100,
                AVNumberOfChannelsKey: 1,
                AVEncoderAudioQualityKey: AVAudioQuality.high.rawValue
            ]
            
            audioRecorder = try AVAudioRecorder(url: url, settings: settings)
            audioRecorder?.record()
            audioURL = url
            isRecording = true
            recordingDuration = 0
            
            timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
                self?.recordingDuration += 1
            }
        } catch {
            lastError = "Recording unavailable: \(error.localizedDescription)"
        }
        #else
        // macOS: recording requires microphone permission; simulate for now
        isRecording = true
        recordingDuration = 0
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
            self?.recordingDuration += 1
        }
        #endif
    }
    
    public func stopRecording() {
        audioRecorder?.stop()
        timer?.invalidate()
        timer = nil
        isRecording = false
        transcribe()
    }
    
    // MARK: - Transcription
    
    private func transcribe() {
        guard let url = audioURL else { return }
        isTranscribing = true
        
        // Note: Full Whisper integration requires the Rust core speech module.
        // For now, simulate transcription with a placeholder + AI enhancement.
        Task {
            // Placeholder: real transcription would call Whisper via FFI
            transcript = "Voice message transcribed (Whisper integration pending)"
            
            // AI summary of the transcript
            let result = await aiHub.process(AIHubRequest(text: transcript, type: .summarize))
            summary = result.text
            isTranscribing = false
        }
    }
    
    public func cancelRecording() {
        audioRecorder?.stop()
        timer?.invalidate()
        timer = nil
        isRecording = false
        audioURL = nil
    }
}

// MARK: - Voice To Text View

public struct VoiceToTextView: View {
    @StateObject private var manager = VoiceToTextManager()
    @State private var showTranscript = false
    
    public init() {}
    
    public var body: some View {
        VStack(spacing: 24) {
            // Recording indicator
            ZStack {
                Circle()
                    .fill(manager.isRecording ? Color.red.opacity(0.15) : Color.blue.opacity(0.1))
                    .frame(width: 160, height: 160)
                
                Circle()
                    .stroke(manager.isRecording ? Color.red : Color.blue, lineWidth: 3)
                    .frame(width: 160, height: 160)
                
                Image(systemName: manager.isRecording ? "waveform" : "mic.fill")
                    .font(.system(size: 48))
                    .foregroundColor(manager.isRecording ? .red : .blue)
            }
            .overlay(
                Group {
                    if manager.isRecording {
                        Text(formatDuration(manager.recordingDuration))
                            .font(.caption.monospacedDigit())
                            .padding(.top, 190)
                    }
                }
            )
            
            Text(manager.isRecording ? "Recording… tap to stop" : "Tap to start recording")
                .font(.headline)
                .foregroundColor(.secondary)
            
            if manager.isTranscribing {
                ProgressView("Transcribing…")
            }
            
            if let summary = manager.summary {
                VStack(alignment: .leading, spacing: 8) {
                    Label("AI Summary", systemImage: "text.alignleft")
                        .font(.headline)
                    Text(summary)
                        .font(.body)
                        .padding()
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(Color.blue.opacity(0.1))
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
                .padding(.horizontal)
            }
            
            if let error = manager.lastError {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
            }
            
            Spacer()
        }
        .padding(.top, 60)
        .navigationTitle("Voice to Text")
        .onTapGesture {
            if manager.isRecording {
                manager.stopRecording()
            } else {
                manager.startRecording()
            }
        }
    }
    
    private var secondary: Color { .secondary }
    
    private func formatDuration(_ interval: TimeInterval) -> String {
        let minutes = Int(interval) / 60
        let seconds = Int(interval) % 60
        return String(format: "%02d:%02d", minutes, seconds)
    }
}