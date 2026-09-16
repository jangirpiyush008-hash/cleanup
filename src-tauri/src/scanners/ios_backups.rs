use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// iPhone / iPad backups made through Finder or iTunes.
/// ALWAYS classified as Protected — these are personal user data and this
/// app will never offer to delete them. We surface the size so the user
/// knows they exist and can manage via Finder → Devices → Manage Backups.
///
/// macOS only.
pub struct IosBackupsScanner;

impl Scanner for IosBackupsScanner {
    fn name(&self) -> &'static str { "iOS device backups" }

    #[cfg(not(target_os = "macos"))]
    fn scan(&self) -> ScanOutcome {
        ScanOutcome { items: vec![], errors: vec![] }
    }

    #[cfg(target_os = "macos")]
    fn scan(&self) -> ScanOutcome {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        let home = crate::platform::home();
        let root = home.join("Library/Application Support/MobileSync/Backup");
        if !root.exists() {
            return ScanOutcome { items, errors };
        }
        match measure_size(&root) {
            Ok(size) if size >= 500 * 1024 * 1024 => {
                items.push(ScanItem::new(
                    root.to_string_lossy().into_owned(),
                    size,
                    "iOS device backups",
                    Category::Other,
                    RiskLevel::Protected,
                    "Backups of your iPhone or iPad made through Finder / iTunes. Contains personal data.",
                    "Mac Cleanup will not delete these. Manage them in Finder → your device → Manage Backups.",
                    None,
                ));
            }
            Ok(_) => {}
            Err(e) => errors.push(ScanError {
                scanner: "iOS device backups".into(),
                path: root.to_string_lossy().into_owned(),
                reason: format!("{}", e),
            }),
        }
        ScanOutcome { items, errors }
    }
}
