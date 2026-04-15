import SwiftUI
import BackgroundTasks

@main
struct ShrankApp: App {
    @State private var database: DatabaseManager?
    @State private var imageStore = ImageStore()
    @State private var syncEngine: SyncEngine?
    @State private var reachability: ReachabilityMonitor?
    @State private var initError: String?

    @Environment(\.scenePhase) private var scenePhase

    static let bgSyncTaskId = "io.shrank.sync"

    var body: some Scene {
        WindowGroup {
            Group {
                if let database, let syncEngine, let reachability {
                    ContentView(
                        database: database,
                        imageStore: imageStore,
                        syncEngine: syncEngine,
                        reachability: reachability
                    )
                } else if let initError {
                    ContentUnavailableView(
                        "Database Error",
                        systemImage: "exclamationmark.triangle",
                        description: Text(initError)
                    )
                } else {
                    ProgressView("Loading...")
                }
            }
            .task {
                initializeApp()
            }
            .onChange(of: scenePhase) { _, newPhase in
                if newPhase == .background {
                    scheduleBackgroundSync()
                }
            }
        }
    }

    private func initializeApp() {
        do {
            let db = try DatabaseManager()
            let apiClient = APIClient()
            let sync = SyncEngine(apiClient: apiClient, database: db, imageStore: imageStore)
            let reach = ReachabilityMonitor(apiClient: apiClient)

            self.database = db
            self.syncEngine = sync
            self.reachability = reach

            registerBackgroundTasks()
        } catch {
            self.initError = error.localizedDescription
        }
    }

    // MARK: - Background sync

    private func registerBackgroundTasks() {
        BGTaskScheduler.shared.register(forTaskWithIdentifier: Self.bgSyncTaskId, using: nil) { task in
            guard let task = task as? BGProcessingTask else { return }
            handleBackgroundSync(task)
        }
    }

    private func handleBackgroundSync(_ task: BGProcessingTask) {
        guard let database, let syncEngine else {
            task.setTaskCompleted(success: false)
            return
        }

        let syncTask = Task {
            await syncEngine.performSync()
            task.setTaskCompleted(success: syncEngine.syncError == nil)
        }

        task.expirationHandler = {
            syncTask.cancel()
        }

        scheduleBackgroundSync()
    }

    private func scheduleBackgroundSync() {
        let request = BGProcessingTaskRequest(identifier: Self.bgSyncTaskId)
        request.requiresNetworkConnectivity = true
        request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60)

        do {
            try BGTaskScheduler.shared.submit(request)
        } catch {
            // Background task scheduling may fail on simulator or when not permitted
        }
    }
}
