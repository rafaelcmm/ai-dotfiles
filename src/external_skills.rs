//! External skill manifest and resolver.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;

use crate::constants::{Platform, EXTERNAL_SKILLS_CACHE_DIR, EXTERNAL_SKILLS_MANIFEST};
use crate::embedded::static_dir;

#[derive(Debug, Deserialize)]
struct ExternalSkillsManifest {
    #[serde(default)]
    source: Vec<ExternalSkillSource>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ExternalSkillSource {
    id: String,
    repository: String,
    commit: String,
    path: String,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
    /// Optional SHA256 checksum of SKILL.md for integrity verification.
    /// Format: lowercase hex string (64 chars).
    #[serde(default)]
    checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubContentEntry {
    #[serde(rename = "type")]
    kind: String,
    path: String,
    download_url: Option<String>,
}

fn default_enabled() -> bool {
    true
}

/// Builds desired platform-relative files for external skills enabled for one platform.
///
/// If live resolution fails for a source, this falls back to already installed
/// files for that source on the same platform.
pub(crate) fn desired_external_skill_files_for_platform(
    home: &Path,
    platform: Platform,
) -> Result<HashMap<PathBuf, Vec<u8>>> {
    let manifest = load_manifest()?;
    if manifest.source.is_empty() {
        return Ok(HashMap::new());
    }

    let mut output = HashMap::<PathBuf, Vec<u8>>::new();
    for source in manifest.source {
        if !source.enabled || !source.matches_platform(platform.root) {
            continue;
        }

        let files = match source.resolve_files(home) {
            Ok(files) => files,
            Err(error) => {
                eprintln!(
                    "warning: failed to resolve external skill '{}' from {} at {}: {}",
                    source.id, source.repository, source.commit, error
                );

                let installed = read_installed_skill_files(
                    home,
                    platform,
                    &source.id,
                    source.checksum.as_deref(),
                )?;
                if installed.is_empty() {
                    continue;
                }

                eprintln!(
                    "warning: using installed fallback for external skill '{}' on {}",
                    source.id, platform.root
                );
                installed
            }
        };

        for (relative, bytes) in files {
            let destination = PathBuf::from("skills").join(&source.id).join(relative);
            if output.insert(destination.clone(), bytes).is_some() {
                anyhow::bail!(
                    "duplicate external skill destination {}",
                    destination.display()
                );
            }
        }
    }

    Ok(output)
}

/// Reads already-installed files for one external skill source.
///
/// This is used as a resilience fallback during update when live resolution
/// fails, keeping previously installed content stable for idempotent updates.
fn read_installed_skill_files(
    home: &Path,
    platform: Platform,
    source_id: &str,
    expected_checksum: Option<&str>,
) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let root = home.join(platform.root).join("skills").join(source_id);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut output = Vec::<(PathBuf, Vec<u8>)>::new();
    collect_installed_skill_files(&root, &root, &mut output, 0)?;

    if let Some(checksum) = expected_checksum {
        verify_skill_checksum(&output, checksum)?;
    }

    Ok(output)
}

/// Recursively collects regular files from an installed external skill directory.
///
/// Symlinks and other non-regular entries are ignored so fallback content is
/// limited to on-disk files the installer can safely read.
fn collect_installed_skill_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<(PathBuf, Vec<u8>)>,
    depth: usize,
) -> Result<()> {
    const MAX_DEPTH: usize = 10;
    const MAX_FILES: usize = 100;
    const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;

    if depth > MAX_DEPTH {
        anyhow::bail!(
            "installed skill at {} exceeded maximum directory depth of {}",
            root.display(),
            MAX_DEPTH
        );
    }

    if output.len() >= MAX_FILES {
        anyhow::bail!(
            "installed skill at {} exceeded maximum file count of {}",
            root.display(),
            MAX_FILES
        );
    }

    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed reading entry in {}", current.display()))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?;

        if metadata.is_dir() {
            collect_installed_skill_files(root, &path, output, depth + 1)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        if metadata.len() > MAX_FILE_SIZE_BYTES {
            anyhow::bail!(
                "installed skill file {} exceeds maximum size of {} bytes",
                path.display(),
                MAX_FILE_SIZE_BYTES
            );
        }

        let relative = path
            .strip_prefix(root)
            .with_context(|| {
                format!(
                    "failed to relativize installed skill file {}",
                    path.display()
                )
            })?
            .to_path_buf();

        if !is_safe_relative_path(&relative) {
            anyhow::bail!(
                "installed external skill '{}' contains unsafe path traversal: {}",
                root.display(),
                relative.display()
            );
        }

        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read installed skill file {}", path.display()))?;
        output.push((relative, bytes));

        if output.len() >= MAX_FILES {
            anyhow::bail!(
                "installed skill at {} exceeded maximum file count of {}",
                root.display(),
                MAX_FILES
            );
        }
    }

    Ok(())
}

fn load_manifest() -> Result<ExternalSkillsManifest> {
    let Some(file) = static_dir().get_file(EXTERNAL_SKILLS_MANIFEST) else {
        return Ok(ExternalSkillsManifest { source: Vec::new() });
    };

    let contents = std::str::from_utf8(file.contents())
        .context("external-skills manifest is not valid UTF-8")?;
    let manifest: ExternalSkillsManifest =
        toml::from_str(contents).context("failed to parse external-skills manifest")?;
    Ok(manifest)
}

impl ExternalSkillSource {
    fn matches_platform(&self, platform_root: &str) -> bool {
        if self.platforms.is_empty() {
            return true;
        }

        let normalized = platform_root.trim_start_matches('.');
        self.platforms.iter().any(|entry| {
            entry.eq_ignore_ascii_case("all") || entry.eq_ignore_ascii_case(normalized)
        })
    }

    fn resolve_files(&self, home: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>> {
        validate_source(self)?;

        let cache_root = home
            .join(EXTERNAL_SKILLS_CACHE_DIR)
            .join(format!("{}-{}", self.id, self.commit));

        // SECURITY: Atomic check - if read succeeds, cache was valid at read time
        if let Ok(cached) = read_cached_files(&cache_root) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        let downloaded = fetch_from_github(self)?;
        if downloaded.is_empty() {
            anyhow::bail!(
                "source {} at path '{}' did not produce any files",
                self.repository,
                self.path
            );
        }

        write_cache(&cache_root, &downloaded)?;
        Ok(downloaded)
    }
}

fn validate_source(source: &ExternalSkillSource) -> Result<()> {
    if source.id.trim().is_empty() {
        anyhow::bail!("source id cannot be empty")
    }
    if source.id.contains('/') {
        anyhow::bail!("source id '{}' cannot include '/'", source.id)
    }
    // SECURITY: Enforce full 40-char SHA-1 commit hash or 64-char SHA-256
    let commit = source.commit.trim();
    let is_valid_sha =
        (commit.len() == 40 || commit.len() == 64) && commit.chars().all(|c| c.is_ascii_hexdigit());

    if !is_valid_sha {
        anyhow::bail!(
            "source '{}' commit must be a full SHA-1 (40 chars) or SHA-256 (64 chars) hash, got: {}",
            source.id,
            source.commit
        );
    }
    if source.path.trim().is_empty() {
        anyhow::bail!("source '{}' path cannot be empty", source.id)
    }
    Ok(())
}

fn fetch_from_github(source: &ExternalSkillSource) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let (owner, repo) = parse_github_repo(&source.repository)?;

    let mut headers = HeaderMap::new();
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let auth_value = HeaderValue::from_str(&format!("Bearer {token}"))
            .context("GITHUB_TOKEN contains invalid header characters")?;
        headers.insert(AUTHORIZATION, auth_value);
    }

    let client = Client::builder()
        .user_agent(format!(
            "ai-dotfiles/{}",
            env!("CARGO_PKG_VERSION")
        ))
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .context("failed to build HTTP client")?;

    let mut output = Vec::<(PathBuf, Vec<u8>)>::new();
    fetch_directory_recursively(
        &client,
        &owner,
        &repo,
        source,
        source.path.trim_matches('/'),
        &mut output,
        0, // Initial depth
    )?;

    if !output
        .iter()
        .any(|(relative, _)| relative.as_path() == Path::new("SKILL.md"))
    {
        anyhow::bail!(
            "source '{}' is missing SKILL.md under {}",
            source.id,
            source.path
        );
    }

    if let Some(expected_checksum) = &source.checksum {
        verify_skill_checksum(&output, expected_checksum)?;
    }

    Ok(output)
}

fn fetch_directory_recursively(
    client: &Client,
    owner: &str,
    repo: &str,
    source: &ExternalSkillSource,
    path: &str,
    output: &mut Vec<(PathBuf, Vec<u8>)>,
    depth: usize,
) -> Result<()> {
    const MAX_DEPTH: usize = 10;
    const MAX_FILES: usize = 100;

    // SECURITY: Prevent resource exhaustion via deep/wide directory trees
    if depth > MAX_DEPTH {
        anyhow::bail!(
            "source '{}' exceeded maximum directory depth of {}",
            source.id,
            MAX_DEPTH
        );
    }
    if output.len() > MAX_FILES {
        anyhow::bail!(
            "source '{}' exceeded maximum file count of {}",
            source.id,
            MAX_FILES
        );
    }

    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={}",
        source.commit
    );

    let entries: Vec<GithubContentEntry> = client
        .get(url)
        .send()
        .with_context(|| {
            format!(
                "failed to list GitHub contents for {}/{}:{}",
                owner, repo, path
            )
        })?
        .error_for_status()
        .with_context(|| {
            format!(
                "GitHub returned a non-success status for {}/{}:{}",
                owner, repo, path
            )
        })?
        .json()
        .context("failed to parse GitHub contents response")?;

    let source_prefix = format!("{}/", source.path.trim_matches('/'));

    for entry in entries {
        match entry.kind.as_str() {
            "dir" => {
                fetch_directory_recursively(
                    client,
                    owner,
                    repo,
                    source,
                    &entry.path,
                    output,
                    depth + 1,
                )?;
            }
            "file" => {
                let Some(download_url) = entry.download_url else {
                    continue;
                };

                let bytes = client
                    .get(download_url)
                    .send()
                    .with_context(|| format!("failed to download {}", entry.path))?
                    .error_for_status()
                    .with_context(|| format!("download failed for {}", entry.path))?
                    .bytes()
                    .with_context(|| format!("failed reading bytes for {}", entry.path))?
                    .to_vec();

                let relative = if entry.path == source.path.trim_matches('/') {
                    PathBuf::from(
                        Path::new(&entry.path)
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    )
                } else if let Some(stripped) = entry.path.strip_prefix(&source_prefix) {
                    PathBuf::from(stripped)
                } else {
                    continue;
                };

                // SECURITY: Validate path does not contain traversal sequences
                if !is_safe_relative_path(&relative) {
                    anyhow::bail!(
                        "source '{}' contains unsafe path traversal: {}",
                        source.id,
                        relative.display()
                    );
                }

                output.push((relative, bytes));
            }
            _ => {}
        }
    }

    Ok(())
}

/// Validates that a relative path does not escape its parent via traversal.
///
/// Returns `false` if the path contains `..` components or would escape
/// when joined against a base directory.
fn is_safe_relative_path(path: &Path) -> bool {
    use std::path::Component;

    for component in path.components() {
        match component {
            Component::ParentDir => return false,
            Component::Prefix(_) | Component::RootDir => return false,
            Component::Normal(_) | Component::CurDir => {}
        }
    }

    true
}

/// Verifies the SHA256 checksum of SKILL.md matches the expected value.
///
/// # Errors
///
/// Returns an error if SKILL.md is missing or checksum does not match.
fn verify_skill_checksum(files: &[(PathBuf, Vec<u8>)], expected: &str) -> Result<()> {
    use sha2::{Digest, Sha256};

    let expected = expected.trim().to_lowercase();
    if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("invalid checksum format: expected 64-char hex string");
    }

    let skill_md = files
        .iter()
        .find(|(path, _)| path.as_path() == Path::new("SKILL.md"))
        .ok_or_else(|| anyhow::anyhow!("SKILL.md not found for checksum verification"))?;

    let mut hasher = Sha256::new();
    hasher.update(&skill_md.1);
    let actual = hex::encode(hasher.finalize());

    if actual != expected {
        anyhow::bail!(
            "checksum mismatch for SKILL.md: expected {}, got {}",
            expected,
            actual
        );
    }

    Ok(())
}

fn parse_github_repo(repository: &str) -> Result<(String, String)> {
    let trimmed = repository.trim_end_matches('/');
    let marker = "github.com/";
    let Some(position) = trimmed.find(marker) else {
        anyhow::bail!(
            "unsupported repository URL '{}': expected github.com",
            repository
        );
    };

    let suffix = &trimmed[position + marker.len()..];
    let parts: Vec<&str> = suffix.split('/').collect();
    if parts.len() < 2 {
        anyhow::bail!(
            "unsupported repository URL '{}': expected owner/repo",
            repository
        );
    }

    let owner = parts[0].to_string();
    let repo = parts[1].trim_end_matches(".git").to_string();
    if owner.is_empty() || repo.is_empty() {
        anyhow::bail!(
            "unsupported repository URL '{}': expected owner/repo",
            repository
        );
    }

    Ok((owner, repo))
}

fn write_cache(cache_root: &Path, files: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    if cache_root.exists() {
        fs::remove_dir_all(cache_root)
            .with_context(|| format!("failed to remove stale cache {}", cache_root.display()))?;
    }

    fs::create_dir_all(cache_root)
        .with_context(|| format!("failed to create cache root {}", cache_root.display()))?;

    for (relative, bytes) in files {
        let destination = cache_root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create cache dir {}", parent.display()))?;
        }

        fs::write(&destination, bytes)
            .with_context(|| format!("failed to write cache file {}", destination.display()))?;
    }

    fs::write(cache_root.join(".complete"), "ok")
        .with_context(|| format!("failed to finalize cache at {}", cache_root.display()))?;

    Ok(())
}

fn read_cached_files(cache_root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    // SECURITY: Check .complete marker exists before reading
    let complete_marker = cache_root.join(".complete");
    if !complete_marker.exists() {
        anyhow::bail!("cache incomplete: missing .complete marker");
    }

    let mut output = Vec::<(PathBuf, Vec<u8>)>::new();
    collect_cached_files(cache_root, cache_root, &mut output)?;

    if output.is_empty() {
        anyhow::bail!("cache corrupt: no files found despite .complete marker");
    }

    Ok(output)
}

#[cfg(test)]
pub(crate) fn seed_test_external_skill_cache(home: &Path) -> Result<()> {
    let manifest = load_manifest()?;

    for source in manifest.source {
        let cache_root = home
            .join(EXTERNAL_SKILLS_CACHE_DIR)
            .join(format!("{}-{}", source.id, source.commit));

        write_cache(
            &cache_root,
            &[(
                PathBuf::from("SKILL.md"),
                b"offline test external skill\n".to_vec(),
            )],
        )?;
    }

    Ok(())
}

fn collect_cached_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<(PathBuf, Vec<u8>)>,
) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed reading entry in {}", current.display()))?;
        let path = entry.path();

        if path
            .file_name()
            .is_some_and(|name| name == std::ffi::OsStr::new(".complete"))
        {
            continue;
        }

        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?;

        if metadata.is_dir() {
            collect_cached_files(root, &path, output)?;
            continue;
        }

        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("failed to relativize cache file {}", path.display()))?
            .to_path_buf();
        output.push((relative, bytes));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parse_github_repo_supports_http_and_git_suffix() {
        let (owner, repo) = parse_github_repo("https://github.com/vercel-labs/agent-skills.git")
            .expect("valid repo should parse");

        assert_eq!(owner, "vercel-labs");
        assert_eq!(repo, "agent-skills");
    }

    #[test]
    fn platform_filter_accepts_all_and_specific() {
        let source = ExternalSkillSource {
            id: "x".to_string(),
            repository: "https://github.com/o/r".to_string(),
            commit: "73140fc5b3a214ad3222bcf557b397b3c02d11c1".to_string(),
            path: "skills/x".to_string(),
            platforms: vec!["copilot".to_string(), "all".to_string()],
            enabled: true,
            checksum: None,
        };

        assert!(source.matches_platform(".claude"));
        assert!(source.matches_platform(".copilot"));
        assert!(source.matches_platform(".cursor"));
    }

    #[test]
    fn path_traversal_blocked() {
        use std::path::PathBuf;

        assert!(!is_safe_relative_path(&PathBuf::from("../etc/passwd")));
        assert!(!is_safe_relative_path(&PathBuf::from("foo/../../bar")));
        assert!(!is_safe_relative_path(&PathBuf::from("/absolute/path")));
        assert!(is_safe_relative_path(&PathBuf::from("foo/bar.md")));
        assert!(is_safe_relative_path(&PathBuf::from("SKILL.md")));
    }

    #[test]
    fn checksum_verification_validates_skill_md() {
        use std::path::PathBuf;

        let files = vec![(PathBuf::from("SKILL.md"), b"test content".to_vec())];

        // Correct checksum for "test content"
        let checksum = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";
        assert!(verify_skill_checksum(&files, checksum).is_ok());

        // Wrong checksum should fail
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(verify_skill_checksum(&files, wrong).is_err());

        // Invalid format should fail
        assert!(verify_skill_checksum(&files, "not-a-checksum").is_err());
    }

    #[test]
    fn commit_validation_requires_full_sha() {
        let valid_sha1 = ExternalSkillSource {
            id: "test".to_string(),
            repository: "https://github.com/owner/repo".to_string(),
            commit: "73140fc5b3a214ad3222bcf557b397b3c02d11c1".to_string(),
            path: "skills/test".to_string(),
            platforms: vec![],
            enabled: true,
            checksum: None,
        };
        assert!(validate_source(&valid_sha1).is_ok());

        let invalid_short = ExternalSkillSource {
            commit: "abc1234".to_string(),
            ..valid_sha1.clone()
        };
        assert!(validate_source(&invalid_short).is_err());

        let invalid_branch = ExternalSkillSource {
            commit: "main".to_string(),
            ..valid_sha1.clone()
        };
        assert!(validate_source(&invalid_branch).is_err());
    }

    #[test]
    fn non_local_manifest_sources_require_checksums() {
        let manifest = load_manifest().expect("embedded external skills manifest should parse");

        let non_local_sources: Vec<_> = manifest
            .source
            .iter()
            .filter(|source| {
                let repo = source.repository.as_str();
                !(repo.starts_with("file://")
                    || repo.starts_with("./")
                    || repo.starts_with("../")
                    || repo.starts_with('/'))
            })
            .collect();

        assert_eq!(
            non_local_sources.len(),
            manifest.source.len(),
            "all manifest sources are currently non-local and should be checksum-protected"
        );

        for source in non_local_sources {
            let checksum = source
                .checksum
                .as_ref()
                .expect("non-local source must include checksum");
            assert_eq!(
                checksum.len(),
                64,
                "checksum for {} must be 64-char hex",
                source.id
            );
            assert!(
                checksum.chars().all(|ch| ch.is_ascii_hexdigit()),
                "checksum for {} must be hex",
                source.id
            );
        }
    }

    #[test]
    #[ignore = "network integration test"]
    fn network_integration_fetch_from_github_downloads_skill_files() {
        let manifest = load_manifest().expect("embedded external skills manifest should parse");
        let source = manifest
            .source
            .into_iter()
            .find(|source| source.enabled)
            .expect("manifest should include at least one enabled external source");

        let files = fetch_from_github(&source).expect("GitHub fetch should succeed");

        assert!(
            files
                .iter()
                .any(|(path, _)| path.as_path() == Path::new("SKILL.md")),
            "fetched source should contain SKILL.md"
        );
    }

    #[test]
    fn seed_test_external_skill_cache_populates_complete_entries() {
        let home = tempdir().expect("tempdir should be created");

        seed_test_external_skill_cache(home.path()).expect("test cache seeding should succeed");

        let manifest = load_manifest().expect("embedded external skills manifest should parse");
        for source in manifest.source {
            let cache_root = home
                .path()
                .join(EXTERNAL_SKILLS_CACHE_DIR)
                .join(format!("{}-{}", source.id, source.commit));

            let cached = read_cached_files(&cache_root).expect("seeded cache should be readable");
            assert_eq!(
                cached.len(),
                1,
                "seeded cache should contain one placeholder file"
            );
            assert_eq!(cached[0].0, PathBuf::from("SKILL.md"));
        }
    }
}
