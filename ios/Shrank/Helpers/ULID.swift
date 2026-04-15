import Foundation
import Security

enum ULID {
    private static let encoding: [Character] = Array("0123456789ABCDEFGHJKMNPQRSTVWXYZ")

    static func generate() -> String {
        let timestamp = UInt64(Date().timeIntervalSince1970 * 1000)
        var chars = [Character](repeating: "0", count: 26)

        // Encode 48-bit timestamp into first 10 characters (big-endian, base32)
        var t = timestamp
        for i in stride(from: 9, through: 0, by: -1) {
            chars[i] = encoding[Int(t & 0x1F)]
            t >>= 5
        }

        // Encode 80 bits of randomness into last 16 characters
        var randomBytes = [UInt8](repeating: 0, count: 10)
        _ = SecRandomCopyBytes(kSecRandomDefault, randomBytes.count, &randomBytes)

        var bitBuffer: UInt64 = 0
        var bitsInBuffer = 0
        var charIndex = 10

        for byte in randomBytes {
            bitBuffer = (bitBuffer << 8) | UInt64(byte)
            bitsInBuffer += 8
            while bitsInBuffer >= 5 && charIndex < 26 {
                bitsInBuffer -= 5
                let index = Int((bitBuffer >> bitsInBuffer) & 0x1F)
                chars[charIndex] = encoding[index]
                charIndex += 1
            }
        }

        return String(chars)
    }
}
