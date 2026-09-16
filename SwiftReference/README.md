# Mac Cleanup

A local-only, opinionated-safe disk cleaner for macOS. Scans read-only,
classifies every finding as **Safe to clean / Review / Protected**, and
only ever moves items to **Trash** — never permanent-deletes, never
touches personal folders automatically, never uploads anything anywhere.

Built with Swift 6 + SwiftUI. No Xcode required to build — just
Command Line Tools.

## Requirements

- macOS 13 (Ventura) or newer
- Swift 6 (comes with Xcode Command Line Tools):
  `xcode-select --install`

## Build

```bash
./scripts/build-app.sh
open build/MacCleanup.app
```

That's it. The script produces a proper `.app` bundle in `build/`
with an ad-hoc signature. Copy the whole `.app` folder anywhere — it's
portable across Apple Silicon Macs on macOS 13+.

## Run tests

```bash
swift test
```

The safety test suite covers the 15 critical rules — no protected path can
be deleted, symlinks can't escape their permitted directory, files that
changed between scan and delete are skipped, and so on.

## Distribute to another Mac (without Gatekeeper warnings)

The ad-hoc-signed build works locally but the recipient will see
"unidentified developer" on first launch (they can right-click → Open once).

For a clean install experience, sign with a Developer ID and notarize:

```bash
# 1. Sign with your Developer ID Application certificate
codesign --deep --force --options runtime \
  --sign "Developer ID Application: Your Name (TEAMID)" \
  --entitlements Entitlements.plist \
  build/MacCleanup.app

# 2. Zip and submit for notarization
ditto -c -k --keepParent build/MacCleanup.app build/MacCleanup.zip
xcrun notarytool submit build/MacCleanup.zip \
  --apple-id you@example.com --team-id TEAMID --password APP_PASSWORD --wait

# 3. Staple the ticket
xcrun stapler staple build/MacCleanup.app
```

## Architecture

```
Sources/MacCleanup/
├── App/            MacCleanupApp · AppState
├── Views/          RootView · Theme · Screens (Onboarding/Home/Scan/Results/Review/Done)
├── Core/
│   ├── ProtectedPaths          — single source of truth for what can never be deleted
│   ├── CommandSafetyValidator  — refuses dangerous shell commands
│   ├── DeletionEngine          — Trash-only; re-validates every item at delete time
│   ├── ScannerEngine           — async orchestrator; scanner failures never abort the run
│   ├── StorageInspector        — volume stats via URLResourceKey
│   └── ProcessDetector         — is the browser open right now?
├── Scanners/       one file per category; add new ones + register in AppState.registerScanners()
│   ├── NpmCacheScanner
│   └── BrowserCacheScanner
├── Models/         ScanItem · Category · RiskLevel · CleanupPlan · ScanReport
└── Resources/      Info.plist
```

## Safety guarantees

1. Every deletion re-checks `ProtectedPaths.isAllowedForDeletion(path:)`
   at delete time. The scan-time risk label is a hint, never a permission.
2. Deletions go through `FileManager.trashItem` — items land in `~/.Trash`
   and can be restored. Trash is never emptied automatically.
3. Symlink escape check — if the item's canonical target lands in a
   protected location, it is skipped.
4. Size sanity — if a file's on-disk size ballooned or shrank by more
   than 10× between scan and delete, it is skipped as "changed".
5. No `sudo`, no privilege escalation, no changes to SIP / Gatekeeper /
   TCC / permissions.
6. No hardcoded usernames — everything resolves through
   `FileManager.default.homeDirectoryForCurrentUser`.
7. Fully local — no network calls, no analytics, no telemetry.

## Adding a new scanner

1. Create `Sources/MacCleanup/Scanners/YourScanner.swift` conforming to
   the `Scanner` protocol.
2. Return `ScanItem`s with a `RiskLevel` — the deletion engine's own
   safety checks apply on top of your labels.
3. Register it in `AppState.swift → registerScanners()`.

## What's shipped today vs. still to build

**Shipped in v0.1:**
- Full safety core (ProtectedPaths, CommandSafetyValidator, DeletionEngine)
- Full test suite for the safety rules
- SwiftUI shell with all screens (Onboarding → Home → Scan → Results
  → Review → Confirm → Done)
- Two working scanners: npm cache, browser caches
- Build script that produces `MacCleanup.app`

**Coming next:**
- More scanners: developer caches (pip, gradle, cargo, Xcode DerivedData),
  logs, temp files, Homebrew inspector, Docker inspector, large-file
  finder, duplicate finder
- Cleanup history persistence
- Settings screen
- Proper Developer ID signing + notarization
- App icon

## License

Personal use. Do not upload user data anywhere. Ever.
