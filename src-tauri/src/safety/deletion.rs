// Moves selected items to Trash / Recycle Bin. Never permanent-delete.
// Re-validates every item at delete time against ProtectedPaths.

use crate::models::{DeletionOutcome, DeletionResult, ScanItem};
use crate::safety::protected_paths::{ProtectedPaths, Verdict};
use std::path::Path;

pub struct DeletionEngine;

pub struct Options {
    pub ignore_size_mismatch: bool,
}
impl Default for Options {
    fn default() -> Self { Self { ignore_size_mismatch: false } }
}

impl DeletionEngine {
    /// Move a batch to Trash. Per-item failures never abort the batch.
    pub fn move_batch_to_trash(items: Vec<ScanItem>, opts: Options) -> Vec<DeletionResult> {
        items.into_iter().map(|i| Self::move_one(i, &opts)).collect()
    }

    pub fn move_one(item: ScanItem, opts: &Options) -> DeletionResult {
        // 1. Re-check ProtectedPaths at delete time — the scan-time risk
        //    label is advisory only. Belt-and-braces.
        match ProtectedPaths::is_allowed_for_deletion(&item.path) {
            Verdict::Blocked(reason) => {
                return DeletionResult {
                    item,
                    outcome: DeletionOutcome::Skipped { reason },
                    bytes_reclaimed: 0,
                };
            }
            Verdict::Ok => {}
        }

        let path = Path::new(&item.path);

        // 2. Must exist.
        if !path.exists() {
            return DeletionResult {
                item,
                outcome: DeletionOutcome::Skipped { reason: "Path no longer exists.".into() },
                bytes_reclaimed: 0,
            };
        }

        // 3. Symlink escape check — resolve to canonical and re-validate.
        let canonical = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                return DeletionResult {
                    item,
                    outcome: DeletionOutcome::Skipped {
                        reason: format!("Could not resolve path: {}", e),
                    },
                    bytes_reclaimed: 0,
                };
            }
        };
        match ProtectedPaths::is_allowed_for_deletion(&canonical) {
            Verdict::Blocked(reason) => {
                return DeletionResult {
                    item,
                    outcome: DeletionOutcome::Skipped {
                        reason: format!("Symlink target rejected: {}", reason),
                    },
                    bytes_reclaimed: 0,
                };
            }
            Verdict::Ok => {}
        }

        // 4. Size sanity — if the item changed drastically between scan
        //    and delete, treat as suspicious and skip.
        if !opts.ignore_size_mismatch && item.size > 0 {
            if let Ok(current) = measure_size(&canonical) {
                let ratio = current as f64 / item.size as f64;
                if !(0.05..=10.0).contains(&ratio) {
                    return DeletionResult {
                        item,
                        outcome: DeletionOutcome::Skipped {
                            reason: "Size changed significantly since scan; skipped for safety.".into(),
                        },
                        bytes_reclaimed: 0,
                    };
                }
            }
        }

        // 5. Move to Trash via the cross-platform `trash` crate.
        //    On macOS this uses NSFileManager, on Windows the Recycle Bin.
        //    Never rm; never permanent delete.
        match trash::delete(&canonical) {
            Ok(()) => DeletionResult {
                bytes_reclaimed: item.size,
                item,
                outcome: DeletionOutcome::MovedToTrash,
            },
            Err(e) => DeletionResult {
                item,
                outcome: DeletionOutcome::Failed { reason: format!("{}", e) },
                bytes_reclaimed: 0,
            },
        }
    }
}

/// Recursive size measurement. Skips files it can't read rather than failing.
pub fn measure_size(path: &Path) -> std::io::Result<u64> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        // Never follow symlinks when measuring — we count the link itself.
        return Ok(meta.len());
    }
    if meta.is_file() {
        return Ok(meta.len());
    }
    let mut total: u64 = 0;
    for entry in walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if let Ok(md) = entry.metadata() {
            if md.is_file() {
                total = total.saturating_add(md.len());
            }
        }
    }
    Ok(total)
}

// ─── Tests ────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, RiskLevel, ScanItem};
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    fn tmp_dir() -> PathBuf {
        let p = std::env::temp_dir().join(format!("mc-tests-{}", uuid_ish()));
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn uuid_ish() -> String {
        format!("{:x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }
    fn item(path: PathBuf, size: u64) -> ScanItem {
        ScanItem::new(
            path.to_string_lossy().into_owned(),
            size,
            "test",
            Category::Other,
            RiskLevel::SafeToClean,
            "test",
            "test",
            None,
        )
    }

    #[test]
    fn protected_input_is_skipped() {
        let bad = ScanItem::new(
            "/System".into(),
            10_000,
            "system",
            Category::Other,
            RiskLevel::SafeToClean,   // mislabelled on purpose
            "", "", None,
        );
        let r = DeletionEngine::move_one(bad, &Options::default());
        assert!(matches!(r.outcome, DeletionOutcome::Skipped { .. }));
    }

    #[test]
    fn missing_path_is_skipped_not_failed() {
        let ghost = std::env::temp_dir().join("mc-does-not-exist");
        let it = item(ghost, 100);
        let r = DeletionEngine::move_one(it, &Options::default());
        assert!(matches!(r.outcome, DeletionOutcome::Skipped { .. }));
    }

    #[test]
    fn size_mismatch_skipped() {
        let dir = tmp_dir();
        let path = dir.join("cache.bin");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(&[0u8; 128]).unwrap();
        // Claim scan-time size was 1MB when it's actually 128 bytes.
        let it = item(path.clone(), 1_000_000);
        let r = DeletionEngine::move_one(it, &Options::default());
        assert!(matches!(r.outcome, DeletionOutcome::Skipped { .. }));
        // Cleanup
        fs::remove_dir_all(dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_refused() {
        let dir = tmp_dir();
        let link = dir.join("trap");
        std::os::unix::fs::symlink("/System", &link).unwrap();
        let it = item(link, 0);
        let r = DeletionEngine::move_one(it, &Options { ignore_size_mismatch: true });
        assert!(matches!(r.outcome, DeletionOutcome::Skipped { .. }));
        fs::remove_dir_all(dir).ok();
    }
}
