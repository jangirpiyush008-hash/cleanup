// Tauri IPC surface. This is the only bridge between the sandboxed
// webview frontend and the OS-privileged Rust core. Every command here
// returns a serialized model — the frontend never sees a raw path or
// file handle.

use crate::models::*;
use crate::platform;
use crate::safety::deletion::{DeletionEngine, Options as DeleteOptions};
use crate::scanners::{all_scanners, ScanOutcome};
use std::collections::HashMap;
use std::sync::Mutex;

/// Global session state — the last scan report, so cleanup requests can
/// resolve item IDs back to the full ScanItem without the frontend
/// having to send the whole path payload back.
///
/// Guarded by a Mutex, held for the duration of scan / cleanup operations.
pub struct SessionState {
    pub last_report: Mutex<Option<ScanReport>>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self { last_report: Mutex::new(None) }
    }
}

#[tauri::command]
pub async fn volume_stats() -> VolumeStats {
    platform::home_volume_stats()
}

#[tauri::command]
pub async fn scan(state: tauri::State<'_, SessionState>) -> Result<ScanReport, String> {
    let started = chrono::Utc::now();
    let scanners = all_scanners();

    // Run scanners on a blocking pool — each scanner is heavy I/O.
    let outcomes: Vec<(String, ScanOutcome)> = tokio::task::spawn_blocking(move || {
        scanners
            .into_iter()
            .map(|s| (s.name().to_string(), s.scan()))
            .collect()
    })
    .await
    .map_err(|e| format!("scan task failed: {}", e))?;

    let mut items = Vec::new();
    let mut errors = Vec::new();
    for (_, out) in outcomes {
        items.extend(out.items);
        errors.extend(out.errors);
    }

    let vol = platform::home_volume_stats();
    let report = ScanReport {
        started_at: started,
        finished_at: chrono::Utc::now(),
        items,
        errors,
        volume_total: vol.total,
        volume_free: vol.free,
        volume_used: vol.used,
    };

    // Cache for cleanup lookup.
    if let Ok(mut guard) = state.last_report.lock() {
        *guard = Some(report.clone());
    }
    Ok(report)
}

#[tauri::command]
pub async fn is_app_running(name: String) -> bool {
    platform::is_app_running(&name)
}

#[tauri::command]
pub async fn cleanup(
    state: tauri::State<'_, SessionState>,
    req: CleanupRequest,
) -> Result<CleanupSummary, String> {
    // Resolve item IDs back to the full ScanItems from the cached report.
    let items: Vec<ScanItem> = {
        let guard = state.last_report.lock().map_err(|_| "state lock poisoned".to_string())?;
        let report = guard.as_ref().ok_or("No scan has been performed yet.".to_string())?;
        let wanted: std::collections::HashSet<&String> = req.item_ids.iter().collect();
        report.items.iter().filter(|i| wanted.contains(&i.id)).cloned().collect()
    };
    if items.is_empty() {
        return Err("No matching items to clean.".into());
    }

    let results = tokio::task::spawn_blocking(move || {
        DeletionEngine::move_batch_to_trash(items, DeleteOptions::default())
    })
    .await
    .map_err(|e| format!("cleanup task failed: {}", e))?;

    let total_reclaimed: u64 = results
        .iter()
        .filter_map(|r| match r.outcome {
            DeletionOutcome::MovedToTrash => Some(r.bytes_reclaimed),
            _ => None,
        })
        .sum();

    let vol = platform::home_volume_stats();
    Ok(CleanupSummary {
        results,
        total_reclaimed,
        volume_free_after: vol.free,
        volume_used_after: vol.used,
    })
}

#[tauri::command]
pub async fn open_trash() -> Result<(), String> {
    open_trash_platform().map_err(|e| format!("{}", e))
}

#[cfg(target_os = "macos")]
fn open_trash_platform() -> std::io::Result<()> {
    let trash = platform::home().join(".Trash");
    std::process::Command::new("open").arg(trash).spawn()?;
    Ok(())
}
#[cfg(target_os = "windows")]
fn open_trash_platform() -> std::io::Result<()> {
    std::process::Command::new("explorer").arg("shell:RecycleBinFolder").spawn()?;
    Ok(())
}
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn open_trash_platform() -> std::io::Result<()> { Ok(()) }

#[tauri::command]
pub async fn app_meta() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("platform".into(), std::env::consts::OS.into());
    m.insert("arch".into(), std::env::consts::ARCH.into());
    m.insert("home".into(), platform::home().to_string_lossy().into_owned());
    m.insert("version".into(), env!("CARGO_PKG_VERSION").into());
    m
}
