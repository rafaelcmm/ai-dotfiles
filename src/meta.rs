//! Metadata rendering helpers for `_meta.md` generation and manifest parsing.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::constants::Platform;
use crate::embedded::static_dir;

const MANIFEST_VERSION: u32 = 1;
const GENERATED_MARKER: &str = "ai-dotfiles";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlatformManifest {
    pub(crate) version: String,
    pub(crate) managed_files: BTreeSet<String>,
    pub(crate) managed_directories: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct RawFrontmatter {
    version: String,
    #[serde(default)]
    manifest_version: Option<u32>,
    #[serde(default)]
    managed_files: Vec<String>,
    #[serde(default)]
    managed_directories: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RenderedFrontmatter {
    title: String,
    description: String,
    manifest_version: u32,
    version: String,
    author: String,
    repository: String,
    generated_by: String,
    managed_files: Vec<String>,
    managed_directories: Vec<String>,
}

impl PlatformManifest {
    pub(crate) fn new(
        version: impl Into<String>,
        managed_files: impl IntoIterator<Item = PathBuf>,
        managed_directories: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self> {
        let managed_files = managed_files
            .into_iter()
            .map(|path| normalize_relative_path(&path))
            .collect::<Result<BTreeSet<_>>>()?;
        let managed_directories = managed_directories
            .into_iter()
            .map(|path| normalize_relative_path(&path))
            .collect::<Result<BTreeSet<_>>>()?;

        Ok(Self {
            version: version.into(),
            managed_files,
            managed_directories,
        })
    }
}

/// Returns the platform-relative metadata path.
pub(crate) fn metadata_relative_path() -> PathBuf {
    PathBuf::from("_meta.md")
}

/// Returns the metadata path under the platform root.
pub(crate) fn metadata_path(platform: Platform) -> PathBuf {
    PathBuf::from(platform.root).join(metadata_relative_path())
}

/// Generates `_meta.md` from the manifest and the static Markdown body template.
pub(crate) fn render_meta(manifest: &PlatformManifest) -> Result<String> {
    let template_file = static_dir()
        .get_file("_meta_template.md")
        .expect("static/_meta_template.md not found");
    let template = String::from_utf8_lossy(template_file.contents()).to_string();

    let version = env!("CARGO_PKG_VERSION");
    let repository = env!("CARGO_PKG_REPOSITORY");
    let authors = env!("CARGO_PKG_AUTHORS");
    let (author_name, author_email) = parse_author(authors);
    let author = if author_email.is_empty() {
        author_name.to_string()
    } else {
        format!("{author_name} ({author_email})")
    };

    let frontmatter = RenderedFrontmatter {
        title: "AI Dotfiles - Metadata".to_string(),
        description: "Metadata for AI Dotfiles platform bootstrap state.".to_string(),
        manifest_version: MANIFEST_VERSION,
        version: manifest.version.clone(),
        author: author.clone(),
        repository: repository.to_string(),
        generated_by: GENERATED_MARKER.to_string(),
        managed_files: manifest.managed_files.iter().cloned().collect(),
        managed_directories: manifest.managed_directories.iter().cloned().collect(),
    };

    let yaml = serde_yaml::to_string(&frontmatter)
        .context("failed to serialize metadata frontmatter")?
        .trim_start_matches("---\n")
        .to_string();
    let body = template
        .replace("{{package.version}}", version)
        .replace("{{package.author.name}}", author_name)
        .replace("{{package.author.email}}", author_email)
        .replace("{{package.repository.url}}", repository);

    Ok(format!("---\n{yaml}---\n\n{}\n", body.trim_end()))
}

/// Loads the structured manifest from `<platform>/_meta.md`, when present.
pub(crate) fn load_manifest(home: &Path, platform: Platform) -> Result<Option<PlatformManifest>> {
    let meta_path = home.join(metadata_path(platform));
    if !meta_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&meta_path)
        .with_context(|| format!("failed to read {}", meta_path.display()))?;
    parse_manifest(&content)
}

/// Reads installed version from `<platform>/_meta.md`, supporting both legacy and current formats.
pub(crate) fn installed_version(home: &Path, platform: &Platform) -> Option<String> {
    let meta = home.join(platform.root).join("_meta.md");
    let content = fs::read_to_string(meta).ok()?;

    match parse_manifest(&content) {
        Ok(Some(manifest)) => Some(manifest.version),
        Ok(None) | Err(_) => parse_legacy_version(&content),
    }
}

fn parse_manifest(content: &str) -> Result<Option<PlatformManifest>> {
    let Some(frontmatter) = extract_frontmatter(content) else {
        return Ok(None);
    };

    let raw: RawFrontmatter =
        serde_yaml::from_str(&frontmatter).context("failed to parse metadata YAML frontmatter")?;

    if raw.manifest_version.is_none() {
        return Ok(None);
    }

    if raw.manifest_version != Some(MANIFEST_VERSION) {
        anyhow::bail!(
            "unsupported metadata manifest version: {:?}",
            raw.manifest_version
        );
    }

    let managed_files = raw
        .managed_files
        .into_iter()
        .map(|path| normalize_relative_string(&path))
        .collect::<Result<BTreeSet<_>>>()?;
    let managed_directories = raw
        .managed_directories
        .into_iter()
        .map(|path| normalize_relative_string(&path))
        .collect::<Result<BTreeSet<_>>>()?;

    Ok(Some(PlatformManifest {
        version: raw.version,
        managed_files,
        managed_directories,
    }))
}

fn extract_frontmatter(content: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }

    let mut yaml = String::new();
    for line in lines {
        if line.trim_end() == "---" {
            return Some(yaml);
        }
        yaml.push_str(line);
        yaml.push('\n');
    }

    None
}

fn parse_legacy_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("version:") {
            let version = value.trim();
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }

    None
}

fn normalize_relative_string(path: &str) -> Result<String> {
    normalize_relative_path(Path::new(path))
}

fn normalize_relative_path(path: &Path) -> Result<String> {
    let mut segments = Vec::<String>::new();

    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                let segment = segment.to_string_lossy();
                if segment.is_empty() {
                    anyhow::bail!("managed path segment cannot be empty");
                }
                segments.push(segment.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("managed path must remain relative: {}", path.display())
            }
        }
    }

    if segments.is_empty() {
        anyhow::bail!("managed path cannot be empty");
    }

    Ok(segments.join("/"))
}

fn parse_author(authors: &str) -> (&str, &str) {
    let first = authors.split(':').next().unwrap_or(authors).trim();
    if let Some(start) = first.find('<') {
        if let Some(end) = first.find('>') {
            let name = first[..start].trim();
            let email = first[start + 1..end].trim();
            return (name, email);
        }
    }

    (first, "")
}
