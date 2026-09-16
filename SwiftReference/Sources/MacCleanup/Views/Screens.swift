import SwiftUI

// All screen views live in one file for easy discovery — each is short,
// stateless, and driven by AppState. Any screen that could kick off a
// destructive action goes through a callback (never mutates state itself).

// ─── OnboardingView ─────────────────────────────────────────────
struct OnboardingView: View {
    let onContinue: () -> Void
    var body: some View {
        VStack(spacing: 20) {
            Spacer()
            Text("Your privacy matters.")
                .font(.system(size: 30, weight: .semibold, design: .serif))
            Text("Mac Cleanup scans your Mac locally.\nYour files and scan results never leave this Mac.")
                .multilineTextAlignment(.center)
                .foregroundStyle(Theme.inkMuted)
                .font(.system(size: 15))
            Spacer().frame(height: 10)
            PrimaryButton(title: "Continue", action: onContinue)
            Text("Nothing is deleted during scanning. You approve every cleanup.")
                .font(.system(size: 11))
                .foregroundStyle(Theme.inkMuted)
                .padding(.top, 8)
            Spacer()
        }
        .padding(.horizontal, 40)
    }
}

// ─── HomeView (idle) ────────────────────────────────────────────
struct HomeView: View {
    let stats: StorageInspector.VolumeStats
    let onScan: () -> Void

    var body: some View {
        VStack(spacing: 22) {
            Spacer().frame(height: 8)
            Text("Mac Cleanup")
                .font(.system(size: 28, weight: .semibold, design: .serif))
            Text("Find unnecessary files and safely free up space.")
                .foregroundStyle(Theme.inkMuted)
                .font(.system(size: 14))

            StorageRing(stats: stats)
                .padding(.top, 8)

            PrimaryButton(title: "Scan my Mac", action: onScan)
                .padding(.top, 4)
            Text("Nothing will be deleted during the scan.")
                .font(.system(size: 12))
                .foregroundStyle(Theme.inkMuted)
            Spacer()
        }
        .padding(30)
    }
}

// ─── ScanProgressView ───────────────────────────────────────────
struct ScanProgressView: View {
    let currentLabel: String
    var body: some View {
        VStack(spacing: 18) {
            Spacer()
            ProgressView().controlSize(.large)
            Text("Scanning your Mac…")
                .font(.system(size: 20, weight: .semibold, design: .serif))
            Text(currentLabel)
                .font(.system(size: 13, design: .monospaced))
                .foregroundStyle(Theme.inkMuted)
                .frame(maxWidth: 480)
                .multilineTextAlignment(.center)
            Text("This is read-only. Nothing is being deleted.")
                .font(.system(size: 11))
                .foregroundStyle(Theme.inkMuted)
                .padding(.top, 12)
            Spacer()
        }
        .padding(30)
    }
}

// ─── ResultsView ────────────────────────────────────────────────
struct ResultsView: View {
    let report: ScanReport
    @Binding var selection: Set<UUID>
    let onSelectAllSafe: () -> Void
    let onClear: () -> Void
    let onReview: () -> Void
    let onBack: () -> Void

    var selectedTotal: Int64 {
        report.items.filter { selection.contains($0.id) }.map(\.size).reduce(0, +)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            // Header
            HStack {
                Button(action: onBack) { Label("Home", systemImage: "chevron.left") }
                    .buttonStyle(.plain)
                    .foregroundStyle(Theme.inkMuted)
                Spacer()
                Text("Scan complete").font(.system(size: 14, weight: .semibold))
                Spacer()
                Spacer().frame(width: 60)
            }
            .padding(.horizontal, 24).padding(.vertical, 14)
            .background(Theme.panel)

            // Summary tiles
            HStack(spacing: 12) {
                summaryTile("Safe to clean", ByteCountFormatter.humanized(report.totalReclaimable), .safeToClean)
                summaryTile("Review",         ByteCountFormatter.humanized(report.totalReview),      .review)
                summaryTile("Protected",      ByteCountFormatter.humanized(report.totalProtected),   .protectedItem)
            }
            .padding(.horizontal, 24).padding(.top, 20).padding(.bottom, 10)

            HStack(spacing: 12) {
                Button("Select all safe", action: onSelectAllSafe).buttonStyle(.link)
                Button("Clear selection", action: onClear).buttonStyle(.link)
                Spacer()
                if !selection.isEmpty {
                    Text("Selected: \(ByteCountFormatter.humanized(selectedTotal))")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(Theme.inkMuted)
                }
            }
            .padding(.horizontal, 24)

            // Grouped list
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    ForEach(Category.allCases, id: \.self) { cat in
                        let itemsInCat = report.items(category: cat)
                        if !itemsInCat.isEmpty {
                            categorySection(title: cat.rawValue, items: itemsInCat)
                        }
                    }
                    if report.items.isEmpty {
                        Text("Nothing to clean right now — your Mac is tidy.")
                            .foregroundStyle(Theme.inkMuted)
                            .padding(.top, 20)
                    }
                    if !report.errors.isEmpty {
                        DisclosureGroup("\(report.errors.count) locations were skipped") {
                            VStack(alignment: .leading, spacing: 6) {
                                ForEach(report.errors) { err in
                                    Text("• \(err.path) — \(err.reason)")
                                        .font(.system(size: 11, design: .monospaced))
                                        .foregroundStyle(Theme.inkMuted)
                                }
                            }
                            .padding(.top, 6)
                        }
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.inkMuted)
                        .padding(.top, 10)
                    }
                }
                .padding(24)
            }

            // Footer
            Divider()
            HStack {
                Text("Reviewing then approving is the only way anything is removed.")
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.inkMuted)
                Spacer()
                Button("Review \(selection.count) items", action: onReview)
                    .disabled(selection.isEmpty)
                    .keyboardShortcut(.return)
                    .buttonStyle(.borderedProminent)
            }
            .padding(.horizontal, 24).padding(.vertical, 14)
        }
    }

    @ViewBuilder
    private func summaryTile(_ label: String, _ value: String, _ risk: RiskLevel) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label).font(.system(size: 11, weight: .semibold)).foregroundStyle(Theme.color(for: risk))
            Text(value).font(.system(size: 20, weight: .semibold))
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(Theme.color(for: risk).opacity(0.10))
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }

    @ViewBuilder
    private func categorySection(title: String, items: [ScanItem]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title.uppercased())
                .font(.system(size: 11, weight: .heavy)).kerning(0.8)
                .foregroundStyle(Theme.inkMuted)
            ForEach(items) { item in
                itemRow(item)
            }
        }
    }

    @ViewBuilder
    private func itemRow(_ item: ScanItem) -> some View {
        let selectable = item.risk != .protectedItem
        HStack(alignment: .top, spacing: 12) {
            Toggle(isOn: Binding(
                get: { selection.contains(item.id) },
                set: { v in
                    guard selectable else { return }
                    if v { selection.insert(item.id) } else { selection.remove(item.id) }
                }
            )) { EmptyView() }
                .toggleStyle(.checkbox)
                .disabled(!selectable)
                .opacity(selectable ? 1 : 0.3)

            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(item.displayName).font(.system(size: 14, weight: .semibold))
                    RiskBadge(risk: item.risk)
                    if let app = item.requiresAppClosed {
                        Text("• \(app) should be closed")
                            .font(.system(size: 11))
                            .foregroundStyle(Theme.review)
                    }
                    Spacer()
                    Text(item.formattedSize)
                        .font(.system(size: 13, weight: .semibold, design: .monospaced))
                }
                Text(item.explanation).font(.system(size: 12)).foregroundStyle(Theme.inkMuted)
                DisclosureGroup("Details") {
                    VStack(alignment: .leading, spacing: 3) {
                        Text("Location: \(item.path)").font(.system(size: 11, design: .monospaced))
                        Text("Recovery: \(item.recovery)").font(.system(size: 11))
                    }
                    .foregroundStyle(Theme.inkMuted)
                    .padding(.top, 4)
                }
                .font(.system(size: 12))
                .foregroundStyle(Theme.inkMuted)
            }
        }
        .padding(12)
        .background(Theme.panel)
        .clipShape(RoundedRectangle(cornerRadius: 10))
    }
}

// ─── ReviewView (assemble cleanup plan) ─────────────────────────
struct ReviewView: View {
    let plan: CleanupPlan
    let onCancel: () -> Void
    let onBack: () -> Void
    let onConfirm: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Button(action: onBack) { Label("Go back", systemImage: "chevron.left") }
                    .buttonStyle(.plain).foregroundStyle(Theme.inkMuted)
                Spacer()
                Text("Review Cleanup").font(.system(size: 14, weight: .semibold))
                Spacer()
                Button("Cancel", action: onCancel).buttonStyle(.plain).foregroundStyle(Theme.inkMuted)
            }
            .padding(.horizontal, 24).padding(.vertical, 14)
            .background(Theme.panel)

            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    Text("You are about to remove")
                        .font(.system(size: 12, weight: .heavy)).kerning(0.8)
                        .foregroundStyle(Theme.inkMuted)
                    ForEach(plan.items) { item in
                        VStack(alignment: .leading, spacing: 4) {
                            HStack {
                                Text(item.displayName).font(.system(size: 14, weight: .semibold))
                                Spacer()
                                Text(item.formattedSize)
                                    .font(.system(size: 13, weight: .semibold, design: .monospaced))
                            }
                            Text(item.path)
                                .font(.system(size: 11, design: .monospaced))
                                .foregroundStyle(Theme.inkMuted)
                        }
                        .padding(12)
                        .background(Theme.panel)
                        .clipShape(RoundedRectangle(cornerRadius: 10))
                    }

                    Divider().padding(.vertical, 8)
                    HStack {
                        Text("Total").font(.system(size: 14, weight: .semibold))
                        Spacer()
                        Text(ByteCountFormatter.humanized(plan.total))
                            .font(.system(size: 18, weight: .semibold, design: .monospaced))
                    }
                    Text("Only the items listed above will be moved to Trash. Nothing else will be touched.")
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.inkMuted)
                        .padding(.top, 4)
                }
                .padding(24)
            }

            Divider()
            HStack {
                Button("Cancel", action: onCancel)
                Button("Go Back", action: onBack)
                Spacer()
                DestructiveButton(title: "Move Selected Items to Trash", action: onConfirm)
            }
            .padding(.horizontal, 24).padding(.vertical, 14)
        }
    }
}

// ─── FinalConfirmSheet ──────────────────────────────────────────
struct FinalConfirmSheet: View {
    let plan: CleanupPlan
    let onCancel: () -> Void
    let onConfirm: () -> Void

    var body: some View {
        VStack(spacing: 18) {
            Spacer()
            Image(systemName: "trash").font(.system(size: 34)).foregroundStyle(Theme.review)
            Text("Move \(plan.items.count) selected item\(plan.items.count == 1 ? "" : "s") to Trash?")
                .font(.system(size: 20, weight: .semibold, design: .serif))
                .multilineTextAlignment(.center)
            Text("Total: \(ByteCountFormatter.humanized(plan.total))")
                .font(.system(size: 14, weight: .semibold, design: .monospaced))
                .foregroundStyle(Theme.inkMuted)
            Text("Items go to Trash. You can restore them until you empty Trash.")
                .font(.system(size: 12))
                .foregroundStyle(Theme.inkMuted)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 380)
            HStack(spacing: 12) {
                Button("Cancel", action: onCancel).keyboardShortcut(.cancelAction)
                DestructiveButton(title: "Move to Trash", action: onConfirm)
            }
            .padding(.top, 6)
            Spacer()
        }
        .padding(30)
    }
}

// ─── CleaningView ───────────────────────────────────────────────
struct CleaningView: View {
    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            ProgressView().controlSize(.large)
            Text("Moving items to Trash…")
                .font(.system(size: 16, weight: .semibold))
            Spacer()
        }
    }
}

// ─── DoneView ───────────────────────────────────────────────────
struct DoneView: View {
    let results: [DeletionResult]
    let stats: StorageInspector.VolumeStats
    let onHome: () -> Void

    private var moved: [DeletionResult]  { results.filter { if case .movedToTrash = $0.outcome { return true }; return false } }
    private var skipped: [DeletionResult] { results.filter { if case .skipped = $0.outcome { return true }; return false } }
    private var failed:  [DeletionResult] { results.filter { if case .failed  = $0.outcome { return true }; return false } }
    private var recovered: Int64          { moved.map(\.bytesReclaimed).reduce(0, +) }

    var body: some View {
        VStack(spacing: 18) {
            Spacer().frame(height: 8)
            Text("Cleanup complete")
                .font(.system(size: 26, weight: .semibold, design: .serif))
            Text("You recovered")
                .font(.system(size: 12, weight: .heavy)).kerning(0.8)
                .foregroundStyle(Theme.inkMuted)
            Text(ByteCountFormatter.humanized(recovered))
                .font(.system(size: 40, weight: .semibold, design: .monospaced))
                .foregroundStyle(Theme.safe)
            Text("Mac now has \(ByteCountFormatter.humanized(stats.free)) free")
                .font(.system(size: 13))
                .foregroundStyle(Theme.inkMuted)

            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    if !moved.isEmpty {
                        SectionHeader(text: "Moved to Trash")
                        ForEach(moved) { r in
                            resultRow(r.item.displayName, r.item.formattedSize, systemImage: "checkmark.circle.fill", color: Theme.safe)
                        }
                    }
                    if !skipped.isEmpty {
                        SectionHeader(text: "Skipped for safety")
                        ForEach(skipped) { r in
                            resultRow(r.item.displayName, reason(r), systemImage: "hand.raised.fill", color: Theme.review)
                        }
                    }
                    if !failed.isEmpty {
                        SectionHeader(text: "Failed")
                        ForEach(failed) { r in
                            resultRow(r.item.displayName, reason(r), systemImage: "exclamationmark.triangle.fill", color: Color.red)
                        }
                    }
                }
                .padding(.horizontal, 24)
            }

            HStack(spacing: 12) {
                Button("Open Trash") {
                    NSWorkspace.shared.open(URL(fileURLWithPath: NSHomeDirectory() + "/.Trash"))
                }
                PrimaryButton(title: "Done", action: onHome)
            }
            .padding(.top, 6)
            .padding(.bottom, 20)
        }
        .padding(.horizontal, 20)
    }

    private func reason(_ r: DeletionResult) -> String {
        switch r.outcome {
        case .skipped(let s): return s
        case .failed(let s):  return s
        case .movedToTrash:   return ""
        }
    }

    @ViewBuilder
    private func resultRow(_ name: String, _ trailing: String, systemImage: String, color: Color) -> some View {
        HStack(alignment: .center, spacing: 10) {
            Image(systemName: systemImage).foregroundStyle(color)
            Text(name).font(.system(size: 13))
            Spacer()
            Text(trailing).font(.system(size: 12, design: .monospaced)).foregroundStyle(Theme.inkMuted)
        }
        .padding(.vertical, 4)
    }
}

private struct SectionHeader: View {
    let text: String
    var body: some View {
        Text(text.uppercased())
            .font(.system(size: 11, weight: .heavy)).kerning(0.8)
            .foregroundStyle(Theme.inkMuted)
            .padding(.top, 12)
    }
}

// AppKit import for Open Trash button.
import AppKit
