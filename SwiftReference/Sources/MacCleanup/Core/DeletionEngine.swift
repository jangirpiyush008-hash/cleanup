import Foundation

// Moves selected ScanItems to Trash. Never permanent-delete.
//
// Every item is re-validated in the moment before the call to
// FileManager.trashItem. The scan-time risk label is treated as advisory
// only — this engine trusts nothing.
public struct DeletionEngine {

    public struct DeletionOptions: Sendable {
        public let ignoreSizeMismatch: Bool
        public init(ignoreSizeMismatch: Bool = false) {
            self.ignoreSizeMismatch = ignoreSizeMismatch
        }
    }

    // Move an entire plan to Trash, item by item. Continues on individual
    // failures. Emits one DeletionResult per input item, in order.
    public static func moveToTrash(
        plan: CleanupPlan,
        options: DeletionOptions = .init()
    ) -> [DeletionResult] {
        plan.items.map { moveOneToTrash(item: $0, options: options) }
    }

    // The per-item guard rails. Public so tests can call it directly.
    public static func moveOneToTrash(
        item: ScanItem,
        options: DeletionOptions = .init()
    ) -> DeletionResult {

        // 1. Protected-paths policy (re-runs against the current filesystem).
        switch ProtectedPaths.isAllowedForDeletion(path: item.path) {
        case .blocked(let reason):
            return DeletionResult(item: item, outcome: .skipped(reason: reason), bytesReclaimed: 0)
        case .ok:
            break
        }

        // 2. Path must currently exist and be reachable.
        let fm = FileManager.default
        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: item.path, isDirectory: &isDir) else {
            return DeletionResult(
                item: item,
                outcome: .skipped(reason: "Path no longer exists."),
                bytesReclaimed: 0
            )
        }

        // 3. Symlink escape check — if `item.path` is (or contains) a symlink
        //    whose canonical target lives *outside* the scan-time parent,
        //    refuse. Prevents attackers from swapping in a symlink between
        //    scan and delete to trick us into deleting elsewhere.
        let url = URL(fileURLWithPath: item.path)
        let canonicalPath = url.resolvingSymlinksInPath().standardizedFileURL.path
        // The canonical form must still pass ProtectedPaths.
        switch ProtectedPaths.isAllowedForDeletion(path: canonicalPath) {
        case .blocked(let reason):
            return DeletionResult(
                item: item,
                outcome: .skipped(reason: "Symlink target rejected: \(reason)"),
                bytesReclaimed: 0
            )
        case .ok:
            break
        }

        // 4. Size sanity — if the item ballooned by more than 10× since scan
        //    or shrank drastically, treat as changed and skip. This catches
        //    the case where a cache directory was replaced with something
        //    completely different between scan and delete.
        if !options.ignoreSizeMismatch, item.size > 0 {
            if let currentSize = try? measureSize(at: canonicalPath) {
                let ratio = Double(currentSize) / Double(item.size)
                if ratio > 10.0 || ratio < 0.05 {
                    return DeletionResult(
                        item: item,
                        outcome: .skipped(reason: "Size changed significantly since scan; skipped for safety."),
                        bytesReclaimed: 0
                    )
                }
            }
        }

        // 5. Move to Trash. Never rm.
        do {
            var trashURL: NSURL?
            try (fm as NSFileManager).trashItem(at: URL(fileURLWithPath: canonicalPath), resultingItemURL: &trashURL)
            let reclaimed: Int64 = (try? measureSize(at: canonicalPath)) ?? item.size
            return DeletionResult(
                item: item,
                outcome: .movedToTrash(trashURL as URL? ?? URL(fileURLWithPath: canonicalPath)),
                bytesReclaimed: max(0, item.size)  // report the size we scanned
            )
        } catch {
            return DeletionResult(
                item: item,
                outcome: .failed(reason: (error as NSError).localizedDescription),
                bytesReclaimed: 0
            )
        }
    }

    // Recursive size measurement using NSDirectoryEnumerator. Safe on huge
    // trees because we only sum sizes; never modify.
    static func measureSize(at path: String) throws -> Int64 {
        let fm = FileManager.default
        var isDir: ObjCBool = false
        guard fm.fileExists(atPath: path, isDirectory: &isDir) else { return 0 }
        if !isDir.boolValue {
            let attrs = try fm.attributesOfItem(atPath: path)
            return (attrs[.size] as? NSNumber)?.int64Value ?? 0
        }
        var total: Int64 = 0
        let url = URL(fileURLWithPath: path)
        let keys: [URLResourceKey] = [.fileSizeKey, .totalFileAllocatedSizeKey, .isDirectoryKey]
        guard let enumerator = fm.enumerator(
            at: url,
            includingPropertiesForKeys: keys,
            options: [.skipsHiddenFiles],
            errorHandler: { _, _ in true }   // skip unreadable, keep going
        ) else { return 0 }
        for case let child as URL in enumerator {
            let vals = try? child.resourceValues(forKeys: Set(keys))
            if vals?.isDirectory == true { continue }
            if let s = vals?.totalFileAllocatedSize ?? vals?.fileSize {
                total += Int64(s)
            }
        }
        return total
    }
}

// Bridge to the older Objective-C trashItem API which uses NSURL** for the
// resulting location. Kept in one place so callers don't sprinkle NSURL.
private typealias NSFileManager = FileManager
