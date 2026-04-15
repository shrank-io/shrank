import SwiftUI

struct DocumentListView: View {
    let database: DatabaseManager
    let imageStore: ImageStore
    let syncEngine: SyncEngine

    @State private var documents: [Document] = []
    @State private var showCamera = false
    @State private var cancellable: AnyObject?

    var body: some View {
        NavigationStack {
            ZStack(alignment: .bottomTrailing) {
                Group {
                    if documents.isEmpty {
                        emptyState
                    } else {
                        documentList
                    }
                }

                // Floating camera button
                Button {
                    showCamera = true
                } label: {
                    Image(systemName: "camera.fill")
                        .font(.title2)
                        .fontWeight(.semibold)
                        .foregroundStyle(.white)
                        .frame(width: 56, height: 56)
                        .background(.blue, in: Circle())
                        .shadow(radius: 4, y: 2)
                }
                .padding(20)
            }
            .navigationTitle("Documents")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    syncStatusView
                }
            }
            .fullScreenCover(isPresented: $showCamera) {
                CameraView(
                    onCapture: { images in
                        showCamera = false
                        handleCaptures(images)
                    },
                    onCancel: {
                        showCamera = false
                    }
                )
            }
            .task {
                startObserving()
            }
        }
    }

    private var documentList: some View {
        List(documents) { document in
            NavigationLink(value: document) {
                DocumentRow(document: document, imageStore: imageStore)
            }
        }
        .refreshable {
            await syncEngine.performSync()
        }
        .navigationDestination(for: Document.self) { document in
            DocumentDetailView(
                document: document,
                database: database,
                imageStore: imageStore
            )
        }
    }

    private var emptyState: some View {
        ContentUnavailableView {
            Label("No Documents", systemImage: "doc.text.magnifyingglass")
        } description: {
            Text("Tap the camera button to scan your first document.")
        }
    }

    @ViewBuilder
    private var syncStatusView: some View {
        if syncEngine.isSyncing {
            ProgressView()
                .controlSize(.small)
        } else if syncEngine.pendingUploadCount > 0 {
            Label("\(syncEngine.pendingUploadCount)", systemImage: "arrow.up.circle")
                .font(.caption)
                .foregroundStyle(.orange)
        }
    }

    private func startObserving() {
        let observation = database.observeDocuments { docs in
            self.documents = docs
        }
        self.cancellable = observation as AnyObject
    }

    private func handleCaptures(_ images: [UIImage]) {
        for image in images {
            guard let processed = ImageProcessor.process(image: image) else { continue }

            let id = ULID.generate()
            let now = ISO8601DateFormatter().string(from: Date())
            let capturedAtStr = ISO8601DateFormatter().string(from: processed.capturedAt)

            // Save images to disk
            let originalPath = (try? imageStore.saveOriginal(imageData: processed.originalData, id: id)) ?? ""
            let thumbnailPath = (try? imageStore.saveThumbnail(imageData: processed.thumbnailData, id: id)) ?? ""

            // Create local document stub
            let document = Document(
                id: id,
                createdAt: now,
                updatedAt: now,
                capturedAt: capturedAtStr,
                originalPath: originalPath,
                thumbnailPath: thumbnailPath,
                status: .pending
            )
            try? database.insertDocument(document)

            // Enqueue for upload
            let uploadItem = UploadQueueItem(
                id: id,
                localImagePath: originalPath,
                capturedAt: capturedAtStr,
                status: .pending,
                retryCount: 0
            )
            try? database.enqueueUpload(uploadItem)
        }

        // Trigger sync if possible
        Task {
            await syncEngine.performSync()
        }
    }
}
