use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::platform;
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Non-browser application caches under ~/Library/Caches (macOS) or
/// AppData\Local (Windows). Each app's cache is its own ScanItem so the
/// user can pick individually. We deliberately scan cache DIRECTORIES,
/// not application support / containers.
pub struct AppCacheScanner;

impl Scanner for AppCacheScanner {
    fn name(&self) -> &'static str { "Application caches" }

    fn scan(&self) -> ScanOutcome {
        let home = platform::home();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        #[cfg(target_os = "macos")]
        let caches_root = home.join("Library").join("Caches");
        #[cfg(target_os = "windows")]
        let caches_root = home.join("AppData").join("Local");

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let caches_root = home.join(".cache");

        if !caches_root.exists() {
            return ScanOutcome { items, errors };
        }

        // Directories we deliberately skip because they're not "caches"
        // even if they live under Caches. Browsers are handled by
        // BrowserCacheScanner; skipping here avoids double-counting.
        let skip = [
            "com.apple.Safari", "Google", "Firefox", "BraveSoftware",
            "Microsoft Edge", "company.thebrowser.Browser",
            // Windows: browsers + system data we don't want to bulk-treat
            "Google", "Microsoft", "Mozilla",
            // Never bulk-delete this even if it lives under caches
            "Packages", "Temp", "Programs",
        ];

        let entries = match std::fs::read_dir(&caches_root) {
            Ok(e) => e,
            Err(e) => {
                errors.push(ScanError {
                    scanner: "Application caches".into(),
                    path: caches_root.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                });
                return ScanOutcome { items, errors };
            }
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Some(name_os) = path.file_name() else { continue };
            let name = name_os.to_string_lossy().to_string();
            if skip.iter().any(|s| name.eq_ignore_ascii_case(s)) { continue; }
            // Skip hidden entries and non-directories.
            if name.starts_with('.') { continue; }
            let Ok(md) = entry.metadata() else { continue };
            if !md.is_dir() { continue; }

            match measure_size(&path) {
                Ok(size) if size >= 50 * 1024 * 1024 => {
                    items.push(ScanItem::new(
                        path.to_string_lossy().into_owned(),
                        size,
                        format!("{} cache", pretty(&name)),
                        Category::ApplicationCache,
                        RiskLevel::SafeToClean,
                        "Application cache — temporary data the app can rebuild.",
                        "Moved to Trash. The app rebuilds its cache as needed.",
                        None,
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Application caches".into(),
                    path: path.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }

        ScanOutcome { items, errors }
    }
}

/// Turn "com.apple.mail" into "Mail", "Slack" into "Slack", etc.
fn pretty(dir_name: &str) -> String {
    if let Some(last) = dir_name.split('.').last() {
        if !last.is_empty() && last != dir_name {
            let mut c = last.chars();
            return c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_else(|| dir_name.into());
        }
    }
    dir_name.to_string()
}
