use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// System temp directories the current user owns. Never /tmp entries the
/// user doesn't own (checked at delete time by ProtectedPaths).
pub struct TempFilesScanner;

impl Scanner for TempFilesScanner {
    fn name(&self) -> &'static str { "Temporary files" }

    fn scan(&self) -> ScanOutcome {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        // Per-user temp candidates. macOS gives each user a private tmp
        // via TMPDIR — safe to enumerate. Windows uses %TEMP% inside the
        // user's AppData.
        let candidates = user_temp_dirs();

        for dir in candidates {
            if !dir.exists() { continue; }
            let entries = match std::fs::read_dir(&dir) {
                Ok(e) => e,
                Err(e) => {
                    errors.push(ScanError {
                        scanner: "Temporary files".into(),
                        path: dir.to_string_lossy().into_owned(),
                        reason: format!("{}", e),
                    });
                    continue;
                }
            };
            let mut per_dir_total: u64 = 0;
            for entry in entries.filter_map(Result::ok) {
                if let Ok(md) = entry.metadata() {
                    per_dir_total = per_dir_total.saturating_add(if md.is_file() {
                        md.len()
                    } else {
                        measure_size(&entry.path()).unwrap_or(0)
                    });
                }
            }
            if per_dir_total >= 100 * 1024 * 1024 {
                items.push(ScanItem::new(
                    dir.to_string_lossy().into_owned(),
                    per_dir_total,
                    "Temporary files",
                    Category::Temp,
                    RiskLevel::SafeToClean,
                    "Your account's temp directory. Contents are transient scratch data.",
                    "Moved to Trash. Nothing depends on temp files persisting.",
                    None,
                ));
            }
        }
        ScanOutcome { items, errors }
    }
}

fn user_temp_dirs() -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    // std::env::temp_dir is per-user on macOS ($TMPDIR under
    // /var/folders/...) and per-user on Windows (%TEMP% in AppData\Local).
    out.push(std::env::temp_dir());

    #[cfg(target_os = "windows")]
    {
        if let Some(local) = dirs::cache_dir() {
            // %LOCALAPPDATA%\Temp — same as above on Windows usually,
            // but include just in case.
            out.push(local.join("Temp"));
        }
    }
    out.sort(); out.dedup();
    out
}
