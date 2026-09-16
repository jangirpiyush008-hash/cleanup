use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};
use std::path::PathBuf;

/// Homebrew inspector. Reports the *size* of Homebrew's data directories
/// so users see how much space it takes, but classifies every entry as
/// Review — Homebrew is developer infrastructure, not a cache we can bulk-
/// delete. The `brew cleanup` command is what users should run; this app
/// won't do it silently.
///
/// macOS only. Windows has no Homebrew.
pub struct HomebrewScanner;

impl Scanner for HomebrewScanner {
    fn name(&self) -> &'static str { "Homebrew" }

    #[cfg(not(target_os = "macos"))]
    fn scan(&self) -> ScanOutcome {
        ScanOutcome { items: vec![], errors: vec![] }
    }

    #[cfg(target_os = "macos")]
    fn scan(&self) -> ScanOutcome {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        // Detect the Homebrew prefix — Apple Silicon uses /opt/homebrew,
        // Intel Macs use /usr/local. Skip if neither exists.
        let candidates: &[&str] = &["/opt/homebrew", "/usr/local/Homebrew"];
        let prefix: Option<PathBuf> = candidates
            .iter()
            .map(PathBuf::from)
            .find(|p| p.exists());
        let Some(prefix) = prefix else {
            return ScanOutcome { items, errors };
        };

        // Homebrew's *removable-with-brew-cleanup* directory: Cellar
        // downloads cache (`~/Library/Caches/Homebrew`) — that we can
        // actually surface for real cleaning.
        let user_cache = crate::platform::home()
            .join("Library").join("Caches").join("Homebrew");
        if user_cache.exists() {
            match measure_size(&user_cache) {
                Ok(size) if size >= 200 * 1024 * 1024 => {
                    items.push(ScanItem::new(
                        user_cache.to_string_lossy().into_owned(),
                        size,
                        "Homebrew download cache",
                        Category::Homebrew,
                        RiskLevel::SafeToClean,
                        "Downloaded bottle archives. Homebrew re-downloads on next install.",
                        "Moved to Trash. Homebrew re-fetches bottles as needed.",
                        None,
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Homebrew".into(),
                    path: user_cache.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }

        // Report Cellar / Caskroom / var sizes as REVIEW entries.
        // These are protected paths — the user cannot delete them from
        // this app, but seeing the sizes is useful. RiskLevel::Protected
        // means the UI hides the delete checkbox.
        for sub in ["Cellar", "Caskroom", "var"] {
            let path = prefix.join(sub);
            if !path.exists() { continue; }
            match measure_size(&path) {
                Ok(size) if size >= 500 * 1024 * 1024 => {
                    let (label, explanation) = match sub {
                        "Cellar"   => ("Homebrew Cellar", "Installed Homebrew packages. Manage via `brew uninstall` / `brew cleanup`."),
                        "Caskroom" => ("Homebrew Caskroom", "Installed Homebrew casks (GUI apps). Manage via `brew uninstall`."),
                        "var"      => ("Homebrew var", "Runtime state for Homebrew services (databases, logs). Managed by `brew services`."),
                        _ => ("Homebrew", "Homebrew-managed data."),
                    };
                    items.push(ScanItem::new(
                        path.to_string_lossy().into_owned(),
                        size,
                        label,
                        Category::Homebrew,
                        RiskLevel::Protected,
                        explanation,
                        "Mac Cleanup will not delete this — use the `brew` command line to manage it.",
                        None,
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Homebrew".into(),
                    path: path.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }

        ScanOutcome { items, errors }
    }
}
