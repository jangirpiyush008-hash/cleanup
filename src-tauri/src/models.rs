// Shared data types. Serde-serializable so the frontend receives them as
// plain JSON via Tauri IPC. The frontend never touches the filesystem
// directly — it only asks the Rust core to do things and displays the
// results.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    SafeToClean,
    Review,
    Protected,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            RiskLevel::SafeToClean => "Safe to clean",
            RiskLevel::Review => "Review",
            RiskLevel::Protected => "Protected",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    BrowserCache,
    ApplicationCache,
    DeveloperCache,
    PackageManager,
    Logs,
    Temp,
    LargeFile,
    Duplicate,
    Xcode,
    Homebrew,
    Docker,
    Snapshots,
    SystemManaged,
    Other,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::BrowserCache => "Browser caches",
            Category::ApplicationCache => "Application caches",
            Category::DeveloperCache => "Developer caches",
            Category::PackageManager => "Package managers",
            Category::Logs => "Logs",
            Category::Temp => "Temporary files",
            Category::LargeFile => "Large files",
            Category::Duplicate => "Duplicates",
            Category::Xcode => "Xcode",
            Category::Homebrew => "Homebrew",
            Category::Docker => "Docker",
            Category::Snapshots => "System snapshots",
            Category::SystemManaged => "System managed",
            Category::Other => "Other",
        }
    }
}

/// One discovered file or directory. `path` is a canonical absolute path
/// captured at scan time. `id` is a deterministic hash of the path so the
/// frontend can identify the item across events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub display_name: String,
    pub category: Category,
    pub risk: RiskLevel,
    pub explanation: String,
    pub recovery: String,
    /// Which application (by user-facing name) should be closed before
    /// cleaning this item. `None` if any running state is fine.
    pub requires_app_closed: Option<String>,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

impl ScanItem {
    pub fn new(
        path: String,
        size: u64,
        display_name: impl Into<String>,
        category: Category,
        risk: RiskLevel,
        explanation: impl Into<String>,
        recovery: impl Into<String>,
        requires_app_closed: Option<String>,
    ) -> Self {
        let id = format!("{:x}", md5_ish(&path));
        Self {
            id,
            path,
            size,
            display_name: display_name.into(),
            category,
            risk,
            explanation: explanation.into(),
            recovery: recovery.into(),
            requires_app_closed,
            discovered_at: chrono::Utc::now(),
        }
    }
}

// Small deterministic non-cryptographic hash — good enough for stable IDs
// without pulling in a crypto crate. Not used for anything security-sensitive.
fn md5_ish(s: &str) -> u128 {
    let mut h: u128 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h = h.wrapping_mul(0x00000100000001B3);
        h ^= b as u128;
    }
    h
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanReport {
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: chrono::DateTime<chrono::Utc>,
    pub items: Vec<ScanItem>,
    pub errors: Vec<ScanError>,
    pub volume_total: u64,
    pub volume_free: u64,
    pub volume_used: u64,
}

impl ScanReport {
    pub fn totals_by_risk(&self, risk: RiskLevel) -> u64 {
        self.items.iter().filter(|i| i.risk == risk).map(|i| i.size).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    pub scanner: String,
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRequest {
    pub item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "outcome", content = "detail")]
#[serde(rename_all = "kebab-case")]
pub enum DeletionOutcome {
    MovedToTrash,
    Skipped { reason: String },
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionResult {
    pub item: ScanItem,
    pub outcome: DeletionOutcome,
    pub bytes_reclaimed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSummary {
    pub results: Vec<DeletionResult>,
    pub total_reclaimed: u64,
    pub volume_free_after: u64,
    pub volume_used_after: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VolumeStats {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

/// Progress event emitted while scanning.
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub current_scanner: String,
    pub items_found_so_far: usize,
}
