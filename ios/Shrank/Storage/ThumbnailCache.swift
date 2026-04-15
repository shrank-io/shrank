import UIKit

final class ThumbnailCache: @unchecked Sendable {
    @MainActor static let shared = ThumbnailCache()

    private let cache = NSCache<NSString, UIImage>()
    private let imageStore: ImageStore

    init(imageStore: ImageStore = ImageStore()) {
        self.imageStore = imageStore
        cache.countLimit = 200
    }

    func image(for documentId: String) -> UIImage? {
        let key = documentId as NSString
        if let cached = cache.object(forKey: key) {
            return cached
        }
        if let loaded = imageStore.loadThumbnail(for: documentId) {
            cache.setObject(loaded, forKey: key)
            return loaded
        }
        return nil
    }

    func set(image: UIImage, for documentId: String) {
        cache.setObject(image, forKey: documentId as NSString)
    }

    func clearMemoryCache() {
        cache.removeAllObjects()
    }
}
