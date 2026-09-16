// swift-tools-version:5.9
// Mac Cleanup — a local-only, opinionated-safe disk-cleaning utility.
// Built as a SwiftPM executable so it can be compiled without Xcode
// (Command Line Tools + `swift build` is enough). scripts/build-app.sh
// wraps the resulting binary into a proper MacCleanup.app bundle.

import PackageDescription

let package = Package(
    name: "MacCleanup",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "MacCleanup", targets: ["MacCleanup"]),
    ],
    targets: [
        .executableTarget(
            name: "MacCleanup",
            path: "Sources/MacCleanup"
        ),
        .testTarget(
            name: "MacCleanupTests",
            dependencies: ["MacCleanup"],
            path: "Tests/MacCleanupTests"
        ),
    ]
)
