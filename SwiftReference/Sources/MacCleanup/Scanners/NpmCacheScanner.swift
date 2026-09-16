import Foundation

// npm cache — downloaded package data that npm will re-fetch on demand.
// Safe-to-clean when present; skipped if not installed or empty.
public struct NpmCacheScanner: Scanner {
    public let name = "npm cache"
    public let category: Category = .packageManager

    public init() {}

    public func scan(context: ScanContext) async -> ScanOutcome {
        let candidates = [
            context.home.appendingPathComponent(".npm").path,
            context.home.appendingPathComponent(".npm/_cacache").path,
        ]
        var items: [ScanItem] = []
        var errors: [ScanError] = []
        var seen = Set<String>()

        for path in candidates {
            let resolved = URL(fileURLWithPath: path).standardizedFileURL.path
            if seen.contains(resolved) { continue }
            seen.insert(resolved)

            var isDir: ObjCBool = false
            guard FileManager.default.fileExists(atPath: resolved, isDirectory: &isDir), isDir.boolValue else {
                continue
            }
            // Only include the outermost cache we recognise. If .npm exists,
            // skip the nested _cacache to avoid double-counting.
            if path.hasSuffix("_cacache"), items.contains(where: { $0.path.hasSuffix(".npm") }) {
                continue
            }
            do {
                let size = try DeletionEngine.measureSize(at: resolved)
                if size < 10 * 1024 * 1024 { continue }   // ignore < 10MB — not worth surfacing
                items.append(ScanItem(
                    path: resolved,
                    size: size,
                    displayName: "npm cache",
                    category: .packageManager,
                    risk: .safeToClean,
                    explanation: "Temporary downloaded package data. npm re-downloads packages as needed.",
                    recovery: "Moved to Trash. npm will rebuild the cache on next install."
                ))
                break   // outer .npm wins over nested _cacache
            } catch {
                errors.append(.init(scanner: name, path: resolved, reason: (error as NSError).localizedDescription))
            }
        }
        return ScanOutcome(items: items, errors: errors)
    }
}
