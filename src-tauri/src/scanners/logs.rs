use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::platform;
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Application logs the user owns. macOS: ~/Library/Logs.
/// Windows: no comparable single directory — we skip.
pub struct LogsScanner;

impl Scanner for LogsScanner {
    fn name(&self) -> &'static str { "Application logs" }

    fn scan(&self) -> ScanOutcome {
        let home = platform::home();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        #[cfg(target_os = "macos")]
        let logs_root = home.join("Library").join("Logs");
        #[cfg(not(target_os = "macos"))]
        let logs_root: std::path::PathBuf = home.join(".log-does-not-exist");

        if !logs_root.exists() {
            return ScanOutcome { items, errors };
        }

        match measure_size(&logs_root) {
            Ok(size) if size >= 100 * 1024 * 1024 => {
                items.push(ScanItem::new(
                    logs_root.to_string_lossy().into_owned(),
                    size,
                    "Application logs",
                    Category::Logs,
                    RiskLevel::SafeToClean,
                    "Diagnostic logs written by apps. Safe to remove; apps recreate them as needed.",
                    "Moved to Trash. Apps write new logs as they run.",
                    None,
                ));
            }
            Ok(_) => {}
            Err(e) => errors.push(ScanError {
                scanner: "Application logs".into(),
                path: logs_root.to_string_lossy().into_owned(),
                reason: format!("{}", e),
            }),
        }
        ScanOutcome { items, errors }
    }
}
