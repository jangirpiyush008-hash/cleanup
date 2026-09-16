import SwiftUI

// Phase-driven top-level router. Every screen is a leaf that receives just
// what it needs — no phase is allowed to jump forward without going through
// AppState's transition methods.
struct RootView: View {
    @EnvironmentObject var state: AppState

    var body: some View {
        Group {
            switch state.phase {
            case .onboarding:
                OnboardingView(onContinue: { state.finishedOnboarding() })

            case .idle:
                HomeView(
                    stats: state.volume,
                    onScan: { state.startScan() }
                )

            case .scanning(let current):
                ScanProgressView(currentLabel: current)

            case .results(let report):
                ResultsView(
                    report: report,
                    selection: $state.selection,
                    onSelectAllSafe: { state.selectAllSafe(from: report) },
                    onClear: { state.clearSelection() },
                    onReview: { state.proceedToReview(report: report) },
                    onBack: { state.returnHome() }
                )

            case .review(let plan):
                ReviewView(
                    plan: plan,
                    onCancel: { state.returnHome() },
                    onBack: { state.startScan() },     // re-scan
                    onConfirm: { state.requestFinalConfirmation(plan: plan) }
                )

            case .confirming(let plan):
                FinalConfirmSheet(
                    plan: plan,
                    onCancel: { state.phase = .review(plan) },
                    onConfirm: { state.performCleanup(plan: plan) }
                )

            case .cleaning:
                CleaningView()

            case .done(let results):
                DoneView(
                    results: results,
                    stats: state.volume,
                    onHome: { state.returnHome() }
                )
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .animation(.easeInOut(duration: 0.18), value: phaseKey(state.phase))
    }

    // Provide a stable identity for animation transitions between phases
    // without exposing the associated values.
    private func phaseKey(_ p: AppState.Phase) -> String {
        switch p {
        case .onboarding: return "onboarding"
        case .idle: return "idle"
        case .scanning: return "scanning"
        case .results: return "results"
        case .review: return "review"
        case .confirming: return "confirming"
        case .cleaning: return "cleaning"
        case .done: return "done"
        }
    }
}
