//! Self-update flow for the `update` command.
//!
//! When a newer release is available on GitHub, this module downloads the
//! prebuilt release binary and re-execs `ai-dotfiles update`
//! with `--no-self-update` to avoid recursion.
//!
//! # Security
//!
//! Before downloading the release binary, this module verifies that the
//! expected platform artifact is listed in the release checksums file.

use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use semver::Version;
use serde::Deserialize;

const BIN_NAME: &str = "ai-dotfiles";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Attempts to self-update before running the dotfile update operation.
///
/// Returns `Ok(true)` when a new binary was installed and control was handed
/// off to the updated binary process. Returns `Ok(false)` when no self-update
/// occurred (no newer version, user declined, or update checks failed safely).
///
/// This function is intentionally fail-open: network/install failures emit a
/// warning and allow the normal update flow to continue with the current
/// binary.
///
/// # Errors
///
/// Returns an error only for local, deterministic failures such as invalid
/// build metadata, unsupported platform mapping, or failed re-exec of the
/// updated binary after a successful install.
///
/// # Panics
///
/// Does not panic.
pub fn maybe_self_update_and_reexec(
    home: &Path,
    allow_outside_home: bool,
    assume_yes: bool,
) -> Result<bool> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
        .context("failed to parse current package version")?;

    let repo_slug = repository_slug_from_url(env!("CARGO_PKG_REPOSITORY"))
        .context("failed to parse repository slug from Cargo.toml repository")?;

    let latest_tag = match fetch_latest_release_tag(&repo_slug) {
        Ok(tag) => tag,
        Err(err) => {
            eprintln!(
                "[update] warning: could not check latest release ({err}). Proceeding with local update."
            );
            return Ok(false);
        }
    };

    let latest_version = match parse_release_tag(&latest_tag) {
        Ok(version) => version,
        Err(err) => {
            eprintln!(
                "[update] warning: invalid release tag format '{latest_tag}' ({err}). Proceeding with local update."
            );
            return Ok(false);
        }
    };

    if latest_version <= current_version {
        return Ok(false);
    }

    println!("A newer version is available: v{latest_version} (current: v{current_version}).");

    if !assume_yes && !confirm("Install latest release before updating dotfiles?")? {
        println!("Proceeding with current binary version.");
        return Ok(false);
    }

    let (target, ext) = detect_target_and_ext()?;
    let asset_name = format!("{BIN_NAME}-{target}.{ext}");
    let release_url =
        format!("https://github.com/{repo_slug}/releases/download/v{latest_version}/{asset_name}");
    ensure_release_has_checksum_entry(&repo_slug, &latest_version, &asset_name)?;

    if let Err(err) = download_and_install_binary(&release_url, &asset_name, ext) {
        eprintln!(
            "[update] warning: could not install v{latest_version} ({err}). Proceeding with current binary."
        );
        return Ok(false);
    }

    let mut cmd = Command::new(BIN_NAME);
    cmd.arg("--home").arg(home);

    if allow_outside_home {
        cmd.arg("--allow-outside-home");
    }

    cmd.arg("update").arg("--no-self-update");

    if assume_yes {
        cmd.arg("--yes");
    }

    let status = cmd.status().context("failed to run updated binary")?;
    if !status.success() {
        anyhow::bail!("updated binary failed to run dotfile update");
    }

    Ok(true)
}

/// Ensures release checksums include an entry for the expected asset.
///
/// # Errors
///
/// Returns an error if checksum metadata cannot be downloaded or if the asset
/// entry is missing from the release checksums file.
fn ensure_release_has_checksum_entry(
    repo_slug: &str,
    version: &Version,
    asset_name: &str,
) -> Result<()> {
    let checksums_url =
        format!("https://github.com/{repo_slug}/releases/download/v{version}/SHA256SUMS");

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("failed to build HTTP client")?;

    let checksums = client
        .get(checksums_url)
        .header("User-Agent", BIN_NAME)
        .send()
        .context("failed to download release checksums")?
        .error_for_status()
        .context("release checksums endpoint returned error status")?
        .text()
        .context("failed to decode release checksums payload")?;

    let has_asset_entry = checksums.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.ends_with(asset_name) || trimmed.ends_with(&format!("./{asset_name}"))
    });

    if !has_asset_entry {
        anyhow::bail!("release checksums do not contain expected asset entry: {asset_name}");
    }

    Ok(())
}

/// Fetches the latest release tag (`vX.Y.Z`) from GitHub Releases API.
///
/// # Errors
///
/// Returns an error when HTTP requests fail, return non-success status, or
/// JSON payload decoding fails.
fn fetch_latest_release_tag(repo_slug: &str) -> Result<String> {
    let url = format!("https://api.github.com/repos/{repo_slug}/releases/latest");

    let client = build_github_client(Duration::from_secs(10))?;

    let release: GitHubRelease = client
        .get(url)
        .header("User-Agent", BIN_NAME)
        .send()
        .context("failed to call GitHub releases API")?
        .error_for_status()
        .context("GitHub releases API returned error status")?
        .json()
        .context("failed to decode GitHub release payload")?;

    Ok(release.tag_name)
}

/// Builds an HTTP client for GitHub requests.
///
/// If `GITHUB_TOKEN` is set, requests are authenticated to increase API rate
/// limits for release checks.
fn build_github_client(timeout: Duration) -> Result<reqwest::blocking::Client> {
    let mut headers = HeaderMap::new();
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let auth_value = HeaderValue::from_str(&format!("Bearer {token}"))
            .context("GITHUB_TOKEN contains invalid header characters")?;
        headers.insert(AUTHORIZATION, auth_value);
    }

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .timeout(timeout)
        .build()
        .context("failed to build HTTP client")
}

/// Parses a release tag into semantic version.
///
/// Accepts either `vX.Y.Z` or `X.Y.Z`.
///
/// # Errors
///
/// Returns an error when the tag is not a valid semantic version.
fn parse_release_tag(tag: &str) -> Result<Version> {
    let version = tag.trim_start_matches('v');
    Version::parse(version).context("tag is not valid semantic version")
}

/// Extracts `owner/repo` from a GitHub HTTPS repository URL.
///
/// # Errors
///
/// Returns an error for unsupported URL formats.
fn repository_slug_from_url(url: &str) -> Result<String> {
    if let Some(stripped) = url.strip_prefix("https://github.com/") {
        return Ok(stripped.trim_end_matches('/').to_string());
    }

    anyhow::bail!("unsupported repository URL format: {url}")
}

/// Resolves the current host target triple and release archive extension.
///
/// Returns `(target_triple, archive_extension)`.
///
/// # Errors
///
/// Returns an error for unsupported OS/architecture combinations.
fn detect_target_and_ext() -> Result<(&'static str, &'static str)> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok(("x86_64-unknown-linux-gnu", "tar.gz")),
        ("macos", "aarch64") => Ok(("aarch64-apple-darwin", "tar.gz")),
        ("windows", "x86_64") => Ok(("x86_64-pc-windows-msvc", "zip")),
        (os, arch) => anyhow::bail!("unsupported platform for self-update: {os}/{arch}"),
    }
}

/// Downloads the release archive and installs the binary by replacing the current executable.
///
/// Uses `tar` on Unix and PowerShell `Expand-Archive` on Windows for extraction.
/// The replacement is staged alongside the current executable to keep the rename
/// on the same filesystem and avoid cross-device move errors.
///
/// # Errors
///
/// Returns an error if the download, extraction, staging, or replacement fails.
fn download_and_install_binary(
    release_url: &str,
    asset_name: &str,
    archive_ext: &str,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .context("failed to build HTTP client")?;

    println!("Downloading {asset_name}...");

    let archive_bytes = client
        .get(release_url)
        .header("User-Agent", BIN_NAME)
        .send()
        .context("failed to download release archive")?
        .error_for_status()
        .context("release download returned error status")?
        .bytes()
        .context("failed to read archive bytes")?;

    let tmp_dir = std::env::temp_dir().join(format!("ai-dotfiles-update-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).context("failed to create temp directory")?;

    let result = install_from_archive(&archive_bytes, asset_name, archive_ext, &tmp_dir);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    result
}

fn install_from_archive(
    archive_bytes: &[u8],
    asset_name: &str,
    archive_ext: &str,
    tmp_dir: &Path,
) -> Result<()> {
    let archive_path = tmp_dir.join(asset_name);
    std::fs::write(&archive_path, archive_bytes).context("failed to write archive")?;

    let bin_file = if cfg!(windows) {
        format!("{BIN_NAME}.exe")
    } else {
        BIN_NAME.to_string()
    };

    let extract_status = if archive_ext == "zip" {
        // zip archives are only produced for Windows targets; use PowerShell.
        Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                    archive_path.display(),
                    tmp_dir.display()
                ),
            ])
            .status()
            .context("failed to run Expand-Archive")?
    } else {
        Command::new("tar")
            .arg("-xzf")
            .arg(&archive_path)
            .arg("-C")
            .arg(tmp_dir)
            .arg(&bin_file)
            .status()
            .context("failed to run tar")?
    };

    anyhow::ensure!(extract_status.success(), "archive extraction failed");

    let extracted = tmp_dir.join(&bin_file);
    let current_exe = std::env::current_exe().context("failed to resolve current executable")?;

    // Stage alongside the current executable so the rename stays on the same filesystem.
    let name = current_exe
        .file_name()
        .context("current executable has no filename")?
        .to_str()
        .context("current executable filename is not valid UTF-8")?;
    let staged = current_exe.with_file_name(format!("{name}.new"));

    std::fs::copy(&extracted, &staged).context("failed to stage new binary")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))
            .context("failed to set permissions on staged binary")?;
    }

    std::fs::rename(&staged, &current_exe).context("failed to replace current binary")?;

    Ok(())
}

/// Prompts for a yes/no confirmation with a conservative default.
///
/// Empty input is treated as "no".
///
/// # Errors
///
/// Returns an error when stdin/stdout interaction fails.
fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N]: ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read confirmation input")?;

    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES"))
}
