use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Code editor caches — VS Code, Cursor, Windsurf, Sublime, JetBrains
/// IDEs (IntelliJ, PyCharm, WebStorm, etc.). These pile up quickly with
/// extension caches, indexing databases, workspace metadata.
///
/// We target only the CACHE subpaths — never the top-level Application
/// Support / config directory (which holds keybindings, extensions,
/// user settings).
pub struct EditorCacheScanner;

struct Target {
    label: &'static str,
    subpath: &'static str,
    risk: RiskLevel,
    explanation: &'static str,
    recovery: &'static str,
    running_app: Option<&'static str>,
}

#[cfg(target_os = "macos")]
const TARGETS: &[Target] = &[
    Target {
        label: "VS Code cached data",
        subpath: "Library/Application Support/Code/CachedData",
        risk: RiskLevel::SafeToClean,
        explanation: "VS Code's per-version cache of pre-parsed extensions. Rebuilt on next launch.",
        recovery: "Moved to Trash. VS Code recreates the cache on next launch.",
        running_app: Some("Visual Studio Code"),
    },
    Target {
        label: "VS Code cache",
        subpath: "Library/Application Support/Code/Cache",
        risk: RiskLevel::SafeToClean,
        explanation: "VS Code's HTTP/asset cache. Safely rebuilt.",
        recovery: "Moved to Trash. VS Code rebuilds the cache automatically.",
        running_app: Some("Visual Studio Code"),
    },
    Target {
        label: "VS Code Code Cache",
        subpath: "Library/Application Support/Code/Code Cache",
        risk: RiskLevel::SafeToClean,
        explanation: "V8 JavaScript code cache. Rebuilt on demand.",
        recovery: "Moved to Trash. VS Code rebuilds it automatically.",
        running_app: Some("Visual Studio Code"),
    },
    Target {
        label: "Cursor cached data",
        subpath: "Library/Application Support/Cursor/CachedData",
        risk: RiskLevel::SafeToClean,
        explanation: "Cursor's per-version cache of pre-parsed extensions. Rebuilt on next launch.",
        recovery: "Moved to Trash. Cursor recreates the cache on next launch.",
        running_app: Some("Cursor"),
    },
    Target {
        label: "Cursor cache",
        subpath: "Library/Application Support/Cursor/Cache",
        risk: RiskLevel::SafeToClean,
        explanation: "Cursor's HTTP/asset cache. Safely rebuilt.",
        recovery: "Moved to Trash. Cursor rebuilds it automatically.",
        running_app: Some("Cursor"),
    },
    Target {
        label: "JetBrains caches",
        subpath: "Library/Caches/JetBrains",
        risk: RiskLevel::Review,
        explanation: "IDE indexes for IntelliJ / PyCharm / WebStorm / etc. Reindexing takes a while on next open.",
        recovery: "Moved to Trash. IDEs will re-index projects on next launch.",
        running_app: None,
    },
    Target {
        label: "Sublime Text cache",
        subpath: "Library/Caches/com.sublimetext.4",
        risk: RiskLevel::SafeToClean,
        explanation: "Sublime Text's cache directory.",
        recovery: "Moved to Trash. Sublime rebuilds it automatically.",
        running_app: Some("Sublime Text"),
    },
];

#[cfg(target_os = "windows")]
const TARGETS: &[Target] = &[
    Target {
        label: "VS Code cached data",
        subpath: "AppData\\Roaming\\Code\\CachedData",
        risk: RiskLevel::SafeToClean,
        explanation: "VS Code's per-version cache. Rebuilt on next launch.",
        recovery: "Moved to Trash. VS Code recreates the cache on next launch.",
        running_app: Some("Code.exe"),
    },
    Target {
        label: "VS Code cache",
        subpath: "AppData\\Roaming\\Code\\Cache",
        risk: RiskLevel::SafeToClean,
        explanation: "VS Code's HTTP/asset cache.",
        recovery: "Moved to Trash. VS Code rebuilds automatically.",
        running_app: Some("Code.exe"),
    },
    Target {
        label: "Cursor cached data",
        subpath: "AppData\\Roaming\\Cursor\\CachedData",
        risk: RiskLevel::SafeToClean,
        explanation: "Cursor's per-version cache. Rebuilt on next launch.",
        recovery: "Moved to Trash. Cursor recreates the cache on next launch.",
        running_app: Some("Cursor.exe"),
    },
    Target {
        label: "JetBrains caches",
        subpath: "AppData\\Local\\JetBrains",
        risk: RiskLevel::Review,
        explanation: "IDE indexes for IntelliJ / PyCharm / WebStorm / etc.",
        recovery: "Moved to Trash. IDEs will re-index projects on next launch.",
        running_app: None,
    },
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const TARGETS: &[Target] = &[];

impl Scanner for EditorCacheScanner {
    fn name(&self) -> &'static str { "Code editors" }

    fn scan(&self) -> ScanOutcome {
        let home = crate::platform::home();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for t in TARGETS {
            let full = home.join(t.subpath);
            if !full.exists() { continue; }
            match measure_size(&full) {
                Ok(size) if size >= 100 * 1024 * 1024 => {
                    items.push(ScanItem::new(
                        full.to_string_lossy().into_owned(),
                        size,
                        t.label,
                        Category::DeveloperCache,
                        t.risk,
                        t.explanation,
                        t.recovery,
                        t.running_app.map(|s| s.to_string()),
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Code editors".into(),
                    path: full.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }
        ScanOutcome { items, errors }
    }
}
