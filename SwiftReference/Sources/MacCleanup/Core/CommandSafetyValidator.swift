import Foundation

// Mac Cleanup prefers native FileManager over shell commands. But when a
// shell command *is* generated (e.g. `brew cleanup --dry-run`, `du -sk`),
// it flows through this validator first. Anything matching a dangerous
// pattern is refused with a reason — never executed.
public enum CommandSafetyValidator {

    public enum Verdict: Equatable {
        case ok
        case blocked(reason: String)
    }

    // Fixed patterns we refuse outright, regardless of context.
    // Order matters: earlier rules match first.
    private static let denyContains: [(pattern: String, reason: String)] = [
        ("rm -rf /",              "Refusing rm -rf on filesystem root."),
        ("rm -rf ~",              "Refusing rm -rf on home directory."),
        ("rm -rf $HOME",          "Refusing rm -rf on $HOME."),
        ("rm -rf /System",        "Refusing rm -rf on /System."),
        ("rm -rf /private",       "Refusing rm -rf on /private."),
        ("rm -rf /Library",       "Refusing rm -rf on /Library."),
        ("rm -rf /Applications",  "Refusing rm -rf on /Applications."),
        ("rm -rf /opt/homebrew",  "Refusing rm -rf on Homebrew installation."),
        ("rm -rf /usr",           "Refusing rm -rf on /usr."),
        ("rm -rf /var",           "Refusing rm -rf on /var."),
        ("rm -rf /etc",           "Refusing rm -rf on /etc."),
        ("sudo",                  "Refusing to elevate privileges."),
        ("chmod -R",              "Refusing recursive permission change."),
        ("chown -R",              "Refusing recursive ownership change."),
        ("dd if=",                "Refusing dd operations."),
        ("mkfs",                  "Refusing filesystem create commands."),
        ("diskutil erase",        "Refusing diskutil erase."),
        ("diskutil reformat",     "Refusing diskutil reformat."),
        (":(){ :|:& };:",         "Refusing fork bomb."),
        ("shutdown",              "Refusing shutdown."),
        ("reboot",                "Refusing reboot."),
    ]

    // Additional destructive verbs — rejected unless the command is on the
    // explicit allow-list below (empty for now; we prefer FileManager APIs).
    private static let destructiveVerbs: Set<String> = [
        "rm", "rmdir", "unlink", "mv", "cp", "ln",
    ]

    // Read-only commands we allow to run during scanning.
    private static let readOnlyAllowList: Set<String> = [
        "df", "du", "diskutil", "ls", "stat", "file", "find",
        "brew", "tmutil", "sw_vers", "system_profiler",
        "sysctl", "ps", "pgrep", "mdfind",
    ]

    // Validate a shell command line. `command` is the full argv-style string
    // as it would be passed to /bin/sh -c. Use argv-array execution wherever
    // possible instead; this exists for the paths that still need shell.
    public static func validate(_ command: String) -> Verdict {
        let trimmed = command.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            return .blocked(reason: "Empty command.")
        }

        // 1. Deny explicit dangerous substrings.
        let lowered = trimmed.lowercased()
        for (pattern, reason) in denyContains {
            if lowered.contains(pattern.lowercased()) {
                return .blocked(reason: reason)
            }
        }

        // 2. Deny shell metacharacters that enable command chaining. Read-only
        //    inspection never needs these, so blocking them is safe.
        for meta in [";", "&&", "||", "|", "`", "$(", ">", "<"] {
            if trimmed.contains(meta) {
                return .blocked(reason: "Refusing shell metacharacter '\(meta)'.")
            }
        }

        // 3. First token must be on the read-only allow-list unless it's a
        //    read-only variant of a destructive verb that we explicitly permit
        //    (none right now — deletion goes through FileManager, not shell).
        let firstToken = trimmed.split(separator: " ", maxSplits: 1)
            .first
            .map(String.init) ?? ""
        let bareTool = (firstToken as NSString).lastPathComponent   // strip any leading path

        if destructiveVerbs.contains(bareTool) {
            return .blocked(reason: "Destructive verb '\(bareTool)' is disallowed; use FileManager.")
        }

        if !readOnlyAllowList.contains(bareTool) {
            return .blocked(reason: "Command '\(bareTool)' is not on the read-only allow-list.")
        }

        return .ok
    }
}
