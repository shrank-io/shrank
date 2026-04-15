import Foundation

@Observable
final class SyncEngine: @unchecked Sendable {
    let apiClient: APIClient
    let database: DatabaseManager
    let imageStore: ImageStore

    private(set) var isSyncing = false
    private(set) var lastSyncDate: Date?
    private(set) var pendingUploadCount = 0
    private(set) var syncError: String?

    init(apiClient: APIClient, database: DatabaseManager, imageStore: ImageStore) {
        self.apiClient = apiClient
        self.database = database
        self.imageStore = imageStore
        refreshState()
    }

    func refreshState() {
        pendingUploadCount = (try? database.pendingUploadCount()) ?? 0
        if let state = try? database.syncState(),
           let dateStr = state.lastSyncAt {
            lastSyncDate = ISO8601DateFormatter().date(from: dateStr)
        }
    }

    // MARK: - Full sync cycle

    func performSync() async {
        guard !isSyncing else { return }
        guard apiClient.isConfigured else {
            syncError = "Server not configured"
            return
        }

        isSyncing = true
        syncError = nil

        do {
            // Check reachability
            let reachable = try await apiClient.healthCheck()
            guard reachable else {
                syncError = "Server not reachable"
                isSyncing = false
                return
            }

            // Register device if first sync
            let state = try database.syncState()
            if state?.lastSyncCursor == nil {
                let deviceId = SyncState.getOrCreateDeviceId()
                try await apiClient.registerDevice(deviceId: deviceId)
            }

            // Upload pending documents
            await uploadPending()

            // Pull new documents from server
            await pullDelta()

            // Clean up confirmed uploads
            try database.removeConfirmedUploads()
        } catch {
            syncError = error.localizedDescription
        }

        refreshState()
        isSyncing = false
    }

    // MARK: - Upload phase

    private func uploadPending() async {
        guard let items = try? database.pendingUploads() else { return }

        for item in items {
            // Check backoff
            if item.status == .failed, let lastAttempt = item.lastAttemptAt {
                let backoff = backoffInterval(retryCount: item.retryCount)
                if let lastDate = ISO8601DateFormatter().date(from: lastAttempt),
                   Date().timeIntervalSince(lastDate) < backoff {
                    continue // Skip, too soon to retry
                }
            }

            try? database.updateUploadStatus(id: item.id, status: .uploading)
            pendingUploadCount = (try? database.pendingUploadCount()) ?? 0

            do {
                let imageURL = imageStore.originalURL(for: item.id)
                let imageData = try Data(contentsOf: imageURL)
                let capturedAt = ISO8601DateFormatter().date(from: item.capturedAt) ?? Date()
                let deviceId = SyncState.getOrCreateDeviceId()

                _ = try await apiClient.uploadDocument(
                    imageData: imageData,
                    capturedAt: capturedAt,
                    deviceId: deviceId
                )

                try database.updateUploadStatus(id: item.id, status: .confirmed)
            } catch {
                try? database.updateUploadStatus(
                    id: item.id,
                    status: .failed,
                    error: error.localizedDescription
                )
            }
        }
    }

    // MARK: - Pull phase

    private func pullDelta() async {
        let state = try? database.syncState()
        var cursor = state?.lastSyncCursor

        while true {
            do {
                let response = try await apiClient.syncDelta(since: cursor)

                for doc in response.documents {
                    try database.upsertDocumentFromSync(doc)

                    // Download thumbnail if we don't have it
                    if !imageStore.thumbnailExists(for: doc.id) {
                        if let thumbData = try? await apiClient.downloadThumbnail(documentId: doc.id) {
                            _ = try? imageStore.saveThumbnail(imageData: thumbData, id: doc.id)
                        }
                    }
                }

                if let nextCursor = response.nextCursor {
                    try database.saveSyncCursor(nextCursor, at: Date())
                    cursor = nextCursor
                } else {
                    // No more pages — save final cursor from last document or current cursor
                    if let lastDoc = response.documents.last {
                        try database.saveSyncCursor(lastDoc.id, at: Date())
                    }
                    break
                }

                // Empty page means we're done
                if response.documents.isEmpty {
                    break
                }
            } catch {
                syncError = error.localizedDescription
                break
            }
        }
    }

    // MARK: - Helpers

    private func backoffInterval(retryCount: Int) -> TimeInterval {
        min(pow(2, Double(retryCount)) * 5, 300)
    }
}
