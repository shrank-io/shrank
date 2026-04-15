import SwiftUI

struct DocumentRow: View {
    let document: Document
    let imageStore: ImageStore

    var body: some View {
        HStack(spacing: 12) {
            // Thumbnail
            thumbnailView
                .frame(width: 56, height: 72)
                .clipShape(RoundedRectangle(cornerRadius: 6))

            // Text content
            VStack(alignment: .leading, spacing: 4) {
                Text(document.sender ?? "Unknown sender")
                    .font(.headline)
                    .lineLimit(1)

                if let subject = document.subject {
                    Text(subject)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                        .lineLimit(2)
                }

                HStack(spacing: 8) {
                    if let dateStr = document.documentDate ?? formattedCaptureDate {
                        Text(dateStr)
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                    }
                    if let type = document.documentType {
                        Text(type.capitalized)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(.fill.tertiary)
                            .clipShape(Capsule())
                    }
                }
            }

            Spacer()

            // Status indicator
            statusView
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private var thumbnailView: some View {
        if let image = imageStore.loadThumbnail(for: document.id) {
            Image(uiImage: image)
                .resizable()
                .aspectRatio(contentMode: .fill)
        } else {
            RoundedRectangle(cornerRadius: 6)
                .fill(.fill.quaternary)
                .overlay {
                    Image(systemName: "doc.text")
                        .foregroundStyle(.tertiary)
                }
        }
    }

    @ViewBuilder
    private var statusView: some View {
        switch document.status {
        case .complete:
            Image(systemName: "checkmark.circle.fill")
                .foregroundStyle(.green)
                .font(.caption)
        case .processing, .pending:
            ProgressView()
                .controlSize(.small)
        case .error:
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.orange)
                .font(.caption)
        }
    }

    private var formattedCaptureDate: String? {
        guard let date = ISO8601DateFormatter().date(from: document.capturedAt) else { return nil }
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .none
        return formatter.string(from: date)
    }
}
