import Foundation
import Network

@Observable
final class ReachabilityMonitor: @unchecked Sendable {
    private let monitor = NWPathMonitor()
    private let queue = DispatchQueue(label: "io.shrank.reachability")
    private let apiClient: APIClient

    private(set) var isNetworkAvailable = false
    private(set) var isServerReachable = false

    init(apiClient: APIClient) {
        self.apiClient = apiClient
        startMonitoring()
    }

    deinit {
        monitor.cancel()
    }

    private func startMonitoring() {
        monitor.pathUpdateHandler = { [weak self] path in
            guard let self else { return }
            let satisfied = path.status == .satisfied
            Task { @MainActor in
                self.isNetworkAvailable = satisfied
                if satisfied {
                    await self.checkServer()
                } else {
                    self.isServerReachable = false
                }
            }
        }
        monitor.start(queue: queue)
    }

    func checkServer() async {
        guard apiClient.isConfigured else {
            isServerReachable = false
            return
        }
        do {
            isServerReachable = try await apiClient.healthCheck()
        } catch {
            isServerReachable = false
        }
    }
}
