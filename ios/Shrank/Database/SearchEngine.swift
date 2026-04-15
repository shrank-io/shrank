import Foundation
import GRDB

struct SearchEngine {
    private let dbQueue: DatabaseQueue

    init(database: DatabaseManager) {
        self.dbQueue = database.reader
    }

    func search(query: String, limit: Int = 20, offset: Int = 0) throws -> [Document] {
        let sanitized = sanitizeQuery(query)
        guard !sanitized.isEmpty else { return [] }

        return try dbQueue.read { db in
            let sql = """
                SELECT d.* FROM documents d
                JOIN documents_fts fts ON fts.rowid = d.rowid
                WHERE documents_fts MATCH ?
                ORDER BY bm25(documents_fts)
                LIMIT ? OFFSET ?
                """
            return try Document.fetchAll(db, sql: sql, arguments: [sanitized, limit, offset])
        }
    }

    func searchCount(query: String) throws -> Int {
        let sanitized = sanitizeQuery(query)
        guard !sanitized.isEmpty else { return 0 }

        return try dbQueue.read { db in
            let sql = """
                SELECT COUNT(*) FROM documents d
                JOIN documents_fts fts ON fts.rowid = d.rowid
                WHERE documents_fts MATCH ?
                """
            return try Int.fetchOne(db, sql: sql, arguments: [sanitized]) ?? 0
        }
    }

    private func sanitizeQuery(_ query: String) -> String {
        // Remove FTS5 special characters that could cause syntax errors,
        // then prefix-match each term for incremental search
        let cleaned = query
            .replacingOccurrences(of: "\"", with: "")
            .replacingOccurrences(of: "'", with: "")
            .replacingOccurrences(of: "*", with: "")
            .replacingOccurrences(of: "(", with: "")
            .replacingOccurrences(of: ")", with: "")
            .trimmingCharacters(in: .whitespacesAndNewlines)

        guard !cleaned.isEmpty else { return "" }

        // Split into terms and add prefix matching
        let terms = cleaned.components(separatedBy: .whitespaces).filter { !$0.isEmpty }
        return terms.map { "\($0)*" }.joined(separator: " ")
    }
}
