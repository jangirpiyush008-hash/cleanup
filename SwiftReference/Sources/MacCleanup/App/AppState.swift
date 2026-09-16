import Foundation
import SwiftUI

// The top-level state machine. Every screen transition goes through here so
// the UI can never accidentally jump from "scan" to "cleanup done" without
// the report + user's selection + confirmation.
@MainActor
public final class AppState: ObservableObject {

    public enum Phase: Equatable {
        case onboarding
        case idle                          // home screen, ready to scan
        case scanning(currentScanner: String)
        case results(ScanReport)
        case review(CleanupPlan)
        case confirming(CleanupPlan)
        case cleaning
        case done(results: [DeletionResult])
    }

    @Published public var phase: Phase = .onboarding
    @Published public var volume: StorageInspector.VolumeStats = StorageInspector.homeVolumeStats()
    @Published public var selection = Set<UUID>()

    public init() {
        // Skip onboarding if we've shown it before.
        if UserDefaults.standard.bool(forKey: Keys.onboardingSeen) {
            self.phase = .idle
        }
    }

    private enum Keys {
        static let onboardingSeen = "mc.onboarding.seen"
    }

    public func finishedOnboarding() {
        UserDefaults.standard.set(true, forKey: Keys.onboardingSeen)
        phase = .idle
    }

    public func refreshVolume() {
        volume = StorageInspector.homeVolumeStats()
    }

    public func startScan() {
        selection.removeAll()
        phase = .scanning(currentScanner: "Preparing…")
        Task {
            let engine = ScannerEngine(scanners: registerScanners())
            let report = await engine.run(onProgress: { [weak self] name in
                self?.phase = .scanning(currentScanner: name)
            })
            phase = .results(report)
        }
    }

    public func toggle(_ item: ScanItem) {
        if selection.contains(item.id) { selection.remove(item.id) }
        else { selection.insert(item.id) }
    }

    public func selectAllSafe(from report: ScanReport) {
        selection.formUnion(report.items(risk: .safeToClean).map(\.id))
    }

    public func clearSelection() { selection.removeAll() }

    public func proceedToReview(report: ScanReport) {
        let picked = report.items.filter { selection.contains($0.id) }
        guard !picked.isEmpty else { return }
        phase = .review(CleanupPlan(items: picked))
    }

    public func requestFinalConfirmation(plan: CleanupPlan) {
        phase = .confirming(plan)
    }

    public func performCleanup(plan: CleanupPlan) {
        phase = .cleaning
        Task {
            let results = await Task.detached(priority: .userInitiated) {
                DeletionEngine.moveToTrash(plan: plan)
            }.value
            refreshVolume()
            phase = .done(results: results)
        }
    }

    public func returnHome() {
        selection.removeAll()
        refreshVolume()
        phase = .idle
    }
}

// Every scanner we ship, registered in one place. Add new scanners here.
func registerScanners() -> [Scanner] {
    [
        NpmCacheScanner(),
        BrowserCacheScanner(),
    ]
}
