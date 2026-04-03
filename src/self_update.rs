use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

const BIN_NAME: &str = "rafaelcmm-ai-dotfiles";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

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

    let (target, format, ext) = detect_target_and_format()?;
    let release_url = format!(
        "https://github.com/{repo_slug}/releases/download/v{latest_version}/{BIN_NAME}-{target}.{ext}"
    );

    let status = Command::new("cargo")
        .arg("binstall")
        .arg(BIN_NAME)
        .arg("--no-confirm")
        .arg("--pkg-url")
        .arg(&release_url)
        .arg("--pkg-fmt")
        .arg(format)
        .status()
        .context("failed to execute cargo-binstall")?;

    if !status.success() {
        anyhow::bail!("cargo-binstall failed while installing v{latest_version}");
    }

    let mut cmd = Command::new(BIN_NAME);
    cmd.arg("update")
        .arg("--no-self-update")
        .arg("--home")
        .arg(home);

    if allow_outside_home {
        cmd.arg("--allow-outside-home");
    }
    if assume_yes {
        cmd.arg("--yes");
    }

    let status = cmd.status().context("failed to run updated binary")?;
    if !status.success() {
        anyhow::bail!("updated binary failed to run dotfile update");
    }

    Ok(true)
}

fn fetch_latest_release_tag(repo_slug: &str) -> Result<String> {
    let url = format!("https://api.github.com/repos/{repo_slug}/releases/latest");

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("failed to build HTTP client")?;

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

fn parse_release_tag(tag: &str) -> Result<Version> {
    let version = tag.trim_start_matches('v');
    Version::parse(version).context("tag is not valid semantic version")
}

fn repository_slug_from_url(url: &str) -> Result<String> {
    if let Some(stripped) = url.strip_prefix("https://github.com/") {
        return Ok(stripped.trim_end_matches('/').to_string());
    }

    anyhow::bail!("unsupported repository URL format: {url}")
}

fn detect_target_and_format() -> Result<(&'static str, &'static str, &'static str)> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok(("x86_64-unknown-linux-gnu", "tgz", "tar.gz")),
        ("macos", "aarch64") => Ok(("aarch64-apple-darwin", "tgz", "tar.gz")),
        ("windows", "x86_64") => Ok(("x86_64-pc-windows-msvc", "zip", "zip")),
        (os, arch) => anyhow::bail!("unsupported platform for self-update: {os}/{arch}"),
    }
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N]: ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("failed to read confirmation input")?;

    Ok(matches!(input.trim(), "y" | "Y" | "yes" | "YES"))
}
