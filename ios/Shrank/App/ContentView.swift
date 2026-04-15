import SwiftUI

struct ContentView: View {
    let database: DatabaseManager
    let imageStore: ImageStore
    let syncEngine: SyncEngine
    let reachability: ReachabilityMonitor

    var body: some View {
        TabView {
            DocumentListView(
                database: database,
                imageStore: imageStore,
                syncEngine: syncEngine
            )
            .tabItem {
                Label("Documents", systemImage: "doc.text.fill")
            }

            SearchView(
                database: database,
                imageStore: imageStore,
                apiClient: syncEngine.apiClient
            )
            .tabItem {
                Label("Search", systemImage: "magnifyingglass")
            }

            SettingsView(
                database: database,
                syncEngine: syncEngine,
                reachability: reachability
            )
            .tabItem {
                Label("Settings", systemImage: "gear")
            }
        }
    }
}
