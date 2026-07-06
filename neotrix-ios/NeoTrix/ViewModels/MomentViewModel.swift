import Foundation

@MainActor
class MomentViewModel: ObservableObject {
    @Published var moments: [MomentItem] = []
    @Published var socialStatuses: [SocialStatus] = []
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var filterConfig: FilterConfig?

    func loadSocialStatus() async {
        do {
            socialStatuses = try await NeoTrixAPI.shared.socialStatus()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func loadFilterConfig() async {
        do {
            filterConfig = try await NeoTrixAPI.shared.filterConfig()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func scoreMoments(_ submissions: [VideoSubmission]) async {
        isLoading = true
        errorMessage = nil
        do {
            moments = try await NeoTrixAPI.shared.scoreMoments(submissions)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    func submitFeedback(itemId: String, liked: Bool) async {
        do {
            try await NeoTrixAPI.shared.sendFeedback(momentId: itemId, liked: liked)
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    func loginPlatform(_ platform: String, token: String) async {
        do {
            try await NeoTrixAPI.shared.socialLogin(platform: platform, token: token)
            await loadSocialStatus()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
