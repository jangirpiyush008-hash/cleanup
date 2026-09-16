import Foundation

// Every category has a Scanner. Scanners are async, read-only, and return
// ScanItems. If a scanner can't run (tool not installed, no permission),
// it returns [] and — optionally — appends a ScanError explaining why.
public protocol Scanner: Sendable {
    var name: String { get }
    var category: Category { get }
    func scan(context: ScanContext) async -> ScanOutcome
}

public struct ScanContext: Sendable {
    public let home: URL
    public init(home: URL) { self.home = home }
}

public struct ScanOutcome: Sendable {
    public let items: [ScanItem]
    public let errors: [ScanError]
    public init(items: [ScanItem], errors: [ScanError] = []) {
        self.items = items; self.errors = errors
    }
    public static let empty = ScanOutcome(items: [], errors: [])
}

// Orchestrates all registered scanners concurrently and folds their outputs
// into a single ScanReport. Individual scanner failures never abort the run.
public actor ScannerEngine {
    private let scanners: [Scanner]
    private let context: ScanContext

    public init(scanners: [Scanner], home: URL = FileManager.default.homeDirectoryForCurrentUser) {
        self.scanners = scanners
        self.context = ScanContext(home: home)
    }

    // Progress callback fires on the main actor.
    public func run(
        onProgress: @escaping @Sendable @MainActor (String) -> Void = { _ in }
    ) async -> ScanReport {
        let started = Date()
        var all: [ScanItem] = []
        var errors: [ScanError] = []

        await withTaskGroup(of: (String, ScanOutcome).self) { group in
            for scanner in scanners {
                group.addTask { [scanner, context] in
                    let out = await scanner.scan(context: context)
                    return (scanner.name, out)
                }
            }
            for await (name, out) in group {
                await onProgress(name)
                all.append(contentsOf: out.items)
                errors.append(contentsOf: out.errors)
            }
        }
        return ScanReport(startedAt: started, finishedAt: Date(), items: all, errors: errors)
    }
}
