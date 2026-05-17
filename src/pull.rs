use anyhow::{bail, Context, Result};
use chrono::Local;
use std::path::PathBuf;

use crate::{files, git, lifecycle, profile, sanitize, state, sync};

pub fn run() -> Result<()> {
    let repo = state::repo_dir()?;
    if !repo.join(".git").is_dir() {
        bail!(
            "no repo at {}. Run `zensync init <repo-url>` first.",
            repo.display()
        );
    }

    if lifecycle::zen_is_running() {
        bail!("Zen Browser is running. Close it before `zensync pull` (file locks would corrupt the profile).");
    }

    println!("Pulling from remote...");
    git::pull(&repo)?;

    let zen_root = profile::detect_zen_root()?;
    let profile_dir = profile::detect_active_profile(&zen_root)?;
    println!("Profile: {}", profile_dir.display());

    let backup = make_backup_dir()?;
    println!("Backup: {}", backup.display());

    // Backup current state of every allowlisted entry (identity transform —
    // no sanitization, just an exact copy of what's about to be overwritten).
    let identity = |s: &str| s.to_string();
    for entry in files::ALLOWLIST {
        sync::apply_entry(entry, &profile_dir, &backup, &identity)?;
    }

    let sanitizer = sanitize::Sanitizer::new()?;
    let restore = |s: &str| sanitizer.desanitize(s);

    let mut total = 0u32;
    let mut missing_entries = 0u32;

    for entry in files::ALLOWLIST {
        let n = sync::apply_entry(entry, &repo, &profile_dir, &restore)?;
        if n == 0 {
            missing_entries += 1;
            continue;
        }
        let rel = entry.rel_path();
        let suffix = match entry {
            files::AllowEntry::Dir(_) => format!("{rel}/ ({n} files)"),
            files::AllowEntry::Binary(_) => format!("{rel} (binary)"),
            files::AllowEntry::Text(_) => rel.to_string(),
        };
        println!("  applied {suffix}");
        total += n;
    }

    println!(
        "{total} file(s) applied across {} entries, {missing_entries} entries not in repo",
        files::ALLOWLIST.len() - missing_entries as usize
    );
    Ok(())
}

fn make_backup_dir() -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dir = state::backups_dir()?.join(timestamp);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create {}", dir.display()))?;
    Ok(dir)
}
