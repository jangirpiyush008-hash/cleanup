// Single source of truth for what can never be deleted.
// Cross-platform: separate deny-lists for macOS and Windows.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    Blocked(String),
}

pub struct ProtectedPaths;

impl ProtectedPaths {
    /// Home directory for the CURRENT logged-in user. Never hardcode.
    pub fn home() -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
    }

    /// Canonicalize a path: resolve symlinks, normalize `.` and `..`,
    /// expand `~` on Unix. Returns `None` if the path can't be resolved.
    pub fn canonical<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        let p = path.as_ref();
        let expanded = if let Some(s) = p.to_str() {
            if let Some(rest) = s.strip_prefix("~/") {
                Self::home().join(rest)
            } else if s == "~" {
                Self::home()
            } else {
                p.to_path_buf()
            }
        } else {
            p.to_path_buf()
        };
        std::fs::canonicalize(&expanded)
            .ok()
            // If canonicalize fails (path doesn't exist), fall back to the
            // syntactic normalized form. Callers separately check existence
            // before deleting.
            .or_else(|| Some(normalize(&expanded)))
    }

    /// The protected policy check — runs before every deletion attempt.
    pub fn is_allowed_for_deletion<P: AsRef<Path>>(path: P) -> Verdict {
        let Some(canon) = Self::canonical(&path) else {
            return Verdict::Blocked("Path could not be resolved.".into());
        };
        let canon_str = canon.to_string_lossy().to_string();

        // 1. Refuse the filesystem root, empty, or absurdly short paths.
        if canon_str.is_empty() || canon == PathBuf::from("/") || canon == PathBuf::from("\\") {
            return Verdict::Blocked("Refusing to delete the filesystem root.".into());
        }
        #[cfg(windows)]
        if canon_str.len() <= 3 {
            // e.g. "C:\", "D:\"
            return Verdict::Blocked("Refusing to delete a drive root.".into());
        }

        // 2. System absolute paths.
        for sys in Self::system_absolute() {
            let sysp = PathBuf::from(sys);
            if canon == sysp {
                return Verdict::Blocked(format!("System path is protected ({}).", sys));
            }
            if is_ancestor_of(&canon, &sysp) {
                // The path we would delete is an ancestor of a protected path.
                return Verdict::Blocked(format!("Refusing to delete an ancestor of {}.", sys));
            }
        }

        // 3. Personal-home protected relative paths.
        let home = Self::home();
        for rel in Self::personal_relative() {
            let full = home.join(rel);
            if canon == full {
                return Verdict::Blocked(format!("Personal folder is protected (~/{}).", rel));
            }
            if is_ancestor_of(&canon, &full) {
                return Verdict::Blocked(format!("Refusing to delete an ancestor of ~/{}.", rel));
            }
        }

        // 4. Home directory itself.
        if canon == home {
            return Verdict::Blocked("The home directory itself is protected.".into());
        }

        // 5. Must live inside HOME or a system temp we own.
        if !is_within(&canon, &home) && !is_within_any_temp(&canon) {
            return Verdict::Blocked(
                "Path is outside the current user's home directory.".into(),
            );
        }

        Verdict::Ok
    }

    /// Is the path allowed to be *scanned* (read-only)?
    /// More permissive than deletion — reading never destroys.
    pub fn is_scannable<P: AsRef<Path>>(path: P) -> bool {
        let Some(canon) = Self::canonical(&path) else {
            return false;
        };
        is_within(&canon, &Self::home()) || is_within_any_temp(&canon)
    }

    #[cfg(target_os = "macos")]
    fn system_absolute() -> &'static [&'static str] {
        &[
            "/", "/System", "/System/Volumes", "/System/Library",
            "/System/Applications", "/private", "/private/var",
            "/private/etc", "/var", "/etc", "/usr", "/bin", "/sbin",
            "/Applications", "/opt", "/opt/homebrew",
            "/opt/homebrew/Cellar", "/opt/homebrew/var",
            "/opt/homebrew/Caskroom", "/usr/local",
            "/usr/local/Homebrew", "/Library",
            "/Library/Application Support", "/Volumes", "/dev",
            "/cores",
        ]
    }

    #[cfg(target_os = "windows")]
    fn system_absolute() -> &'static [&'static str] {
        &[
            "C:\\", "C:\\Windows", "C:\\Windows\\System32",
            "C:\\Windows\\SysWOW64", "C:\\Program Files",
            "C:\\Program Files (x86)", "C:\\ProgramData",
            "C:\\Boot", "C:\\Recovery", "C:\\Users\\Default",
            "C:\\Users\\Public", "C:\\$Recycle.Bin",
            "C:\\System Volume Information",
        ]
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn system_absolute() -> &'static [&'static str] {
        &["/", "/etc", "/usr", "/bin", "/sbin", "/var", "/root", "/opt"]
    }

    #[cfg(target_os = "macos")]
    fn personal_relative() -> &'static [&'static str] {
        &[
            "Documents", "Desktop", "Downloads", "Pictures", "Movies",
            "Music", "Public",
            "Library/Application Support", "Library/Containers",
            "Library/Group Containers", "Library/Mail",
            "Library/Messages", "Library/Keychains",
            "Library/Mobile Documents", "Library/CloudStorage",
            "Library/PersonalAssistant", "Library/Preferences",
            "Library/Safari", "Library/Cookies", "Library/Passwords",
            "Library/Sync Services", "Library/Reminders",
            "Library/Calendars", "Library/AddressBook",
            ".ssh", ".gnupg", ".aws", ".config",
        ]
    }

    #[cfg(target_os = "windows")]
    fn personal_relative() -> &'static [&'static str] {
        &[
            "Documents", "Desktop", "Downloads", "Pictures", "Videos",
            "Music", "Contacts", "Favorites", "Links",
            "OneDrive",
            "AppData\\Roaming\\Microsoft\\Windows\\Cookies",
            "AppData\\Roaming\\Microsoft\\Signatures",
            "AppData\\Roaming\\Microsoft\\Outlook",
            "AppData\\Roaming\\Mozilla\\Firefox\\Profiles",
            "AppData\\Roaming\\Signal",
            "AppData\\Roaming\\Slack\\storage",
            ".ssh", ".gnupg", ".aws",
        ]
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn personal_relative() -> &'static [&'static str] {
        &["Documents", "Desktop", "Downloads", "Pictures", "Videos",
          "Music", ".ssh", ".gnupg", ".config"]
    }
}

/// Syntactically normalize a path (no filesystem I/O). Handles `.` and `..`.
fn normalize<P: AsRef<Path>>(p: P) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.as_ref().components() {
        use std::path::Component::*;
        match comp {
            CurDir => {}
            ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Is `haystack` inside (or equal to) `container`?
fn is_within(haystack: &Path, container: &Path) -> bool {
    let h = normalize(haystack);
    let c = normalize(container);
    if h == c { return true; }
    h.starts_with(&c)
}

/// The inverse — is `container` inside (or equal to) `haystack`?
/// Used to reject deletions whose target CONTAINS a protected path.
fn is_ancestor_of(haystack: &Path, container: &Path) -> bool {
    let h = normalize(haystack);
    let c = normalize(container);
    if h == c { return true; }
    c.starts_with(&h)
}

fn is_within_any_temp(path: &Path) -> bool {
    #[cfg(unix)]
    let temps = ["/tmp", "/private/tmp", "/var/tmp"];
    #[cfg(windows)]
    let temps = ["C:\\Windows\\Temp"];
    for t in temps {
        if is_within(path, Path::new(t)) {
            return true;
        }
    }
    false
}

// ─── Tests ────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn expect_blocked(p: &str) {
        match ProtectedPaths::is_allowed_for_deletion(p) {
            Verdict::Blocked(_) => {}
            Verdict::Ok => panic!("expected {} to be BLOCKED", p),
        }
    }
    fn expect_ok(p: &str) {
        match ProtectedPaths::is_allowed_for_deletion(p) {
            Verdict::Ok => {}
            Verdict::Blocked(r) => panic!("expected {} to be OK, got: {}", p, r),
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn documents_protected() {
        let home = ProtectedPaths::home();
        expect_blocked(home.join("Documents").to_string_lossy().as_ref());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn system_protected() {
        expect_blocked("/");
        expect_blocked("/System");
        expect_blocked("/private");
        expect_blocked("/Applications");
        expect_blocked("/opt/homebrew");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn npm_cache_allowed() {
        let home = ProtectedPaths::home();
        expect_ok(home.join(".npm").to_string_lossy().as_ref());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn traversal_attempt_blocked() {
        let home = ProtectedPaths::home();
        // ~/Library/Caches/../../Documents resolves to ~/Documents → blocked
        let path = home.join("Library/Caches/../../Documents");
        expect_blocked(path.to_string_lossy().as_ref());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn other_users_home_blocked() {
        expect_blocked("/Users/AnotherPerson");
        expect_blocked("/Users/AnotherPerson/Documents/whatever.txt");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_system_protected() {
        expect_blocked("C:\\");
        expect_blocked("C:\\Windows");
        expect_blocked("C:\\Program Files");
    }
}
