import SwiftUI

struct RelatedDocumentsView: View {
    let documentId: String
    let database: DatabaseManager
    let imageStore: ImageStore

    @State private var relatedDocs: [Document] = []

    var body: some View {
        if !relatedDocs.isEmpty {
            VStack(alignment: .leading, spacing: 8) {
                Text("Related Documents")
                    .font(.title3)
                    .fontWeight(.semibold)

                ForEach(relatedDocs) { doc in
                    NavigationLink(value: doc) {
                        HStack(spacing: 10) {
                            if let image = imageStore.loadThumbnail(for: doc.id) {
                                Image(uiImage: image)
                                    .resizable()
                                    .aspectRatio(contentMode: .fill)
                                    .frame(width: 40, height: 52)
                                    .clipShape(RoundedRectangle(cornerRadius: 4))
                            } else {
                                RoundedRectangle(cornerRadius: 4)
                                    .fill(.fill.quaternary)
                                    .frame(width: 40, height: 52)
                            }

                            VStack(alignment: .leading, spacing: 2) {
                                Text(doc.sender ?? "Unknown")
                                    .font(.subheadline)
                                    .fontWeight(.medium)
                                    .lineLimit(1)
                                if let date = doc.documentDate {
                                    Text(date)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                            }

                            Spacer()

                            Image(systemName: "chevron.right")
                                .font(.caption)
                                .foregroundStyle(.tertiary)
                        }
                        .padding(.vertical, 4)
                    }
                    .buttonStyle(.plain)
                }
            }
            .task {
                loadRelated()
            }
        } else {
            EmptyView()
                .task {
                    loadRelated()
                }
        }
    }

    private func loadRelated() {
        relatedDocs = (try? database.relatedDocuments(for: documentId)) ?? []
    }
}
