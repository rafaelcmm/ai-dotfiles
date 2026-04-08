//! High-level command execution for install, update and debloat flows.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::constants::{Command, Platform, PLATFORMS};
use crate::embedded::desired_files_for_platform;
use crate::external_skills::desired_external_skill_files_for_platform;
use crate::fs_ops::{
    cleanup_legacy_managed_entries, cleanup_tracked_directories, collect_legacy_managed_files,
    collect_tracked_directories, collect_tracked_files,
};
use crate::meta::{
    installed_version, load_manifest, metadata_path, metadata_relative_path, render_meta,
    PlatformManifest,
};

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
        let desired = build_desired_files(home, platform)?;
        let manifest = build_manifest(version, &desired)?;
        written_files += write_platform_state(home, platform, &desired, &manifest)?;
    }

    Ok(format!(
        "Installed configuration version {version} ({written_files} files created)."
    ))
}

/// Updates managed files to the currently packaged version.
fn update(home: &Path) -> Result<String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let mut removed = 0usize;
    let mut written = 0usize;

    for platform in PLATFORMS {
        let existing_manifest = load_manifest(home, platform)?;
        let legacy_files = collect_legacy_managed_files(home, platform)?;
        let desired = build_desired_files(home, platform)?;
        let manifest = build_manifest(current_version, &desired)?;
        let desired_paths: HashSet<String> = manifest.managed_files.iter().cloned().collect();

        if let Some(existing_manifest) = existing_manifest.as_ref() {
            removed +=
                remove_stale_manifest_files(home, platform, existing_manifest, &desired_paths)?;
            removed += cleanup_tracked_directories(&collect_tracked_directories(
                home,
                platform,
                existing_manifest,
            ))?;
        }

        removed += remove_files(&legacy_files)?;
        removed += cleanup_legacy_managed_entries(home, platform)?;
        written += write_platform_state(home, platform, &desired, &manifest)?;
    }

    if written == 0 && removed == 0 {
        return Ok("Configuration is already up to date.".to_string());
    }

    Ok(format!(
        "Updated configuration to version {current_version} ({written} files written, {removed} files removed)."
    ))
}

/// Removes only managed files while preserving user-owned custom files.
fn debloat(home: &Path) -> Result<String> {
    let mut removed = 0usize;

    for platform in PLATFORMS {
        if let Some(manifest) = load_manifest(home, platform)? {
            let metadata_relative = metadata_relative_path();
            let metadata_relative_string = metadata_relative.to_string_lossy().to_string();

            let tracked_files: Vec<PathBuf> = collect_tracked_files(home, platform, &manifest)
                .into_iter()
                .filter(|path| path != &home.join(platform.root).join(&metadata_relative_string))
                .collect();
            removed += remove_files(&tracked_files)?;
            removed += cleanup_tracked_directories(&collect_tracked_directories(
                home, platform, &manifest,
            ))?;

            let meta_path = home.join(metadata_path(platform));
            if meta_path.exists() {
                fs::remove_file(&meta_path)
                    .with_context(|| format!("failed to remove {}", meta_path.display()))?;
                removed += 1;
            }
        } else {
            let legacy_files = collect_legacy_managed_files(home, platform)?;
            removed += remove_files(&legacy_files)?;
            removed += cleanup_legacy_managed_entries(home, platform)?;
            removed += remove_generated_meta_if_present(home, platform)?;
        }
    }

    Ok(format!("Debloat completed ({removed} files removed)."))
}

fn build_desired_files(home: &Path, platform: Platform) -> Result<HashMap<PathBuf, Vec<u8>>> {
    let mut desired = desired_files_for_platform(platform)?;
    merge_desired_files(
        &mut desired,
        desired_external_skill_files_for_platform(home, platform)?,
    )?;
    Ok(desired)
}

fn merge_desired_files(
    target: &mut HashMap<PathBuf, Vec<u8>>,
    source: HashMap<PathBuf, Vec<u8>>,
) -> Result<()> {
    for (path, bytes) in source {
        if target.insert(path.clone(), bytes).is_some() {
            anyhow::bail!("duplicate desired destination {}", path.display());
        }
    }

    Ok(())
}

fn build_manifest(version: &str, desired: &HashMap<PathBuf, Vec<u8>>) -> Result<PlatformManifest> {
    let mut managed_files = desired.keys().cloned().collect::<BTreeSet<_>>();
    managed_files.insert(metadata_relative_path());

    let managed_directories = collect_parent_directories(&managed_files);
    PlatformManifest::new(version.to_string(), managed_files, managed_directories)
}

fn collect_parent_directories(paths: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    let mut directories = BTreeSet::<PathBuf>::new();

    for path in paths {
        let mut current = path.parent();
        while let Some(parent) = current {
            if parent.as_os_str().is_empty() {
                break;
            }
            directories.insert(parent.to_path_buf());
            current = parent.parent();
        }
    }

    directories
}

fn remove_stale_manifest_files(
    home: &Path,
    platform: Platform,
    existing_manifest: &PlatformManifest,
    desired_paths: &HashSet<String>,
) -> Result<usize> {
    let mut removed = 0usize;

    for relative in &existing_manifest.managed_files {
        if desired_paths.contains(relative) {
            continue;
        }

        let path = home.join(platform.root).join(relative);
        if !path.exists() {
            continue;
        }

        fs::remove_file(&path)
            .with_context(|| format!("failed to remove stale managed file {}", path.display()))?;
        removed += 1;
    }

    Ok(removed)
}

fn remove_files(paths: &[PathBuf]) -> Result<usize> {
    let mut removed = 0usize;

    for path in paths {
        if !path.exists() {
            continue;
        }

        fs::remove_file(path)
            .with_context(|| format!("failed to remove managed file {}", path.display()))?;
        removed += 1;
    }

    Ok(removed)
}

fn write_platform_state(
    home: &Path,
    platform: Platform,
    desired: &HashMap<PathBuf, Vec<u8>>,
    manifest: &PlatformManifest,
) -> Result<usize> {
    let mut written = write_changed_files(home, platform, desired)?;
    let metadata = render_meta(manifest)?.into_bytes();
    let metadata_path = home.join(metadata_path(platform));
    written += write_file_if_changed(&metadata_path, &metadata)?;
    Ok(written)
}

fn write_changed_files(
    home: &Path,
    platform: Platform,
    desired: &HashMap<PathBuf, Vec<u8>>,
) -> Result<usize> {
    let mut written = 0usize;

    for (relative, bytes) in desired {
        let destination = home.join(platform.root).join(relative);
        written += write_file_if_changed(&destination, bytes)?;
    }

    Ok(written)
}

fn write_file_if_changed(destination: &Path, bytes: &[u8]) -> Result<usize> {
    let should_write = match fs::read(destination) {
        Ok(existing) => existing != bytes,
        Err(_) => true,
    };

    if !should_write {
        return Ok(0);
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create destination directory {}",
                parent.display()
            )
        })?;
    }

    fs::write(destination, bytes)
        .with_context(|| format!("failed to write {}", destination.display()))?;
    Ok(1)
}

fn remove_generated_meta_if_present(home: &Path, platform: Platform) -> Result<usize> {
    let meta_path = home.join(metadata_path(platform));
    if !meta_path.exists() {
        return Ok(0);
    }

    let meta = fs::read_to_string(&meta_path)
        .with_context(|| format!("failed to read {}", meta_path.display()))?;

    if !meta.contains("automatically generated") && !meta.contains("generated_by:") {
        return Ok(0);
    }

    fs::remove_file(&meta_path)
        .with_context(|| format!("failed to remove {}", meta_path.display()))?;
    Ok(1)
}
