use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::platform;
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Browser caches per platform. We target specific cache subdirectories —
/// never the whole browser Application Support / AppData / profile folder
/// (which holds bookmarks, logins, extensions, cookies).
pub struct BrowserCacheScanner;

struct Target {
    label: &'static str,
    subpath: &'static str,       // path suffix under $HOME
    running_app: Option<&'static str>,
}

#[cfg(target_os = "macos")]
const TARGETS: &[Target] = &[
    Target { label: "Google Chrome cache",       subpath: "Library/Caches/Google/Chrome/Default/Cache",       running_app: Some("Google Chrome") },
    Target { label: "Google Chrome code cache",  subpath: "Library/Caches/Google/Chrome/Default/Code Cache",   running_app: Some("Google Chrome") },
    Target { label: "Google Chrome GPU cache",   subpath: "Library/Caches/Google/Chrome/Default/GPUCache",     running_app: Some("Google Chrome") },
    Target { label: "Safari cache",              subpath: "Library/Caches/com.apple.Safari",                    running_app: Some("Safari") },
    Target { label: "Firefox cache",             subpath: "Library/Caches/Firefox",                             running_app: Some("Firefox") },
    Target { label: "Arc cache",                 subpath: "Library/Caches/company.thebrowser.Browser",          running_app: Some("Arc") },
    Target { label: "Brave cache",               subpath: "Library/Caches/BraveSoftware/Brave-Browser",         running_app: Some("Brave Browser") },
    Target { label: "Microsoft Edge cache",      subpath: "Library/Caches/Microsoft Edge",                      running_app: Some("Microsoft Edge") },
];

#[cfg(target_os = "windows")]
const TARGETS: &[Target] = &[
    Target { label: "Google Chrome cache",       subpath: "AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache",     running_app: Some("chrome.exe") },
    Target { label: "Google Chrome code cache",  subpath: "AppData\\Local\\Google\\Chrome\\User Data\\Default\\Code Cache", running_app: Some("chrome.exe") },
    Target { label: "Google Chrome GPU cache",   subpath: "AppData\\Local\\Google\\Chrome\\User Data\\Default\\GPUCache",   running_app: Some("chrome.exe") },
    Target { label: "Microsoft Edge cache",      subpath: "AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache",     running_app: Some("msedge.exe") },
    Target { label: "Brave cache",               subpath: "AppData\\Local\\BraveSoftware\\Brave-Browser\\User Data\\Default\\Cache", running_app: Some("brave.exe") },
    // Firefox cache dir varies by profile name; we scan the shared Cache2 area.
    Target { label: "Firefox cache",             subpath: "AppData\\Local\\Mozilla\\Firefox\\Profiles",                     running_app: Some("firefox.exe") },
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const TARGETS: &[Target] = &[];

impl Scanner for BrowserCacheScanner {
    fn name(&self) -> &'static str { "Browser caches" }

    fn scan(&self) -> ScanOutcome {
        let home = platform::home();
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for t in TARGETS {
            let full = home.join(t.subpath);
            if !full.exists() { continue; }
            match measure_size(&full) {
                Ok(size) if size >= 20 * 1024 * 1024 => {
                    items.push(ScanItem::new(
                        full.to_string_lossy().into_owned(),
                        size,
                        t.label,
                        Category::BrowserCache,
                        RiskLevel::SafeToClean,
                        "Temporary browser cache — helps pages load faster. Safely rebuilt as you browse.",
                        "Moved to Trash. Browser rebuilds its cache automatically.",
                        t.running_app.map(|s| s.to_string()),
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Browser caches".into(),
                    path: full.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }
        ScanOutcome { items, errors }
    }
}
