use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::platform;
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// npm cache — downloaded package data. npm re-downloads on demand.
/// Cross-platform: uses the platform home directory, same relative subpath.
pub struct NpmCacheScanner;

impl Scanner for NpmCacheScanner {
    fn name(&self) -> &'static str { "npm cache" }

    fn scan(&self) -> ScanOutcome {
        let home = platform::home();
        let candidate = home.join(".npm");
        let mut items = Vec::new();
        let mut errors = Vec::new();

        if !candidate.exists() {
            return ScanOutcome { items, errors };
        }

        match measure_size(&candidate) {
            Ok(size) if size >= 10 * 1024 * 1024 => {
                items.push(ScanItem::new(
                    candidate.to_string_lossy().into_owned(),
                    size,
                    "npm cache",
                    Category::PackageManager,
                    RiskLevel::SafeToClean,
                    "Temporary downloaded package data. npm re-downloads packages as needed.",
                    "Moved to Trash. npm will rebuild the cache on next install.",
                    None,
                ));
            }
            Ok(_) => {}  // too small to bother
            Err(e) => errors.push(ScanError {
                scanner: "npm cache".into(),
                path: candidate.to_string_lossy().into_owned(),
                reason: format!("{}", e),
            }),
        }

        ScanOutcome { items, errors }
    }
}
