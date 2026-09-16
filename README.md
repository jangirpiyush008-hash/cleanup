# Mac Cleanup

A cross-platform (macOS + Windows) safe disk cleaner. Scans read-only,
classifies findings as **Safe / Review / Protected**, and only ever moves
items to Trash / Recycle Bin after explicit confirmation. Fully local —
no network calls.

Built with **Tauri 2 + Rust** for the safety-critical core; the UI is
plain HTML/CSS/JS. Ships as a real native app (`~10 MB`) on both platforms
from one codebase.

---

## Repo layout

```
MacCleanup/
├── src/                       Frontend (HTML/CSS/JS)
├── src-tauri/                 Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands.rs        Tauri IPC surface
│       ├── models.rs
│       ├── platform/          Cross-platform helpers (volume stats, process detect)
│       ├── safety/            ProtectedPaths, DeletionEngine, CommandValidator
│       └── scanners/          One file per category
├── .github/workflows/         Cross-platform builds (macOS + Windows on Actions)
├── SwiftReference/            Archived SwiftUI prototype (kept for reference)
└── package.json               Tauri CLI dependency
```

---

## Requirements

- **Rust 1.75+** (`rustup` — you have 1.95 already).
- **Node 20+** (for `@tauri-apps/cli`; you have 22).
- **macOS**: Xcode Command Line Tools (`xcode-select --install`).
- **Windows**: Visual Studio Build Tools + WebView2. GitHub Actions handles
  this automatically.

## First-time setup

```bash
cd MacCleanup
npm install
```

## Dev mode (hot-reload the UI, auto-rebuild the Rust)

```bash
npm run tauri dev
```

Opens the app in a native window pointing at `src/index.html`.

## Build a distributable

```bash
npm run tauri build
```

Outputs:
- `src-tauri/target/release/bundle/dmg/Mac Cleanup_0.1.0_*.dmg` on macOS
- `src-tauri/target/release/bundle/nsis/Mac Cleanup_0.1.0_x64-setup.exe` on Windows

## Cross-platform build via GitHub Actions

Push a `v*` tag and the workflow at `.github/workflows/build.yml` builds
for both macOS (universal ARM64 + x86_64) and Windows in parallel, then
publishes a draft release with both installers attached.

```bash
git tag v0.1.0
git push origin v0.1.0
```

---

## Safety guarantees

1. **`ProtectedPaths::is_allowed_for_deletion`** runs at delete time on
   every item, independent of the scan-time risk label. Scan bugs cannot
   trick the deletion engine into removing `~/Documents`.
2. **Cross-platform deny-list** — `/System`, `/Applications`, `/opt/homebrew`
   on macOS; `C:\Windows`, `C:\Program Files`, `C:\ProgramData` on Windows.
   Home protected: Documents, Desktop, Downloads, Pictures, Movies, Music,
   Application Support / AppData\Roaming, iCloud, Mail, Messages, Keychains.
3. **Trash only** — uses the cross-platform `trash` crate. macOS → `~/.Trash`,
   Windows → Recycle Bin. Never `rm -rf`. Trash is never emptied.
4. **Symlink escape check** — every item's canonical target is re-validated
   against ProtectedPaths before deletion.
5. **Size sanity check** — if an item's on-disk size changed by more than
   10× between scan and delete, it is skipped as "changed since scan".
6. **No hardcoded usernames** — everything derives from
   `dirs::home_dir()`, works for any logged-in user on any Mac or PC.
7. **No sudo, no privilege escalation, no shell for destructive ops.**
   The tiny amount of shell we still use (Homebrew inspection later) runs
   through `CommandSafetyValidator` first.
8. **Fully local** — no HTTP client in the tree, no analytics, no telemetry.

## Scanners currently shipped

**Package managers & caches**
- `NpmCacheScanner` — `~/.npm`
- `BrowserCacheScanner` — Chrome, Safari, Firefox, Arc, Brave, Edge
  (macOS + Windows path sets)
- `TempFilesScanner` — per-user temp directories
- `AppCacheScanner` — non-browser app caches under `~/Library/Caches`
  / `AppData\Local`
- `LogsScanner` — `~/Library/Logs` (macOS)
- `EditorCacheScanner` — VS Code, Cursor, JetBrains IDEs, Sublime
- `DevCacheScanner` — Xcode DerivedData, CocoaPods, Yarn, pnpm, pip,
  Gradle, Cargo, Playwright

**Developer heavy hitters**
- `HomebrewScanner` — `~/Library/Caches/Homebrew` (safe) plus Cellar
  / Caskroom / var visible-but-protected
- `XcodeSimulatorsScanner` — simulator devices + runtime caches
- `DockerScanner` — Docker.raw / ext4.vhdx and Docker Desktop app data

**Personal (always safe)**
- `IosBackupsScanner` — iPhone / iPad backups (always Protected;
  surfaced only so you know they exist)
- `LargeFilesScanner` — files > 500 MB in Downloads / Movies / Desktop
  (always Protected — this app never deletes personal files)

Adding a new scanner: implement the `Scanner` trait in
`src-tauri/src/scanners/`, then register it in `all_scanners()`.

## Distributing to another Mac / PC

**Without warnings** — sign with an Apple Developer ID + notarize
(macOS), and a code-signing certificate (Windows). Add the secrets to
GitHub Actions and update `tauri.conf.json`.

**Quick share (with warnings)** — send the unsigned `.dmg` / `.exe`.
On first launch macOS asks the user to right-click → Open once; Windows
SmartScreen shows "More info → Run anyway".

## License

Personal / educational use. Do not upload user data anywhere. Ever.
