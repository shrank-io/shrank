import XCTest
@testable import Shrank

final class ShrankTests: XCTestCase {
    func testULIDGeneration() throws {
        let id = ULID.generate()
        XCTAssertEqual(id.count, 26, "ULID should be 26 characters")

        // Generate two ULIDs — they should be different
        let id2 = ULID.generate()
        XCTAssertNotEqual(id, id2)
    }

    func testULIDTimestampOrdering() throws {
        let id1 = ULID.generate()
        // Small delay to ensure different timestamp
        Thread.sleep(forTimeInterval: 0.01)
        let id2 = ULID.generate()
        XCTAssertTrue(id1 < id2, "Later ULID should sort after earlier one")
    }
}
