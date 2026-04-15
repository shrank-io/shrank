import Foundation
import GRDB

// MARK: - Entity

struct Entity: Codable, Identifiable {
    var id: String
    var entityType: String
    var value: String
    var displayName: String?
    var metadata: String?
    var createdAt: String
}

extension Entity: FetchableRecord, PersistableRecord, TableRecord {
    static let databaseTableName = "entities"

    enum Columns: String, ColumnExpression {
        case id, entityType, value, displayName, metadata, createdAt
    }
}

// MARK: - Document–Entity link

struct DocumentEntity: Codable {
    var documentId: String
    var entityId: String
    var role: String
    var confidence: Double
}

extension DocumentEntity: FetchableRecord, PersistableRecord, TableRecord {
    static let databaseTableName = "document_entities"

    enum Columns: String, ColumnExpression {
        case documentId, entityId, role, confidence
    }
}

// MARK: - Document–Document edge

struct DocumentEdge: Codable {
    var sourceId: String
    var targetId: String
    var relationType: String
    var confidence: Double
    var inferredBy: String
    var createdAt: String
}

extension DocumentEdge: FetchableRecord, PersistableRecord, TableRecord {
    static let databaseTableName = "document_edges"

    enum Columns: String, ColumnExpression {
        case sourceId, targetId, relationType, confidence, inferredBy, createdAt
    }
}
