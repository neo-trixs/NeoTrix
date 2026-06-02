import SwiftUI
import WebKit

struct WebLoginView: View {
    let platform: String
    @Environment(\.dismiss) private var dismiss
    @State private var isLoading = true
    
    private var loginURL: URL? {
        switch platform.lowercased() {
        case "youtube":
            return URL(string: "https://accounts.google.com/Login")
        case "tiktok":
            return URL(string: "https://www.tiktok.com/login")
        case "douyin":
            return URL(string: "https://www.douyin.com/login")
        case "instagram":
            return URL(string: "https://www.instagram.com/accounts/login/")
        case "twitter":
            return URL(string: "https://twitter.com/login")
        case "reddit":
            return URL(string: "https://www.reddit.com/login")
        case "bilibili":
            return URL(string: "https://passport.bilibili.com/login")
        default:
            return nil
        }
    }
    
    var body: some View {
        NavigationStack {
            ZStack {
                if let url = loginURL {
                    WebView(url: url, isLoading: $isLoading)
                } else {
                    Text("Unsupported platform: \(platform)")
                }
                
                if isLoading {
                    ProgressView("Loading \(platform.capitalized)...")
                        .padding()
                        .background(.regularMaterial)
                        .clipShape(RoundedRectangle(cornerRadius: 12))
                }
            }
            .navigationTitle("Login to \(platform.capitalized)")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Save Cookies") {
                        dismiss()
                    }
                }
            }
        }
    }
}

struct WebView: UIViewRepresentable {
    let url: URL
    @Binding var isLoading: Bool
    
    func makeCoordinator() -> Coordinator {
        Coordinator(self)
    }
    
    func makeUIView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration()
        config.websiteDataStore = WKWebsiteDataStore.default()
        let webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = context.coordinator
        webView.load(URLRequest(url: url))
        return webView
    }
    
    func updateUIView(_ webView: WKWebView, context: Context) {}
    
    class Coordinator: NSObject, WKNavigationDelegate {
        var parent: WebView
        init(_ parent: WebView) { self.parent = parent }
        
        func webView(_ webView: WKWebView, didStartProvisionalNavigation nav: WKNavigation!) {
            parent.isLoading = true
        }
        
        func webView(_ webView: WKWebView, didFinishNavigation nav: WKNavigation!) {
            parent.isLoading = false
        }
        
        func webView(_ webView: WKWebView, didFail nav: WKNavigation!, withError error: Error) {
            parent.isLoading = false
        }
    }
}