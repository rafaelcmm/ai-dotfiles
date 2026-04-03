//! Filesystem traversal and cleanup helpers with symlink-safe behavior.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::constants::{Platform, MANAGED_PREFIX, MANAGED_ROOTS};

/// Collects managed files under a platform root.
///
/// Symlinks are treated as leaf entries and never traversed.
pub(crate) fn collect_existing_managed_files(
    home: &Path,
    platform: Platform,
) -> Result<Vec<PathBuf>> {
    let mut managed = Vec::<PathBuf>::new();
    let platform_root = home.join(platform.root);

    for root in MANAGED_ROOTS {
        let managed_root = platform_root.join(root);
        if !managed_root.exists() {
            continue;
        }

        for entry in fs::read_dir(&managed_root)
            .with_context(|| format!("failed to read managed root {}", managed_root.display()))?
        {
            let entry = entry.with_context(|| {
                format!(
                    "failed to read directory entry in {}",
                    managed_root.display()
                )
            })?;

            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with(MANAGED_PREFIX) {
                continue;
            }

            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .with_context(|| format!("failed to read metadata for {}", path.display()))?;

            if metadata.file_type().is_symlink() {
                managed.push(path);
            } else if metadata.is_dir() {
                collect_files_from_fs(&path, &mut managed)?;
            } else {
                managed.push(path);
            }
        }
    }

    Ok(managed)
}

/// Removes empty directories under managed roots while preserving symlinks.
pub(crate) fn cleanup_empty_managed_dirs(home: &Path, platform: Platform) -> Result<()> {
    let platform_root = home.join(platform.root);

    for root in MANAGED_ROOTS {
        let managed_root = platform_root.join(root);
        if !managed_root.exists() {
            continue;
        }

        remove_empty_descendants(&managed_root)?;
    }

    Ok(())
}

fn collect_files_from_fs(path: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry.with_context(|| format!("failed reading entry in {}", path.display()))?;
        let entry_path = entry.path();

        let metadata = fs::symlink_metadata(&entry_path)
            .with_context(|| format!("failed to read metadata for {}", entry_path.display()))?;

        if metadata.file_type().is_symlink() {
            output.push(entry_path);
        } else if metadata.is_dir() {
            collect_files_from_fs(&entry_path, output)?;
        } else {
            output.push(entry_path);
        }
    }

    Ok(())
}

fn remove_empty_descendants(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let mut is_empty = true;

    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry.with_context(|| format!("failed reading entry in {}", path.display()))?;
        let entry_path = entry.path();

        let metadata = fs::symlink_metadata(&entry_path)
            .with_context(|| format!("failed to read metadata for {}", entry_path.display()))?;

        if metadata.file_type().is_symlink() {
            is_empty = false;
            continue;
        }

        if metadata.is_dir() {
            let child_empty = remove_empty_descendants(&entry_path)?;
            if child_empty {
                fs::remove_dir(&entry_path).with_context(|| {
                    format!("failed to remove empty dir {}", entry_path.display())
                })?;
            } else {
                is_empty = false;
            }
        } else {
            is_empty = false;
        }
    }

    Ok(is_empty)
}
