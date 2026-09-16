use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::safety::protected_paths::{ProtectedPaths, Verdict};
use crate::scanners::{ScanOutcome, Scanner};

/// Files larger than THRESHOLD_BYTES anywhere under $HOME.
/// EVERY match is classified as REVIEW — a large file is not junk.
/// Frequent examples: video downloads, virtual machine disk images,
/// database dumps, older project archives. The user decides.
///
/// We enumerate a shallow set of well-known "large-file heavy" locations
/// rather than walking the entire home tree — that keeps scan time bounded
/// and results relevant. Skipped: anything ProtectedPaths blocks from
/// deletion (Documents, Desktop, iCloud, Photos, etc.). We still list a
/// few Downloads-adjacent buckets marked Protected so the user sees them.
pub struct LargeFilesScanner;

const THRESHOLD_BYTES: u64 = 500 * 1024 * 1024; // 500 MB

impl Scanner for LargeFilesScanner {
    fn name(&self) -> &'static str { "Large files" }

    fn scan(&self) -> ScanOutcome {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        let home = crate::platform::home();

        // Locations we scan for large files. Order matters — first match wins.
        // Personal locations (Downloads, Desktop, Documents, Movies) are
        // scanned so the user SEES what's there, but every file surfaced
        // from them is marked Protected — this app will never let you
        // delete them from here. Nudge the user to Finder instead.
        struct Root {
            path: std::path::PathBuf,
            label: &'static str,
            /// Force Protected regardless of ProtectedPaths result.
            /// Personal roots are Review-in-name but marked Protected in
            /// the UI so no delete button appears.
            personal: bool,
        }

        let roots = vec![
            Root { path: home.join("Downloads"), label: "Downloads",     personal: true },
            Root { path: home.join("Movies"),    label: "Movies",        personal: true },
            #[cfg(target_os = "windows")]
            Root { path: home.join("Videos"),    label: "Videos",        personal: true },
            Root { path: home.join("Desktop"),   label: "Desktop",       personal: true },
        ];

        for root in roots {
            if !root.path.exists() { continue; }
            // Enumerate the immediate directory + one nested level. Any
            // deeper and scan time blows up on large trees.
            let iter = match std::fs::read_dir(&root.path) {
                Ok(i) => i,
                Err(e) => {
                    errors.push(ScanError {
                        scanner: "Large files".into(),
                        path: root.path.to_string_lossy().into_owned(),
                        reason: format!("{}", e),
                    });
                    continue;
                }
            };
            for entry in iter.filter_map(Result::ok) {
                let path = entry.path();
                let Ok(md) = entry.metadata() else { continue };
                if md.file_type().is_symlink() { continue; }

                let size = if md.is_file() {
                    md.len()
                } else if md.is_dir() {
                    match measure_size(&path) {
                        Ok(s) => s,
                        Err(_) => continue,
                    }
                } else {
                    continue
                };

                if size < THRESHOLD_BYTES { continue; }

                let (risk, recovery) = if root.personal
                    || matches!(
                        ProtectedPaths::is_allowed_for_deletion(&path),
                        Verdict::Blocked(_)
                    )
                {
                    (
                        RiskLevel::Protected,
                        "Mac Cleanup won't delete personal files. Open Finder / Explorer to move or delete.",
                    )
                } else {
                    (
                        RiskLevel::Review,
                        "Moved to Trash. If it turns out you needed it, restore from Trash.",
                    )
                };

                let name = entry.file_name().to_string_lossy().into_owned();
                items.push(ScanItem::new(
                    path.to_string_lossy().into_owned(),
                    size,
                    format!("Large file in {}: {}", root.label, name),
                    Category::LargeFile,
                    risk,
                    "Large personal file — this app will not touch personal files, but wanted you to see it.",
                    recovery,
                    None,
                ));
            }
        }

        // Sort largest-first, cap at 40 entries so we don't overwhelm the UI.
        items.sort_by(|a, b| b.size.cmp(&a.size));
        items.truncate(40);

        ScanOutcome { items, errors }
    }
}
