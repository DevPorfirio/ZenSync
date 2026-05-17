use anyhow::{bail, Context, Result};

use crate::{git, state};

pub fn run(repo_url: &str) -> Result<()> {
    let state_dir = state::state_dir()?;
    let repo_dir = state::repo_dir()?;

    if repo_dir.exists() {
        bail!(
            "repo already exists at {}. Remove it manually if you want to re-init.",
            repo_dir.display()
        );
    }

    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("failed to create {}", state_dir.display()))?;

    println!("Cloning {} into {}", repo_url, repo_dir.display());
    git::clone(repo_url, &repo_dir)?;

    println!("Done. Next:");
    println!("  zensync push     # upload current profile to repo");
    println!("  zensync pull     # apply repo configs to local profile (close Zen first)");
    Ok(())
}
