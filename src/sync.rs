use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::files::{self, AllowEntry};

/// Copy one allowlist entry from `src_base` into `dst_base`, applying
/// `transform` to text content. Returns number of files processed (0 if
/// the entry didn't exist on the source side).
pub fn apply_entry<F: Fn(&str) -> String>(
    entry: &AllowEntry,
    src_base: &Path,
    dst_base: &Path,
    transform: &F,
) -> Result<u32> {
    let rel = entry.rel_path();
    // Belt-and-suspenders: denylist beats allowlist even at the entry level.
    if files::is_denied(rel) {
        return Ok(0);
    }
    let src = src_base.join(rel);
    if !src.exists() {
        return Ok(0);
    }

    match entry {
        AllowEntry::Text(_) => {
            copy_text(&src, &dst_base.join(rel), transform)?;
            Ok(1)
        }
        AllowEntry::Binary(_) => {
            copy_binary(&src, &dst_base.join(rel))?;
            Ok(1)
        }
        AllowEntry::Dir(_) => sync_dir(&src, &dst_base.join(rel), transform),
    }
}

fn sync_dir<F: Fn(&str) -> String>(
    src: &Path,
    dst: &Path,
    transform: &F,
) -> Result<u32> {
    let mut count = 0u32;
    for rel in walk_files(src)? {
        let rel_str = rel.to_string_lossy();
        if files::is_denied(&rel_str) {
            continue;
        }
        let src_file = src.join(&rel);
        let dst_file = dst.join(&rel);
        if files::is_text_path(&rel_str) {
            copy_text(&src_file, &dst_file, transform)?;
        } else {
            copy_binary(&src_file, &dst_file)?;
        }
        count += 1;
    }
    Ok(count)
}

fn walk_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        walk_inner(dir, dir, &mut out)?;
    }
    Ok(out)
}

fn walk_inner(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_inner(&path, base, out)?;
        } else if path.is_file() {
            // strip_prefix can't fail here: we descended from `base`.
            out.push(path.strip_prefix(base).unwrap().to_path_buf());
        }
    }
    Ok(())
}

fn copy_text<F: Fn(&str) -> String>(
    src: &Path,
    dst: &Path,
    transform: &F,
) -> Result<()> {
    let content = std::fs::read_to_string(src)
        .with_context(|| format!("failed to read {} as UTF-8", src.display()))?;
    ensure_parent(dst)?;
    std::fs::write(dst, transform(&content))
        .with_context(|| format!("failed to write {}", dst.display()))?;
    Ok(())
}

fn copy_binary(src: &Path, dst: &Path) -> Result<()> {
    ensure_parent(dst)?;
    std::fs::copy(src, dst)
        .with_context(|| format!("failed to copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}

fn ensure_parent(p: &Path) -> Result<()> {
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    Ok(())
}
