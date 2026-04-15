import SwiftUI

struct DocumentDetailView: View {
    let document: Document
    let database: DatabaseManager
    let imageStore: ImageStore

    @State private var showFullImage = false

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                // Hero image
                heroImage

                // Status banner
                statusBanner

                // Metadata
                metadataSection

                // Amounts
                if !document.decodedAmounts.isEmpty {
                    amountsSection
                }

                // Important dates
                if !document.decodedDates.isEmpty {
                    datesSection
                }

                // Reference IDs
                if !document.decodedReferenceIds.isEmpty {
                    referencesSection
                }

                // Tags
                if !document.decodedTags.isEmpty {
                    tagsSection
                }

                // Extracted text
                if let text = document.extractedText, !text.isEmpty {
                    extractedTextSection(text)
                }

                // Related documents
                RelatedDocumentsView(
                    documentId: document.id,
                    database: database,
                    imageStore: imageStore
                )
            }
            .padding()
        }
        .navigationTitle(document.sender ?? "Document")
        .navigationBarTitleDisplayMode(.inline)
        .fullScreenCover(isPresented: $showFullImage) {
            fullImageViewer
        }
    }

    // MARK: - Sections

    private var heroImage: some View {
        Group {
            if let image = imageStore.loadThumbnail(for: document.id) {
                Image(uiImage: image)
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(maxHeight: 300)
                    .clipShape(RoundedRectangle(cornerRadius: 12))
                    .onTapGesture { showFullImage = true }
            } else {
                RoundedRectangle(cornerRadius: 12)
                    .fill(.fill.quaternary)
                    .frame(height: 200)
                    .overlay {
                        Image(systemName: "doc.text")
                            .font(.largeTitle)
                            .foregroundStyle(.tertiary)
                    }
            }
        }
        .frame(maxWidth: .infinity)
    }

    @ViewBuilder
    private var statusBanner: some View {
        switch document.status {
        case .pending, .processing:
            Label("Processing...", systemImage: "clock")
                .font(.subheadline)
                .padding(10)
                .frame(maxWidth: .infinity)
                .background(.yellow.opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 8))
        case .error:
            Label(document.processingError ?? "Processing failed", systemImage: "exclamationmark.triangle")
                .font(.subheadline)
                .padding(10)
                .frame(maxWidth: .infinity)
                .background(.red.opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 8))
        case .complete:
            EmptyView()
        }
    }

    private var metadataSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            sectionHeader("Details")

            metadataRow("Sender", value: document.sender)
            metadataRow("Date", value: document.documentDate)
            metadataRow("Type", value: document.documentType?.capitalized)
            metadataRow("Language", value: document.language?.uppercased())

            if let subject = document.subject {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Subject")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Text(subject)
                        .font(.body)
                }
            }

            if let confidence = document.confidence {
                metadataRow("Confidence", value: "\(Int(confidence * 100))%")
            }
        }
    }

    private var amountsSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionHeader("Amounts")
            ForEach(document.decodedAmounts, id: \.label) { amount in
                HStack {
                    Text(amount.label)
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text(formatAmount(amount))
                        .fontWeight(.medium)
                        .monospacedDigit()
                }
            }
        }
    }

    private var datesSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionHeader("Important Dates")
            ForEach(document.decodedDates, id: \.date) { entry in
                HStack {
                    Text(entry.label)
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text(entry.date)
                        .monospacedDigit()
                }
            }
        }
    }

    private var referencesSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionHeader("Reference Numbers")
            ForEach(document.decodedReferenceIds, id: \.value) { ref in
                HStack {
                    Text(ref.type.capitalized)
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text(ref.value)
                        .font(.system(.body, design: .monospaced))
                }
            }
        }
    }

    private var tagsSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionHeader("Tags")
            FlowLayout(spacing: 6) {
                ForEach(document.decodedTags, id: \.self) { tag in
                    Text(tag.replacingOccurrences(of: "_", with: " "))
                        .font(.caption)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 5)
                        .background(.blue.opacity(0.1))
                        .foregroundStyle(.blue)
                        .clipShape(Capsule())
                }
            }
        }
    }

    private func extractedTextSection(_ text: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            sectionHeader("Extracted Text")
            Text(text)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
        }
    }

    private var fullImageViewer: some View {
        NavigationStack {
            Group {
                if let image = imageStore.loadOriginal(for: document.id)
                    ?? imageStore.loadThumbnail(for: document.id) {
                    ScrollView([.horizontal, .vertical]) {
                        Image(uiImage: image)
                            .resizable()
                            .aspectRatio(contentMode: .fit)
                    }
                } else {
                    Text("Image not available")
                        .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("Original")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Done") { showFullImage = false }
                }
            }
        }
    }

    // MARK: - Helpers

    private func sectionHeader(_ title: String) -> some View {
        Text(title)
            .font(.title3)
            .fontWeight(.semibold)
    }

    private func metadataRow(_ label: String, value: String?) -> some View {
        Group {
            if let value, !value.isEmpty {
                HStack {
                    Text(label)
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text(value)
                }
            }
        }
    }

    private func formatAmount(_ amount: Amount) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = amount.currency
        return formatter.string(from: NSNumber(value: amount.value)) ?? "\(amount.value) \(amount.currency)"
    }
}

// MARK: - Flow Layout for tags

struct FlowLayout: Layout {
    var spacing: CGFloat = 8

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let result = layout(subviews: subviews, width: proposal.width ?? .infinity)
        return result.size
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let result = layout(subviews: subviews, width: bounds.width)
        for (index, position) in result.positions.enumerated() {
            subviews[index].place(
                at: CGPoint(x: bounds.minX + position.x, y: bounds.minY + position.y),
                proposal: .unspecified
            )
        }
    }

    private func layout(subviews: Subviews, width: CGFloat) -> (size: CGSize, positions: [CGPoint]) {
        var positions: [CGPoint] = []
        var x: CGFloat = 0
        var y: CGFloat = 0
        var rowHeight: CGFloat = 0
        var maxWidth: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if x + size.width > width && x > 0 {
                x = 0
                y += rowHeight + spacing
                rowHeight = 0
            }
            positions.append(CGPoint(x: x, y: y))
            rowHeight = max(rowHeight, size.height)
            x += size.width + spacing
            maxWidth = max(maxWidth, x)
        }

        return (CGSize(width: maxWidth, height: y + rowHeight), positions)
    }
}
