//! High-level command execution for install, update and debloat flows.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use crate::constants::{Command, Platform, HOME_SCOPE, PLATFORMS};
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

/// Returns all managed platform targets.
fn all_targets() -> Vec<Platform> {
    PLATFORMS.to_vec()
}

/// Performs first-time installation when no prior metadata is found.
fn install(home: &Path) -> Result<String> {
    if all_targets()
        .iter()
        .any(|platform| installed_version(home, platform).is_some())
    {
        return Ok("Configuration already installed. Run `update` instead.".to_string());
    }

    let version = env!("CARGO_PKG_VERSION");
    let mut written_files = 0usize;

    for platform in all_targets() {
        let desired = build_desired_files(home, platform)?;
        let manifest = build_manifest(version, platform, &desired)?;
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

    for platform in all_targets() {
        let existing_manifest = load_manifest(home, platform)?;
        let legacy_files = if platform.allow_legacy_cleanup {
            collect_legacy_managed_files(home, platform)?
        } else {
            Vec::new()
        };
        let desired = build_desired_files(home, platform)?;
        let manifest = build_manifest(current_version, platform, &desired)?;
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
        if platform.allow_legacy_cleanup {
            removed += cleanup_legacy_managed_entries(home, platform)?;
        }
        written += write_platform_state(home, platform, &desired, &manifest)?;
    }

    // Migration cleanup: previous versions tracked shared docs in HOME root.
    removed += cleanup_legacy_home_scope(home)?;

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

    for platform in all_targets() {
        if let Some(manifest) = load_manifest(home, platform)? {
            let metadata_relative = metadata_relative_path(platform);
            let metadata_relative_string = metadata_relative.to_string_lossy().to_string();

            let metadata_full_path = home.join(platform.root).join(&metadata_relative_string);

            let tracked_files: Vec<PathBuf> = collect_tracked_files(home, platform, &manifest)
                .into_iter()
                .filter(|path| path != &metadata_full_path)
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
        } else if platform.allow_legacy_cleanup {
            let legacy_files = collect_legacy_managed_files(home, platform)?;
            removed += remove_files(&legacy_files)?;
            removed += cleanup_legacy_managed_entries(home, platform)?;
            removed += remove_generated_meta_if_present(home, platform)?;
        }
    }

    removed += cleanup_legacy_home_scope(home)?;

    Ok(format!("Debloat completed ({removed} files removed)."))
}

fn build_desired_files(home: &Path, platform: Platform) -> Result<HashMap<PathBuf, Vec<u8>>> {
    let mut desired = desired_files_for_platform(platform)?;

    if platform.normalized_name() == "claude" {
        merge_desired_files(
            &mut desired,
            desired_external_skill_files_for_platform(home, platform)?,
        )?;
    }

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

fn build_manifest(
    version: &str,
    platform: Platform,
    desired: &HashMap<PathBuf, Vec<u8>>,
) -> Result<PlatformManifest> {
    let mut managed_files = desired.keys().cloned().collect::<BTreeSet<_>>();
    managed_files.insert(metadata_relative_path(platform));

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

    if platform.normalized_name() == "copilot" {
        written += sync_vscode_mcp_servers(home, desired)?;
    }

    let metadata = render_meta(manifest)?.into_bytes();
    let metadata_path = home.join(metadata_path(platform));
    written += write_file_if_changed(&metadata_path, &metadata)?;
    Ok(written)
}

/// Syncs managed Copilot MCP entries into VS Code's user MCP config.
///
/// VS Code reads servers from `~/.config/Code/User/mcp.json` under `servers`.
/// The embedded Copilot config stores them under `mcpServers`, so this method
/// merges managed entries into the VS Code file while preserving user-defined ones.
fn sync_vscode_mcp_servers(home: &Path, desired: &HashMap<PathBuf, Vec<u8>>) -> Result<usize> {
    let Some(copilot_mcp) = desired.get(&PathBuf::from("mcp.json")) else {
        return Ok(0);
    };

    let parsed_copilot: Value = serde_json::from_slice(copilot_mcp)
        .context("failed to parse embedded .copilot/mcp.json")?;

    let managed_servers = parsed_copilot
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    if managed_servers.is_empty() {
        return Ok(0);
    }

    let vscode_mcp_path = home.join(".config/Code/User/mcp.json");
    ensure_safe_destination(&vscode_mcp_path)?;

    let mut vscode_root = read_json_object_or_empty(&vscode_mcp_path)?;
    let existing_servers = vscode_root
        .remove("servers")
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();

    let mut merged_servers = existing_servers;
    for (name, server_config) in managed_servers {
        // Preserve user-owned server definitions if names collide.
        merged_servers.entry(name).or_insert(server_config);
    }

    vscode_root.insert("servers".to_string(), Value::Object(merged_servers));
    let mut rendered = serde_json::to_vec_pretty(&Value::Object(vscode_root))?;
    rendered.push(b'\n');

    write_file_if_changed(&vscode_mcp_path, &rendered)
}

/// Reads a JSON object from disk or returns an empty object when absent.
///
/// This helper is used for resilient MCP merge behavior:
/// - missing file means no prior user configuration, so merge starts empty
/// - invalid JSON surfaces as an error to avoid silently corrupting config
/// - non-object JSON is treated as empty because MCP root must be object-shaped
fn read_json_object_or_empty(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }

    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse {} as JSON", path.display()))?;

    Ok(parsed.as_object().cloned().unwrap_or_default())
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
    ensure_safe_destination(destination)?;

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

/// Rejects writes that would target symlinks or pass through symlinked directories.
fn ensure_safe_destination(destination: &Path) -> Result<()> {
    if destination.exists() {
        let metadata = fs::symlink_metadata(destination)
            .with_context(|| format!("failed to stat {}", destination.display()))?;
        if metadata.file_type().is_symlink() {
            anyhow::bail!(
                "refusing to write through symlink at {}",
                destination.display()
            );
        }
    }

    let mut current = destination.parent();
    while let Some(parent) = current {
        if parent.exists() {
            let metadata = fs::symlink_metadata(parent)
                .with_context(|| format!("failed to stat {}", parent.display()))?;
            if metadata.file_type().is_symlink() {
                anyhow::bail!(
                    "refusing to traverse symlinked directory {}",
                    parent.display()
                );
            }
        }
        current = parent.parent();
    }

    Ok(())
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

fn cleanup_legacy_home_scope(home: &Path) -> Result<usize> {
    let mut removed = 0usize;

    if let Some(home_manifest) = load_manifest(home, HOME_SCOPE)? {
        let tracked_files = collect_tracked_files(home, HOME_SCOPE, &home_manifest);
        removed += remove_files(&tracked_files)?;
        removed += cleanup_tracked_directories(&collect_tracked_directories(
            home,
            HOME_SCOPE,
            &home_manifest,
        ))?;

        let home_meta = home.join(metadata_path(HOME_SCOPE));
        if home_meta.exists() {
            fs::remove_file(&home_meta)
                .with_context(|| format!("failed to remove {}", home_meta.display()))?;
            removed += 1;
        }
    } else {
        removed += remove_generated_meta_if_present(home, HOME_SCOPE)?;
    }

    Ok(removed)
}
