// PasscodeUI - Telegram passcode & Face ID lock (Premium)
// Mirrors Telegram's PasscodeController + LocalAuth

import SwiftUI
import LocalAuthentication

// MARK: - Passcode Manager

@MainActor
public final class PasscodeManager: ObservableObject {
    @Published public var isLocked = false
    @Published public var isPasscodeEnabled = false
    @Published public var isBiometricsEnabled = false
    @Published public var autoLockInterval: AutoLockInterval = .fiveMinutes
    
    private let core = NeoGramCore.shared
    
    public enum AutoLockInterval: String, CaseIterable {
        case immediately = "Immediately"
        case oneMinute = "1 minute"
        case fiveMinutes = "5 minutes"
        case oneHour = "1 hour"
        case fiveHours = "5 hours"
        
        public var seconds: TimeInterval {
            switch self {
            case .immediately: return 0
            case .oneMinute: return 60
            case .fiveMinutes: return 300
            case .oneHour: return 3600
            case .fiveHours: return 18000
            }
        }
    }
    
    public init() {
        loadPreferences()
    }
    
    private func loadPreferences() {
        isPasscodeEnabled = UserDefaults.standard.bool(forKey: "passcode_enabled")
        isBiometricsEnabled = UserDefaults.standard.bool(forKey: "biometrics_enabled")
        if let raw = UserDefaults.standard.string(forKey: "autolock_interval"),
           let interval = AutoLockInterval(rawValue: raw) {
            autoLockInterval = interval
        }
    }
    
    public func enablePasscode(_ code: String) {
        UserDefaults.standard.set(code, forKey: "passcode")
        UserDefaults.standard.set(true, forKey: "passcode_enabled")
        isPasscodeEnabled = true
    }
    
    public func verifyPasscode(_ code: String) -> Bool {
        let saved = UserDefaults.standard.string(forKey: "passcode") ?? ""
        return code == saved
    }
    
    public func disablePasscode() {
        UserDefaults.standard.set(false, forKey: "passcode_enabled")
        UserDefaults.standard.removeObject(forKey: "passcode")
        isPasscodeEnabled = false
        isLocked = false
    }
    
    public func enableBiometrics() async -> Bool {
        let context = LAContext()
        var error: NSError?
        
        guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
            return false
        }
        
        do {
            let success = try await context.evaluatePolicy(
                .deviceOwnerAuthenticationWithBiometrics,
                localizedReason: "Unlock NeoGram"
            )
            if success {
                isBiometricsEnabled = true
                UserDefaults.standard.set(true, forKey: "biometrics_enabled")
            }
            return success
        } catch {
            return false
        }
    }
    
    public func authenticate() async -> Bool {
        if isBiometricsEnabled {
            let success = await enableBiometrics()
            if success {
                isLocked = false
                return true
            }
        }
        return false
    }
    
    public func lock() {
        isLocked = true
    }
    
    public func unlock() {
        isLocked = false
    }
}

// MARK: - Passcode Lock View

public struct PasscodeLockView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var manager = PasscodeManager()
    
    @State private var enteredCode = ""
    @State private var showError = false
    @State private var isVerifying = false
    
    public var body: some View {
        VStack(spacing: 32) {
            Spacer()
            
            // Lock icon
            Image(systemName: "lock.fill")
                .font(.system(size: 60))
                .foregroundColor(.blue)
            
            Text("Enter Passcode")
                .font(.title2.bold())
            
            // Dots
            HStack(spacing: 16) {
                ForEach(0..<4, id: \.self) { index in
                    Circle()
                        .fill(index < enteredCode.count ? Color.blue : Color.gray.opacity(0.3))
                        .frame(width: 20, height: 20)
                }
            }
            
            if showError {
                Text("Incorrect passcode")
                    .font(.subheadline)
                    .foregroundColor(.red)
            }
            
            // Biometric button
            if manager.isBiometricsEnabled {
                Button {
                    Task {
                        if await manager.authenticate() {
                            dismiss()
                        }
                    }
                } label: {
                    Image(systemName: "faceid")
                        .font(.system(size: 40))
                        .foregroundColor(.blue)
                }
                .padding(.top, 8)
            }
            
            Spacer()
            
            // Number pad
            VStack(spacing: 12) {
                ForEach(0..<3, id: \.self) { row in
                    HStack(spacing: 12) {
                        ForEach(1...3, id: \.self) { col in
                            let number = row * 3 + col
                            NumberButton(number: number) {
                                appendDigit(number)
                            }
                        }
                    }
                }
                
                HStack(spacing: 12) {
                    Button {
                        // Biometric or empty
                    } label: {
                        Color.clear.frame(width: 80, height: 80)
                    }
                    
                    NumberButton(number: 0) {
                        appendDigit(0)
                    }
                    
                    Button {
                        if !enteredCode.isEmpty {
                            enteredCode.removeLast()
                        }
                    } label: {
                        Image(systemName: "delete.left")
                            .font(.title2)
                            .foregroundColor(.primary)
                            .frame(width: 80, height: 80)
                    }
                }
            }
            .padding(.bottom, 40)
        }
        .padding()
        .onChange(of: enteredCode) { _, newValue in
            if newValue.count == 4 {
                verifyCode()
            }
        }
    }
    
    private func appendDigit(_ digit: Int) {
        guard enteredCode.count < 4 else { return }
        enteredCode.append("\(digit)")
    }
    
    private func verifyCode() {
        isVerifying = true
        if manager.verifyPasscode(enteredCode) {
            manager.unlock()
            dismiss()
        } else {
            showError = true
            enteredCode = ""
            isVerifying = false
            DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
                showError = false
            }
        }
    }
}

struct NumberButton: View {
    let number: Int
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            Text("\(number)")
                .font(.title)
                .frame(width: 80, height: 80)
                .background(Color.gray.opacity(0.15))
                .clipShape(Circle())
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Passcode Settings

public struct PasscodeSettingsView: View {
    @StateObject private var manager = PasscodeManager()
    @State private var showSetup = false
    @State private var newPasscode = ""
    @State private var confirmPasscode = ""
    @State private var setupStep = 0
    
    public var body: some View {
        List {
            Section("Lock") {
                Toggle("Passcode Lock", isOn: Binding(
                    get: { manager.isPasscodeEnabled },
                    set: { enabled in
                        if enabled {
                            showSetup = true
                        } else {
                            manager.disablePasscode()
                        }
                    }
                ))
                
                if manager.isPasscodeEnabled {
                    Toggle("Face ID", isOn: Binding(
                        get: { manager.isBiometricsEnabled },
                        set: { enabled in
                            if enabled {
                                Task { _ = await manager.enableBiometrics() }
                            } else {
                                manager.isBiometricsEnabled = false
                                UserDefaults.standard.set(false, forKey: "biometrics_enabled")
                            }
                        }
                    ))
                }
            }
            
            if manager.isPasscodeEnabled {
                Section("Auto-Lock") {
                    Picker("Auto-Lock", selection: $manager.autoLockInterval) {
                        ForEach(PasscodeManager.AutoLockInterval.allCases, id: \.self) { interval in
                            Text(interval.rawValue).tag(interval)
                        }
                    }
                }
            }
        }
        .navigationTitle("Passcode & Face ID")
        .sheet(isPresented: $showSetup) {
            PasscodeSetupView { code in
                manager.enablePasscode(code)
                showSetup = false
            }
        }
    }
}

struct PasscodeSetupView: View {
    @Environment(\.dismiss) private var dismiss
    let onComplete: (String) -> Void
    
    @State private var firstEntry = ""
    @State private var secondEntry = ""
    @State private var step = 0
    @State private var errorMessage: String?
    
    var body: some View {
        VStack(spacing: 32) {
            Spacer()
            
            Text(step == 0 ? "Set Passcode" : "Confirm Passcode")
                .font(.title2.bold())
            
            HStack(spacing: 16) {
                ForEach(0..<4, id: \.self) { index in
                    Circle()
                        .fill(index < currentEntry.count ? Color.blue : Color.secondary)
                        .frame(width: 20, height: 20)
                }
            }
            
            if let errorMessage {
                Text(errorMessage)
                    .foregroundColor(.red)
            }
            
            Spacer()
            
            VStack(spacing: 12) {
                ForEach(0..<3, id: \.self) { row in
                    HStack(spacing: 12) {
                        ForEach(1...3, id: \.self) { col in
                            let number = row * 3 + col
                            NumberButton(number: number) {
                                appendDigit(number)
                            }
                        }
                    }
                }
                
                HStack(spacing: 12) {
                    Color.clear.frame(width: 80, height: 80)
                    NumberButton(number: 0) { appendDigit(0) }
                    Button {
                        if step == 0 {
                            if !firstEntry.isEmpty { firstEntry.removeLast() }
                        } else {
                            if !secondEntry.isEmpty { secondEntry.removeLast() }
                        }
                    } label: {
                        Image(systemName: "delete.left")
                            .font(.title2)
                            .frame(width: 80, height: 80)
                    }
                }
            }
            .padding(.bottom, 40)
        }
        .padding()
    }
    
    private var currentEntry: String {
        step == 0 ? firstEntry : secondEntry
    }
    
    private func appendDigit(_ digit: Int) {
        if currentEntry.count < 4 {
            if step == 0 {
                firstEntry.append("\(digit)")
                if firstEntry.count == 4 {
                    step = 1
                }
            } else {
                secondEntry.append("\(digit)")
                if secondEntry.count == 4 {
                    if firstEntry == secondEntry {
                        onComplete(firstEntry)
                    } else {
                        errorMessage = "Passcodes don't match"
                        firstEntry = ""
                        secondEntry = ""
                        step = 0
                    }
                }
            }
        }
    }
}