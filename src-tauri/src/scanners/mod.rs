// Every category has a Scanner. Scanners are read-only, side-effect free,
// and return ScanItems + optional ScanErrors. Failures never abort the
// whole scan — one bad path is one skipped entry, not a crash.

use crate::models::{ScanError, ScanItem};

pub mod npm_cache;
pub mod browser_cache;
pub mod temp_files;
pub mod app_cache;
pub mod logs;
pub mod dev_cache;

pub struct ScanOutcome {
    pub items: Vec<ScanItem>,
    pub errors: Vec<ScanError>,
}

pub trait Scanner: Send + Sync {
    fn name(&self) -> &'static str;
    fn scan(&self) -> ScanOutcome;
}

/// The registered set of scanners the app runs by default.
/// Add a new scanner: implement Scanner, then append here.
pub fn all_scanners() -> Vec<Box<dyn Scanner>> {
    vec![
        Box::new(npm_cache::NpmCacheScanner),
        Box::new(browser_cache::BrowserCacheScanner),
        Box::new(temp_files::TempFilesScanner),
        Box::new(app_cache::AppCacheScanner),
        Box::new(logs::LogsScanner),
        Box::new(dev_cache::DevCacheScanner),
    ]
}
