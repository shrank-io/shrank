import Foundation
import GRDB

@Observable
final class DatabaseManager {
    private let dbQueue: DatabaseQueue

    init() throws {
        let documentsURL = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let dbURL = documentsURL.appendingPathComponent("shrank.db")

        var config = Configuration()
        config.foreignKeysEnabled = true
        dbQueue = try DatabaseQueue(path: dbURL.path, configuration: config)

        var migrator = DatabaseMigrator()
        AppMigrations.registerAll(in: &migrator)
        try migrator.migrate(dbQueue)

        // Ensure sync_state row exists
        let deviceId = SyncState.getOrCreateDeviceId()
        try dbQueue.write { db in
            let exists = try SyncState.fetchOne(db, key: deviceId) != nil
            if !exists {
                var state = SyncState(deviceId: deviceId, lastSyncCursor: nil, lastSyncAt: nil)
                try state.insert(db)
            }
        }
    }

    // MARK: - Documents

    func allDocuments(limit: Int = 50, offset: Int = 0) throws -> [Document] {
        try dbQueue.read { db in
            try Document
                .order(Document.Columns.capturedAt.desc)
                .limit(limit, offset: offset)
                .fetchAll(db)
        }
    }

    func document(id: String) throws -> Document? {
        try dbQueue.read { db in
            try Document.fetchOne(db, key: id)
        }
    }

    func documentCount() throws -> Int {
        try dbQueue.read { db in
            try Document.fetchCount(db)
        }
    }

    func insertDocument(_ document: Document) throws {
        try dbQueue.write { db in
            var doc = document
            try doc.insert(db)
        }
    }

    func updateDocument(_ document: Document) throws {
        try dbQueue.write { db in
            var doc = document
            try doc.update(db)
        }
    }

    func upsertDocumentFromSync(_ document: Document) throws {
        try dbQueue.write { db in
            var doc = document
            if try Document.fetchOne(db, key: doc.id) != nil {
                try doc.update(db)
            } else {
                try doc.insert(db)
            }
        }
    }

    func deleteDocument(id: String) throws {
        try dbQueue.write { db in
            _ = try Document.deleteOne(db, key: id)
        }
    }

    // MARK: - Upload Queue

    func pendingUploads() throws -> [UploadQueueItem] {
        try dbQueue.read { db in
            try UploadQueueItem
                .filter(UploadQueueItem.Columns.status == UploadStatus.pending.rawValue
                    || UploadQueueItem.Columns.status == UploadStatus.failed.rawValue)
                .filter(UploadQueueItem.Columns.retryCount < 5)
                .order(UploadQueueItem.Columns.capturedAt.asc)
                .fetchAll(db)
        }
    }

    func enqueueUpload(_ item: UploadQueueItem) throws {
        try dbQueue.write { db in
            var item = item
            try item.insert(db)
        }
    }

    func updateUploadStatus(id: String, status: UploadStatus, error: String? = nil) throws {
        try dbQueue.write { db in
            guard var item = try UploadQueueItem.fetchOne(db, key: id) else { return }
            item.status = status
            item.lastAttemptAt = ISO8601DateFormatter().string(from: Date())
            if status == .failed {
                item.retryCount += 1
                item.errorMessage = error
            }
            try item.update(db)
        }
    }

    func removeConfirmedUploads() throws {
        try dbQueue.write { db in
            _ = try UploadQueueItem
                .filter(UploadQueueItem.Columns.status == UploadStatus.confirmed.rawValue)
                .deleteAll(db)
        }
    }

    func pendingUploadCount() throws -> Int {
        try dbQueue.read { db in
            try UploadQueueItem
                .filter(UploadQueueItem.Columns.status != UploadStatus.confirmed.rawValue)
                .fetchCount(db)
        }
    }

    // MARK: - Sync State

    func syncState() throws -> SyncState? {
        try dbQueue.read { db in
            try SyncState.fetchOne(db)
        }
    }

    func saveSyncCursor(_ cursor: String, at date: Date) throws {
        try dbQueue.write { db in
            guard var state = try SyncState.fetchOne(db) else { return }
            state.lastSyncCursor = cursor
            state.lastSyncAt = ISO8601DateFormatter().string(from: date)
            try state.update(db)
        }
    }

    // MARK: - Entities and Graph

    func upsertEntity(_ entity: Entity) throws {
        try dbQueue.write { db in
            var entity = entity
            let existing = try Entity
                .filter(Entity.Columns.entityType == entity.entityType && Entity.Columns.value == entity.value)
                .fetchOne(db)
            if existing != nil {
                try entity.update(db)
            } else {
                try entity.insert(db)
            }
        }
    }

    func linkDocumentEntity(_ link: DocumentEntity) throws {
        try dbQueue.write { db in
            var link = link
            try link.insert(db, onConflict: .ignore)
        }
    }

    func upsertDocumentEdge(_ edge: DocumentEdge) throws {
        try dbQueue.write { db in
            var edge = edge
            try edge.insert(db, onConflict: .replace)
        }
    }

    func relatedDocuments(for documentId: String) throws -> [Document] {
        try dbQueue.read { db in
            // Direct edges (both directions)
            let sql = """
                SELECT DISTINCT d.* FROM documents d
                WHERE d.id IN (
                    SELECT targetId FROM document_edges WHERE sourceId = ?
                    UNION
                    SELECT sourceId FROM document_edges WHERE targetId = ?
                    UNION
                    SELECT de2.documentId FROM document_entities de1
                    JOIN document_entities de2 ON de2.entityId = de1.entityId
                    WHERE de1.documentId = ? AND de2.documentId != ?
                )
                ORDER BY d.capturedAt DESC
                LIMIT 20
                """
            return try Document.fetchAll(db, sql: sql, arguments: [documentId, documentId, documentId, documentId])
        }
    }

    // MARK: - Observation

    func observeDocuments(onChange: @escaping ([Document]) -> Void) -> DatabaseCancellable {
        let observation = ValueObservation.tracking { db in
            try Document
                .order(Document.Columns.capturedAt.desc)
                .limit(100)
                .fetchAll(db)
        }
        return observation.start(
            in: dbQueue,
            onError: { _ in },
            onChange: onChange
        )
    }

    // MARK: - Raw access for search

    var reader: DatabaseQueue { dbQueue }
}
