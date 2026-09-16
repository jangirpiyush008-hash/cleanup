use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::platform;
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Developer tool caches. Each is optional — we only surface it if
/// present with meaningful size. Every one is classified as Review
/// (not Safe-to-clean) because losing them means a slower next build /
/// bigger re-download.
pub struct DevCacheScanner;

struct Target {
    label: &'static str,
    subpath: &'static str,
    explanation: &'static str,
    recovery: &'static str,
    risk: RiskLevel,
}

#[cfg(target_os = "macos")]
const TARGETS: &[Target] = &[
    Target {
        label: "Xcode Derived Data",
        subpath: "Library/Developer/Xcode/DerivedData",
        explanation: "Xcode's build cache. Removing forces a full rebuild of every project.",
        recovery: "Moved to Trash. Xcode rebuilds derived data on next build.",
        risk: RiskLevel::Review,
    },
    Target {
        label: "Xcode Archives",
        subpath: "Library/Developer/Xcode/Archives",
        // Archives contain historical release builds — do NOT auto-clean.
        explanation: "Old Xcode archives. May be needed to resubmit or debug prior releases.",
        recovery: "Moved to Trash. You can restore from Trash if you need the archive back.",
        risk: RiskLevel::Review,
    },
    Target {
        label: "CocoaPods cache",
        subpath: "Library/Caches/CocoaPods",
        explanation: "Downloaded pod specs and archives. Rebuilt on next `pod install`.",
        recovery: "Moved to Trash. `pod install` will re-fetch as needed.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "Yarn cache",
        subpath: "Library/Caches/Yarn",
        explanation: "Yarn's downloaded package cache. Yarn re-downloads on demand.",
        recovery: "Moved to Trash. Yarn rebuilds the cache on next install.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "pnpm cache",
        subpath: "Library/pnpm/store",
        explanation: "pnpm's global store of downloaded packages.",
        recovery: "Moved to Trash. pnpm re-fetches packages on next install.",
        risk: RiskLevel::Review,
    },
    Target {
        label: "pip cache",
        subpath: "Library/Caches/pip",
        explanation: "pip's wheel cache. pip re-downloads as needed.",
        recovery: "Moved to Trash. pip rebuilds the cache on next install.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "Gradle cache",
        subpath: ".gradle/caches",
        explanation: "Gradle's downloaded dependency + build cache. Large but rebuildable.",
        recovery: "Moved to Trash. Gradle re-downloads on next build.",
        risk: RiskLevel::Review,
    },
    Target {
        label: "Cargo registry cache",
        subpath: ".cargo/registry/cache",
        explanation: "Downloaded crate tarballs. Cargo re-fetches on demand.",
        recovery: "Moved to Trash. Cargo re-downloads the crates on next build.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "Playwright browsers",
        subpath: "Library/Caches/ms-playwright",
        explanation: "Bundled browser binaries used by Playwright tests.",
        recovery: "Moved to Trash. `npx playwright install` reinstalls them.",
        risk: RiskLevel::Review,
    },
];

#[cfg(target_os = "windows")]
const TARGETS: &[Target] = &[
    Target {
        label: "Yarn cache",
        subpath: "AppData\\Local\\Yarn\\Cache",
        explanation: "Yarn's downloaded package cache.",
        recovery: "Moved to Trash. Yarn rebuilds the cache on next install.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "pnpm store",
        subpath: "AppData\\Local\\pnpm\\store",
        explanation: "pnpm's global store of downloaded packages.",
        recovery: "Moved to Trash. pnpm re-fetches packages on next install.",
        risk: RiskLevel::Review,
    },
    Target {
        label: "pip cache",
        subpath: "AppData\\Local\\pip\\Cache",
        explanation: "pip's wheel cache. pip re-downloads as needed.",
        recovery: "Moved to Trash. pip rebuilds the cache on next install.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "Gradle cache",
        subpath: ".gradle\\caches",
        explanation: "Gradle's downloaded dependency + build cache.",
        recovery: "Moved to Trash. Gradle re-downloads on next build.",
        risk: RiskLevel::Review,
    },
    Target {
        label: "Cargo registry cache",
        subpath: ".cargo\\registry\\cache",
        explanation: "Downloaded crate tarballs.",
        recovery: "Moved to Trash. Cargo re-downloads on next build.",
        risk: RiskLevel::SafeToClean,
    },
    Target {
        label: "Playwright browsers",
        subpath: "AppData\\Local\\ms-playwright",
        explanation: "Bundled browser binaries used by Playwright tests.",
        recovery: "Moved to Trash. `npx playwright install` reinstalls them.",
        risk: RiskLevel::Review,
    },
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const TARGETS: &[Target] = &[];

impl Scanner for DevCacheScanner {
    fn name(&self) -> &'static str { "Developer caches" }

    fn scan(&self) -> ScanOutcome {
        let home = platform::home();
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
                        None,
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Developer caches".into(),
                    path: full.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }
        ScanOutcome { items, errors }
    }
}
