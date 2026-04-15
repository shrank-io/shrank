import Foundation
import UIKit
import ImageIO

struct ProcessedCapture {
    let originalData: Data
    let thumbnailData: Data
    let capturedAt: Date
}

enum ImageProcessor {
    static func process(image: UIImage) -> ProcessedCapture? {
        // Normalize orientation
        let normalized = normalizeOrientation(image)

        guard let originalData = normalized.jpegData(compressionQuality: 0.85) else { return nil }

        // Generate thumbnail (400px wide)
        let thumbnailData = generateThumbnail(from: normalized, maxWidth: 400)

        // Try to extract EXIF date, fall back to now
        let capturedAt = extractExifDate(from: originalData) ?? Date()

        return ProcessedCapture(
            originalData: originalData,
            thumbnailData: thumbnailData ?? originalData,
            capturedAt: capturedAt
        )
    }

    private static func normalizeOrientation(_ image: UIImage) -> UIImage {
        guard image.imageOrientation != .up else { return image }
        let renderer = UIGraphicsImageRenderer(size: image.size)
        return renderer.image { _ in
            image.draw(at: .zero)
        }
    }

    private static func generateThumbnail(from image: UIImage, maxWidth: CGFloat) -> Data? {
        let scale = maxWidth / image.size.width
        guard scale < 1 else {
            return image.jpegData(compressionQuality: 0.75)
        }
        let newSize = CGSize(width: maxWidth, height: image.size.height * scale)
        let renderer = UIGraphicsImageRenderer(size: newSize)
        let resized = renderer.image { _ in
            image.draw(in: CGRect(origin: .zero, size: newSize))
        }
        return resized.jpegData(compressionQuality: 0.75)
    }

    private static func extractExifDate(from jpegData: Data) -> Date? {
        guard let source = CGImageSourceCreateWithData(jpegData as CFData, nil),
              let properties = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [String: Any],
              let exif = properties[kCGImagePropertyExifDictionary as String] as? [String: Any],
              let dateString = exif[kCGImagePropertyExifDateTimeOriginal as String] as? String
        else { return nil }

        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy:MM:dd HH:mm:ss"
        formatter.locale = Locale(identifier: "en_US_POSIX")
        return formatter.date(from: dateString)
    }
}
