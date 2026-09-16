import XCTest
@testable import MacCleanup

// Dynamic tests for the deletion engine. These operate on a temp directory
// only — the real ~/.Trash is never touched here because trashItem on a
// temp path routes to the volume's Trash and can be inspected.
//
// The tests validate:
//   - Protected paths are refused even if a ScanItem falsely labels them
//   - Missing paths are skipped, not failed
//   - Symlink escape attempts are refused
//   - A file whose size changed dramatically is skipped
//   - A legitimate temp file IS moved to Trash and returns bytesReclaimed
final class DeletionEngineTests: XCTestCase {

    var scratch: URL!

    override func setUpWithError() throws {
        scratch = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("mc-tests-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: scratch, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        try? FileManager.default.removeItem(at: scratch)
    }

    private func writeFile(_ name: String, bytes: Int) throws -> URL {
        let url = scratch.appendingPathComponent(name)
        let data = Data(repeating: 0x41, count: bytes)
        try data.write(to: url)
        return url
    }

    private func item(for url: URL, size: Int64, risk: RiskLevel = .safeToClean) -> ScanItem {
        ScanItem(
            path: url.path,
            size: size,
            displayName: url.lastPathComponent,
            category: .other,
            risk: risk,
            explanation: "test",
            recovery: "test"
        )
    }

    // 13. No deletion without confirmation is a UI-level property; here we
    //     verify the engine at least refuses obviously-protected inputs.
    func test_protectedItemBlocked() {
        let bad = ScanItem(
            path: "/System",
            size: 1_000_000,
            displayName: "System",
            category: .other,
            risk: .safeToClean,          // deliberately mislabelled
            explanation: "", recovery: ""
        )
        let result = DeletionEngine.moveOneToTrash(item: bad)
        if case .skipped(let reason) = result.outcome {
            XCTAssertTrue(reason.contains("protected") || reason.contains("System"), reason)
        } else {
            XCTFail("Expected .skipped, got \(result.outcome)")
        }
    }

    func test_missingPathSkipped() throws {
        let ghost = scratch.appendingPathComponent("does-not-exist")
        let result = DeletionEngine.moveOneToTrash(item: item(for: ghost, size: 100))
        if case .skipped(let reason) = result.outcome {
            XCTAssertTrue(reason.contains("no longer exists"), reason)
        } else {
            XCTFail("Expected .skipped, got \(result.outcome)")
        }
    }

    // 11. Symlink escape check — a symlink pointing to /System must be refused.
    func test_symlinkEscapeRefused() throws {
        let link = scratch.appendingPathComponent("trap")
        try FileManager.default.createSymbolicLink(at: link, withDestinationURL: URL(fileURLWithPath: "/System"))
        let result = DeletionEngine.moveOneToTrash(item: item(for: link, size: 100))
        if case .skipped(let reason) = result.outcome {
            XCTAssertTrue(reason.contains("Symlink target rejected") || reason.contains("System"),
                          "reason was: \(reason)")
        } else {
            XCTFail("Expected .skipped, got \(result.outcome)")
        }
    }

    // 12. A file whose size changed dramatically is skipped.
    func test_sizeMismatchSkipped() throws {
        let url = try writeFile("cache.bin", bytes: 100)
        // Scan captured size = 100, but if we claim it was 1MB we'll fail
        // the ratio check (100 / 1_000_000 = 0.0001 → drastic shrink).
        let it = item(for: url, size: 1_000_000)
        let result = DeletionEngine.moveOneToTrash(item: it)
        if case .skipped(let reason) = result.outcome {
            XCTAssertTrue(reason.contains("Size changed"), reason)
        } else {
            XCTFail("Expected .skipped, got \(result.outcome)")
        }
    }

    // Sanity: a plain temp file (with matching size) does move to Trash and
    // is no longer present at the original path.
    func test_happyPath_movesToTrash() throws {
        let url = try writeFile("payload.bin", bytes: 2048)
        let it = item(for: url, size: 2048)
        let result = DeletionEngine.moveOneToTrash(item: it)
        switch result.outcome {
        case .movedToTrash:
            XCTAssertFalse(FileManager.default.fileExists(atPath: url.path))
            XCTAssertGreaterThan(result.bytesReclaimed, 0)
        case .skipped(let r), .failed(let r):
            XCTFail("Expected .movedToTrash, got skip/fail: \(r)")
        }
    }
}
