import SwiftUI

struct SettingsView: View {
    @State private var serverURL = UserDefaults.standard.string(forKey: "server_url") ?? "http://localhost:3000"
    @State private var showCopied = false

    var body: some View {
        Form {
            Section("Server") {
                TextField("Server URL", text: $serverURL)
                    .textContentType(.URL)
                    .autocapitalization(.none)
                    .disableAutocorrection(true)
                    .onChange(of: serverURL) { _, new in
                        UserDefaults.standard.set(new, forKey: "server_url")
                    }
            }

            Section("About") {
                LabeledContent("Version", value: "1.0.0")
                LabeledContent("Platforms", value: "7")
                LabeledContent("Filter Dimensions", value: "6")
            }

            Section("How to Use") {
                VStack(alignment: .leading, spacing: 8) {
                    Text("1. Start the NeoTrix server")
                        .font(.subheadline)
                    Text("2. Set the server URL above")
                        .font(.subheadline)
                    Text("3. Login to social platforms via Moments tab")
                        .font(.subheadline)
                    Text("4. Import content and let the filter optimize your feed")
                        .font(.subheadline)
                    Text("5. View real-time scored stream in the Stream tab")
                        .font(.subheadline)
                }
                .padding(.vertical, 4)
            }

            Section("Build Instructions") {
                VStack(alignment: .leading, spacing: 4) {
                    Text("1. Open this project in Xcode")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("2. Select your Team in Signing & Capabilities")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("3. Build to iOS device or simulator")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.vertical, 4)
            }
        }
        .navigationTitle("Settings")
    }
}
