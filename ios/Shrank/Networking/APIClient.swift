import Foundation

// MARK: - Response types

struct UploadResponse: Codable {
    let id: String
    let status: String
}

struct SyncResponse: Codable {
    let documents: [Document]
    let nextCursor: String?
}

struct SearchResult: Codable {
    let document: Document
    let score: Double
    let matchSources: [String]?
}

struct SearchResponse: Codable {
    let results: [SearchResult]
    let total: Int
}

struct HealthResponse: Codable {
    let status: String
}

// MARK: - API Client

final class APIClient: @unchecked Sendable {
    var baseURL: URL?
    var apiKey: String?

    init() {
        if let host = SyncState.serverHost {
            self.baseURL = URL(string: "http://\(host):3420")
        }
        self.apiKey = SyncState.apiKey
    }

    func refreshConfiguration() {
        if let host = SyncState.serverHost {
            self.baseURL = URL(string: "http://\(host):3420")
        }
        self.apiKey = SyncState.apiKey
    }

    var isConfigured: Bool {
        baseURL != nil && apiKey != nil && !(apiKey?.isEmpty ?? true)
    }

    // MARK: - Health

    func healthCheck() async throws -> Bool {
        let (_, response) = try await request(path: "/api/health", method: "GET", timeout: 5)
        return (response as? HTTPURLResponse)?.statusCode == 200
    }

    // MARK: - Upload

    func uploadDocument(imageData: Data, capturedAt: Date, deviceId: String) async throws -> UploadResponse {
        guard let url = url(for: "/api/documents") else { throw APIError.notConfigured }

        let boundary = UUID().uuidString
        var request = URLRequest(url: url, timeoutInterval: 60)
        request.httpMethod = "POST"
        request.setValue("multipart/form-data; boundary=\(boundary)", forHTTPHeaderField: "Content-Type")
        addAuth(to: &request)

        var body = Data()

        // Image field
        body.appendMultipart(boundary: boundary, name: "image", filename: "scan.jpg", mimeType: "image/jpeg", data: imageData)

        // captured_at field
        let dateString = ISO8601DateFormatter().string(from: capturedAt)
        body.appendMultipart(boundary: boundary, name: "captured_at", value: dateString)

        // device_id field
        body.appendMultipart(boundary: boundary, name: "device_id", value: deviceId)

        // Close boundary
        body.append("--\(boundary)--\r\n".data(using: .utf8)!)

        request.httpBody = body

        let (data, response) = try await URLSession.shared.data(for: request)
        try validateResponse(response)
        return try JSONDecoder().decode(UploadResponse.self, from: data)
    }

    // MARK: - Sync

    func registerDevice(deviceId: String) async throws {
        let body = try JSONEncoder().encode(["device_id": deviceId])
        let (_, response) = try await request(path: "/api/sync/register", method: "POST", body: body)
        try validateResponse(response)
    }

    func syncDelta(since cursor: String?, limit: Int = 50) async throws -> SyncResponse {
        var path = "/api/sync?limit=\(limit)"
        if let cursor {
            path += "&since=\(cursor)"
        }
        let (data, response) = try await request(path: path, method: "GET")
        try validateResponse(response)
        return try JSONDecoder().decode(SyncResponse.self, from: data)
    }

    // MARK: - Thumbnails

    func downloadThumbnail(documentId: String) async throws -> Data {
        let (data, response) = try await request(path: "/api/images/thumbnail/\(documentId)", method: "GET")
        try validateResponse(response)
        return data
    }

    // MARK: - Search

    func search(query: String, limit: Int = 20, offset: Int = 0) async throws -> SearchResponse {
        let encoded = query.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? query
        let path = "/api/search?q=\(encoded)&limit=\(limit)&offset=\(offset)"
        let (data, response) = try await request(path: path, method: "GET")
        try validateResponse(response)
        return try JSONDecoder().decode(SearchResponse.self, from: data)
    }

    // MARK: - Private helpers

    private func url(for path: String) -> URL? {
        guard let baseURL else { return nil }
        return URL(string: path, relativeTo: baseURL)
    }

    private func addAuth(to request: inout URLRequest) {
        if let apiKey {
            request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
        }
    }

    private func request(
        path: String,
        method: String,
        body: Data? = nil,
        timeout: TimeInterval = 30
    ) async throws -> (Data, URLResponse) {
        guard let url = url(for: path) else { throw APIError.notConfigured }

        var request = URLRequest(url: url, timeoutInterval: timeout)
        request.httpMethod = method
        addAuth(to: &request)

        if let body {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = body
        }

        return try await URLSession.shared.data(for: request)
    }

    private func validateResponse(_ response: URLResponse) throws {
        guard let http = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        guard (200..<300).contains(http.statusCode) else {
            throw APIError.httpError(http.statusCode)
        }
    }
}

// MARK: - Errors

enum APIError: LocalizedError {
    case notConfigured
    case invalidResponse
    case httpError(Int)

    var errorDescription: String? {
        switch self {
        case .notConfigured: "Server not configured"
        case .invalidResponse: "Invalid server response"
        case .httpError(let code): "Server error (\(code))"
        }
    }
}

// MARK: - Multipart helpers

private extension Data {
    mutating func appendMultipart(boundary: String, name: String, filename: String, mimeType: String, data: Data) {
        append("--\(boundary)\r\n".data(using: .utf8)!)
        append("Content-Disposition: form-data; name=\"\(name)\"; filename=\"\(filename)\"\r\n".data(using: .utf8)!)
        append("Content-Type: \(mimeType)\r\n\r\n".data(using: .utf8)!)
        append(data)
        append("\r\n".data(using: .utf8)!)
    }

    mutating func appendMultipart(boundary: String, name: String, value: String) {
        append("--\(boundary)\r\n".data(using: .utf8)!)
        append("Content-Disposition: form-data; name=\"\(name)\"\r\n\r\n".data(using: .utf8)!)
        append("\(value)\r\n".data(using: .utf8)!)
    }
}
