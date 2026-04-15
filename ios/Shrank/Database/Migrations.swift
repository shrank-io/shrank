import Foundation
import GRDB

enum AppMigrations {
    static func registerAll(in migrator: inout DatabaseMigrator) {
        migrator.registerMigration("v1-schema") { db in
            // Documents table
            try db.execute(sql: """
                CREATE TABLE documents (
                    id              TEXT PRIMARY KEY,
                    createdAt       TEXT NOT NULL,
                    updatedAt       TEXT NOT NULL,
                    capturedAt      TEXT NOT NULL,
                    syncedAt        TEXT,

                    originalPath    TEXT NOT NULL,
                    thumbnailPath   TEXT NOT NULL,

                    status          TEXT NOT NULL DEFAULT 'pending',
                    processingError TEXT,
                    rawLlmResponse  TEXT,

                    language        TEXT,
                    sender          TEXT,
                    senderNormalized TEXT,
                    documentDate    TEXT,
                    documentType    TEXT,
                    subject         TEXT,
                    extractedText   TEXT,

                    amounts         TEXT,
                    dates           TEXT,
                    referenceIds    TEXT,
                    tags            TEXT,

                    confidence      REAL
                )
                """)

            // FTS5 full-text search index
            try db.execute(sql: """
                CREATE VIRTUAL TABLE documents_fts USING fts5(
                    sender,
                    subject,
                    extractedText,
                    tags,
                    content='documents',
                    content_rowid='rowid',
                    tokenize='unicode61 remove_diacritics 2'
                )
                """)

            // FTS5 sync triggers
            try db.execute(sql: """
                CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
                    INSERT INTO documents_fts(rowid, sender, subject, extractedText, tags)
                    VALUES (new.rowid, new.sender, new.subject, new.extractedText, new.tags);
                END
                """)

            try db.execute(sql: """
                CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
                    INSERT INTO documents_fts(documents_fts, rowid, sender, subject, extractedText, tags)
                    VALUES ('delete', old.rowid, old.sender, old.subject, old.extractedText, old.tags);
                END
                """)

            try db.execute(sql: """
                CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
                    INSERT INTO documents_fts(documents_fts, rowid, sender, subject, extractedText, tags)
                    VALUES ('delete', old.rowid, old.sender, old.subject, old.extractedText, old.tags);
                    INSERT INTO documents_fts(rowid, sender, subject, extractedText, tags)
                    VALUES (new.rowid, new.sender, new.subject, new.extractedText, new.tags);
                END
                """)

            // Entities table
            try db.execute(sql: """
                CREATE TABLE entities (
                    id              TEXT PRIMARY KEY,
                    entityType      TEXT NOT NULL,
                    value           TEXT NOT NULL,
                    displayName     TEXT,
                    metadata        TEXT,
                    createdAt       TEXT NOT NULL,
                    UNIQUE(entityType, value)
                )
                """)

            // Document-entity links
            try db.execute(sql: """
                CREATE TABLE document_entities (
                    documentId      TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                    entityId        TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
                    role            TEXT NOT NULL,
                    confidence      REAL DEFAULT 1.0,
                    PRIMARY KEY (documentId, entityId, role)
                )
                """)

            // Document-document edges
            try db.execute(sql: """
                CREATE TABLE document_edges (
                    sourceId        TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                    targetId        TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                    relationType    TEXT NOT NULL,
                    confidence      REAL DEFAULT 1.0,
                    inferredBy      TEXT NOT NULL,
                    createdAt       TEXT NOT NULL,
                    PRIMARY KEY (sourceId, targetId, relationType)
                )
                """)

            // Graph indexes
            try db.execute(sql: "CREATE INDEX idx_doc_entities_entity ON document_entities(entityId)")
            try db.execute(sql: "CREATE INDEX idx_doc_entities_doc ON document_entities(documentId)")
            try db.execute(sql: "CREATE INDEX idx_doc_edges_source ON document_edges(sourceId)")
            try db.execute(sql: "CREATE INDEX idx_doc_edges_target ON document_edges(targetId)")

            // Upload queue
            try db.execute(sql: """
                CREATE TABLE upload_queue (
                    id              TEXT PRIMARY KEY,
                    localImagePath  TEXT NOT NULL,
                    capturedAt      TEXT NOT NULL,
                    status          TEXT NOT NULL DEFAULT 'pending',
                    retryCount      INTEGER DEFAULT 0,
                    lastAttemptAt   TEXT,
                    errorMessage    TEXT
                )
                """)

            // Sync state (single-row table)
            try db.execute(sql: """
                CREATE TABLE sync_state (
                    deviceId        TEXT PRIMARY KEY,
                    lastSyncCursor  TEXT,
                    lastSyncAt      TEXT
                )
                """)

            // Document indexes for common queries
            try db.execute(sql: "CREATE INDEX idx_documents_captured ON documents(capturedAt)")
            try db.execute(sql: "CREATE INDEX idx_documents_status ON documents(status)")
            try db.execute(sql: "CREATE INDEX idx_documents_sender ON documents(senderNormalized)")
            try db.execute(sql: "CREATE INDEX idx_upload_queue_status ON upload_queue(status)")
        }
    }
}
