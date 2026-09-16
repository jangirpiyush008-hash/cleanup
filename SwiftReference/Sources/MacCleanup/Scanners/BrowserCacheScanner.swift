import Foundation

// Browser caches for the browsers we can detect. Each browser's cache is a
// SEPARATE ScanItem so the user can pick and choose. We deliberately target
// specific cache subdirectories — never the whole browser Application
// Support / Container folder (which holds bookmarks, logins, extensions).
public struct BrowserCacheScanner: Scanner {
    public let name = "Browser caches"
    public let category: Category = .browserCache

    // Each entry is: (display name, path suffix under $HOME, running-app name for the "close first" nudge)
    private static let browserCaches: [(String, String, String?)] = [
        // Chrome
        ("Google Chrome cache",  "Library/Caches/Google/Chrome/Default/Cache",                  "Google Chrome"),
        ("Google Chrome code cache", "Library/Caches/Google/Chrome/Default/Code Cache",         "Google Chrome"),
        ("Google Chrome GPU cache",  "Library/Caches/Google/Chrome/Default/GPUCache",            "Google Chrome"),
        // Chrome-generic caches folder (may exist on newer builds)
        ("Google Chrome caches",     "Library/Caches/Google/Chrome",                             "Google Chrome"),

        // Safari — only its Caches directory (never the Safari folder itself,
        // which contains history, bookmarks, reading list).
        ("Safari cache",             "Library/Caches/com.apple.Safari",                          "Safari"),

        // Firefox — the Cache2 folder inside each profile is safe. We match
        // the top Caches folder; scan-time size check filters empty ones.
        ("Firefox cache",            "Library/Caches/Firefox",                                   "Firefox"),

        // Arc
        ("Arc cache",                "Library/Caches/company.thebrowser.Browser",                "Arc"),

        // Brave
        ("Brave cache",              "Library/Caches/BraveSoftware/Brave-Browser",               "Brave Browser"),

        // Edge
        ("Microsoft Edge cache",     "Library/Caches/Microsoft Edge",                            "Microsoft Edge"),
    ]

    public init() {}

    public func scan(context: ScanContext) async -> ScanOutcome {
        var items: [ScanItem] = []
        var errors: [ScanError] = []
        var seenParents = Set<String>()

        for (label, suffix, runningApp) in Self.browserCaches {
            let full = context.home.appendingPathComponent(suffix).path
            let resolved = URL(fileURLWithPath: full).standardizedFileURL.path

            var isDir: ObjCBool = false
            guard FileManager.default.fileExists(atPath: resolved, isDirectory: &isDir), isDir.boolValue else {
                continue
            }
            // Avoid double-counting when a specific subdir AND its parent
            // are both listed. Keep the first (more specific) match; skip
            // the ancestor if we've already added something under it.
            let alreadyChild = items.contains {
                ProtectedPaths.isPathContained(child: $0.path, in: resolved) && $0.path != resolved
            }
            if alreadyChild { seenParents.insert(resolved); continue }

            do {
                let size = try DeletionEngine.measureSize(at: resolved)
                if size < 20 * 1024 * 1024 { continue }   // ignore < 20MB
                items.append(ScanItem(
                    path: resolved,
                    size: size,
                    displayName: label,
                    category: .browserCache,
                    risk: .safeToClean,
                    explanation: "Temporary browser cache — helps pages load faster. Safely rebuilt as you browse.",
                    recovery: "Moved to Trash. Browser will rebuild the cache automatically.",
                    requiresAppClosed: runningApp
                ))
            } catch {
                errors.append(.init(scanner: name, path: resolved, reason: (error as NSError).localizedDescription))
            }
        }
        return ScanOutcome(items: items, errors: errors)
    }
}
