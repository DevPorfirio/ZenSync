use anyhow::{bail, Context, Result};
use directories::BaseDirs;
use std::path::{Path, PathBuf};

pub fn detect_zen_root() -> Result<PathBuf> {
    let base = BaseDirs::new().context("could not resolve user home directory")?;
    let home = base.home_dir();

    let candidates = [
        home.join(".zen"),
        home.join(".var/app/app.zen_browser.zen/.zen"),
        home.join(".config/zen"),
    ];

    for path in &candidates {
        if path.join("profiles.ini").is_file() {
            return Ok(path.clone());
        }
    }

    bail!(
        "Zen profile root not found. Checked: {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Parse `profiles.ini` and return the active profile directory.
///
/// Zen records the active profile in an `[InstallXXXXX]` section under the
/// `Default=` key — not in the first `[ProfileN]` section. Some profiles
/// have `Default=1` set in their `[Profile]` section but are not actually
/// the active one.
pub fn detect_active_profile(zen_root: &Path) -> Result<PathBuf> {
    let ini_path = zen_root.join("profiles.ini");
    let contents = std::fs::read_to_string(&ini_path)
        .with_context(|| format!("failed to read {}", ini_path.display()))?;

    let mut in_install_section = false;
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_install_section = line.starts_with("[Install");
            continue;
        }
        if in_install_section {
            if let Some(rel) = line.strip_prefix("Default=") {
                return Ok(zen_root.join(rel.trim()));
            }
        }
    }

    bail!(
        "no [Install*] section with `Default=` found in {}",
        ini_path.display()
    )
}
