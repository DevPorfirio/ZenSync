use std::path::Path;

pub enum AllowEntry {
    /// UTF-8 text file — sanitized on push, desanitized on pull.
    Text(&'static str),
    /// Binary file — copied verbatim, no transformation.
    Binary(&'static str),
    /// Directory — walked recursively. Files inside are treated as text or
    /// binary based on extension (see `is_text_path`). Denylist applies.
    Dir(&'static str),
}

impl AllowEntry {
    pub fn rel_path(&self) -> &'static str {
        match self {
            AllowEntry::Text(p) | AllowEntry::Binary(p) | AllowEntry::Dir(p) => p,
        }
    }
}

pub const ALLOWLIST: &[AllowEntry] = &[
    // === User preferences ===
    AllowEntry::Text("user.js"),
    AllowEntry::Text("prefs.js"),
    AllowEntry::Text("xulstore.json"),
    AllowEntry::Text("compatibility.ini"),

    // === Extensions ===
    AllowEntry::Text("extensions.json"),
    AllowEntry::Text("extension-preferences.json"),
    AllowEntry::Text("extension-settings.json"),

    // === Handlers, containers, broadcast, sessions metadata ===
    AllowEntry::Text("handlers.json"),
    AllowEntry::Text("containers.json"),
    AllowEntry::Text("broadcast-listeners.json"),
    AllowEntry::Text("sessionCheckpoints.json"),
    AllowEntry::Text("serviceworker.txt"),
    AllowEntry::Text("pkcs11.txt"),
    AllowEntry::Text("shield-preference-experiments.json"),

    // === Bookmarks, history, favicons (SQLite) ===
    AllowEntry::Binary("places.sqlite"),
    AllowEntry::Binary("favicons.sqlite"),

    // === Passwords + security stores (user opted in) ===
    AllowEntry::Text("logins.json"),
    AllowEntry::Binary("key4.db"),
    AllowEntry::Binary("key3.db"),
    AllowEntry::Binary("cert9.db"),

    // === Cookies, forms, permissions, content prefs ===
    AllowEntry::Binary("cookies.sqlite"),
    AllowEntry::Binary("formhistory.sqlite"),
    AllowEntry::Binary("permissions.sqlite"),
    AllowEntry::Binary("content-prefs.sqlite"),

    // === Network/security state ===
    AllowEntry::Binary("AlternateServices.bin"),
    AllowEntry::Binary("SiteSecurityServiceState.bin"),

    // === Search engines ===
    AllowEntry::Binary("search.json.mozlz4"),

    // === Current live session (changes constantly — daemon will commit often) ===
    AllowEntry::Binary("sessionstore.jsonlz4"),

    // === Zen-specific top-level ===
    AllowEntry::Text("zen-themes.json"),
    AllowEntry::Text("zen-keyboard-shortcuts.json"),
    AllowEntry::Binary("zen-sessions.jsonlz4"),
    AllowEntry::Binary("zen-live-folders.jsonlz4"),

    // === Recursive directories ===
    AllowEntry::Dir("chrome"),
    AllowEntry::Dir("bookmarkbackups"),
    AllowEntry::Dir("extension-store"),
    AllowEntry::Dir("security_state"),
    AllowEntry::Dir("sessionstore-backups"),
    AllowEntry::Dir("settings"),
    AllowEntry::Dir("storage"),
    AllowEntry::Dir("zen-sessions-backup"),
];

/// Filenames or filename suffixes that are NEVER copied — these are transient
/// state (lock files, SQLite write-ahead logs, journals) that would either
/// corrupt the target profile or be regenerated automatically.
pub const DENYLIST_NAMES: &[&str] = &[
    ".parentlock",
    "parent.lock",
    "lock",
];

pub const DENYLIST_SUFFIXES: &[&str] = &[
    "-wal",
    "-shm",
    "-journal",
    ".lock",
    ".lck",
];

pub fn is_denied(rel_path: &str) -> bool {
    let filename = Path::new(rel_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(rel_path);

    if DENYLIST_NAMES.iter().any(|d| filename == *d) {
        return true;
    }
    DENYLIST_SUFFIXES.iter().any(|s| filename.ends_with(s))
}

pub fn is_text_path(rel_path: &str) -> bool {
    let ext = Path::new(rel_path)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);
    matches!(
        ext.as_deref(),
        Some("js" | "json" | "css" | "html" | "htm" | "svg" | "xml" | "txt" | "md" | "toml" | "ini")
    )
}
