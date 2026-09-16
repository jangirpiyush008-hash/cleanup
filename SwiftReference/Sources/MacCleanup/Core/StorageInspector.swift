import Foundation

// Volume stats for the main storage indicator on the home screen.
// Uses URL resource values — no shell required.
public enum StorageInspector {

    public struct VolumeStats: Sendable {
        public let total: Int64
        public let free: Int64             // "available" — what the OS reports
        public let used: Int64             // total - free
        public var usedFraction: Double {
            total > 0 ? Double(used) / Double(total) : 0
        }
    }

    public static func homeVolumeStats() -> VolumeStats {
        let url = FileManager.default.homeDirectoryForCurrentUser
        let keys: Set<URLResourceKey> = [
            .volumeTotalCapacityKey,
            .volumeAvailableCapacityForImportantUsageKey,
            .volumeAvailableCapacityKey,
        ]
        guard let values = try? url.resourceValues(forKeys: keys) else {
            return .init(total: 0, free: 0, used: 0)
        }
        let total = Int64(values.volumeTotalCapacity ?? 0)
        let free = values.volumeAvailableCapacityForImportantUsage.map(Int64.init)
            ?? Int64(values.volumeAvailableCapacity ?? 0)
        return .init(total: total, free: free, used: max(0, total - free))
    }
}
