import Foundation
import AppKit

// Is a given application running right now? Used before offering to clean
// caches that a live app might currently be writing to.
public enum ProcessDetector {

    // Match against NSRunningApplication.localizedName. Returns true if the
    // app appears to be running for the current user.
    public static func isAppRunning(named name: String) -> Bool {
        let normalized = name.lowercased()
        return NSWorkspace.shared.runningApplications.contains {
            let local = ($0.localizedName ?? "").lowercased()
            let bundle = ($0.bundleIdentifier ?? "").lowercased()
            return local == normalized
                || local.contains(normalized)
                || bundle.contains(normalized)
        }
    }
}
