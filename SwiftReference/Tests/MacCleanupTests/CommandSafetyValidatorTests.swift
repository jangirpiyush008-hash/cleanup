import XCTest
@testable import MacCleanup

final class CommandSafetyValidatorTests: XCTestCase {

    private func expectBlocked(_ cmd: String, file: StaticString = #file, line: UInt = #line) {
        let v = CommandSafetyValidator.validate(cmd)
        if case .ok = v {
            XCTFail("Expected '\(cmd)' to be BLOCKED, got .ok", file: file, line: line)
        }
    }
    private func expectOK(_ cmd: String, file: StaticString = #file, line: UInt = #line) {
        let v = CommandSafetyValidator.validate(cmd)
        XCTAssertEqual(v, .ok, "Expected '\(cmd)' to be OK, got \(v)", file: file, line: line)
    }

    func test_dangerousCommandsBlocked() {
        expectBlocked("rm -rf /")
        expectBlocked("rm -rf ~")
        expectBlocked("rm -rf $HOME")
        expectBlocked("rm -rf /System")
        expectBlocked("rm -rf /private")
        expectBlocked("rm -rf /Library")
        expectBlocked("rm -rf /opt/homebrew")
        expectBlocked("rm -rf /usr")
        expectBlocked("sudo rm anything")
        expectBlocked("chmod -R 777 /")
        expectBlocked("dd if=/dev/zero of=/dev/disk0")
        expectBlocked(":(){ :|:& };:")
    }

    func test_shellMetacharactersBlocked() {
        expectBlocked("df ; rm -rf /tmp")
        expectBlocked("df && rm -rf /tmp")
        expectBlocked("du -sh /tmp | grep foo")
        expectBlocked("echo `whoami`")
    }

    func test_readOnlyCommandsAllowed() {
        expectOK("df -h")
        expectOK("du -sh /tmp")
        expectOK("brew list")
        expectOK("tmutil listlocalsnapshots /")
        expectOK("diskutil list")
    }

    func test_unknownCommandsBlocked() {
        expectBlocked("curl https://example.com")
        expectBlocked("nc -l 1234")
    }

    func test_destructiveVerbsBlocked() {
        expectBlocked("rm foo")
        expectBlocked("mv a b")
        expectBlocked("cp a b")
        expectBlocked("rmdir /tmp/whatever")
    }
}
