use anyhow::{bail, Result};
use std::process::Command;

use crate::{files, git, profile, sanitize, state, sync};

pub fn run() -> Result<()> {
    let repo = state::repo_dir()?;
    if !repo.join(".git").is_dir() {
        bail!(
            "no repo at {}. Run `zensync init <repo-url>` first.",
            repo.display()
        );
    }

    let zen_root = profile::detect_zen_root()?;
    let profile_dir = profile::detect_active_profile(&zen_root)?;
    println!("Profile: {}", profile_dir.display());

    let sanitizer = sanitize::Sanitizer::new()?;
    let transform = |s: &str| sanitizer.sanitize(s);

    let mut total = 0u32;
    let mut skipped_entries = 0u32;

    for entry in files::ALLOWLIST {
        let n = sync::apply_entry(entry, &profile_dir, &repo, &transform)?;
        if n == 0 {
            skipped_entries += 1;
            continue;
        }
        let rel = entry.rel_path();
        let suffix = match entry {
            files::AllowEntry::Dir(_) => format!("{rel}/ ({n} files)"),
            files::AllowEntry::Binary(_) => format!("{rel} (binary)"),
            files::AllowEntry::Text(_) => rel.to_string(),
        };
        println!("  copied {suffix}");
        total += n;
    }

    println!(
        "{total} file(s) copied across {} entries, {skipped_entries} entries skipped (not present in profile)",
        files::ALLOWLIST.len() - skipped_entries as usize
    );

    if !git::has_changes(&repo)? {
        println!("Repo already up to date — nothing to commit.");
        return Ok(());
    }

    let msg = format!("sync from {}", hostname());
    git::commit_all(&repo, &msg)?;
    println!("Committed: {msg}");

    git::push(&repo)?;
    println!("Pushed to remote.");
    Ok(())
}

fn hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        })
        .unwrap_or_else(|| "unknown-host".to_string())
}
