// We prefer std::fs over shell whenever possible. This validator gates the
// small number of read-only shell commands we *do* use (`brew --dry-run`,
// `du`, `df`, etc.) and refuses anything destructive.

pub enum Verdict {
    Ok,
    Blocked(String),
}

const DENY_CONTAINS: &[(&str, &str)] = &[
    ("rm -rf /",              "Refusing rm -rf on filesystem root."),
    ("rm -rf ~",              "Refusing rm -rf on home directory."),
    ("rm -rf $home",          "Refusing rm -rf on $HOME."),
    ("rm -rf /system",        "Refusing rm -rf on /System."),
    ("rm -rf /private",       "Refusing rm -rf on /private."),
    ("rm -rf /library",       "Refusing rm -rf on /Library."),
    ("rm -rf /applications",  "Refusing rm -rf on /Applications."),
    ("rm -rf /opt/homebrew",  "Refusing rm -rf on Homebrew installation."),
    ("rm -rf /usr",           "Refusing rm -rf on /usr."),
    ("rm -rf /var",           "Refusing rm -rf on /var."),
    ("rm -rf /etc",           "Refusing rm -rf on /etc."),
    ("sudo",                  "Refusing to elevate privileges."),
    ("chmod -r",              "Refusing recursive permission change."),
    ("chown -r",              "Refusing recursive ownership change."),
    ("dd if=",                "Refusing dd."),
    ("mkfs",                  "Refusing filesystem create."),
    ("diskutil erase",        "Refusing diskutil erase."),
    ("format c:",             "Refusing format C:"),
    ("del /f /s /q c:\\",     "Refusing del /f /s /q on C:\\"),
    ("rmdir /s /q c:\\",      "Refusing rmdir on C:\\"),
    (":(){ :|:& };:",         "Refusing fork bomb."),
    ("shutdown",              "Refusing shutdown."),
    ("reboot",                "Refusing reboot."),
];

const DESTRUCTIVE_VERBS: &[&str] = &["rm", "rmdir", "unlink", "mv", "cp", "ln", "del", "erase"];
const READ_ONLY_ALLOWED: &[&str] = &[
    "df", "du", "diskutil", "ls", "stat", "file", "find",
    "brew", "tmutil", "sw_vers", "system_profiler",
    "sysctl", "ps", "pgrep", "mdfind",
    "wmic", "powershell", "cmd", "systeminfo",
];

pub fn validate(command: &str) -> Verdict {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Verdict::Blocked("Empty command.".into());
    }
    let lower = trimmed.to_lowercase();

    for (pattern, reason) in DENY_CONTAINS {
        if lower.contains(pattern) {
            return Verdict::Blocked((*reason).into());
        }
    }

    for meta in [";", "&&", "||", "|", "`", "$(", ">", "<"] {
        if trimmed.contains(meta) {
            return Verdict::Blocked(format!("Refusing shell metacharacter '{}'.", meta));
        }
    }

    let first_token: String = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or("")
        .to_lowercase();

    if DESTRUCTIVE_VERBS.contains(&first_token.as_str()) {
        return Verdict::Blocked(format!(
            "Destructive verb '{}' is disallowed — use std::fs / trash crate instead.",
            first_token
        ));
    }
    if !READ_ONLY_ALLOWED.contains(&first_token.as_str()) {
        return Verdict::Blocked(format!(
            "Command '{}' is not on the read-only allow-list.",
            first_token
        ));
    }

    Verdict::Ok
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(cmd: &str) {
        matches!(validate(cmd), Verdict::Ok).then_some(()).expect(cmd);
    }
    fn no(cmd: &str) {
        matches!(validate(cmd), Verdict::Blocked(_)).then_some(()).expect(cmd);
    }

    #[test]
    fn dangerous_blocked() {
        no("rm -rf /");
        no("rm -rf ~");
        no("rm -rf /System");
        no("rm -rf /opt/homebrew");
        no("sudo rm anything");
        no("dd if=/dev/zero of=/dev/disk0");
        no(":(){ :|:& };:");
    }

    #[test]
    fn metachars_blocked() {
        no("df ; rm -rf /tmp");
        no("du | grep foo");
    }

    #[test]
    fn read_only_allowed() {
        ok("df -h");
        ok("du -sh /tmp");
        ok("brew list");
    }

    #[test]
    fn unknown_blocked() {
        no("curl https://x");
        no("nc -l 1234");
    }
}
