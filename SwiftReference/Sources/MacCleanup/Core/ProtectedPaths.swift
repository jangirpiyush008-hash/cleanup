import Foundation

// The single source of truth for what Mac Cleanup can *never* delete.
//
// Every path — no matter which scanner produced it, no matter what risk
// label it carries — must pass `isAllowedForDeletion(path:)` immediately
// before the DeletionEngine calls FileManager.trashItem. This is a
// belt-and-braces layer: even if a scanner bug labels a personal folder
// as "safe to clean", the deletion engine rejects it here.
public enum ProtectedPaths {

    // Absolute system paths that must never be touched.
    public static let systemAbsolute: [String] = [
        "/",
        "/System",
        "/System/Volumes",
        "/System/Library",
        "/System/Applications",
        "/private",
        "/private/var",
        "/private/etc",
        "/var",
        "/etc",
        "/usr",
        "/bin",
        "/sbin",
        "/Applications",
        "/opt",
        "/opt/homebrew",
        "/opt/homebrew/Cellar",
        "/opt/homebrew/var",
        "/opt/homebrew/Caskroom",
        "/usr/local",
        "/usr/local/Homebrew",
        "/Library",
        "/Library/Application Support",
        "/Volumes",
        "/dev",
        "/tmp",   // shell-writable but system-managed; use ~/Library/Caches instead
        "/cores",
    ]

    // Paths under $HOME that must never be auto-deleted. Stored as suffixes
    // relative to the user's home so this works for any username.
    public static let personalHomeRelative: [String] = [
        "Documents",
        "Desktop",
        "Downloads",
        "Pictures",
        "Movies",
        "Music",
        "Public",
        "Library/Application Support",
        "Library/Containers",
        "Library/Group Containers",
        "Library/Mail",
        "Library/Messages",
        "Library/Keychains",
        "Library/Mobile Documents",   // iCloud Drive
        "Library/CloudStorage",
        "Library/PersonalAssistant",
        "Library/Preferences",        // apps rely on this — never bulk-delete
        "Library/Safari",             // history, bookmarks
        "Library/Cookies",
        "Library/Passwords",
        "Library/Sync Services",
        "Library/Reminders",
        "Library/Calendars",
        "Library/AddressBook",
        ".ssh",
        ".gnupg",
        ".aws",
        ".config",                    // user configs (git, gh, etc.)
    ]

    // Return the resolved home directory for the CURRENT user.
    // Uses NSHomeDirectory / FileManager — never a hardcoded username.
    public static func home() -> URL {
        FileManager.default.homeDirectoryForCurrentUser
    }

    // Canonicalize a path: expand ~, resolve symlinks, normalize `.` and `..`.
    // Returns nil if the path can't be resolved.
    public static func canonical(_ path: String) -> URL? {
        let expanded = (path as NSString).expandingTildeInPath
        let url = URL(fileURLWithPath: expanded).standardizedFileURL.resolvingSymlinksInPath()
        return url
    }

    // Top-level policy check. Runs before every deletion.
    // Returns .ok if the path is safe to send to Trash under our rules,
    // or a Reason describing why it must be blocked.
    public enum Verdict: Equatable {
        case ok
        case blocked(reason: String)
    }

    public static func isAllowedForDeletion(path: String) -> Verdict {
        // 1. Must resolve to a concrete file URL.
        guard let url = canonical(path) else {
            return .blocked(reason: "Path could not be resolved.")
        }
        let resolved = url.path

        // 2. Reject empty and root explicitly.
        if resolved.isEmpty || resolved == "/" {
            return .blocked(reason: "Refusing to delete the filesystem root.")
        }

        // 3. Reject anything at or above a system-protected absolute path.
        for sys in systemAbsolute {
            if resolved == sys {
                return .blocked(reason: "System path is protected (\(sys)).")
            }
            if isPathContained(child: sys, in: resolved) {
                // The resolved path *contains* a system path — pathological.
                return .blocked(reason: "Refusing to delete an ancestor of \(sys).")
            }
        }

        // 4. Reject anything at a personal-home protected path or above it.
        let homePath = home().path
        for suffix in personalHomeRelative {
            let full = (homePath as NSString).appendingPathComponent(suffix)
            if resolved == full {
                return .blocked(reason: "Personal folder is protected (~/\(suffix)).")
            }
            if isPathContained(child: full, in: resolved) {
                return .blocked(reason: "Refusing to delete an ancestor of ~/\(suffix).")
            }
        }

        // 5. Reject the home directory itself.
        if resolved == homePath {
            return .blocked(reason: "The home directory itself is protected.")
        }

        // 6. Must live inside the current user's home OR /tmp OR system caches
        //    the user owns. This blocks accidental cross-user access.
        //    (/tmp entries are only allowed if the running user owns them,
        //     validated by the DeletionEngine at delete time.)
        if !isPathContained(child: resolved, in: homePath)
            && !isPathContained(child: resolved, in: "/tmp")
            && !isPathContained(child: resolved, in: "/private/tmp") {
            return .blocked(reason: "Path is outside the current user's home directory.")
        }

        return .ok
    }

    // True if `child` is `parent` OR lives strictly inside `parent`.
    // Uses string prefix on standardized paths — safe because both inputs
    // have been canonicalized (symlinks resolved).
    public static func isPathContained(child: String, in parent: String) -> Bool {
        let c = (child as NSString).standardizingPath
        let p = (parent as NSString).standardizingPath
        if c == p { return true }
        let pWithSlash = p.hasSuffix("/") ? p : p + "/"
        return c.hasPrefix(pWithSlash)
    }

    // Convenience: is `path` allowed to be *scanned* (read-only)?
    // We allow scanning of anything under the home directory — reading is
    // never destructive. The deletion allow-list is stricter.
    public static func isScannable(path: String) -> Bool {
        guard let url = canonical(path) else { return false }
        return isPathContained(child: url.path, in: home().path)
            || isPathContained(child: url.path, in: "/tmp")
            || isPathContained(child: url.path, in: "/private/tmp")
    }
}
