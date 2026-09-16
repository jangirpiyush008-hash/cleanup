use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Xcode iOS/tvOS/watchOS simulator devices.
/// Notoriously huge on developer Macs (often 30–80 GB).
///
/// Classified as Review (not Safe): removing simulators means the user has
/// to re-download runtimes next time they build for that OS. The right
/// tool is Xcode → Devices window or `xcrun simctl delete unavailable`;
/// this scanner surfaces the size and lets the user decide.
///
/// macOS only.
pub struct XcodeSimulatorsScanner;

impl Scanner for XcodeSimulatorsScanner {
    fn name(&self) -> &'static str { "Xcode simulators" }

    #[cfg(not(target_os = "macos"))]
    fn scan(&self) -> ScanOutcome {
        ScanOutcome { items: vec![], errors: vec![] }
    }

    #[cfg(target_os = "macos")]
    fn scan(&self) -> ScanOutcome {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        let home = crate::platform::home();

        let devices = home.join("Library/Developer/CoreSimulator/Devices");
        let caches  = home.join("Library/Developer/CoreSimulator/Caches");

        for (path, label, expl, recovery, risk) in [
            (
                &devices,
                "Xcode simulator devices",
                "Installed iOS / iPadOS / watchOS / tvOS simulator instances plus their apps and data.",
                "Moved to Trash. Xcode recreates simulators on demand; installed simulator apps and their data are lost.",
                RiskLevel::Review,
            ),
            (
                &caches,
                "Xcode simulator caches",
                "Cached data used by simulator runtimes. Safe to reclaim; simulators re-cache on next launch.",
                "Moved to Trash. Simulator runtimes rebuild their caches automatically.",
                RiskLevel::SafeToClean,
            ),
        ] {
            if !path.exists() { continue; }
            match measure_size(path) {
                Ok(size) if size >= 500 * 1024 * 1024 => {
                    items.push(ScanItem::new(
                        path.to_string_lossy().into_owned(),
                        size,
                        label,
                        Category::Xcode,
                        risk,
                        expl,
                        recovery,
                        // Xcode / Simulator should be closed first
                        Some("Simulator".into()),
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Xcode simulators".into(),
                    path: path.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }

        ScanOutcome { items, errors }
    }
}
