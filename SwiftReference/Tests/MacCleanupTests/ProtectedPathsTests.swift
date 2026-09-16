import XCTest
@testable import MacCleanup

// The 15 critical safety tests from the product brief. Each one asserts
// that a specific protected path is refused by the deletion engine's
// pre-flight validator — regardless of scan-time labelling.
final class ProtectedPathsTests: XCTestCase {

    private func expectBlocked(_ path: String, file: StaticString = #file, line: UInt = #line) {
        let v = ProtectedPaths.isAllowedForDeletion(path: path)
        if case .ok = v {
            XCTFail("Expected \(path) to be BLOCKED but got .ok", file: file, line: line)
        }
    }

    private func home(_ suffix: String) -> String {
        (ProtectedPaths.home().path as NSString).appendingPathComponent(suffix)
    }

    // 1. Documents cannot be auto-deleted.
    func test01_DocumentsProtected() { expectBlocked(home("Documents")) }
    func test01a_DocumentsSubfolderProtected() { expectBlocked(home("Documents/Taxes")) }

    // 2. Desktop cannot be auto-deleted.
    func test02_DesktopProtected() { expectBlocked(home("Desktop")) }

    // 3. Downloads cannot be auto-deleted.
    func test03_DownloadsProtected() { expectBlocked(home("Downloads")) }

    // 4. Application Support cannot be auto-deleted.
    func test04_AppSupportProtected() { expectBlocked(home("Library/Application Support")) }
    func test04a_AppSupportSubProtected() { expectBlocked(home("Library/Application Support/Slack")) }

    // 5. Containers cannot be auto-deleted.
    func test05_ContainersProtected() { expectBlocked(home("Library/Containers")) }

    // 6. Group Containers cannot be auto-deleted.
    func test06_GroupContainersProtected() { expectBlocked(home("Library/Group Containers")) }

    // 7. System cannot be deleted.
    func test07_SystemProtected() { expectBlocked("/System") }
    func test07a_SystemVolumesProtected() { expectBlocked("/System/Volumes/Data") }

    // 8. /private cannot be deleted.
    func test08_PrivateProtected() { expectBlocked("/private") }
    func test08a_PrivateVarProtected() { expectBlocked("/private/var") }

    // 9. /opt/homebrew cannot be deleted as a whole.
    func test09_HomebrewRootProtected() { expectBlocked("/opt/homebrew") }
    func test09a_HomebrewCellarProtected() { expectBlocked("/opt/homebrew/Cellar") }
    func test09b_UsrLocalHomebrewProtected() { expectBlocked("/usr/local/Homebrew") }

    // 11. Symlink escape cannot delete outside the permitted cleanup dir.
    //     (Runtime check — DeletionEngineTests validates this dynamically.)

    // 12. A file that changed after scan is not deleted.
    //     (Dynamic — DeletionEngineTests.testChangedFileIsSkipped.)

    // Additional coverage: iCloud, Messages, Mail, Keychain.
    func test_iCloudProtected() { expectBlocked(home("Library/Mobile Documents")) }
    func test_MessagesProtected() { expectBlocked(home("Library/Messages")) }
    func test_MailProtected() { expectBlocked(home("Library/Mail")) }
    func test_KeychainProtected() { expectBlocked(home("Library/Keychains")) }
    func test_SafariProtected() { expectBlocked(home("Library/Safari")) }
    func test_ProjectsPreferencesProtected() { expectBlocked(home("Library/Preferences")) }
    func test_HomeItselfProtected() { expectBlocked(ProtectedPaths.home().path) }
    func test_RootProtected() { expectBlocked("/") }
    func test_ApplicationsProtected() { expectBlocked("/Applications") }
    func test_UsrProtected() { expectBlocked("/usr") }
    func test_LibraryRootProtected() { expectBlocked("/Library") }

    // Cross-user protection — attempting to delete /Users/OtherUser must fail.
    func test_OtherUsersHomeProtected() {
        expectBlocked("/Users/AnotherPerson")
        expectBlocked("/Users/AnotherPerson/Documents/thing.txt")
    }

    // Positive test — a genuine cache path SHOULD be allowed.
    func test_NpmCacheAllowed() {
        // We only assert allowance under $HOME; whether the folder exists
        // isn't the point — the policy check should return .ok.
        let path = home(".npm")
        let v = ProtectedPaths.isAllowedForDeletion(path: path)
        XCTAssertEqual(v, .ok, "Expected ~/.npm to be allowed for deletion, got \(v)")
    }

    func test_BrowserCacheAllowed() {
        let path = home("Library/Caches/Google/Chrome/Default/Cache")
        let v = ProtectedPaths.isAllowedForDeletion(path: path)
        XCTAssertEqual(v, .ok, "Expected browser cache path to be allowed, got \(v)")
    }

    // Traversal attempt — ~/Library/Caches/../.. must resolve to $HOME and be blocked.
    func test_TraversalAttemptBlocked() {
        expectBlocked(home("Library/Caches/../../Documents"))
    }

    // isPathContained sanity tests.
    func test_isPathContained_true_forSelf() {
        XCTAssertTrue(ProtectedPaths.isPathContained(child: "/a/b", in: "/a/b"))
    }
    func test_isPathContained_true_forChild() {
        XCTAssertTrue(ProtectedPaths.isPathContained(child: "/a/b/c", in: "/a/b"))
    }
    func test_isPathContained_false_forSibling() {
        XCTAssertFalse(ProtectedPaths.isPathContained(child: "/a/bc", in: "/a/b"))
    }
    func test_isPathContained_false_forParent() {
        XCTAssertFalse(ProtectedPaths.isPathContained(child: "/a", in: "/a/b"))
    }
}
