import Foundation
import GRDB

struct SyncState: Codable {
    static let deviceIdKey = "shrank_device_id"
    static let serverHostKey = "shrank_server_host"
    static let apiKeyKeychainKey = "api_key"

    var deviceId: String
    var lastSyncCursor: String?
    var lastSyncAt: String?

    // Persisted in SQLite as a single-row table
    static let databaseTableName = "sync_state"
}

extension SyncState: FetchableRecord, PersistableRecord, TableRecord {
    enum Columns: String, ColumnExpression {
        case deviceId, lastSyncCursor, lastSyncAt
    }
}

// MARK: - Convenience

extension SyncState {
    static func getOrCreateDeviceId() -> String {
        if let existing = UserDefaults.standard.string(forKey: deviceIdKey) {
            return existing
        }
        let newId = ULID.generate()
        UserDefaults.standard.set(newId, forKey: deviceIdKey)
        return newId
    }

    static var serverHost: String? {
        get { UserDefaults.standard.string(forKey: serverHostKey) }
        set { UserDefaults.standard.set(newValue, forKey: serverHostKey) }
    }

    static var apiKey: String? {
        get { KeychainHelper.loadString(key: apiKeyKeychainKey) }
        set {
            if let value = newValue {
                KeychainHelper.save(key: apiKeyKeychainKey, string: value)
            } else {
                KeychainHelper.delete(key: apiKeyKeychainKey)
            }
        }
    }
}
