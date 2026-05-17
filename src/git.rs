use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

fn run(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .with_context(|| format!("failed to spawn `git {}`", args.join(" ")))?;

    if !output.status.success() {
        bail!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn clone(url: &str, dest: &Path) -> Result<()> {
    let dest_str = dest.to_str().context("destination path is not valid UTF-8")?;
    run(&["clone", url, dest_str], None)?;
    Ok(())
}

pub fn pull(repo: &Path) -> Result<()> {
    run(&["pull", "--ff-only"], Some(repo))?;
    Ok(())
}

pub fn has_changes(repo: &Path) -> Result<bool> {
    let out = run(&["status", "--porcelain"], Some(repo))?;
    Ok(!out.trim().is_empty())
}

pub fn commit_all(repo: &Path, message: &str) -> Result<()> {
    run(&["add", "-A"], Some(repo))?;
    run(&["commit", "-m", message], Some(repo))?;
    Ok(())
}

pub fn push(repo: &Path) -> Result<()> {
    run(&["push"], Some(repo))?;
    Ok(())
}
