import SwiftUI

struct SearchView: View {
    let database: DatabaseManager
    let imageStore: ImageStore
    let apiClient: APIClient

    @State private var query = ""
    @State private var results: [Document] = []
    @State private var isSearching = false
    @State private var searchTask: Task<Void, Never>?

    var body: some View {
        NavigationStack {
            Group {
                if query.isEmpty && results.isEmpty {
                    ContentUnavailableView(
                        "Search Documents",
                        systemImage: "magnifyingglass",
                        description: Text("Search by sender, subject, or document text.")
                    )
                } else if results.isEmpty && !isSearching {
                    ContentUnavailableView.search(text: query)
                } else {
                    List(results) { document in
                        NavigationLink(value: document) {
                            DocumentRow(document: document, imageStore: imageStore)
                        }
                    }
                    .navigationDestination(for: Document.self) { document in
                        DocumentDetailView(
                            document: document,
                            database: database,
                            imageStore: imageStore
                        )
                    }
                }
            }
            .navigationTitle("Search")
            .searchable(text: $query, prompt: "Sender, subject, text...")
            .onChange(of: query) { _, newValue in
                performSearch(newValue)
            }
            .overlay {
                if isSearching {
                    ProgressView()
                }
            }
        }
    }

    private func performSearch(_ text: String) {
        searchTask?.cancel()

        guard !text.trimmingCharacters(in: .whitespaces).isEmpty else {
            results = []
            return
        }

        searchTask = Task {
            // Debounce 300ms
            try? await Task.sleep(for: .milliseconds(300))
            guard !Task.isCancelled else { return }

            isSearching = true

            // Local FTS5 search
            let searchEngine = SearchEngine(database: database)
            let localResults = (try? searchEngine.search(query: text)) ?? []

            guard !Task.isCancelled else { return }
            results = localResults
            isSearching = false

            // Also try remote search if server is configured
            if apiClient.isConfigured {
                if let remoteResponse = try? await apiClient.search(query: text) {
                    guard !Task.isCancelled else { return }
                    mergeRemoteResults(remoteResponse.results.map { $0.document.toDocument() })
                }
            }
        }
    }

    private func mergeRemoteResults(_ remote: [Document]) {
        let existingIds = Set(results.map(\.id))
        let newDocs = remote.filter { !existingIds.contains($0.id) }
        if !newDocs.isEmpty {
            results.append(contentsOf: newDocs)
        }
    }
}
