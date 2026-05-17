use anyhow::{Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

fn base() -> Result<BaseDirs> {
    BaseDirs::new().context("could not resolve user home directory")
}

pub fn state_dir() -> Result<PathBuf> {
    Ok(base()?.data_local_dir().join("zensync"))
}

pub fn repo_dir() -> Result<PathBuf> {
    Ok(state_dir()?.join("repo"))
}

pub fn backups_dir() -> Result<PathBuf> {
    Ok(state_dir()?.join("backups"))
}
