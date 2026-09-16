import Foundation

// Every discovered item is classified into exactly one of these risk levels.
// The DeletionEngine only accepts items whose stored path also passes a
// *second* ProtectedPaths check at delete time — the risk label is a hint,
// never a permission.
public enum RiskLevel: String, Codable, Sendable, CaseIterable {
    case safeToClean = "safe"
    case review = "review"
    case protectedItem = "protected"

    public var displayName: String {
        switch self {
        case .safeToClean:   return "Safe to clean"
        case .review:        return "Review"
        case .protectedItem: return "Protected"
        }
    }

    public var accentSuffix: String {
        switch self {
        case .safeToClean:   return "safe"      // greens
        case .review:        return "review"    // ambers
        case .protectedItem: return "protected" // greys
        }
    }
}

// A single discovered item — a cache directory, a build artifact, a log,
// a large file. `path` is the canonical absolute path resolved at scan time.
public struct ScanItem: Identifiable, Codable, Hashable, Sendable {
    public let id: UUID
    public let path: String
    public let size: Int64                 // bytes; -1 if unknown
    public let displayName: String         // "Chrome cache", not "~/Library/Caches/…"
    public let category: Category
    public let risk: RiskLevel
    public let explanation: String         // one plain-English sentence
    public let recovery: String            // "Chrome will rebuild the cache on next launch."
    public let requiresAppClosed: String?  // "Google Chrome" if the app should be closed
    public let discoveredAt: Date          // race-condition anchor

    public init(
        id: UUID = UUID(),
        path: String,
        size: Int64,
        displayName: String,
        category: Category,
        risk: RiskLevel,
        explanation: String,
        recovery: String,
        requiresAppClosed: String? = nil,
        discoveredAt: Date = Date()
    ) {
        self.id = id
        self.path = path
        self.size = size
        self.displayName = displayName
        self.category = category
        self.risk = risk
        self.explanation = explanation
        self.recovery = recovery
        self.requiresAppClosed = requiresAppClosed
        self.discoveredAt = discoveredAt
    }

    public var formattedSize: String { ByteCountFormatter.humanized(size) }
}

// Broad grouping in the results screen. Categories don't affect safety —
// they only shape the UI.
public enum Category: String, Codable, Sendable, CaseIterable {
    case browserCache        = "Browser caches"
    case applicationCache    = "Application caches"
    case developerCache      = "Developer caches"
    case packageManager      = "Package managers"
    case logs                = "Logs"
    case temp                = "Temporary files"
    case largeFile           = "Large files"
    case duplicate           = "Duplicates"
    case xcode               = "Xcode"
    case homebrew            = "Homebrew"
    case docker              = "Docker"
    case snapshots           = "macOS snapshots"
    case systemManaged       = "System managed"
    case other               = "Other"
}

// A scan report is the immutable output of one scan run.
public struct ScanReport: Codable, Sendable {
    public let startedAt: Date
    public let finishedAt: Date
    public let items: [ScanItem]
    public let errors: [ScanError]

    public var totalReclaimable: Int64 {
        items.filter { $0.risk == .safeToClean }.map(\.size).reduce(0, +)
    }
    public var totalReview: Int64 {
        items.filter { $0.risk == .review }.map(\.size).reduce(0, +)
    }
    public var totalProtected: Int64 {
        items.filter { $0.risk == .protectedItem }.map(\.size).reduce(0, +)
    }
    public func items(risk: RiskLevel) -> [ScanItem] {
        items.filter { $0.risk == risk }
    }
    public func items(category: Category) -> [ScanItem] {
        items.filter { $0.category == category }
    }
}

// A scanner may fail for one path without failing the whole scan.
public struct ScanError: Codable, Sendable, Identifiable {
    public var id: String { "\(scanner)|\(path)" }
    public let scanner: String
    public let path: String
    public let reason: String
    public init(scanner: String, path: String, reason: String) {
        self.scanner = scanner; self.path = path; self.reason = reason
    }
}

// A cleanup plan is what the user assembled from a report. It's re-validated
// again immediately before deletion in the DeletionEngine.
public struct CleanupPlan: Codable, Sendable {
    public let items: [ScanItem]
    public var total: Int64 { items.map(\.size).reduce(0, +) }
    public init(items: [ScanItem]) { self.items = items }
}

// The result of one deletion attempt on one item.
public struct DeletionResult: Identifiable, Sendable {
    public var id: UUID { item.id }
    public let item: ScanItem
    public let outcome: Outcome
    public let bytesReclaimed: Int64
    public enum Outcome: Sendable {
        case movedToTrash(URL)         // URL of the item's new Trash location
        case skipped(reason: String)   // safety re-check failed, changed, protected, etc.
        case failed(reason: String)    // OS error
    }
}

// Format bytes for the UI. Uses macOS-standard binary units.
public enum ByteCountFormatter {
    private static let fmt: Foundation.ByteCountFormatter = {
        let f = Foundation.ByteCountFormatter()
        f.countStyle = .file
        f.allowedUnits = [.useMB, .useGB, .useTB, .useKB, .useBytes]
        f.includesUnit = true
        f.zeroPadsFractionDigits = false
        return f
    }()
    public static func humanized(_ bytes: Int64) -> String {
        if bytes < 0 { return "—" }
        return fmt.string(fromByteCount: bytes)
    }
}
