import SwiftUI

struct SettingsView: View {
    let database: DatabaseManager
    let syncEngine: SyncEngine
    let reachability: ReachabilityMonitor

    @State private var serverHost: String = SyncState.serverHost ?? ""
    @State private var apiKey: String = SyncState.apiKey ?? ""
    @State private var showApiKey = false
    @State private var connectionStatus: ConnectionStatus = .unknown
    @State private var documentCount = 0
    @State private var storageUsed: Int64 = 0

    enum ConnectionStatus {
        case unknown, checking, connected, failed
    }

    var body: some View {
        NavigationStack {
            Form {
                serverSection
                authSection
                syncSection
                storageSection
                aboutSection
            }
            .navigationTitle("Settings")
            .task {
                refreshStats()
            }
        }
    }

    // MARK: - Server

    private var serverSection: some View {
        Section("Server") {
            TextField("Tailscale hostname", text: $serverHost)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .keyboardType(.URL)
                .onChange(of: serverHost) { _, newValue in
                    SyncState.serverHost = newValue.isEmpty ? nil : newValue
                    syncEngine.apiClient.refreshConfiguration()
                }

            HStack {
                connectionIndicator
                Text(connectionLabel)
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Spacer()
                Button("Test") {
                    testConnection()
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .disabled(serverHost.isEmpty)
            }
        }
    }

    // MARK: - Auth

    private var authSection: some View {
        Section("Authentication") {
            HStack {
                if showApiKey {
                    TextField("API Key", text: $apiKey)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .font(.system(.body, design: .monospaced))
                } else {
                    SecureField("API Key", text: $apiKey)
                }
                Button {
                    showApiKey.toggle()
                } label: {
                    Image(systemName: showApiKey ? "eye.slash" : "eye")
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
            }
            .onChange(of: apiKey) { _, newValue in
                SyncState.apiKey = newValue.isEmpty ? nil : newValue
                syncEngine.apiClient.refreshConfiguration()
            }
        }
    }

    // MARK: - Sync

    private var syncSection: some View {
        Section("Sync") {
            if let lastSync = syncEngine.lastSyncDate {
                HStack {
                    Text("Last synced")
                    Spacer()
                    Text(lastSync, style: .relative)
                        .foregroundStyle(.secondary)
                }
            }

            if syncEngine.pendingUploadCount > 0 {
                HStack {
                    Text("Pending uploads")
                    Spacer()
                    Text("\(syncEngine.pendingUploadCount)")
                        .foregroundStyle(.orange)
                }
            }

            Button {
                Task { await syncEngine.performSync() }
            } label: {
                HStack {
                    Text("Sync Now")
                    Spacer()
                    if syncEngine.isSyncing {
                        ProgressView()
                            .controlSize(.small)
                    }
                }
            }
            .disabled(syncEngine.isSyncing || serverHost.isEmpty || apiKey.isEmpty)

            if let error = syncEngine.syncError {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
            }
        }
    }

    // MARK: - Storage

    private var storageSection: some View {
        Section("Storage") {
            HStack {
                Text("Documents")
                Spacer()
                Text("\(documentCount)")
                    .foregroundStyle(.secondary)
            }
            HStack {
                Text("Local storage")
                Spacer()
                Text(formatBytes(storageUsed))
                    .foregroundStyle(.secondary)
            }
        }
    }

    // MARK: - About

    private var aboutSection: some View {
        Section("About") {
            HStack {
                Text("Version")
                Spacer()
                Text(Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0.1.0")
                    .foregroundStyle(.secondary)
            }
            HStack {
                Text("Build")
                Spacer()
                Text(Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "1")
                    .foregroundStyle(.secondary)
            }
        }
    }

    // MARK: - Helpers

    @ViewBuilder
    private var connectionIndicator: some View {
        switch connectionStatus {
        case .unknown:
            Circle().fill(.gray).frame(width: 8, height: 8)
        case .checking:
            ProgressView().controlSize(.mini)
        case .connected:
            Circle().fill(.green).frame(width: 8, height: 8)
        case .failed:
            Circle().fill(.red).frame(width: 8, height: 8)
        }
    }

    private var connectionLabel: String {
        switch connectionStatus {
        case .unknown: "Not tested"
        case .checking: "Checking..."
        case .connected: "Connected"
        case .failed: "Unreachable"
        }
    }

    private func testConnection() {
        connectionStatus = .checking
        Task {
            do {
                let ok = try await syncEngine.apiClient.healthCheck()
                connectionStatus = ok ? .connected : .failed
            } catch {
                connectionStatus = .failed
            }
        }
    }

    private func refreshStats() {
        documentCount = (try? database.documentCount()) ?? 0
        storageUsed = ImageStore().totalStorageUsed()
    }

    private func formatBytes(_ bytes: Int64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: bytes)
    }
}
