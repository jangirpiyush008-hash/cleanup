use crate::models::{Category, RiskLevel, ScanError, ScanItem};
use crate::safety::deletion::measure_size;
use crate::scanners::{ScanOutcome, Scanner};

/// Docker Desktop data. On both platforms the "vm" disk image (`Docker.raw`
/// on macOS, `ext4.vhdx` on Windows) grows to whatever the container disk
/// has ever used. Docker never reclaims it automatically.
///
/// Classified as Review — deleting Docker.raw destroys all containers,
/// volumes and images. The right tool is `docker system prune`; this
/// scanner surfaces the size so the user knows it exists.
pub struct DockerScanner;

impl Scanner for DockerScanner {
    fn name(&self) -> &'static str { "Docker" }

    fn scan(&self) -> ScanOutcome {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        let home = crate::platform::home();

        #[cfg(target_os = "macos")]
        let targets = vec![
            (
                home.join("Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw"),
                "Docker.raw disk image",
                "Docker Desktop's virtual disk. Contains all containers, images and volumes.",
                RiskLevel::Review,
            ),
            (
                home.join("Library/Group Containers/group.com.docker"),
                "Docker Group Container",
                "Docker Desktop's app data.",
                RiskLevel::Protected,
            ),
        ];

        #[cfg(target_os = "windows")]
        let targets = vec![
            (
                home.join("AppData\\Local\\Docker\\wsl\\data\\ext4.vhdx"),
                "Docker WSL disk (ext4.vhdx)",
                "Docker Desktop's WSL disk. Contains all containers, images and volumes.",
                RiskLevel::Review,
            ),
            (
                home.join("AppData\\Roaming\\Docker Desktop"),
                "Docker Desktop app data",
                "Docker Desktop's app data.",
                RiskLevel::Protected,
            ),
        ];

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let targets: Vec<(std::path::PathBuf, &str, &str, RiskLevel)> = vec![];

        for (path, label, expl, risk) in targets {
            if !path.exists() { continue; }
            match measure_size(&path) {
                Ok(size) if size >= 500 * 1024 * 1024 => {
                    items.push(ScanItem::new(
                        path.to_string_lossy().into_owned(),
                        size,
                        label,
                        Category::Docker,
                        risk,
                        expl,
                        if matches!(risk, RiskLevel::Protected) {
                            "Mac Cleanup won't touch this — use Docker Desktop's built-in cleanup or `docker system prune`."
                        } else {
                            "Moved to Trash. All Docker images, containers and volumes will be lost — re-pull as needed."
                        },
                        Some("Docker".into()),
                    ));
                }
                Ok(_) => {}
                Err(e) => errors.push(ScanError {
                    scanner: "Docker".into(),
                    path: path.to_string_lossy().into_owned(),
                    reason: format!("{}", e),
                }),
            }
        }

        ScanOutcome { items, errors }
    }
}
