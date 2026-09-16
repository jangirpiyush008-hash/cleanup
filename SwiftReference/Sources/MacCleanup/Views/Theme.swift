import SwiftUI

// One place for colors + shared UI primitives. Keeps the design coherent
// across all screens without ad-hoc styling.
enum Theme {
    static let background = Color(nsColor: .windowBackgroundColor)
    static let panel = Color(nsColor: .underPageBackgroundColor)
    static let ink = Color.primary
    static let inkMuted = Color.secondary
    static let accent = Color(red: 0.20, green: 0.55, blue: 0.95)     // trust blue

    static let safe = Color(red: 0.14, green: 0.60, blue: 0.35)
    static let review = Color(red: 0.85, green: 0.55, blue: 0.10)
    static let protectedItem = Color(red: 0.55, green: 0.55, blue: 0.58)

    static func color(for risk: RiskLevel) -> Color {
        switch risk {
        case .safeToClean:   return safe
        case .review:        return review
        case .protectedItem: return protectedItem
        }
    }
}

// A pill badge for risk levels.
struct RiskBadge: View {
    let risk: RiskLevel
    var body: some View {
        Text(risk.displayName.uppercased())
            .font(.system(size: 10, weight: .heavy))
            .kerning(0.6)
            .foregroundStyle(Theme.color(for: risk))
            .padding(.horizontal, 8).padding(.vertical, 3)
            .background(Theme.color(for: risk).opacity(0.14))
            .clipShape(Capsule())
    }
}

// A ring-style storage indicator on the home screen.
struct StorageRing: View {
    let stats: StorageInspector.VolumeStats
    var body: some View {
        ZStack {
            Circle()
                .stroke(Theme.panel, lineWidth: 14)
            Circle()
                .trim(from: 0, to: stats.usedFraction)
                .stroke(Theme.accent, style: .init(lineWidth: 14, lineCap: .round))
                .rotationEffect(.degrees(-90))
            VStack(spacing: 2) {
                Text(ByteCountFormatter.humanized(stats.used))
                    .font(.system(size: 26, weight: .semibold, design: .rounded))
                Text("of \(ByteCountFormatter.humanized(stats.total)) used")
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.inkMuted)
                Text("\(ByteCountFormatter.humanized(stats.free)) free")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(Theme.safe)
                    .padding(.top, 2)
            }
        }
        .frame(width: 170, height: 170)
    }
}

// A primary button with our accent.
struct PrimaryButton: View {
    let title: String
    let action: () -> Void
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(.white)
                .padding(.horizontal, 22).padding(.vertical, 12)
                .background(Theme.accent)
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}

// A destructive-styled button (used only for "Move to Trash").
struct DestructiveButton: View {
    let title: String
    let action: () -> Void
    var body: some View {
        Button(action: action) {
            Text(title)
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(.white)
                .padding(.horizontal, 22).padding(.vertical, 12)
                .background(Color(red: 0.80, green: 0.22, blue: 0.20))
                .clipShape(Capsule())
        }
        .buttonStyle(.plain)
    }
}
