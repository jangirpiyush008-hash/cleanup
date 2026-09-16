import SwiftUI

// SwiftPM executables need @main and a plain top-level struct that
// conforms to App. The App scene wires our AppState into the root view.
@main
struct MacCleanupApp: App {
    @StateObject private var state = AppState()

    var body: some Scene {
        Window("Mac Cleanup", id: "main") {
            RootView()
                .environmentObject(state)
                .frame(minWidth: 780, minHeight: 560)
                .background(Theme.background)
        }
        .windowResizability(.contentSize)
        .commands {
            CommandGroup(replacing: .newItem) {}   // no "New" menu — this is not a document app
        }
    }
}
