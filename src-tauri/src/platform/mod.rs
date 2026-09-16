// Cross-platform helpers for filesystem and process introspection.

use crate::models::VolumeStats;
use std::path::PathBuf;
use sysinfo::{ProcessesToUpdate, System};

/// Home directory of the CURRENT user. Never hardcoded.
pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

/// Free / used / total for the volume that hosts $HOME.
pub fn home_volume_stats() -> VolumeStats {
    volume_stats_for(&home())
}

pub fn volume_stats_for(path: &std::path::Path) -> VolumeStats {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let c = match CString::new(path.as_os_str().as_bytes()) {
            Ok(c) => c,
            Err(_) => return VolumeStats { total: 0, free: 0, used: 0 },
        };
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::statvfs(c.as_ptr(), &mut stat) };
        if rc != 0 {
            return VolumeStats { total: 0, free: 0, used: 0 };
        }
        // f_frsize is the fragment size in bytes; use it for exact bytes.
        let frag = stat.f_frsize as u64;
        let total = (stat.f_blocks as u64).saturating_mul(frag);
        let free  = (stat.f_bavail as u64).saturating_mul(frag);
        let used  = total.saturating_sub(free);
        VolumeStats { total, free, used }
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        let wide: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let mut free_to_caller: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free: u64 = 0;
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR(wide.as_ptr()),
                Some(&mut free_to_caller as *mut _ as *mut _),
                Some(&mut total_bytes as *mut _ as *mut _),
                Some(&mut total_free as *mut _ as *mut _),
            )
        };
        if ok.is_err() {
            return VolumeStats { total: 0, free: 0, used: 0 };
        }
        VolumeStats {
            total: total_bytes,
            free: free_to_caller,
            used: total_bytes.saturating_sub(total_free),
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        VolumeStats { total: 0, free: 0, used: 0 }
    }
}

/// Is a named application currently running?
/// Matches by process name (case-insensitive substring).
pub fn is_app_running(name: &str) -> bool {
    let needle = name.to_lowercase();
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All);
    sys.processes().values().any(|p| {
        let n = p.name().to_string_lossy().to_lowercase();
        n == needle || n.contains(&needle)
    })
}
