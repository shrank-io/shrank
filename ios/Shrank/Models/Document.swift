import Foundation
import GRDB

// MARK: - Document

struct Document: Codable, Identifiable, Hashable {
    var id: String
    var createdAt: String
    var updatedAt: String
    var capturedAt: String
    var syncedAt: String?

    var originalPath: String
    var thumbnailPath: String

    var status: DocumentStatus
    var processingError: String?
    var rawLlmResponse: String?

    var language: String?
    var sender: String?
    var senderNormalized: String?
    var documentDate: String?
    var documentType: String?
    var subject: String?
    var extractedText: String?

    // JSON-encoded columns
    var amounts: String?
    var dates: String?
    var referenceIds: String?
    var tags: String?

    var confidence: Double?
}

enum DocumentStatus: String, Codable, DatabaseValueConvertible {
    case pending
    case processing
    case complete
    case error
}

// MARK: - GRDB conformance

extension Document: FetchableRecord, PersistableRecord, TableRecord {
    static let databaseTableName = "documents"

    enum Columns: String, ColumnExpression {
        case id, createdAt, updatedAt, capturedAt, syncedAt
        case originalPath, thumbnailPath
        case status, processingError, rawLlmResponse
        case language, sender, senderNormalized, documentDate, documentType, subject, extractedText
        case amounts, dates, referenceIds, tags
        case confidence
    }
}

// MARK: - JSON accessors

extension Document {
    var decodedAmounts: [Amount] {
        guard let data = amounts?.data(using: .utf8) else { return [] }
        return (try? JSONDecoder().decode([Amount].self, from: data)) ?? []
    }

    var decodedDates: [DateEntry] {
        guard let data = dates?.data(using: .utf8) else { return [] }
        return (try? JSONDecoder().decode([DateEntry].self, from: data)) ?? []
    }

    var decodedReferenceIds: [ReferenceId] {
        guard let data = referenceIds?.data(using: .utf8) else { return [] }
        return (try? JSONDecoder().decode([ReferenceId].self, from: data)) ?? []
    }

    var decodedTags: [String] {
        guard let data = tags?.data(using: .utf8) else { return [] }
        return (try? JSONDecoder().decode([String].self, from: data)) ?? []
    }
}

// MARK: - Nested types

struct Amount: Codable, Hashable {
    var value: Double
    var currency: String
    var label: String
}

struct DateEntry: Codable, Hashable {
    var date: String
    var label: String
}

struct ReferenceId: Codable, Hashable {
    var type: String
    var value: String
}

// MARK: - Upload Queue Item

struct UploadQueueItem: Codable, Identifiable {
    var id: String
    var localImagePath: String
    var capturedAt: String
    var status: UploadStatus
    var retryCount: Int
    var lastAttemptAt: String?
    var errorMessage: String?
}

enum UploadStatus: String, Codable, DatabaseValueConvertible {
    case pending
    case uploading
    case confirmed
    case failed
}

extension UploadQueueItem: FetchableRecord, PersistableRecord, TableRecord {
    static let databaseTableName = "upload_queue"

    enum Columns: String, ColumnExpression {
        case id, localImagePath, capturedAt, status, retryCount, lastAttemptAt, errorMessage
    }
}
