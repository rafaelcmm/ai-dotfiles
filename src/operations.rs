//! High-level command execution for install, update and debloat flows.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::constants::{Command, Platform, PLATFORMS};
use crate::embedded::desired_files_for_platform;
use crate::fs_ops::{cleanup_empty_managed_dirs, collect_existing_managed_files};
use crate::meta::installed_version;

/// Executes one command against the provided home directory.
pub fn run(command: Command, home: &Path) -> Result<String> {
    fs::create_dir_all(home)
        .with_context(|| format!("failed to create home directory at {}", home.display()))?;

    match command {
        Command::Install => install(home),
        Command::Update => update(home),
        Command::Debloat => debloat(home),
    }
}

/// Performs first-time installation when no prior metadata is found.
fn install(home: &Path) -> Result<String> {
    if PLATFORMS
        .iter()
        .any(|platform| installed_version(home, platform).is_some())
    {
        return Ok("Configuration already installed. Run `update` instead.".to_string());
    }

    let version = env!("CARGO_PKG_VERSION");
    let mut written_files = 0usize;

    for platform in PLATFORMS {
        let desired = desired_files_for_platform(platform, version)?;
        for (relative_path, contents) in desired {
            let destination = home.join(relative_path);
            if destination.exists() {
                continue;
            }

            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create destination directory {}", parent.display())
                })?;
            }

            fs::write(&destination, contents)
                .with_context(|| format!("failed to write {}", destination.display()))?;
            written_files += 1;
        }
    }

    Ok(format!("Installed configuration version {version} ({written_files} files created)."))
}

/// Updates managed files to the currently packaged version.
fn update(home: &Path) -> Result<String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let installed: Vec<String> = PLATFORMS
        .iter()
        .filter_map(|platform| installed_version(home, platform))
        .collect();

    if !installed.is_empty() && installed.iter().all(|version| version == current_version) {
        return Ok("Configuration is already up to date.".to_string());
    }

    let mut removed = 0usize;
    let mut written = 0usize;

    for platform in PLATFORMS {
        let desired = desired_files_for_platform(platform, current_version)?;
        let desired_paths: HashSet<PathBuf> = desired.keys().cloned().collect();
        let existing_managed = collect_existing_managed_files(home, platform)?;

        removed += remove_stale_managed_files(existing_managed, &desired_paths)?;
        cleanup_empty_managed_dirs(home, platform)?;
        written += write_changed_files(home, desired)?;
    }

    Ok(format!(
        "Updated configuration to version {current_version} ({written} files written, {removed} files removed)."
    ))
}

/// Removes only managed files while preserving user-owned custom files.
fn debloat(home: &Path) -> Result<String> {
    let mut removed = 0usize;

    for platform in PLATFORMS {
        let managed = collect_existing_managed_files(home, platform)?;
        for path in managed {
            if path.exists() {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to remove managed file {}", path.display()))?;
                removed += 1;
            }
        }

        removed += remove_generated_meta_if_present(home, platform)?;
        cleanup_empty_managed_dirs(home, platform)?;
    }

    Ok(format!("Debloat completed ({removed} files removed)."))
}

fn remove_stale_managed_files(
    existing_managed: Vec<PathBuf>,
    desired_paths: &HashSet<PathBuf>,
) -> Result<usize> {
    let mut removed = 0usize;

    for path in existing_managed {
        if desired_paths.contains(&path) || !path.exists() {
            continue;
        }

        fs::remove_file(&path)
            .with_context(|| format!("failed to remove stale managed file {}", path.display()))?;
        removed += 1;
    }

    Ok(removed)
}

fn write_changed_files(home: &Path, desired: std::collections::HashMap<PathBuf, Vec<u8>>) -> Result<usize> {
    let mut written = 0usize;

    for (relative, bytes) in desired {
        let destination = home.join(relative);

        let should_write = match fs::read(&destination) {
            Ok(existing) => existing != bytes,
            Err(_) => true,
        };

        if !should_write {
            continue;
        }

        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create destination directory {}", parent.display()))?;
        }

        fs::write(&destination, bytes)
            .with_context(|| format!("failed to write {}", destination.display()))?;
        written += 1;
    }

    Ok(written)
}

fn remove_generated_meta_if_present(home: &Path, platform: Platform) -> Result<usize> {
    let meta_path = home.join(platform.root).join("_meta.md");
    if !meta_path.exists() {
        return Ok(0);
    }

    let meta = fs::read_to_string(&meta_path)
        .with_context(|| format!("failed to read {}", meta_path.display()))?;

    if !meta.contains("automatically generated") {
        return Ok(0);
    }

    fs::remove_file(&meta_path).with_context(|| format!("failed to remove {}", meta_path.display()))?;
    Ok(1)
}
