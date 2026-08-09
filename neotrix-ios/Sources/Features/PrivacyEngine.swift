// PrivacyEngine - 隐私与安全引擎 (Fusion Architecture)
// 融合: Nicegram Ghost Mode + Swiftgram 隐藏 + Turrit Privacy Detection
// 独有: AI 智能通知 (E8 判断重要性) + 隐私评分 + 一键隐私优化

import Foundation
import Combine
import SwiftUI

// MARK: - Privacy Settings

public struct PrivacySettings {
    public var ghostMode: Bool = false          // 不发送已读回执
    public var hideOnlineStatus: Bool = false   // 隐藏在线状态
    public var hideReadReceipts: Bool = false   // 隐藏已读回执
    public var smartNotifications: Bool = true  // AI 智能通知
    public var blockUnknownDMs: Bool = false    // 屏蔽陌生人私信
    public var hidePhoneNumber: Bool = true     // 隐藏手机号
    public var autoLock: Bool = true            // 自动锁定
    public var autoLockDelay: Int = 60          // 秒
}

// MARK: - Privacy Engine

@MainActor
public final class PrivacyEngine: ObservableObject {
    public static let shared = PrivacyEngine()
    
    @Published public var settings = PrivacySettings()
    @Published public var privacyScore: Int = 60
    @Published public var riskFindings: [PrivacyFinding] = []
    
    private let aiHub = AIHub.shared
    
    public init() {
        loadSettings()
        evaluatePrivacy()
    }
    
    private func loadSettings() {
        // Load from UserDefaults
        settings.ghostMode = UserDefaults.standard.bool(forKey: "privacy_ghost_mode")
        settings.hideOnlineStatus = UserDefaults.standard.bool(forKey: "privacy_hide_online")
        settings.hideReadReceipts = UserDefaults.standard.bool(forKey: "privacy_hide_read")
        settings.smartNotifications = UserDefaults.standard.bool(forKey: "privacy_smart_notify")
        settings.blockUnknownDMs = UserDefaults.standard.bool(forKey: "privacy_block_unknown")
        settings.hidePhoneNumber = UserDefaults.standard.bool(forKey: "privacy_hide_phone")
        settings.autoLock = UserDefaults.standard.bool(forKey: "privacy_auto_lock")
    }
    
    /// 保存设置（修复 P0: 此前 private 且仅 applyBestPractices 调用，手动 toggle 不持久化，重启全丢）
    public func saveSettings() {
        UserDefaults.standard.set(settings.ghostMode, forKey: "privacy_ghost_mode")
        UserDefaults.standard.set(settings.hideOnlineStatus, forKey: "privacy_hide_online")
        UserDefaults.standard.set(settings.hideReadReceipts, forKey: "privacy_hide_read")
        UserDefaults.standard.set(settings.smartNotifications, forKey: "privacy_smart_notify")
        UserDefaults.standard.set(settings.blockUnknownDMs, forKey: "privacy_block_unknown")
        UserDefaults.standard.set(settings.hidePhoneNumber, forKey: "privacy_hide_phone")
        UserDefaults.standard.set(settings.autoLock, forKey: "privacy_auto_lock")
    }
    
    // MARK: - Privacy Evaluation
    
    public func evaluatePrivacy() {
        var score = 100.0
        var findings: [PrivacyFinding] = []
        
        if !settings.ghostMode {
            score -= 15
            findings.append(PrivacyFinding(severity: .medium, message: "Ghost Mode off", detail: "Your read receipts are visible to others"))
        }
        if !settings.hideOnlineStatus {
            score -= 10
            findings.append(PrivacyFinding(severity: .low, message: "Online status visible", detail: "Others can see when you're online"))
        }
        if !settings.hidePhoneNumber {
            score -= 20
            findings.append(PrivacyFinding(severity: .high, message: "Phone number exposed", detail: "Your phone number is visible to contacts"))
        }
        if !settings.blockUnknownDMs {
            score -= 15
            findings.append(PrivacyFinding(severity: .medium, message: "Unknown DMs allowed", detail: "Strangers can message you directly"))
        }
        if !settings.autoLock {
            score -= 10
            findings.append(PrivacyFinding(severity: .low, message: "Auto-lock disabled", detail: "App stays unlocked after backgrounding"))
        }
        
        privacyScore = Int(max(0, min(100, score)))
        riskFindings = findings
    }
    
    // MARK: - One-tap Privacy Optimization
    
    public func applyBestPractices() {
        settings.ghostMode = true
        settings.hideOnlineStatus = true
        settings.hideReadReceipts = true
        settings.smartNotifications = true
        settings.blockUnknownDMs = true
        settings.hidePhoneNumber = true
        settings.autoLock = true
        saveSettings()
        evaluatePrivacy()
    }
    
    // MARK: - Smart Notification (AI importance routing)
    
    public func shouldNotify(message: String, fromUnknown: Bool = false) async -> Bool {
        if fromUnknown && settings.blockUnknownDMs {
            return false
        }
        
        if !settings.smartNotifications {
            return true
        }
        
        // AI importance classification via E8
        let result = await aiHub.process(AIHubRequest(text: message, type: .classify))
        return result.confidence > 0.3
    }
}

// MARK: - Privacy Finding

public struct PrivacyFinding: Identifiable {
    public let id = UUID()
    public let severity: Severity
    public let message: String
    public let detail: String
    
    public enum Severity {
        case low, medium, high
        
        public var color: Color {
            switch self {
            case .low: return NeoTrixTheme.Colors.warning
            case .medium: return NeoTrixTheme.Colors.warning
            case .high: return NeoTrixTheme.Colors.danger
            }
        }
    }
}

// MARK: - Privacy Settings View

public struct PrivacySettingsView: View {
    @StateObject private var engine = PrivacyEngine()
    
    public init() {}
    
    public var body: some View {
        Form {
            // Privacy score
            Section {
                VStack(spacing: 12) {
                    ZStack {
                        Circle()
                            .stroke(Color.gray.opacity(0.2), lineWidth: 8)
                        Circle()
                            .trim(from: 0, to: CGFloat(engine.privacyScore) / 100)
                            .stroke(scoreColor, style: StrokeStyle(lineWidth: 8, lineCap: .round))
                            .rotationEffect(.degrees(-90))
                        Text("\(Int(engine.privacyScore))")
                            .font(.title.bold())
                    }
                    .frame(width: 100, height: 100)
                    
                    Text("Privacy Score")
                        .font(.headline)
                    
                    Button("Apply Best Practices") {
                        engine.applyBestPractices()
                    }
                    .buttonStyle(.borderedProminent)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 8)
            }
            
            // Risk findings
            if !engine.riskFindings.isEmpty {
                Section("Findings") {
                    ForEach(engine.riskFindings) { finding in
                        HStack(alignment: .top, spacing: 8) {
                            Circle()
                                .fill(finding.severity.color)
                                .frame(width: 8, height: 8)
                                .padding(.top, 6)
                            VStack(alignment: .leading, spacing: 2) {
                                Text(finding.message)
                                    .font(.subheadline)
                                Text(finding.detail)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                }
            }
            
            Section("Ghost Mode") {
                Toggle("Ghost Mode (no read receipts)", isOn: $engine.settings.ghostMode)
                Toggle("Hide online status", isOn: $engine.settings.hideOnlineStatus)
                Toggle("Hide read receipts", isOn: $engine.settings.hideReadReceipts)
            }
            
            Section("Notifications") {
                Toggle("AI smart notifications", isOn: $engine.settings.smartNotifications)
            }
            
            Section("Security") {
                Toggle("Block unknown DMs", isOn: $engine.settings.blockUnknownDMs)
                Toggle("Hide phone number", isOn: $engine.settings.hidePhoneNumber)
                Toggle("Auto-lock", isOn: $engine.settings.autoLock)
            }
        }
        .navigationTitle("Privacy & Security")
        // 修复 P0: 每个 toggle 变更都持久化 + 重新评分（此前 saveSettings 无人调用，手动开关重启全丢；
        // hideReadReceipts/smartNotifications 此前连 onChange 都没有）
        .onChange(of: engine.settings.ghostMode) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
        .onChange(of: engine.settings.hideOnlineStatus) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
        .onChange(of: engine.settings.hideReadReceipts) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
        .onChange(of: engine.settings.smartNotifications) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
        .onChange(of: engine.settings.hidePhoneNumber) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
        .onChange(of: engine.settings.blockUnknownDMs) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
        .onChange(of: engine.settings.autoLock) { _, _ in
            engine.saveSettings()
            engine.evaluatePrivacy()
        }
    }
    
    private var scoreColor: Color {
        switch engine.privacyScore {
        case 80...: return NeoTrixTheme.Colors.success
        case 50..<80: return NeoTrixTheme.Colors.warning
        default: return NeoTrixTheme.Colors.danger
        }
    }
}