import Foundation
import UIKit

final class ImageStore {
    private let baseURL: URL

    init() {
        let documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        self.baseURL = documents.appendingPathComponent("images")
        createDirectories()
    }

    private func createDirectories() {
        let fm = FileManager.default
        try? fm.createDirectory(at: originalsDirectory, withIntermediateDirectories: true)
        try? fm.createDirectory(at: thumbnailsDirectory, withIntermediateDirectories: true)
    }

    private var originalsDirectory: URL {
        baseURL.appendingPathComponent("originals")
    }

    private var thumbnailsDirectory: URL {
        baseURL.appendingPathComponent("thumbnails")
    }

    // MARK: - Save

    func saveOriginal(imageData: Data, id: String) throws -> String {
        let relativePath = "originals/\(id).jpg"
        let url = baseURL.appendingPathComponent(relativePath)
        try imageData.write(to: url)
        return relativePath
    }

    func saveThumbnail(imageData: Data, id: String) throws -> String {
        let relativePath = "thumbnails/\(id).jpg"
        let url = baseURL.appendingPathComponent(relativePath)
        try imageData.write(to: url)
        return relativePath
    }

    // MARK: - Load

    func originalURL(for id: String) -> URL {
        baseURL.appendingPathComponent("originals/\(id).jpg")
    }

    func thumbnailURL(for id: String) -> URL {
        baseURL.appendingPathComponent("thumbnails/\(id).jpg")
    }

    func loadThumbnail(for id: String) -> UIImage? {
        let url = thumbnailURL(for: id)
        guard FileManager.default.fileExists(atPath: url.path) else { return nil }
        return UIImage(contentsOfFile: url.path)
    }

    func loadOriginal(for id: String) -> UIImage? {
        let url = originalURL(for: id)
        guard FileManager.default.fileExists(atPath: url.path) else { return nil }
        return UIImage(contentsOfFile: url.path)
    }

    func thumbnailExists(for id: String) -> Bool {
        FileManager.default.fileExists(atPath: thumbnailURL(for: id).path)
    }

    // MARK: - Delete

    func deleteImages(for id: String) {
        let fm = FileManager.default
        try? fm.removeItem(at: originalURL(for: id))
        try? fm.removeItem(at: thumbnailURL(for: id))
    }

    // MARK: - Thumbnail generation

    func generateThumbnail(from imageData: Data, maxWidth: CGFloat = 400) -> Data? {
        guard let image = UIImage(data: imageData) else { return nil }
        return generateThumbnail(from: image, maxWidth: maxWidth)
    }

    func generateThumbnail(from image: UIImage, maxWidth: CGFloat = 400) -> Data? {
        let scale = maxWidth / image.size.width
        guard scale < 1 else {
            // Image is already smaller than target
            return image.jpegData(compressionQuality: 0.75)
        }
        let newSize = CGSize(width: maxWidth, height: image.size.height * scale)
        let renderer = UIGraphicsImageRenderer(size: newSize)
        let resized = renderer.image { _ in
            image.draw(in: CGRect(origin: .zero, size: newSize))
        }
        return resized.jpegData(compressionQuality: 0.75)
    }

    // MARK: - Storage stats

    func totalStorageUsed() -> Int64 {
        let fm = FileManager.default
        var total: Int64 = 0
        for dir in [originalsDirectory, thumbnailsDirectory] {
            guard let enumerator = fm.enumerator(at: dir, includingPropertiesForKeys: [.fileSizeKey]) else { continue }
            for case let url as URL in enumerator {
                if let size = try? url.resourceValues(forKeys: [.fileSizeKey]).fileSize {
                    total += Int64(size)
                }
            }
        }
        return total
    }
}
